//! 仕様由来オラクル（`corpus/spec/**`）の形と、引用が実在することの検査。
//!
//! ケース集そのものはこの checkout にあるので必ず読む。引用先は sibling repository
//! （`unlaxer-parser` / `tinyexpression`）なので、無い環境では照合だけを省略する。

use std::path::{Path, PathBuf};
use ubnfc_harness::corpus::spec::{self, Scope, SpecFile, Verdict};

fn sibling(variable: &str, name: &str) -> PathBuf {
    std::env::var_os(variable).map_or_else(
        || {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../..")
                .join(name)
        },
        PathBuf::from,
    )
}

fn load(family: &str) -> Vec<SpecFile> {
    spec::load_family(spec::default_root(), family)
        .unwrap_or_else(|error| panic!("{family} の読み込みに失敗: {error}"))
}

/// 見出し行が実在し、引用文が逐語で含まれることを確かめる。
/// 引用文を実装の挙動に合わせて書き換えると、この corpus は仕様由来でなくなる。
fn check_citations(files: &[SpecFile], root: &Path, label: &str) -> usize {
    if !root.is_dir() {
        eprintln!("{label} が無いため引用の照合を省略: {}", root.display());
        return 0;
    }
    let mut checked = 0;
    for file in files {
        for case in &file.cases {
            let spec_path = root.join(&case.citation.file);
            let text = std::fs::read_to_string(&spec_path)
                .unwrap_or_else(|e| panic!("{}: {e}", spec_path.display()));
            let headings: Vec<String> = text
                .lines()
                .filter(|line| line.starts_with('#'))
                .map(|line| line.trim().to_owned())
                .collect();
            assert!(
                headings.iter().any(|h| h == &case.citation.heading),
                "{}: 見出しが {} に無い: {}",
                case.id,
                case.citation.file,
                case.citation.heading
            );
            let body = spec::normalize(&text);
            assert!(
                body.contains(&spec::normalize(&case.citation.sentence)),
                "{}: 引用文が {} に逐語で無い: {}",
                case.id,
                case.citation.file,
                case.citation.sentence
            );
            checked += 1;
        }
    }
    checked
}

#[test]
fn ubnf_cases_load_and_cover_the_spec_sections() {
    let files = load("ubnf");
    assert!(
        files.len() >= 8,
        "節ごとのファイルが少なすぎる: {}",
        files.len()
    );
    let total: usize = files.iter().map(|f| f.cases.len()).sum();
    assert!(total >= 100, "ケースが少なすぎる: {total}");
    // 仕様が沈黙している問いを「推測で埋めない」ことが corpus の性質なので、
    // undetermined が 1 件も無い状態は書き方を誤った兆候として扱う。
    let undetermined = files
        .iter()
        .flat_map(|f| &f.cases)
        .filter(|c| c.expectation.verdict == Verdict::Undetermined)
        .count();
    assert!(undetermined > 0, "undetermined が 0 件");
    // validate scope は診断コードを、parse scope は受理判定を持つ。
    for case in files.iter().flat_map(|f| &f.cases) {
        if case.scope == Scope::Validate {
            assert_eq!(
                case.expectation.verdict,
                Verdict::Diagnostic,
                "{}: validate scope は diagnostic 期待であるべき",
                case.id
            );
        }
    }
}

#[test]
fn ubnf_citations_resolve_to_real_spec_headings() {
    let files = load("ubnf");
    let root = sibling("UNLAXER_PARSER_DIR", "unlaxer-parser");
    let checked = check_citations(&files, &root, "unlaxer-parser");
    if root.is_dir() {
        assert_eq!(checked, files.iter().map(|f| f.cases.len()).sum::<usize>());
    }
}

#[test]
fn p4_cases_load_and_citations_resolve() {
    let files = load("p4");
    assert!(!files.is_empty(), "P4 のケース集が無い");
    let root = sibling("TINYEXPRESSION_DIR", "tinyexpression");
    let checked = check_citations(&files, &root, "tinyexpression");
    if root.is_dir() {
        assert_eq!(checked, files.iter().map(|f| f.cases.len()).sum::<usize>());
    }
}
