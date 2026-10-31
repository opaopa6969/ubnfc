//! ソースを一度ずつ読み、検証済みの実行用 Grammar IR を組み立てる。
//!
//! loader は字句的に正規化した絶対パスを受け取る（実在ファイルは不要）。
//! 論理ソースルートは main の最寄りの Git ルート（`.git` file/dir）。
//! Git 外では main の親。import が外にあれば全ソースを含む共通親まで広げる。
//! これにより配置先・起動ディレクトリ・`./` に ID が依存しない。
//! 複数 grammar は宣言順に保持し、最後の grammar をエントリにする。

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::analysis::{self, AnalysisError};
use crate::ir::{self, *};
use crate::lower::{self, LowerError, SourceInput};
use crate::resolve::{self, ResolveError, ResolvedGrammar};
use crate::syntax::{self, GrammarAst, SyntaxError};
use crate::validate::{self, ValidationIssue};

mod checks;

#[derive(Debug, Clone, PartialEq)]
pub struct Compiled {
    pub ir: GrammarIr,
    /// validate の警告に lower の互換性警告を加えた、決定的な順序の診断。
    pub warnings: Vec<ValidationIssue>,
}

/// 呼び出し側で解析済みの AST と、その AST が参照する元 UTF-8。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSource {
    pub text: String,
    pub ast: GrammarAst,
}

impl ParsedSource {
    pub fn new(text: impl Into<String>, ast: GrammarAst) -> Self {
        Self {
            text: text.into(),
            ast,
        }
    }
}

/// `compile_parsed` の import loader が区別する読込失敗と構文失敗。
#[derive(Debug)]
pub enum ParsedLoadError {
    Io(io::Error),
    Syntax(SyntaxError),
}

impl From<io::Error> for ParsedLoadError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<SyntaxError> for ParsedLoadError {
    fn from(error: SyntaxError) -> Self {
        Self::Syntax(error)
    }
}

#[derive(Debug)]
pub enum PipelineError {
    Io {
        path: PathBuf,
        error: io::Error,
    },
    Syntax {
        path: PathBuf,
        error: SyntaxError,
    },
    Resolve(Vec<ResolveError>),
    Validation(Vec<ValidationIssue>),
    Strict(Vec<ValidationIssue>),
    Lower(Vec<LowerError>),
    Analysis(Vec<AnalysisError>),
    /// front 自身が生成した IR の契約違反。
    Internal(Vec<IrError>),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn lines<T: fmt::Display>(f: &mut fmt::Formatter<'_>, items: &[T]) -> fmt::Result {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }
                write!(f, "{item}")?;
            }
            Ok(())
        }
        match self {
            Self::Io { path, error } => write!(f, "{}: {error}", path.display()),
            Self::Syntax { path, error } => write!(f, "{}: {error}", path.display()),
            Self::Resolve(errors) => lines(f, errors),
            Self::Validation(issues) | Self::Strict(issues) => lines(f, issues),
            Self::Lower(errors) => lines(f, errors),
            Self::Analysis(errors) => lines(f, errors),
            Self::Internal(errors) => {
                writeln!(f, "内部エラー: 生成 IR が契約に違反しています")?;
                lines(f, errors)
            }
        }
    }
}

impl std::error::Error for PipelineError {}

pub fn compile(
    main_path: impl AsRef<Path>,
    loader: &dyn Fn(&Path) -> io::Result<String>,
) -> Result<Compiled, PipelineError> {
    // `pkg:` の import（crate::packages）は仮想パスとして loader に届く。
    let packaged = crate::packages::loader(loader);
    let loader: &dyn Fn(&Path) -> io::Result<String> = &packaged;
    let path = absolute_path(main_path.as_ref()).map_err(|error| PipelineError::Io {
        path: main_path.as_ref().into(),
        error,
    })?;
    let main = parse_source(&path, loader)?;
    compile_parsed(&path, main, &|import| {
        let text = loader(import).map_err(ParsedLoadError::Io)?;
        let ast = syntax::parse_file(&text).map_err(ParsedLoadError::Syntax)?;
        Ok(ParsedSource::new(text, ast))
    })
}

/// 解析済み AST を起点に IR を構築する。呼び出し側は AST とその元 source を組にする。
/// import loader も解析済み AST を返すため、この経路は手書き parser を呼ばない。
pub fn compile_parsed(
    main_path: impl AsRef<Path>,
    main: ParsedSource,
    loader: &dyn Fn(&Path) -> Result<ParsedSource, ParsedLoadError>,
) -> Result<Compiled, PipelineError> {
    compile_parsed_with_strict(main_path.as_ref(), main, loader, false)
}

/// strict は validate の警告を lower より前に拒否し、lower の警告も拒否する。
pub fn compile_with_strict(
    main_path: impl AsRef<Path>,
    loader: &dyn Fn(&Path) -> io::Result<String>,
    strict: bool,
) -> Result<Compiled, PipelineError> {
    // `pkg:` の import（crate::packages）は仮想パスとして loader に届く。
    let packaged = crate::packages::loader(loader);
    let loader: &dyn Fn(&Path) -> io::Result<String> = &packaged;
    let path = absolute_path(main_path.as_ref()).map_err(|error| PipelineError::Io {
        path: main_path.as_ref().into(),
        error,
    })?;
    let main = parse_source(&path, loader)?;
    compile_parsed_with_strict(
        &path,
        main,
        &|import| {
            let text = loader(import).map_err(ParsedLoadError::Io)?;
            let ast = syntax::parse_file(&text).map_err(ParsedLoadError::Syntax)?;
            Ok(ParsedSource::new(text, ast))
        },
        strict,
    )
}

fn compile_parsed_with_strict(
    main_path: &Path,
    main: ParsedSource,
    loader: &dyn Fn(&Path) -> Result<ParsedSource, ParsedLoadError>,
    strict: bool,
) -> Result<Compiled, PipelineError> {
    let prepared = prepare_parsed(main_path, main, loader)?;
    let mut warnings = prepared.issues.clone();
    warnings.extend(validate::validate(&prepared.grammar));
    if warnings.iter().any(|i| i.severity() == "ERROR") {
        return Err(PipelineError::Validation(warnings));
    }
    if strict && !warnings.is_empty() {
        return Err(PipelineError::Strict(warnings));
    }
    let mut lowered =
        lower::lower(&prepared.grammar, &prepared.sources).map_err(PipelineError::Lower)?;
    finalize_origins(&mut lowered)?;
    finalize_separated_captures(&mut lowered)?;
    warnings.extend(lowered.warnings.iter().map(|warning| {
        let (LowerError::Warning { code, message, .. } | LowerError::Error { code, message, .. }) =
            warning;
        ValidationIssue {
            code: code.clone(),
            message: message.clone(),
            hint: String::new(),
            rule: None,
            grammar: String::new(),
        }
    }));
    sort_issues(&mut warnings);
    if strict && !warnings.is_empty() {
        return Err(PipelineError::Strict(warnings));
    }
    let (analysis, plan, choice_issues) = analysis::analyze_with_choice_issues(&lowered.semantics)
        .map_err(PipelineError::Analysis)?;
    warnings.extend(choice_issues.into_iter().map(choice_warning));
    sort_issues(&mut warnings);
    if strict && !warnings.is_empty() {
        return Err(PipelineError::Strict(warnings));
    }
    analysis::attach_regex_plans(&mut lowered.semantics, &plan);
    for recovery in &mut lowered.semantics.recovery {
        if matches!(recovery.mode, RecoveryMode::Auto | RecoveryMode::Skip) {
            let follow = analysis
                .rules
                .iter()
                .find(|r| r.rule_id == recovery.rule_id)
                .and_then(|r| r.follow_sync_patterns.clone())
                .ok_or_else(|| {
                    internal("/semantics/recovery", "AUTO / SKIP の FOLLOW がありません")
                })?;
            // design §3.10: AUTO の同期集合は FOLLOW。空なら既定の `;` に落とすが、
            // それは外側が待っている字句ではないので SYNC と同じく消費する。
            let fallback = follow.is_empty() && recovery.mode == RecoveryMode::Auto;
            recovery.resolved_sync_patterns = if fallback { vec![";".into()] } else { follow };
            recovery.consume_sync = recovery.consume_sync || fallback;
        }
    }
    let entry = lowered.entries.pop().ok_or_else(|| {
        PipelineError::Validation(vec![ValidationIssue {
            code: "E-GENERAL-NO-ENTRY".into(),
            message: "エントリにできる rule がありません".into(),
            hint: "少なくとも一つの rule を宣言してください。".into(),
            rule: None,
            grammar: String::new(),
        }])
    })?;
    let ir = GrammarIr {
        language_version: prepared.language_version,
        required_features: required_features(&lowered.semantics, &plan, &lowered.import_edges),
        analysis,
        plan,
        analysis_version: GrammarIrAnalysisVersion,
        serialization: GrammarIrSerialization,
        version: GrammarIrVersion,
        entry,
        semantics: lowered.semantics,
        sources: lowered.sources,
        grammar_units: lowered.grammar_units,
        import_edges: lowered.import_edges,
    };
    checks::check(&ir)?;
    Ok(Compiled { ir, warnings })
}

/// 順序付き選択の到達不能候補（D-037）を既存の診断機械へ載せる。
pub(crate) fn choice_warning(issue: analysis::ChoiceIssue) -> ValidationIssue {
    ValidationIssue {
        code: issue.code,
        message: issue.message,
        hint: issue.hint,
        rule: issue.rule,
        grammar: issue.grammar,
    }
}

/// 既存 validate 契約。lower / analysis による生成可否とは別に診断を返す。
/// エラーが無いときに限り lower / analysis まで進め、到達不能候補の警告も足す。
pub fn validate_file(
    main_path: impl AsRef<Path>,
    loader: &dyn Fn(&Path) -> io::Result<String>,
) -> Result<Vec<ValidationIssue>, PipelineError> {
    // `pkg:` の import（crate::packages）は仮想パスとして loader に届く。
    let packaged = crate::packages::loader(loader);
    let loader: &dyn Fn(&Path) -> io::Result<String> = &packaged;
    let path = absolute_path(main_path.as_ref()).map_err(|error| PipelineError::Io {
        path: main_path.as_ref().into(),
        error,
    })?;
    let main = parse_source(&path, loader)?;
    let prepared = prepare_parsed(&path, main, &|import| {
        let text = loader(import).map_err(ParsedLoadError::Io)?;
        let ast = syntax::parse_file(&text).map_err(ParsedLoadError::Syntax)?;
        Ok(ParsedSource::new(text, ast))
    })?;
    let mut issues = prepared.issues.clone();
    issues.extend(validate::validate(&prepared.grammar));
    if issues.iter().all(|i| i.severity() != "ERROR") {
        // lower / analysis の失敗は ir / gen が報告する。ここでは警告を足せないだけ。
        if let Ok(lowered) = lower::lower(&prepared.grammar, &prepared.sources) {
            if let Ok((_, _, choice_issues)) =
                analysis::analyze_with_choice_issues(&lowered.semantics)
            {
                issues.extend(choice_issues.into_iter().map(choice_warning));
            }
        }
    }
    sort_issues(&mut issues);
    Ok(issues)
}

pub(crate) fn sort_issues(issues: &mut [ValidationIssue]) {
    issues.sort_by(|a, b| {
        (&a.grammar, &a.rule, &a.code, &a.message, &a.hint)
            .cmp(&(&b.grammar, &b.rule, &b.code, &b.message, &b.hint))
    });
}

struct Prepared {
    grammar: ResolvedGrammar,
    sources: Vec<SourceInput>,
    /// `@ubnf` の最大宣言値（D-038）。IR の `languageVersion` になる。
    language_version: u64,
    /// 脱糖・バージョン門の診断（resolve より前に出る）。
    issues: Vec<ValidationIssue>,
}

fn parse_source(
    main_path: &Path,
    loader: &dyn Fn(&Path) -> io::Result<String>,
) -> Result<ParsedSource, PipelineError> {
    let text = loader(main_path).map_err(|error| PipelineError::Io {
        path: main_path.into(),
        error,
    })?;
    let ast = syntax::parse_file(&text).map_err(|error| PipelineError::Syntax {
        path: main_path.into(),
        error,
    })?;
    Ok(ParsedSource::new(text, ast))
}

fn prepare_parsed(
    main_path: &Path,
    main: ParsedSource,
    loader: &dyn Fn(&Path) -> Result<ParsedSource, ParsedLoadError>,
) -> Result<Prepared, PipelineError> {
    let path = absolute_path(main_path).map_err(|error| PipelineError::Io {
        path: main_path.into(),
        error,
    })?;
    // v2 の表層構文はここで v1 の正規形へ脱糖する。resolve 以降は v1 しか見ない。
    // sources にも脱糖後の AST を入れる（lower の E-LOWER-SOURCE-STALE 検査と一致させる）。
    let issues = RefCell::new(Vec::new());
    let language_version = RefCell::new(crate::desugar::DEFAULT_LANGUAGE_VERSION);
    let mut main = main;
    {
        let mut issues = issues.borrow_mut();
        let version = crate::desugar::file(&mut main.ast, &mut issues);
        *language_version.borrow_mut() = version;
    }
    let inputs = RefCell::new(BTreeMap::from([(path.clone(), main.clone())]));
    let grammar = resolve::resolve_with(&[(path.clone(), main.ast)], &|import| {
        let mut parsed = loader(import).map_err(|error| match error {
            ParsedLoadError::Io(error) => resolve::LoadError::Io(error),
            ParsedLoadError::Syntax(error) => resolve::LoadError::Syntax(error),
        })?;
        let version = crate::desugar::file(&mut parsed.ast, &mut issues.borrow_mut());
        let mut current = language_version.borrow_mut();
        *current = (*current).max(version);
        let ast = parsed.ast.clone();
        inputs.borrow_mut().insert(import.to_path_buf(), parsed);
        Ok(ast)
    })
    .map_err(PipelineError::Resolve)?;
    let parent = path.parent().unwrap_or(Path::new("/"));
    // リポジトリの論理パスを既存 IR と共有する。Git コマンド・revision・cwd は不要。
    // 仮想 loader / Git 外のファイルも使えるよう、marker が無い場合は親を使う。
    let mut root = parent
        .ancestors()
        .find(|p| p.join(".git").exists())
        .unwrap_or(parent)
        .to_path_buf();
    for unit in &grammar.units {
        while !unit.path.starts_with(&root) {
            if !root.pop() {
                return Err(internal("/sources", "ソースの共通親を決定できません"));
            }
        }
    }
    let sources = inputs
        .into_inner()
        .into_iter()
        .map(|(path, parsed)| {
            let relative = path
                .strip_prefix(&root)
                .map_err(|_| internal("/sources", "ソースが論理ルートの外にあります"))?;
            let logical = relative
                .to_str()
                .ok_or_else(|| internal("/sources", "ソースパスは UTF-8 が必要です"))?
                .replace(std::path::MAIN_SEPARATOR, "/");
            Ok(SourceInput::parsed(path, logical, parsed.text, parsed.ast))
        })
        .collect::<Result<_, PipelineError>>()?;
    Ok(Prepared {
        grammar,
        sources,
        language_version: language_version.into_inner(),
        issues: issues.into_inner(),
    })
}

pub(crate) fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    Ok(normalized)
}

fn internal(path: &str, message: &str) -> PipelineError {
    PipelineError::Internal(vec![IrError::Validation {
        path: path.into(),
        message: message.into(),
    }])
}

/// lower の統合後シンボル ID を保ち、GrammarUnit は出所 metadata にする。
/// import された symbol の grammarId もその宣言元 source を指すのが IR 契約。
fn finalize_origins(lowered: &mut lower::SemanticsIr) -> Result<(), PipelineError> {
    for symbol in &mut lowered.semantics.symbols {
        let origin = lowered
            .grammar_units
            .iter()
            .find(|unit| {
                unit.source_id == symbol.source_id
                    && unit.source.start <= symbol.source.start
                    && symbol.source.end <= unit.source.end
            })
            .ok_or_else(|| {
                internal(
                    "/semantics/symbols",
                    "シンボルの宣言元 grammar がありません",
                )
            })?;
        symbol.grammar_id = origin.id.clone();
    }
    for entry in &mut lowered.entries {
        let symbol = lowered
            .semantics
            .symbols
            .iter()
            .find(|s| s.id == entry.rule_id)
            .ok_or_else(|| internal("/entry", "エントリのシンボルがありません"))?;
        entry.grammar_id = symbol.grammar_id.clone();
    }
    Ok(())
}

/// lower が別表に返す capture binding を、直列化可能な式への参照に反映する。
/// `element (separator element)*` の capture は全体の Seq で一度ではなく、
/// 共有 element の各完了時に発生する。site の構文順・cardinality は保持する。
fn finalize_separated_captures(lowered: &mut lower::SemanticsIr) -> Result<(), PipelineError> {
    let s = &mut lowered.semantics;
    let mut changed = false;
    for capture in &mut s.captures {
        let expr = s
            .expressions
            .iter()
            .find(|e| e.id == capture.expr_id)
            .ok_or_else(|| internal("/semantics/captures", "capture の式がありません"))?;
        if !expr.id.as_str().ends_with("/separated/lower/separated/0") {
            continue;
        }
        let ExprNode::Seq { items } = &expr.node else {
            return Err(internal(
                "/semantics/captures",
                "区切り付き反復の lowering 形が不正です",
            ));
        };
        let binding = lowered
            .capture_bindings
            .iter()
            .find(|b| b.site_id == capture.site_id)
            .ok_or_else(|| internal("/semantics/captures", "capture binding がありません"))?;
        if items.len() != 2 || binding.value_expr_ids.is_empty() {
            return Err(internal(
                "/semantics/captures",
                "区切り付き反復の binding が不正です",
            ));
        }
        capture.expr_id = items[0].clone();
        changed = true;
    }
    if !changed {
        return Ok(());
    }
    // ID は所属式と式内の宣言順から再構成し、全ての site 参照を同時に更新する。
    let mut names = BTreeMap::new();
    for expr in &mut s.expressions {
        expr.capture_site_ids.clear();
    }
    for capture in &mut s.captures {
        let expr = s
            .expressions
            .iter_mut()
            .find(|e| e.id == capture.expr_id)
            .ok_or_else(|| internal("/semantics/captures", "binding の式がありません"))?;
        let id = format!("{}/capture/{}", expr.id, expr.capture_site_ids.len());
        names.insert(capture.site_id.clone(), id.clone());
        capture.site_id = id.clone();
        expr.capture_site_ids.push(id);
    }
    let rename = |id: &mut String| {
        if let Some(new) = names.get(id) {
            *id = new.clone();
        }
    };
    for rule in &mut s.rules {
        if let Some(op) = &mut rule.operator {
            rename(&mut op.left_site_id);
            rename(&mut op.op_site_id);
            rename(&mut op.right_site_id);
        }
    }
    for recipe in &mut s.recipes {
        for field in &mut recipe.fields {
            for id in &mut field.site_ids {
                rename(id);
            }
        }
    }
    for catalog in &mut s.catalogs {
        rename(&mut catalog.capture_site_id);
    }
    for constraint in &mut s.typeof_constraints {
        rename(&mut constraint.own_capture);
        rename(&mut constraint.referenced_capture);
    }
    for scope in &mut s.scope.effects {
        for event in &mut scope.events {
            if let Some(id) = &mut event.capture_site_id {
                rename(id);
            }
        }
    }
    Ok(())
}

fn required_features(s: &Semantics, plan: &Plan, imports: &[ImportEdge]) -> Vec<RequiredFeature> {
    use RequiredFeature::*;
    let mut features = vec![
        DualCursor,
        CursorFrameFlag,
        BackendDiagnostics,
        StructuredOccurrences,
    ];
    for (feature, used) in [
        (
            DeclarativeTokens,
            s.tokens
                .iter()
                .any(|t| matches!(t.spec, TokenSpec::Declarative { .. })),
        ),
        (Captures, !s.captures.is_empty()),
        (
            DeferredMapping,
            !s.recipes.is_empty()
                || !s.typeof_constraints.is_empty()
                || s.rules.iter().any(|r| r.mapping_type_id.is_some())
                || s.captures.iter().any(|c| c.conversion != Conversion::Text),
        ),
        (TriviaInsertion, !s.trivia.insertion_plan.is_empty()),
        (ScopeUndo, !s.scope.effects.is_empty()),
        (
            ExternTokens,
            s.tokens
                .iter()
                .any(|t| matches!(t.spec, TokenSpec::Extern { .. })),
        ),
        (
            Regex,
            s.tokens
                .iter()
                .any(|t| matches!(t.spec, TokenSpec::Regex { .. })),
        ),
        (Operators, s.rules.iter().any(|r| r.operator.is_some())),
        // normalizations は lowering の記録。実行本体は通常の Seq/Repeat。
        (Recovery, !s.recovery.is_empty()),
        (Imports, !imports.is_empty()),
        (
            Enum,
            s.ast_types
                .iter()
                .any(|t| matches!(t, AstType::Enum { .. }))
                || s.recipes.iter().any(|r| r.kind == RecipeKind::Enum),
        ),
        (CommonField, !s.common_fields.is_empty()),
        (Eval, s.rules.iter().any(|r| r.eval.is_some())),
        (Catalog, !s.catalogs.is_empty()),
        (Typeof, !s.typeof_constraints.is_empty()),
    ] {
        if used {
            features.push(feature);
        }
    }
    for expr in &s.expressions {
        if let ExprNode::Choice { selection, .. } = expr.node {
            features.push(match selection {
                ChoiceSelection::Ordered => OrderedChoice,
                ChoiceSelection::Predictive => PredictiveChoice,
                ChoiceSelection::Longest => LongestChoice,
            });
        }
    }
    features.extend(analysis::plan_required_features(plan));
    features.sort_by_cached_key(|f| serde_json::to_string(f).expect("feature serialization"));
    features.dedup();
    features
}
