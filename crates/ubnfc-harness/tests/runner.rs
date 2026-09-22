use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use ubnfc_harness::{
    default_root,
    divergence::{Observation, Verdict},
    load_all,
    runner::{
        compare, select, unobserved, Actual, Backend, Executable, Options, Report, RunCase, Runner,
        Scratch,
    },
    Case, Expectation,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn case(expect: Value) -> RunCase {
    RunCase {
        case: Case {
            id: "probe/0".into(),
            grammar_ref: vec![0],
            entry_rule: None,
            input: "a😀".into(),
            expect: serde_json::from_value(expect).unwrap(),
            known_divergences: vec![],
            follow_spec: vec![],
            rust_oracle: None,
        },
        grammar: "Probe".into(),
        variant: "normal".into(),
        family: "test".into(),
        fixture: Value::Null,
        options: Options {
            lexical: true,
            scope: true,
        },
        extra_expectations: BTreeMap::new(),
    }
}
fn actual(backend: Backend, response: Value) -> Actual {
    Actual {
        backend,
        variant: "normal".into(),
        response,
    }
}
fn fail_paths(verdict: Verdict) -> Vec<String> {
    let Verdict::Fail(diffs) = verdict else {
        panic!("Fail を期待: {verdict:?}")
    };
    diffs.into_iter().map(|d| d.path).collect()
}
fn corpus_cases() -> Vec<RunCase> {
    select(
        &load_all(default_root()).unwrap(),
        &["associative".into(), "right-associative".into()],
        None,
    )
    .unwrap()
}

fn handwritten(backend: Backend) {
    let scratch = Scratch::new().unwrap();
    let language = match backend {
        Backend::Rust => "rust",
        Backend::Java => "java",
    };
    let runner = Runner {
        backend,
        build_dir: scratch.0.clone(),
    };
    // javac/java が PATH に無ければ unwrap で失敗。skip はしない。
    let exec = runner
        .build(root().join("examples/handwritten-target").join(language))
        .unwrap();
    let cases = corpus_cases();
    let mut actual = runner.run(&exec, &cases);
    let original = Report::from_actual(&cases, &actual).unwrap();
    assert_eq!(cases.len(), 52);
    assert_eq!(original.known_divergence, 0);
    assert_eq!(original.unobserved, 0);
    if backend == Backend::Java {
        assert_eq!(original.pass, 51);
        assert_eq!(original.fail.len(), 1);
        assert_eq!(original.fail[0].1.len(), 1);
        assert_eq!(original.fail[0].1[0].path, "/value");
        let index = cases
            .iter()
            .position(|c| c.case.input == "1e2/4-0.5")
            .unwrap();
        assert_eq!(actual[index].response["allConsumed"], true);
        assert!(actual[index].response["mappingError"].as_str().is_some());
        assert_eq!(actual[index].response["value"], Value::Null);
        // 手書き目標の既知逸脱: (NUMBER) @value を Java int に変換しており、
        // 指数/小数で mapping が失敗する。parser/golden は変更せず、この value
        // 観測だけをテストで除外する。D-017 台帳には登録せず、CLI は上記 Fail を返す。
        actual[index]
            .response
            .as_object_mut()
            .unwrap()
            .remove("value");
    }
    let report = Report::from_actual(&cases, &actual).unwrap();
    assert_eq!(report.pass, 52, "{}", report.markdown());
    assert!(report.fail.is_empty(), "{}", report.markdown());
    assert_eq!(report.families["associative"].pass, 24);
    assert_eq!(report.families["right-associative"].pass, 28);
    assert_eq!(report.unobserved, usize::from(backend == Backend::Java));
    assert_eq!(report.unobserved_cases, report.unobserved);
    let encoded: Value = serde_json::from_str(&report.json()).unwrap();
    assert_eq!(encoded["pass"], 52);
    assert!(report
        .markdown()
        .contains("| right-associative | 28 | 0 | 0 | 0 | 0 |"));

    let golden_dir = root().join("examples/handwritten-target").join(language);
    for (family, name) in [("associative", "operators"), ("right-associative", "power")] {
        let path = if backend == Backend::Rust {
            golden_dir.join(format!("tests/golden/{name}.json"))
        } else {
            golden_dir.join(format!("src/test/resources/golden/{name}.json"))
        };
        let golden: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let rows = if backend == Backend::Rust && name == "power" {
            &golden["text"]
        } else {
            &golden
        };
        for ((case, actual), golden) in cases
            .iter()
            .zip(&actual)
            .filter(|(c, _)| c.family == family)
            .zip(rows.as_array().unwrap())
        {
            let row = &actual.response;
            assert_eq!(golden["input"], case.case.input);
            let expected = if backend == Backend::Rust {
                &golden["result"]
            } else {
                golden
            };
            assert_eq!(row["allConsumed"], expected["ok"], "{}", case.label());
            assert_eq!(row["ast"], expected["ast"], "{}", case.label());
            let (consumed, matched) = if backend == Backend::Rust {
                ("consumed_cp", "matched_cp")
            } else {
                ("consumedCp", "matchedCp")
            };
            assert_eq!(row["consumed"], expected[consumed], "{}", case.label());
            assert_eq!(row["matched"], expected[matched], "{}", case.label());
            if backend == Backend::Java {
                assert_eq!(
                    !row["mappingError"].is_null(),
                    golden["mappingError"].as_bool().unwrap()
                );
                let captures: Vec<_> = golden["captures"].as_array().unwrap().iter().map(|c| {
                    let start = c["span"][0].as_u64().unwrap() as usize;
                    let end = c["span"][1].as_u64().unwrap() as usize;
                    json!({"name":c["name"], "span":c["span"], "text":case.case.input.chars().skip(start).take(end-start).collect::<String>()})
                }).collect();
                assert_eq!(row["captures"], json!(captures));
                let diagnostic = &golden["diagnostics"];
                assert_eq!(row["diagnostics"][0]["offset"], diagnostic["farthestCp"]);
                assert_eq!(row["diagnostics"][0]["expected"], diagnostic["expected"]);
            }
            assert_eq!(
                row["allConsumed"],
                json!(
                    row["accepted"] == true
                        && row["consumed"].as_u64().unwrap() as usize
                            == case.case.input.chars().count()
                )
            );
        }
    }
    // 末尾不正入力でも prefix 受理は true。全消費の AST は golden 通り null。
    let index = cases.iter().position(|c| c.case.input == "1+").unwrap();
    assert_eq!(actual[index].response["accepted"], true);
    assert_eq!(actual[index].response["allConsumed"], false);

    let mut probes = cases[..3].to_vec();
    probes[0].grammar = "Unknown".into();
    probes[1].case.entry_rule = Some("Unknown".into());
    probes[2].case.entry_rule = None;
    probes[2].options = Options::default();
    let response = runner.run(&exec, &probes);
    assert!(response[0].response["error"].is_string());
    assert!(response[1].response["error"].is_string());
    assert_eq!(response[2].response["allConsumed"], true);
    assert!(response[2].response.get("captures").is_none());
    assert!(response[2].response.get("declarations").is_none());
}
#[test]
fn handwritten_rust_matches_52_cases_and_golden() {
    handwritten(Backend::Rust);
}
#[test]
fn handwritten_java_matches_52_cases_and_golden() {
    handwritten(Backend::Java);
}

#[test]
fn missing_is_counted_but_explicit_null_and_wrong_values_fail() {
    let c = case(json!({"TokenValue":{"value":"a", "prefix":[true,2,2]}}));
    let missing = actual(Backend::Rust, json!({"accepted":true,"consumed":2}));
    assert_eq!(compare(&c, &missing, &[]), Verdict::Pass);
    assert_eq!(
        unobserved(&c, &missing),
        ["/allConsumed", "/ast", "/matched"]
    );
    let report =
        Report::from_actual(std::slice::from_ref(&c), std::slice::from_ref(&missing)).unwrap();
    assert_eq!(
        (report.pass, report.unobserved, report.unobserved_cases),
        (1, 3, 1)
    );
    assert!(report.markdown().contains("/ast"));
    assert!(Report::from_actual(std::slice::from_ref(&c), &[]).is_err());
    let explicit_null = actual(
        Backend::Rust,
        json!({"allConsumed":true,"ast":null,"value":null,"accepted":true,"consumed":2,"matched":1}),
    );
    assert_eq!(
        fail_paths(compare(&c, &explicit_null, &[])),
        ["/ast", "/matched"]
    );
}

#[test]
fn token_null_success_modes_rule_fields_texts_and_numeric_offsets() {
    let token = case(json!({"TokenValue":{"value":null,"prefix":[true,0,0]}}));
    let observed = actual(
        Backend::Rust,
        json!({"value":null,"ast":{"type":"Value","span":[0,2],"fields":{"value":null}},"allConsumed":true,"accepted":true,"consumed":0,"matched":0}),
    );
    assert_eq!(compare(&token, &observed, &[]), Verdict::Pass);
    let reject = case(json!({"TokenValue":{}}));
    assert_eq!(
        fail_paths(compare(&reject, &observed, &[])),
        ["/allConsumed"]
    );
    let mut mode = case(json!({"RuleValue":{"value":2,"modes":["grouped"]}}));
    assert_eq!(
        compare(
            &mode,
            &actual(Backend::Rust, json!({"allConsumed":false,"value":null})),
            &[]
        ),
        Verdict::Pass
    );
    mode.case.expect =
        serde_json::from_value(json!({"RuleValue":{"value":2,"texts":[[0,2,"a😀"]]}})).unwrap();
    assert_eq!(
        compare(
            &mode,
            &actual(
                Backend::Rust,
                json!({"allConsumed":true,"value":2.0,"texts":[[0,2,"a😀"]]})
            ),
            &[]
        ),
        Verdict::Pass
    );
    let rule = case(
        json!({"Accept":{"accepted":true,"prefix":[true,2,2],"span":[0,2],"fields":{"value":"a😀"},"value":"a😀"}}),
    );
    let observed = actual(
        Backend::Rust,
        json!({"allConsumed":true,"accepted":true,"consumed":2,"matched":2,"value":"a😀","ast":{"span":[0,2],"fields":{"value":"a😀"}}}),
    );
    assert_eq!(compare(&rule, &observed, &[]), Verdict::Pass);
    let numeric = case(json!({"Numeric":{"java_int":null,"rust_lexeme":"1.5","extra":{}}}));
    assert_eq!(
        compare(
            &numeric,
            &actual(
                Backend::Java,
                json!({"allConsumed":true,"mappingError":"int overflow"})
            ),
            &[]
        ),
        Verdict::Pass
    );
    assert_eq!(
        fail_paths(compare(
            &numeric,
            &actual(
                Backend::Java,
                json!({"allConsumed":true,"mappingError":null})
            ),
            &[]
        )),
        ["/mappingError"]
    );
    assert_eq!(
        compare(
            &numeric,
            &actual(
                Backend::Rust,
                json!({"allConsumed":true,"ast":{"fields":{"value":"1.5"}}})
            ),
            &[]
        ),
        Verdict::Pass
    );
    let diagnostic = case(json!({"Diagnostics":{"kind":"syntax","javaOffset":7,"rustOffset":5}}));
    for (backend, offset) in [(Backend::Java, 7), (Backend::Rust, 5)] {
        assert_eq!(
            compare(
                &diagnostic,
                &actual(
                    backend,
                    json!({"diagnostics":[{"kind":"syntax","offset":offset,"expected":["NUMBER"]}]})
                ),
                &[]
            ),
            Verdict::Pass
        );
        assert_eq!(
            fail_paths(compare(
                &diagnostic,
                &actual(backend, json!({"diagnostics":[]})),
                &[]
            )),
            ["/expected", "/kind", "/offset"]
        );
    }
}

#[test]
fn ast_declarations_and_oracle_fields_are_compared_without_waiving_unknowns() {
    let ast = json!({"type":"Value","span":[0,2],"fields":{"value":"a😀"}});
    let c = case(
        json!({"Ast":{"ast":ast,"captures":[{"name":"value","span":[0,2],"text":"a😀"}],"declarations":[{"name":"a😀","sourceOffset":0}]}}),
    );
    let mut response = json!({"allConsumed":true,"ast":ast,"captures":[{"name":"value","span":[0,2],"text":"a😀"}],"declarations":[{"name":"a😀","sourceOffset":0}]});
    assert_eq!(
        compare(&c, &actual(Backend::Rust, response.clone()), &[]),
        Verdict::Pass
    );
    response["ast"]["span"][1] = json!(3);
    assert_eq!(
        fail_paths(compare(
            &c,
            &actual(Backend::Rust, response.clone()),
            &[Observation::Ast]
        )),
        ["/ast/span/1"]
    );
    let mut oracle = case(
        json!({"Oracle":{"accepted":true,"allConsumed":true,"consumed":2,"matched":2,"ast":ast,
        "captures":[{"name":"value","binding":"Root:0","span":[0,2],"text":"a😀"}],"declarations":[{"name":"a😀","sourceOffset":0}],
        "diagnostics":[],"mappingError":null}}),
    );
    response["accepted"] = json!(true);
    response["consumed"] = json!(2);
    response["matched"] = json!(2);
    response["diagnostics"] = json!([]);
    response["mappingError"] = Value::Null;
    response["prefixCaptures"] = response["captures"].clone();
    let observed = actual(Backend::Rust, response.clone());
    assert_eq!(unobserved(&oracle, &observed), ["/captures/0/binding"]);
    assert!(matches!(
        compare(&oracle, &observed, &[Observation::Ast]),
        Verdict::KnownDivergence(_)
    ));
    oracle.case.known_divergences = vec![Observation::Ast];
    let report = Report::from_actual(
        std::slice::from_ref(&oracle),
        std::slice::from_ref(&observed),
    )
    .unwrap();
    assert_eq!(
        (report.pass, report.known_divergence, report.unobserved),
        (0, 1, 1)
    );
    assert_eq!(report.known_diffs[0].1[0].path, "/ast/span/1");
    response["declarations"] = json!([]);
    assert_eq!(
        fail_paths(compare(
            &oracle,
            &actual(Backend::Rust, response.clone()),
            &[Observation::Ast]
        )),
        ["/ast/span/1", "/declarations/0"]
    );
    response["ast"] = ast;
    response["declarations"] = json!([{"name":"a😀","sourceOffset":0}]);
    oracle
        .extra_expectations
        .insert("references".into(), json!([]));
    response["references"] = json!([{"name":"x","offset":1,"length":1}]);
    assert_eq!(
        fail_paths(compare(
            &oracle,
            &actual(Backend::Rust, response),
            &[Observation::Ast]
        )),
        ["/references/0"]
    );
}

fn shell(script: &str) -> Executable {
    Executable {
        program: "/bin/sh".into(),
        args: vec!["-c".into(), script.into()],
        grammars: BTreeMap::new(),
    }
}
#[test]
fn driver_errors_missing_malformed_duplicate_lines_and_nonzero_exit_are_failures() {
    let scratch = Scratch::new().unwrap();
    let runner = Runner {
        backend: Backend::Rust,
        build_dir: scratch.0.clone(),
    };
    let c = case(json!({"TokenValue":{"value":"a"}}));
    for script in [
        "cat >/dev/null; printf '%s\n' '{\"id\":0,\"error\":\"broken\"}'",
        "cat >/dev/null",
        "cat >/dev/null; printf '%s\n' 'broken'",
        "cat >/dev/null; printf '%s\n' '{\"id\":0}' '{\"id\":0}'",
        "cat >/dev/null; printf '%s\n' '{\"id\":42}'",
        "cat >/dev/null; printf '%s\n' '{\"id\":0,\"allConsumed\":true,\"value\":\"a\"}'; exit 3",
    ] {
        let result = runner.run(&shell(script), std::slice::from_ref(&c));
        assert_eq!(result.len(), 1);
        assert_eq!(
            fail_paths(compare(&c, &result[0], &[])),
            ["/error"],
            "{script}"
        );
        assert!(unobserved(&c, &result[0]).is_empty());
    }
    // 複数行を一つのプロセスへ送り、受信 id により入力順へ戻す。
    let result = runner.run(&shell("cat >/dev/null; printf '%s\n' '{\"id\":1,\"value\":\"second\"}' '{\"id\":0,\"value\":\"first\"}'"), &[c.clone(),c]);
    assert_eq!(result[0].response["value"], "first");
    assert_eq!(result[1].response["value"], "second");
}

#[test]
fn large_duplex_stream_is_drained_and_grammar_specific_executable_is_selected() {
    let scratch = Scratch::new().unwrap();
    let runner = Runner {
        backend: Backend::Rust,
        build_dir: scratch.0.clone(),
    };
    let mut c = case(json!({"Raw":null}));
    c.case.input = "😀".repeat(8192);
    let cases = vec![c; 128];
    // /bin/cat の応答はリクエストそのもの。往復とも pipe 容量を大幅に超える。
    let executable = Executable {
        program: "/bin/cat".into(),
        args: vec![],
        grammars: BTreeMap::from([("probe".into(), PathBuf::from("/bin/cat"))]),
    };
    let actual = runner.run(&executable, &cases);
    assert_eq!(actual.len(), 128);
    assert_eq!(actual[127].response["id"], 127);
    assert_eq!(actual[127].response["input"], cases[127].case.input);
}

#[test]
fn offline_build_failures_return_diagnostics_and_do_not_reuse_old_classes() {
    let scratch = Scratch::new().unwrap();
    let rust = scratch.0.join("rust-source");
    fs::create_dir_all(rust.join("src")).unwrap();
    fs::write(
        rust.join("Cargo.toml"),
        "[package]\nname='broken-driver'\nversion='0.0.0'\nedition='2021'\n[workspace]\n",
    )
    .unwrap();
    fs::write(rust.join("src/main.rs"), "fn main() { does_not_exist(); }").unwrap();
    let runner = Runner {
        backend: Backend::Rust,
        build_dir: scratch.0.join("build-rust"),
    };
    let error = runner.build(&rust).unwrap_err().to_string();
    assert!(
        error.contains("cargo build 失敗") && error.contains("does_not_exist"),
        "{error}"
    );
    let java = scratch.0.join("java-source/src/main/java/target/driver");
    fs::create_dir_all(&java).unwrap();
    fs::write(
        java.join("Driver.java"),
        "package target.driver; public class Driver { public static void main(String[] args) {} }",
    )
    .unwrap();
    let runner = Runner {
        backend: Backend::Java,
        build_dir: scratch.0.join("build-java"),
    };
    let artifact = scratch.0.join("java-source");
    runner.build(&artifact).unwrap();
    fs::write(java.join("Driver.java"), "not Java").unwrap();
    assert!(runner
        .build(&artifact)
        .unwrap_err()
        .to_string()
        .contains("javac 失敗"));
    fs::write(
        java.join("Driver.java"),
        "package target.driver; class NotDriver {}",
    )
    .unwrap();
    assert!(runner
        .build(&artifact)
        .unwrap_err()
        .to_string()
        .contains("Driver がありません"));
}

#[test]
fn selection_preserves_variant_and_oracle_extra_observations() {
    let files = load_all(default_root()).unwrap();
    let cases = select(&files, &["right-associative".into()], Some("grouped")).unwrap();
    assert_eq!(cases.len(), 28);
    assert!(cases
        .iter()
        .all(|c| c.grammar == "Power" && c.variant == "grouped"));
    assert!(select(&files, &["typo".into()], None).is_err());
    assert!(select(&files, &["associative".into()], Some("typo")).is_err());
    let oracle = select(&files, &["oracle".into()], None).unwrap();
    assert!(oracle
        .iter()
        .filter(|c| matches!(c.case.expect, Expectation::Oracle { .. }))
        .all(|c| c.extra_expectations.contains_key("references")));
}

#[test]
fn cli_writes_markdown_and_json_reports_and_fails_on_java_mapping() {
    let scratch = Scratch::new().unwrap();
    let binary = env!("CARGO_BIN_EXE_ubnfc-harness");
    for (backend, extension, fail) in [("rust", "json", 0), ("java", "md", 1)] {
        let report = scratch.0.join(format!("{backend}.{extension}"));
        let output = Command::new(binary)
            .args(["run", "--backend", backend, "--artifact"])
            .arg(root().join(format!("examples/handwritten-target/{backend}")))
            .args(["--families", "associative,right-associative", "--report"])
            .arg(&report)
            .current_dir(&scratch.0)
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            fail == 0,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let content = fs::read_to_string(report).unwrap();
        if extension == "json" {
            let report: Value = serde_json::from_str(&content).unwrap();
            assert_eq!(report["pass"], 52);
            assert_eq!(report["fail"], json!([]));
        } else {
            assert!(content.contains("| associative | 23 | 0 | 1 | 0 | 0 |"));
        }
    }
}

#[test]
fn explicit_null_ast_is_not_missing_rule_observations_and_scope_uses_wire_events() {
    let c = case(
        json!({"Accept":{"accepted":true,"prefix":[true,2,2],"span":[0,2],"fields":{"value":"a😀"}}}),
    );
    let response = actual(
        Backend::Rust,
        json!({"allConsumed":true,"accepted":true,"consumed":2,"matched":2,"ast":null}),
    );
    assert_eq!(
        fail_paths(compare(&c, &response, &[])),
        ["/fields", "/span"]
    );
    assert!(unobserved(&c, &response).is_empty());
    let c = case(json!({"Ast":{"accepted":true,"ast":null,"scope":{
        "depth":0,"resolved":[],"declarations":[{"name":"a😀","sourceOffset":0}],"references":[],"diagnostics":[]
    }}}));
    let response = actual(
        Backend::Rust,
        json!({"allConsumed":true,"ast":null,"declarations":[],"references":[],"diagnostics":[]}),
    );
    assert_eq!(
        fail_paths(compare(&c, &response, &[])),
        ["/scope/declarations/0"]
    );
    assert_eq!(
        unobserved(&c, &response),
        ["/scope/depth", "/scope/resolved"]
    );
}
