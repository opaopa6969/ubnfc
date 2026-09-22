#![allow(dead_code)]
mod generated;
use std::{hint::black_box, time::Instant};
fn main() {
    let iterations: usize = std::env::var("UBNFC_BENCH_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    let inputs = [
        ("short", "1+2*3".into()),
        ("depth8", format!("{}1+2*3{}", "(".repeat(8), ")".repeat(8))),
        ("atoms512", vec!["1"; 512].join("+")),
    ];
    println!("implementation,input,ast,median_us,samples_us,iterations");
    for (name, input) in inputs {
        for ast in [false, true] {
            for target in [false, true] {
                let parse = || {
                    if target {
                        let r = ubnfc_target_rust::operators::parse_with_options(
                            black_box(&input),
                            black_box(ubnfc_target_rust::ParseOptions {
                                build_ast: ast,
                                require_eof: true,
                            }),
                        );
                        assert!(r.ok);
                        black_box(r);
                    } else {
                        let r = generated::parse_with_options(
                            black_box(&input),
                            black_box(generated::ParseOptions {
                                build_ast: ast,
                                require_eof: true,
                                lexical: false,
                            }),
                        );
                        assert!(r.ok);
                        assert_eq!(r.ast.is_some(), ast);
                        black_box(r);
                    }
                };
                for _ in 0..100 {
                    parse();
                }
                let mut samples = [0.0; 5];
                for sample in &mut samples {
                    let start = Instant::now();
                    for _ in 0..iterations {
                        parse();
                    }
                    *sample = start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64;
                }
                let mut sorted = samples;
                sorted.sort_by(f64::total_cmp);
                println!(
                    "{},{name},{ast},{:.3},\"{samples:?}\",{iterations}",
                    if target { "handwritten" } else { "generated" },
                    sorted[2]
                );
            }
        }
    }
}
