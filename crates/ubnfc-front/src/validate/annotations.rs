use crate::resolve::{ResolvedBody, ResolvedRule};
use crate::syntax::ast::Annotation;

use super::{captures, general::valid_whitespace, nested_body, trim, Context};

pub(super) fn validate(rule: &ResolvedRule, scope: bool, cx: &mut Context<'_>) {
    let name = &rule.name;
    let mut whitespace = 0;
    let mut interleave = Vec::new();
    let mut backref = 0;
    let mut scopes = Vec::new();
    let mut declares = 0;
    let mut catalog = false;
    for annotation in &rule.annotations {
        match annotation {
            Annotation::Whitespace { style, .. } => {
                whitespace += 1;
                let style = trim(style.as_deref().unwrap_or("javaStyle"));
                if !valid_whitespace(style) {
                    cx.rule(rule, "E-WHITESPACE-RULE-STYLE", format!("rule {name} uses unsupported @whitespace style: {style} (allowed: javaStyle, none)"),
                        "Use @whitespace or @whitespace(none).");
                }
            }
            Annotation::Interleave { profile, .. } => interleave.push(profile),
            Annotation::Backref { .. } => backref += 1,
            Annotation::ScopeTree { mode, .. } => scopes.push(mode),
            Annotation::Declares { .. } => declares += 1,
            Annotation::Catalog { .. } => catalog = true,
            _ => {}
        }
    }
    for (count, annotation, code, hint) in [
        (
            whitespace,
            "whitespace",
            "E-WHITESPACE-RULE-DUPLICATE",
            "Keep a single @whitespace annotation.",
        ),
        (
            interleave.len(),
            "interleave",
            "E-ANNOTATION-DUPLICATE-INTERLEAVE",
            "Keep a single @interleave(profile=...) annotation.",
        ),
        (
            backref,
            "backref",
            "E-ANNOTATION-DUPLICATE-BACKREF",
            "Keep a single @backref(name=...) annotation.",
        ),
        (
            scopes.len(),
            "scopeTree",
            "E-ANNOTATION-DUPLICATE-SCOPETREE",
            "Keep a single @scopeTree(mode=...) annotation.",
        ),
        (
            declares,
            "declares",
            "E-ANNOTATION-DUPLICATE-DECLARES",
            "Keep a single @declares(symbol=...) annotation.",
        ),
    ] {
        if count > 1 {
            cx.rule(
                rule,
                code,
                format!("rule {name} has duplicate @{annotation} annotations"),
                hint,
            );
        }
    }
    if let Some(profile) = interleave.first() {
        let profile = trim(profile);
        if profile != "javaStyle" && profile != "commentsAndSpaces" {
            cx.rule(
                rule,
                "E-ANNOTATION-INTERLEAVE-PROFILE",
                format!("rule {name} uses unsupported @interleave profile: {profile}"),
                "Use @interleave(profile=javaStyle) or @interleave(profile=commentsAndSpaces).",
            );
        }
    }
    if let Some(mode) = scopes.first() {
        let mode = trim(mode);
        if mode != "lexical" && mode != "dynamic" {
            cx.rule(
                rule,
                "E-ANNOTATION-SCOPETREE-MODE",
                format!("rule {name} uses unsupported @scopeTree mode: {mode}"),
                "Use @scopeTree(mode=lexical) or @scopeTree(mode=dynamic).",
            );
        }
    }
    let captures = captures(rule);
    for annotation in &rule.annotations {
        let (target, code) = match annotation {
            Annotation::Declares { symbol, .. } => (symbol, "E-ANNOTATION-DECLARES-CAPTURE"),
            Annotation::Backref { name, .. } if scope => (name, "E-ANNOTATION-BACKREF-CAPTURE"),
            _ => continue,
        };
        if !captures.contains(&target.as_str()) {
            cx.rule(
                rule,
                code,
                format!("rule {name} has no scope capture: {target}"),
                format!("Capture the target in this rule's body using @{target}."),
            );
        }
    }
    if catalog && captures.is_empty() {
        cx.rule(
            rule,
            "E-ANNOTATION-CATALOG-NO-CAPTURE",
            format!("rule {name} has @catalog but no capture names in its body"),
            "Add at least one @captureName in the rule body for catalog completion to work.",
        );
    }
    typeof_elements(rule, &rule.body, &captures, cx);
}

fn typeof_elements(
    rule: &ResolvedRule,
    body: &ResolvedBody,
    captures: &[&str],
    cx: &mut Context<'_>,
) {
    for element in body.alternatives.iter().flatten() {
        if let Some(target) = &element.typeof_constraint {
            if !captures.contains(&target.as_str()) {
                cx.rule(
                    rule,
                    "E-TYPEOF-UNKNOWN-CAPTURE",
                    format!(
                        "rule {}: @typeof({target}) refers to unknown capture '{target}'",
                        rule.name
                    ),
                    "Use a capture name defined in the same rule (e.g. @captureRef).",
                );
            }
            if element.capture.as_deref().is_none_or(str::is_empty) {
                cx.rule(rule, "E-TYPEOF-MISSING-CAPTURE", format!("rule {}: @typeof({target}) must be paired with a capture name (e.g. @typeof(x) @myCapture)", rule.name),
                    "Add a capture name after @typeof(x).");
            }
        } else if let Some(body) = nested_body(&element.atom) {
            typeof_elements(rule, body, captures, cx);
        }
    }
}
