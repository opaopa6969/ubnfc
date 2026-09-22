#[path = "../templates/driver_json.rs"]
mod json;
use json::{Json, ToJson};

#[test]
fn json_roundtrip_matches_reference_and_keeps_number_lexemes() {
    for source in [
        r#"{"id":9007199254740993,"n":-1.2500e+123,"text":"\"\\\/\b\f\n\r\t\u0000\ud83d\ude00日本語","a":[true,false,null,{},[]]}"#,
        r#"[0,-0,1e-99,10.2,"é"]"#,
    ] {
        let parsed = json::read(source).unwrap();
        let mut output = String::new();
        parsed.write(&mut output);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&output).unwrap(),
            serde_json::from_str::<serde_json::Value>(source).unwrap()
        );
        assert_eq!(json::read(&output).unwrap(), parsed);
        if source.contains("9007199254740993") {
            assert!(output.contains("9007199254740993"));
            assert!(output.contains("-1.2500e+123"));
            assert!(parsed.get("text").unwrap().string().unwrap().contains('😀'));
        }
    }
    let values = [1usize, 2];
    let value = Json::object([
        ("array", values.to_json()),
        ("optional", Some(Box::new("text".to_owned())).to_json()),
    ]);
    assert!(value.get("array").unwrap().string().is_err());
    assert!(value.boolean().is_err());
    assert!(Json::Bool(true).boolean().unwrap());
    assert_eq!(Json::Null.get("missing"), None);
    assert_eq!(None::<String>.to_json(), Json::Null);
}
#[test]
fn malformed_json_and_excessive_depth_are_rejected() {
    for source in [
        "",
        "null false",
        "01",
        "-01",
        "1.",
        ".1",
        "+1",
        "1e",
        "--1",
        "NaN",
        "[1,]",
        "{\"a\":0,}",
        "{\"a\":0,\"a\":1}",
        "{a:0}",
        r#""\ud800""#,
        r#""\udc00""#,
        r#""\ud800\u0000""#,
        r#""\x00""#,
        r#""\uqqqq""#,
        "\"a\nb\"",
        "\"unterminated",
    ] {
        assert!(json::read(source).is_err(), "{source}");
    }
    let nested = format!("{}0{}", "[".repeat(130), "]".repeat(130));
    assert!(json::read(&nested).is_err());
}
