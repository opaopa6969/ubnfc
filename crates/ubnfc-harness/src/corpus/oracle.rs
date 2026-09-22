//! ubnfc 独自の Java 実走 oracle。採取値を補正せず、stage を先に判定する。
use super::*;
use crate::{canonical::AstValue, divergence};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Capture {
    pub name: String,
    pub binding: String,
    pub span: [usize; 2],
    pub text: String,
    #[serde(flatten)]
    pub extra: RawFields,
}

/// Mapper.diagnose と ScopeStore の両形式。欠落した項目を捏造しない。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub kind: String,
    pub offset: usize,
    pub expected: Vec<String>,
    pub origin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub farthest_offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub farthest_expected: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(flatten)]
    pub extra: RawFields,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Executed,
    CompilationFailed,
    GenerationFailed,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OracleCase {
    id: String,
    input: String,
    accepted: Option<bool>,
    all_consumed: Option<bool>,
    consumed: Option<usize>,
    matched: Option<usize>,
    ast: Option<AstValue>,
    captures: Vec<Capture>,
    declarations: Vec<operations::Declaration>,
    diagnostics: Vec<Diagnostic>,
    mapping_error: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Generation {
    cli_exit: i32,
    cli_error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_error: Option<String>,
    #[serde(flatten)]
    extra: RawFields,
}

#[derive(Serialize, Deserialize)]
struct CompilationDiagnostic {
    kind: String,
    code: String,
    file: Option<String>,
    line: i64,
    message: String,
    #[serde(flatten)]
    extra: RawFields,
}

#[derive(Deserialize)]
struct Compilation {
    success: bool,
    diagnostics: Vec<CompilationDiagnostic>,
}

#[derive(Deserialize)]
struct Fixture {
    feature: String,
    grammar: String,
    source: String,
    stage: Stage,
    cases: Vec<OracleCase>,
    generation: Generation,
    compilation: Option<Compilation>,
}

/// unlaxer-parser の root や呼出時 cwd と独立した、この checkout の oracle。
pub fn default_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/oracle")
}

pub(crate) fn invalid(path: &Path, message: impl Into<String>) -> LoadError {
    LoadError {
        path: path.into(),
        line: None,
        column: None,
        message: message.into(),
    }
}

/// 単一の採取ファイルを正規化する。乖離台帳の適用は load_all が行う。
pub fn load(path: &Path) -> Result<CorpusFile, LoadError> {
    // 採取 JSON は標準形式なので、upstream fixture 用の escape 修復は行わない。
    let text = read_text(path)?;
    let fixture: Fixture = decode(path, &text)?;
    let mut out = file(Family::Oracle, path, decode(path, &text)?);
    if fixture.feature.is_empty() || fixture.source.is_empty() {
        return Err(invalid(path, "feature と source は空にできません"));
    }
    let failure = match fixture.stage {
        Stage::Executed => None,
        Stage::CompilationFailed => {
            let compilation = fixture
                .compilation
                .filter(|c| !c.success && !c.diagnostics.is_empty())
                .ok_or_else(|| invalid(path, "compilation_failed に失敗診断がありません"))?;
            Some(format!(
                "compilation_failed: {}",
                serde_json::to_string(&compilation.diagnostics).expect("診断は JSON に直列化可能")
            ))
        }
        Stage::GenerationFailed => Some(format!(
            "generation_failed: {}",
            serde_json::to_string(&fixture.generation).expect("生成結果は JSON に直列化可能")
        )),
    };
    out.grammars.push(GrammarSource::Inline {
        name: fixture.feature,
        source: fixture.grammar,
        // source revision、imports、parseError 等も raw/metadata で完全に保持する。
        metadata: metadata(&out.raw),
    });
    let mut ids = BTreeSet::new();
    for row in fixture.cases {
        if row.id.is_empty() || !ids.insert(row.id.clone()) {
            return Err(invalid(
                path,
                format!("空または重複した case ID: {}", row.id),
            ));
        }
        let mut expect = match &failure {
            Some(message) => Expectation::GenerationFailure {
                message: message.clone(),
            },
            None => Expectation::Oracle {
                accepted: row.accepted,
                all_consumed: row.all_consumed,
                consumed: row.consumed,
                matched: row.matched,
                ast: row.ast,
                captures: row.captures,
                declarations: row.declarations,
                diagnostics: row.diagnostics,
                mapping_error: row.mapping_error,
            },
        };
        // resolve.rs:143-155 は循環 import を明示的に拒否する契約。
        // 採取済み apiError と照合する。RustConformanceTest.java:391-455 の
        // reversed-metadata は正のテストなのでこの扱いにしない。
        if path
            .file_name()
            .is_some_and(|name| name == "import-cycle.json")
            && fixture.stage == Stage::GenerationFailed
            && out.raw["generation"]["apiError"]
                .as_str()
                .is_some_and(|e| e.contains("循環 import"))
        {
            expect = Expectation::ExpectedRejection {
                stage: "ir".into(),
                code: "E-IMPORT-CYCLE".into(),
                evidence: "crates/ubnfc-front/tests/resolve.rs:143-155; corpus/oracle/import-cycle.json generation.apiError".into(),
                original: Box::new(expect),
            };
        }
        out.cases.push(Case {
            id: row.id,
            grammar_ref: vec![0],
            // grammar の @root に従う。rule 名を決め打ちしない。
            entry_rule: None,
            input: row.input,
            expect,
            known_divergences: Vec::new(),
        });
    }
    Ok(out)
}

/// *.json をファイル名順に読む。台帳は corpus の件数に含めない。
pub fn load_all(root: impl AsRef<Path>) -> Result<Vec<CorpusFile>, LoadError> {
    let root = root.as_ref();
    let mut paths = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| invalid(root, e.to_string()))? {
        let path = entry.map_err(|e| invalid(root, e.to_string()))?.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && path
                .file_name()
                .is_some_and(|name| name != "divergences.json")
        {
            paths.push(path);
        }
    }
    paths.sort();
    let mut files = paths
        .iter()
        .map(|path| load(path))
        .collect::<Result<Vec<_>, _>>()?;
    divergence::apply(&root.join("divergences.json"), &mut files)?;
    Ok(files)
}
