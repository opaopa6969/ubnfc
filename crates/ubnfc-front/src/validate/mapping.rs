use std::collections::{BTreeMap, BTreeSet};

use crate::resolve::{Grammar, ResolvedAtom, ResolvedRule};
use crate::syntax::ast::Annotation;

use super::{captures, first_mapping, last_mapping, Context};

pub(super) fn validate_mapping(rule: &ResolvedRule, cx: &mut Context<'_>) {
    let Some((class, params)) = last_mapping(rule) else {
        return;
    };
    let name = &rule.name;
    let mut unique = Vec::new();
    let mut duplicates = Vec::new();
    for param in params {
        if unique.contains(param) {
            if !duplicates.contains(&param.as_str()) {
                duplicates.push(param.as_str());
            }
        } else {
            unique.push(param.clone());
        }
    }
    if !duplicates.is_empty() {
        cx.rule(
            rule,
            "E-MAPPING-DUPLICATE-PARAM",
            format!(
                "rule {name} @mapping({class}) has duplicate params: [{}]",
                duplicates.join(", ")
            ),
            "Remove duplicate parameter names in @mapping params.",
        );
    }
    let captures = captures(rule);
    for param in &unique {
        if !captures.contains(&param.as_str()) {
            cx.rule(
                rule,
                "E-MAPPING-MISSING-CAPTURE",
                format!("rule {name} @mapping({class}) param '{param}' has no matching capture"),
                format!("Add @{param} capture in the rule body or remove it from params."),
            );
        }
    }
    for capture in captures {
        if !unique.iter().any(|p| p == capture) {
            cx.rule(
                rule,
                "E-MAPPING-UNLISTED-CAPTURE",
                format!(
                    "rule {name} has capture @{capture} not listed in @mapping({class}) params"
                ),
                format!("Add '{capture}' to @mapping params."),
            );
        }
    }
}

pub(super) fn validate_shapes(cx: &mut Context<'_>) {
    let grammar = cx.grammar;
    for rule in &grammar.rules {
        if rule
            .annotations
            .iter()
            .any(|a| matches!(a, Annotation::Enum { .. }))
            && !rule
                .body
                .alternatives
                .iter()
                .all(|a| a.len() == 1 && matches!(a[0].atom, ResolvedAtom::Literal { .. }))
        {
            cx.rule(
                rule,
                "E-ENUM-NON-TERMINAL",
                format!(
                    "@enum rule {} must consist only of terminal literals (e.g. 'a' | 'b')",
                    rule.name
                ),
                "Use only quoted string literals as alternatives in an @enum rule.",
            );
        }
        for annotation in &rule.annotations {
            let Annotation::CommonField { fields, .. } = annotation else {
                continue;
            };
            let Some((class, _)) = first_mapping(rule) else {
                continue;
            };
            for field in fields {
                for permit in &grammar.rules {
                    let Some((permit_class, params)) = first_mapping(permit) else {
                        continue;
                    };
                    if permit_class.starts_with(&format!("{class}.")) && !params.contains(field) {
                        cx.rule(rule, "E-COMMONFIELD-MISMATCH", format!("@commonField({field}) is declared on {} but permit {permit_class} does not have param '{field}'", rule.name),
                            format!("Add @{field} capture to {permit_class} rule."));
                    }
                }
            }
        }
    }
    // MappingShape.astMappings / sumTypeNames と EvaluatorGenerator.generate の契約。
    let mut mappings = BTreeMap::new();
    let mut ordered = Vec::new();
    for rule in &grammar.rules {
        if rule
            .annotations
            .iter()
            .any(|a| matches!(a, Annotation::Skip { .. } | Annotation::Enum { .. }))
        {
            continue;
        }
        if let Some((class, _)) = first_mapping(rule) {
            if let std::collections::btree_map::Entry::Vacant(entry) = mappings.entry(class) {
                entry.insert(rule);
                ordered.push((class, rule));
            }
        }
    }
    let mut sums = BTreeSet::new();
    for &(class, rule) in &ordered {
        if let Some((owner, _)) = class.split_once('.') {
            sums.insert(owner);
        } else if is_sum(grammar, rule) {
            sums.insert(class);
        }
    }
    for (class, rule) in ordered {
        if !sums.contains(class) {
            continue;
        }
        let eval = rule.annotations.iter().find_map(|a| {
            if let Annotation::Eval { entries, .. } = a {
                Some(entries)
            } else {
                None
            }
        });
        if let Some(entries) = eval {
            // UBNFMapper は引用値の2番目を strategy とし、省略時は default。
            // 追加 params を含む場合も、この位置による判定を維持する。
            let strategy = entries.get(1).map_or("default", |e| e.value.as_str());
            if strategy != "manual" {
                cx.rule(rule, "E-EVAL-SUM-TYPE", format!("Automatic @eval requires a concrete mapping, not sum {class}; annotate its variants instead"),
                    "Annotate concrete variants or use strategy='manual'.");
            }
        }
    }
}

fn is_sum(grammar: &Grammar, rule: &ResolvedRule) -> bool {
    let Some((class, params)) = first_mapping(rule) else {
        return false;
    };
    // Java AST の ChoiceBody は 2 選択肢以上。単一参照の alias は SequenceBody。
    if !params.is_empty() || class.contains('.') || rule.body.alternatives.len() < 2 {
        return false;
    }
    rule.body.alternatives.iter().all(|alternative| {
        if alternative.len() != 1 {
            return false;
        }
        let ResolvedAtom::Ref { symbol, .. } = alternative[0].atom else {
            return false;
        };
        let Some(target) = grammar.rules.iter().find(|r| r.id == symbol) else {
            return false;
        };
        if target
            .annotations
            .iter()
            .any(|a| matches!(a, Annotation::Skip { .. }))
        {
            return false;
        }
        first_mapping(target).is_some_and(|(name, _)| {
            name != class && (!name.contains('.') || name.starts_with(&format!("{class}.")))
        })
    })
}
