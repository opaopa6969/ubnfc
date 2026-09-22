use super::Context;
use crate::ir::*;
use std::collections::BTreeMap;

/// 自動 Predict の診断証明は単一 code point の直接 literal に限定。
/// backend は元順で試すはずだった不一致 literal の入口診断を再生する。
/// 複数文字・rule 呼出しの失敗位置/stack を推測した最適化はしない。
pub(super) fn exclusive(cx: &Context<'_>, alternatives: &[ExprId]) -> bool {
    !alternatives.is_empty() && alternatives.iter().all(|id| {
        let f = cx.fact(id);
        !f.nullable() && f.certainty != MayStartCertainty::Unknown && !f.trivia
            && matches!(&cx.sem.expressions[cx.index(id)].node, ExprNode::Literal { case_sensitive: true,value,.. } if value.chars().count() == 1)
            && cx.sem.expressions[cx.index(id)].capture_site_ids.is_empty()
    }) && alternatives.iter().enumerate().all(|(i,a)| alternatives[i+1..].iter().all(|b| {
        let (a,b) = (cx.fact(a),cx.fact(b)); a.chars.disjoint(&b.chars) && !(a.eof && b.eof)
    }))
}

fn trie(cx: &Context<'_>, expr: &Expr, alternatives: &[ExprId]) -> Option<PrefixTrie> {
    // 非 literal 候補は各 node に残す。trivia 付き literal は未知候補。
    let words: Vec<_> = alternatives
        .iter()
        .map(|id| {
            let e = &cx.sem.expressions[cx.index(id)];
            match &e.node {
                ExprNode::Literal {
                    value,
                    case_sensitive: true,
                    ..
                } if !cx.fact(id).trivia => {
                    Some(value.chars().map(|c| c as u64).collect::<Vec<_>>())
                }
                _ => None,
            }
        })
        .collect();
    if words.iter().all(Option::is_none) {
        return None;
    }
    let mut prefixes = vec![Vec::<u64>::new()];
    let mut edges = vec![BTreeMap::<u64, usize>::new()];
    for word in words.iter().flatten() {
        let mut node = 0;
        for &cp in word {
            node = if let Some(&next) = edges[node].get(&cp) {
                next
            } else {
                let next = prefixes.len();
                let mut prefix = prefixes[node].clone();
                prefix.push(cp);
                prefixes.push(prefix);
                edges.push(BTreeMap::new());
                edges[node].insert(cp, next);
                next
            };
        }
    }
    let nodes = prefixes
        .iter()
        .enumerate()
        .map(|(i, prefix)| {
            let mut candidates = Vec::new();
            let mut complete = Vec::new();
            let mut unknown = Vec::new();
            for (id, word) in alternatives.iter().zip(&words) {
                match word {
                    None => {
                        candidates.push(id.clone());
                        unknown.push(id.clone());
                    }
                    Some(word) if prefix.starts_with(word) => {
                        candidates.push(id.clone());
                        complete.push(id.clone());
                    }
                    Some(word) if word.starts_with(prefix) => candidates.push(id.clone()),
                    _ => {}
                }
            }
            TrieNode {
                index: i as u64,
                edges: edges[i]
                    .iter()
                    .map(|(&code_point, &target)| TrieEdge {
                        code_point,
                        target: target as u64,
                    })
                    .collect(),
                accepts_eof: alternatives
                    .iter()
                    .zip(&words)
                    .any(|(id, w)| w.is_none() && cx.fact(id).eof)
                    || words.iter().flatten().any(Vec::is_empty),
                candidate_expr_ids: candidates,
                completed_candidate_expr_ids: complete,
                unknown_candidate_expr_ids: unknown,
            }
        })
        .collect();
    Some(PrefixTrie {
        id: format!("trie:{}", expr.id),
        max_depth: prefixes.iter().map(Vec::len).max().unwrap_or(0) as u64,
        nodes,
    })
}

pub(super) fn plans(cx: &Context<'_>) -> (Vec<ChoicePlan>, Vec<PrefixTrie>) {
    let mut choices = Vec::new();
    let mut tries = Vec::new();
    for e in &cx.sem.expressions {
        let ExprNode::Choice {
            alternatives,
            selection,
            ..
        } = &e.node
        else {
            continue;
        };
        let t = trie(cx, e, alternatives);
        let mut plan = ChoicePlan {
            expr_id: e.id.clone(),
            candidate_order: alternatives.clone(),
            strategy: ChoicePlanStrategy::Ordered,
            fallback: ChoicePlanFallback::None,
            proof: None,
            trie_id: t.as_ref().map(|t| t.id.clone()),
        };
        match selection {
            ChoiceSelection::Longest => plan.strategy = ChoicePlanStrategy::Longest,
            ChoiceSelection::Predictive => {
                plan.strategy = ChoicePlanStrategy::Predict;
                plan.fallback = ChoicePlanFallback::FullChoice;
                plan.proof = Some(Proof { kind: ProofKind::DiagnosticEquivalent, expr_ids: alternatives.clone(),
                    explanation: "mayStart は必要条件の上界。nullable/unknown 候補を残し、候補なし・全失敗時は元順の完全選択を再試行する。除外候補の診断は semantics.diagnostics に従って保持する。".into() });
            }
            ChoiceSelection::Ordered if exclusive(cx, alternatives) => {
                plan.strategy = ChoicePlanStrategy::Predict;
                plan.proof = Some(Proof { kind: ProofKind::ExclusiveCandidates, expr_ids: alternatives.clone(),
                    explanation: "非 nullable・trivia なしの単文字 literal の mayStart が EOF を含め pairwise disjoint。元順で訪問される不一致候補は入口で失敗し consume(0) するため、その診断と M リセットを再生できる。採用枝より後は ordered choice と同じく訪問しない。".into() });
            }
            _ => {}
        }
        if let Some(t) = t {
            tries.push(t);
        }
        choices.push(plan);
    }
    (choices, tries)
}
