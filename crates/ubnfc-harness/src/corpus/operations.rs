//! ScopeStoreConformanceTest の run は独立 context、transaction 内も順序付き操作列。
//! catalog は解析入力を持たない静的 metadata oracle（RustCatalogConformanceTest）。
use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Declaration {
    pub name: String,
    pub source_offset: usize,
    #[serde(flatten)]
    pub extra: RawFields,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reference {
    pub name: String,
    pub offset: usize,
    pub length: usize,
    #[serde(flatten)]
    pub extra: RawFields,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub offset: usize,
    pub length: usize,
    pub severity: String,
    #[serde(flatten)]
    pub extra: RawFields,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resolution {
    pub query: String,
    #[serde(default)]
    pub declared: Option<bool>,
    pub symbol: Option<Declaration>,
    #[serde(flatten)]
    pub extra: RawFields,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeState {
    pub depth: usize,
    #[serde(default)]
    pub cursor: Option<[usize; 2]>,
    pub resolved: Vec<Resolution>,
    #[serde(default)]
    pub declarations: Option<Vec<Declaration>>,
    #[serde(default)]
    pub declared_current: Option<Vec<Declaration>>,
    #[serde(default)]
    pub all_declarations: Option<Vec<Declaration>>,
    pub references: Vec<Reference>,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub retained: Option<BTreeMap<String, Vec<Declaration>>>,
    #[serde(flatten)]
    pub extra: RawFields,
}

/// op ごとの引数を型付けし、将来追加される操作名・引数も raw のまま残す。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    pub op: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub length: Option<usize>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub points: Option<usize>,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub names: Option<Vec<String>>,
    #[serde(default)]
    pub expected: Option<ScopeState>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub operations: Option<Vec<Operation>>,
    #[serde(flatten)]
    pub extra: RawFields,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScopeExpectation {
    pub operations: Vec<Operation>,
    #[serde(flatten)]
    pub extra: RawFields,
}
#[derive(Deserialize)]
pub struct Run {
    pub id: String,
    #[serde(flatten)]
    pub expect: ScopeExpectation,
}
#[derive(Deserialize)]
pub struct Fixture {
    pub name: String,
    pub source: String,
    pub runs: Vec<Run>,
    #[serde(flatten)]
    pub extra: RawFields,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    pub rule: String,
    /// JSON decode 後の文字列を保持する。trim や UBNF の再 unescape は行わない。
    pub context: String,
    /// 自ルール内の capture 名の重複除去・ソート済み期待値。
    pub captures: Vec<String>,
    #[serde(flatten)]
    pub extra: RawFields,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CatalogExpectation {
    /// ルール宣言順を保持する。
    pub catalogs: Vec<Catalog>,
    #[serde(flatten)]
    pub extra: RawFields,
}
#[derive(Deserialize)]
pub struct CatalogFixture {
    pub name: String,
    pub grammar: String,
    #[serde(flatten)]
    pub expect: CatalogExpectation,
}

pub(crate) fn scope(path: &Path) -> Result<CorpusFile, LoadError> {
    let (fixtures, raw) = read::<Vec<Fixture>>(path)?;
    let mut out = file(Family::Operations, path, raw);
    for fixture in fixtures {
        for run in fixture.runs {
            out.cases.push(Case {
                id: format!("{}/{}", fixture.name, run.id),
                grammar_ref: Vec::new(),
                entry_rule: None,
                input: fixture.source.clone(),
                expect: Expectation::ScopeOps(run.expect),
            });
        }
    }
    Ok(out)
}

pub(crate) fn catalog(path: &Path) -> Result<CorpusFile, LoadError> {
    let (fixtures, raw) = read::<Vec<CatalogFixture>>(path)?;
    let mut out = file(Family::Operations, path, raw);
    for (i, fixture) in fixtures.into_iter().enumerate() {
        out.grammars.push(GrammarSource::Inline {
            name: fixture.name.clone(),
            source: fixture.grammar,
            metadata: metadata(&out.raw[i]),
        });
        out.cases.push(Case {
            id: fixture.name,
            grammar_ref: vec![i],
            entry_rule: None,
            input: String::new(),
            expect: Expectation::Catalog(fixture.expect),
        });
    }
    Ok(out)
}
