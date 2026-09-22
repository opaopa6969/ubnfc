//! 6 系統の独立 oracle。生成器・実行時から期待値を導出しない。
pub mod ast;
pub mod external;
pub mod operations;
pub mod oracle;
pub mod rule;
pub mod token;

use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
};

pub type RawFields = BTreeMap<String, Value>;
/// [prefix 成功, consumed, matched]。位置は UTF-16/byte でなく code point。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prefix(pub bool, pub usize, pub usize);
/// [start, end, text]。同値の値も occurrence ごとに順序と位置を保持する。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextSpan(pub usize, pub usize, pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Family {
    InlineAst,
    Token,
    Rule,
    ExternalGrammar,
    Operations,
    Oracle,
}
impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InlineAst => "文法内蔵 AST",
            Self::Token => "token 単体",
            Self::Rule => "ルール単体",
            Self::ExternalGrammar => "文法別置き",
            Self::Operations => "操作列・catalog",
            Self::Oracle => "oracle",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GrammarSource {
    Inline {
        name: String,
        source: String,
        metadata: Value,
    },
    File {
        name: String,
        path: PathBuf,
        metadata: Value,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorpusFile {
    pub family: Family,
    pub path: PathBuf,
    pub grammars: Vec<GrammarSource>,
    pub cases: Vec<Case>,
    /// 寛容処理後の元 JSON 全体。未知の fixture metadata も欠落させない。
    pub raw: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Case {
    /// ファイル内で一意で、読み込み順によって安定した ID。
    pub id: String,
    /// grammars の添字。複数なら同じ入力行を全 variant で試す。操作列は空。
    /// ケース件数は元の行/run 単位で数え、variant 数で水増ししない。
    pub grammar_ref: Vec<usize>,
    pub entry_rule: Option<String>,
    pub input: String,
    pub expect: Expectation,
    /// D-017 の follow-spec のみ。undecided は台帳に残し、比較の免除には使わない。
    #[serde(default)]
    pub known_divergences: Vec<crate::divergence::Observation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expectation {
    Ast(Box<ast::AstExpectation>),
    TokenValue(token::TokenExpectation),
    RuleValue(rule::RuleExpectation),
    Accept(rule::AcceptExpectation),
    Diagnostics(external::DiagnosticsExpectation),
    Numeric(external::NumericExpectation),
    ScopeOps(operations::ScopeExpectation),
    Catalog(operations::CatalogExpectation),
    /// Java を実走した観測。None は未観測であり false/0 とは異なる。
    #[serde(rename_all = "camelCase")]
    Oracle {
        accepted: Option<bool>,
        all_consumed: Option<bool>,
        consumed: Option<usize>,
        matched: Option<usize>,
        ast: Option<crate::canonical::AstValue>,
        captures: Vec<oracle::Capture>,
        declarations: Vec<operations::Declaration>,
        diagnostics: Vec<oracle::Diagnostic>,
        mapping_error: Option<String>,
    },
    /// parser 未実行。生成失敗と生成 Java のコンパイル失敗を原文付きで保持。
    GenerationFailure {
        message: String,
    },
    /// 明示的な負の契約だけ。単なる既存生成失敗を期待拒否に読み替えない。
    ExpectedRejection {
        stage: String,
        code: String,
        evidence: String,
        original: Box<Expectation>,
    },
    /// 例: evolution/conformance.json は入力列だけで独立した結果 oracle がない。
    Raw(Value),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadError {
    pub path: PathBuf,
    /// I/O エラーは位置不明のため None。JSON/型エラーは必ず行・列を持つ。
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
}
impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())?;
        if let (Some(line), Some(column)) = (self.line, self.column) {
            write!(f, ":{line}:{column}")?;
        }
        write!(f, ": {}", self.message)
    }
}
impl std::error::Error for LoadError {}

pub(crate) fn read_text(path: &Path) -> Result<String, LoadError> {
    fs::read_to_string(path).map_err(|e| LoadError {
        path: path.into(),
        line: None,
        column: None,
        message: e.to_string(),
    })
}

/// catalog 9 行目の非標準 \' のみを ' に戻す。\\ は一組として保持するので、
/// 正規の \\' や tiny-string の引用符 escape は壊さない。他の不正 JSON はエラー。
/// 改行は変えず、型エラーも元ファイルの行番号で報告する（列は補正後の列）。
fn normalize_json(text: &str) -> String {
    let mut chars = text.chars();
    let mut out = String::with_capacity(text.len());
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\'') => out.push('\''),
                Some(next) => {
                    out.push(c);
                    out.push(next);
                }
                None => out.push(c),
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub(crate) fn decode<T: DeserializeOwned>(path: &Path, text: &str) -> Result<T, LoadError> {
    serde_json::from_str(text).map_err(|e| LoadError {
        path: path.into(),
        line: Some(e.line()),
        column: Some(e.column()),
        message: e.to_string(),
    })
}

pub(crate) fn read<T: DeserializeOwned>(path: &Path) -> Result<(T, Value), LoadError> {
    let text = normalize_json(&read_text(path)?);
    Ok((decode(path, &text)?, decode(path, &text)?))
}

/// serde の通常の Option は missing と null を併合するため、present null も Some にする。
pub(crate) fn present_value<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Value>, D::Error> {
    Value::deserialize(d).map(Some)
}

#[derive(Deserialize)]
pub(crate) struct Row<T> {
    #[serde(default)]
    pub id: Option<String>,
    pub input: String,
    #[serde(flatten)]
    pub expect: T,
}

pub(crate) fn cases<T>(
    rows: Vec<Row<T>>,
    name: &str,
    refs: Vec<usize>,
    entry: Option<&str>,
    wrap: impl Fn(T) -> Expectation,
) -> Vec<Case> {
    rows.into_iter()
        .enumerate()
        .map(|(i, row)| Case {
            id: format!("{name}/{}", row.id.unwrap_or_else(|| i.to_string())),
            grammar_ref: refs.clone(),
            entry_rule: entry.map(str::to_owned),
            input: row.input,
            expect: wrap(row.expect),
            known_divergences: Vec::new(),
        })
        .collect()
}

pub(crate) fn metadata(raw: &Value) -> Value {
    let mut raw = raw.clone();
    if let Some(object) = raw.as_object_mut() {
        object.remove("cases");
        object.remove("runs");
        object.remove("grammar");
    }
    raw
}

pub(crate) fn file(family: Family, path: &Path, raw: Value) -> CorpusFile {
    CorpusFile {
        family,
        path: path.into(),
        grammars: Vec::new(),
        cases: Vec::new(),
        raw,
    }
}

pub(crate) fn grammar_path(path: PathBuf, name: &str) -> Result<GrammarSource, LoadError> {
    // 指定した .ubnf が実在し、UTF-8 で読めることも loader の段階で保証する。
    read_text(&path)?;
    Ok(GrammarSource::File {
        name: name.into(),
        path,
        metadata: Value::Null,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_fields_and_present_null_survive_typed_loading() {
        let row: Row<token::TokenExpectation> = decode(
            Path::new("future.json"),
            r#"{"input":"!","value":null,"future":{"x":[false,1.25,null]}}"#,
        )
        .unwrap();
        assert_eq!(row.expect.value, Some(Value::Null));
        assert_eq!(
            row.expect.extra["future"],
            serde_json::json!({"x":[false,1.25,null]})
        );
        let encoded = serde_json::to_value(&row.expect).unwrap();
        assert_eq!(encoded["future"], row.expect.extra["future"]);
        let expectation: token::TokenExpectation = serde_json::from_value(encoded).unwrap();
        assert_eq!(expectation, row.expect);
        let missing: Row<token::TokenExpectation> =
            decode(Path::new("future.json"), r#"{"input":"!"}"#).unwrap();
        assert_eq!(missing.expect.value, None);
        assert!(serde_json::to_value(missing.expect)
            .unwrap()
            .get("value")
            .is_none());
    }

    #[test]
    fn only_nonstandard_quote_escape_is_repaired() {
        assert_eq!(
            normalize_json(r#"["\'", "\\'", "\\\'", "\\q"]"#),
            r#"["'", "\\'", "\\'", "\\q"]"#
        );
        let err =
            decode::<Value>(Path::new("broken.json"), &normalize_json("[\n  \\\n]")).unwrap_err();
        assert_eq!(err.line, Some(2));
        assert!(err.to_string().starts_with("broken.json:2:"));
    }
}
