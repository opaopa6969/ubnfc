//! UBNF ソースから構文木を作る。名前解決・意味検証・IR への変換は行わない。

pub mod ast;
pub(crate) mod lexer;
mod parser;

pub use ast::{GrammarAst, Span, UbnfFile};

/// 最初に見つかった字句または構文エラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    pub message: String,
    pub span: Span,
    pub expected: Vec<&'static str>,
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (bytes {}..{})",
            self.message, self.span.start, self.span.end
        )?;
        if !self.expected.is_empty() {
            write!(f, "; expected {}", self.expected.join(" or "))?;
        }
        Ok(())
    }
}

impl std::error::Error for SyntaxError {}

pub fn parse_file(source: &str) -> Result<UbnfFile, SyntaxError> {
    parser::parse(source, lexer::lex(source)?)
}

/// byte offset を1始まりの行・列へ変換する。列はタブも1 code point と数える。
#[derive(Debug, Clone)]
pub struct SourceMap<'a> {
    source: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> SourceMap<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut line_starts = vec![0];
        let bytes = source.as_bytes();
        for (offset, &byte) in bytes.iter().enumerate() {
            // CRLF は一つの改行。単独の CR と LF も改行として扱う。
            if byte == b'\n' || (byte == b'\r' && bytes.get(offset + 1) != Some(&b'\n')) {
                line_starts.push(offset + 1);
            }
        }
        Self {
            source,
            line_starts,
        }
    }

    /// 範囲外は EOF、UTF-8 の途中はその文字の先頭に丸める。
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let mut offset = offset.min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset -= 1;
        }
        let line = self.line_starts.partition_point(|&start| start <= offset) - 1;
        let col = self.source[self.line_starts[line]..offset].chars().count();
        (line + 1, col + 1)
    }
}
