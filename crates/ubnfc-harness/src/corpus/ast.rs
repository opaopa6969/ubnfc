//! NestedCapture / RuleTrivia / ScopeAnnotationConformanceTest の独立 AST oracle。
use super::*;
use crate::canonical::AstValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Capture {
    pub name: String,
    pub span: [usize; 2],
    pub text: String,
    #[serde(flatten)]
    pub extra: RawFields,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstExpectation {
    #[serde(default)]
    pub ast: Option<AstValue>,
    /// 明示値がなければ nested-captures は全件受理。
    #[serde(default)]
    pub accepted: Option<bool>,
    #[serde(default)]
    pub prefix: Option<Prefix>,
    /// 捕捉の完了順（同名の nested capture は内側→外側）。
    #[serde(default)]
    pub captures: Option<Vec<Capture>>,
    /// nested-captures では欠落時も空の宣言列と比較する。
    #[serde(default)]
    pub declarations: Option<Vec<operations::Declaration>>,
    /// owned mixed 値の occurrence span。欠落時は空列が oracle。
    #[serde(default)]
    pub value_spans: Option<Vec<[usize; 2]>>,
    #[serde(default)]
    pub scope: Option<operations::ScopeState>,
    /// upstream はキーの有無で外側 transaction の rollback 検証を起動する。
    #[serde(default)]
    pub parent_rollback: Option<bool>,
    #[serde(flatten)]
    pub extra: RawFields,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    pub name: String,
    pub grammar: String,
    #[serde(default)]
    pub java_field_types: Option<BTreeMap<String, BTreeMap<String, String>>>,
    #[serde(default)]
    pub rust_field_types: Option<BTreeMap<String, BTreeMap<String, String>>>,
    #[serde(default)]
    pub capture_names: Option<Vec<String>>,
    #[serde(default)]
    pub value_span_field: Option<String>,
    #[serde(default)]
    pub value_span_list_field: Option<String>,
    #[serde(default)]
    pub queries: Option<Vec<String>>,
    /// 宣言/参照 annotation の静的 metadata。内側の拡張キーはそのまま保持。
    #[serde(default)]
    pub metadata: Option<Vec<Value>>,
    pub(crate) cases: Vec<Row<AstExpectation>>,
    #[serde(flatten)]
    pub extra: RawFields,
}

pub(crate) fn load(path: &Path) -> Result<CorpusFile, LoadError> {
    let (fixtures, raw) = read::<Vec<Fixture>>(path)?;
    let mut out = file(Family::InlineAst, path, raw);
    for (i, fixture) in fixtures.into_iter().enumerate() {
        out.grammars.push(GrammarSource::Inline {
            name: fixture.name.clone(),
            source: fixture.grammar,
            metadata: metadata(&out.raw[i]),
        });
        out.cases.extend(cases(
            fixture.cases,
            &fixture.name,
            vec![i],
            None,
            |expect| Expectation::Ast(Box::new(expect)),
        ));
    }
    Ok(out)
}
