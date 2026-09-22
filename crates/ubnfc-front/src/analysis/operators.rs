use super::{planning::unsafe_token, AnalysisError, Context};
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
struct Tier {
    rule: usize,
    base: usize,
    op: usize,
    left: usize,
    right: usize,
    assoc: OperatorAssociativity,
}

fn ungroup(cx: &Context<'_>, mut i: usize) -> usize {
    while let ExprNode::Group { child } = &cx.sem.expressions[i].node {
        i = cx.index(child);
    }
    i
}
fn reference(cx: &Context<'_>, i: usize) -> Option<usize> {
    if let ExprNode::RuleRef { symbol_id } = &cx.sem.expressions[ungroup(cx, i)].node {
        Some(cx.rules[symbol_id])
    } else {
        None
    }
}
fn sequence(cx: &Context<'_>, i: usize) -> Option<Vec<usize>> {
    if let ExprNode::Seq { items } = &cx.sem.expressions[ungroup(cx, i)].node {
        Some(items.iter().map(|id| ungroup(cx, cx.index(id))).collect())
    } else {
        None
    }
}
fn canonical(cx: &Context<'_>, r: usize) -> Option<Tier> {
    let root = ungroup(cx, cx.index(&cx.sem.rules[r].body));
    if let Some(seq) = sequence(cx, root) {
        if seq.len() == 2 {
            let base = reference(cx, seq[0])?;
            if let ExprNode::Repeat {
                child,
                min: 0,
                max: None,
                terminator: None,
            } = &cx.sem.expressions[seq[1]].node
            {
                let pair = sequence(cx, cx.index(child))?;
                if pair.len() == 2 && reference(cx, pair[1]) == Some(base) {
                    return Some(Tier {
                        rule: r,
                        base,
                        op: pair[0],
                        left: seq[0],
                        right: pair[1],
                        assoc: OperatorAssociativity::Left,
                    });
                }
            }
        }
    }
    if let ExprNode::Choice {
        alternatives,
        selection: ChoiceSelection::Ordered,
        ..
    } = &cx.sem.expressions[root].node
    {
        if alternatives.len() == 2 {
            let seq = sequence(cx, cx.index(&alternatives[0]))?;
            if seq.len() == 3 {
                let base = reference(cx, seq[0])?;
                if reference(cx, cx.index(&alternatives[1])) == Some(base)
                    && reference(cx, seq[2]) == Some(r)
                {
                    return Some(Tier {
                        rule: r,
                        base,
                        op: seq[1],
                        left: seq[0],
                        right: seq[2],
                        assoc: OperatorAssociativity::Right,
                    });
                }
            }
        }
    }
    None
}

fn words(cx: &Context<'_>, i: usize, seen: &mut BTreeSet<usize>) -> Option<Vec<String>> {
    if !seen.insert(i) {
        return None;
    }
    let result = match &cx.sem.expressions[i].node {
        ExprNode::Literal {
            value,
            case_sensitive: true,
            ..
        } if !value.is_empty() => Some(vec![value.clone()]),
        ExprNode::RuleRef { symbol_id } => words(cx, cx.body(symbol_id), seen),
        ExprNode::Group { child } => words(cx, cx.index(child), seen),
        ExprNode::Choice {
            alternatives,
            selection: ChoiceSelection::Ordered,
            ..
        } => {
            let mut all = Vec::new();
            let mut valid = true;
            for id in alternatives {
                if let Some(w) = words(cx, cx.index(id), seen) {
                    all.extend(w);
                } else {
                    valid = false;
                }
            }
            valid.then_some(all)
        }
        _ => None,
    };
    seen.remove(&i);
    result
}

pub(super) fn plans(cx: &Context<'_>, errors: &mut Vec<AnalysisError>) -> Vec<OperatorPlan> {
    let mut tiers = BTreeMap::new();
    let mut noncanonical = Vec::new();
    for (r, rule) in cx.sem.rules.iter().enumerate() {
        let annotated = rule.annotations.iter().any(|a| {
            matches!(
                a.name.trim_start_matches('@'),
                "leftAssoc" | "rightAssoc" | "precedence"
            )
        });
        let tier = canonical(cx, r);
        if tier.is_none() && (annotated || rule.operator.is_some()) {
            noncanonical.push(r);
        }
        if let Some(t) = tier {
            let fail = |message: &str| AnalysisError::InvalidOperator {
                rule_id: rule.symbol_id.clone(),
                message: message.into(),
            };
            if rule.annotations.iter().any(|a| {
                (a.name.trim_start_matches('@') == "leftAssoc"
                    && t.assoc != OperatorAssociativity::Left)
                    || (a.name.trim_start_matches('@') == "rightAssoc"
                        && t.assoc != OperatorAssociativity::Right)
            }) {
                errors.push(fail("結合性注釈と正規形が矛盾します"));
            }
            if let Some(op) = &rule.operator {
                let form = if t.assoc == OperatorAssociativity::Left {
                    OperatorCanonicalForm::LeftOpRightLists
                } else {
                    OperatorCanonicalForm::BaseOpSelfOrBase
                };
                let mut valid = op.associativity == t.assoc
                    && op.canonical_form == form
                    && op.base_rule_id == cx.sem.rules[t.base].symbol_id
                    && reference(cx, t.op)
                        .is_some_and(|r| cx.sem.rules[r].symbol_id == op.operator_rule_id);
                for (site, i) in [
                    (&op.left_site_id, t.left),
                    (&op.op_site_id, t.op),
                    (&op.right_site_id, t.right),
                ] {
                    valid &= cx.sem.captures.iter().any(|c| {
                        &c.site_id == site
                            && c.rule_id == rule.symbol_id
                            && ungroup(cx, cx.index(&c.expr_id)) == i
                    });
                }
                for annotation in &rule.annotations {
                    if annotation.name.trim_start_matches('@') == "precedence" {
                        if let Some(JsonValue::Number(level)) = annotation.arguments.get("level") {
                            valid &= level.as_i64() == Some(op.precedence);
                        }
                    }
                }
                if !valid {
                    errors.push(fail(
                        "operator metadata と Expr 木/capture/precedence が矛盾します",
                    ));
                }
            }
            tiers.insert(r, t);
        } else if rule.operator.is_some() {
            errors.push(AnalysisError::InvalidOperator {
                rule_id: rule.symbol_id.clone(),
                message: "operator metadata が指定する正規形ではありません".into(),
            });
        }
    }
    let bases: BTreeSet<_> = tiers.values().map(|t| t.base).collect();
    let mut roots: Vec<_> = tiers
        .keys()
        .filter(|r| !bases.contains(r))
        .copied()
        .collect();
    roots.extend(tiers.keys().filter(|r| bases.contains(r)).copied());
    let mut visited = BTreeSet::new();
    let mut plans = Vec::new();
    for root in roots {
        if visited.contains(&root) {
            continue;
        }
        let mut tower = Vec::new();
        let mut next = root;
        while let Some(t) = tiers.get(&next) {
            if !visited.insert(next) {
                break;
            }
            tower.push(t.clone());
            next = t.base;
        }
        if tower.is_empty() {
            continue;
        }
        let reachable = cx.reachable(cx.index(&cx.sem.rules[root].body));
        let mut eligible = tower.last().is_some_and(|t| !tiers.contains_key(&t.base));
        let mut reasons = Vec::new();
        for &i in &reachable {
            let e = &cx.sem.expressions[i];
            let unsafe_node = cx.facts[i].trivia
                || matches!(
                    e.node,
                    ExprNode::Lookahead { .. }
                        | ExprNode::NegativeLookahead { .. }
                        | ExprNode::Empty { .. }
                        | ExprNode::Eof { .. }
                        | ExprNode::Error { .. }
                )
                || matches!(&e.node,ExprNode::TokenRef { symbol_id } if unsafe_token(&cx.sem.tokens[cx.tokens[symbol_id]]) || matches!(cx.sem.tokens[cx.tokens[symbol_id]].spec,TokenSpec::Extern { .. } | TokenSpec::Until { .. } | TokenSpec::Lookahead { .. } | TokenSpec::NegativeLookahead { .. } | TokenSpec::Empty { .. } | TokenSpec::Eof { .. } | TokenSpec::Regex { .. } | TokenSpec::Builtin { which: BuiltinToken::EndOfSource, .. }))
                || cx
                    .sem
                    .scope
                    .effects
                    .iter()
                    .any(|s| s.rule_id == e.rule_id && !s.events.is_empty())
                || cx
                    .sem
                    .recovery
                    .iter()
                    .any(|r| r.rule_id == e.rule_id || r.application_sites.contains(&e.id));
            if unsafe_node {
                eligible = false;
            }
        }
        if !eligible {
            reasons.push("到達領域に trivia・副作用・recovery・先読み等がある、または塔が循環する");
        }
        let mut operator_words = Vec::new();
        for t in &tower {
            if let Some(w) = words(cx, t.op, &mut BTreeSet::new()) {
                operator_words.extend(w);
            } else {
                eligible = false;
                reasons.push("演算子が有限 literal 選択ではない");
            }
            if cx.facts[cx.index(&cx.sem.rules[t.base].body)].minimum() == 0 {
                eligible = false;
                reasons.push("基底の前進を証明できない");
            }
            let rule = &cx.sem.rules[t.rule];
            if cx.sem.captures.iter().any(|c| {
                c.rule_id == rule.symbol_id
                    && !rule.operator.as_ref().is_some_and(|op| {
                        [&op.left_site_id, &op.op_site_id, &op.right_site_id].contains(&&c.site_id)
                    })
            }) {
                eligible = false;
                reasons.push("canonical operator metadata 外の capture を lift へ投影できない");
            }
            if rule.mapping_type_id.is_some()
                && !cx
                    .sem
                    .recipes
                    .iter()
                    .any(|r| r.rule_id == rule.symbol_id && r.kind == RecipeKind::AssocLists)
            {
                eligible = false;
                reasons.push("各段の assocLists lift recipe がない");
            }
        }
        if operator_words.iter().enumerate().any(|(i, a)| {
            operator_words[i + 1..]
                .iter()
                .any(|b| a.starts_with(b) || b.starts_with(a))
        }) {
            eligible = false;
            reasons.push("演算子候補が接頭辞で重なる");
        }
        let ids: Vec<_> = tower
            .iter()
            .map(|t| cx.sem.rules[t.rule].body.clone())
            .collect();
        let rule_ids: Vec<_> = tower
            .iter()
            .map(|t| cx.sem.rules[t.rule].symbol_id.clone())
            .collect();
        plans.push(OperatorPlan {
            boundary_expr_ids: ids.clone(), lift_recipe_ids: tower.iter().flat_map(|t| cx.sem.recipes.iter().filter(move |r| r.rule_id == cx.sem.rules[t.rule].symbol_id).map(|r| r.id.clone())).collect(),
            rule_ids, strategy: if eligible { OperatorPlanStrategy::PrecedenceClimbing } else { OperatorPlanStrategy::TierFunctions },
            proof: Some(Proof { kind: if eligible { ProofKind::NoTriviaEffectsRecovery } else { ProofKind::None },expr_ids: ids,
                explanation: if eligible { "参照階層順の正規形。到達領域に trivia・scope・recovery・先読み・Extern がなく、演算子は接頭辞排他的、基底は前進する。D-004 の lift/ceiling/右辺失敗 rollback を保持する。左右結合の段別参照評価との差分テストでこの形を検証。".into() }
                    else { reasons.join("。") } }),
        });
    }
    for r in noncanonical {
        let rule = &cx.sem.rules[r];
        plans.push(OperatorPlan {
            rule_ids: vec![rule.symbol_id.clone()],
            boundary_expr_ids: vec![rule.body.clone()],
            lift_recipe_ids: Vec::new(),
            strategy: OperatorPlanStrategy::TierFunctions,
            proof: Some(Proof {
                kind: ProofKind::None,
                expr_ids: vec![rule.body.clone()],
                explanation: "注釈付きだが D-004 の正規形ではないため段別関数を使用する。".into(),
            }),
        });
    }
    plans
}
