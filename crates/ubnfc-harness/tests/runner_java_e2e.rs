use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
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
fn generator() -> &'static Path {
    static CLASSES: OnceLock<PathBuf> = OnceLock::new();
    CLASSES.get_or_init(|| {
        // 必須ツールが無ければ失敗する。skip しない。
        for tool in ["java", "javac"] {
            assert!(Command::new(tool)
                .arg("-version")
                .output()
                .expect(tool)
                .status
                .success());
        }
        let project = root().join("ubnfc-java");
        let classes = project.join("target/classes");
        if !classes.join("org/ubnfc/java/Main.class").is_file() {
            let output = Command::new("mvn")
                .current_dir(&project)
                .args(["-o", "-q", "-DskipTests", "package"])
                .output()
                .expect("mvn");
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        classes
    })
}
fn artifact(ir: &str) -> Artifact {
    Artifact::FromIr {
        ir_path: root().join(ir),
        backend: Backend::Java,
    }
}
fn run_family(ir: &str, family: &str, count: usize) {
    let scratch = Scratch::new().unwrap();
    let runner = Runner {
        backend: Backend::Java,
        build_dir: scratch.0.clone(),
    };
    let executable = runner
        .build_with_java_generator(artifact(ir), generator())
        .unwrap();
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
        "ubnfc-java/src/test/resources/ir/operators.ir.json",
        "associative",
        24,
    );
}
#[test]
fn generated_power_pass_all_28_without_exclusions() {
    run_family(
        "ubnfc-java/src/test/resources/ir/power.ir.json",
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
        extra_expectations: BTreeMap::new(),
    }
}
#[test]
fn tinycalc_acceptance_entries_and_options_through_driver() {
    let scratch = Scratch::new().unwrap();
    let runner = Runner {
        backend: Backend::Java,
        build_dir: scratch.0.clone(),
    };
    let executable = runner
        .build_with_java_generator(artifact("ir/examples/tinycalc.ir.json"), generator())
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
fn ir_build_errors_are_explicit() {
    let scratch = Scratch::new().unwrap();
    let runner = Runner {
        backend: Backend::Java,
        build_dir: scratch.0.clone(),
    };
    let mismatch = Artifact::FromIr {
        ir_path: root().join("ir/examples/tinycalc.ir.json"),
        backend: Backend::Rust,
    };
    assert!(runner
        .build(mismatch.clone())
        .unwrap_err()
        .to_string()
        .contains("一致しません"));
    let rust = Runner {
        backend: Backend::Rust,
        build_dir: scratch.0.clone(),
    };
    assert!(rust
        .build(mismatch)
        .unwrap_err()
        .to_string()
        .contains("Java backend のみ"));
    let bad = scratch.0.join("bad.ir.json");
    std::fs::write(&bad, "{}").unwrap();
    let error = runner
        .build_with_java_generator(
            Artifact::FromIr {
                ir_path: bad,
                backend: Backend::Java,
            },
            generator(),
        )
        .unwrap_err();
    assert!(
        error.to_string().contains("Java IR generation 失敗"),
        "{error}"
    );
    let error = runner
        .build_with_java_generator(
            artifact("ir/examples/tinycalc.ir.json"),
            scratch.0.join("missing.jar"),
        )
        .unwrap_err();
    assert!(error.to_string().contains("Java 生成器がありません"));
}
