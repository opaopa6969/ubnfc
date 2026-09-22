//! RustConformanceTest.tokenCorpus: value の存在が全入力受理の oracle。
use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenExpectation {
    /// None=不受理、Some(Null)=成功した optional capture。値は文字列以外もある。
    #[serde(
        default,
        deserialize_with = "present_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub value: Option<Value>,
    #[serde(default)]
    pub prefix: Option<Prefix>,
    #[serde(flatten)]
    pub extra: RawFields,
}

#[derive(Deserialize)]
pub struct Fixture {
    pub token: String,
    pub body: String,
    #[serde(default)]
    pub whitespace: Option<bool>,
    #[serde(default)]
    pub extra: Option<String>,
    pub(crate) cases: Vec<Row<TokenExpectation>>,
    #[serde(flatten)]
    pub metadata: RawFields,
}

pub(crate) fn load(path: &Path) -> Result<CorpusFile, LoadError> {
    let (fixtures, raw) = read::<Vec<Fixture>>(path)?;
    let mut out = file(Family::Token, path, raw);
    for (i, fixture) in fixtures.into_iter().enumerate() {
        let name = format!("{i}:{}", fixture.token);
        // upstream の fixture.has("whitespace") と同じく存在で判定する。
        let whitespace = if fixture.whitespace.is_some() {
            "@whitespace: javaStyle"
        } else {
            ""
        };
        let source = format!(
            "grammar Primitive {{ @package: org.example.primitive {whitespace}\n\
            token T = {}\ntoken END = EOF\n{}\n\
            @root @mapping(Value, params=[value]) Root ::= {}; }}",
            fixture.token,
            fixture.extra.unwrap_or_default(),
            fixture.body
        );
        out.grammars.push(GrammarSource::Inline {
            name: name.clone(),
            source,
            metadata: metadata(&out.raw[i]),
        });
        out.cases.extend(cases(
            fixture.cases,
            &name,
            vec![i],
            Some("Root"),
            Expectation::TokenValue,
        ));
    }
    Ok(out)
}
