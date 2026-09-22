use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
use ubnfc_harness::{
    canonical::AstValue, default_root, load_all, summary, CorpusFile, Counts, Expectation, Family,
    GrammarSource,
};

fn corpus() -> &'static [CorpusFile] {
    static CORPUS: OnceLock<Vec<CorpusFile>> = OnceLock::new();
    CORPUS.get_or_init(|| load_all(default_root()).unwrap_or_else(|error| panic!("{error}")))
}
fn file(suffix: &str) -> &'static CorpusFile {
    corpus()
        .iter()
        .find(|file| file.path.ends_with(suffix))
        .unwrap()
}

#[test]
fn every_required_file_loads_with_the_measured_counts() {
    use Family::*;
    let expected = [
        ("nested-captures/corpus.json", InlineAst, 10),
        ("rule-trivia/corpus.json", InlineAst, 33),
        ("scope-annotations/corpus.json", InlineAst, 14),
        ("primitives/corpus.json", Token, 109),
        ("lexical/corpus.json", Token, 78),
        ("semantic-cardinality/corpus.json", Rule, 94),
        ("semantic-cardinality/pure-alias.json", Rule, 69),
        ("tiny-code-fence/corpus.json", Rule, 73),
        ("tiny-string-token/corpus.json", Rule, 39),
        ("mixed-values/corpus.json", Rule, 23),
        ("mixed-values/delimited-corpus.json", Rule, 15),
        ("mixed-values/shared-corpus.json", Rule, 11),
        ("evolution/conformance.json", ExternalGrammar, 148),
        ("evolution/diagnostics.json", ExternalGrammar, 8),
        ("associative/corpus.json", ExternalGrammar, 24),
        ("right-associative/corpus.json", ExternalGrammar, 28),
        ("cardinality/corpus.json", ExternalGrammar, 14),
        ("numeric-capture/conformance.json", ExternalGrammar, 13),
        ("scope-store/corpus.json", Operations, 9),
        ("catalog/corpus.json", Operations, 2),
    ];
    assert_eq!(corpus().len(), expected.len() + 107);
    for (suffix, family, count) in expected {
        let file = file(suffix);
        assert_eq!((file.family, file.cases.len()), (family, count), "{suffix}");
    }
    for file in corpus() {
        let mut ids = BTreeSet::new();
        for case in &file.cases {
            assert!(
                ids.insert(&case.id),
                "{}: duplicate {}",
                file.path.display(),
                case.id
            );
            assert!(case.grammar_ref.iter().all(|i| *i < file.grammars.len()));
            assert_eq!(
                case.grammar_ref.is_empty(),
                matches!(case.expect, Expectation::ScopeOps(_))
            );
        }
    }
    assert_eq!(
        summary(corpus()),
        BTreeMap::from([
            (
                InlineAst,
                Counts {
                    files: 3,
                    cases: 57
                }
            ),
            (
                Token,
                Counts {
                    files: 2,
                    cases: 187
                }
            ),
            (
                Rule,
                Counts {
                    files: 7,
                    cases: 324
                }
            ),
            (
                ExternalGrammar,
                Counts {
                    files: 6,
                    cases: 235
                }
            ),
            (
                Operations,
                Counts {
                    files: 2,
                    cases: 11
                }
            ),
            (
                Oracle,
                Counts {
                    files: 107,
                    cases: 398
                }
            ),
        ])
    );
    let total: usize = corpus().iter().map(|file| file.cases.len()).sum();
    assert_eq!(total, 1212);
}

// 各元 row の期待値キーを正規化側と照合。型付けによるキーや null の欠落を検出する。
fn retained(original: &Value, normalized: &Value) {
    match original {
        Value::Object(object) => {
            for (key, value) in object {
                assert!(normalized.get(key).is_some(), "missing {key}: {normalized}");
                retained(value, &normalized[key]);
            }
        }
        Value::Array(array) => {
            assert_eq!(Some(array.len()), normalized.as_array().map(Vec::len));
            for (i, value) in array.iter().enumerate() {
                retained(value, &normalized[i]);
            }
        }
        _ => assert_eq!(original, normalized),
    }
}

#[test]
fn original_oracles_and_metadata_survive_normalization_and_serde_roundtrip() {
    for file in corpus() {
        let serialized = serde_json::to_value(file).unwrap();
        let restored: CorpusFile = serde_json::from_value(serialized).unwrap();
        assert_eq!(*file, restored, "{}", file.path.display());
        if file.path.ends_with("evolution/conformance.json") {
            let raw: Value =
                serde_json::from_str(&std::fs::read_to_string(&file.path).unwrap()).unwrap();
            assert_eq!(file.raw["inputs"], raw);
            // 版別 sidecar との全項目照合は conformance_oracle.rs。
            continue;
        }
        if !file.path.ends_with("catalog/corpus.json") {
            let raw: Value =
                serde_json::from_str(&std::fs::read_to_string(&file.path).unwrap()).unwrap();
            assert_eq!(file.raw, raw);
        }
        if file.family == Family::Oracle {
            // object 形式の実走 oracle は tests/oracle.rs で各フィールドを照合する。
            continue;
        }
        let mut rows = Vec::new();
        for fixture in file.raw.as_array().unwrap() {
            if let Some(cases) = fixture.get("cases") {
                rows.extend(cases.as_array().unwrap().iter());
            } else if let Some(runs) = fixture.get("runs") {
                rows.extend(runs.as_array().unwrap().iter());
            } else {
                rows.push(fixture);
            }
        }
        assert_eq!(rows.len(), file.cases.len());
        for (row, case) in rows.into_iter().zip(&file.cases) {
            let mut oracle = row.clone();
            if let Some(object) = oracle.as_object_mut() {
                object.remove("input");
                object.remove("id");
                if matches!(case.expect, Expectation::Catalog(_)) {
                    object.remove("name");
                    object.remove("grammar");
                }
            }
            let expected = match &case.expect {
                Expectation::Numeric(n) => json!({"javaInt":n.java_int}),
                expectation => {
                    let wrapped = serde_json::to_value(expectation).unwrap();
                    wrapped
                        .as_object()
                        .unwrap()
                        .values()
                        .next()
                        .unwrap()
                        .clone()
                }
            };
            retained(&oracle, &expected);
            if let Expectation::Ast(expected) = &case.expect {
                if let Some(ast) = &expected.ast {
                    assert_eq!(AstValue::try_from(Value::from(ast)).unwrap(), *ast);
                }
            }
        }
    }
}

#[test]
fn prefix_full_acceptance_null_and_backend_specific_expectations_stay_distinct() {
    let lexical = file("lexical/corpus.json");
    assert!(lexical.cases.iter().any(|c| matches!(&c.expect,
        Expectation::TokenValue(e) if e.value == Some(Value::Null))));
    assert!(
        lexical.cases.iter().any(|c| matches!(&c.expect,
        Expectation::TokenValue(e) if e.value.is_none() && e.prefix.as_ref().is_some_and(|p| p.0)))
    );
    assert!(file("rule-trivia/corpus.json").cases.iter().any(|c| matches!(&c.expect,
        Expectation::Ast(e) if e.accepted == Some(false) && e.prefix.as_ref().is_some_and(|p| p.0))));
    let diagnostic = file("evolution/diagnostics.json")
        .cases
        .iter()
        .find(|c| c.input == "neg(1e+)")
        .unwrap();
    let Expectation::Diagnostics(e) = &diagnostic.expect else {
        panic!()
    };
    assert_eq!((&*e.kind, e.java_offset, e.rust_offset), ("syntax", 7, 5));
    for case in &file("numeric-capture/conformance.json").cases {
        let Expectation::Numeric(e) = &case.expect else {
            panic!()
        };
        assert_eq!(case.input, format!("n{};", e.rust_lexeme));
        assert_eq!(e.java_int, e.rust_lexeme.parse::<i32>().ok());
    }
    let numeric = &file("numeric-capture/conformance.json").cases[5];
    let Expectation::Numeric(e) = &numeric.expect else {
        panic!()
    };
    assert_eq!((&*e.rust_lexeme, e.java_int), ("0004", Some(4)));
}

#[test]
fn grammar_variants_and_catalog_metadata_are_preserved_without_inventing_an_oracle() {
    let evolution = file("evolution/conformance.json");
    assert_eq!(evolution.grammars.len(), 4);
    for (i, case) in evolution.cases.iter().enumerate() {
        assert_eq!(case.grammar_ref, [i / 37]);
        assert!(matches!(case.expect, Expectation::Oracle { .. }));
    }
    for (suffix, count) in [
        ("right-associative/corpus.json", 4),
        ("cardinality/corpus.json", 5),
        ("associative/corpus.json", 2),
        ("mixed-values/corpus.json", 2),
    ] {
        let corpus = file(suffix);
        assert_eq!(corpus.grammars.len(), count);
        let sources: BTreeSet<_> = corpus
            .grammars
            .iter()
            .map(|g| match g {
                GrammarSource::Inline { source, .. } => source.clone(),
                GrammarSource::File { path, .. } => std::fs::read_to_string(path).unwrap(),
            })
            .collect();
        assert_eq!(
            sources.len(),
            count,
            "{suffix}: variant transformation failed"
        );
    }
    let shared = file("mixed-values/shared-corpus.json");
    let GrammarSource::Inline { source, .. } = &shared.grammars[1] else {
        panic!()
    };
    assert!(source.find("Second ::=").unwrap() < source.find("First ::=").unwrap());
    let fence = file("tiny-code-fence/corpus.json");
    let GrammarSource::Inline { source, .. } = &fence.grammars[0] else {
        panic!()
    };
    assert!(source.contains("org.unlaxer.tinyexpression.parser.javalang.CodeStartParser"));
    let catalog = file("catalog/corpus.json");
    let Expectation::Catalog(e) = &catalog.cases[1].expect else {
        panic!()
    };
    assert_eq!(e.catalogs[1].context, "提示😀\n\t\r\\'\"");
    assert_eq!(
        e.catalogs[1].captures,
        ["alpha", "bounded", "plus", "repeat", "separated", "zeta"]
    );
    assert_eq!(
        e.catalogs
            .iter()
            .map(|c| c.rule.as_str())
            .collect::<Vec<_>>(),
        ["Root", "Helper", "Child"]
    );
    let GrammarSource::Inline { metadata, .. } = &file("nested-captures/corpus.json").grammars[0]
    else {
        panic!()
    };
    assert_eq!(metadata["javaFieldTypes"]["Root"]["value"], "List<String>");
    assert_eq!(metadata["rustFieldTypes"]["Root"]["value"], "Vec<String>");
}
