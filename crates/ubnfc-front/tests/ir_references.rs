use serde_json::{json, Value};
use ubnfc_front::ir::*;

const EXAMPLE: &str = include_str!("../../../ir/examples/tinycalc.ir.json");

fn example() -> GrammarIr {
    from_json(EXAMPLE).unwrap()
}
fn value() -> Value {
    serde_json::from_str(EXAMPLE).unwrap()
}
fn errors(value: Value) -> Vec<IrError> {
    validate_references(&from_json(&value.to_string()).unwrap())
}
fn has_error(errors: &[IrError], path: &str, text: &str) -> bool {
    errors.iter().any(|e| matches!(e, IrError::Validation { path: actual, message } if actual.starts_with(path) && message.contains(text)))
}

#[test]
fn missing_references_are_reported_in_every_populated_section() {
    for pointer in [
        "/entry/grammarId",
        "/entry/ruleId",
        "/grammarUnits/0/sourceId",
        "/grammarUnits/0/source/sourceId",
        "/semantics/symbols/0/grammarId",
        "/semantics/symbols/0/sourceId",
        "/semantics/symbols/0/source/sourceId",
        "/semantics/tokens/0/symbolId",
        "/semantics/rules/0/symbolId",
        "/semantics/rules/0/triviaPolicyId",
        "/semantics/rules/0/mappingTypeId",
        "/semantics/captures/0/ruleId",
        "/semantics/recipes/0/ruleId",
        "/semantics/recipes/0/resultTypeId",
        "/semantics/recipes/0/fields/0/siteIds/0",
        "/semantics/astTypes/0/variants/0",
        "/semantics/astTypes/1/implements/0",
        "/semantics/astTypes/1/fields/0/valueType/typeId",
        "/semantics/trivia/insertionPlan/0/policyId",
        "/analysis/rules/0/ruleId",
        "/analysis/rules/0/sccId",
        "/analysis/sccs/0/ruleIds/0",
        "/plan/lexical/0/tokenId",
    ] {
        let mut v = value();
        *v.pointer_mut(pointer).unwrap() = json!("missing-id");
        assert!(
            has_error(&errors(v), pointer, "unknown"),
            "unvalidated {pointer}"
        );
    }
    for pointer in [
        "/semantics/rules/0/body",
        "/semantics/expressions/0/node/items/0",
        "/semantics/captures/0/exprId",
        "/analysis/expressions/0/exprId",
        "/semantics/trivia/insertionPlan/0/boundary/ownerExprId",
        "/plan/memo/entries/0/exprId",
        "/plan/choices/0/exprId",
        "/plan/choices/0/candidateOrder/0",
        "/analysis/prefixTries/0/nodes/0/candidateExprIds/0",
    ] {
        let mut v = value();
        *v.pointer_mut(pointer).unwrap() = json!("expr:missing.ubnf:0:1:body");
        assert!(
            errors(v)
                .iter()
                .any(|e| e.to_string().contains("unknown expression")),
            "unvalidated {pointer}"
        );
    }
}

#[test]
fn duplicate_ids_are_rejected_in_each_namespace() {
    for pointer in [
        "/sources",
        "/grammarUnits",
        "/semantics/symbols",
        "/semantics/rules",
        "/semantics/tokens",
        "/semantics/expressions",
        "/semantics/captures",
        "/semantics/astTypes",
        "/semantics/recipes",
        "/semantics/trivia/policies",
        "/analysis/prefixTries",
        "/analysis/sccs",
    ] {
        let mut v = value();
        let items = v.pointer_mut(pointer).unwrap().as_array_mut().unwrap();
        items.push(items[0].clone());
        assert!(
            has_error(&errors(v), pointer, "duplicate"),
            "unvalidated {pointer}"
        );
    }
}

#[test]
fn grammar_source_and_span_consistency_are_checked() {
    let mut ir = example();
    let mut other = ir.sources[0].clone();
    other.id = "other.ubnf".into();
    other.path = other.id.clone();
    ir.sources.push(other);
    ir.grammar_units[0].source_id = "other.ubnf".into();
    assert!(has_error(
        &validate_references(&ir),
        "/grammarUnits/0/sourceId",
        "disagrees"
    ));
    let mut ir = example();
    ir.grammar_units[0].source.end = ir.sources[0].code_point_length + 1;
    assert!(has_error(
        &validate_references(&ir),
        "/grammarUnits/0/source",
        "exceeds"
    ));
    let mut ir = example();
    ir.semantics.expressions[0].source.start += 1;
    assert!(has_error(
        &validate_references(&ir),
        "/semantics/expressions/0/id",
        "source span"
    ));
}

#[test]
fn source_paths_and_percent_encoded_ids_are_stable() {
    for bad_path in [
        "/absolute.ubnf",
        "../escape.ubnf",
        "x/../a.ubnf",
        "./a.ubnf",
        "a//b.ubnf",
        "C:/a.ubnf",
        "a\\b.ubnf",
    ] {
        let mut ir = example();
        ir.sources[0].path = bad_path.into();
        assert!(has_error(
            &validate_references(&ir),
            "/sources/0/path",
            "POSIX"
        ));
    }
    let mut ir = example();
    ir.sources.push(Source {
        id: "%E6%96%87%20%25.ubnf".into(),
        path: "文 %.ubnf".into(),
        sha256: "0".repeat(64),
        code_point_length: 0,
    });
    assert!(validate_references(&ir).is_empty());
}

#[test]
fn trivia_boundary_indices_and_capture_ownership_are_checked() {
    let mut ir = example();
    ir.semantics.trivia.insertion_plan[0].boundary.child_index = Some(0);
    assert!(has_error(
        &validate_references(&ir),
        "/semantics/trivia/insertionPlan/0",
        "only afterChild"
    ));
    let mut ir = example();
    ir.semantics.trivia.insertion_plan[1].boundary.child_index = Some(u64::MAX);
    assert!(has_error(
        &validate_references(&ir),
        "/semantics/trivia/insertionPlan/1",
        "child index"
    ));
    let mut ir = example();
    ir.semantics.trivia.insertion_plan[1].boundary.child_index = None;
    assert!(has_error(
        &validate_references(&ir),
        "/semantics/trivia/insertionPlan/1",
        "child index"
    ));
    let mut ir = example();
    ir.semantics.captures[0].rule_id = ir.semantics.rules[1].symbol_id.clone();
    assert!(has_error(
        &validate_references(&ir),
        "/semantics/captures/0/exprId",
        "another rule"
    ));
    let mut ir = example();
    let id = ir.semantics.captures[0].expr_id.clone();
    ir.semantics
        .expressions
        .iter_mut()
        .find(|e| e.id == id)
        .unwrap()
        .capture_site_ids
        .clear();
    assert!(has_error(
        &validate_references(&ir),
        "/semantics/captures/0/siteId",
        "absent"
    ));
}

#[test]
fn every_feature_used_by_tinycalc_is_required() {
    let original = example();
    for feature in &original.required_features {
        let mut ir = original.clone();
        ir.required_features.retain(|f| f != feature);
        assert!(
            has_error(
                &validate_references(&ir),
                "/requiredFeatures",
                "missing required feature"
            ),
            "not checked: {feature:?}"
        );
    }
    let mut ir = original.clone();
    ir.required_features.push(ir.required_features[0]);
    assert!(has_error(
        &validate_references(&ir),
        "/requiredFeatures",
        "duplicate"
    ));
    let mut ir = original;
    ir.required_features.swap(0, 1);
    assert!(has_error(
        &validate_references(&ir),
        "/requiredFeatures",
        "sorted"
    ));
}

#[test]
fn additional_feature_gates_cover_semantics_and_plan() {
    let original = value();
    let rule = original["semantics"]["rules"][0]["symbolId"].clone();
    let capture = original["semantics"]["captures"][0]["siteId"].clone();
    let ast = original["semantics"]["astTypes"][0]["id"].clone();
    let expr = original["semantics"]["expressions"][0]["id"].clone();
    let cases = [
        (
            "regex",
            "/semantics/tokens/0/spec",
            json!({"kind":"regex","pattern":"a","regularSubset":true,"matching":"lookingAt","zeroWidth":"fail","javaFallback":"none","rustFallback":"generationError"}),
        ),
        (
            "externTokens",
            "/semantics/tokens/0/spec",
            json!({"kind":"extern","logicalId":"scanner","javaBinding":null,"rustBinding":null,"originalClassName":"Scanner","inputState":[],"resultFields":["ok","consumedEnd","matchedEnd","valueSpan","diagnostics","effects"]}),
        ),
        (
            "predictiveChoice",
            "/plan/choices/0/strategy",
            json!("predict"),
        ),
        (
            "longestChoice",
            "/plan/choices/0/strategy",
            json!("longest"),
        ),
        (
            "scopeUndo",
            "/semantics/scope/effects",
            json!([{"ruleId":rule,"mode":"lexical","events":[]} ]),
        ),
        (
            "eval",
            "/semantics/rules/0/eval",
            json!({"kind":"literal","strategy":"auto","params":{}}),
        ),
        (
            "catalog",
            "/semantics/catalogs",
            json!([{"context":"names","ruleId":rule,"captureSiteId":capture}]),
        ),
        (
            "typeof",
            "/semantics/typeofConstraints",
            json!([{"siteId":"constraint:0","ownCapture":capture,"referencedCapture":capture,"phase":"mapping","comparisonKind":"runtimeClassEquality","mismatch":"mappingFailure"}]),
        ),
        (
            "commonField",
            "/semantics/commonFields",
            json!([{"interfaceTypeId":ast,"variantTypeIds":[],"field":{"name":"value","cardinality":"scalar","valueType":{"kind":"text"}}}]),
        ),
        (
            "enum",
            "/semantics/astTypes/0",
            json!({"kind":"enum","id":ast,"name":"Enum","enclosingTypeId":null,"ruleId":rule,"variants":[{"literal":"a","variant":"A","javaName":"A","rustName":"A"}]}),
        ),
        (
            "imports",
            "/importEdges",
            json!([{"sourceId":original["sources"][0]["id"],"fromGrammarId":original["entry"]["grammarId"],"resolvedGrammarId":original["entry"]["grammarId"],"requestedPath":"x.ubnf","alias":null,"bindings":[]}]),
        ),
        (
            "recovery",
            "/semantics/recovery",
            json!([{"ruleId":rule,"mode":"SYNC","resolvedSyncPatterns":[";"],"consumeSync":true,"noSyncAction":"fail","progressRule":"legacySync","applicationSites":[],"astProjection":"legacyMarker","recipeId":null}]),
        ),
        (
            "leftRecursionRewrite",
            "/semantics/normalizations",
            json!([{"ruleId":rule,"originalBody":expr,"rewrittenBody":expr,"kind":"directLeftRecursionExtension","proof":{"kind":"leftRecursionNormalization","exprIds":[expr],"explanation":"fixture"}}]),
        ),
    ];
    for (feature, pointer, new_value) in cases {
        let mut v = original.clone();
        v["requiredFeatures"]
            .as_array_mut()
            .unwrap()
            .retain(|declared| declared.as_str() != Some(feature));
        *v.pointer_mut(pointer).unwrap() = new_value;
        assert!(
            has_error(&errors(v), "/requiredFeatures", &format!("\"{feature}\"")),
            "not checked: {feature}"
        );
    }
}

#[test]
fn analysis_coverage_and_local_graph_targets_are_checked() {
    let mut ir = example();
    ir.analysis.expressions.pop();
    assert!(has_error(
        &validate_references(&ir),
        "/analysis/expressions",
        "missing analysis"
    ));
    let mut ir = example();
    ir.analysis
        .expressions
        .push(ir.analysis.expressions[0].clone());
    assert!(has_error(
        &validate_references(&ir),
        "/analysis/expressions",
        "duplicate"
    ));
    let mut ir = example();
    ir.analysis.prefix_tries[0].nodes[0].edges[0].target = u64::MAX;
    assert!(has_error(
        &validate_references(&ir),
        "/analysis/prefixTries/0",
        "unknown trie"
    ));
    let mut ir = example();
    ir.plan.lexical[0].dfa = Some(Dfa {
        start_state: 0,
        states: vec![DfaState {
            id: 0,
            accepting: false,
            transitions: vec![DfaTransition {
                range: CharRange { start: 65, end: 90 },
                target: 7,
            }],
        }],
    });
    assert!(has_error(
        &validate_references(&ir),
        "/plan/lexical/0/dfa",
        "unknown DFA"
    ));
}

#[test]
fn auto_recovery_distinguishes_unanalyzed_follow_from_analyzed_empty_follow() {
    let mut ir = example();
    ir.semantics.recovery.push(Recovery {
        application_sites: Vec::new(),
        ast_projection: RecoveryAstProjection::LegacyMarker,
        consume_sync: true,
        mode: RecoveryMode::Auto,
        no_sync_action: RecoveryNoSyncAction::Fail,
        progress_rule: RecoveryProgressRule::LegacySync,
        recipe_id: None,
        resolved_sync_patterns: vec![";".into()],
        rule_id: ir.semantics.rules[0].symbol_id.clone(),
    });
    ir.required_features.push(RequiredFeature::Recovery);
    ir.required_features
        .sort_by_key(|f| serde_json::to_string(f).unwrap());
    assert!(has_error(
        &validate_references(&ir),
        "/semantics/recovery/0",
        "null is not an empty set"
    ));
    ir.analysis.rules[0].follow_sync_patterns = Some(Vec::new());
    assert_eq!(validate_references(&ir), Vec::<IrError>::new());
}
