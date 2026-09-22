//! D-016 が未確定のため、順序付き E ::= E Op Base | Base に限定する。
//! Op/Base の進捗が証明でき、当該 rule に capture/mapping/scope/recovery が
//! 無い場合だけ Base (Op Base)* とする。trivia は通常の chain 境界で挿入。
use super::*;
use r::{ResolvedAtom as A, ResolvedBody, ResolvedRule, SymbolId};
use std::collections::BTreeSet;

fn nullable_atom(a: &A, nullable: &BTreeSet<SymbolId>, cx: &Context<'_>) -> bool {
    match a {
        A::Literal { value, .. } => value.is_empty(),
        A::Error { .. } => false,
        A::Ref { symbol, .. } => {
            if cx.rule(*symbol).is_some() {
                nullable.contains(symbol)
            } else {
                cx.grammar
                    .grammars
                    .iter()
                    .flat_map(|g| &g.tokens)
                    .find(|t| t.id == *symbol)
                    .is_none_or(|t| {
                        matches!(
                            t.value,
                            r::ResolvedTokenValue::Eof
                                | r::ResolvedTokenValue::Empty
                                | r::ResolvedTokenValue::Until { .. }
                                | r::ResolvedTokenValue::Lookahead { .. }
                                | r::ResolvedTokenValue::NegativeLookahead { .. }
                                | r::ResolvedTokenValue::Extern { .. }
                                | r::ResolvedTokenValue::Builtin {
                                    kind: r::BuiltinKind::EndOfSource
                                }
                        )
                    })
            }
        }
        A::Group { body, .. } => nullable_body(body, nullable, cx),
        A::Optional { .. } | A::Repeat { .. } => true,
        A::OneOrMore { body, .. } => nullable_atom(body, nullable, cx),
        A::Bounded { body, min, .. } => *min == 0 || nullable_atom(body, nullable, cx),
        A::Separated { element, .. } => nullable_atom(element, nullable, cx),
    }
}
fn nullable_body(b: &ResolvedBody, n: &BTreeSet<SymbolId>, cx: &Context<'_>) -> bool {
    b.alternatives
        .iter()
        .any(|s| s.iter().all(|e| nullable_atom(&e.atom, n, cx)))
}
fn nullable(cx: &Context<'_>) -> BTreeSet<SymbolId> {
    let mut n = BTreeSet::new();
    loop {
        let prev = n.len();
        for r in cx.grammar.grammars.iter().flat_map(|g| &g.rules) {
            if nullable_body(&r.body, &n, cx) {
                n.insert(r.id);
            }
        }
        if n.len() == prev {
            return n;
        }
    }
}
fn heads(a: &A, n: &BTreeSet<SymbolId>, cx: &Context<'_>, out: &mut BTreeSet<SymbolId>) {
    match a {
        A::Ref { symbol, .. } => {
            if cx.rule(*symbol).is_some() {
                out.insert(*symbol);
            }
        }
        A::Group { body, .. } | A::Optional { body, .. } | A::Repeat { body, .. } => {
            body_heads(body, n, cx, out)
        }
        A::OneOrMore { body, .. }
        | A::Bounded { body, .. }
        | A::Separated { element: body, .. } => heads(body, n, cx, out),
        _ => {}
    }
}
fn body_heads(
    b: &ResolvedBody,
    n: &BTreeSet<SymbolId>,
    cx: &Context<'_>,
    out: &mut BTreeSet<SymbolId>,
) {
    for seq in &b.alternatives {
        for e in seq {
            heads(&e.atom, n, cx, out);
            if !nullable_atom(&e.atom, n, cx) {
                break;
            }
        }
    }
}
pub(super) fn check(cx: &mut Context<'_>) {
    let n = nullable(cx);
    let mut graph = BTreeMap::new();
    for rule in cx.grammar.grammars.iter().flat_map(|g| &g.rules) {
        let mut h = BTreeSet::new();
        body_heads(&rule.body, &n, cx, &mut h);
        graph.insert(rule.id, h);
    }
    for rule in cx.grammar.grammars.iter().flat_map(|g| &g.rules) {
        let mut visiting = BTreeSet::new();
        let mut pending: Vec<_> = graph[&rule.id]
            .iter()
            .filter(|s| **s != rule.id)
            .copied()
            .collect();
        while let Some(id) = pending.pop() {
            if id == rule.id {
                cx.errors.push(error(
                    "E-LOWER-INDIRECT-LEFT-RECURSION",
                    format!("{}: 間接左再帰は D-010 で未対応", rule.name),
                ));
                break;
            }
            if visiting.insert(id) {
                pending.extend(graph[&id].iter().copied());
            }
        }
        if graph[&rule.id].contains(&rule.id) && !supported(rule, &n, cx) {
            cx.errors.push(error("E-LOWER-D016",format!("{}: D-016 未定義の左再帰形（順序・nullable suffix・capture/mapping/scope/recovery の投影）",rule.name)));
        }
    }
}
fn supported(rule: &ResolvedRule, n: &BTreeSet<SymbolId>, cx: &Context<'_>) -> bool {
    let [recursive, base] = rule.body.alternatives.as_slice() else {
        return false;
    };
    let [self_ref, op, tail] = recursive.as_slice() else {
        return false;
    };
    let [base] = base.as_slice() else {
        return false;
    };
    if !matches!(self_ref.atom,A::Ref {symbol,..} if symbol==rule.id) {
        return false;
    }
    if !same_atom(&tail.atom, &base.atom)
        || nullable_atom(&op.atom, n, cx)
        || nullable_atom(&base.atom, n, cx)
    {
        return false;
    }
    if recursive
        .iter()
        .chain(std::iter::once(base))
        .any(|e| e.capture.is_some() || e.typeof_constraint.is_some())
    {
        return false;
    }
    // この初期拡張では group 内の occurrence 投影も未定義。
    if !matches!(op.atom, A::Literal { .. } | A::Ref { .. })
        || !matches!(base.atom, A::Literal { .. } | A::Ref { .. })
    {
        return false;
    }
    projection_free(rule, cx, &mut BTreeSet::new())
        && !reaches(&op.atom, rule.id, cx, &mut BTreeSet::new())
}
fn plain_annotation(a: &a::Annotation) -> bool {
    matches!(
        a,
        a::Annotation::Root { .. }
            | a::Annotation::Doc { .. }
            | a::Annotation::Whitespace { .. }
            | a::Annotation::Interleave { .. }
    )
}
// 参照先の scope/mapping/capture を含む投影は D-016 の対象外。
fn projection_free(rule: &ResolvedRule, cx: &Context<'_>, seen: &mut BTreeSet<SymbolId>) -> bool {
    if !seen.insert(rule.id) {
        return true;
    }
    rule.annotations.iter().all(plain_annotation)
        && rule.body.alternatives.iter().flatten().all(|e| {
            e.capture.is_none()
                && e.typeof_constraint.is_none()
                && atom_projection_free(&e.atom, cx, seen)
        })
}
fn atom_projection_free(atom: &A, cx: &Context<'_>, seen: &mut BTreeSet<SymbolId>) -> bool {
    match atom {
        A::Ref { symbol, .. } => cx
            .rule(*symbol)
            .is_none_or(|r| projection_free(r, cx, seen)),
        A::Group { body, .. } | A::Optional { body, .. } | A::Repeat { body, .. } => {
            body.alternatives.iter().flatten().all(|e| {
                e.capture.is_none()
                    && e.typeof_constraint.is_none()
                    && atom_projection_free(&e.atom, cx, seen)
            })
        }
        A::OneOrMore { body, .. } | A::Bounded { body, .. } => atom_projection_free(body, cx, seen),
        A::Separated {
            element, separator, ..
        } => atom_projection_free(element, cx, seen) && atom_projection_free(separator, cx, seen),
        _ => true,
    }
}
fn reaches(atom: &A, target: SymbolId, cx: &Context<'_>, seen: &mut BTreeSet<SymbolId>) -> bool {
    match atom {
        A::Ref { symbol, .. } => {
            *symbol == target
                || (seen.insert(*symbol)
                    && cx.rule(*symbol).is_some_and(|r| {
                        r.body
                            .alternatives
                            .iter()
                            .flatten()
                            .any(|e| reaches(&e.atom, target, cx, seen))
                    }))
        }
        A::Group { body, .. } | A::Optional { body, .. } | A::Repeat { body, .. } => body
            .alternatives
            .iter()
            .flatten()
            .any(|e| reaches(&e.atom, target, cx, seen)),
        A::OneOrMore { body, .. } | A::Bounded { body, .. } => reaches(body, target, cx, seen),
        A::Separated {
            element, separator, ..
        } => reaches(element, target, cx, seen) || reaches(separator, target, cx, seen),
        _ => false,
    }
}
fn same_atom(a: &A, b: &A) -> bool {
    match (a, b) {
        (A::Ref { symbol: a, .. }, A::Ref { symbol: b, .. }) => a == b,
        (A::Literal { value: a, .. }, A::Literal { value: b, .. }) => a == b,
        _ => false,
    }
}
pub(super) fn rewrite(
    cx: &mut Context<'_>,
    rule: &ResolvedRule,
    original: ExprId,
    policy: &str,
) -> ExprId {
    if !supported(rule, &nullable(cx), cx) {
        return original;
    }
    let expressions = &cx.out.semantics.expressions;
    let original_expr = expressions.iter().find(|e| e.id == original).unwrap();
    let ExprNode::Choice { alternatives, .. } = &original_expr.node else {
        return original;
    };
    let base = alternatives[1].clone();
    let recursive = expressions
        .iter()
        .find(|e| e.id == alternatives[0])
        .unwrap();
    let ExprNode::Seq { items } = &recursive.node else {
        return original;
    };
    let suffix = items[1..].to_vec();
    let source = original_expr.source.clone();
    let root = ExprId(format!("{original}/lower/leftRecursion/0"));
    let repeat = ExprId(format!("{original}/lower/leftRecursion/1"));
    let pair = ExprId(format!("{original}/lower/leftRecursion/2"));
    for (id, node) in [
        (
            root.clone(),
            ExprNode::Seq {
                items: vec![base, repeat.clone()],
            },
        ),
        (
            repeat,
            ExprNode::Repeat {
                child: pair.clone(),
                min: 0,
                max: None,
                terminator: None,
            },
        ),
        (pair, ExprNode::Seq { items: suffix }),
    ] {
        if let ExprNode::Seq { items } = &node {
            trivia::chain(cx, policy, &id, items.len());
        }
        cx.out.semantics.expressions.push(Expr {
            id,
            node: node.clone(),
            source: source.clone(),
            rule_id: cx.sid(rule.id),
            capture_site_ids: vec![],
            diagnostic_label: None,
            execution: expr::execution(&node),
        });
    }
    cx.out.semantics.normalizations.push(Rewrite {kind:RewriteKind,original_body:original.clone(),rewritten_body:root.clone(),rule_id:cx.sid(rule.id),proof:Proof {kind:ProofKind::LeftRecursionNormalization,expr_ids:vec![original,root.clone()],explanation:"D-010 extension: ordered E ::= E Op Base | Base; Op and Base consume; no reachable captures/mapping/scope/recovery; operator does not reach E; lower to Base (Op Base)* with chain entry/after-child trivia. Original expressions retained.".into()}});
    root
}
