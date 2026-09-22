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

/// 生成パーサのみで解析する。手書き syntax へのフォールバックはない。
pub fn parse_file(source: &str) -> Result<UbnfFile, SelfhostError> {
    let result = generated::parse_with_options(
        source,
        generated::ParseOptions {
            lexical: true,
            ..Default::default()
        },
    );
    let offsets = SourceOffsets::new(source);
    if !result.ok {
        // UBNFFile は GrammarDecl の反復なので失敗時 C は通常0。構文失敗の
        // 主経路の失敗位置 (hints の offset_cp) を使い、trailing-input の
        // 巻戻し位置と混同しない。farthest_cp は D-014 の表示用最遠観測で、
        // 未閉の文字列では EOF を指すため手書き lexer の開始位置と一致しない。
        let diagnostic = result
            .hints
            .iter()
            .chain(&result.diagnostics)
            .filter(|d| d.kind == generated::DiagnosticKind::Syntax)
            .max_by_key(|d| d.offset_cp)
            .or_else(|| result.diagnostics.last());
        let cp = diagnostic.map_or(result.consumed_cp, |d| d.offset_cp);
        let span = syntax_error_span(source, &offsets, cp)?;
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
    let mut nodes = Vec::new();
    projection::project(ast, &mut nodes);
    let captures = result
        .captures
        .iter()
        .map(|capture| {
            let rule_id = capture_sites::CAPTURE_RULES
                .iter()
                .find(|(site, _)| *site == capture.site_id)
                .map(|(_, rule)| *rule)
                .ok_or_else(|| SelfhostError::Mapping {
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
