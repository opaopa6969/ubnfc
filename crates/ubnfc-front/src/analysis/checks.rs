use super::{children, graph, AnalysisError, Context};
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};

fn invalid(errors: &mut Vec<AnalysisError>, location: impl ToString, message: impl Into<String>) {
    errors.push(AnalysisError::InvalidSemantics {
        location: location.to_string(),
        message: message.into(),
    });
}

pub(super) fn validate(sem: &Semantics) -> Result<(), Vec<AnalysisError>> {
    let mut errors = Vec::new();
    let exprs: BTreeMap<_, _> = sem
        .expressions
        .iter()
        .enumerate()
        .map(|(i, e)| (&e.id, i))
        .collect();
    let rules: BTreeMap<_, _> = sem.rules.iter().map(|r| (&r.symbol_id, r)).collect();
    let tokens: BTreeSet<_> = sem.tokens.iter().map(|t| &t.symbol_id).collect();
    if exprs.len() != sem.expressions.len() {
        invalid(&mut errors, "expressions", "ExprId が重複しています");
    }
    if rules.len() != sem.rules.len() {
        invalid(&mut errors, "rules", "rule ID が重複しています");
    }
    if tokens.len() != sem.tokens.len() {
        invalid(&mut errors, "tokens", "token ID が重複しています");
    }
    for r in &sem.rules {
        if !exprs.contains_key(&r.body) {
            invalid(&mut errors, &r.symbol_id, "body が存在しません");
        } else if sem.expressions[exprs[&r.body]].rule_id != r.symbol_id {
            invalid(&mut errors, &r.symbol_id, "body の所属 rule が異なります");
        }
    }
    let mut structural = vec![Vec::new(); sem.expressions.len()];
    for (i, e) in sem.expressions.iter().enumerate() {
        if !rules.contains_key(&e.rule_id) {
            invalid(&mut errors, &e.id, "所属 rule が存在しません");
        }
        for id in children(&e.node) {
            if let Some(&j) = exprs.get(id) {
                structural[i].push(j);
                if sem.expressions[j].rule_id != e.rule_id {
                    invalid(&mut errors, &e.id, "子式の所属 rule が異なります");
                }
            } else {
                invalid(&mut errors, &e.id, format!("子式 {id} が存在しません"));
            }
        }
        match &e.node {
            ExprNode::RuleRef { symbol_id } if !rules.contains_key(symbol_id) => {
                invalid(&mut errors, &e.id, "参照 rule が存在しません")
            }
            ExprNode::TokenRef { symbol_id } if !tokens.contains(symbol_id) => {
                invalid(&mut errors, &e.id, "参照 token が存在しません")
            }
            ExprNode::Repeat {
                min,
                max: Some(max),
                ..
            } if min > max => invalid(&mut errors, &e.id, "repeat の min > max"),
            _ => {}
        }
    }
    for group in graph::scc(&structural) {
        if group.len() > 1 || structural[group[0]].contains(&group[0]) {
            invalid(
                &mut errors,
                &sem.expressions[group[0]].id,
                "Expr 木に構造循環があります（再帰は RuleRef を使用）",
            );
        }
    }
    for t in &sem.tokens {
        match &t.spec {
            TokenSpec::CharRange { min, max }
                if min > max || *max > 0xffff || (*min <= 0xdfff && *max >= 0xd800) =>
            {
                invalid(
                    &mut errors,
                    &t.symbol_id,
                    "CHAR_RANGE は BMP 非 surrogate の昇順範囲が必要です",
                )
            }
            TokenSpec::Builtin { code_points, .. }
                if code_points.iter().any(|cp| *cp > 0x10ffff) =>
            {
                invalid(&mut errors, &t.symbol_id, "code point が範囲外です")
            }
            _ => {}
        }
    }
    let policies: BTreeSet<_> = sem.trivia.policies.iter().map(|p| &p.id).collect();
    for p in &sem.trivia.insertion_plan {
        if !exprs.contains_key(&p.boundary.owner_expr_id) || !policies.contains(&p.policy_id) {
            invalid(
                &mut errors,
                &p.boundary.owner_expr_id,
                "trivia の式/policy 参照が不正です",
            );
            continue;
        }
        if p.boundary.at == BoundaryAt::AfterChild {
            let n = children(&sem.expressions[exprs[&p.boundary.owner_expr_id]].node).len();
            if p.boundary.child_index.is_none_or(|i| i >= n as u64) {
                invalid(
                    &mut errors,
                    &p.boundary.owner_expr_id,
                    "afterChild の childIndex が不正です",
                );
            }
        } else if p.boundary.child_index.is_some() {
            invalid(
                &mut errors,
                &p.boundary.owner_expr_id,
                "afterChild 以外の childIndex は null です",
            );
        }
    }
    for n in &sem.normalizations {
        if !exprs.contains_key(&n.original_body)
            || !exprs.contains_key(&n.rewritten_body)
            || !rules
                .get(&n.rule_id)
                .is_some_and(|r| r.body == n.rewritten_body)
            || n.proof.kind != ProofKind::LeftRecursionNormalization
            || n.original_body == n.rewritten_body
        {
            invalid(&mut errors, &n.rule_id, "直接左再帰の正規化記録が不正です");
        }
    }
    let sites: BTreeMap<_, _> = sem.captures.iter().map(|c| (&c.site_id, c)).collect();
    if sites.len() != sem.captures.len() {
        invalid(&mut errors, "captures", "capture site ID が重複しています");
    }
    for c in &sem.captures {
        if !rules.contains_key(&c.rule_id)
            || !exprs
                .get(&c.expr_id)
                .is_some_and(|&i| sem.expressions[i].rule_id == c.rule_id)
        {
            invalid(&mut errors, &c.site_id, "capture の式/rule 参照が不正です");
        }
    }
    for e in &sem.expressions {
        for site in &e.capture_site_ids {
            if !sites.get(site).is_some_and(|c| c.expr_id == e.id) {
                invalid(
                    &mut errors,
                    &e.id,
                    "captureSiteIds が capture 定義と一致しません",
                );
            }
        }
    }
    for r in &sem.recovery {
        if !rules.contains_key(&r.rule_id) {
            invalid(&mut errors, &r.rule_id, "recovery rule が存在しません");
        }
        for id in &r.application_sites {
            if !exprs.get(id).is_some_and(|&i| matches!(&sem.expressions[i].node,ExprNode::RuleRef { symbol_id } if symbol_id == &r.rule_id)) {
                invalid(&mut errors,id,"recovery site は対象 rule への参照が必要です");
            }
        }
    }
    for effect in &sem.scope.effects {
        if !rules.contains_key(&effect.rule_id) {
            invalid(&mut errors, &effect.rule_id, "scope rule が存在しません");
        }
        let mut phase = 0;
        for event in &effect.events {
            let next = super::planning::event_phase(event.when);
            if next < phase {
                invalid(
                    &mut errors,
                    &effect.rule_id,
                    "scope イベント順序が entry/leave/declare/use 契約と矛盾します",
                );
            }
            phase = next;
            let valid = match event.action {
                ScopeEffectEventsAction::Enter => event.when == ScopeEffectEventsWhen::Entry,
                ScopeEffectEventsAction::Leave => {
                    event.when == ScopeEffectEventsWhen::SuccessBeforeLeave
                }
                ScopeEffectEventsAction::Declare => {
                    event.when == ScopeEffectEventsWhen::SuccessAfterLeave
                }
                ScopeEffectEventsAction::Use => {
                    event.when == ScopeEffectEventsWhen::AfterDeclarations
                }
                ScopeEffectEventsAction::Clear => true,
            };
            if !valid {
                invalid(
                    &mut errors,
                    &effect.rule_id,
                    "scope action の実行時点が契約と矛盾します",
                );
            }
            if let Some(site) = &event.capture_site_id {
                if !sites.get(site).is_some_and(|c| c.rule_id == effect.rule_id) {
                    invalid(
                        &mut errors,
                        site,
                        "scope capture が存在しないか別 rule に属します",
                    );
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub(super) fn left_recursion(cx: &Context<'_>) -> Vec<AnalysisError> {
    fn collect(cx: &Context<'_>, i: usize, out: &mut BTreeSet<usize>) {
        match &cx.sem.expressions[i].node {
            ExprNode::RuleRef { symbol_id } => {
                out.insert(cx.rules[symbol_id]);
            }
            ExprNode::Seq { items } => {
                for id in items {
                    collect(cx, cx.index(id), out);
                    if !cx.fact(id).nullable() {
                        break;
                    }
                }
            }
            ExprNode::Separated {
                element, separator, ..
            } => {
                collect(cx, cx.index(element), out);
                if cx.fact(element).nullable() {
                    collect(cx, cx.index(separator), out);
                }
            }
            node => {
                for id in children(node) {
                    collect(cx, cx.index(id), out);
                }
            }
        }
    }
    let graph: Vec<_> = cx
        .sem
        .rules
        .iter()
        .map(|r| {
            let mut refs = BTreeSet::new();
            collect(cx, cx.index(&r.body), &mut refs);
            refs.into_iter().collect()
        })
        .collect();
    let mut groups = graph::scc(&graph);
    groups.sort_by_key(|s| s[0]);
    groups
        .into_iter()
        .filter_map(|g| {
            if g.len() > 1 {
                Some(AnalysisError::IndirectLeftRecursion {
                    rule_ids: g
                        .iter()
                        .map(|&r| cx.sem.rules[r].symbol_id.clone())
                        .collect(),
                })
            } else if graph[g[0]].contains(&g[0]) {
                Some(AnalysisError::DirectLeftRecursion {
                    rule_id: cx.sem.rules[g[0]].symbol_id.clone(),
                })
            } else {
                None
            }
        })
        .collect()
}

// 0/1/複数の飽和区間。同名 nested capture は加算、choice は枝ごとの区間の和集合。
#[derive(Clone, Copy, Default)]
struct Count {
    min: u8,
    max: u8,
}
impl Count {
    fn add(self, b: Self) -> Self {
        Self {
            min: (self.min + b.min).min(2),
            max: (self.max + b.max).min(2),
        }
    }
    fn times(self, min: u64, max: Option<u64>) -> Self {
        Self {
            min: self.min.saturating_mul(min.min(2) as u8).min(2),
            max: self
                .max
                .saturating_mul(max.unwrap_or(2).min(2) as u8)
                .min(2),
        }
    }
    fn cardinality(self) -> Cardinality {
        if self.max > 1 {
            Cardinality::List
        } else if self.min == 0 {
            Cardinality::Optional
        } else {
            Cardinality::Scalar
        }
    }
}

pub(super) fn cardinality(cx: &Context<'_>) -> Vec<AnalysisError> {
    fn count(cx: &Context<'_>, i: usize, name: &str) -> Count {
        let e = &cx.sem.expressions[i];
        let own_count = cx
            .sem
            .captures
            .iter()
            .filter(|c| c.expr_id == e.id && c.name == name)
            .count()
            .min(2) as u8;
        let own = Count {
            min: own_count,
            max: own_count,
        };
        match &e.node {
            ExprNode::Choice { alternatives, .. } => {
                let values: Vec<_> = alternatives
                    .iter()
                    .map(|id| count(cx, cx.index(id), name))
                    .collect();
                own.add(Count {
                    min: values.iter().map(|v| v.min).min().unwrap_or(0),
                    max: values.iter().map(|v| v.max).max().unwrap_or(0),
                })
            }
            ExprNode::Optional { child } => {
                own.add(count(cx, cx.index(child), name)).times(0, Some(1))
            }
            ExprNode::Repeat {
                child, min, max, ..
            } => own.add(count(cx, cx.index(child), name)).times(*min, *max),
            ExprNode::Separated {
                element,
                separator,
                min,
                ..
            } => own
                .add(count(cx, cx.index(element), name))
                .times(*min, None)
                .add(count(cx, cx.index(separator), name).times(min.saturating_sub(1), None)),
            _ => children(&e.node)
                .into_iter()
                .fold(own, |c, id| c.add(count(cx, cx.index(id), name))),
        }
    }
    let mut errors = Vec::new();
    for c in &cx.sem.captures {
        // 保存された正規化前の式の metadata は新しい body と比較しない。
        if !cx.active.contains(&cx.index(&c.expr_id)) {
            continue;
        }
        let inferred = count(cx, cx.body(&c.rule_id), &c.name).cardinality();
        if inferred != c.cardinality {
            errors.push(AnalysisError::CaptureCardinality {
                site_id: c.site_id.clone(),
                declared: c.cardinality,
                inferred,
            });
        }
    }
    errors
}
