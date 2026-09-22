use super::ast::Ast;
pub use super::rt::cursor::State;
pub use super::rt::scope::{Decl, Diagnostic as ScopeDiagnostic, Reference, Severity};
pub type Span = [usize; 2];
#[derive(Clone, Copy, Debug)]
pub struct ParseOptions {
    pub build_ast: bool,
    pub require_eof: bool,
    pub lexical: bool,
}
impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            build_ast: true,
            require_eof: true,
            lexical: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capture {
    pub occurrence_id: usize,
    pub site_id: &'static str,
    pub name: &'static str,
    pub span: Span,
    pub completion_order: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexicalOccurrence {
    pub occurrence_id: usize,
    pub parent_occurrence_id: Option<usize>,
    pub rule_id: &'static str,
    pub expr_id: &'static str,
    pub span: Span,
    pub completion_order: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeSpan {
    pub node_id: usize,
    pub rule_id: &'static str,
    pub node_type: &'static str,
    pub span: Span,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Syntax,
    TrailingInput,
    Mapping,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub offset_cp: usize,
    pub farthest_cp: usize,
    pub expected: Vec<&'static str>,
    pub deepest_rule: Option<&'static str>,
    pub rule_path: Vec<&'static str>,
    pub message: Option<String>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub rule_evaluations: usize,
    pub memo_hits: usize,
    pub memo_entries: usize,
    pub recipes: usize,
    pub ast_nodes: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ParseResult {
    pub ok: bool,
    pub consumed_cp: usize,
    pub matched_cp: usize,
    pub ast: Option<Ast>,
    pub captures: Vec<Capture>,
    pub lexical: Vec<LexicalOccurrence>,
    pub node_spans: Vec<NodeSpan>,
    pub diagnostics: Vec<Diagnostic>,
    pub declarations: Vec<Decl>,
    pub references: Vec<Reference>,
    pub scope_diagnostics: Vec<ScopeDiagnostic>,
    pub statistics: Statistics,
}
#[derive(Clone, Debug)]
pub enum ScanEffect {
    Diagnostic {
        message: String,
        offset_cp: usize,
        len_cp: usize,
        severity: Severity,
    },
    Enter,
    Leave,
    Declare {
        name: String,
        offset_cp: usize,
    },
    Use {
        name: String,
        offset_cp: usize,
        len_cp: usize,
    },
    Clear,
}
#[derive(Clone, Debug)]
pub struct ScanDiagnostic {
    pub offset: usize,
    pub expected: &'static str,
}
/// offsets と value_span は byte。scanner は UTF-8 境界の単調な位置を返す。
#[derive(Clone, Debug)]
pub struct ScanResult {
    pub ok: bool,
    pub consumed_end: usize,
    pub matched_end: usize,
    pub value_span: Span,
    pub diagnostics: Vec<ScanDiagnostic>,
    pub effects: Vec<ScanEffect>,
}
pub trait TokenScanner {
    fn scan(
        &mut self,
        logical_id: &str,
        input: &str,
        state: State,
        match_only: bool,
        scope: &super::rt::scope::ScopeStore,
    ) -> ScanResult;
}
pub struct RejectExtern;
impl TokenScanner for RejectExtern {
    fn scan(
        &mut self,
        _: &str,
        _: &str,
        state: State,
        _: bool,
        _: &super::rt::scope::ScopeStore,
    ) -> ScanResult {
        ScanResult {
            ok: false,
            consumed_end: state.consumed,
            matched_end: state.matched,
            value_span: [state.consumed, state.consumed],
            diagnostics: vec![],
            effects: vec![],
        }
    }
}
