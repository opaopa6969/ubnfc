use std::{hint::black_box, time::Instant};
use ubnfc_p4_rust::{parse_formula, ParseOptions};
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let inputs: Vec<_> = ["complex", "complex-x64"]
        .iter()
        .map(|name| {
            (
                *name,
                std::fs::read_to_string(format!("{dir}/{name}.tiny")).unwrap(),
            )
        })
        .collect();
    let cells: Vec<_> = inputs
        .iter()
        .flat_map(|(name, text)| {
            [true, false].map(|ast| {
                (
                    *name,
                    text,
                    ParseOptions {
                        build_ast: ast,
                        ..Default::default()
                    },
                )
            })
        })
        .collect();
    for (_, text, opts) in &cells {
        assert!(parse_formula(text, *opts).ok);
        for _ in 0..1000 {
            black_box(parse_formula(text, *opts));
        }
    }
    let mut samples = vec![vec![]; cells.len()];
    println!(
        "# load={}",
        std::fs::read_to_string("/proc/loadavg").unwrap().trim()
    );
    for run in 0..7 {
        for j in 0..cells.len() {
            let i = if run % 2 == 0 { j } else { cells.len() - j - 1 };
            let (name, text, opts) = &cells[i];
            let n = if name.ends_with("x64") { 50 } else { 2000 };
            let start = Instant::now();
            for _ in 0..n {
                black_box(parse_formula(text, *opts));
            }
            samples[i].push(start.elapsed().as_secs_f64() * 1000.0 / n as f64);
        }
        println!(
            "# run={} load={}",
            run + 1,
            std::fs::read_to_string("/proc/loadavg").unwrap().trim()
        );
    }
    for (i, (name, _, opts)) in cells.iter().enumerate() {
        let raw = samples[i].clone();
        samples[i].sort_by(f64::total_cmp);
        println!(
            "{name},{},{:.6},{:.6},{:?}",
            if opts.build_ast { "ast" } else { "recognize" },
            samples[i][3],
            samples[i][0],
            raw
        );
    }
}
