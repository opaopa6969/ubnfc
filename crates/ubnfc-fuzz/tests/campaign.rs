//! CI 用の固定予算 campaign。seed を固定しているので結果は決定的で、
//! 新しい差分が生まれたときだけ赤くなる。既知の所見は signature で明示登録する。
//!
//! 前提ツールは harness の e2e テストと同じ（`javac` / `java` / `ubnfc-rust` /
//! `ubnfc-java` の jar）。無ければ skip せず失敗する。

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use ubnfc_fuzz::campaign::{self, Config};

/// `docs/reports/2026-09-23-differential-fuzzing.md` で triage 済みの所見。
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
    // B. driver の wire 形式（`docs/reports/2026-09-23-differential-fuzzing.md` §4.B）
    "diff:/catalogs",
    "diff:/recoveries",
    "diff:/scope/declarations",
    "diff:/scope/depth",
    "diff:/scope/diagnostics",
    "diff:/scope/references",
    // A-3: 成功した解析に Java だけが投機的 syntax 診断を残す
    "diff:/diagnostics/*",
    // A-6: Java の farthestExpected に合成 parser の表示名が混じる
    "diff:/diagnostics/*/farthestExpected/*",
    // A-4: 全入力解析が失敗したのに Rust が capture を返す
    "diff:/captures/*",
    // C: allConsumed は consumed からは導けない（両 backend 共通、契約の不足）
    "java:allconsumed-consistency",
    "rust:allconsumed-consistency",
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
