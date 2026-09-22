use serde_json::{json, Value};
use std::{collections::BTreeMap, fs};
use ubnfc_harness::{
    corpus::{operations::Operation, oracle},
    default_root, load_all, runner, Expectation, Family,
};

fn retains(measured: &Value, normalized: &Value) {
    match measured {
        Value::Object(fields) => {
            for (key, value) in fields {
                retains(
                    value,
                    normalized.get(key).unwrap_or_else(|| panic!("欠落: {key}")),
                );
            }
        }
        Value::Array(items) => {
            assert_eq!(items.len(), normalized.as_array().unwrap().len());
            for (i, value) in items.iter().enumerate() {
                retains(value, &normalized[i]);
            }
        }
        _ => assert_eq!(measured, normalized),
    }
}

#[test]
fn all_230_additional_observations_keep_their_family_counts_and_raw_fields() {
    let files = oracle::load_all(oracle::default_root()).unwrap();
    let added: Vec<_> = files
        .iter()
        .filter(|f| f.raw["collection"] == "triage-2026-09-22")
        .collect();
    assert_eq!(added.len(), 44);
    let mut counts = BTreeMap::new();
    for file in added {
        assert_eq!(file.family, Family::Oracle);
        assert_eq!(file.raw["stage"], "executed");
        assert_eq!(
            file.raw["source"],
            "ed4178e673f93e719b62a7bd181f3a219abf8ed1"
        );
        let kind = file.raw["observation"].as_str().unwrap();
        *counts.entry(kind).or_insert(0) += file.cases.len();
        let restored: ubnfc_harness::CorpusFile =
            serde_json::from_value(serde_json::to_value(file).unwrap()).unwrap();
        assert_eq!(file, &restored);
        for (case, row) in file.cases.iter().zip(file.raw["cases"].as_array().unwrap()) {
            assert_eq!(case.id, row["id"]);
            assert_eq!(case.input, row["input"]);
            let encoded = serde_json::to_value(&case.expect).unwrap();
            let expected = encoded.as_object().unwrap().values().next().unwrap();
            for (key, value) in row.as_object().unwrap() {
                if matches!(key.as_str(), "id" | "input") {
                    continue;
                }
                if matches!(case.expect, Expectation::Oracle { .. }) && expected.get(key).is_none()
                {
                    // 補助観測は runner の extra_expectations へ届くことを別途確認。
                    continue;
                }
                retains(value, &expected[key]);
            }
            match kind {
                "scope-store" => {
                    assert!(case.grammar_ref.is_empty());
                    assert!(matches!(case.expect, Expectation::ScopeOps(_)));
                    assert!(row.get("accepted").is_none());
                }
                "catalog" => {
                    assert!(matches!(case.expect, Expectation::Catalog(_)));
                    assert!(row.get("ast").is_none());
                }
                "cardinality" | "mixed-values" => {
                    let Expectation::Ast(e) = &case.expect else {
                        panic!()
                    };
                    assert_eq!(e.accepted, Some(false));
                    for key in [
                        "ast",
                        "value",
                        "texts",
                        "captures",
                        "diagnostics",
                        "declarations",
                    ] {
                        assert!(row.get(key).is_none(), "未観測: {key}");
                    }
                    assert_eq!(e.prefix.is_some(), kind == "mixed-values");
                }
                "oracle" => {
                    let origin = file.raw["origin"]
                        .as_str()
                        .unwrap()
                        .strip_prefix("oracle/")
                        .unwrap();
                    let original = files.iter().find(|f| f.path.ends_with(origin)).unwrap();
                    let old = original.raw["cases"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|c| c["id"] == row["id"])
                        .unwrap();
                    assert_eq!(row, old, "既存 JSON の実測値を変更しない");
                }
                _ => {}
            }
        }
    }
    assert_eq!(
        counts,
        BTreeMap::from([
            ("evolution", 148),
            ("cardinality", 20),
            ("mixed-values", 5),
            ("scope-store", 9),
            ("catalog", 2),
            ("scope-annotations", 6),
            ("nested-captures", 4),
            ("oracle", 36),
        ])
    );
}

#[test]
fn evolution_input_index_and_all_four_sidecars_have_the_same_versioned_oracles() {
    let files = load_all(default_root()).unwrap();
    let external = files
        .iter()
        .find(|f| f.path.ends_with("evolution/conformance.json"))
        .unwrap();
    let inputs: Vec<String> =
        serde_json::from_str(&fs::read_to_string(&external.path).unwrap()).unwrap();
    assert_eq!(inputs.len(), 37);
    assert_eq!(external.cases.len(), 148);
    let mut outcomes = Vec::new();
    for version in 0..4 {
        let sidecar = files
            .iter()
            .find(|f| f.path.ends_with(format!("evolution-v{version}.json")))
            .unwrap();
        assert_eq!(sidecar.cases.len(), 37);
        let selected = runner::select(std::slice::from_ref(sidecar), &[], None).unwrap();
        let (mut accepted, mut evaluated, mut errors) = (0, 0, 0);
        for (i, case) in sidecar.cases.iter().enumerate() {
            let alias = &external.cases[version * 37 + i];
            assert_eq!(case.input, inputs[i]);
            assert_eq!(case.expect, alias.expect);
            assert_eq!(alias.grammar_ref, [version]);
            assert_eq!(alias.id, format!("evolution/v{version}/{i}"));
            let Expectation::Oracle {
                accepted: Some(prefix),
                all_consumed: Some(full),
                ast,
                ..
            } = &case.expect
            else {
                panic!()
            };
            assert_eq!(ast.is_some(), *prefix && *full);
            accepted += usize::from(*prefix && *full);
            let raw = &sidecar.raw["cases"][i];
            let mut expected_alias = raw.clone();
            expected_alias["id"] = alias.id.clone().into();
            assert_eq!(external.raw["cases"][version * 37 + i], expected_alias);
            evaluated += usize::from(raw.get("value").is_some());
            errors += usize::from(raw.get("evaluationError").is_some());
            for key in [
                "value",
                "evaluationError",
                "nativeFailure",
                "parsedStatus",
                "scopeDepth",
            ] {
                assert_eq!(selected[i].extra_expectations.get(key), raw.get(key));
            }
            if raw.get("value").is_some() {
                assert!(raw.get("evaluationError").is_none());
            }
        }
        outcomes.push((accepted, evaluated, errors));
    }
    assert_eq!(
        outcomes,
        [(17, 12, 5), (18, 13, 5), (22, 17, 5), (25, 20, 5)]
    );
    // 入力索引から実行しても、評価値や診断等が失われない。
    let selected = runner::select(std::slice::from_ref(external), &[], None).unwrap();
    assert_eq!(selected.len(), 148);
    assert_eq!(
        selected
            .iter()
            .filter(|c| c.extra_expectations.contains_key("evaluationError"))
            .count(),
        20
    );
}

fn snapshot_count(ops: &[Operation]) -> usize {
    ops.iter()
        .map(|op| {
            usize::from(op.op == "observe")
                + op.operations.as_deref().map(snapshot_count).unwrap_or(0)
        })
        .sum()
}

#[test]
fn scope_snapshots_catalog_contexts_and_retained_spans_match_the_independent_fixtures() {
    let files = load_all(default_root()).unwrap();
    let mut snapshots = 0;
    let mut rollbacks = 0;
    let mut mixed_spans = 0;
    for file in files
        .iter()
        .filter(|f| f.raw["collection"] == "triage-2026-09-22")
    {
        let origin = file.raw["origin"].as_str().unwrap();
        let Some(original) = files.iter().find(|f| f.path.ends_with(origin)) else {
            continue;
        };
        for (case, row) in file.cases.iter().zip(file.raw["cases"].as_array().unwrap()) {
            let Some(old) = original.cases.iter().find(|c| c.id == case.id) else {
                continue;
            };
            match (&case.expect, &old.expect) {
                (Expectation::ScopeOps(measured), Expectation::ScopeOps(expected)) => {
                    assert_eq!(measured, expected);
                    snapshots += snapshot_count(&measured.operations);
                    assert!(file.grammars.is_empty());
                }
                (Expectation::Catalog(measured), Expectation::Catalog(expected)) => {
                    assert_eq!(measured, expected);
                    if case.id.contains("escaped-context") {
                        assert_eq!(measured.catalogs[1].context, "提示😀\n\t\r\\'\"");
                    }
                }
                (Expectation::Ast(measured), Expectation::Ast(expected)) => {
                    assert_eq!(measured.ast, expected.ast);
                    assert_eq!(measured.scope, expected.scope);
                    assert_eq!(measured.prefix, expected.prefix);
                    if file.raw["observation"] == "scope-annotations" {
                        assert_eq!(measured.parent_rollback, expected.parent_rollback);
                        if measured.parent_rollback.is_some() {
                            rollbacks += 1;
                            assert_eq!(row["rollback"]["cursor"], json!([0, 0]));
                            assert_eq!(row["rollback"]["scope"]["declarations"], json!([]));
                        }
                    } else {
                        assert_eq!(
                            measured.declarations.as_deref().unwrap_or_default(),
                            expected.declarations.as_deref().unwrap_or_default()
                        );
                        assert_eq!(
                            measured.value_spans.as_deref().unwrap_or_default(),
                            expected.value_spans.as_deref().unwrap_or_default()
                        );
                        assert_eq!(row["valueSpans"], row["retainedValueSpans"]);
                        if !row["valueSpans"].as_array().unwrap().is_empty() {
                            mixed_spans += 1;
                        }
                        assert!(
                            row.get("captures").is_none(),
                            "Java テストで採取しない Rust の capture 列を転記しない"
                        );
                    }
                }
                _ => {}
            }
        }
    }
    assert_eq!((snapshots, rollbacks, mixed_spans), (17, 2, 2));
}
