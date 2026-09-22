use super::ast::Ast;
pub use super::rt::cursor::State;
pub use super::rt::scope::{Decl, Diagnostic as ScopeDiagnostic, Reference, Severity};
pub type Span = [usize; 2];
#[derive(Clone, Copy, Debug)]
pub struct ParseOptions {
    pub build_ast: bool,
    pub require_eof: bool,
    pub lexical: bool,
    /// Maximum active rule calls; native stack budget may stop earlier.
    pub max_depth: usize,
    pub memo: bool,
    /// Predictive exclusion of ordered-choice alternatives that provably fail at
    /// their first token (design v1 §3.4). false is the reference path that tries
    /// every alternative; both produce the same result and diagnostics.
    pub predict: bool,
    /// Left factoring of ordered-choice alternatives that share a common prefix
    /// (design v1 §3.4): the shared prefix is evaluated once and the continuations
    /// are tried in declaration order. false is the reference path that evaluates
    /// every alternative from its start; both produce the same result and diagnostics.
    pub factor: bool,
    /// Reuse the session's large buffers (memo table, diagnostics arena, event arena)
    /// across parses on the same thread instead of allocating them per parse. The
    /// retained set is bounded (`POOL_RETAIN_BYTES`) and only affects allocation
    /// behaviour, never results.
    pub reuse_buffers: bool,
    /// Per-position lexical cache: a combined DFA over the grammar's literal terminals is
    /// run once per input position and its result (the set of literals that match there)
    /// is kept for that position, together with the trivia policies' `next_non_trivia`
    /// table. Every terminal is a pure predicate on (input, position), so the cache cannot
    /// change any parse or diagnostic; false is the reference path that scans characters at
    /// every attempt. Both produce the same result and diagnostics.
    pub lexical_cache: bool,
    pub limits: ResourceLimits,
}
/// Per-session hard limits, checked before indexing input or retaining data.
#[derive(Clone, Copy, Debug)]
pub struct ResourceLimits {
    pub input_bytes: usize,
    pub arena_entries: usize,
    pub diagnostics: usize,
    pub memo_entries: usize,
    pub effect_entries: usize,
    pub effect_bytes: usize,
    pub mapping_depth: usize,
}
impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            input_bytes: 16 * 1024 * 1024,
            arena_entries: 2_000_000,
            diagnostics: 2_000_000,
            memo_entries: 1_000_000,
            effect_entries: 1_000_000,
            effect_bytes: 64 * 1024 * 1024,
            mapping_depth: 512,
        }
    }
}
impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            build_ast: true,
            require_eof: true,
            lexical: false,
            max_depth: 10_000,
            memo: true,
            predict: true,
            factor: true,
            reuse_buffers: true,
            lexical_cache: false,
            limits: ResourceLimits::default(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capture {
    pub rule_completion_order: usize,
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
/// Retained mapped value span, addressed by a canonical AST JSON pointer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueSpan {
    pub path: String,
    pub span: Span,
    pub text: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Syntax,
    TrailingInput,
    Mapping,
    Resource,
    Recovery,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub offset_cp: usize,
    pub farthest_cp: usize,
    pub expected: Vec<&'static str>,
    pub farthest_expected: Vec<&'static str>,
    pub deepest_rule: Option<&'static str>,
    pub rule_path: Vec<&'static str>,
    pub message: Option<String>,
    pub recovery_id: Option<usize>,
    pub length_cp: usize,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub rule_evaluations: usize,
    pub memo_hits: usize,
    pub memo_entries: usize,
    pub recipes: usize,
    pub ast_nodes: usize,
    pub memo_max_probe: usize,
    /// Number of combined-DFA runs (lexical cache misses; one per newly touched position).
    pub lexical_runs: u64,
    /// Number of terminal matches answered from the lexical cache (hits + misses).
    pub lexical_probes: u64,
    /// `next_non_trivia` table hits / misses (diagnostic-free trivia skipping).
    pub trivia_skip_hits: u64,
    pub trivia_skip_misses: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ParseResult {
    pub ok: bool,
    pub consumed_cp: usize,
    pub matched_cp: usize,
    pub ast: Option<Ast>,
    pub captures: Vec<Capture>,
    /// Rule occurrences, retained for frontend projection compatibility.
    pub lexical: Vec<LexicalOccurrence>,
    /// Token occurrences; parents address the rule table, IDs follow rule IDs.
    pub tokens: Vec<LexicalOccurrence>,
    pub node_spans: Vec<NodeSpan>,
    pub value_spans: Vec<ValueSpan>,
    pub diagnostics: Vec<Diagnostic>,
    /// Local farthest failures, including speculative failures on successful parses.
    pub hints: Vec<Diagnostic>,
    pub recoveries: Vec<Recovery>,
    pub declarations: Vec<Decl>,
    pub references: Vec<Reference>,
    pub scope_diagnostics: Vec<ScopeDiagnostic>,
    pub statistics: Statistics,
    pub scope: ScopeResult,
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

/// Events retained by the selected parse, in execution order. Offsets are code points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeEvent {
    pub order: usize,
    pub action: &'static str,
    pub mode: &'static str,
    pub name: Option<String>,
    pub offset_cp: usize,
    pub length_cp: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeResult {
    pub depth: usize,
    /// Grammar modes in declaration order (a grammar may mix lexical and dynamic effects).
    pub mode: Vec<&'static str>,
    pub events: Vec<ScopeEvent>,
}

/// Adopted recovery regions. The associated typed node is omitted, never fabricated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recovery {
    pub rule_id: &'static str,
    pub mode: &'static str,
    pub span: Span,
    pub sync_span: Option<Span>,
    pub capture_occurrence_ids: Vec<usize>,
}
