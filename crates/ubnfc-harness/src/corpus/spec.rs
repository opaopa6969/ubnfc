//! 仕様由来オラクル（`corpus/spec/**`）の読み込み。
//!
//! `corpus/oracle/` が旧 Java 実装の**観測値**を固定したものであるのに対し、こちらは
//! 規範文書の文だけから導いた期待値を持つ。各ケースは引用元（ファイル・見出し・逐語の
//! 一文）を必ず持ち、`tests/spec_corpus.rs` が引用が実在することを検査する。
//! ここは読み込みと形の検査だけで、実装を走らせる比較は
//! `examples/{ubnf,p4}-crosscheck/scripts/run-spec.sh` が行う。

use super::{decode, read_text, LoadError};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

/// ケースを駆動できる層。期待値の意味ではなく「どこで観測するか」を表す。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// ソースの受理・拒否・構文木の形。4 実装すべてで観測できる。
    Parse,
    /// 文法の意味制約（診断コード）。validator を持つ実装だけで観測できる。
    Validate,
    /// 生成パーサの実行時挙動。構文木の比較では観測できない。
    Runtime,
}

/// 引用元。`file` は sibling repository のルート相対。
#[derive(Clone, Debug, Deserialize)]
pub struct Citation {
    pub file: String,
    /// 仕様ファイルに実在する見出し行そのもの（`## …` を含む）。
    pub heading: String,
    /// 逐語の一文。空白の畳み込みだけを許して照合する。
    pub sentence: String,
}

/// 期待値。`verdict` ごとに意味のあるフィールドが変わる。
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expectation {
    pub verdict: Verdict,
    /// `accept` のときだけ: 正準 AST JSON に必ず現れる部分文字列。
    #[serde(default)]
    pub canonical_contains: Vec<String>,
    /// `accept` のときだけ: 構文木が一致すべき別ケースの ID。
    #[serde(default)]
    pub same_canonical_as: Option<String>,
    /// `diagnostic` のときだけ: 期待する診断コード。
    #[serde(default)]
    pub code: Option<String>,
    /// `undetermined` のときだけ: 仕様から何が決まらないのか。
    #[serde(default)]
    pub question: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Accept,
    Reject,
    Diagnostic,
    Undetermined,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpecCase {
    pub id: String,
    pub scope: Scope,
    pub citation: Citation,
    pub input: String,
    pub expectation: Expectation,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpecFile {
    pub family: String,
    pub section: String,
    pub summary: String,
    pub cases: Vec<SpecCase>,
    #[serde(skip)]
    pub path: PathBuf,
}

/// `corpus/spec`。cwd にも sibling repository にも依存しない。
pub fn default_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/spec")
}

fn invalid(path: &Path, message: impl Into<String>) -> LoadError {
    LoadError {
        path: path.into(),
        line: None,
        column: None,
        message: message.into(),
    }
}

/// `root/<family>/*.json` を名前順に読む。family ディレクトリが無ければ空。
pub fn load_family(root: impl AsRef<Path>, family: &str) -> Result<Vec<SpecFile>, LoadError> {
    let directory = root.as_ref().join(family);
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(&directory)
        .map_err(|e| invalid(&directory, e.to_string()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    let mut files = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for path in paths {
        let text = read_text(&path)?;
        let mut file: SpecFile = decode(&path, &text)?;
        if file.family != family {
            return Err(invalid(
                &path,
                format!("family が {family} ではなく {}", file.family),
            ));
        }
        if file.cases.is_empty() {
            return Err(invalid(&path, "ケースが 0 件"));
        }
        for case in &file.cases {
            if !seen.insert(case.id.clone()) {
                return Err(invalid(&path, format!("ケース ID の重複: {}", case.id)));
            }
            check_case(&path, case)?;
        }
        file.path = path;
        files.push(file);
    }
    // sameCanonicalAs の参照先が同じ family の中に実在することを確かめる。
    for file in &files {
        for case in &file.cases {
            if let Some(other) = &case.expectation.same_canonical_as {
                if !seen.contains(other) {
                    return Err(invalid(
                        &file.path,
                        format!("{}: sameCanonicalAs の参照先が無い: {other}", case.id),
                    ));
                }
            }
        }
    }
    Ok(files)
}

fn check_case(path: &Path, case: &SpecCase) -> Result<(), LoadError> {
    let fail = |message: String| Err(invalid(path, format!("{}: {message}", case.id)));
    if case.citation.sentence.trim().is_empty() {
        return fail("引用文が空".to_owned());
    }
    if !case.citation.heading.starts_with('#') {
        return fail("heading は見出し行そのものである必要がある".to_owned());
    }
    match case.expectation.verdict {
        Verdict::Diagnostic if case.expectation.code.is_none() => {
            fail("diagnostic には code が要る".to_owned())
        }
        Verdict::Undetermined if case.expectation.question.is_none() => {
            fail("undetermined には question が要る".to_owned())
        }
        Verdict::Accept => Ok(()),
        Verdict::Reject
            if !case.expectation.canonical_contains.is_empty()
                || case.expectation.same_canonical_as.is_some() =>
        {
            fail("reject に構文木の期待は書けない".to_owned())
        }
        _ => Ok(()),
    }
}

/// 引用文・見出しの照合に使う正規化。改行や連続空白の違いだけを吸収する。
pub fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
