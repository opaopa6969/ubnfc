//! 同じ生成 parser の LL(1) 表 on/off を、同じ反復数で交互に測る。
use std::{hint::black_box, time::Instant};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let grammar = &args[1];
    let rounds: usize = std::env::var("UBNFC_LL1_ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(9);
    let seconds: f64 = std::env::var("UBNFC_LL1_SECONDS").ok().and_then(|v| v.parse().ok()).unwrap_or(0.2);
    println!("input,bytes,round,ll1,iterations,ms,us_per_byte");
    for path in &args[2..] {
        let text = std::fs::read_to_string(path).unwrap();
        let parse = |enabled| {
            if grammar == "json" {
                let r = gen_json::parse_with_options(black_box(&text), gen_json::ParseOptions { build_ast: false, ll1: black_box(enabled), ..Default::default() });
                assert!(r.ok); black_box(r);
            } else {
                let r = gen_p4::parse_formula(black_box(&text), gen_p4::ParseOptions { build_ast: false, ll1: black_box(enabled), ..Default::default() });
                assert!(r.ok); black_box(r);
            }
        };
        for _ in 0..20 { parse(false); parse(true); }
        let start = Instant::now();
        for _ in 0..3 { parse(false); }
        let once = start.elapsed().as_secs_f64() / 3.0;
        let iterations = ((seconds / once) as usize).clamp(3, 200_000);
        for round in 0..rounds {
            for enabled in if round % 2 == 0 { [false, true] } else { [true, false] } {
                let start = Instant::now();
                for _ in 0..iterations { parse(enabled); }
                let ms = start.elapsed().as_secs_f64() * 1e3 / iterations as f64;
                println!("{},{},{round},{enabled},{iterations},{ms:.9},{:.9}", std::path::Path::new(path).file_name().unwrap().to_str().unwrap(), text.len(), ms * 1e3 / text.len() as f64);
            }
        }
    }
}
