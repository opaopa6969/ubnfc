//! 根拠の行番号は upstream codegen/ 配下（resolve は ubnfc-front/tests）。
use serde_json::{json, Value};
use ubnfc_harness::{
    default_root,
    divergence::Verdict,
    load_all,
    runner::{compare, select, unobserved, Actual, Backend, Options, RunCase},
    Case, Expectation, GrammarSource,
};

fn case(expect: Value, family: &str, input: &str) -> RunCase {
    RunCase {
        case: Case {
            id: "probe".into(),
            grammar_ref: vec![0],
            entry_rule: None,
            input: input.into(),
            expect: serde_json::from_value(expect).unwrap(),
            known_divergences: vec![],
        },
        grammar: "Probe".into(),
        variant: "normal".into(),
        family: family.into(),
        options: Options::default(),
        fixture: Value::Null,
        extra_expectations: Default::default(),
    }
}
fn actual(response: Value) -> Actual {
    Actual {
        backend: Backend::Java,
        variant: "normal".into(),
        response,
    }
}
fn pass(c: &RunCase, a: &Actual) {
    assert_eq!(compare(c, a, &[]), Verdict::Pass);
    assert!(unobserved(c, a).is_empty(), "{:?}", unobserved(c, a));
}
fn fails(c: &RunCase, a: &Actual) {
    assert!(matches!(compare(c, a, &[]), Verdict::Fail(_)));
}

#[test]
fn token_value_is_a_complete_mapped_ast_including_null_lists_and_code_point_spans() {
    // RustConformanceTest.java:561-575。評価 value が一致しても壊れた AST は Fail。
    for value in [Value::Null, json!("😀"), json!(["😀", "a"])] {
        let c = case(json!({"TokenValue":{"value":value}}), "primitives", "😀a");
        let mut a = actual(json!({"allConsumed":true,"value":null,
            "ast":{"type":"Value","span":[0,2],"fields":{"value":value}}}));
        pass(&c, &a);
        a.response["ast"]["span"][1] = json!(3);
        fails(&c, &a);
        a.response["ast"] = Value::Null;
        a.response["value"] = value;
        fails(&c, &a);
        a.response.as_object_mut().unwrap().remove("ast");
        assert_eq!(unobserved(&c, &a), ["/ast"]);
    }
}

#[test]
fn numeric_field_and_semantic_evaluation_remain_distinct() {
    // NumericCaptureRuntimeTest.java:31-49,81-86。数値の型も比較する。
    let c = case(
        json!({"Numeric":{"java_int":7,"rust_lexeme":"+007","extra":{}}}),
        "numeric-capture",
        "n+007;",
    );
    let mut a = actual(
        json!({"allConsumed":true,"mappingError":null,"value":null,"ast":{"fields":{"value":7}}}),
    );
    pass(&c, &a);
    a.response["ast"]["fields"]["value"] = json!("7");
    fails(&c, &a);
    a.backend = Backend::Rust;
    a.response["ast"]["fields"]["value"] = json!("+007");
    pass(&c, &a);
    // SemanticCardinalityConformance.java:154-175: L:/T:/O: は評価値。
    let c = case(
        json!({"RuleValue":{"value":"L:x","texts":[]}}),
        "semantic-cardinality",
        "x",
    );
    let mut a = actual(json!({"allConsumed":true,"value":null,"ast":{"fields":{"values":"L:x"}}}));
    fails(&c, &a);
    a.response["value"] = json!("L:x");
    pass(&c, &a); // texts は upstream Java テストの runtime 観測ではない。
    a.backend = Backend::Rust;
    assert_eq!(unobserved(&c, &a), ["/texts"]);
}

#[test]
fn cardinality_loader_oracle_is_projected_to_ast_only_for_accepted_modes() {
    // RustConformanceTest.java:680-721。
    let c = case(
        json!({"RuleValue":{"value":2,"modes":["normal"],
        "expected":{"head":null,"items":[["2",1,3]],"tail":null,"flags":""}}}),
        "cardinality",
        ":2;::",
    );
    let mut a = actual(json!({"allConsumed":true,"value":2,"ast":{
        "type":"Container","span":[0,5],"fields":{"head":null,"values":[
            {"type":"Item","span":[1,3],"fields":{"value":"2"}}],"tail":null,"flags":[]}}}));
    pass(&c, &a);
    a.response["ast"]["fields"]["values"][0]["span"][1] = json!(4);
    fails(&c, &a);
    a.variant = "rejected".into();
    a.response = json!({"allConsumed":false});
    pass(&c, &a);
}

#[test]
fn captures_use_fixture_names_without_sorting_deduplicating_or_guessing_from_oracle() {
    // NestedCaptureConformanceTest.java:283-284,320-327。
    let capture = json!({"name":"x","span":[0,1],"text":"x"});
    let mut c = case(
        json!({"Ast":{"ast":null,"captures":[capture,capture]}}),
        "test",
        "x",
    );
    c.fixture = json!({"captureNames":["x","unexpected"]});
    let mut a = actual(json!({"allConsumed":true,"ast":null,"captures":[
        {"name":"wrapper","span":[0,1],"text":"x"},capture,capture]}));
    pass(&c, &a);
    a.response["captures"].as_array_mut().unwrap().pop();
    fails(&c, &a);
    a.response["captures"] = json!([capture,capture,{"name":"unexpected","span":[0,1],"text":"x"}]);
    fails(&c, &a);
}

#[test]
fn scope_projection_keeps_message_and_offsets_but_excludes_syntax_diagnostics() {
    // ScopeAnnotationConformanceTest.java:138-141,222-249。
    let d = json!({"message":"未定義のシンボル: 'x'","offset":1,"length":1,"severity":"WARNING"});
    let c = case(
        json!({"Ast":{"ast":null,"scope":{"depth":0,"resolved":[],"references":[],"diagnostics":[d]}}}),
        "scope-annotations",
        " x",
    );
    let mut wire = d;
    wire["kind"] = json!("scope");
    wire["expected"] = json!([]);
    let mut a = actual(
        json!({"allConsumed":true,"ast":null,"scope":{"depth":0,"resolved":[]},"references":[],"diagnostics":[
        {"kind":"syntax","offset":2,"expected":["EOF"]},wire]}),
    );
    pass(&c, &a);
    a.response["diagnostics"][1]["message"] = json!("Undefined reference: x");
    fails(&c, &a);
}

#[test]
fn parent_rollback_is_a_state_probe_even_when_the_trigger_value_is_false() {
    // ScopeAnnotationConformanceTest.java:162-169,238-257。
    let mut c = case(
        json!({"Ast":{"ast":null,"parentRollback":false}}),
        "scope-annotations",
        "x",
    );
    c.fixture = json!({"queries":["x"]});
    let mut a = actual(json!({"allConsumed":true,"ast":null,"rollback":{
        "cursor":[0,0],"scope":{"depth":0,"resolved":[{"query":"x","symbol":null}],
            "declarations":[],"references":[],"diagnostics":[]}}}));
    pass(&c, &a);
    a.response["rollback"]["cursor"][0] = json!(1);
    fails(&c, &a);
    a.response["rollback"] = json!(true);
    fails(&c, &a);

    // request に probe 条件だけを渡し、oracle 本体を driver に渡さない。
    use ubnfc_harness::runner::{Executable, Runner, Scratch};
    let scratch = Scratch::new().unwrap();
    let runner = Runner {
        backend: Backend::Java,
        build_dir: scratch.0.clone(),
    };
    let cat = Executable {
        program: "/bin/cat".into(),
        args: vec![],
        grammars: Default::default(),
    };
    let response = runner.run(&cat, &[c]);
    let request = &response[0].response;
    assert_eq!(request["observations"]["queries"], json!(["x"]));
    assert_eq!(request["observations"]["parentRollback"], true);
    assert!(request.get("expected").is_none());
    assert!(request["observations"].get("ast").is_none());
}

fn oracle_case(input: &str, accepted: bool, all_consumed: bool, diagnostics: Value) -> RunCase {
    case(
        json!({"Oracle":{"accepted":accepted,"allConsumed":all_consumed,
        "consumed":0,"matched":0,"ast":null,"captures":[],"declarations":[],
        "diagnostics":diagnostics,"mappingError":null}}),
        "oracle",
        input,
    )
}
fn oracle_actual(accepted: bool, all_consumed: bool, diagnostics: Value) -> Actual {
    actual(
        json!({"accepted":accepted,"allConsumed":all_consumed,"consumed":0,"matched":0,
        "ast":null,"captures":[],"prefixCaptures":[],"declarations":[],"diagnostics":diagnostics,"mappingError":null}),
    )
}

#[test]
fn oracle_captures_require_prefix_observation_even_after_full_parse_failure() {
    // OracleCollector.java:171-174 / SemanticsTest.java:143-151。
    let capture = json!({"name":"value","binding":"Root:0","span":[0,1],"text":"a"});
    let c = case(
        json!({"Oracle":{"accepted":true,"allConsumed":false,"consumed":1,"matched":1,
        "ast":null,"captures":[capture,capture],"declarations":[],"diagnostics":[],"mappingError":null}}),
        "oracle",
        "a!",
    );
    for backend in [Backend::Java, Backend::Rust] {
        let mut a = oracle_actual(true, false, json!([]));
        a.backend = backend;
        a.response["consumed"] = json!(1);
        a.response["matched"] = json!(1);
        a.response["prefixCaptures"] = json!([capture, capture]);
        pass(&c, &a);
        // 全入力の capture の有無は prefix 観測を補完・上書きしない。
        a.response["captures"] = json!([{"name":"wrong"}]);
        pass(&c, &a);
        a.response["prefixCaptures"] = json!([capture]);
        fails(&c, &a);
        a.response["prefixCaptures"] = Value::Null;
        fails(&c, &a);
        a.response["captures"] = json!([capture, capture]);
        a.response.as_object_mut().unwrap().remove("prefixCaptures");
        assert_eq!(compare(&c, &a, &[]), Verdict::Pass);
        assert_eq!(unobserved(&c, &a), ["/prefixCaptures"]);
        let report =
            ubnfc_harness::runner::Report::from_actual(std::slice::from_ref(&c), &[a]).unwrap();
        assert_eq!((report.unobserved, report.unobserved_cases), (1, 1));
    }
}

#[test]
fn scope_declared_distinguishes_unspecified_null_and_boolean_without_weakening_symbols() {
    // ScopeAnnotationConformanceTest.java:219-235 は declared を出力しない。
    let resolution = json!({"query":"x","symbol":null});
    for expected_declared in [
        None,
        Some(Value::Null),
        Some(json!(false)),
        Some(json!(true)),
    ] {
        let mut expected = resolution.clone();
        if let Some(value) = &expected_declared {
            expected["declared"] = value.clone();
        }
        let c = case(
            json!({"Ast":{"ast":null,"scope":{"depth":0,"resolved":[expected],
            "references":[],"diagnostics":[]}}}),
            "scope-annotations",
            "x",
        );
        for actual_declared in [
            None,
            Some(Value::Null),
            Some(json!(false)),
            Some(json!(true)),
        ] {
            let mut observed = resolution.clone();
            if let Some(value) = &actual_declared {
                observed["declared"] = value.clone();
            }
            let a = actual(json!({"allConsumed":true,"ast":null,"scope":{"depth":0,
                "resolved":[observed],"references":[],"diagnostics":[]}}));
            let matches = match &expected_declared {
                None => true,
                Some(Value::Null) => actual_declared.as_ref().is_none_or(Value::is_null),
                Some(value) => actual_declared.as_ref() == Some(value),
            };
            if matches {
                pass(&c, &a);
            } else {
                fails(&c, &a);
            }
        }
    }
    let c = case(
        json!({"Ast":{"ast":null,"scope":{"depth":0,"resolved":[resolution],
        "references":[],"diagnostics":[]}}}),
        "scope-annotations",
        "x",
    );
    for resolved in [
        json!([]),
        json!([{"query":"x"}]),
        json!([{"query":"y","symbol":null}]),
        json!([{"query":"x","symbol":{"name":"x","sourceOffset":0}}]),
    ] {
        fails(
            &c,
            &actual(json!({"allConsumed":true,"ast":null,"scope":{"depth":0,
            "resolved":resolved,"references":[],"diagnostics":[]}})),
        );
    }
}

#[test]
fn regex_non_bmp_status_is_known_only_for_the_three_registered_observations() {
    use ubnfc_harness::divergence::Observation;
    let cases = select(&load_all(default_root()).unwrap(), &["oracle".into()], None).unwrap();
    for (id, end, start, text) in [
        ("regex-emoji/1", 1, 0, "😀"),
        ("regex-prefix/1", 2, 1, "a"),
        ("regex-prefix/2", 2, 1, "a"),
    ] {
        let c = cases.iter().find(|c| c.case.id == id).unwrap();
        let full = id != "regex-prefix/2";
        let a = actual(
            json!({"accepted":true,"allConsumed":full,"consumed":end,"matched":end,
            "ast":if full { json!({"type":"Value","span":[0,end],"fields":{"value":text}}) } else { Value::Null },
            "captures":[],"prefixCaptures":[{"name":"value","binding":"Root:0","span":[start,end],"text":text}],
            "declarations":[],"references":[],"scopeDepth":0,"mappingError":null,"parsedStatus":"succeeded",
            "diagnostics":if full { json!([]) } else { json!([{"kind":"trailing_input","offset":end,
                "expected":["end of input"],"farthestOffset":end,"farthestExpected":[],"origin":"Mapper.diagnose"}]) }}),
        );
        let known = &c.case.known_divergences;
        assert!(known.contains(&Observation::ParsedStatus));
        let Verdict::KnownDivergence(diffs) = compare(c, &a, known) else {
            panic!("{}: {:?}", id, compare(c, &a, known));
        };
        assert!(diffs.iter().any(|d| d.path == "/parsedStatus"));
        assert!(unobserved(c, &a).is_empty());
        let without_status: Vec<_> = known
            .iter()
            .copied()
            .filter(|o| *o != Observation::ParsedStatus)
            .collect();
        assert!(matches!(compare(c, &a, &without_status), Verdict::Fail(_)));
        let mut wrong = a.clone();
        wrong.response["scopeDepth"] = json!(1);
        assert!(matches!(compare(c, &wrong, known), Verdict::Fail(_)));
        let mut unrelated = c.clone();
        unrelated
            .extra_expectations
            .insert("futureObservation".into(), json!(true));
        wrong.response["scopeDepth"] = json!(0);
        wrong.response["futureObservation"] = json!(false);
        assert!(matches!(
            compare(&unrelated, &wrong, known),
            Verdict::Fail(_)
        ));
    }
    assert_eq!(
        cases
            .iter()
            .filter(|c| c
                .case
                .known_divergences
                .contains(&Observation::ParsedStatus))
            .count(),
        3
    );
}

#[test]
fn oracle_all_consumed_is_cursor_only_and_missing_or_invalid_cursor_is_not_success() {
    // OracleCollector.java:175; RustConformanceTest.java:554-564 は prefix と diagnose を分離。
    let c = oracle_case("", false, true, json!([]));
    let mut a = oracle_actual(false, false, json!([]));
    pass(&c, &a);
    a.response["consumed"] = json!(1);
    fails(&c, &a);
    a.response.as_object_mut().unwrap().remove("consumed");
    assert_eq!(unobserved(&c, &a), ["/allConsumed", "/consumed"]);
}

#[test]
fn diagnostic_projection_preserves_failure_kind_offset_nonempty_and_trailing_hints() {
    // RustConformanceTest.java:756-765,840-844。
    let e =
        json!({"kind":"syntax","offset":0,"expected":["DigitParser"],"origin":"Mapper.diagnose"});
    let c = oracle_case("x", false, false, json!([e]));
    let mut a = oracle_actual(
        false,
        false,
        json!([{"kind":"syntax","offset":0,"expected":["Digit"],"origin":"Mapper.diagnose"}]),
    );
    pass(&c, &a);
    a.response["diagnostics"][0]["offset"] = json!(1);
    fails(&c, &a);
    a.response["diagnostics"][0]["offset"] = json!(0);
    a.response["diagnostics"][0]["expected"] = json!([]);
    fails(&c, &a);
    let c = case(
        json!({"Diagnostics":{"kind":"trailing_input","javaOffset":1,"rustOffset":1}}),
        "evolution",
        "1x",
    );
    let mut a =
        actual(json!({"diagnostics":[{"kind":"trailing_input","offset":1,"expected":["EOF"]}]}));
    fails(&c, &a);
    a.response["diagnostics"][0]["expected"] = json!(["end of input"]);
    pass(&c, &a);
    let c = oracle_case("", true, true, json!([]));
    let mut a = oracle_actual(
        true,
        true,
        json!([{"kind":"syntax","offset":0,"expected":["ANY"]}]),
    );
    pass(&c, &a);
    a.response["diagnostics"][0]["kind"] = json!("scope");
    fails(&c, &a);
}

#[test]
fn semantic_grammar_synthesis_matches_upstream_and_positive_variants_never_expect_rejection() {
    // SemanticCardinalityConformance.java:42-52、RustConformanceTest.java:391-455。
    let files = load_all(default_root()).unwrap();
    for file in files
        .iter()
        .filter(|f| f.path.parent().unwrap().ends_with("semantic-cardinality"))
    {
        let template =
            std::fs::read_to_string(file.path.with_file_name("Grammar.ubnf.txt")).unwrap();
        for (grammar, row) in file.grammars.iter().zip(file.raw.as_array().unwrap()) {
            let GrammarSource::Inline { source, .. } = grammar else {
                panic!()
            };
            assert_eq!(
                *source,
                template
                    .replace(
                        "ROOT_BODY",
                        row["root"].as_str().unwrap_or("Helper @values")
                    )
                    .replace("HELPER_BODY", row["body"].as_str().unwrap())
                    .replace("EXTRA_RULES", row["extra"].as_str().unwrap_or(""))
            );
        }
    }
    let reversed = select(&files, &["associative".into()], Some("reversed-metadata")).unwrap();
    assert_eq!(reversed.len(), 24);
    assert!(reversed
        .iter()
        .all(|c| matches!(c.case.expect, Expectation::RuleValue(_))));
    // resolve.rs:143-155 と保存済み apiError が一致する1件だけ。
    let negatives: Vec<_> = files
        .iter()
        .flat_map(|f| &f.cases)
        .filter(|c| matches!(c.expect, Expectation::ExpectedRejection { .. }))
        .collect();
    assert_eq!(negatives.len(), 1);
    assert_eq!(negatives[0].id, "import-cycle/1");
    let c = select(&files, &["oracle".into()], None)
        .unwrap()
        .into_iter()
        .find(|c| c.case.id == "import-cycle/1")
        .unwrap();
    fails(&c, &actual(json!({"allConsumed":true})));
}

#[test]
fn d019_native_failure_is_preserved_but_excluded_from_verdict_and_missing() {
    let mut c = case(
        json!({"Accept":{"accepted":false,"prefix":[false,0,0]}}),
        "lexical",
        "x",
    );
    c.extra_expectations.insert(
        "nativeFailure".into(),
        json!({"offset":7,"expected":["OldParser"]}),
    );
    let saved = c.extra_expectations.clone();
    let mut a = actual(
        json!({"allConsumed":false,"accepted":false,"consumed":0,"matched":0,
        "nativeFailure":{"offset":1,"expected":["NewParser"]}}),
    );
    let observed = a.response.clone();
    pass(&c, &a);
    assert_eq!(c.extra_expectations, saved);
    assert_eq!(a.response, observed);
    a.response.as_object_mut().unwrap().remove("nativeFailure");
    pass(&c, &a);
    a.response["consumed"] = json!(1);
    fails(&c, &a);
    // D-014 の公開診断は差分を維持する。
    c.extra_expectations.insert(
        "diagnostics".into(),
        json!([{"offset":1,"farthestOffset":2}]),
    );
    a.response["consumed"] = json!(0);
    a.response["diagnostics"] = json!([{"offset":1,"farthestOffset":3}]);
    fails(&c, &a);
}
