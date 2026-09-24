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
    // D-045: 未知の grammar 設定は `grammarUnits[].annotations` へ素通りする。
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
        let v2 = declares_v2(grammar);
        match &token.declaration {
            TokenValue::CharRange {
                min_raw, max_raw, ..
            } => {
                let min = escapes(&token.name, "minimum", min_raw, v2, cx);
                let max = escapes(&token.name, "maximum", max_raw, v2, cx);
                if let (Some(min), Some(max)) = (min, max) {
                    char_range(&token.name, &min, &max, v2, cx);
                }
            }
            TokenValue::Negation { chars_raw, .. } => {
                escapes(&token.name, "exclusion set", chars_raw, v2, cx);
            }
            _ => {}
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

/// `CHAR_RANGE` の境界と `NEGATION` の除外集合に書かれた `\uXXXX` / `\u{X…}` の検査（D-062）。
///
/// 返すのは**展開後のリテラル**。診断を出したときは `None` を返し、後続の境界検査を
/// 行わない（同じ 1 つの誤りで 2 つ以上の診断を出さない）。
///
/// v1 では `\u` はエスケープではなく「`\` と `u` の 2 文字」であり、それが v1 の
/// 受理される書き方だった（仕様 `ubnf-syntax.md`「表にないエスケープ列はエラーにしない」）。
/// v2 で意味が変わるので、v1 側は `E-TOKEN-ESCAPE-V2-ONLY` で**明示的に拒否**する
/// （D-038 の「v1 は凍結」を守るには、v1 で黙って別の意味に読む道を塞ぐのが確実）。
/// 実在の v1 文法でこの綴りを使っているものは 1 つも無い（D-062 の調査）。
fn escapes(name: &str, label: &str, raw: &str, v2: bool, cx: &mut Context<'_>) -> Option<String> {
    if !crate::escape::has_unicode_escape(raw) {
        return Some(crate::escape::expand(raw).unwrap_or_else(|_| raw.to_owned()));
    }
    if !v2 {
        cx.add(
            None,
            "E-TOKEN-ESCAPE-V2-ONLY",
            format!(
                "token {name} uses a \\u escape in its {label} but the grammar does not declare '@ubnf: 2'"
            ),
            "Declare '@ubnf: 2' to use \\uXXXX / \\u{X...} escapes in CHAR_RANGE boundaries and NEGATION exclusion sets, or spell the character literally.",
        );
        return None;
    }
    match crate::escape::expand(raw) {
        Ok(value) => Some(value),
        Err(error) => {
            let (code, hint) = match error.code {
                crate::escape::EscapeErrorKind::Surrogate => (
                    error.code.code(),
                    "A lone surrogate is not a code point; write the character itself or the code point it forms with its pair.",
                ),
                crate::escape::EscapeErrorKind::Malformed => (
                    error.code.code(),
                    "Write \\uXXXX with exactly four hex digits, or \\u{X...} with one to six hex digits up to 10FFFF.",
                ),
            };
            cx.add(
                None,
                code,
                format!(
                    "token {name} has an invalid unicode escape in its {label}: {}",
                    error.text
                ),
                hint,
            );
            None
        }
    }
}

/// `CHAR_RANGE('min','max')` の境界検査。
///
/// 正本は `unlaxer-parser/unlaxer-dsl/specs/ubnf-syntax.md`
/// 「各境界はエスケープ展開後に1個の非 surrogate BMP 文字で、`min <= max` でなければならない。
/// 空文字・複数文字・surrogate・補助文字・逆順の範囲は明示的に拒否し、先頭文字へ切り捨てない」
/// と `docs/design/02-facts-tokens-bench-corpus.md` §A（`char` 境界、両端含む）。
/// 参照実装 `rust/unlaxer-ubnf/src/parser.rs`（`bmp()` と "minimum exceeds maximum"）と
/// Java bootstrap も同じ 3 条件で拒否する。
///
/// **v2 では BMP 制限だけを外す**（D-062）。`E-TOKEN-CHAR-RANGE-SUPPLEMENTARY` は
/// v1 専用になり、v2 の境界は U+0000..U+10FFFF の任意の code point 1 個を取れる。
/// 「エスケープ展開後に 1 個」「`min <= max`」「surrogate 不可」は v1 と同じまま
/// （surrogate は `crate::escape` が `E-TOKEN-ESCAPE-SURROGATE` で先に落とす）。
fn char_range(name: &str, min: &str, max: &str, v2: bool, cx: &mut Context<'_>) {
    let boundary = |label: &str, value: &str, cx: &mut Context<'_>| -> Option<u32> {
        let mut chars = value.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) if v2 || (c as u32) <= 0xffff => Some(c as u32),
            (Some(c), None) => {
                cx.add(
                    None,
                    "E-TOKEN-CHAR-RANGE-SUPPLEMENTARY",
                    format!(
                        "token {name} has a CHAR_RANGE {label} boundary outside the BMP: U+{:04X}",
                        c as u32
                    ),
                    "CHAR_RANGE boundaries are Java char values; use a BMP character or a different token construct for supplementary code points.",
                );
                None
            }
            (None, _) => {
                cx.add(
                    None,
                    "E-TOKEN-CHAR-RANGE-BOUNDARY",
                    format!("token {name} has an empty CHAR_RANGE {label} boundary"),
                    "Each CHAR_RANGE boundary must be exactly one non-surrogate BMP character after escape expansion.",
                );
                None
            }
            (Some(_), Some(_)) => {
                cx.add(
                    None,
                    "E-TOKEN-CHAR-RANGE-BOUNDARY",
                    format!(
                        "token {name} has a CHAR_RANGE {label} boundary of {} characters: {value}",
                        value.chars().count()
                    ),
                    "Each CHAR_RANGE boundary must be exactly one non-surrogate BMP character after escape expansion; the first character is not taken silently.",
                );
                None
            }
        }
    };
    // surrogate 単独は UTF-8 の Rust `char` として表現できないので、字句が
    // 生成し得るのは「空 / 1 文字 / 複数文字 / 非 BMP」の 4 つだけ。
    let min_value = boundary("minimum", min, cx);
    let max_value = boundary("maximum", max, cx);
    if let (Some(low), Some(high)) = (min_value, max_value) {
        if low > high {
            cx.add(
                None,
                "E-TOKEN-CHAR-RANGE-ORDER",
                format!("token {name} has a CHAR_RANGE minimum U+{low:04X} greater than its maximum U+{high:04X}"),
                "Write CHAR_RANGE('min','max') with min <= max; the range includes both bounds.",
            );
        }
    }
}

fn bounded(rule: &ResolvedRule, body: &ResolvedBody, cx: &mut Context<'_>) {
    for element in body.alternatives.iter().flatten() {
        if let ResolvedAtom::Bounded { min, max, .. } = &element.atom {
            if let Some(bound) = [Some(*min), *max]
                .into_iter()
                .flatten()
                .find(|v| *v > I32_MAX)
            {
                cx.rule(
                    rule,
                    "E-BOUNDED-OVERFLOW",
                    format!(
                        "rule {} has a BoundedRepeatElement bound outside the 32-bit range: {bound}",
                        rule.name
                    ),
                    format!("Use a {{min,max}} bound of at most {I32_MAX}."),
                );
            }
            if let Some(max) = max.filter(|max| min > max) {
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

/// 量指定子の上限と `@precedence(level=N)` が収まらなければならない範囲。
/// 旧実装が Java `int` で読むため（`Integer.parseInt`、`rust/unlaxer-ubnf/src/parser.rs:102-108`
/// の `parse::<i32>()` と "integer exceeds 2147483647"）、両者が拒否する値を ubnfc も拒否する。
pub(super) const I32_MAX: u64 = i32::MAX as u64;

/// `@ubnf: 2` 以上を宣言しているか（D-038）。
pub(super) fn declares_v2(grammar: &crate::resolve::Grammar) -> bool {
    crate::escape::declares_v2(&grammar.settings)
}

/// front が意味を与える grammar 設定。ここに無いキーは黙って metadata になる（D-045）。
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
