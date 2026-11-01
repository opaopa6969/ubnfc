//! LL(1) の排他選択を持つ閉領域の認識表。意味論・既存 analysis は変更しない。
use super::{charset::CharSet, children, Context};
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};

fn token_ok(spec: &TokenSpec) -> bool {
    matches!(
        spec,
        TokenSpec::CharRange { .. }
            | TokenSpec::Negation { .. }
            | TokenSpec::Any {}
            | TokenSpec::Eof {}
            | TokenSpec::Builtin {
                which: BuiltinToken::Digit
                    | BuiltinToken::Alphabet
                    | BuiltinToken::AlphabetUnderScore
                    | BuiltinToken::AlphabetNumericUnderScore
                    | BuiltinToken::Space
                    | BuiltinToken::CodePointSet
                    | BuiltinToken::EndOfSource,
                ..
            }
    )
}

fn rule_ok(cx: &Context<'_>, r: usize) -> bool {
    let rule = &cx.sem.rules[r];
    rule.operator.is_none()
        && !cx
            .sem
            .normalizations
            .iter()
            .any(|n| n.rule_id == rule.symbol_id)
        && !cx
            .sem
            .scope
            .effects
            .iter()
            .any(|e| e.rule_id == rule.symbol_id)
        && !cx.sem.recovery.iter().any(|e| e.rule_id == rule.symbol_id)
}

fn local_ok(cx: &Context<'_>, i: usize) -> bool {
    let e = &cx.sem.expressions[i];
    // mapping / capture は認識専用 session では値を生成しない。scope / recovery と
    // trivia は認識にも作用するので、その所有規則・境界を領域から除く。
    if !rule_ok(cx, cx.rules[&e.rule_id])
        || e.execution.child_mode != ExecutionChildMode::Inherit
        || cx
            .sem
            .trivia
            .insertion_plan
            .iter()
            .any(|p| p.boundary.owner_expr_id == e.id)
        || cx
            .sem
            .recovery
            .iter()
            .any(|r| r.application_sites.contains(&e.id))
    {
        return false;
    }
    match &e.node {
        ExprNode::Seq { .. } | ExprNode::Group { .. } | ExprNode::Eof {} | ExprNode::Any {} => true,
        ExprNode::Literal { case_sensitive, .. } => *case_sensitive,
        ExprNode::TokenRef { symbol_id } => token_ok(&cx.sem.tokens[cx.tokens[symbol_id]].spec),
        ExprNode::RuleRef { symbol_id } => rule_ok(cx, cx.rules[symbol_id]),
        ExprNode::Optional { .. } => true,
        ExprNode::Repeat {
            min: 0,
            max: Some(0),
            ..
        } => true,
        ExprNode::Repeat {
            child,
            terminator,
            max,
            ..
        } => {
            terminator.is_none()
                && *max != Some(0)
                && !cx.fact(child).nullable()
                && cx.fact(child).minimum() > 0
        }
        ExprNode::Choice {
            alternatives,
            selection,
            ..
        } => {
            if *selection == ChoiceSelection::Longest {
                return false;
            }
            let mut seen = CharSet::default();
            for a in alternatives {
                let fact = cx.fact(a);
                if fact.nullable() || fact.eof || !seen.disjoint(&fact.chars) {
                    return false;
                }
                seen.union(&fact.chars);
            }
            true
        }
        _ => false,
    }
}

pub(super) fn plans(cx: &Context<'_>) -> Vec<Ll1State> {
    // 最大不動点。右再帰は許し、拒否節点を含む閉包だけを落とす。
    let mut ok: Vec<_> = (0..cx.sem.expressions.len())
        .map(|i| local_ok(cx, i))
        .collect();
    loop {
        let mut changed = false;
        for i in 0..ok.len() {
            let zero = matches!(
                cx.sem.expressions[i].node,
                ExprNode::Repeat {
                    min: 0,
                    max: Some(0),
                    ..
                }
            );
            if ok[i] && !zero && cx.deps[i].iter().any(|&j| !ok[j]) {
                ok[i] = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut parents = BTreeMap::<usize, Vec<usize>>::new();
    for (i, e) in cx.sem.expressions.iter().enumerate() {
        for child in children(&e.node) {
            parents.entry(cx.index(child)).or_default().push(i);
        }
    }
    let bodies: BTreeSet<_> = cx.sem.rules.iter().map(|r| cx.index(&r.body)).collect();
    let indices: BTreeMap<_, _> = ok
        .iter()
        .enumerate()
        .filter(|(i, yes)| **yes && cx.active.contains(i))
        .enumerate()
        .map(|(index, (i, _))| (i, index as u64))
        .collect();
    let target = |id: &ExprId| indices[&cx.index(id)];
    indices
        .keys()
        .map(|&i| {
            let e = &cx.sem.expressions[i];
            let operation = match &e.node {
                ExprNode::Seq { items } => Ll1Operation::Sequence {
                    targets: items.iter().map(target).collect(),
                },
                ExprNode::Group { child } => Ll1Operation::Sequence {
                    targets: vec![target(child)],
                },
                ExprNode::Choice { alternatives, .. } => Ll1Operation::Choice {
                    transitions: alternatives
                        .iter()
                        .map(|a| Ll1Transition {
                            ranges: cx.fact(a).chars.ranges(),
                            target: target(a),
                        })
                        .collect(),
                },
                ExprNode::Optional { child } => Ll1Operation::Optional {
                    target: target(child),
                },
                ExprNode::Repeat {
                    min: 0,
                    max: Some(0),
                    ..
                } => Ll1Operation::Empty {},
                ExprNode::Repeat {
                    child, min, max, ..
                } => Ll1Operation::Repeat {
                    target: target(child),
                    min: *min,
                    max: *max,
                },
                ExprNode::RuleRef { symbol_id } => Ll1Operation::Call {
                    target: indices[&cx.body(symbol_id)],
                },
                ExprNode::Literal {
                    value,
                    word_boundary,
                    ..
                } => Ll1Operation::Literal {
                    value: value.clone(),
                    word_boundary: *word_boundary,
                },
                ExprNode::Eof {} => Ll1Operation::Eof {},
                ExprNode::Empty {} => Ll1Operation::Empty {},
                ExprNode::Any {} => Ll1Operation::Class {
                    ranges: CharSet::all().ranges(),
                },
                ExprNode::TokenRef { symbol_id } => {
                    match cx.sem.tokens[cx.tokens[symbol_id]].spec {
                        TokenSpec::Eof {}
                        | TokenSpec::Builtin {
                            which: BuiltinToken::EndOfSource,
                            ..
                        } => Ll1Operation::Eof {},
                        TokenSpec::Empty {} => Ll1Operation::Empty {},
                        _ => Ll1Operation::Class {
                            ranges: cx.facts[i].chars.ranges(),
                        },
                    }
                }
                _ => unreachable!("局所条件で除外済み"),
            };
            Ll1State {
                expr_id: e.id.clone(),
                root: bodies.contains(&i)
                    || parents.get(&i).is_none_or(|ps| ps.iter().any(|&p| !ok[p])),
                operation,
            }
        })
        .collect()
}
