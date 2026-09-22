//! P4 生成 Rust parser のベンチマーク（std のみ、`std::time::Instant`）。
//!
//! ```sh
//! cargo run -q --offline --release -p ubnfc-p4-rust --example bench
//! cargo run -q --offline --release -p ubnfc-p4-rust --features counters --example bench -- --counters
//! ```
//! 各入力・各モードで warmup 後、5 run の中央値を ms/parse で出す。1 run の反復数は
//! 1 parse の所要から `UBNFC_BENCH_RUN_SECONDS`（既定 0.3 秒）を満たすよう決める（3〜5000）。
//! 確保回数・byte は GlobalAlloc の alloc/realloc を数えた別 parse で、計時には含めない。
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
use std::time::Instant;
use ubnfc_p4_rust::{parse_entry, parse_formula, ParseOptions, ParseResult};

struct Counting;
static ACTIVE: AtomicBool = AtomicBool::new(false);
static COUNT: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if ACTIVE.load(Relaxed) {
            COUNT.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size(), Relaxed);
        }
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        if ACTIVE.load(Relaxed) {
            COUNT.fetch_add(1, Relaxed);
            BYTES.fetch_add(n, Relaxed);
        }
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static ALLOC: Counting = Counting;

struct Case {
    name: &'static str,
    entry: Option<&'static str>,
}
const CASES: &[Case] = &[
    Case {
        name: "complex",
        entry: None,
    },
    Case {
        name: "complex-x4",
        entry: None,
    },
    Case {
        name: "complex-x16",
        entry: None,
    },
    Case {
        name: "complex-x64",
        entry: None,
    },
    Case {
        name: "flat-arithmetic",
        entry: None,
    },
    Case {
        name: "large-match",
        entry: None,
    },
    Case {
        name: "comparison-heavy",
        entry: Some("BooleanExpression"),
    },
    Case {
        name: "comparison-heavy-x4",
        entry: Some("BooleanExpression"),
    },
    Case {
        name: "comparison-heavy-x16",
        entry: Some("BooleanExpression"),
    },
    Case {
        name: "comparison-heavy-x64",
        entry: Some("BooleanExpression"),
    },
];

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}.tiny", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}
/// 既定の診断予算 200 万 node は comparison-heavy-x4 以上で不足する（report 参照）。
/// ベンチは受理を前提にするため予算だけ広げる。予算は事前確保ではないので計時に影響しない。
fn options(build_ast: bool) -> ParseOptions {
    let mut options = ParseOptions {
        build_ast,
        ..Default::default()
    };
    options.limits.diagnostics = 1 << 30;
    options.limits.arena_entries = 1 << 30;
    options.limits.memo_entries = 1 << 30;
    options
}
fn run(case: &Case, text: &str, build_ast: bool) -> ParseResult {
    let options = options(build_ast);
    let result = match case.entry {
        None => parse_formula(black_box(text), black_box(options)),
        Some(entry) => parse_entry(Some(entry), black_box(text), black_box(options)).unwrap(),
    };
    assert!(result.ok, "{}: {:?}", case.name, result.diagnostics);
    assert_eq!(result.ast.is_some(), build_ast);
    result
}

fn timing(cases: &[&Case]) {
    let run_seconds: f64 = std::env::var("UBNFC_BENCH_RUN_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.3);
    println!("| 入力 | bytes | mode | iterations/run | 中央値 ms/parse | min | max | µs/byte | allocs/parse | alloc bytes/parse | rule_evaluations | memo_hits | memo_entries | memo_max_probe | recipes | ast_nodes |");
    println!("|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|");
    for case in cases {
        let text = fixture(case.name);
        for build_ast in [false, true] {
            let once = Instant::now();
            black_box(run(case, &text, build_ast));
            let per = once.elapsed().as_secs_f64().max(1e-9);
            let iterations = ((run_seconds / per) as usize).clamp(3, 5000);
            let warmup = (iterations / 2).clamp(3, 500);
            for _ in 0..warmup {
                black_box(run(case, &text, build_ast));
            }
            let mut samples = [0.0f64; 5];
            for sample in &mut samples {
                let start = Instant::now();
                for _ in 0..iterations {
                    black_box(run(case, &text, build_ast));
                }
                *sample = start.elapsed().as_secs_f64() * 1e3 / iterations as f64;
            }
            samples.sort_by(f64::total_cmp);
            COUNT.store(0, Relaxed);
            BYTES.store(0, Relaxed);
            ACTIVE.store(true, Relaxed);
            let result = run(case, &text, build_ast);
            ACTIVE.store(false, Relaxed);
            let (allocs, bytes) = (COUNT.load(Relaxed), BYTES.load(Relaxed));
            let s = result.statistics;
            println!(
                "| {} | {} | {} | {} | {:.3} | {:.3} | {:.3} | {:.2} | {} | {} | {} | {} | {} | {} | {} | {} |",
                case.name,
                text.len(),
                if build_ast { "AST" } else { "認識" },
                iterations,
                samples[2],
                samples[0],
                samples[4],
                samples[2] * 1e3 / text.len() as f64,
                allocs,
                bytes,
                s.rule_evaluations,
                s.memo_hits,
                s.memo_entries,
                s.memo_max_probe,
                s.recipes,
                s.ast_nodes
            );
        }
    }
}

#[cfg(feature = "counters")]
fn counters(cases: &[&Case]) {
    use ubnfc_p4_rust::counters;
    let ir = ubnfc_front::ir::from_json(include_str!(
        "../../../ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json"
    ))
    .unwrap();
    let symbol = |id: &str| {
        ir.semantics
            .symbols
            .iter()
            .find(|s| s.id == id)
            .map_or(id.to_owned(), |s| s.name.clone())
    };
    let rule_names: Vec<String> = ir
        .semantics
        .rules
        .iter()
        .map(|r| symbol(&r.symbol_id))
        .collect();
    let expr_desc: Vec<String> = ir
        .semantics
        .expressions
        .iter()
        .map(|e| {
            let node = format!("{:?}", e.node);
            let kind = node.split([' ', '{', '(']).next().unwrap_or("?");
            format!("{} {kind}", symbol(&e.rule_id))
        })
        .collect();
    for case in cases {
        let text = fixture(case.name);
        for build_ast in [false, true] {
            let options = options(build_ast);
            // parse_entry だけが計測フック付きの複製を通る。
            let entry = case.entry.or(Some("Formula"));
            black_box(parse_entry(entry, &text, options).unwrap());
            counters::reset();
            let start = Instant::now();
            let result = parse_entry(entry, &text, options).unwrap();
            let elapsed = start.elapsed();
            assert!(result.ok);
            let snap = counters::snapshot();
            println!(
                "\n### counters: {} ({} bytes, {}), 計測込み {:.3} ms\n",
                case.name,
                text.len(),
                if build_ast { "AST" } else { "認識" },
                elapsed.as_secs_f64() * 1e3
            );
            println!("| runtime 呼出し | 回数/parse |\n|---|---:|");
            for (name, count) in counters::NAMES.iter().zip(&snap.events) {
                println!("| {name} | {count} |");
            }
            let total_rule_calls: u64 = snap
                .rules
                .iter()
                .map(|r| r.calls_consumed + r.calls_match_only)
                .sum();
            let total_expr_calls: u64 = snap
                .expressions
                .iter()
                .map(|e| e.calls_consumed + e.calls_match_only)
                .sum();
            let total_self: u64 = snap.rules.iter().map(|r| r.self_ns).sum();
            println!("\nrule 関数呼出し合計 {total_rule_calls}、expression 関数呼出し合計 {total_expr_calls}、rule 自己時間合計 {:.3} ms\n", total_self as f64 / 1e6);
            let mut rules: Vec<(usize, counters::RuleCounter)> =
                snap.rules.iter().copied().enumerate().collect();
            rules.sort_by_key(|(_, r)| std::cmp::Reverse(r.self_ns));
            println!("| rule | calls (consumed) | calls (matchOnly) | 自己時間 % | 包含時間 ms | 自己 ns/call |\n|---|---:|---:|---:|---:|---:|");
            for (id, r) in rules.iter().take(25) {
                let calls = r.calls_consumed + r.calls_match_only;
                if calls == 0 {
                    continue;
                }
                println!(
                    "| r{id} {} | {} | {} | {:.1} | {:.3} | {} |",
                    rule_names.get(*id).map_or("?", String::as_str),
                    r.calls_consumed,
                    r.calls_match_only,
                    100.0 * r.self_ns as f64 / total_self.max(1) as f64,
                    r.inclusive_ns as f64 / 1e6,
                    r.self_ns / calls
                );
            }
            let mut exprs: Vec<(usize, counters::ExpressionCounter)> =
                snap.expressions.iter().copied().enumerate().collect();
            exprs.sort_by_key(|(_, e)| std::cmp::Reverse(e.calls_consumed + e.calls_match_only));
            println!("\n| expression | 所属 rule / 種別 | calls (consumed) | calls (matchOnly) |\n|---|---|---:|---:|");
            for (id, e) in exprs.iter().take(25) {
                println!(
                    "| e{id} | {} | {} | {} |",
                    expr_desc.get(*id).map_or("?", String::as_str),
                    e.calls_consumed,
                    e.calls_match_only
                );
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let selected: Vec<&Case> = CASES
        .iter()
        .filter(|c| {
            let names: Vec<&str> = args
                .iter()
                .filter(|a| !a.starts_with("--"))
                .map(String::as_str)
                .collect();
            names.is_empty() || names.contains(&c.name)
        })
        .collect();
    if args.iter().any(|a| a == "--counters") {
        #[cfg(feature = "counters")]
        {
            counters(&selected);
            return;
        }
        #[cfg(not(feature = "counters"))]
        panic!("--features counters でビルドする");
    }
    timing(&selected);
}
