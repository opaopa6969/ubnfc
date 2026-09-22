use super::{children, graph, Context};
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};

/// FOLLOW は文字クラスと独立した、文法に書かれた終端文字列の有限集合。
/// opaque token の spelling は捏造しない。EOF は同期文字列ではない。
pub(super) fn compute(cx: &Context<'_>) -> BTreeMap<String, Vec<String>> {
    if !cx.sem.recovery.iter().any(|r| r.mode == RecoveryMode::Auto) {
        return BTreeMap::new();
    }
    let mut first = vec![BTreeSet::<String>::new(); cx.sem.expressions.len()];
    for group in graph::scc(&cx.deps) {
        loop {
            let mut changed = false;
            for &i in &group {
                let mut set = BTreeSet::new();
                match &cx.sem.expressions[i].node {
                    ExprNode::Literal { value, .. } if !value.is_empty() => {
                        set.insert(value.clone());
                    }
                    ExprNode::TokenRef { symbol_id } => {
                        if let TokenSpec::CaseInsensitive { word, .. } =
                            &cx.sem.tokens[cx.tokens[symbol_id]].spec
                        {
                            if !word.is_empty() {
                                set.insert(word.clone());
                            }
                        }
                    }
                    ExprNode::RuleRef { symbol_id } => {
                        set.extend(first[cx.body(symbol_id)].iter().cloned())
                    }
                    ExprNode::Seq { items } => {
                        for id in items {
                            set.extend(first[cx.index(id)].iter().cloned());
                            if !cx.fact(id).nullable() {
                                break;
                            }
                        }
                    }
                    ExprNode::Lookahead { .. } | ExprNode::NegativeLookahead { .. } => {}
                    ExprNode::Separated {
                        element, separator, ..
                    } => {
                        set.extend(first[cx.index(element)].iter().cloned());
                        if cx.fact(element).nullable() {
                            set.extend(first[cx.index(separator)].iter().cloned());
                        }
                    }
                    node => {
                        for id in children(node) {
                            set.extend(first[cx.index(id)].iter().cloned());
                        }
                    }
                }
                if first[i] != set {
                    first[i] = set;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }
    let mut follow = vec![BTreeSet::new(); first.len()];
    loop {
        let mut changed = false;
        for &i in &cx.active {
            let outer = follow[i].clone();
            let mut updates: Vec<(usize, BTreeSet<String>)> = Vec::new();
            match &cx.sem.expressions[i].node {
                ExprNode::RuleRef { symbol_id } => updates.push((cx.body(symbol_id), outer)),
                ExprNode::Seq { items } => {
                    let mut suffix = outer;
                    for id in items.iter().rev() {
                        let j = cx.index(id);
                        updates.push((j, suffix.clone()));
                        if !cx.fact(id).nullable() {
                            suffix.clear();
                        }
                        suffix.extend(first[j].iter().cloned());
                    }
                }
                ExprNode::Repeat {
                    child,
                    terminator,
                    max,
                    ..
                } => {
                    let j = cx.index(child);
                    let mut suffix = outer.clone();
                    if max.is_none_or(|n| n > 1) {
                        suffix.extend(first[j].iter().cloned());
                    }
                    if let Some(t) = terminator {
                        suffix.extend(first[cx.index(t)].iter().cloned());
                        updates.push((cx.index(t), outer));
                    }
                    updates.push((j, suffix));
                }
                ExprNode::Separated {
                    element, separator, ..
                } => {
                    let (e, s) = (cx.index(element), cx.index(separator));
                    let mut after = outer;
                    after.extend(first[s].iter().cloned());
                    if cx.fact(separator).nullable() {
                        after.extend(first[e].iter().cloned());
                    }
                    let mut after_separator = first[e].clone();
                    if cx.fact(element).nullable() {
                        after_separator.extend(after.iter().cloned());
                    }
                    updates.push((e, after));
                    updates.push((s, after_separator));
                }
                // 先読み内は別のカーソル。消費列の FOLLOW へ混ぜない。
                ExprNode::Lookahead { .. } | ExprNode::NegativeLookahead { .. } => {}
                node => {
                    for id in children(node) {
                        updates.push((cx.index(id), outer.clone()));
                    }
                }
            }
            for (j, set) in updates {
                let before = follow[j].len();
                follow[j].extend(set);
                changed |= follow[j].len() != before;
            }
        }
        if !changed {
            break;
        }
    }
    cx.sem
        .recovery
        .iter()
        .filter(|r| r.mode == RecoveryMode::Auto)
        .map(|r| {
            let mut patterns: Vec<_> = follow[cx.body(&r.rule_id)].iter().cloned().collect();
            if patterns.is_empty() {
                patterns.push(";".into());
            }
            (r.rule_id.clone(), patterns)
        })
        .collect()
}
