//! 適合性 corpus の読み込み・正規化と canonical AST 比較。生成・実行は扱わない。
pub mod canonical;
pub mod corpus;

pub use corpus::{Case, CorpusFile, Expectation, Family, GrammarSource, LoadError};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// 相対の環境変数は呼出時の cwd 基準。既定値は crate ディレクトリ基準なので
/// workspace ルートから cargo test / cargo run しても同じ sibling repo を参照する。
pub fn default_root() -> PathBuf {
    std::env::var_os("UNLAXER_PARSER_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../unlaxer-parser"))
}

/// root は unlaxer-parser repo。欠落や不正ファイルはパス付きエラー（skip しない）。
/// JSON 件数には文法ファイルや .txt テンプレートを数えない。
pub fn load_all(root: impl AsRef<Path>) -> Result<Vec<CorpusFile>, LoadError> {
    use corpus::{ast, external, operations, rule, token};
    let resources = root.as_ref().join("unlaxer-dsl/src/test/resources");
    let mut out = Vec::new();
    for name in ["nested-captures", "rule-trivia", "scope-annotations"] {
        out.push(ast::load(&resources.join(name).join("corpus.json"))?);
    }
    for name in ["primitives", "lexical"] {
        out.push(token::load(&resources.join(name).join("corpus.json"))?);
    }
    for name in ["corpus.json", "pure-alias.json"] {
        out.push(rule::semantic(
            &resources.join("semantic-cardinality").join(name),
        )?);
    }
    out.push(rule::tiny(
        &resources.join("tiny-code-fence/corpus.json"),
        true,
    )?);
    out.push(rule::tiny(
        &resources.join("tiny-string-token/corpus.json"),
        false,
    )?);
    for (corpus, grammar) in [
        ("corpus.json", "Mixed.ubnf"),
        ("delimited-corpus.json", "Delimited.ubnf"),
        ("shared-corpus.json", "Shared.ubnf"),
    ] {
        out.push(rule::mixed(
            &resources.join("mixed-values").join(corpus),
            grammar,
        )?);
    }
    out.push(external::evolution(
        &resources.join("evolution/conformance.json"),
        false,
    )?);
    out.push(external::evolution(
        &resources.join("evolution/diagnostics.json"),
        true,
    )?);
    for (name, grammar) in [
        ("associative", "Operators.ubnf"),
        ("right-associative", "Power.ubnf"),
        ("cardinality", "Cardinality.ubnf"),
    ] {
        out.push(external::values(
            &resources.join(name).join("corpus.json"),
            grammar,
        )?);
    }
    out.push(external::numeric(
        &resources.join("numeric-capture/conformance.json"),
    )?);
    out.push(operations::scope(
        &resources.join("scope-store/corpus.json"),
    )?);
    out.push(operations::catalog(&resources.join("catalog/corpus.json"))?);
    Ok(out)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub files: usize,
    pub cases: usize,
}

pub fn summary(files: &[CorpusFile]) -> BTreeMap<Family, Counts> {
    let mut counts = BTreeMap::<Family, Counts>::new();
    for file in files {
        let count = counts.entry(file.family).or_default();
        count.files += 1;
        count.cases += file.cases.len();
    }
    counts
}
