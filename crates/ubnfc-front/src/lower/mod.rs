//! 解決済み文法から認識・capture・mapping の意味論を作る。
//!
//! `SemanticsIr` は analysis/plan を捏造しない、統合前の IR。出所表と警告も
//! 一緒に返す。`semantics` はそのまま `GrammarIr.semantics` に渡せる。
//! AUTO recovery の FOLLOW は未計算 (`recovery_follow_sync_patterns` の None)。
//! analysis はこれを解決してから実行用 GrammarIr を組み立てること。
mod annotations;
mod captures;
mod expr;
mod normalize;
mod recursion;
mod schema;
mod source;
mod tokens;
mod trivia;

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use crate::ir::*;
use crate::resolve::{self as r, ResolvedGrammar};
use crate::syntax::ast as a;
use serde::Serialize;

pub use source::SourceInput;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LowerError {
    Warning {
        code: String,
        message: String,
        source: Option<a::Span>,
    },
    Error {
        code: String,
        message: String,
        source: Option<a::Span>,
    },
}
impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Warning { code, message, .. } | Self::Error { code, message, .. } => {
                write!(f, "{code}: {message}")
            }
        }
    }
}
impl std::error::Error for LowerError {}
fn error(code: &str, message: impl Into<String>) -> LowerError {
    LowerError::Error {
        code: code.into(),
        message: message.into(),
        source: None,
    }
}

/// スキーマに無い静的 binding 情報。quantifier の外側の site は各 value
/// に結び付く。同じ Expr の bindings は内側→外側（完了順）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureBinding {
    pub site_id: String,
    pub value_expr_ids: Vec<ExprId>,
    pub inside_quantifier: bool,
    pub quantifier_depth: usize,
    pub completion_order: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticsIr {
    pub sources: Vec<Source>,
    pub grammar_units: Vec<GrammarUnit>,
    pub import_edges: Vec<ImportEdge>,
    pub entries: Vec<Entry>,
    pub semantics: Semantics,
    pub capture_bindings: Vec<CaptureBinding>,
    pub recovery_follow_sync_patterns: BTreeMap<String, Option<Vec<String>>>,
    pub warnings: Vec<LowerError>,
}

/// `sources` は resolve に渡したファイル（import を含む）の元 UTF-8。
/// path は resolve の unit.path、logical_path は明示したルート相対パス。
/// 検証 ERROR がある文法を実行用 semantics として返さない。
pub fn lower(
    grammar: &ResolvedGrammar,
    sources: &[SourceInput],
) -> Result<SemanticsIr, Vec<LowerError>> {
    let issues: Vec<_> = crate::validate::validate(grammar)
        .into_iter()
        .filter(|i| i.severity() == "ERROR")
        .map(|i| error(&i.code, i.message))
        .collect();
    if !issues.is_empty() {
        return Err(issues);
    }
    let mut grammar = grammar.clone();
    let warnings = normalize::normalize(&mut grammar);
    let source_table = source::Sources::new(&grammar, sources)?;
    // D-040: リテラルの語境界は文法ごとの設定。`@ubnf` 版で既定が決まる。
    let literal_boundary: Vec<crate::boundary::Mode> = grammar
        .grammars
        .iter()
        .map(|g| {
            let mut ignored = Vec::new();
            let version = crate::desugar::language_version_of(&g.settings, &mut ignored);
            crate::boundary::mode_of(&g.settings, version)
        })
        .collect();
    let mut cx = Context {
        grammar: &grammar,
        sources: source_table,
        out: SemanticsIr {
            sources: Vec::new(),
            grammar_units: Vec::new(),
            import_edges: Vec::new(),
            entries: Vec::new(),
            semantics: empty_semantics(),
            capture_bindings: Vec::new(),
            recovery_follow_sync_patterns: BTreeMap::new(),
            warnings,
        },
        errors: Vec::new(),
        symbol_ids: BTreeMap::new(),
        unit_ids: BTreeMap::new(),
        types: BTreeMap::new(),
        fields: BTreeMap::new(),
        literal_boundary,
    };
    cx.metadata();
    schema::prepare(&mut cx);
    recursion::check(&mut cx);
    for grammar in &grammar.grammars {
        tokens::lower(&mut cx, grammar);
        for rule in &grammar.rules {
            if rule.annotations.iter().any(
                |a| matches!(a,a::Annotation::Precedence {level,..} if *level > i64::MAX as u64),
            ) {
                cx.errors.push(error(
                    "E-LOWER-PRECEDENCE-RANGE",
                    format!("{}: precedence が IR i64 範囲を超えています", rule.name),
                ));
            }
            let policy = trivia::policy(&mut cx, grammar, rule);
            let fields = captures::fields(&cx, rule);
            cx.fields.insert(rule.id, fields);
            let (body, sites) = expr::lower(&mut cx, rule, &policy);
            let body = recursion::rewrite(&mut cx, rule, body, &policy);
            annotations::rule(&mut cx, grammar, rule, body, policy, sites);
        }
    }
    schema::finish(&mut cx);
    annotations::recovery_sites(&mut cx);
    if cx.errors.is_empty() {
        Ok(cx.out)
    } else {
        cx.errors.extend(cx.out.warnings);
        Err(cx.errors)
    }
}

struct Context<'a> {
    grammar: &'a ResolvedGrammar,
    sources: source::Sources,
    out: SemanticsIr,
    errors: Vec<LowerError>,
    symbol_ids: BTreeMap<r::SymbolId, String>,
    unit_ids: BTreeMap<r::UnitId, String>,
    types: BTreeMap<r::SymbolId, String>,
    fields: BTreeMap<r::SymbolId, BTreeMap<String, captures::Shape>>,
    /// GrammarId 順のリテラル語境界モード（D-040）。
    literal_boundary: Vec<crate::boundary::Mode>,
}
impl Context<'_> {
    /// その symbol を宣言した文法のリテラル語境界モード。
    fn boundary_mode(&self, id: r::SymbolId) -> crate::boundary::Mode {
        self.literal_boundary[self.grammar.symbol(id).grammar.0]
    }
    fn sid(&self, id: r::SymbolId) -> String {
        self.symbol_ids[&id].clone()
    }
    fn span(&self, unit: r::UnitId, span: a::Span) -> Span {
        self.sources.span(&self.grammar.units[unit.0].path, span)
    }
    fn text(&self, unit: r::UnitId) -> &str {
        self.sources.text(&self.grammar.units[unit.0].path)
    }
    fn rule(&self, id: r::SymbolId) -> Option<&r::ResolvedRule> {
        self.grammar
            .grammars
            .iter()
            .flat_map(|g| &g.rules)
            .find(|r| r.id == id)
    }
    fn ast_root(&self, id: r::SymbolId) -> String {
        format!(
            "{}AST",
            self.grammar.grammars[self.grammar.symbol(id).grammar.0].name
        )
    }
    fn metadata(&mut self) {
        self.out.sources = self.sources.records.clone();
        for unit in &self.grammar.units {
            let duplicate = self
                .grammar
                .units
                .iter()
                .filter(|u| u.ast.name == unit.ast.name)
                .count()
                > 1;
            let span = self.span(unit.id, unit.ast.span);
            let id = if duplicate {
                format!("grammar:{}:{}", span.source_id, unit.ast.name)
            } else {
                format!("grammar:{}", unit.ast.name)
            };
            self.unit_ids.insert(unit.id, id.clone());
            self.out.grammar_units.push(GrammarUnit {
                id,
                name: unit.ast.name.clone(),
                source_id: span.source_id.clone(),
                source: span,
                annotations: unit
                    .ast
                    .settings
                    .iter()
                    .filter_map(|setting| {
                        let (key, value) = match setting {
                            a::GlobalSetting::String { key, value, .. } => {
                                (key, JsonValue::String(value.clone()))
                            }
                            a::GlobalSetting::Block { key, entries, .. } => (
                                key,
                                JsonValue::Object(
                                    entries
                                        .iter()
                                        .map(|e| {
                                            (e.key.clone(), JsonValue::String(e.value.clone()))
                                        })
                                        .collect(),
                                ),
                            ),
                        };
                        (!["package", "comment", "whitespace", "interleave"]
                            .contains(&key.as_str()))
                        .then(|| Annotation {
                            name: key.clone(),
                            arguments: BTreeMap::from([("value".into(), value)]),
                            source: self.span(unit.id, setting.span()),
                        })
                    })
                    .collect(),
                package: unit.ast.settings.iter().find_map(|s| match s {
                    a::GlobalSetting::String { key, value, .. } if key == "package" => {
                        Some(value.clone())
                    }
                    _ => None,
                }),
                raw_settings: trivia::settings(&unit.ast.settings),
            });
        }
        let mut groups = Vec::new();
        for s in &self.grammar.symbols {
            let group = (
                s.grammar,
                s.unit,
                s.name.rsplit_once('.').map(|(p, _)| p.to_owned()),
            );
            if !groups.contains(&group) {
                groups.push(group);
            }
        }
        let mut symbols: Vec<_> = self.grammar.symbols.iter().collect();
        symbols.sort_by_key(|s| {
            (
                groups
                    .iter()
                    .position(|g| {
                        g == &(
                            s.grammar,
                            s.unit,
                            s.name.rsplit_once('.').map(|(p, _)| p.to_owned()),
                        )
                    })
                    .unwrap(),
                s.span.start,
            )
        });
        for symbol in symbols {
            let grammar = &self.grammar.grammars[symbol.grammar.0];
            let gid = self.unit_ids[&grammar.unit].clone();
            let id = format!("{}::{}", gid.strip_prefix("grammar:").unwrap(), symbol.name);
            self.symbol_ids.insert(symbol.id, id.clone());
            let mut source_span = symbol.span;
            if let Some(rule) = self.rule(symbol.id) {
                if let Some(last) = rule.annotations.last() {
                    let mut offset = last.span().end;
                    let text = self.text(symbol.unit);
                    loop {
                        offset += text[offset..].len() - text[offset..].trim_start().len();
                        if !text[offset..].starts_with("//") {
                            break;
                        }
                        offset += text[offset..]
                            .find(['\r', '\n'])
                            .unwrap_or(text.len() - offset);
                    }
                    source_span.start = offset;
                }
            }
            let span = self.span(symbol.unit, source_span);
            self.out.semantics.symbols.push(Symbol {
                id: id.clone(),
                qualified_symbol_id: id,
                grammar_id: gid,
                name: symbol.name.clone(),
                original_name: symbol.name.rsplit('.').next().unwrap().into(),
                source_id: span.source_id.clone(),
                source: span,
                kind: match symbol.kind {
                    r::SymbolKind::Rule => SymbolKind::Rule,
                    r::SymbolKind::Token => SymbolKind::Token,
                },
            });
        }
        for edge in &self.grammar.imports {
            let unit = &self.grammar.units[edge.from.0];
            let requested_path = unit
                .ast
                .imports
                .iter()
                .find(|i| i.span == edge.span)
                .map(|i| i.path.clone())
                .unwrap_or_default();
            self.out.import_edges.push(ImportEdge {
                alias: Some(edge.alias.clone()),
                from_grammar_id: self.unit_ids[&edge.from].clone(),
                resolved_grammar_id: self.unit_ids[&edge.to].clone(),
                requested_path,
                source_id: self.span(edge.from, edge.span).source_id,
                bindings: self
                    .grammar
                    .symbols
                    .iter()
                    .filter(|s| {
                        s.grammar == edge.grammar
                            && s.name.starts_with(&format!("{}.", edge.namespace))
                    })
                    .map(|s| ImportBinding {
                        local_name: s.name.clone(),
                        qualified_symbol_id: self.sid(s.id),
                    })
                    .collect(),
            });
        }
        for grammar in &self.grammar.grammars {
            if let Some(root) = grammar.root.or_else(|| grammar.rules.first().map(|r| r.id)) {
                self.out.entries.push(Entry {
                    grammar_id: self.unit_ids[&grammar.unit].clone(),
                    rule_id: self.sid(root),
                });
            }
        }
    }
}

fn mapping(rule: &r::ResolvedRule) -> Option<(&str, &[String])> {
    rule.annotations.iter().rev().find_map(|a| match a {
        a::Annotation::Mapping {
            class_name, params, ..
        } => Some((class_name.as_str(), params.as_slice())),
        _ => None,
    })
}
fn assoc(rule: &r::ResolvedRule) -> Option<OperatorAssociativity> {
    rule.annotations.iter().find_map(|a| match a {
        a::Annotation::LeftAssoc { .. } => Some(OperatorAssociativity::Left),
        a::Annotation::RightAssoc { .. } => Some(OperatorAssociativity::Right),
        _ => None,
    })
}
fn skipped(rule: &r::ResolvedRule) -> bool {
    rule.annotations
        .iter()
        .any(|a| matches!(a, a::Annotation::Skip { .. }))
}
fn empty_semantics() -> Semantics {
    Semantics {
        ast_types: vec![],
        captures: vec![],
        catalogs: vec![],
        common_fields: vec![],
        expressions: vec![],
        normalizations: vec![],
        recipes: vec![],
        recovery: vec![],
        rules: vec![],
        symbols: vec![],
        tokens: vec![],
        typeof_constraints: vec![],
        trivia: Trivia {
            policies: vec![],
            insertion_plan: vec![],
        },
        scope: Scope {
            effects: vec![],
            rollback: ScopeRollback,
            trim_max_code_point: ScopeTrimMaxCodePoint,
            version_changes: ScopeVersionChanges,
        },
        contracts: Contracts {
            choice: ContractsChoice,
            cursor: CursorContract {
                begin: CursorContractBegin,
                commit: CursorContractCommit,
                consume: CursorContractConsume,
                coordinate: CursorContractCoordinate,
                frame_flag: CursorContractFrameFlag,
                internal_java: CursorContractInternalJava,
                internal_rust: CursorContractInternalRust,
                match_only: CursorContractMatchOnly,
                mode: CursorContractMode,
                positions: CursorContractPositions,
                rollback: CursorContractRollback,
            },
            left_recursion: ContractsLeftRecursion,
            longest: ContractsLongest,
            mapping_failure: ContractsMappingFailure,
            mapping_phase: ContractsMappingPhase,
            occurrences: OccurrenceContract {
                capture_fields: OccurrenceContractCaptureFields,
                diagnostics: OccurrenceContractDiagnostics,
                lexical_fields: OccurrenceContractLexicalFields,
                lexical_output: OccurrenceContractLexicalOutput,
                node_spans: OccurrenceContractNodeSpans,
                scope_events: OccurrenceContractScopeEvents,
                self_host_transform: OccurrenceContractSelfHostTransform,
            },
            predictive: ContractsPredictive,
            regex_policy: ContractsRegexPolicy,
            repeat: ContractsRepeat,
            repeat_zero_max: ContractsRepeatZeroMax,
            span_unit: ContractsSpanUnit,
        },
        diagnostics: Diagnostics {
            deepest_rule: DiagnosticsDeepestRule,
            expected_display: vec![],
            java_syntax_offset: DiagnosticsJavaSyntaxOffset,
            merge: DiagnosticsMerge,
            negative_lookahead: DiagnosticsNegativeLookahead,
            offset_unit: DiagnosticsOffsetUnit,
            retain_farthest_separately: DiagnosticsRetainFarthestSeparately,
            rust_syntax_offset: DiagnosticsRustSyntaxOffset,
            successful_rule_failures: DiagnosticsSuccessfulRuleFailures,
            trailing_input_offset: DiagnosticsTrailingInputOffset,
        },
    }
}
