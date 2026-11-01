//! D-072: `plan.scans` が front で何を証明し、Rust backend がそれをどう畳むか。
//!
//! 実行時の同値（走査 on / off で観測が一致すること）は差分ファジングの
//! in-backend A/B（`crates/ubnfc-fuzz`、`options.scan`）が担う。ここでは
//! 「どの形が走査の対象になり、どの形が対象にならないか」を固定する。
use ubnfc_front::ir::{GrammarIr, ScanPlanKind};
use ubnfc_front::pipeline::compile;

fn ir(body: &str) -> GrammarIr {
    let text = format!(
        "grammar Scan {{
            @ubnf: 2
            @package: org.example.scan
            @whitespace: none
            token DIGIT = DIGIT
            token SAFE = NEGATION('\"')
            {body}
        }}"
    );
    compile("scan.ubnf", &|_| Ok(text.clone()))
        .expect("compile")
        .ir
}

fn source(ir: &GrammarIr) -> String {
    ubnfc_rust::emit(
        ir,
        &ubnfc_rust::EmitOptions {
            emit_driver: false,
            ..Default::default()
        },
    )
    .expect("emit")
    .files["parser.rs"]
        .clone()
}

#[test]
fn a_character_class_repetition_carries_a_complete_bulk_table_and_one_tight_loop() {
    let ir = ir("@root Root ::= '\"' { DIGIT } '\"' ;");
    let scans = &ir.plan.scans;
    let class = scans
        .iter()
        .find(|s| s.kind == ScanPlanKind::CharClassRepetition)
        .expect("反復の一括走査表");
    // `[0-9]` の bitset（U+0030..U+0039）。ASCII だけなので区間側は空。
    assert_eq!(class.class.as_ref().unwrap().ascii_low, 0x03ff << 48);
    assert_eq!(class.class.as_ref().unwrap().ascii_high, 0);
    assert!(class.class.as_ref().unwrap().ranges.is_empty());
    // 候補が全部クラスなので、表の外の文字では本体が必ず失敗する。
    assert!(class.class.as_ref().unwrap().complete);
    assert!(scans.iter().any(|s| s.root), "走査関数の起点がある");
    let source = source(&ir);
    assert!(
        source.contains(
            "if self.options.scan && !DIAG && !self.options.lexical && !self.options.occurrences && !out.state.invert {"
        ),
        "走査は options.scan / fast mode / 出現表なし / 非 invert のときだけ"
    );
    assert!(source.contains("fn s"), "走査関数を生成する");
    assert!(
        source.contains("u64::from_le_bytes(chunk.try_into().expect(\"8 bytes\"))"),
        "ASCII は安全な 8 byte 読み込みで SWAR 判定する"
    );
    // 表が完全なので、走査が止まった位置で本体を呼び直さない。
    assert!(
        !source.contains("let Some(n)=self.s"),
        "完全な表では本体を呼ばない"
    );
}

#[test]
fn an_ordered_choice_of_classes_and_a_longer_alternative_keeps_the_general_path_after_the_run() {
    // `{ Escape | SAFE }` は JSON の文字列本体と同じ形。表は SAFE から
    // Escape の先頭文字 `\` を除いた集合で、表が尽きたら本体を 1 度呼ぶ。
    let ir = ir("@root Root ::= { Escape | SAFE } ; Escape ::= '\\\\' SAFE ;");
    let class = ir
        .plan
        .scans
        .iter()
        .find(|s| s.kind == ScanPlanKind::CharClassRepetition)
        .expect("一括走査表");
    let table = class.class.as_ref().unwrap();
    assert!(!table.complete, "Escape は 1 文字のクラスではない");
    // `"`（0x22）は SAFE の除外、`\`（0x5c）は Escape の先頭文字なので表に無い。
    assert_eq!((table.ascii_low >> 0x22) & 1, 0);
    assert_eq!((table.ascii_high >> (0x5c - 64)) & 1, 0);
    assert_eq!((table.ascii_high >> (0x5d - 64)) & 1, 1);
    assert!(
        source(&ir).contains("let Some(n)=self.s"),
        "表の外は本体へ落とす"
    );
}

#[test]
fn a_capture_inside_the_repetition_keeps_the_reference_path() {
    let ir = ir("@root @mapping(Root, params=[text]) Root ::= { DIGIT @text } ;");
    assert!(
        ir.plan.scans.is_empty(),
        "capture site を含む領域は畳まない"
    );
    assert!(!source(&ir).contains("self.options.scan"));
}

#[test]
fn implicit_trivia_inside_the_repetition_keeps_the_reference_path() {
    let java_style = |body: &str| {
        let text = format!(
            "grammar Scan {{
                @ubnf: 2
                @package: org.example.scan
                @whitespace: javaStyle
                token DIGIT = DIGIT
                {body}
            }}"
        );
        compile("scan.ubnf", &|_| Ok(text.clone()))
            .expect("compile")
            .ir
    };
    // 本体が 2 要素の列なので、周回の中に区切りの読み飛ばしが入る。
    assert!(
        java_style("@root Root ::= { DIGIT DIGIT } ;")
            .plan
            .scans
            .is_empty(),
        "trivia 挿入の境界を持つ式は走査で再生できない"
    );
    // 本体が 1 つの終端なら周回の中に境界は無く（挿入は外側の列の境界に付く）、
    // 暗黙 trivia のある文法でも反復そのものは畳める。
    assert!(!java_style("@root Root ::= { DIGIT } ;")
        .plan
        .scans
        .is_empty());
}

#[test]
fn a_single_terminal_without_a_repetition_is_not_worth_a_scan() {
    let ir = ir("@root Root ::= DIGIT DIGIT ;");
    assert!(ir.plan.scans.is_empty(), "反復の無い領域は回数が減らない");
}

#[test]
fn a_terminal_only_sequence_of_optionals_and_repetitions_becomes_one_scan() {
    // JSON の `Number ::= [ '-' ] Int [ Frac ] [ Exp ]` と同じ形。
    let ir = ir("@root Root ::= [ '-' ] Int [ Frac ] ; Int ::= DIGIT { DIGIT } ; Frac ::= '.' DIGIT { DIGIT } ;");
    let root = ir
        .plan
        .scans
        .iter()
        .find(|s| s.root && s.kind == ScanPlanKind::TerminalSequence)
        .expect("終端だけの列は 1 本の走査");
    assert_eq!(
        root.rule_ids,
        vec!["Scan::Frac".to_owned(), "Scan::Int".to_owned()],
        "走査が畳み込む規則を記録する"
    );
}
