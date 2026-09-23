//! D-038: UBNF 表層言語のバージョン門。v2 の表層機能は front が v1 の正規形へ
//! 脱糖するので生成物は変わらないが、**未知のバージョンは明示拒否**する。
use ubnfc_front::ir::{from_json, to_canonical_json, GrammarIr};
use ubnfc_rust::{emit, EmitOptions};

fn calc() -> GrammarIr {
    from_json(include_str!(
        "../../../ir/fixtures/story__code__calc.ubnf.ir.json"
    ))
    .unwrap()
}

#[test]
fn known_versions_emit_and_unknown_versions_are_rejected() {
    let options = EmitOptions {
        emit_driver: false,
        ..Default::default()
    };
    let mut ir = calc();
    assert_eq!(ir.language_version, 1);
    let v1 = emit(&ir, &options).unwrap();
    // v2 を宣言しただけでは生成物は 1 バイトも変わらない（脱糖は front で完結する）。
    ir.language_version = 2;
    assert_eq!(emit(&ir, &options).unwrap().files, v1.files);
    for version in [0, 3, u64::MAX] {
        ir.language_version = version;
        let error = emit(&ir, &options).unwrap_err().to_string();
        assert!(error.contains("languageVersion"), "{version}: {error}");
    }
}

#[test]
fn the_ir_round_trips_the_language_version() {
    let mut ir = calc();
    ir.language_version = 2;
    let json = to_canonical_json(&ir);
    assert!(json.contains("\"languageVersion\": 2,"), "{json:.400}");
    assert_eq!(from_json(&json).unwrap().language_version, 2);
    // 省略した IR は読めない（必須キー。IR は再生成する）。
    assert!(from_json(&json.replace("\"languageVersion\": 2,\n", "")).is_err());
}
