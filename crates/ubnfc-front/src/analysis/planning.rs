use super::{children, AnalysisError, Context};
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};

/// rule の scope 境界を実行順に表す。Evaluate 内の Choice/Repeat/RuleRef は
/// Expr 木の実際に採用した経路で評価するため、分岐を平坦化しない。
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeStep {
    Event { rule_id: String, event: ScopeEvent },
    Evaluate { expr_id: ExprId },
}

pub(super) fn event_phase(when: ScopeEffectEventsWhen) -> u8 {
    match when {
        ScopeEffectEventsWhen::Entry => 0,
        ScopeEffectEventsWhen::SuccessBeforeLeave => 1,
        ScopeEffectEventsWhen::SuccessAfterLeave => 2,
        ScopeEffectEventsWhen::AfterDeclarations => 3,
    }
}

pub fn scope_schedule(
    sem: &Semantics,
    rule_id: &str,
) -> Result<Vec<ScopeStep>, Vec<AnalysisError>> {
    super::checks::validate(sem)?;
    let Some(rule) = sem.rules.iter().find(|r| r.symbol_id == rule_id) else {
        return Err(vec![AnalysisError::InvalidSemantics {
            location: rule_id.into(),
            message: "scope 対象 rule が存在しません".into(),
        }]);
    };
    let events: Vec<_> = sem
        .scope
        .effects
        .iter()
        .filter(|e| e.rule_id == rule_id)
        .flat_map(|e| &e.events)
        .collect();
    let mut steps = Vec::new();
    for phase in 0..4 {
        for &event in &events {
            if event_phase(event.when) == phase {
                steps.push(ScopeStep::Event {
                    rule_id: rule_id.into(),
                    event: event.clone(),
                });
            }
        }
        if phase == 0 {
            steps.push(ScopeStep::Evaluate {
                expr_id: rule.body.clone(),
            });
        }
    }
    Ok(steps)
}

pub(super) fn unsafe_token(t: &Token) -> bool {
    matches!(t.spec, TokenSpec::Extern { .. })
        && !(t.memo_safety.success
            && t.memo_safety.failure
            && t.memo_safety.provenance == MemoSafetyProvenance::UserAliasAssertion
            && t.memo_safety.asserted_alias.as_ref().is_some_and(|a| {
                a == &t.symbol_id || Some(a.as_str()) == t.symbol_id.rsplit("::").next()
            }))
}

fn effects(cx: &Context<'_>, root: usize) -> Vec<String> {
    fn visit(cx: &Context<'_>, i: usize, visiting: &mut BTreeSet<usize>, result: &mut Vec<String>) {
        if !visiting.insert(i) {
            return;
        }
        let e = &cx.sem.expressions[i];
        let body = cx.body(&e.rule_id) == i;
        let own: Vec<_> = cx
            .sem
            .scope
            .effects
            .iter()
            .filter(|s| body && s.rule_id == e.rule_id)
            .flat_map(|s| s.events.iter().enumerate())
            .collect();
        for (n, event) in &own {
            if event.when == ScopeEffectEventsWhen::Entry {
                result.push(format!("scope:{}:{n}", e.rule_id));
            }
        }
        match &e.node {
            ExprNode::Lookahead { .. } | ExprNode::NegativeLookahead { .. } => {}
            ExprNode::TokenRef { symbol_id }
                if matches!(
                    cx.sem.tokens[cx.tokens[symbol_id]].spec,
                    TokenSpec::Extern { .. }
                ) =>
            {
                result.push(format!("extern:{symbol_id}"));
            }
            _ => {
                for &j in &cx.deps[i] {
                    visit(cx, j, visiting, result);
                }
            }
        }
        for phase in 1..4 {
            for (n, event) in &own {
                if event_phase(event.when) == phase {
                    result.push(format!("scope:{}:{n}", e.rule_id));
                }
            }
        }
        for r in &cx.sem.recovery {
            if r.application_sites.contains(&e.id) {
                result.push(format!("recovery:{}", r.rule_id));
            }
        }
        // 同一 root の要約では共有部分グラフを一度だけ訪問する。
        // effects は出現列ではなく、最初の出現順を保つ unique な要約。
    }
    let mut result = Vec::new();
    visit(cx, root, &mut BTreeSet::new(), &mut result);
    let mut seen = BTreeSet::new();
    result.retain(|s| seen.insert(s.clone()));
    result
}

type MemoResult = (MemoPlan, Vec<MemoSafety>, Vec<Vec<String>>, BTreeSet<usize>);

pub(super) fn memo(cx: &Context<'_>, choices: &[ChoicePlan]) -> MemoResult {
    let mut unsafe_set: Vec<bool> = cx.sem.expressions.iter().map(|e| {
        matches!(&e.node,ExprNode::TokenRef { symbol_id } if unsafe_token(&cx.sem.tokens[cx.tokens[symbol_id]]))
    }).collect();
    loop {
        let mut changed = false;
        for i in 0..unsafe_set.len() {
            if !unsafe_set[i] && cx.deps[i].iter().any(|&j| unsafe_set[j]) {
                unsafe_set[i] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let summaries: Vec<_> = (0..cx.facts.len()).map(|i| effects(cx, i)).collect();
    let safety: Vec<_> = cx.sem.expressions.iter().enumerate().map(|(i,e)| {
        if let ExprNode::TokenRef { symbol_id } = &e.node {
            let token = &cx.sem.tokens[cx.tokens[symbol_id]];
            if matches!(token.spec,TokenSpec::Extern { .. }) && !unsafe_set[i] { return token.memo_safety.clone(); }
        }
        MemoSafety { asserted_alias: None, original_declaration: None, success: !unsafe_set[i], failure: !unsafe_set[i],
            provenance: if unsafe_set[i] { MemoSafetyProvenance::None } else { MemoSafetyProvenance::DerivedProof },
            purity: if unsafe_set[i] { MemoSafetyPurity::Unknown } else if summaries[i].is_empty() { MemoSafetyPurity::Pure } else { MemoSafetyPurity::Effectful },
            reason: if unsafe_set[i] { "保証のない Extern へ到達するため成功・失敗とも memo 不可。" }
                else if summaries[i].is_empty() { "既知組込み token と純粋な規則のみ。scope、Extern、recovery はない。" }
                else { "保証された token と scope/recovery 効果を順序付きで再生。stateVersion を memo キーに含める。" }.into() }
    }).collect();
    let mut omissions = BTreeMap::<usize, Proof>::new();
    let mut omitted_rules = BTreeSet::new();
    for group in &cx.rule_sccs {
        if group.len() != 1 || cx.rule_graph[group[0]].contains(&group[0]) {
            continue;
        }
        let r = group[0];
        let root = cx.index(&cx.sem.rules[r].body);
        let local = cx.local_tree(root);
        if local
            .iter()
            .any(|&i| unsafe_set[i] || cx.facts[i].trivia || !summaries[i].is_empty())
        {
            continue;
        }
        let mut incoming = BTreeMap::<usize, usize>::new();
        for &i in &local {
            for id in children(&cx.sem.expressions[i].node) {
                *incoming.entry(cx.index(id)).or_default() += 1;
            }
        }
        let mut stack = vec![(
            root,
            ProofKind::SingleExecutionPath,
            vec![cx.sem.expressions[root].id.clone()],
        )];
        while let Some((i, kind, path)) = stack.pop() {
            if i != root && incoming[&i] != 1 {
                continue;
            }
            if i != root {
                omissions.insert(i,Proof { kind,expr_ids: path.clone(), explanation:
                    "rule 本体の memo を保持。非再帰 SCC の単一所有パスで、巻戻して同一式を再実行する祖先がない。排他的 choice は fallback なし、反復を通る場合は各周回の consumed 前進を証明。".into() });
                omitted_rules.insert(r);
            }
            let (next, next_kind) = match &cx.sem.expressions[i].node {
                ExprNode::Seq { items } => (items.iter().collect::<Vec<_>>(), kind),
                ExprNode::Group { child } => (vec![child], kind),
                ExprNode::Choice { alternatives, .. }
                    if choices.iter().any(|c| {
                        c.expr_id == cx.sem.expressions[i].id
                            && c.strategy == ChoicePlanStrategy::Predict
                            && c.fallback == ChoicePlanFallback::None
                    }) =>
                {
                    (
                        alternatives.iter().collect(),
                        ProofKind::ExclusiveCandidates,
                    )
                }
                ExprNode::Repeat {
                    child,
                    terminator: None,
                    max,
                    ..
                } if *max != Some(0) && cx.fact(child).minimum() > 0 => {
                    (vec![child], ProofKind::MonotoneNoRollback)
                }
                _ => (Vec::new(), kind),
            };
            for id in next.into_iter().rev() {
                let mut p = path.clone();
                p.push(id.clone());
                stack.push((cx.index(id), next_kind, p));
            }
        }
    }
    let entries = cx
        .sem
        .expressions
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let proof = omissions.remove(&i);
            let eligible = !unsafe_set[i] && proof.is_none();
            MemoEntry {
                expr_id: e.id.clone(),
                cache_success: eligible,
                cache_failure: eligible,
                reason: if unsafe_set[i] {
                    MemoEntryReason::UnsafeExternClosure
                } else if proof.is_some() {
                    MemoEntryReason::ProvedOmission
                } else {
                    MemoEntryReason::DefaultSafe
                },
                omission_proof: proof,
            }
        })
        .collect();
    (
        MemoPlan {
            entries,
            reference_mode: MemoPlanReferenceMode,
            replay: MemoPlanReplay,
            required_key: MemoPlanRequiredKey,
        },
        safety,
        summaries,
        omitted_rules,
    )
}
