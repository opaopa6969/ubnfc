//! ASCII の識別子・記号と UTF-8 の文字列を走査する手書き lexer。

use super::{Span, SyntaxError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind<'a> {
    Identifier(&'a str),
    Digits(&'a str),
    String {
        value: String,
        raw: &'a str,
    },
    /// examples/tinycalc.ubnf の設定値専用。通常の STRING とは区別する。
    SettingString(String),
    Define,
    Punct(char),
    Eof,
}

#[derive(Debug, Clone)]
pub(crate) struct Token<'a> {
    pub kind: Kind<'a>,
    pub span: Span,
}

pub(crate) fn lex(source: &str) -> Result<Vec<Token<'_>>, SyntaxError> {
    let bytes = source.as_bytes();
    let mut offset = 0;
    let mut tokens = Vec::new();
    while offset < bytes.len() {
        let start = offset;
        let byte = bytes[offset];
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 0x0b | 0x0c) {
            offset += 1;
            continue;
        }
        if source[offset..].starts_with("//") {
            offset += source[offset..]
                .find(['\r', '\n'])
                .unwrap_or(bytes.len() - offset);
            continue;
        }
        let kind = match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                offset += 1;
                while bytes
                    .get(offset)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
                {
                    offset += 1;
                }
                Kind::Identifier(&source[start..offset])
            }
            b'0'..=b'9' => {
                offset += 1;
                while bytes.get(offset).is_some_and(u8::is_ascii_digit) {
                    offset += 1;
                }
                Kind::Digits(&source[start..offset])
            }
            b'\'' => {
                let (value, end) = string(source, start, '\'')?;
                offset = end;
                Kind::String {
                    value,
                    raw: &source[start..end],
                }
            }
            b'"' => {
                let (value, end) = string(source, start, '"')?;
                offset = end;
                Kind::SettingString(value)
            }
            b':' if source[offset..].starts_with("::=") => {
                offset += 3;
                Kind::Define
            }
            // '<' / '>' はパラメータ化ルール（D-043）専用。v1 ではこの 2 文字は
            // どの位置でも構文エラーだったので、字句として認めても v1 の受理は
            // 1 つも増えない（構文解析が必ず拒否する）。
            b'@' | b'|' | b';' | b'(' | b')' | b'[' | b']' | b'{' | b'}' | b',' | b'=' | b':'
            | b'.' | b'+' | b'?' | b'*' | b'%' | b'<' | b'>' => {
                offset += 1;
                Kind::Punct(char::from(byte))
            }
            _ => {
                let unexpected = source[offset..].chars().next().unwrap();
                return Err(SyntaxError {
                    message: if source[offset..].starts_with("/*") {
                        "UBNF のコメントは // 行コメントのみです".into()
                    } else {
                        format!("使用できない文字: {unexpected:?}")
                    },
                    span: Span {
                        start,
                        end: start + unexpected.len_utf8(),
                    },
                    expected: vec!["identifier", "single-quoted string", "punctuation"],
                });
            }
        };
        tokens.push(Token {
            kind,
            span: Span { start, end: offset },
        });
    }
    tokens.push(Token {
        kind: Kind::Eof,
        span: Span {
            start: offset,
            end: offset,
        },
    });
    Ok(tokens)
}

fn string(source: &str, start: usize, quote: char) -> Result<(String, usize), SyntaxError> {
    let mut value = String::new();
    let mut escaped = false;
    for (relative, ch) in source[start + 1..].char_indices() {
        if escaped {
            match ch {
                'n' => value.push('\n'),
                't' => value.push('\t'),
                'r' => value.push('\r'),
                '\\' | '\'' => value.push(ch),
                '"' if quote == '"' => value.push(ch),
                _ => {
                    // 既存 Rust lexer の未知エスケープは '\\' と次の文字を保持する。
                    // REGEX の \d や fixture の \q を壊さない互換動作。
                    value.push('\\');
                    value.push(ch);
                }
            }
            escaped = false;
        } else {
            match ch {
                '\\' => escaped = true,
                ch if ch == quote => return Ok((value, start + 1 + relative + 1)),
                _ => value.push(ch),
            }
        }
    }
    Err(SyntaxError {
        message: format!("{quote} で始まる文字列が閉じられていません"),
        span: Span {
            start,
            end: source.len(),
        },
        expected: vec![if quote == '\'' { "'" } else { "\"" }],
    })
}
