use crate::resolve::{ResolvedBody, ResolvedRule};
use crate::syntax::ast::Annotation;

use super::{captures, general::valid_whitespace, general::I32_MAX, nested_body, trim, Context};

/// `@recovery(mode)` で受理する mode（design v1 §3.10）。大小は無視する。
const RECOVERY_MODES: &[&str] = &["sync", "auto", "skip"];

/// 表層で受理する規則注釈の名前（parser.rs の分岐と 1:1）。要素注釈 `@typeof` は
/// ここに含めない（`Element::typeof_constraint` で別に構文化されている）。
const KNOWN_ANNOTATIONS: &[&str] = &[
    "root",
    "mapping",
    "eval",
    "whitespace",
    "interleave",
    "backref",
    "symbolRef",
    "sameText",
    "scopeTree",
    "declares",
    "catalog",
    "leftAssoc",
    "rightAssoc",
    "longestChoice",
    "predictiveChoice",
    "precedence",
    "doc",
    "recovery",
    "skip",
    "commonField",
    "enum",
    // Classic の `@autoCommit`（unlaxer-parser #702）。FIRST 集合で「入れても受理と AST が
    // 変わらない」位置にだけ COMMIT を挿入する注釈で、変わるのは失敗の位置と診断だけ。
    // ubnfc は COMMIT を持たないので、Classic が解析の対象外の文法でするのと同じく何も挿入しない。
    "autoCommit",
];

/// v2 で非推奨になった綴りの警告（D-046 / D-048 / D-049）。v1 では旧綴りが唯一の綴り
/// なので出さない（凍結した fixture・P4・corpus に無意味な警告を足さない。D-042 と同じ扱い）。
fn deprecations(rule: &ResolvedRule, cx: &mut Context<'_>) {
    let name = &rule.name;
    for annotation in &rule.annotations {
        match annotation {
            Annotation::Backref { mode: None, .. } => cx.rule(
                rule,
                "W-ANNOTATION-BACKREF-AMBIGUOUS",
                format!("rule {name} uses @backref, whose meaning depends on whether any rule in the grammar has @scopeTree"),
                "Use @symbolRef(name=...) for a scope lookup, or @sameText(name=...) for a same-rule text match.",
            ),
            Annotation::Interleave { profile, .. } => cx.rule(
                rule,
                "W-ANNOTATION-INTERLEAVE-DEPRECATED",
                format!(
                    "rule {name} uses @interleave(profile={}), which resolves to the same delimiter set as @whitespace",
                    trim(profile)
                ),
                "Use @whitespace (or @whitespace(javaStyle)) instead; @interleave is removed in a later version.",
            ),
            Annotation::PredictiveChoice { .. } => cx.rule(
                rule,
                "W-ANNOTATION-PREDICTIVE-REDUNDANT",
                format!("rule {name} uses @predictiveChoice, which the choice analysis already decides on its own"),
                "Remove @predictiveChoice; the front picks strategy=predict with proof=exclusiveCandidates where it is provable. @longestChoice is a different (semantic) choice and stays.",
            ),
            _ => {}
        }
    }
}

pub(super) fn validate(rule: &ResolvedRule, scope: bool, v2: bool, cx: &mut Context<'_>) {
    // D-045: catch-all `@<任意の識別子>` は綴り間違いを黙って metadata にする。
    // 版に関係なく警告する（受理・AST・生成コードは変えない純粋な診断の追加）。
    for annotation in &rule.annotations {
        if let Annotation::Simple { name: unknown, .. } = annotation {
            if !KNOWN_ANNOTATIONS.contains(&unknown.as_str()) {
                cx.rule(
                    rule,
                    "W-ANNOTATION-UNKNOWN",
                    format!("rule {} has an unknown annotation: @{unknown}", rule.name),
                    format!("Remove @{unknown} or fix the spelling; unknown annotations are carried as metadata and never reach code generation."),
                );
            }
        }
    }
    if v2 {
        deprecations(rule, cx);
    }
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
            // design v1 §3.10 の mode は sync / auto / skip の 3 つ。未知の綴りは
            // lower が黙って SYNC に落としていた（`lower/annotations.rs` の `_ =>`）。
            // unlaxer-ubnf も Java bootstrap も拒否するので拒否に揃える。
            Annotation::Recovery {
                mode: Some(mode), ..
            } if !RECOVERY_MODES
                .iter()
                .any(|known| trim(mode).eq_ignore_ascii_case(known)) =>
            {
                cx.rule(
                    rule,
                    "E-ANNOTATION-RECOVERY-MODE",
                    format!(
                        "rule {name} uses unsupported @recovery mode: {}",
                        trim(mode)
                    ),
                    "Use @recovery(sync), @recovery(sync='...'), @recovery(auto) or @recovery(skip).",
                );
            }
            // 旧実装は level を Java int で読む。i64 を超える値は lower の
            // E-LOWER-PRECEDENCE-RANGE が拾うが、そこまで行く前に同じ範囲で拒否する。
            Annotation::Precedence { level, .. } if *level > I32_MAX => {
                cx.rule(
                    rule,
                    "E-PRECEDENCE-OVERFLOW",
                    format!("rule {name} has @precedence(level={level}) outside the 32-bit range"),
                    format!("Use a precedence level of at most {I32_MAX}."),
                );
            }
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
            // 旧 `@backref` は scope モードのときだけ検査する（v1 の診断を変えない）。
            // 明示綴り（D-046）はどちらのモードでも capture 名を検査する。
            Annotation::Backref { name, mode, .. } if scope || mode.is_some() => {
                (name, "E-ANNOTATION-BACKREF-CAPTURE")
            }
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
