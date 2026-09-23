// Family B: Operators / Power（小さな式文法）の Rust 実装 2 本を 1 プロセスで交互に測る。
//
// - `gen_operators` / `gen_power` … ubnfc の Rust backend が同じ IR から生成した parser
// - `ubnfc-target-rust` … 「生成物の目標形」として人手で書いた parser
//
// 入力は両 README のベンチと同じもの（`1+2*3`、括弧深さ 8、512 atom、`2^3^2`）。
// 生成側は既定 options（predict / factor / lexical cache / buffer reuse が有効）で測る。
// 手書き側にはこれらの最適化が無い。条件差は報告に明記する。
include!("bench_util.rs");

#[global_allocator]
static ALLOC: Counting = Counting;

fn gen_options(build_ast: bool) -> gen_operators::ParseOptions {
    gen_operators::ParseOptions {
        build_ast,
        ..Default::default()
    }
}
fn gen_power_options(build_ast: bool) -> gen_power::ParseOptions {
    gen_power::ParseOptions {
        build_ast,
        ..Default::default()
    }
}

fn main() {
    let operators: Vec<(String, String)> = vec![
        ("1+2*3".to_owned(), "1+2*3".to_owned()),
        (
            "parentheses-8".to_owned(),
            format!("{}1+2*3{}", "(".repeat(8), ")".repeat(8)),
        ),
        ("atoms-512".to_owned(), vec!["1"; 512].join("+")),
    ];
    let powers: Vec<(String, String)> = vec![
        ("2^3^2".to_owned(), "2^3^2".to_owned()),
        ("power-64".to_owned(), vec!["2"; 64].join("^")),
    ];
    header();
    for (name, text) in &operators {
        let bytes = text.len();
        let gen_ok = gen_operators::parse_with_options(text, gen_options(true)).ok;
        let hw = ubnfc_target_rust::operators::parse_with_options(
            text,
            ubnfc_target_rust::ParseOptions { build_ast: true, require_eof: true },
        );
        eprintln!("# Operators {name}: gen={gen_ok} handwritten={}", hw.ok);
        for build_ast in [false, true] {
            let mode = if build_ast { "AST" } else { "認識" };
            let options = gen_options(build_ast);
            row("B/Operators", "ubnfc-rust(生成)", name, bytes, mode, || {
                let r = gen_operators::parse_with_options(std::hint::black_box(text.as_str()), options);
                assert!(r.ok);
                std::hint::black_box(r);
            });
            let hwo = ubnfc_target_rust::ParseOptions { build_ast, require_eof: true };
            row("B/Operators", "手書き Rust(目標形)", name, bytes, mode, || {
                let r = ubnfc_target_rust::operators::parse_with_options(
                    std::hint::black_box(text.as_str()),
                    std::hint::black_box(hwo),
                );
                assert!(r.ok);
                std::hint::black_box(r);
            });
        }
    }
    for (name, text) in &powers {
        let bytes = text.len();
        let gen_ok = gen_power::parse_with_options(text, gen_power_options(true)).ok;
        let hw = ubnfc_target_rust::power::parse_with_options(
            text,
            ubnfc_target_rust::power::Mode::Text,
            ubnfc_target_rust::ParseOptions { build_ast: true, require_eof: true },
        );
        eprintln!("# Power {name}: gen={gen_ok} handwritten={}", hw.ok);
        for build_ast in [false, true] {
            let mode = if build_ast { "AST" } else { "認識" };
            let options = gen_power_options(build_ast);
            row("B/Power", "ubnfc-rust(生成)", name, bytes, mode, || {
                let r = gen_power::parse_with_options(std::hint::black_box(text.as_str()), options);
                assert!(r.ok);
                std::hint::black_box(r);
            });
            let hwo = ubnfc_target_rust::ParseOptions { build_ast, require_eof: true };
            row("B/Power", "手書き Rust(目標形)", name, bytes, mode, || {
                let r = ubnfc_target_rust::power::parse_with_options(
                    std::hint::black_box(text.as_str()),
                    ubnfc_target_rust::power::Mode::Text,
                    std::hint::black_box(hwo),
                );
                assert!(r.ok);
                std::hint::black_box(r);
            });
        }
    }
}
