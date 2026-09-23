//! 生成 crate に注入する計時 bin。examples/p4-rust/examples/bench.rs と同じ方式
//! （warmup 後 5 run の中央値、1 run は UBNFC_BENCH_RUN_SECONDS 秒ぶんの反復）。
//! Extern token があるので scanner registry つきの `parse_with_scanner` を使う。
#[path = "scanners.rs"]
mod scanners;

use std::hint::black_box;
use std::time::Instant;
use ubnfc_generated::{parse_with_scanner, ParseOptions};

const EXTERN_CLASSES: &[(&str, &str)] = &[
    (
        "TinyExpressionP4::STRING",
        "org.unlaxer.tinyexpression.parser.StringLiteralParser",
    ),
    (
        "TinyExpressionP4::CODE_START",
        "org.unlaxer.tinyexpression.parser.javalang.CodeStartParser",
    ),
    (
        "TinyExpressionP4::CODE_END",
        "org.unlaxer.tinyexpression.parser.javalang.CodeEndParser",
    ),
];

fn options(ast: bool) -> ParseOptions {
    ParseOptions {
        build_ast: ast,
        require_eof: true,
        ..Default::default()
    }
}

fn once(input: &str, ast: bool) -> bool {
    let mut registry = scanners::Registry::new(EXTERN_CLASSES).expect("scanner registry");
    let result = parse_with_scanner(black_box(input), black_box(options(ast)), &mut registry);
    black_box(result.ok)
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values[values.len() / 2]
}

fn main() {
    let seconds: f64 = std::env::var("UBNFC_BENCH_RUN_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.3);
    println!("input,mode,median_ms_per_parse,iterations,accepted");
    for path in std::env::args().skip(1) {
        let input = std::fs::read_to_string(&path).expect("fixture");
        let name = std::path::Path::new(&path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        for ast in [false, true] {
            // comparison-heavy は root 入口では拒否される（設計 06 §2 の候補順の既知問題）。
            // 拒否入力の計時も v1/v2 で比べる意味があるので、成否を列に出すだけにする。
            let ok = once(&input, ast);
            let probe = Instant::now();
            once(&input, ast);
            let single = probe.elapsed().as_secs_f64().max(1e-9);
            let iterations = ((seconds / single).round() as usize).clamp(3, 5000);
            for _ in 0..iterations.min(50) {
                once(&input, ast);
            }
            let mut samples = Vec::new();
            for _ in 0..5 {
                let start = Instant::now();
                for _ in 0..iterations {
                    once(&input, ast);
                }
                samples.push(start.elapsed().as_secs_f64() * 1000.0 / iterations as f64);
            }
            println!(
                "{name},{},{:.4},{iterations},{ok}",
                if ast { "ast" } else { "recognize" },
                median(samples)
            );
        }
    }
}
