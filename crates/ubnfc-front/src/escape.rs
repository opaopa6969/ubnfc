//! v2 の code point エスケープ（D-063）。
//!
//! v1 のリテラルのエスケープは `\n` `\t` `\r` `\\` `\'` の 5 つだけで、表に無い列
//! （`'\q'`）は「バックスラッシュと次の文字をそのまま残す」（`syntax/lexer.rs` の
//! `string()`。仕様 `ubnf-syntax.md`「表にないエスケープ列はエラーにしない…展開結果は
//! 実装依存」）。したがって v1 では `'A'` は **6 文字**（`\` `u` `0` `0` `4` `1`）で、
//! これは受理される v1 の書き方である。
//!
//! v2 では `CHAR_RANGE(…)` の境界と `NEGATION(…)` の除外集合に限って `\uXXXX` と
//! `\u{X…}` を **code point 1 個**へ展開する（D-063）。**それ以外の文字列**
//! （rule 本体のリテラル・`UNTIL` / `CI` / `LOOKAHEAD` / `NEGATIVE_LOOKAHEAD` / `REGEX`）は
//! v1 のまま触らない。`REGEX` は正則部分集合の解析器が生の `\` 列を見るので、
//! 前段で展開すると意味が二重になる（issue #34 の候補 (c) は別問題として残す）。
//!
//! ## なぜ字句ではなく原文から展開するか
//!
//! 字句の `value` は `\\` を 1 個の `\` に畳んでしまうので、`'\\u0041'`（バックスラッシュ +
//! `u0041` の 7 文字）と `'A'` が**同じ値になる**。後段で `value` を見て `\u` を探すと
//! この 2 つを区別できない。そこで AST は原文（引用符の内側）も持ち、v2 の展開は
//! **原文を 1 度だけ走査**して行う。走査規則は v1 と同じで `\u` の枝を足しただけなので、
//! **`\u` を含まない原文では v1 の `value` と 1 文字も違わない**
//! （`crates/ubnfc-front/tests/escape.rs` が固定する）。

use crate::syntax::ast::GlobalSetting;

/// v2 エスケープの失敗。位置は原文（引用符の内側）の byte offset。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscapeError {
    /// `E-TOKEN-ESCAPE-*` の後半。
    pub code: EscapeErrorKind,
    /// 診断に出す綴り（`\uD800` など）。
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeErrorKind {
    /// `\u` の綴りが 4 桁 16 進でも `\u{X…}` でもない、または U+10FFFF を超える。
    Malformed,
    /// 値が surrogate（U+D800..U+DFFF）。単独 surrogate は code point ではない。
    Surrogate,
}

impl EscapeErrorKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::Malformed => "E-TOKEN-ESCAPE-UNICODE",
            Self::Surrogate => "E-TOKEN-ESCAPE-SURROGATE",
        }
    }
}

/// `@ubnf: 2` 以上を宣言しているか（D-038）。resolve と validate が同じ判定を使う。
pub fn declares_v2(settings: &[GlobalSetting]) -> bool {
    settings.iter().any(|s| match s {
        // 余白の刈り方は validate の `trim` と同じ（制御文字も落とす）。
        GlobalSetting::String { key, value, .. } if key == "ubnf" => {
            language_version(value).is_some_and(|v| v >= 2)
        }
        _ => false,
    })
}

/// Both the historical numeric spelling and shared UBNF `v2` spelling are accepted.
pub fn language_version(value: &str) -> Option<u64> {
    let value = value.trim_matches(|c: char| c <= '\u{20}');
    value.strip_prefix('v').unwrap_or(value).parse().ok()
}

/// 原文に `\uXXXX` / `\u{X…}` の**エスケープとして**現れる列があるか。
/// `'\\u0041'`（`\\` の直後の `u`）は false。v1 の門（`E-TOKEN-ESCAPE-V2-ONLY`）に使う。
pub fn has_unicode_escape(raw: &str) -> bool {
    scan(raw).1
}

/// v2 の展開。`\u` を含まない原文では v1 の字句値と一致する。
pub fn expand(raw: &str) -> Result<String, EscapeError> {
    scan(raw).0
}

/// 原文を 1 度走査し、展開結果と「`\u` エスケープが現れたか」を返す。
fn scan(raw: &str) -> (Result<String, EscapeError>, bool) {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut unicode = false;
    let mut i = 0;
    while i < bytes.len() {
        let ch = raw[i..].chars().next().expect("char boundary");
        if ch != '\\' {
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let Some(next) = raw[i + 1..].chars().next() else {
            // 閉じていない末尾の `\`。字句が先に「閉じられていない文字列」で落とすので
            // 到達しないが、v1 と同じく原文のまま残す。
            out.push('\\');
            i += 1;
            continue;
        };
        match next {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '\\' | '\'' => out.push(next),
            'u' => {
                unicode = true;
                match unicode_escape(raw, i) {
                    Ok((c, len)) => {
                        out.push(c);
                        i += len;
                    }
                    Err(error) => return (Err(error), true),
                }
                continue;
            }
            // v1 と同じ「未知のエスケープは `\` と次の文字を残す」。
            _ => {
                out.push('\\');
                out.push(next);
            }
        }
        i += 1 + next.len_utf8();
    }
    (Ok(out), unicode)
}

/// `raw[at..]` は `\u` で始まる。code point と消費した byte 数を返す。
fn unicode_escape(raw: &str, at: usize) -> Result<(char, usize), EscapeError> {
    let rest = &raw[at + 2..];
    let (digits, consumed) = if let Some(body) = rest.strip_prefix('{') {
        match body.find('}') {
            Some(end) => (&body[..end], 2 + 1 + end + 1),
            None => return Err(malformed(&raw[at..])),
        }
    } else if rest.len() >= 4 && rest.is_char_boundary(4) {
        (&rest[..4], 2 + 4)
    } else {
        return Err(malformed(&raw[at..]));
    };
    let text = &raw[at..at + consumed];
    if digits.is_empty() || digits.len() > 6 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(malformed(text));
    }
    let value = u32::from_str_radix(digits, 16).map_err(|_| malformed(text))?;
    if (0xd800..=0xdfff).contains(&value) {
        return Err(EscapeError {
            code: EscapeErrorKind::Surrogate,
            text: text.to_owned(),
        });
    }
    let Some(c) = char::from_u32(value) else {
        return Err(malformed(text));
    };
    Ok((c, consumed))
}

fn malformed(text: &str) -> EscapeError {
    EscapeError {
        code: EscapeErrorKind::Malformed,
        text: text.chars().take(12).collect(),
    }
}
