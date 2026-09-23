//! `ubnf.ubnf` の IR から生成した UBNF フロントエンド。

// 再生成の byte 一致を保つため生成物は rustfmt の対象外。
#[rustfmt::skip]
pub mod generated;

mod capture_sites;
mod projection;

use ubnfc_front::selfhost::{self as convert, ConversionError, SourceOffsets};
use ubnfc_front::syntax::ast::{Span, UbnfFile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelfhostError {
    Syntax { span: Span, expected: Vec<String> },
    Mapping { message: String },
    Conversion(ConversionError),
}
impl SelfhostError {
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::Syntax { span, .. } => Some(*span),
            Self::Conversion(error) => Some(error.span),
            Self::Mapping { .. } => None,
        }
    }
}
impl std::fmt::Display for SelfhostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax { span, expected } => write!(
                f,
                "構文エラー (bytes {}..{}): expected {}",
                span.start,
                span.end,
                expected.join(" or ")
            ),
            Self::Mapping { message } => write!(f, "生成 AST の mapping エラー: {message}"),
            Self::Conversion(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for SelfhostError {}
impl From<ConversionError> for SelfhostError {
    fn from(error: ConversionError) -> Self {
        Self::Conversion(error)
    }
}

/// UBNF 自身のコメント構文は `//` 行コメントだけで、`/* ... */` は無い
/// （`unlaxer-parser/unlaxer-dsl/specs/ubnf-syntax.md`「UBNF ファイル自体のコメント構文は
/// `//` のみで、`/* ... */` は未対応です」。unlaxer-ubnf と Java bootstrap も拒否する）。
///
/// ところが自己記述文法 `unlaxer-dsl/grammar/ubnf.ubnf:18` は `@whitespace: javaStyle` を
/// 宣言しており、javaStyle の区切り集合には旧 Java 実装の挙動としてブロックコメントが入る
/// （`corpus/oracle/comment-whitespace.json` の `expected` が `'/*'` / `'*/'` を含む）。
/// つまり **自己記述文法は UBNF より広い言語を記述している**。`ubnf.ubnf` は読み取り専用の
/// 別 repo にあり、javaStyle の中身は corpus が固定しているので、どちらも動かせない。
/// そこで「生成 frontend も UBNF を実装する」という契約のほうを、この 1 箇所で満たす。
///
/// 戻り値は最初の `/*` の開始 byte 位置。手書き lexer は**解析の前に入力全体を走査する**ので、
/// 字句エラーである `/*` は構文エラーより必ず手前で報告される。一方でそれより手前に別の
/// 字句エラー（未閉の文字列・使用できない文字）があればそちらが勝つ。この関数は手書き lexer と
/// 同じ規則で走査し、**最初の字句エラーが `/*` のときだけ** `Some` を返す。他の字句エラーは
/// 生成パーサ自身が同じ位置で報告するので `None` を返して任せる。
fn block_comment_start(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        match bytes[offset] {
            b'/' if bytes.get(offset + 1) == Some(&b'*') => return Some(offset),
            b'/' if bytes.get(offset + 1) == Some(&b'/') => {
                offset += source[offset..]
                    .find(['\r', '\n'])
                    .unwrap_or(bytes.len() - offset);
            }
            quote @ (b'\'' | b'"') => {
                offset += 1;
                while offset < bytes.len() && bytes[offset] != quote {
                    // `\'` / `\"` / `\\` は閉じ引用符にならない。手書き lexer と同じ。
                    offset += if bytes[offset] == b'\\' { 2 } else { 1 };
                }
                // 未閉の文字列は手書き lexer がそこで失敗する（= `/*` より手前の字句エラー）。
                if offset >= bytes.len() {
                    return None;
                }
                offset += 1;
            }
            b' ' | b'\t' | b'\r' | b'\n' | 0x0b | 0x0c => offset += 1,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => offset += 1,
            b'@' | b'|' | b';' | b'(' | b')' | b'[' | b']' | b'{' | b'}' | b',' | b'=' | b':'
            | b'.' | b'+' | b'?' | b'*' | b'%' | b'<' | b'>' => offset += 1,
            // 手書き lexer が「使用できない文字」として拒否する位置。`/*` より手前なので
            // こちらが勝つ。位置は生成パーサも同じ場所を指すため、そのまま任せる。
            _ => return None,
        }
    }
    None
}

/// 生成パーサのみで解析する。手書き syntax へのフォールバックはない。
///
/// ブロックコメントだけは生成パーサの外で拒否する（`block_comment_start` の説明を参照）。
/// 再解析ではなく字句的な門で、どちらの診断も「エラー」なので受理判定は一意に決まる。
/// 位置は手書き lexer と揃う（`block_comment_start` の説明を参照）。
pub fn parse_file(source: &str) -> Result<UbnfFile, SelfhostError> {
    let result = generated::parse_with_options(
        source,
        generated::ParseOptions {
            lexical: true,
            ..Default::default()
        },
    );
    let offsets = SourceOffsets::new(source);
    if let Some(start) = block_comment_start(source) {
        return Err(SelfhostError::Syntax {
            span: Span {
                start,
                end: start + 1,
            },
            expected: ["identifier", "single-quoted string", "punctuation"]
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
        });
    }
    if !result.ok {
        // UBNFFile は GrammarDecl の反復なので失敗時 C は通常0。構文失敗の
        // 主経路の失敗位置 (hints の offset_cp) を使い、trailing-input の
        // 巻戻し位置と混同しない。farthest_cp は D-014 の表示用最遠観測で、
        // 未閉の文字列では EOF を指すため手書き lexer の開始位置と一致しない。
        let diagnostic = syntax_diagnostic(&result);
        let span = syntax_error_span(source, &offsets, failure_cp(&result))?;
        return Err(SelfhostError::Syntax {
            span,
            expected: diagnostic.map_or_else(Vec::new, |d| {
                d.expected.iter().map(|s| (*s).to_owned()).collect()
            }),
        });
    }
    let ast = result.ast.as_ref().ok_or_else(|| SelfhostError::Mapping {
        message: result
            .diagnostics
            .iter()
            .filter_map(|d| d.message.as_deref())
            .collect::<Vec<_>>()
            .join("; "),
    })?;
    // 生成物の occurrence 数を上限の目安にして push の再確保を避ける（実測: 764 node / 1,286 occurrence）。
    let mut nodes = Vec::with_capacity(result.lexical.len());
    projection::project(ast, &mut nodes);
    let captures = result
        .captures
        .iter()
        .map(|capture| {
            let rule_id =
                capture_sites::rule_for(capture.site_id).ok_or_else(|| SelfhostError::Mapping {
                    message: format!("未知の capture site: {}", capture.site_id),
                })?;
            Ok(convert::Capture {
                rule_id,
                name: capture.name,
                span_cp: capture.span,
                completion_order: capture.completion_order,
            })
        })
        .collect::<Result<Vec<_>, SelfhostError>>()?;
    let occurrences = result
        .lexical
        .iter()
        .map(|o| convert::Occurrence {
            id: o.occurrence_id,
            parent: o.parent_occurrence_id,
            rule_id: o.rule_id,
            expr_id: o.expr_id,
            span_cp: o.span,
            completion_order: o.completion_order,
        })
        .collect::<Vec<_>>();
    Ok(convert::convert(source, &nodes, &captures, &occurrences)?)
}

/// 構文失敗の主経路の失敗位置を返す診断（失敗時のみ意味がある）。
fn syntax_diagnostic(result: &generated::ParseResult) -> Option<&generated::Diagnostic> {
    result
        .hints
        .iter()
        .chain(&result.diagnostics)
        .filter(|d| d.kind == generated::DiagnosticKind::Syntax)
        .max_by_key(|d| d.offset_cp)
        .or_else(|| result.diagnostics.last())
}

fn failure_cp(result: &generated::ParseResult) -> usize {
    syntax_diagnostic(result).map_or(result.consumed_cp, |d| d.offset_cp)
}

fn syntax_error_span(
    source: &str,
    offsets: &SourceOffsets<'_>,
    cp: usize,
) -> Result<Span, ConversionError> {
    let start = offsets.byte_span([cp, cp])?.start;
    let total = source.chars().count();
    let end_cp = if cp == total {
        cp
    } else if source[start..].starts_with('\'') && single_quote_is_unclosed(&source[start..]) {
        total
    } else {
        cp + 1
    };
    offsets.byte_span([cp, end_cp])
}

fn single_quote_is_unclosed(source: &str) -> bool {
    let mut escaped = false;
    for ch in source.chars().skip(1) {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '\'' {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn capture_site_metadata_matches_unmodified_golden() {
        let ir = ubnfc_front::ir::from_json(include_str!(
            "../../../ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json"
        ))
        .unwrap();
        let actual = ir
            .semantics
            .captures
            .iter()
            .map(|c| (c.site_id.as_str(), c.rule_id.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(super::capture_sites::CAPTURE_RULES, actual);
    }
}
