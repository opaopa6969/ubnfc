//! semantic-cardinality と mixed-values は評価値、tiny 系は全入力受理と mapped 値。
//! 文字列 oracle の L:/T:/O: 等は変換せず独立期待値として保持する。
use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuleExpectation {
    #[serde(
        default,
        deserialize_with = "present_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub value: Option<Value>,
    #[serde(default)]
    pub texts: Option<Vec<TextSpan>>,
    #[serde(default)]
    pub nodes: Option<Vec<TextSpan>>,
    /// 受理する文法 variant の名前。未指定なら全 variant（value がある場合）。
    #[serde(default)]
    pub modes: Option<Vec<String>>,
    /// cardinality の簡略 AST 記述。[value,start,end] 等で canonical node ではない。
    #[serde(default)]
    pub expected: Option<Value>,
    #[serde(default)]
    pub prefix: Option<Prefix>,
    #[serde(flatten)]
    pub extra: RawFields,
}
impl RuleExpectation {
    pub fn accepted_in(&self, mode: &str) -> bool {
        self.value.is_some()
            && self
                .modes
                .as_ref()
                .is_none_or(|modes| modes.iter().any(|m| m == mode))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcceptExpectation {
    /// prefix 解析成功とは別の全入力受理。
    pub accepted: bool,
    pub prefix: Prefix,
    #[serde(default)]
    pub span: Option<[usize; 2]>,
    #[serde(default)]
    pub fields: Option<BTreeMap<String, Value>>,
    #[serde(
        default,
        deserialize_with = "present_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub value: Option<Value>,
    #[serde(flatten)]
    pub extra: RawFields,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticFixture {
    pub name: String,
    pub kind: String,
    pub cardinality: String,
    pub body: String,
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub extra: Option<String>,
    #[serde(default)]
    pub java_type: Option<String>,
    #[serde(default)]
    pub other: Option<bool>,
    pub(crate) cases: Vec<Row<RuleExpectation>>,
    #[serde(flatten)]
    pub metadata: RawFields,
}

#[derive(Deserialize)]
pub struct TinyFixture {
    pub name: String,
    pub body: String,
    #[serde(default)]
    pub params: Option<Vec<String>>,
    #[serde(default)]
    pub extra: Option<String>,
    #[serde(default)]
    pub whitespace: Option<bool>,
    pub(crate) cases: Vec<Row<AcceptExpectation>>,
    #[serde(flatten)]
    pub metadata: RawFields,
}

pub(crate) fn semantic(path: &Path) -> Result<CorpusFile, LoadError> {
    let (fixtures, raw) = read::<Vec<SemanticFixture>>(path)?;
    let template = read_text(&path.with_file_name("Grammar.ubnf.txt"))?;
    let mut out = file(Family::Rule, path, raw);
    for (i, fixture) in fixtures.into_iter().enumerate() {
        let source = template
            .replace(
                "ROOT_BODY",
                fixture.root.as_deref().unwrap_or("Helper @values"),
            )
            .replace("HELPER_BODY", &fixture.body)
            .replace("EXTRA_RULES", fixture.extra.as_deref().unwrap_or(""));
        out.grammars.push(GrammarSource::Inline {
            name: fixture.name.clone(),
            source,
            metadata: metadata(&out.raw[i]),
        });
        out.cases.extend(cases(
            fixture.cases,
            &fixture.name,
            vec![i],
            Some("Document"),
            Expectation::RuleValue,
        ));
    }
    Ok(out)
}

pub(crate) fn tiny(path: &Path, fence: bool) -> Result<CorpusFile, LoadError> {
    let (fixtures, raw) = read::<Vec<TinyFixture>>(path)?;
    let mut out = file(Family::Rule, path, raw);
    for (i, fixture) in fixtures.into_iter().enumerate() {
        let whitespace = if fixture.whitespace.is_some() {
            "@whitespace: javaStyle"
        } else {
            ""
        };
        let (grammar, package, tokens, mapping) = if fence {
            let params = fixture.params.as_deref().unwrap_or_default().join(", ");
            (
                "TinyFence",
                "org.example.tinyfence",
                "token START = org.unlaxer.tinyexpression.parser.javalang.CodeStartParser\n\
                 token CLOSE = org.unlaxer.tinyexpression.parser.javalang.CodeEndParser\n\
                 token LONG = org.unlaxer.tinyexpression.parser.javalang.LongCodeBlockParser\n\
                 token BODY = UNTIL('```')\ntoken CHAR = ANY\n",
                if params.is_empty() {
                    "@mapping(Fence)".into()
                } else {
                    format!("@mapping(Fence, params=[{params}])")
                },
            )
        } else {
            (
                "TinyString",
                "org.example.tinystring",
                "token STRING = org.unlaxer.tinyexpression.parser.StringLiteralParser\n",
                "@mapping(Value, params=[value])".into(),
            )
        };
        let source = format!(
            "grammar {grammar} {{ @package: {package} {whitespace}\n\
            {tokens}token END = EOF\n{}\n@root {mapping} Root ::= {}; }}",
            fixture.extra.unwrap_or_default(),
            fixture.body
        );
        out.grammars.push(GrammarSource::Inline {
            name: fixture.name.clone(),
            source,
            metadata: metadata(&out.raw[i]),
        });
        out.cases.extend(cases(
            fixture.cases,
            &fixture.name,
            vec![i],
            Some("Root"),
            Expectation::Accept,
        ));
    }
    Ok(out)
}

pub(crate) fn mixed(path: &Path, grammar: &str) -> Result<CorpusFile, LoadError> {
    let (rows, raw) = read::<Vec<Row<RuleExpectation>>>(path)?;
    let grammar_path = path.with_file_name(grammar);
    let source = read_text(&grammar_path)?;
    let shared = grammar == "Shared.ubnf";
    let mut out = file(Family::Rule, path, raw);
    out.grammars.push(super::grammar_path(
        grammar_path,
        if shared { "first" } else { "two" },
    )?);
    // RustConformanceTest の文法 variants。oracle 行や modes は展開・変更しない。
    let alternate = if shared {
        let first = "  @mapping(Value, params=[value]) First ::= 'f' Factor @value;";
        let second = "  @mapping(Value, params=[value]) Second ::= 's' LeafRule @value;";
        source.replace(&format!("{first}\n{second}"), &format!("{second}\n{first}"))
    } else {
        source.replace(" | OtherRule", "")
    };
    out.grammars.push(GrammarSource::Inline {
        name: if shared { "second" } else { "one" }.into(),
        source: alternate,
        metadata: Value::Null,
    });
    out.cases = cases(
        rows,
        grammar,
        vec![0, 1],
        Some(if shared { "Entry" } else { "Document" }),
        Expectation::RuleValue,
    );
    Ok(out)
}
