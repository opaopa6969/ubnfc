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
    let unknown: Vec<_> = summary
        .findings
        .iter()
        .filter(|finding| !KNOWN.contains(&finding.signature.as_str()))
        .map(|finding| format!("{} ({})", finding.signature, finding.slug))
        .collect();
    assert!(unknown.is_empty(), "未登録の差分: {unknown:#?}");
}

/// UBNF v2（`@ubnf: 2` の左再帰ルール、D-038）を両 backend で突き合わせる。
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
    let unknown: Vec<_> = summary
        .findings
        .iter()
        .filter(|finding| !KNOWN.contains(&finding.signature.as_str()))
        .map(|finding| format!("{} ({})", finding.signature, finding.slug))
        .collect();
    assert!(unknown.is_empty(), "未登録の差分: {unknown:#?}");
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
