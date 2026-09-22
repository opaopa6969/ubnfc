use serde_json::{json, Value};
use ubnfc_front::ir::*;

const EXAMPLE: &str = include_str!("../../../ir/examples/tinycalc.ir.json");

#[test]
fn tinycalc_is_byte_identical_and_references_are_valid() {
    let ir = from_json(EXAMPLE).unwrap();
    assert_eq!(to_canonical_json(&ir).as_bytes(), EXAMPLE.as_bytes());
    assert_eq!(validate_references(&ir), Vec::<IrError>::new());
}

#[test]
fn canonical_output_preserves_unicode_escapes_and_array_order() {
    let mut ir = from_json(EXAMPLE).unwrap();
    let payload = json!({
        "😀": "non-BMP", "\u{e000}": "BMP", "z": [3, 1, 2],
        "a": "日本語/e\u{301}/é\"\\\u{0000}\u{0001}\u{0008}\t\n\u{000c}\r\u{001f}\u{007f}\u{2028}"
    });
    let source = ir.grammar_units[0].source.clone();
    ir.grammar_units[0].annotations.push(Annotation {
        name: "escape-test".into(),
        source,
        arguments: [(
            "payload".to_owned(),
            serde_json::from_value(payload).unwrap(),
        )]
        .into(),
    });
    let encoded = to_canonical_json(&ir);
    assert!(encoded
        .contains("日本語/e\u{301}/é\\\"\\\\\\u0000\\u0001\\b\\t\\n\\f\\r\\u001f\u{007f}\u{2028}"));
    assert!(!encoded.contains("\\/"));
    assert!(encoded.find('😀').unwrap() > encoded.find('\u{e000}').unwrap());
    assert!(encoded.ends_with("}\n"));
    assert!(!encoded.ends_with("\n\n"));
    assert!(!encoded.contains('\r'));
    assert_eq!(from_json(&encoded).unwrap(), ir);
    let v: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        v["grammarUnits"][0]["annotations"][0]["arguments"]["payload"]["z"],
        json!([3, 1, 2])
    );
}

#[test]
fn nullable_fields_are_required_and_null_is_distinct_from_empty() {
    let mut value: Value = serde_json::from_str(EXAMPLE).unwrap();
    let original = from_json(EXAMPLE).unwrap();
    assert_eq!(original.analysis.rules[0].follow_sync_patterns, None);
    value["analysis"]["rules"][0]["followSyncPatterns"] = json!([]);
    let parsed = from_json(&value.to_string()).unwrap();
    assert_eq!(
        parsed.analysis.rules[0].follow_sync_patterns,
        Some(Vec::new())
    );
    assert_ne!(original, parsed);
    value["analysis"]["rules"][0]
        .as_object_mut()
        .unwrap()
        .remove("followSyncPatterns");
    assert!(matches!(
        from_json(&value.to_string()),
        Err(IrError::Json { .. })
    ));
}

#[test]
fn malformed_unknown_missing_variant_and_constant_errors_are_ir_errors() {
    assert!(matches!(from_json("{"), Err(IrError::Json { .. })));
    for (pointer, invalid) in [
        ("/unknown", json!(true)),
        ("/semantics/expressions/0/node/unknown", json!(true)),
        (
            "/semantics/expressions/0/node/kind",
            json!("notAnExpression"),
        ),
        (
            "/semantics/expressions/0/execution/transaction",
            json!("invalid"),
        ),
        ("/version", json!(2)),
        ("/analysisVersion", json!("unsupported")),
        ("/serialization", json!("unsupported")),
        (
            "/semantics/contracts/cursor/positions",
            json!(["matched", "consumed"]),
        ),
        ("/semantics/contracts/occurrences/diagnostics", json!(false)),
        ("/grammarUnits/0/rawSettings/comment/line", Value::Null),
    ] {
        let mut v: Value = serde_json::from_str(EXAMPLE).unwrap();
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        v.pointer_mut(parent).unwrap()[key] = invalid;
        assert!(
            matches!(from_json(&v.to_string()), Err(IrError::Json { .. })),
            "accepted {pointer}"
        );
    }
    let mut v: Value = serde_json::from_str(EXAMPLE).unwrap();
    v.as_object_mut().unwrap().remove("entry");
    assert!(matches!(
        from_json(&v.to_string()),
        Err(IrError::Json { .. })
    ));
}

#[test]
fn optional_comment_members_preserve_absence() {
    let absent: Settings =
        serde_json::from_value(json!({"whitespace": null, "interleave": null, "comment": {}}))
            .unwrap();
    let present: Settings = serde_json::from_value(
        json!({"whitespace": null, "interleave": null, "comment": {"line": ""}}),
    )
    .unwrap();
    assert_eq!(absent.comment.as_ref().unwrap().line, None);
    assert_eq!(present.comment.as_ref().unwrap().line.as_deref(), Some(""));
    assert_eq!(serde_json::to_value(absent).unwrap()["comment"], json!({}));
}

#[test]
fn canonical_output_sorts_sets_without_reordering_semantic_arrays() {
    let mut ir = from_json(EXAMPLE).unwrap();
    ir.required_features.reverse();
    ir.analysis.expressions[0].may_start.ranges = vec![
        CharRange { start: 90, end: 91 },
        CharRange { start: 65, end: 66 },
    ];
    ir.semantics.tokens[0].spec = TokenSpec::Builtin {
        which: BuiltinToken::CodePointSet,
        code_points: vec![90, 65],
    };
    ir.semantics.trivia.policies[0].effective_delimiter_set = vec![Delimiter::Characters {
        code_points: vec![32, 9],
    }];
    let original: Value = serde_json::to_value(&ir).unwrap();
    let ordered: Value = serde_json::from_str(&to_canonical_json(&ir)).unwrap();
    assert_eq!(
        ordered["analysis"]["expressions"][0]["mayStart"]["ranges"][0]["start"],
        65
    );
    assert_eq!(
        ordered["semantics"]["tokens"][0]["spec"]["codePoints"],
        json!([65, 90])
    );
    assert_eq!(
        ordered["semantics"]["trivia"]["policies"][0]["effectiveDelimiterSet"][0]["codePoints"],
        json!([9, 32])
    );
    let feature_names = ordered["requiredFeatures"].as_array().unwrap();
    assert!(feature_names
        .windows(2)
        .all(|w| w[0].as_str() < w[1].as_str()));
    for pointer in [
        "/sources",
        "/grammarUnits",
        "/semantics/expressions",
        "/semantics/rules",
        "/semantics/captures",
        "/semantics/astTypes",
        "/semantics/trivia/insertionPlan",
        "/plan/choices",
    ] {
        assert_eq!(
            original.pointer(pointer),
            ordered.pointer(pointer),
            "reordered {pointer}"
        );
    }
    assert_eq!(
        serde_json::to_value(&ir).unwrap(),
        original,
        "serializer mutated input"
    );
}
