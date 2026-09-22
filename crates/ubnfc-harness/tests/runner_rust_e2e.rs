use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use ubnfc_harness::{
    default_root, load_all,
    runner::{select, Artifact, Backend, Options, Report, RunCase, Runner, Scratch},
    Case,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn artifact(ir: &str) -> Artifact {
    Artifact::FromIr {
        ir_path: root().join(ir),
        backend: Backend::Rust,
    }
}
fn run_family(ir: &str, family: &str, count: usize) {
    let scratch = Scratch::new().unwrap();
    let runner = Runner {
        backend: Backend::Rust,
        build_dir: scratch.0.clone(),
    };
    let executable = runner.build(artifact(ir)).unwrap();
    let cases = select(&load_all(default_root()).unwrap(), &[family.into()], None).unwrap();
    assert_eq!(cases.len(), count);
    let actual = runner.run(&executable, &cases);
    let report = Report::from_actual(&cases, &actual).unwrap();
    assert_eq!(
        (
            report.pass,
            report.known_divergence,
            report.fail.len(),
            report.unobserved,
            report.unobserved_cases
        ),
        (count, 0, 0, 0, 0),
        "{}",
        report.markdown()
    );
    let counts = &report.families[family];
    assert_eq!(
        (
            counts.pass,
            counts.known_divergence,
            counts.fail,
            counts.unobserved,
            counts.unobserved_cases
        ),
        (count, 0, 0, 0, 0)
    );
    for row in &actual {
        for key in [
            "accepted",
            "allConsumed",
            "consumed",
            "matched",
            "ast",
            "captures",
            "declarations",
            "references",
            "diagnostics",
            "mappingError",
            "value",
        ] {
            assert!(row.response.get(key).is_some(), "{key}: {}", row.response);
        }
    }
    if family == "associative" {
        let index = cases
            .iter()
            .position(|case| case.case.input == "1e2/4-0.5")
            .unwrap();
        assert_eq!(actual[index].response["value"], 24.5);
        assert_eq!(actual[index].response["mappingError"], Value::Null);
        assert!(actual[index].response["ast"].is_object());
    }
}
#[test]
fn generated_operators_pass_all_24_without_exclusions() {
    run_family(
        "crates/ubnfc-rust/tests/ir/operators.ir.json",
        "associative",
        24,
    );
}
#[test]
fn generated_power_pass_all_28_without_exclusions() {
    run_family(
        "crates/ubnfc-rust/tests/ir/power.ir.json",
        "right-associative",
        28,
    );
}
fn tiny_case(id: usize, input: &str, accepted: bool, entry: Option<&str>) -> RunCase {
    let prefix = if accepted {
        json!([true, input.chars().count(), input.chars().count()])
    } else if input == "1+" {
        json!([true, 1, 1])
    } else {
        json!([false, 0, 0])
    };
    RunCase {
        case: Case {
            id: format!("tiny/{id}"),
            grammar_ref: vec![0],
            entry_rule: entry.map(str::to_owned),
            input: input.into(),
            expect: serde_json::from_value(json!({"Accept":{"accepted":accepted,"prefix":prefix}}))
                .unwrap(),
            known_divergences: vec![],
        },
        grammar: "TinyCalc".into(),
        variant: "normal".into(),
        family: "tinycalc".into(),
        options: Options {
            lexical: true,
            scope: true,
        },
        fixture: Value::Null,
        extra_expectations: BTreeMap::new(),
    }
}
#[test]
fn tinycalc_acceptance_entries_and_options_through_driver() {
    let scratch = Scratch::new().unwrap();
    let runner = Runner {
        backend: Backend::Rust,
        build_dir: scratch.0.clone(),
    };
    let executable = runner
        .build(artifact("ir/examples/tinycalc.ir.json"))
        .unwrap();
    let examples = [
        ("1 + 2 * 3", true, None),
        ("var x set 1; x + 2", true, None),
        ("/*😀*/ 1+2", true, Some("TinyCalc")),
        ("", false, None),
        ("@", false, None),
        ("1+", false, None),
        ("1+2", true, Some("Expression")),
        ("+", true, Some("AddOp")),
    ];
    let mut cases: Vec<_> = examples
        .iter()
        .enumerate()
        .map(|(id, (input, ok, entry))| tiny_case(id, input, *ok, *entry))
        .collect();
    let actual = runner.run(&executable, &cases);
    let report = Report::from_actual(&cases, &actual).unwrap();
    assert_eq!(
        (
            report.pass,
            report.known_divergence,
            report.fail.len(),
            report.unobserved,
            report.unobserved_cases
        ),
        (8, 0, 0, 0, 0),
        "{}",
        report.markdown()
    );
    assert_eq!(actual[0].response["ast"]["type"], "TinyCalcProgram");
    assert_eq!(actual[0].response["value"], Value::Null);
    assert_eq!(
        actual[2].response["consumed"],
        examples[2].0.chars().count()
    );
    assert_eq!(actual[5].response["accepted"], true);
    assert_eq!(actual[5].response["ast"], Value::Null);
    assert_eq!(actual[6].response["value"], 3.0);
    assert_eq!(actual[7].response["value"], Value::Null);
    cases[0].options = Options::default();
    cases[1].case.entry_rule = Some("Unknown".into());
    cases[2].grammar = "Unknown".into();
    let rows = runner.run(&executable, &cases[..4]);
    for key in ["captures", "declarations", "references"] {
        assert!(rows[0].response.get(key).is_none());
    }
    for index in [1, 2] {
        assert!(rows[index].response["error"].as_str().is_some());
    }
    assert_eq!(rows[3].response["allConsumed"], false);
}

#[test]
fn rust_cache_reuses_identical_outputs_and_preserves_other_grammars() {
    let scratch = Scratch::new().unwrap();
    let runner = Runner {
        backend: Backend::Rust,
        build_dir: scratch.0.clone(),
    };
    let tiny = artifact("ir/examples/tinycalc.ir.json");
    let first = runner.build(tiny.clone()).unwrap();
    let before = std::fs::metadata(&first.program)
        .unwrap()
        .modified()
        .unwrap();
    let other = runner
        .build(artifact("crates/ubnfc-rust/tests/ir/operators.ir.json"))
        .unwrap();
    assert_ne!(first.program, other.program);
    // 完成キャッシュだけで実行でき、cargo の target を再作成しない。
    std::fs::remove_dir_all(scratch.0.join("rust-target")).unwrap();
    let copied = scratch.0.join("same.ir.json");
    let mut bytes = std::fs::read(root().join("ir/examples/tinycalc.ir.json")).unwrap();
    bytes.extend_from_slice(b"\n ");
    std::fs::write(&copied, bytes).unwrap();
    let second = runner
        .build(Artifact::FromIr {
            ir_path: copied,
            backend: Backend::Rust,
        })
        .unwrap();
    assert!(!scratch.0.join("rust-target").exists());
    assert_eq!(first.program, second.program);
    assert_eq!(
        before,
        std::fs::metadata(&second.program)
            .unwrap()
            .modified()
            .unwrap()
    );
    let case = tiny_case(0, "1+2", true, Some("Expression"));
    let rows = runner.run(&first, &[case]);
    assert_eq!(rows[0].response["value"], 3.0);
    let bad = scratch.0.join("bad.ir.json");
    std::fs::write(&bad, "{}").unwrap();
    let error = runner
        .build(Artifact::FromIr {
            ir_path: bad,
            backend: Backend::Rust,
        })
        .unwrap_err();
    assert!(
        error.to_string().contains("Rust IR generation 失敗"),
        "{error}"
    );
    let mismatch = Artifact::FromIr {
        ir_path: root().join("ir/examples/tinycalc.ir.json"),
        backend: Backend::Java,
    };
    assert!(runner
        .build(mismatch)
        .unwrap_err()
        .to_string()
        .contains("一致しません"));
}
