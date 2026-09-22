//! cargo run --release -p ubnfc-target-rust --example bench
use std::{hint::black_box, time::Instant};
use ubnfc_target_rust::{operators, ParseOptions};

const WARMUP: usize = 100;
const ITERATIONS: usize = 100_000;
const RUNS: usize = 5;

fn main() {
    let inputs = [
        ("1+2*3", "1+2*3".to_owned()),
        (
            "parentheses-8",
            format!("{}1+2*3{}", "(".repeat(8), ")".repeat(8)),
        ),
        ("atoms-512", vec!["1"; 512].join("+")),
    ];
    println!("input,bytes,mode,median_ns_per_parse,runs_ns_per_parse");
    for (name, input) in inputs {
        for build_ast in [false, true] {
            let options = ParseOptions {
                build_ast,
                require_eof: true,
            };
            let check = operators::parse_with_options(&input, options);
            assert!(check.ok && check.consumed_cp == input.len());
            if build_ast {
                assert_eq!(
                    check.ast.as_ref().unwrap().evaluate(),
                    if name == "atoms-512" { 512.0 } else { 7.0 }
                );
            }
            let parse = || {
                black_box(operators::parse_with_options(
                    black_box(&input),
                    black_box(options),
                ));
            };
            for _ in 0..WARMUP {
                parse();
            }
            let mut samples = [0.0; RUNS];
            for (run, sample) in samples.iter_mut().enumerate() {
                let start = Instant::now();
                for _ in 0..ITERATIONS {
                    parse();
                }
                *sample = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
                eprintln!(
                    "{name} build_ast={build_ast} run={} ns/parse={sample:.1}",
                    run + 1
                );
            }
            let mut sorted = samples;
            sorted.sort_by(f64::total_cmp);
            println!(
                "{name},{},{},{:.1},\"{samples:?}\"",
                input.len(),
                if build_ast { "ast" } else { "recognize" },
                sorted[RUNS / 2]
            );
        }
    }
}
