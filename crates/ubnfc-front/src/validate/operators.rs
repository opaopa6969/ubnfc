use std::collections::{BTreeMap, BTreeSet};

use crate::resolve::{ResolvedAtom, ResolvedBody, ResolvedRule, SymbolId};
use crate::syntax::ast::Annotation;

use super::{captures, last_mapping, nested_body, single_sequence, Context};

fn assoc(rule: &ResolvedRule) -> &'static str {
    let left = rule
        .annotations
        .iter()
        .any(|a| matches!(a, Annotation::LeftAssoc { .. }));
    let right = rule
        .annotations
        .iter()
        .any(|a| matches!(a, Annotation::RightAssoc { .. }));
    match (left, right) {
        (true, true) => "BOTH",
        (true, false) => "LEFT",
        (false, true) => "RIGHT",
        _ => "NONE",
    }
}

fn levels(rule: &ResolvedRule) -> Vec<u64> {
    rule.annotations
        .iter()
        .filter_map(|a| {
            if let Annotation::Precedence { level, .. } = a {
                Some(*level)
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn validate_rule(rule: &ResolvedRule, cx: &mut Context<'_>) {
    let name = &rule.name;
    let assoc = assoc(rule);
    if assoc == "BOTH" {
        cx.rule(
            rule,
            "E-ASSOC-BOTH",
            format!("rule {name} cannot use both @leftAssoc and @rightAssoc"),
            "Keep exactly one associativity annotation per rule.",
        );
    } else if assoc != "NONE" {
        let annotation = if assoc == "RIGHT" {
            "@rightAssoc"
        } else {
            "@leftAssoc"
        };
        if let Some((class, params)) = last_mapping(rule) {
            for required in ["left", "op", "right"] {
                if !params.iter().any(|p| p == required) {
                    cx.rule(rule, "E-ASSOC-MAPPING-PARAM", format!("rule {name} uses {annotation} but @mapping({class}) params does not contain '{required}'"),
                        "Include left/op/right in @mapping params.");
                }
            }
        } else {
            cx.rule(
                rule,
                "E-ASSOC-NO-MAPPING",
                format!("rule {name} uses {annotation} but has no @mapping"),
                "Add @mapping(ClassName, params=[left, op, right]) to this rule.",
            );
        }
        let captures = captures(rule);
        for required in ["left", "op", "right"] {
            if !captures.contains(&required) {
                cx.rule(
                    rule,
                    "E-ASSOC-MISSING-CAPTURE",
                    format!("rule {name} uses {annotation} but capture @{required} is missing"),
                    format!("Add @{required} capture in the rule body."),
                );
            }
        }
        if !contains_repeat(&rule.body) {
            cx.rule(
                rule,
                "E-ASSOC-NO-REPEAT",
                format!("rule {name} uses {annotation} but has no repeat segment"),
                "Use canonical operator pattern: Base { Op Right }.",
            );
        }
        if assoc == "RIGHT" && !canonical_right(rule) {
            cx.rule(rule, "E-RIGHTASSOC-NONCANONICAL", format!("rule {name} uses @rightAssoc but body is not canonical: expected Base {{ Op {name} }}"),
                format!("Rewrite right-assoc rule as Base {{ op {name} }}."));
        }
    }
    let longest = rule
        .annotations
        .iter()
        .filter(|a| matches!(a, Annotation::LongestChoice { .. }))
        .count();
    let predictive = rule
        .annotations
        .iter()
        .filter(|a| matches!(a, Annotation::PredictiveChoice { .. }))
        .count();
    for (count, annotation, prefix, dispatch) in [
        (
            longest,
            "longestChoice",
            "E-LONGEST-CHOICE",
            "Move longest matching to a separate dispatch rule.",
        ),
        (
            predictive,
            "predictiveChoice",
            "E-PREDICTIVE-CHOICE",
            "Move predictive dispatch to a separate rule.",
        ),
    ] {
        if count == 0 {
            continue;
        }
        if count > 1 {
            cx.rule(
                rule,
                &format!("{prefix}-DUPLICATE"),
                format!("rule {name} has duplicate @{annotation} annotations"),
                format!("Keep a single @{annotation} annotation."),
            );
        }
        if rule.body.alternatives.len() < 2 {
            cx.rule(
                rule,
                &format!("{prefix}-SHAPE"),
                format!("rule {name} uses @{annotation} without multiple alternatives"),
                format!("Use @{annotation} only on a rule with at least two '|' alternatives."),
            );
        }
        if assoc != "NONE" {
            cx.rule(
                rule,
                &format!("{prefix}-ASSOC"),
                format!("rule {name} combines @{annotation} with associativity rewriting"),
                dispatch,
            );
        }
    }
    if longest > 0 && predictive > 0 {
        cx.rule(
            rule,
            "E-PREDICTIVE-CHOICE-CONFLICT",
            format!("rule {name} combines @predictiveChoice with @longestChoice"),
            "Choose ordered predictive dispatch or longest-match dispatch.",
        );
    }
    let levels = levels(rule);
    if levels.len() > 1 {
        cx.rule(
            rule,
            "E-PRECEDENCE-DUPLICATE",
            format!("rule {name} has duplicate @precedence annotations"),
            "Keep a single @precedence(level=...) annotation.",
        );
    }
    if !levels.is_empty() && assoc == "NONE" {
        cx.rule(
            rule,
            "E-PRECEDENCE-NO-ASSOC",
            format!("rule {name} uses @precedence but has no @leftAssoc/@rightAssoc"),
            "Add one associativity annotation alongside @precedence.",
        );
    }
}

fn contains_repeat(body: &ResolvedBody) -> bool {
    body.alternatives.iter().flatten().any(|element| {
        matches!(element.atom, ResolvedAtom::Repeat { .. })
            || nested_body(&element.atom).is_some_and(contains_repeat)
    })
}

fn canonical_right(rule: &ResolvedRule) -> bool {
    let Some(top) = single_sequence(&rule.body) else {
        return false;
    };
    if top.len() != 2 {
        return false;
    }
    let ResolvedAtom::Repeat { body, .. } = &top[1].atom else {
        return false;
    };
    let Some(repeat) = single_sequence(body) else {
        return false;
    };
    repeat.len() == 2
        && matches!(repeat[1].atom, ResolvedAtom::Ref { symbol, .. } if symbol == rule.id)
}

pub(super) fn validate_topology(cx: &mut Context<'_>) {
    let grammar = cx.grammar;
    let mut associations = BTreeMap::new();
    for rule in &grammar.rules {
        let kind = assoc(rule);
        let level = levels(rule).last().copied();
        if let Some(level) = level.filter(|_| kind != "NONE") {
            let mut refs = Vec::new();
            references(&rule.body, &mut refs);
            for reference in refs {
                if reference == rule.id {
                    continue;
                }
                let Some(target) = grammar.rules.iter().find(|r| r.id == reference) else {
                    continue;
                };
                if assoc(target) == "NONE" {
                    continue;
                }
                if let Some(target_level) = levels(target)
                    .last()
                    .filter(|&&target_level| target_level <= level)
                {
                    cx.rule(rule, "E-PRECEDENCE-ORDER", format!("rule {} precedence {level} must be lower than referenced operator rule {} precedence {target_level}", rule.name, target.name),
                        format!("Decrease {} level or increase {} level.", rule.name, target.name));
                }
            }
        }
        if kind == "NONE" || kind == "BOTH" {
            continue;
        }
        let Some(level) = level else {
            cx.rule(
                rule,
                "E-ASSOC-NO-PRECEDENCE",
                format!(
                    "rule {} uses @{}Assoc but has no @precedence",
                    rule.name,
                    kind.to_lowercase()
                ),
                "Add @precedence(level=...) to this operator rule.",
            );
            continue;
        };
        let existing = associations.entry(level).or_insert(kind);
        if *existing != kind {
            cx.rule(
                rule,
                "E-PRECEDENCE-MIXED-ASSOC",
                format!("precedence level {level} mixes associativity: {existing} and {kind}"),
                "Use one associativity per precedence level.",
            );
        }
    }
}

fn references(body: &ResolvedBody, out: &mut Vec<SymbolId>) {
    for element in body.alternatives.iter().flatten() {
        if let ResolvedAtom::Ref { symbol, .. } = element.atom {
            push_unique(out, symbol);
        } else if let Some(body) = nested_body(&element.atom) {
            references(body, out);
        }
    }
}

fn push_unique(out: &mut Vec<SymbolId>, id: SymbolId) {
    if !out.contains(&id) {
        out.push(id);
    }
}

pub(super) fn left_recursion(cx: &mut Context<'_>) {
    let grammar = cx.grammar;
    let mut graph = BTreeMap::new();
    for rule in &grammar.rules {
        let mut refs = Vec::new();
        leftmost(&rule.body, &mut refs);
        graph.insert(rule.id, refs);
    }
    let mut cycles = BTreeSet::new();
    for rule in &grammar.rules {
        find_cycles(rule.id, &graph, &mut Vec::new(), &mut cycles);
    }
    for cycle in cycles {
        let names: Vec<_> = cycle
            .iter()
            .map(|id| {
                grammar
                    .rules
                    .iter()
                    .find(|r| r.id == *id)
                    .unwrap()
                    .name
                    .as_str()
            })
            .collect();
        cx.add(
            Some(names[0]),
            "W-LEFT-RECURSION",
            format!("left-recursive cycle detected: {}", names.join(" -> ")),
            "Rewrite using repetition (e.g. A ::= B { Op B }) or right recursion.",
        );
    }
}

fn leftmost(body: &ResolvedBody, out: &mut Vec<SymbolId>) {
    for alternative in &body.alternatives {
        for element in alternative {
            let empty = match &element.atom {
                ResolvedAtom::Ref { symbol, .. } => {
                    push_unique(out, *symbol);
                    false
                }
                ResolvedAtom::Optional { body, .. } | ResolvedAtom::Repeat { body, .. } => {
                    leftmost(body, out);
                    true
                }
                ResolvedAtom::Group { body, .. } => {
                    leftmost(body, out);
                    false
                }
                _ => false,
            };
            if !empty {
                break;
            }
        }
    }
}

fn find_cycles(
    current: SymbolId,
    graph: &BTreeMap<SymbolId, Vec<SymbolId>>,
    path: &mut Vec<SymbolId>,
    cycles: &mut BTreeSet<Vec<SymbolId>>,
) {
    if let Some(index) = path.iter().position(|&id| id == current) {
        let mut cycle = path[index..].to_vec();
        cycle.push(current);
        cycles.insert(cycle);
        return;
    }
    let Some(next) = graph.get(&current) else {
        return;
    };
    path.push(current);
    for &next in next {
        find_cycles(next, graph, path, cycles);
    }
    path.pop();
}
