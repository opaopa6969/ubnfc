use serde_json::{json, Value};
use ubnfc_harness::canonical::{diff, AstValue};

fn ast(json: Value) -> AstValue {
    AstValue::try_from(json).unwrap()
}

#[test]
fn json_roundtrip_preserves_all_value_kinds_and_code_point_spans() {
    let json = json!({"type":"Root", "span":[0,3], "fields":{
        "text":"😀あ", "int":i64::MIN, "max":i64::MAX, "none":null,
        "items":["",-3,[],null,{"type":"Leaf","span":[1,2],"fields":{}}]
    }});
    let expected = ast(json.clone());
    assert_eq!(Value::from(&expected), json);
    let encoded = serde_json::to_string(&expected).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&encoded).unwrap(), json);
    assert_eq!(
        serde_json::from_str::<AstValue>(&encoded).unwrap(),
        expected
    );
    assert!(diff(&expected, &expected).is_empty());
    for scalar in [json!(null), json!("😀"), json!(42), json!([])] {
        assert_eq!(Value::from(ast(scalar.clone())), scalar);
    }
}

#[test]
fn unsupported_or_malformed_values_are_rejected_without_data_loss() {
    for json in [
        json!(true),
        json!(1.5),
        json!(18446744073709551615_u64),
        json!({}),
        json!({"type":"N","span":[0,1],"fields":{},"extra":true}),
        json!({"type":"N","span":[2,1],"fields":{}}),
        json!({"type":"N","span":[-1,1],"fields":{}}),
    ] {
        assert!(AstValue::try_from(json).is_err());
    }
    let error =
        AstValue::try_from(json!({"type":"N","span":[0,1],"fields":{"a/b~":true}})).unwrap_err();
    assert_eq!(error.path, "/fields/a~1b~0");
}

#[test]
fn diff_reports_every_field_span_and_list_difference_in_stable_pointer_order() {
    let expected = ast(json!({"type":"Before", "span":[0,5], "fields":{
        "a/b~":1, "gone":null, "list":["old",2,3],
        "nested":{"type":"Leaf","span":[1,3],"fields":{"x":"old"}}
    }}));
    let actual = ast(json!({"type":"After", "span":[1,6], "fields":{
        "a/b~":"1", "added":null, "list":["new"],
        "nested":{"type":"Leaf","span":[2,4],"fields":{"x":"new"}}
    }}));
    let differences = diff(&expected, &actual);
    assert_eq!(
        differences
            .iter()
            .map(|d| d.path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "/fields/a~1b~0",
            "/fields/added",
            "/fields/gone",
            "/fields/list/0",
            "/fields/list/1",
            "/fields/list/2",
            "/fields/nested/fields/x",
            "/fields/nested/span/0",
            "/fields/nested/span/1",
            "/span/0",
            "/span/1",
            "/type",
        ]
    );
    assert_eq!(differences[1].expected, None);
    assert_eq!(differences[1].actual, Some(Value::Null));
    assert_eq!(differences[2].expected, Some(Value::Null));
    assert_eq!(differences[2].actual, None);
    let backwards = diff(&actual, &expected);
    for (forward, backward) in differences.iter().zip(backwards) {
        assert_eq!(forward.path, backward.path);
        assert_eq!(forward.expected, backward.actual);
        assert_eq!(forward.actual, backward.expected);
    }
    let root = diff(&AstValue::Null, &AstValue::List(vec![AstValue::Int(1)]));
    assert_eq!(root.len(), 1);
    assert_eq!(root[0].path, "");
}
