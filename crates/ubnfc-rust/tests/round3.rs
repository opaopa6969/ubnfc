//! M5 round 3: Extern token の先頭文字宣言（IR `firstChars`）と共通接頭辞の左因数分解が
//! 生成器で使われることを、生成ソースの形で固定する。実行時の同値は
//! `examples/p4-rust/tests/fixtures.rs::predict_matches_reference_path_on_truncated_and_corrupted_inputs`
//! （predict / factor の on/off × memo の on/off を参照経路と比較）が担う。
use serde_json::{json, Value};
#[path = "support/fixture.rs"]
#[allow(dead_code)]
mod fixture;
use fixture::*;

fn external(first_chars: Option<Value>) -> Value {
    let mut spec = json!({"kind":"extern","logicalId":"scan","originalClassName":"Scanner","javaBinding":null,"rustBinding":null,"inputState":["consumed","matched","mode","invertMatch","resetMatchedWithConsumed"],"resultFields":["ok","consumedEnd","matchedEnd","valueSpan","diagnostics","effects"]});
    if let Some(first) = first_chars {
        spec["firstChars"] = first;
    }
    spec
}

fn parser_source(ir: &ubnfc_front::ir::GrammarIr) -> String {
    let generated = ubnfc_rust::emit(
        ir,
        &ubnfc_rust::EmitOptions {
            emit_driver: false,
            ..Default::default()
        },
    )
    .unwrap();
    generated.files["parser.rs"].clone()
}

#[test]
fn extern_first_chars_declaration_enables_candidate_exclusion() {
    // Root ::= NUMBER 'x' | 'y'。先頭が Extern token の候補は宣言が無ければ unknown。
    let body = || {
        choice(
            vec![seq(vec![token(), literal("x")]), literal("y")],
            "ordered",
        )
    };
    let unknown = grammar(vec![("Root", body())], Some(external(None)));
    assert!(
        !parser_source(&unknown).contains("guard_e"),
        "宣言の無い Extern で始まる候補は除外しない"
    );
    let declared = grammar(
        vec![("Root", body())],
        Some(external(Some(json!([{"start":48,"end":57}])))),
    );
    let source = parser_source(&declared);
    assert!(
        source.contains("fn guard_e"),
        "宣言があれば候補除外の再生関数を生成する"
    );
    assert!(
        source.contains("matches!(c as u32,48..=57)"),
        "判定は宣言した先頭文字集合"
    );
    // 参照実装の検証（validate_references）は昇順・互いに素な区間だけを受け付ける。
    let mut bad = declared.clone();
    if let ubnfc_front::ir::TokenSpec::Extern { first_chars, .. } =
        &mut bad.semantics.tokens[0].spec
    {
        *first_chars = Some(vec![
            ubnfc_front::ir::CharRange { start: 50, end: 57 },
            ubnfc_front::ir::CharRange { start: 48, end: 49 },
        ]);
    }
    assert!(!ubnfc_front::ir::validate_references(&bad).is_empty());
    assert!(ubnfc_front::ir::validate_references(&declared).is_empty());
}

#[test]
fn common_prefix_candidates_are_left_factored() {
    // Root ::= A 'x' | A 'y' | 'z'; A ::= 'a'。先頭 2 候補は A を共有する。
    let ir = grammar(
        vec![
            (
                "Root",
                choice(
                    vec![
                        seq(vec![reference("A"), literal("x")]),
                        seq(vec![reference("A"), literal("y")]),
                        literal("z"),
                    ],
                    "ordered",
                ),
            ),
            ("A", literal("a")),
        ],
        None,
    );
    let source = parser_source(&ir);
    assert!(
        source.contains("if self.options.factor && !out.state.invert {"),
        "共通接頭辞の候補列は因数分解経路を生成する"
    );
    assert!(source.contains("'factored: {"));
    // 最長一致の選択には適用しない。
    let longest = grammar(
        vec![
            (
                "Root",
                choice(
                    vec![
                        seq(vec![reference("A"), literal("x")]),
                        seq(vec![reference("A"), literal("y")]),
                    ],
                    "longest",
                ),
            ),
            ("A", literal("a")),
        ],
        None,
    );
    assert!(!parser_source(&longest).contains("'factored: {"));
}
