use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use ubnfc_front::resolve::resolve;
use ubnfc_front::syntax::parse_file;
use ubnfc_front::validate::validate;

fn fixture_root(variable: &str, default: &str) -> PathBuf {
    let root = std::env::var_os(variable).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join(default),
        PathBuf::from,
    );
    assert!(
        root.is_dir(),
        "fixture ディレクトリがありません: {} ({variable})",
        root.display()
    );
    root
}

const FILES: &[&str] = &[
    "story/code/calc-var.ubnf",
    "story/code/calc.ubnf",
    "story/code/calc-leftrec.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/boundaries.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/positive/elements.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/eval-default.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/keyword-boundary.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/namespace.ubnf",
    "rust/unlaxer-ubnf/tests/fixtures/known/common-field-space.ubnf",
    "unlaxer-dsl/src/test/resources/right-associative/Power.ubnf",
    "unlaxer-dsl/src/test/resources/mixed-values/Delimited.ubnf",
    "unlaxer-dsl/src/test/resources/mixed-values/Mixed.ubnf",
    "unlaxer-dsl/src/test/resources/mixed-values/Shared.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/0/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/1/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/2/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/evolution/3/Evolution.ubnf",
    "unlaxer-dsl/src/test/resources/associative/Operators.ubnf",
    "unlaxer-dsl/src/test/resources/cardinality/Cardinality.ubnf",
    "unlaxer-dsl/grammar/ubnf.ubnf",
    "unlaxer-dsl/tinycalc-vscode/grammar/tinycalc.ubnf",
    "unlaxer-dsl/examples/tinycalc.ubnf",
];

#[test]
fn external_fixture_diagnostics() {
    let unlaxer = fixture_root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser");
    let tiny = fixture_root("TINYEXPRESSION_DIR", "../../../tinyexpression");
    assert_eq!(FILES.len(), 24);
    let files = FILES
        .iter()
        .map(|p| (p.to_string(), unlaxer.join(p)))
        .chain(std::iter::once((
            "P4".into(),
            tiny.join("tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf"),
        )));
    let mut total_warnings = BTreeMap::new();
    let mut resolved_count = 0;
    let mut error_free_count = 0;
    for (name, path) in files {
        let source =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let ast = parse_file(&source).unwrap_or_else(|e| panic!("{name}: {e}"));
        // evolution/{0..3} を含め、異なる版は独立して解決する。
        let files = [(path, ast)];
        let result = resolve(&files, &|path| std::fs::read_to_string(path));
        if name.ends_with("/all-tokens.ubnf") || name.ends_with("/namespace.ubnf") {
            let errors = result.expect_err(&format!("既知の不正 fixture が受理された: {name}"));
            assert_eq!(errors.len(), 1, "{name}: {errors:?}");
            let error = &errors[0];
            if name.ends_with("/all-tokens.ubnf") {
                assert_eq!(error.code, "E-IMPORT-UNRESOLVED");
                assert_eq!(error.message, "import が解決できません: 'json.ubnf'");
            } else {
                // syntax-only fixture。a.b.Value は定義も import も存在しない。
                assert_eq!(error.code, "E-RULE-UNDEFINED");
                assert_eq!(
                    error.message,
                    "rule Root references undefined rule or token 'a.b.Value'"
                );
            }
            continue;
        }
        match result {
            Ok(grammar) => {
                resolved_count += 1;
                let issues = validate(&grammar);
                let mut warnings = BTreeMap::new();
                for issue in issues.iter().filter(|i| i.severity() == "WARNING") {
                    *warnings.entry(issue.code.as_str()).or_insert(0usize) += 1;
                    *total_warnings.entry(issue.code.clone()).or_insert(0usize) += 1;
                }
                let expected = if name == "story/code/calc-leftrec.ubnf" {
                    BTreeMap::from([("W-LEFT-RECURSION", 2)])
                } else if name.ends_with("/keyword-boundary.ubnf") {
                    // `@rooted` は `@root` の綴り間違い（D-045 で初めて診断が付いた）。
                    // entry が `lower/mod.rs` の「最初の規則」fallback に落ちて動いていた。
                    BTreeMap::from([("W-GENERAL-NO-ROOT", 1), ("W-ANNOTATION-UNKNOWN", 1)])
                } else if ["/eval-default.ubnf", "/common-field-space.ubnf"]
                    .iter()
                    .any(|suffix| name.ends_with(suffix))
                {
                    BTreeMap::from([("W-GENERAL-NO-ROOT", 1)])
                } else if name.ends_with("/positive/annotations.ubnf") {
                    // 合成 fixture の `@custom`（D-045）。
                    BTreeMap::from([("W-ANNOTATION-UNKNOWN", 1)])
                } else if name == "unlaxer-dsl/grammar/ubnf.ubnf" {
                    // Capability metadata is not yet an ubnfc semantic contract.
                    BTreeMap::from([
                        ("W-ANNOTATION-UNKNOWN", 4),
                        ("W-ANNOTATION-BACKREF-AMBIGUOUS", 1),
                    ])
                } else {
                    BTreeMap::new()
                };
                assert_eq!(warnings, expected, "{name}");
                let errors: Vec<_> = issues.iter().filter(|i| i.severity() == "ERROR").collect();
                if name.ends_with("/positive/annotations.ubnf") {
                    // Java 実装も12件を拒否する。この構文fixtureを特別に受理しない。
                    let actual: Vec<_> = errors.iter().map(|i| json!({"grammar":i.grammar,"rule":i.rule,"code":i.code,"message":i.message,"hint":i.hint,"severity":i.severity(),"category":i.category()})).collect();
                    let mut expected: Vec<Value> =
                        serde_json::from_str(include_str!("fixtures/annotations-java.json"))
                            .unwrap();
                    expected.sort_by_key(|v| {
                        ["grammar", "rule", "code", "message"]
                            .map(|k| v[k].as_str().unwrap_or("").to_owned())
                    });
                    assert_eq!(expected.len(), 12);
                    assert_eq!(actual, expected, "{name}");
                } else {
                    assert!(errors.is_empty(), "{name}: {errors:?}");
                    error_free_count += 1;
                }
                // 再実行してシンボル採番・出所・site path も含めた決定性を確認する。
                if name == "P4" {
                    assert_eq!(
                        grammar,
                        resolve(&files, &|path| std::fs::read_to_string(path)).unwrap()
                    );
                    assert_eq!(grammar.grammars[0].rules.len(), 124);
                    assert!(issues.is_empty());
                    for rule in &grammar.grammars[0].rules {
                        for site in &rule.captures {
                            assert!(
                                site.span.start <= site.span.end && site.span.end <= source.len()
                            );
                        }
                    }
                }
            }
            Err(errors) => panic!("{name}: {errors:?}"),
        }
    }
    assert_eq!(resolved_count, 23);
    assert_eq!(error_free_count, 22);
    assert_eq!(
        total_warnings,
        BTreeMap::from([
            ("W-LEFT-RECURSION".into(), 2),
            ("W-GENERAL-NO-ROOT".into(), 3),
            ("W-ANNOTATION-UNKNOWN".into(), 6),
            ("W-ANNOTATION-BACKREF-AMBIGUOUS".into(), 1)
        ])
    );
}
