//! CI 用の固定予算 campaign。seed を固定しているので結果は決定的で、
//! 新しい差分が生まれたときだけ赤くなる。既知の所見は signature で明示登録する。
//!
//! 前提ツールは harness の e2e テストと同じ（`javac` / `java` / `ubnfc-rust` /
//! `ubnfc-java` の jar）。無ければ skip せず失敗する。

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use ubnfc_fuzz::campaign::{self, Config};

/// `docs/reports/2026-09-23-differential-fuzzing.md` で triage 済みの所見のうち、まだ残っているもの。
/// 契約・観測側（B-1 / C-1 / A-5 / A-8 / A-9 と A-6 の一部）は
/// `docs/reports/2026-09-23-observation-contract.md` の D-024 〜 D-029 で閉じたので一覧から外した。
/// A-1 / A-2 / A-3 / A-4 はこの固定 seed では出ないので登録しない（出たら赤になる）。
/// 新しい signature が出たらこのテストが落ち、report に追記するまで通らない。
/// 受理そのものが食い違う差分クラス。A-1（`@recovery(AUTO)` の適用）を直した後は
/// 1 件も出てはならないので、既知 signature には決して入れない。
const ACCEPTANCE: &[&str] = &[
    "diff:/accepted",
    "diff:/allConsumed",
    "diff:/consumed",
    "diff:/matched",
    "diff:/parsedStatus",
    "diff:/ast",
    "diff:/captures",
    "diff:/prefixCaptures",
];

const KNOWN: &[&str] = &[
    // A-7: 全入力解析が失敗したのに Rust が prefix 側の capture を返す
    "diff:/captures/*",
    // A-6 の残り: 最遠位置に載る候補の差（`@leftAssoc` の静的失敗ヒント計画、Extern token の表示名）。
    // 蓄積規則そのものは共有契約のまま（D-027）。
    "diff:/diagnostics/*/farthestExpected/*",
];

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn scratch() -> Scratch {
    let output = Command::new("mktemp").arg("-d").output().unwrap();
    assert!(output.status.success());
    Scratch(PathBuf::from(
        String::from_utf8(output.stdout).unwrap().trim(),
    ))
}

#[test]
fn a_fixed_seed_campaign_reaches_both_backends_and_finds_no_unregistered_divergence() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let summary = campaign::run(&Config {
        start_seed: 4001,
        seeds: 4,
        inputs: 10,
        jobs: 2,
        budget: Some(Duration::from_secs(600)),
        findings: None,
        // 0 は縮小なし。CI では最小化まではやらない。
        shrink_builds: 0,
        driver_timeout: Duration::from_secs(120),
        timeout_retries: 2,
        workdir: scratch.0.clone(),
    });
    // front が自分の IR / 解析の契約で自分の出力を拒否したら discard になる
    // （2026-09-23 の所見 D、修正済み）。`validate` を通った文法は必ず IR まで通る。
    assert_eq!(
        summary.discarded, 0,
        "front の discard: {:?}",
        summary.discards
    );
    assert!(
        summary.compiled >= 3,
        "文法が build できていない: {:?}",
        summary.discards
    );
    assert!(
        summary.comparisons >= 30 && summary.driver_errors == 0,
        "driver が動いていない: 比較 {} / error {}",
        summary.comparisons,
        summary.driver_errors
    );
    // 受理・不受理の両方が出ていなければ、入力生成が偏っていて差分を探せていない。
    assert!(
        summary.accepted_all > 0 && summary.rejected > 0,
        "受理 {} / 部分 {} / 不受理 {}",
        summary.accepted_all,
        summary.partial,
        summary.rejected
    );
    // D-072: 同じ Rust backend を `options.scan` だけ変えた in-backend A/B。
    assert!(
        summary.scan_comparisons >= 30,
        "走査 A/B が回っていない: {}",
        summary.scan_comparisons
    );
    assert!(
        summary.scan_findings.is_empty(),
        "一括走査 on / off で観測が食い違う: {:#?}",
        summary
            .scan_findings
            .iter()
            .map(|f| (f.seed, f.input.clone(), f.differences.clone()))
            .collect::<Vec<_>>()
    );
    let unknown: Vec<_> = summary
        .findings
        .iter()
        .filter(|finding| !KNOWN.contains(&finding.signature.as_str()))
        .map(|finding| format!("{} ({})", finding.signature, finding.slug))
        .collect();
    assert!(unknown.is_empty(), "未登録の差分: {unknown:#?}");
}

/// UBNF v2（`@ubnf: 2` の左再帰ルール、D-039）を両 backend で突き合わせる。
/// front の脱糖が v1 の正規形を作るので IR は両 backend で同じだが、
/// 「脱糖した文法が実際に build でき、同じ入力で同じ観測を返す」ことは
/// ここで初めて機械的に確かめられる。seed は `gen::V2_SEED_BASE` の帯（既存
/// seed の生成物は 1 バイトも変わらない）。
#[test]
fn c_v2_left_recursive_grammars_agree_across_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let start = ubnfc_fuzz::gen::V2_SEED_BASE + 1;
    for seed in start..start + 3 {
        let grammar = ubnfc_fuzz::gen::generate(seed);
        assert!(grammar.features.contains("ubnf2"), "seed {seed}");
    }
    let summary = campaign::run(&Config {
        start_seed: start,
        seeds: 3,
        inputs: 10,
        jobs: 2,
        budget: Some(Duration::from_secs(900)),
        findings: None,
        shrink_builds: 0,
        driver_timeout: Duration::from_secs(120),
        timeout_retries: 2,
        workdir: scratch.0.clone(),
    });
    assert_eq!(
        summary.discarded, 0,
        "front の discard: {:?}",
        summary.discards
    );
    assert!(summary.compiled >= 2, "文法が build できていない");
    assert!(
        summary.comparisons >= 20 && summary.driver_errors == 0,
        "driver が動いていない: 比較 {} / error {}",
        summary.comparisons,
        summary.driver_errors
    );
    // D-072: 同じ Rust backend を `options.scan` だけ変えた in-backend A/B。
    assert!(
        summary.scan_comparisons >= 20,
        "走査 A/B が回っていない: {}",
        summary.scan_comparisons
    );
    assert!(
        summary.scan_findings.is_empty(),
        "一括走査 on / off で観測が食い違う: {:#?}",
        summary
            .scan_findings
            .iter()
            .map(|f| (f.seed, f.input.clone(), f.differences.clone()))
            .collect::<Vec<_>>()
    );
    let unknown: Vec<_> = summary
        .findings
        .iter()
        .filter(|finding| !KNOWN.contains(&finding.signature.as_str()))
        .map(|finding| format!("{} ({})", finding.signature, finding.slug))
        .collect();
    assert!(unknown.is_empty(), "未登録の差分: {unknown:#?}");
}

/// UBNF v2 のリテラル語境界（D-040）を両 backend で突き合わせる。
/// v2 の seed 帯は `V2_LITERALS` から `let` / `letter` / `in` / `int` のような
/// **末尾が識別子構成文字の語**を引くので、`@literalBoundary: word`（v2 の既定）が
/// 受理を変える位置を必ず踏む。Java は UTF-16 単位、Rust は UTF-8 byte 単位で
/// 「一致直後の文字」を見るので、非 BMP・非 ASCII が続く入力で食い違わないことも
/// ここで押さえる。
#[test]
fn d_v2_word_boundary_literals_agree_across_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let start = ubnfc_fuzz::gen::V2_SEED_BASE + 40;
    for seed in start..start + 3 {
        let grammar = ubnfc_fuzz::gen::generate(seed);
        assert!(
            grammar.features.contains("wordBoundary"),
            "seed {seed} が語境界つきリテラルを含んでいない"
        );
    }
    let summary = campaign::run(&Config {
        start_seed: start,
        seeds: 3,
        inputs: 12,
        jobs: 2,
        budget: Some(Duration::from_secs(900)),
        findings: None,
        shrink_builds: 0,
        driver_timeout: Duration::from_secs(120),
        timeout_retries: 2,
        workdir: scratch.0.clone(),
    });
    assert_eq!(
        summary.discarded, 0,
        "front の discard: {:?}",
        summary.discards
    );
    assert!(summary.compiled >= 2, "文法が build できていない");
    assert!(
        summary.comparisons >= 20 && summary.driver_errors == 0,
        "driver が動いていない: 比較 {} / error {}",
        summary.comparisons,
        summary.driver_errors
    );
    // D-072: 同じ Rust backend を `options.scan` だけ変えた in-backend A/B。
    assert!(
        summary.scan_comparisons >= 20,
        "走査 A/B が回っていない: {}",
        summary.scan_comparisons
    );
    assert!(
        summary.scan_findings.is_empty(),
        "一括走査 on / off で観測が食い違う: {:#?}",
        summary
            .scan_findings
            .iter()
            .map(|f| (f.seed, f.input.clone(), f.differences.clone()))
            .collect::<Vec<_>>()
    );
    let unknown: Vec<_> = summary
        .findings
        .iter()
        .filter(|finding| !KNOWN.contains(&finding.signature.as_str()))
        .map(|finding| format!("{} ({})", finding.signature, finding.slug))
        .collect();
    assert!(unknown.is_empty(), "未登録の差分: {unknown:#?}");
}

/// UBNF v2 の code point クラス（D-063）を両 backend で突き合わせる。
/// 3 つの seed は、`NEGATION` の除外集合のエスケープ・`CHAR_RANGE` の**補助面**の範囲・
/// `CHAR_RANGE` の `\uXXXX` / `\u{X…}` 境界を 1 つずつ含む（生成器の v2 seed 帯）。
/// 前後 2 つは v1 の綴りと**同じ集合**を別の書き方で表すので、front の展開が狂えば
/// 両 backend が同時にモデルと食い違う。補助面のほうは Java（UTF-16 の surrogate 対）と
/// Rust（UTF-8 の 1 scalar）で「1 文字」の読み方が構造的に違う唯一の token なので、
/// ここで必ず踏ませる。
#[test]
fn e_v2_code_point_class_tokens_agree_across_backends() {
    ubnfc_fuzz::tune_generated_builds();
    for (offset, feature) in [
        (3u64, "token:negationEscape"),
        (24, "token:charRangeSupplementary"),
        (29, "token:charRangeEscape"),
    ] {
        let seed = ubnfc_fuzz::gen::V2_SEED_BASE + offset;
        let grammar = ubnfc_fuzz::gen::generate(seed);
        assert!(
            grammar.features.contains(feature),
            "seed {seed} に {feature} が無い: {:?}",
            grammar.features
        );
        let scratch = scratch();
        let summary = campaign::run(&Config {
            start_seed: seed,
            seeds: 1,
            inputs: 12,
            jobs: 1,
            budget: Some(Duration::from_secs(600)),
            findings: None,
            shrink_builds: 0,
            driver_timeout: Duration::from_secs(120),
            timeout_retries: 2,
            workdir: scratch.0.clone(),
        });
        assert_eq!(
            summary.discarded, 0,
            "front の discard: {:?}",
            summary.discards
        );
        assert_eq!(summary.compiled, 1, "seed {seed} が build できていない");
        assert!(
            summary.comparisons >= 10 && summary.driver_errors == 0,
            "seed {seed}: 比較 {} / error {}",
            summary.comparisons,
            summary.driver_errors
        );
        // D-072: 同じ Rust backend を `options.scan` だけ変えた in-backend A/B。
        assert!(
            summary.scan_comparisons >= 10,
            "seed {seed}: 走査 A/B が回っていない ({})",
            summary.scan_comparisons
        );
        assert!(
            summary.scan_findings.is_empty(),
            "seed {seed}: 一括走査 on / off で観測が食い違う: {:#?}",
            summary
                .scan_findings
                .iter()
                .map(|f| (f.input.clone(), f.differences.clone()))
                .collect::<Vec<_>>()
        );
        let unknown: Vec<_> = summary
            .findings
            .iter()
            .filter(|finding| !KNOWN.contains(&finding.signature.as_str()))
            .map(|finding| format!("{} ({})", finding.signature, finding.slug))
            .collect();
        assert!(
            unknown.is_empty(),
            "seed {seed} の未登録の差分: {unknown:#?}"
        );
        // 受理そのものが食い違うのは、どの seed でも 1 件も出てはならない。
        let acceptance: Vec<_> = summary
            .findings
            .iter()
            .filter(|f| ACCEPTANCE.iter().any(|p| f.signature.starts_with(p)))
            .filter(|f| !KNOWN.contains(&f.signature.as_str()))
            .map(|f| format!("{} ({})", f.signature, f.slug))
            .collect();
        assert!(
            acceptance.is_empty(),
            "seed {seed} で受理が食い違う: {acceptance:#?}"
        );
    }
}

/// issue #41 の回帰試験。v2 帯の seed 276（= 100276）は `@recovery(SKIP)` を通った解析で
/// 回復診断の候補（`expected` / `farthestExpected`）と `recoveries[].captureOccurrences` が
/// 食い違っていた。原因はどちらも Rust 側で、契約は `docs/harness-driver-protocol.md` の
/// D-025（`captureOccurrences`）と D-027（最遠到達の候補）。
/// この seed は `KNOWN` の signature も含めて 1 件も差分を出してはならない。
#[test]
fn f_v2_recovery_diagnostics_agree_across_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let seed = ubnfc_fuzz::gen::V2_SEED_BASE + 276;
    let grammar = ubnfc_fuzz::gen::generate(seed);
    assert!(
        grammar.features.contains("recovery"),
        "seed {seed} に recovery が無い: {:?}",
        grammar.features
    );
    let summary = campaign::run(&Config {
        start_seed: seed,
        seeds: 1,
        inputs: 12,
        jobs: 1,
        budget: Some(Duration::from_secs(600)),
        findings: None,
        shrink_builds: 0,
        driver_timeout: Duration::from_secs(120),
        timeout_retries: 2,
        workdir: scratch.0.clone(),
    });
    assert_eq!(
        summary.discarded, 0,
        "front の discard: {:?}",
        summary.discards
    );
    assert_eq!(summary.compiled, 1, "seed {seed} が build できていない");
    assert!(
        summary.comparisons >= 10 && summary.driver_errors == 0,
        "seed {seed}: 比較 {} / error {}",
        summary.comparisons,
        summary.driver_errors
    );
    let found: Vec<_> = summary
        .findings
        .iter()
        .map(|finding| format!("{} ({})", finding.signature, finding.slug))
        .collect();
    assert!(found.is_empty(), "seed {seed} の差分: {found:#?}");
}

/// A-1 の回帰試験。seed 269 は `@recovery(AUTO)` の適用範囲が両 backend で食い違い、
/// 同じ入力の受理・カーソル・capture・AST が変わっていた（13 差分クラス）。
/// 契約は `docs/design/03-design-v1.md` §3.10 / D-024。
#[test]
fn b_recovery_auto_keeps_acceptance_identical_across_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let scratch = scratch();
    let summary = campaign::run(&Config {
        start_seed: 269,
        seeds: 1,
        inputs: 24,
        jobs: 1,
        budget: Some(Duration::from_secs(600)),
        findings: None,
        shrink_builds: 0,
        driver_timeout: Duration::from_secs(120),
        timeout_retries: 2,
        workdir: scratch.0.clone(),
    });
    assert_eq!(summary.compiled, 1, "seed 269 が build できていない");
    assert!(
        summary.accepted_all > 0 && summary.rejected > 0,
        "受理 {} / 不受理 {}",
        summary.accepted_all,
        summary.rejected
    );
    let acceptance: Vec<_> = summary
        .findings
        .iter()
        .filter(|f| ACCEPTANCE.iter().any(|p| f.signature.starts_with(p)))
        .map(|f| format!("{} ({})", f.signature, f.slug))
        .collect();
    assert!(acceptance.is_empty(), "受理が食い違う差分: {acceptance:#?}");
}

/// #75 の回帰試験（固定 seed campaign）。直接 capture の帯（`gen::DIRECT_CAPTURE_SEED_BASE`）は
/// 乱数を引かずに `Direct ::= DN @dn ':' ( DN ) @dg [ DI @di ] { ',' DN @dl } DS @ds` を足し、
/// 根の `@mapping` の field に載せる。v1 帯 4 本と v2 帯 2 本（奇数 seed は `DS` が extern）。
/// 旧 front（D-022: Java int / Rust 字句）では、同じ帯の seed 1000002 が
/// `diff:/ast/fields/direct/fields/dn` と `.../dl/*` を出した（縮小入力 `iあ0:1''`、
/// `tests/direct_token_capture.rs`）。seed 1000002 は乱択部分に別の差
/// （`diff:/ast/fields/tail/*`、issue #78: mixed な選択の text 候補を Rust が意味値にしない）を
/// 持っていた。D-078 で Rust を直したので固定帯へ戻した（`tests/mixed_choice_text_value.rs`）。
/// seed 1000189 は同じ signature を別原因（issue #86: 意味値の領域の中の text だけの規則を
/// Rust が字句の値にしていた）で出していた。直したので固定帯に加えた（`tests/text_rule_value_region.rs`）。
#[test]
fn g_direct_token_captures_agree_across_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let base = ubnfc_fuzz::gen::DIRECT_CAPTURE_SEED_BASE;
    let v2 = base + ubnfc_fuzz::gen::V2_SEED_BASE;
    let mut accepted = 0usize;
    for seed in [base + 1, base + 2, base + 3, base + 189, v2 + 1, v2 + 2] {
        let grammar = ubnfc_fuzz::gen::generate(seed);
        assert!(
            grammar.features.contains("capture:directToken"),
            "seed {seed}: {:?}",
            grammar.features
        );
        let scratch = scratch();
        let summary = campaign::run(&Config {
            start_seed: seed,
            seeds: 1,
            inputs: 12,
            jobs: 1,
            budget: Some(Duration::from_secs(600)),
            findings: None,
            shrink_builds: 0,
            driver_timeout: Duration::from_secs(120),
            timeout_retries: 2,
            workdir: scratch.0.clone(),
        });
        assert_eq!(
            summary.discarded, 0,
            "front の discard: {:?}",
            summary.discards
        );
        assert_eq!(summary.compiled, 1, "seed {seed} が build できていない");
        assert!(
            summary.comparisons >= 10 && summary.driver_errors == 0,
            "seed {seed}: 比較 {} / error {}",
            summary.comparisons,
            summary.driver_errors
        );
        accepted += summary.accepted_all;
        let unknown: Vec<_> = summary
            .findings
            .iter()
            .filter(|finding| !KNOWN.contains(&finding.signature.as_str()))
            .map(|finding| format!("{} ({})", finding.signature, finding.slug))
            .collect();
        assert!(
            unknown.is_empty(),
            "seed {seed} の未登録の差分: {unknown:#?}"
        );
    }
    // 全入力受理が無ければ `Direct` の AST を比較できていない。
    assert!(accepted > 0, "直接 capture の帯で受理が 0 件");
}

/// #93 の回帰試験（固定 seed campaign）。text group の帯（`gen::TEXT_GROUP_SEED_BASE`）は
/// 乱数を引かずに `TextGroup` を足し、根の `@mapping` の field に載せる。3 つの形は
/// いずれも修正前の Rust では出現の値が Java と食い違っていた:
/// (a) issue #89（`text_group_capture_value.rs`）: `( TgP TgP ) @ga TgLeaf @ga` の
/// plain / `?`（`gao`）/ `*`（`gas`）variant。写像なし text 規則だけの group を、
/// node 型の field（`TgLeaf`）と同じ名前で capture する。
/// (b) issue #86（`text_rule_value_region.rs`）: `( TgL? TgP+ | 'c' 'd' )+ @gb`。
/// 意味値の領域の中の text だけの規則（`TgP+`）は区切り。
/// (c) issue #78 / D-078（`mixed_choice_text_value.rs`）: `( ',' @gc ( TgLeaf+ @gd1 | '=' @gd2 )? )* @gc`。
/// 省略可能な mixed choice の text 候補は字句 1 つ。
/// v1 帯・v2 帯とも 3 seed。現 master ではどの seed も 1 件も差分を出してはならない。
#[test]
fn h_text_group_captures_agree_across_backends() {
    ubnfc_fuzz::tune_generated_builds();
    let base = ubnfc_fuzz::gen::TEXT_GROUP_SEED_BASE;
    let v2 = base + ubnfc_fuzz::gen::V2_SEED_BASE;
    let mut accepted = 0usize;
    for seed in [base + 1, base + 2, base + 3, v2 + 1, v2 + 2, v2 + 3] {
        let grammar = ubnfc_fuzz::gen::generate(seed);
        assert!(
            grammar.features.contains("textGroup"),
            "seed {seed}: {:?}",
            grammar.features
        );
        let scratch = scratch();
        // 48 入力: pre-fix の 3 コミット（7dbcdec / 4c1c543 / cefdbd7）へ
        // `UBNFC_RUST_GENERATOR` を差し替えて確かめたとき、seed 2000002（`base + 2`）の
        // D-078 の形（`gc`）は 12 入力では踏めず 48 入力で踏めた
        // （`docs/reports/2026-09-25-fuzz-93.md`）。
        let summary = campaign::run(&Config {
            start_seed: seed,
            seeds: 1,
            inputs: 48,
            jobs: 1,
            budget: Some(Duration::from_secs(600)),
            findings: None,
            shrink_builds: 0,
            driver_timeout: Duration::from_secs(120),
            timeout_retries: 2,
            workdir: scratch.0.clone(),
        });
        assert_eq!(
            summary.discarded, 0,
            "front の discard: {:?}",
            summary.discards
        );
        assert_eq!(summary.compiled, 1, "seed {seed} が build できていない");
        assert!(
            summary.comparisons >= 10 && summary.driver_errors == 0,
            "seed {seed}: 比較 {} / error {}",
            summary.comparisons,
            summary.driver_errors
        );
        accepted += summary.accepted_all;
        let unknown: Vec<_> = summary
            .findings
            .iter()
            .filter(|finding| !KNOWN.contains(&finding.signature.as_str()))
            .map(|finding| format!("{} ({})", finding.signature, finding.slug))
            .collect();
        assert!(
            unknown.is_empty(),
            "seed {seed} の未登録の差分: {unknown:#?}"
        );
    }
    // 全入力受理が無ければ `TextGroup` の AST を比較できていない。
    assert!(accepted > 0, "text group の帯で受理が 0 件");
}
