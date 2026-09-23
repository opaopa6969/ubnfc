use std::collections::BTreeSet;

use crate::resolve::{ResolvedAtom, ResolvedBody, ResolvedRule};
use crate::syntax::ast::{GlobalSetting, TokenValue};

use super::{nested_body, trim, Context};

pub(super) fn validate(cx: &mut Context<'_>) {
    let grammar = cx.grammar;
    let whitespace: Vec<_> = grammar
        .settings
        .iter()
        .filter(|s| key(s) == "whitespace")
        .collect();
    if whitespace.len() > 1 {
        cx.add(
            None,
            "E-WHITESPACE-GLOBAL-DUPLICATE",
            "duplicate global @whitespace settings",
            "Keep a single global @whitespace setting.",
        );
    }
    for setting in whitespace {
        let message = match setting {
            GlobalSetting::String { value, .. } if valid_whitespace(trim(value)) => continue,
            GlobalSetting::String { value, .. } => format!(
                "global @whitespace style must be javaStyle or none: {}",
                trim(value)
            ),
            GlobalSetting::Block { .. } => "global @whitespace requires a style name".into(),
        };
        cx.add(
            None,
            "E-WHITESPACE-GLOBAL-STYLE",
            message,
            "Use '@whitespace: javaStyle' or '@whitespace: none'.",
        );
    }
    let mut seen = BTreeSet::new();
    for setting in &grammar.settings {
        if key(setting) != "memoSafeToken" {
            continue;
        }
        let GlobalSetting::String { value, .. } = setting else {
            cx.add(
                None,
                "E-MEMO-SAFE-TOKEN-VALUE",
                "global @memoSafeToken requires a token alias string",
                "Use one setting per alias, for example '@memoSafeToken: IDENTIFIER'.",
            );
            continue;
        };
        let alias = trim(value);
        if !seen.insert(alias) {
            cx.add(
                None,
                "E-MEMO-SAFE-TOKEN-DUPLICATE",
                format!("duplicate global @memoSafeToken alias: {alias}"),
                format!("Keep a single @memoSafeToken setting for alias {alias}."),
            );
            continue;
        }
        match grammar.tokens.iter().find(|t| t.name == alias) {
            None => cx.add(
                None,
                "E-MEMO-SAFE-TOKEN-UNDEFINED",
                format!("global @memoSafeToken references undefined token alias: {alias}"),
                format!("Declare token {alias} or remove this @memoSafeToken setting."),
            ),
            Some(t)
                if !matches!(
                    t.declaration,
                    TokenValue::ClassRef { .. }
                        | TokenValue::Builtin { .. }
                        | TokenValue::Extern { .. }
                ) =>
            {
                cx.add(
                    None,
                    "E-MEMO-SAFE-TOKEN-KIND",
                    format!("global @memoSafeToken alias must name a Simple token: {alias}"),
                    "Remove the setting; built-in token declarations are already proven safe.",
                )
            }
            _ => {}
        }
    }
    if grammar.root.is_none()
        && (grammar.rules.is_empty() || !grammar.rules.iter().all(|r| r.name.contains('.')))
    {
        cx.add(
            None,
            "W-GENERAL-NO-ROOT",
            format!("grammar {} has no @root rule", grammar.name),
            "Add @root to at least one entry rule.",
        );
    }
    // v2 は「文法に host のクラス名を書かない」形式なので、旧綴りが残っていることを
    // 知らせる。**警告どまりで拒否しない**理由は D-042（移行費用と `--strict`）。
    // v1 では旧綴りが唯一の綴りなので何も言わない（v1 の出力は 1 バイトも変えない）。
    let v2 = declares_v2(grammar);
    // D-043: 未知の grammar 設定は `grammarUnits[].annotations` へ素通りする。
    // 綴り間違いを黙って metadata にしないよう警告する（拒否はしない）。
    // **設定だけは v2 に閉じる**: 未知設定の黙認は旧 Java 検証器と揃えた明示の
    // 前方互換契約（`tests/fixtures/validation-java.json` の
    // `testUnknownGlobalSettingCompatibilityIsUnchanged`）で、v1 ではこれを崩さない。
    // 未知**注釈**のほうは同じ契約を持たず、実際に綴り間違い（`@rooted`）を隠していたので
    // 版に関係なく警告する。
    if v2 {
        for setting in &grammar.settings {
            let key = key(setting);
            if !KNOWN_SETTINGS.contains(&key) {
                cx.add(
                    None,
                    "W-ANNOTATION-UNKNOWN",
                    format!("grammar {} has an unknown setting: @{key}", grammar.name),
                    format!(
                        "Remove @{key} or use a known setting ({}).",
                        KNOWN_SETTINGS.join(", ")
                    ),
                );
            }
        }
    }
    for token in &grammar.tokens {
        if v2 {
            if let TokenValue::ClassRef { dotted, .. } = &token.declaration {
                cx.add(
                    None,
                    "W-TOKEN-HOST-CLASS",
                    format!(
                        "token {} names a host-language class: {dotted}",
                        token.name
                    ),
                    "Use the neutral builtin vocabulary (for example 'token NUMBER = NUMBER'), or declare the host implementation explicitly with extern '<id>'.",
                );
            }
        }
        if token.unresolved {
            if let TokenValue::ClassRef { dotted, .. } = &token.declaration {
                cx.add(None, "W-TOKEN-UNRESOLVED", format!("token {} references unresolved parser class: {dotted}", token.name),
                    "Ensure the parser class is fully qualified or in a known package (org.unlaxer.parser.clang, elementary, posix).");
            }
        }
        let reference = match &token.declaration {
            TokenValue::ClassRef { dotted, .. } | TokenValue::Extern { name: dotted, .. }
                if trim(dotted).is_empty() =>
            {
                continue
            }
            TokenValue::ClassRef { dotted, .. } | TokenValue::Extern { name: dotted, .. }
                if !dotted.contains('.') =>
            {
                dotted.clone()
            }
            TokenValue::ClassRef { .. }
            | TokenValue::Extern { .. }
            | TokenValue::Builtin { .. }
            | TokenValue::Until { .. }
            | TokenValue::Negation { .. }
            | TokenValue::CharRange { .. }
            | TokenValue::Regex { .. }
            | TokenValue::CaseInsensitive { .. } => parser_name(&token.name),
            _ => continue,
        };
        if let Some(rule) = grammar
            .rules
            .iter()
            .rev()
            .find(|r| format!("{}Parser", r.name) == reference)
        {
            cx.rule(
                rule,
                "E-RULE-TOKEN-NAME-COLLISION",
                format!(
                    "rule {} and token {} both resolve to generated parser class {reference}",
                    rule.name, token.name
                ),
                "Rename the rule or token so their generated parser class names differ.",
            );
        }
    }
    for rule in &grammar.rules {
        bounded(rule, &rule.body, cx);
    }
}

fn bounded(rule: &ResolvedRule, body: &ResolvedBody, cx: &mut Context<'_>) {
    for element in body.alternatives.iter().flatten() {
        if let ResolvedAtom::Bounded {
            min,
            max: Some(max),
            ..
        } = &element.atom
        {
            if min > max {
                cx.rule(
                    rule,
                    "E-BOUNDED-INVALID",
                    format!(
                        "rule {} has BoundedRepeatElement with min={min} > max={max}",
                        rule.name
                    ),
                    "Ensure min <= max in {min,max} quantifier.",
                );
            }
        } else if let Some(body) = nested_body(&element.atom) {
            bounded(rule, body, cx);
        }
    }
}

/// `@ubnf: 2` 以上を宣言しているか（D-038）。
pub(super) fn declares_v2(grammar: &crate::resolve::Grammar) -> bool {
    grammar.settings.iter().any(|s| match s {
        GlobalSetting::String { key, value, .. } if key == "ubnf" => {
            trim(value).parse::<u64>().is_ok_and(|version| version >= 2)
        }
        _ => false,
    })
}

/// front が意味を与える grammar 設定。ここに無いキーは黙って metadata になる（D-043）。
pub(super) const KNOWN_SETTINGS: &[&str] = &[
    "package",
    "whitespace",
    "interleave",
    "comment",
    "memoSafeToken",
    "ubnf",
    "literalBoundary",
];

fn key(setting: &GlobalSetting) -> &str {
    match setting {
        GlobalSetting::String { key, .. } | GlobalSetting::Block { key, .. } => key,
    }
}

pub(super) fn valid_whitespace(style: &str) -> bool {
    style.eq_ignore_ascii_case("javaStyle") || style.eq_ignore_ascii_case("none")
}

fn parser_name(token: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for c in token.chars() {
        if c == '_' {
            capitalize = true;
        } else if capitalize {
            result.extend(c.to_uppercase());
            capitalize = false;
        } else {
            result.extend(c.to_lowercase());
        }
    }
    result.push_str("Parser");
    result
}
