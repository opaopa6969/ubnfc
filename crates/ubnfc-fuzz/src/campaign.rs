//! campaign の進行。seed ごとに 文法 → 入力 → 両バックエンド実行 → 比較 を行い、
//! 差分を見つけたら入力・文法の順に縮小して再現最小例を書き出す。

use crate::engine::{is_driver_timeout, Built, Discard, Engine};
use crate::model::Grammar;
use crate::observe::{differences, invariants, Violation};
use crate::{gen, input, model};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ubnfc_harness::canonical::Difference;
use ubnfc_harness::runner::Backend;

#[derive(Clone, Debug)]
pub struct Config {
    pub start_seed: u64,
    pub seeds: usize,
    pub inputs: usize,
    pub jobs: usize,
    pub budget: Option<Duration>,
    /// 最小再現例の出力先。None なら書き出さない。
    pub findings: Option<PathBuf>,
    /// 1 件の縮小で許す build 回数の上限（javac / cargo が支配的なので必ず要る）。
    pub shrink_builds: usize,
    /// 無負荷時の基準タイムアウト。実際に driver へ適用されるのは
    /// `crate::load::scale` で現在のホスト負荷を掛けた値（issue #73）。
    pub driver_timeout: Duration,
    /// `driver_timeout`（負荷率を掛けた後）で打ち切られた比較を、該当 backend だけ
    /// 再実行する回数の上限。0 回目の実行を含まない（=既定 2 なら最大 3 回試す）。
    /// 打ち切りは「失敗」ではなく「未観測」なので、使い切っても差分検出には使わない。
    pub timeout_retries: usize,
    pub workdir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            start_seed: 1,
            seeds: 8,
            inputs: 24,
            jobs: 2,
            budget: None,
            findings: None,
            shrink_builds: 30,
            driver_timeout: Duration::from_secs(90),
            timeout_retries: 2,
            workdir: PathBuf::from("."),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    pub slug: String,
    pub signature: String,
    pub seed: u64,
    pub grammar: String,
    pub imported: Option<String>,
    pub input: String,
    pub input_code_points: usize,
    pub differences: Vec<Difference>,
    pub java_violations: Vec<Violation>,
    pub rust_violations: Vec<Violation>,
    pub java: Value,
    pub rust: Value,
    pub shrink_rounds: usize,
}

/// D-072 の in-backend A/B で見つけた差。`on` / `off` は同じ backend を
/// `options.scan` だけ変えて走らせた観測。
#[derive(Clone, Debug, Serialize)]
pub struct ScanFinding {
    /// `"rust"`（D-072）か `"java"`（D-080）。
    pub backend: &'static str,
    pub seed: u64,
    pub grammar: String,
    pub input: String,
    pub differences: Vec<Difference>,
    pub on: Value,
    pub off: Value,
}

/// discard ごとの再現情報。一時ディレクトリが消えても本文を辿れるよう値で保持する。
#[derive(Clone, Debug, Serialize)]
pub struct DiscardDetail {
    pub seed: u64,
    pub grammar_name: String,
    pub grammar_digest: String,
    pub backend: Option<String>,
    pub code: String,
    pub message: String,
    pub grammar: String,
    pub imported: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Summary {
    pub seeds: usize,
    pub generated: usize,
    pub compiled: usize,
    pub discarded: usize,
    /// discard 理由コード → 件数。
    pub discards: BTreeMap<String, usize>,
    /// seed 順の詳細。理由コードの集計・受け入れ判定とは独立して保存する。
    pub discard_details: Vec<DiscardDetail>,
    /// 機能 → その機能を含む生成文法の数 / うち discard された数。
    pub feature_grammars: BTreeMap<String, usize>,
    pub feature_discarded: BTreeMap<String, usize>,
    pub inputs: usize,
    pub comparisons: usize,
    pub accepted_all: usize,
    pub rejected: usize,
    pub partial: usize,
    /// 本物の driver エラー（timeout 以外: 異常終了・不正 JSON・未知 id など）の比較数。
    pub driver_errors: usize,
    pub builds: usize,
    /// `exchange` が壁時計タイムアウトで打ち切った生の回数（再試行の分も含む）。
    pub driver_timeouts: usize,
    /// 再試行を使い切っても driver_timeout のままだった比較数。
    /// 「失敗」ではなく「未観測」として集計し、`findings` / `driver_errors` に混ぜない（issue #73）。
    pub unobserved_timeouts: usize,
    /// 1 回以上再試行した比較の数（元は driver_timeout だったが再試行対象になったもの）。
    pub retried_comparisons: usize,
    /// 実際に発行した再試行バッチの回数（1 回で複数比較をまとめて再試行することがある）。
    pub retry_attempts: usize,
    /// campaign 中に観測した負荷率の最大値（1.0 = 無負荷、上限 4.0）。report の説明用。
    pub max_load_factor: f64,
    pub violation_counts: BTreeMap<String, usize>,
    pub findings: Vec<Finding>,
    /// D-072: 一括走査 on / off を同じ backend で比べた回数（Rust と Java の合計）と、その差。
    pub scan_comparisons: usize,
    /// そのうち Java backend（D-080）で比べた回数。
    pub java_scan_comparisons: usize,
    pub scan_findings: Vec<ScanFinding>,
    /// Rust の LL(1) 表駆動 on/off 比較。
    pub ll1_comparisons: usize,
    pub ll1_findings: Vec<ScanFinding>,
    pub elapsed_seconds: f64,
}

/// 1 回の比較で観測した「差の種類」。所見はこの単位で 1 件にし、同じ入力に
/// 複数の種類が出たらそれぞれ別の最小再現例へ縮小する。
fn classes(differences: &[Difference], java: &[Violation], rust: &[Violation]) -> BTreeSet<String> {
    let mut parts = BTreeSet::new();
    for difference in differences {
        parts.insert(format!("diff:{}", normalize_path(&difference.path)));
    }
    for violation in java {
        parts.insert(format!("java:{}", violation.id));
    }
    for violation in rust {
        parts.insert(format!("rust:{}", violation.id));
    }
    parts
}

/// 配列添字を伏せ、同じ種類の差分を 1 つの signature にまとめる。
fn normalize_path(path: &str) -> String {
    path.split('/')
        .map(|part| {
            if !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()) {
                "*"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// 差分クラスごとに一意なディレクトリ名。長い pointer は切り詰めるので、
/// 切り詰めで衝突しないように signature の hash を必ず付ける。
fn slug_of(signature: &str, seed: u64) -> String {
    let mut slug: String = signature
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    let slug = slug.trim_matches('-').to_lowercase();
    let short: String = slug.chars().take(52).collect();
    let hash = signature
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("{}-{:08x}-seed{seed}", short.trim_matches('-'), hash as u32)
}

struct Observed {
    differences: Vec<Difference>,
    java_violations: Vec<Violation>,
    rust_violations: Vec<Violation>,
}

fn observe(text: &str, java: &Value, rust: &Value) -> Observed {
    Observed {
        differences: differences(java, rust),
        java_violations: invariants(text, java),
        rust_violations: invariants(text, rust),
    }
}

fn run_pair(engine: &mut Engine, built: &Built, inputs: &[String]) -> Vec<(Value, Value)> {
    let java = engine.run(built, Backend::Java, inputs);
    let rust = engine.run(built, Backend::Rust, inputs);
    java.into_iter().zip(rust).collect()
}

#[derive(Default)]
struct RetryStats {
    /// 1 回以上再試行した比較の数（初回で driver_timeout になったもの）。
    retried_comparisons: usize,
    /// 実際に発行した再試行バッチの回数。
    attempts: usize,
    /// 再試行を使い切っても driver_timeout のままだった比較の数。
    still_timeout: usize,
}

fn timeout_indices(rows: &[Value]) -> Vec<usize> {
    rows.iter()
        .enumerate()
        .filter(|(_, value)| is_driver_timeout(value))
        .map(|(index, _)| index)
        .collect()
}

/// `run_pair` のうち driver_timeout になった比較だけを、該当 backend で
/// 最大 `max_retries` 回再試行する（issue #73）。一度も timeout が起きなければ
/// 通常の `run_pair` と同じ 1 回きりの実行のまま返る。
///
/// 再試行は「その比較を発行したのと同じ backend・同じ入力」だけを新しいプロセスで
/// 再実行する。timeout はプロセス丸ごとの停止から来るので、同じバッチに含まれていた
/// 他の入力を巻き込まずに切り出せる。
fn run_pair_with_retries(
    engine: &mut Engine,
    built: &Built,
    texts: &[String],
    max_retries: usize,
) -> (Vec<(Value, Value)>, RetryStats) {
    let mut java = engine.run(built, Backend::Java, texts);
    let mut rust = engine.run(built, Backend::Rust, texts);
    let mut stats = RetryStats::default();
    let initial: BTreeSet<usize> = timeout_indices(&java)
        .into_iter()
        .chain(timeout_indices(&rust))
        .collect();
    if initial.is_empty() || max_retries == 0 {
        stats.still_timeout = initial.len();
        return (java.into_iter().zip(rust).collect(), stats);
    }
    stats.retried_comparisons = initial.len();
    for _ in 0..max_retries {
        let java_idx = timeout_indices(&java);
        let rust_idx = timeout_indices(&rust);
        if java_idx.is_empty() && rust_idx.is_empty() {
            break;
        }
        stats.attempts += 1;
        if !java_idx.is_empty() {
            let subset: Vec<String> = java_idx.iter().map(|&i| texts[i].clone()).collect();
            let redone = engine.run(built, Backend::Java, &subset);
            for (slot, value) in java_idx.into_iter().zip(redone) {
                java[slot] = value;
            }
        }
        if !rust_idx.is_empty() {
            let subset: Vec<String> = rust_idx.iter().map(|&i| texts[i].clone()).collect();
            let redone = engine.run(built, Backend::Rust, &subset);
            for (slot, value) in rust_idx.into_iter().zip(redone) {
                rust[slot] = value;
            }
        }
    }
    stats.still_timeout = timeout_indices(&java)
        .into_iter()
        .chain(timeout_indices(&rust))
        .collect::<BTreeSet<_>>()
        .len();
    (java.into_iter().zip(rust).collect(), stats)
}

/// 1 つの (文法, 入力) の差分が保たれる範囲で、入力 → 文法 → 入力 の順に縮小する。
fn shrink(
    engine: &mut Engine,
    grammar: &Grammar,
    text: &str,
    target: &str,
    budget: usize,
) -> (Grammar, String, usize) {
    let mut grammar = grammar.clone();
    let mut text = text.to_owned();
    let mut rounds = 0;
    let mut builds = 0;
    // 予算 0 は「縮小しない」。CI 用の短い campaign で driver 実行が膨らむのを防ぐ。
    if budget == 0 {
        return (grammar, text, 0);
    }
    // 再現確認 1 回につき driver を 2 回起動する（JVM 起動が支配的）。
    // build 予算に比例した上限を置き、1 件の縮小が campaign を食い潰さないようにする。
    let mut checks = 0usize;
    let max_checks = budget * 8;
    let mut reproduces = |engine: &mut Engine, grammar: &Grammar, text: &str| -> bool {
        if checks >= max_checks {
            return false;
        }
        checks += 1;
        let Ok(built) = engine.prepare(grammar) else {
            return false;
        };
        let pair = run_pair(engine, &built, std::slice::from_ref(&text.to_owned()));
        let Some((java, rust)) = pair.into_iter().next() else {
            return false;
        };
        let observed = observe(text, &java, &rust);
        classes(
            &observed.differences,
            &observed.java_violations,
            &observed.rust_violations,
        )
        .contains(target)
    };
    for _ in 0..2 {
        // 入力の縮小は build を伴わないので先に回す。
        while let Some(smaller) = input::shrink_candidates(&text)
            .into_iter()
            .find(|candidate| reproduces(engine, &grammar, candidate))
        {
            text = smaller;
            rounds += 1;
        }
        // 文法の縮小は 1 候補ごとに build が要る。上限で必ず止める。
        loop {
            let mut improved = false;
            for candidate in model::shrink_candidates(&grammar) {
                if builds >= budget {
                    return (grammar, text, rounds);
                }
                builds += 1;
                if reproduces(engine, &candidate, &text) {
                    grammar = candidate;
                    rounds += 1;
                    improved = true;
                    break;
                }
            }
            if !improved {
                break;
            }
        }
    }
    (grammar, text, rounds)
}

/// front が拒否した文法を、同じ理由コードのまま最小化する。build を伴わないので安い。
/// 所見の最小再現例が「runtime の差」ではなく「front が受け付けない形」のときに使う。
pub fn minimize_front(engine: &mut Engine, grammar: &Grammar, code: &str) -> Grammar {
    let mut current = grammar.clone();
    loop {
        let Some(smaller) = model::shrink_candidates(&current)
            .into_iter()
            .find(|candidate| {
                engine
                    .compile_front(candidate)
                    .err()
                    .is_some_and(|discard| discard.code == code)
            })
        else {
            return current;
        };
        current = smaller;
    }
}

struct Shared {
    next: AtomicUsize,
    seen: Mutex<BTreeSet<String>>,
    summary: Mutex<Summary>,
    start: Instant,
}

/// campaign 本体。`config.jobs` 個の worker が seed を取り合う。
pub fn run(config: &Config) -> Summary {
    let shared = Arc::new(Shared {
        next: AtomicUsize::new(0),
        seen: Mutex::new(BTreeSet::new()),
        summary: Mutex::new(Summary {
            seeds: config.seeds,
            ..Summary::default()
        }),
        start: Instant::now(),
    });
    std::thread::scope(|scope| {
        for worker in 0..config.jobs.max(1) {
            let shared = shared.clone();
            let config = config.clone();
            scope.spawn(move || {
                let mut engine = Engine::new(config.workdir.join(format!("w{worker}")));
                engine.timeout = config.driver_timeout;
                loop {
                    let index = shared.next.fetch_add(1, Ordering::Relaxed);
                    if index >= config.seeds {
                        break;
                    }
                    if config
                        .budget
                        .is_some_and(|budget| shared.start.elapsed() >= budget)
                    {
                        break;
                    }
                    let seed = config.start_seed.wrapping_add(index as u64);
                    run_seed(&mut engine, &config, &shared, seed);
                }
                let mut summary = shared.summary.lock().expect("summary");
                summary.builds += engine.builds;
                summary.driver_timeouts += engine.timeouts;
            });
        }
    });
    let mut summary = shared.summary.lock().expect("summary").clone();
    summary.elapsed_seconds = shared.start.elapsed().as_secs_f64();
    summary.findings.sort_by(|a, b| a.slug.cmp(&b.slug));
    summary.discard_details.sort_by_key(|detail| detail.seed);
    summary
}

fn record_discard(shared: &Shared, seed: u64, grammar: &Grammar, discard: &Discard) {
    let detail = DiscardDetail {
        seed,
        grammar_name: grammar.name.clone(),
        grammar_digest: grammar.digest(),
        backend: discard.code.strip_prefix("build:").map(str::to_owned),
        code: discard.code.clone(),
        message: discard.message.clone(),
        grammar: grammar.render(),
        imported: grammar.imported_source().map(|(_, source)| source),
    };
    if detail.backend.is_some() {
        // 1 行の JSON にして worker 間の混在を防ぐ。cargo test の失敗時にも出力される。
        eprintln!(
            "ビルド失敗の詳細: {}",
            serde_json::to_string(&detail).expect("discard json")
        );
    }
    let mut summary = shared.summary.lock().expect("summary");
    summary.discard_details.push(detail);
    summary.discarded += 1;
    *summary.discards.entry(discard.code.clone()).or_default() += 1;
    for feature in &grammar.features {
        *summary
            .feature_discarded
            .entry((*feature).to_owned())
            .or_default() += 1;
    }
}

fn run_seed(engine: &mut Engine, config: &Config, shared: &Shared, seed: u64) {
    let grammar = gen::generate(seed);
    {
        let mut summary = shared.summary.lock().expect("summary");
        summary.generated += 1;
        for feature in &grammar.features {
            *summary
                .feature_grammars
                .entry((*feature).to_owned())
                .or_default() += 1;
        }
    }
    let built = match engine.prepare(&grammar) {
        Ok(built) => built,
        Err(discard) => return record_discard(shared, seed, &grammar, &discard),
    };
    shared.summary.lock().expect("summary").compiled += 1;
    let mut rng = crate::rng::Rng::new(seed ^ 0xF00D_BEEF);
    let inputs = input::generate(&grammar, &mut rng, config.inputs);
    let texts: Vec<String> = inputs.iter().map(|i| i.text.clone()).collect();
    let (pairs, retry_stats) =
        run_pair_with_retries(engine, &built, &texts, config.timeout_retries);
    {
        let mut summary = shared.summary.lock().expect("summary");
        summary.retried_comparisons += retry_stats.retried_comparisons;
        summary.retry_attempts += retry_stats.attempts;
        summary.unobserved_timeouts += retry_stats.still_timeout;
        if engine.max_load_factor > summary.max_load_factor {
            summary.max_load_factor = engine.max_load_factor;
        }
    }
    // D-072 / D-080: 同じ backend を `options.scan` だけ変えて 2 度走らせる（Rust と Java の両方）。
    // 走査は fast mode でだけ効くので、driver はこのキーがあるとき診断・出現表を外す。
    // 失敗した解析は D-023 の再解析で参照経路の観測へ戻るので、ここで比べているのは
    // 「fast session が成功したまま返した観測」——走査が変え得る全部である。
    for (backend, label) in [(Backend::Rust, "rust"), (Backend::Java, "java")] {
        let scan_on = engine.run_scan(&built, backend, &texts, true);
        let scan_off = engine.run_scan(&built, backend, &texts, false);
        let mut summary = shared.summary.lock().expect("summary");
        summary.scan_comparisons += texts.len();
        if label == "java" {
            summary.java_scan_comparisons += texts.len();
        }
        for ((text, on), off) in texts.iter().zip(scan_on).zip(scan_off) {
            let difference = differences(&on, &off);
            if !difference.is_empty() {
                summary.scan_findings.push(ScanFinding {
                    backend: label,
                    seed,
                    grammar: grammar.render(),
                    input: text.clone(),
                    differences: difference,
                    on,
                    off,
                });
            }
        }
    }
    let ll1_on = engine.run_ll1(&built, &texts, true);
    let ll1_off = engine.run_ll1(&built, &texts, false);
    {
        let mut summary = shared.summary.lock().expect("summary");
        summary.ll1_comparisons += texts.len();
        for ((text, on), off) in texts.iter().zip(ll1_on).zip(ll1_off) {
            let difference = differences(&on, &off);
            if !difference.is_empty() {
                summary.ll1_findings.push(ScanFinding {
                    backend: "rust",
                    seed,
                    grammar: grammar.render(),
                    input: text.clone(),
                    differences: difference,
                    on,
                    off,
                });
            }
        }
    }
    let mut new_findings = Vec::new();
    for (text, (java, rust)) in texts.iter().zip(pairs) {
        // 再試行を使い切ってもなお driver_timeout の比較は「未観測」であって、
        // 片側だけの応答欠落を差分として登録しない（issue #73）。本物のエラー
        // （異常終了・不正 JSON 等）はこれまで通り差分・driver_errors の対象にする。
        let java_timeout = is_driver_timeout(&java);
        let rust_timeout = is_driver_timeout(&rust);
        let java_hard_error = java.get("error").is_some() && !java_timeout;
        let rust_hard_error = rust.get("error").is_some() && !rust_timeout;
        let unobserved = (java_timeout || rust_timeout) && !java_hard_error && !rust_hard_error;
        {
            let mut summary = shared.summary.lock().expect("summary");
            summary.inputs += 1;
            summary.comparisons += 1;
            if unobserved {
                // すでに retry_stats.still_timeout で数えている。ここでは
                // accepted/partial/rejected の分類にも driver_errors にも入れない。
            } else if java_hard_error || rust_hard_error {
                summary.driver_errors += 1;
            } else if java["allConsumed"] == json!(true) {
                summary.accepted_all += 1;
            } else if java["accepted"] == json!(true) {
                summary.partial += 1;
            } else {
                summary.rejected += 1;
            }
        }
        if unobserved {
            continue;
        }
        let observed = observe(text, &java, &rust);
        {
            let mut summary = shared.summary.lock().expect("summary");
            for violation in observed
                .java_violations
                .iter()
                .map(|v| format!("java:{}", v.id))
                .chain(
                    observed
                        .rust_violations
                        .iter()
                        .map(|v| format!("rust:{}", v.id)),
                )
            {
                *summary.violation_counts.entry(violation).or_default() += 1;
            }
        }
        for class in classes(
            &observed.differences,
            &observed.java_violations,
            &observed.rust_violations,
        ) {
            if shared.seen.lock().expect("seen").insert(class.clone()) {
                new_findings.push((class, text.clone()));
            }
        }
    }
    for (signature, text) in new_findings {
        let (small_grammar, small_input, rounds) =
            shrink(engine, &grammar, &text, &signature, config.shrink_builds);
        let Ok(built) = engine.prepare(&small_grammar) else {
            continue;
        };
        let pair = run_pair(engine, &built, std::slice::from_ref(&small_input));
        let Some((java, rust)) = pair.into_iter().next() else {
            continue;
        };
        let observed = observe(&small_input, &java, &rust);
        let finding = Finding {
            slug: slug_of(&signature, seed),
            signature,
            seed,
            grammar: small_grammar.render(),
            imported: small_grammar.imported_source().map(|(_, source)| source),
            input: small_input.clone(),
            input_code_points: small_input.chars().count(),
            differences: observed.differences,
            java_violations: observed.java_violations,
            rust_violations: observed.rust_violations,
            java,
            rust,
            shrink_rounds: rounds,
        };
        if let Some(directory) = &config.findings {
            let _ = write_finding(directory, &finding);
        }
        shared
            .summary
            .lock()
            .expect("summary")
            .findings
            .push(finding);
    }
}

/// 最小再現例を 1 ディレクトリに固める。入力は生ファイルと JSON の両方で残す
/// （NUL や非 BMP をエディタ経由で壊さないため）。
pub fn write_finding(directory: &std::path::Path, finding: &Finding) -> std::io::Result<()> {
    let target = directory.join(&finding.slug);
    std::fs::create_dir_all(&target)?;
    std::fs::write(target.join("grammar.ubnf"), &finding.grammar)?;
    if let Some(imported) = &finding.imported {
        std::fs::write(target.join("imported.ubnf"), imported)?;
    }
    std::fs::write(target.join("input.txt"), &finding.input)?;
    std::fs::write(
        target.join("finding.json"),
        serde_json::to_string_pretty(finding).expect("finding json") + "\n",
    )?;
    let mut readme = format!(
        "# {}\n\nseed: {} / 入力 {} code point / 縮小 {} 回\n\n## signature\n\n`{}`\n\n## 入力（JSON escape）\n\n```json\n{}\n```\n\n",
        finding.slug,
        finding.seed,
        finding.input_code_points,
        finding.shrink_rounds,
        finding.signature,
        json!(finding.input)
    );
    let _ = writeln!(readme, "## 文法\n\n```ubnf\n{}```\n", finding.grammar);
    if !finding.differences.is_empty() {
        let _ = writeln!(readme, "## 観測差（左 Java / 右 Rust）\n");
        for difference in &finding.differences {
            let _ = writeln!(
                readme,
                "- `{}`: java=`{}` rust=`{}`",
                difference.path,
                difference
                    .expected
                    .as_ref()
                    .map_or("<欠落>".into(), Value::to_string),
                difference
                    .actual
                    .as_ref()
                    .map_or("<欠落>".into(), Value::to_string)
            );
        }
        readme.push('\n');
    }
    for (label, violations) in [
        ("Java", &finding.java_violations),
        ("Rust", &finding.rust_violations),
    ] {
        if violations.is_empty() {
            continue;
        }
        let _ = writeln!(readme, "## {label} の不変条件違反\n");
        for violation in violations {
            let _ = writeln!(readme, "- `{}`: {}", violation.id, violation.detail);
        }
        readme.push('\n');
    }
    let _ = writeln!(
        readme,
        "## 応答\n\n```json\njava = {}\nrust = {}\n```",
        finding.java, finding.rust
    );
    std::fs::write(target.join("README.md"), readme)?;
    Ok(())
}

/// campaign の Markdown 要約。
pub fn markdown(summary: &Summary) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## campaign の規模\n");
    let _ = writeln!(out, "| 項目 | 値 |\n|---|---:|");
    for (label, value) in [
        ("seed 数", summary.seeds),
        ("生成した文法", summary.generated),
        ("IR/生成/build を通った文法", summary.compiled),
        ("discard された文法", summary.discarded),
        ("build 回数（縮小を含む）", summary.builds),
        ("入力（= 比較）件数", summary.comparisons),
        ("全入力受理", summary.accepted_all),
        ("prefix のみ受理（部分消費）", summary.partial),
        ("不受理", summary.rejected),
        ("driver error（timeout 以外）", summary.driver_errors),
        ("driver の制限時間超過（延べ回数）", summary.driver_timeouts),
        ("再試行した比較", summary.retried_comparisons),
        ("再試行バッチの発行回数", summary.retry_attempts),
        (
            "再試行後も未観測（driver_timeout）",
            summary.unobserved_timeouts,
        ),
        ("最小化済みの所見", summary.findings.len()),
        ("LL(1) on/off 比較", summary.ll1_comparisons),
        ("LL(1) on/off 差分", summary.ll1_findings.len()),
    ] {
        let _ = writeln!(out, "| {label} | {value} |");
    }
    let _ = writeln!(
        out,
        "| 観測した負荷率の最大（1.0=無負荷、上限4.0） | {:.2} |",
        summary.max_load_factor
    );
    let _ = writeln!(out, "| 所要 | {:.1} 秒 |", summary.elapsed_seconds);
    let _ = writeln!(out, "\n## 機能別の被覆と discard 率\n");
    let _ = writeln!(
        out,
        "| 機能 | 生成文法数 | うち discard | discard 率 |\n|---|---:|---:|---:|"
    );
    for feature in gen::FEATURES {
        let total = summary.feature_grammars.get(*feature).copied().unwrap_or(0);
        let discarded = summary
            .feature_discarded
            .get(*feature)
            .copied()
            .unwrap_or(0);
        let rate = if total == 0 {
            "-".to_string()
        } else {
            format!("{:.0}%", 100.0 * discarded as f64 / total as f64)
        };
        let _ = writeln!(out, "| `{feature}` | {total} | {discarded} | {rate} |");
    }
    if !summary.discards.is_empty() {
        let _ = writeln!(out, "\n## discard の理由\n");
        let _ = writeln!(out, "| コード | 件数 |\n|---|---:|");
        let mut rows: Vec<_> = summary.discards.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (code, count) in rows {
            let _ = writeln!(out, "| `{code}` | {count} |");
        }
    }
    if !summary.discard_details.is_empty() {
        let _ = writeln!(out, "\n## discard の詳細\n");
        for detail in &summary.discard_details {
            let _ = writeln!(
                out,
                "### seed {} / `{}` / `{}`\n\n- 文法 digest: `{}`\n- backend: `{}`\n\nエラー本文（省略なし）:\n",
                detail.seed,
                detail.grammar_name,
                detail.code,
                detail.grammar_digest,
                detail.backend.as_deref().unwrap_or("該当なし")
            );
            // 本文中の fence によって Markdown が壊れないよう字下げコードブロックにする。
            for line in detail.message.split('\n') {
                let _ = writeln!(out, "    {line}");
            }
            let _ = writeln!(out);
        }
    }
    if !summary.violation_counts.is_empty() {
        let _ = writeln!(out, "\n## 不変条件違反（観測件数）\n");
        let _ = writeln!(out, "| 条件 | 件数 |\n|---|---:|");
        let mut rows: Vec<_> = summary.violation_counts.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (id, count) in rows {
            let _ = writeln!(out, "| `{id}` | {count} |");
        }
    }
    let _ = writeln!(out, "\n## 所見\n");
    if summary.findings.is_empty() {
        let _ = writeln!(out, "差分・不変条件違反はいずれも観測されなかった。");
    }
    for finding in &summary.findings {
        let _ = writeln!(
            out,
            "### `{}`\n\n- signature: `{}`\n- seed: {} / 入力 {} code point\n- 最小再現: `crates/ubnfc-fuzz/findings/{}/`\n",
            finding.slug, finding.signature, finding.seed, finding.input_code_points, finding.slug
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difference_classes_ignore_array_indexes_and_slugs_stay_filesystem_safe() {
        let difference = |path: &str| Difference {
            path: path.into(),
            expected: None,
            actual: None,
        };
        let a = classes(&[difference("/captures/0/span/1")], &[], &[]);
        assert_eq!(a, classes(&[difference("/captures/3/span/1")], &[], &[]));
        assert_eq!(a.len(), 1);
        let slug = slug_of(a.iter().next().unwrap(), 12);
        assert!(
            slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "{slug}"
        );
        assert!(slug.ends_with("-seed12"));
        // 切り詰めても別クラスは別ディレクトリになる。
        let long = |tail: &str| format!("diff:/ast/fields/right/0/fields/right/0/fields/{tail}");
        assert_ne!(slug_of(&long("left"), 1), slug_of(&long("right"), 1));
        assert!(classes(&[], &[], &[]).is_empty());
        let violation = Violation {
            id: "cursor-order",
            detail: "x".into(),
        };
        // 1 回の比較に複数種類が出たら、種類ごとに別の所見へ分ける。
        let mixed = classes(
            &[difference("/consumed")],
            std::slice::from_ref(&violation),
            &[],
        );
        assert_eq!(
            mixed.into_iter().collect::<Vec<_>>(),
            ["diff:/consumed", "java:cursor-order"]
        );
    }

    #[test]
    fn markdown_lists_every_feature_even_when_a_campaign_never_reached_it() {
        let text = markdown(&Summary::default());
        for feature in gen::FEATURES {
            assert!(text.contains(&format!("`{feature}`")), "{feature}");
        }
        assert!(text.contains("差分・不変条件違反はいずれも観測されなかった。"));
    }

    // --- issue #73: driver_timeout の再試行と未観測分類。
    //
    // 実ホスト負荷を必要としない: `Built` は前段の front/build を経ずに直接組み立てられる
    // （`run_with` は `built.java` / `built.rust` の `Executable` を呼ぶだけ）。driver の
    // 代わりに shell script を挟み、「止まったままの driver」を確定的に再現する。

    use std::ffi::OsString;
    use ubnfc_harness::runner::Executable;

    fn shell(script: &str) -> Executable {
        Executable {
            program: PathBuf::from("sh"),
            args: vec![OsString::from("-c"), OsString::from(script)],
            // 空なら全ケースを program に直接 dispatch する（`Executable::grammars` の既定）。
            grammars: BTreeMap::new(),
        }
    }

    /// 応答を要求どおりの `id` で返し続ける driver。常に成功する。
    fn always_ok() -> Executable {
        shell(
            r#"while IFS= read -r line; do
  id=$(printf '%s' "$line" | grep -o '"id":[0-9]*' | head -n1 | cut -d: -f2)
  printf '{"id":%s,"accepted":true,"allConsumed":true,"consumed":0,"matched":0,"ast":null,"captures":[],"declarations":[],"references":[],"diagnostics":[],"mappingError":null}\n' "$id"
done"#,
        )
    }

    /// stdin を一切読まず眠り続ける driver。`Engine::timeout` を超えたら必ず kill される。
    ///
    /// `exec` で `sleep` に置き換わる（同じ pid のまま）ことが重要: `sh -c "sleep 999"`
    /// のように `sleep` を**子プロセスとして fork** すると、`wait_or_kill` が kill するのは
    /// 直接の子（`sh`）だけなので、孫の `sleep` が親を失っても生き残り、その pipe
    /// (stdout) を掴んだままにして `drain` スレッドが EOF を待ち続けてテストが
    /// 無期限に固まる（このテストを書く過程で実際にハングして踏んだ）。
    fn always_hang() -> Executable {
        shell("exec sleep 999")
    }

    /// 1 回目の起動（`marker` がまだ無い）は `always_hang` と同じで、`marker` を作って
    /// 眠る。2 回目以降（再試行）は `marker` があるので `always_ok` と同じに振る舞う。
    /// 「最初の壁時計タイムアウトを、再試行が新しいプロセスで解消する」を模す。
    fn hang_once_then_ok(marker: &std::path::Path) -> Executable {
        let marker = marker.display();
        shell(&format!(
            r#"if [ ! -e '{marker}' ]; then
  : > '{marker}'
  exec sleep 999
else
  while IFS= read -r line; do
    id=$(printf '%s' "$line" | grep -o '"id":[0-9]*' | head -n1 | cut -d: -f2)
    printf '{{"id":%s,"accepted":true,"allConsumed":true,"consumed":0,"matched":0,"ast":null,"captures":[],"declarations":[],"references":[],"diagnostics":[],"mappingError":null}}\n' "$id"
  done
fi"#
        ))
    }

    struct TestScratch(PathBuf);
    impl TestScratch {
        fn new(label: &str) -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos();
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "ubnfc-fuzz-timeout-test-{label}-{}-{n}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("scratch dir");
            Self(path)
        }
    }
    impl Drop for TestScratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn stub_built(java: Executable, rust: Executable) -> Built {
        Built {
            grammar_name: "Stub".into(),
            java,
            rust,
        }
    }

    #[test]
    fn a_timed_out_comparison_is_retried_and_stops_being_unobserved_once_it_recovers() {
        let scratch = TestScratch::new("recovers");
        let mut engine = Engine::new(scratch.0.join("engine"));
        // 基準タイムアウトを短くして「負荷」を待たずに再現する。負荷率は 1〜4 倍の
        // どちらでもテストの成否には影響しない（数百 ms 単位の余裕で足りる）。
        engine.timeout = Duration::from_millis(150);
        let marker = scratch.0.join("marker");
        let built = stub_built(hang_once_then_ok(&marker), always_ok());
        let texts = vec!["a".to_owned(), "b".to_owned()];

        let (pairs, stats) = run_pair_with_retries(&mut engine, &built, &texts, 2);

        assert_eq!(stats.retried_comparisons, 2, "初回は両方 timeout のはず");
        assert_eq!(stats.attempts, 1, "1 回の再試行で解消するはず");
        assert_eq!(stats.still_timeout, 0, "再試行後は timeout が残らないはず");
        for (java, rust) in &pairs {
            assert!(!is_driver_timeout(java), "再試行後なのに timeout: {java}");
            assert!(!is_driver_timeout(rust), "{rust}");
            assert_eq!(java["accepted"], json!(true));
            assert_eq!(rust["accepted"], json!(true));
        }
    }

    #[test]
    fn a_comparison_that_never_recovers_stays_unobserved_after_retries_are_exhausted() {
        let scratch = TestScratch::new("stuck");
        let mut engine = Engine::new(scratch.0.join("engine"));
        engine.timeout = Duration::from_millis(120);
        let built = stub_built(always_hang(), always_ok());
        let texts = vec!["a".to_owned()];

        let (pairs, stats) = run_pair_with_retries(&mut engine, &built, &texts, 1);

        assert_eq!(stats.retried_comparisons, 1);
        assert_eq!(stats.attempts, 1, "上限 1 回で打ち切るはず");
        assert_eq!(
            stats.still_timeout, 1,
            "解消しない timeout は再試行後も残るはず"
        );
        let (java, rust) = &pairs[0];
        assert!(is_driver_timeout(java), "{java}");
        assert!(!is_driver_timeout(rust), "もう片方は正常なはず: {rust}");
    }

    #[test]
    fn zero_retries_reports_the_timeout_without_attempting_any_retry() {
        let scratch = TestScratch::new("zero-retries");
        let mut engine = Engine::new(scratch.0.join("engine"));
        engine.timeout = Duration::from_millis(120);
        let built = stub_built(always_hang(), always_ok());
        let texts = vec!["a".to_owned()];

        let (pairs, stats) = run_pair_with_retries(&mut engine, &built, &texts, 0);

        assert_eq!(
            stats.retried_comparisons, 0,
            "max_retries=0 は再試行そのものをしない"
        );
        assert_eq!(stats.attempts, 0);
        assert_eq!(stats.still_timeout, 1);
        assert!(is_driver_timeout(&pairs[0].0));
    }

    /// campaign 本体（`run_seed`）が実際に使う分類規則: 再試行を尽くしてもなお
    /// timeout の比較は `differences` / `invariants` を経由させず、「未観測」として
    /// 素通りさせる（`driver_errors` にも `findings` にも入れない）。ここでは
    /// その判定条件そのものを、run_seed の内側に埋め込むのではなく直接検証する。
    #[test]
    fn a_lingering_timeout_is_not_treated_as_a_hard_driver_error() {
        let scratch = TestScratch::new("classify");
        let mut engine = Engine::new(scratch.0.join("engine"));
        engine.timeout = Duration::from_millis(120);
        let built = stub_built(always_hang(), always_ok());
        let texts = vec!["a".to_owned()];
        let (pairs, _) = run_pair_with_retries(&mut engine, &built, &texts, 1);
        let (java, rust) = &pairs[0];

        let java_timeout = is_driver_timeout(java);
        let rust_timeout = is_driver_timeout(rust);
        let java_hard_error = java.get("error").is_some() && !java_timeout;
        let rust_hard_error = rust.get("error").is_some() && !rust_timeout;
        let unobserved = (java_timeout || rust_timeout) && !java_hard_error && !rust_hard_error;
        assert!(unobserved, "片側だけの driver_timeout は未観測のはず");
    }
}
