// 暖機・入力 I/O を集計対象から外すため、measured だけを callgrind で収集する。
use std::{hint::black_box, time::Instant};
#[allow(dead_code)]
mod scanners;
extern "C" {
    fn ubnfc_profile_start();
    fn ubnfc_profile_stop();
}
const EXTERN: &[(&str, &str)] = &[
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
// 関数境界を残す。parser 本体の inline 判断は変更しない。
#[inline(never)]
fn measured(parse: &dyn Fn(), count: usize) {
    for _ in 0..count {
        parse();
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(
        args.len(),
        6,
        "driver <p4|json> <recognize|ast> <file> <warmup> <count>"
    );
    let text = std::fs::read_to_string(&args[3]).unwrap();
    let ast = match args[2].as_str() {
        "recognize" => false,
        "ast" => true,
        _ => panic!("不明な mode"),
    };
    let parse: Box<dyn Fn()> = match args[1].as_str() {
        "p4" => Box::new(|| {
            let mut scanner = scanners::Registry::new(EXTERN).unwrap();
            let r = gen_p4::parse_with_scanner(
                black_box(&text),
                gen_p4::ParseOptions {
                    build_ast: ast,
                    ..Default::default()
                },
                &mut scanner,
            );
            assert!(r.ok && (!ast || r.ast.is_some()));
            black_box(r);
        }),
        "json" => Box::new(|| {
            let r = gen_json::parse_with_options(
                black_box(&text),
                gen_json::ParseOptions {
                    build_ast: ast,
                    ..Default::default()
                },
            );
            assert!(r.ok && (!ast || r.ast.is_some()));
            black_box(r);
        }),
        _ => panic!("不明な文法"),
    };
    let warmup: usize = args[4].parse().unwrap();
    let count: usize = args[5].parse().unwrap();
    assert!(warmup > 0 && count > 0);
    for _ in 0..warmup {
        parse();
    }
    let start = Instant::now();
    // 公式 header の client request。C 側はポインタを保持しない。
    unsafe {
        ubnfc_profile_start();
    }
    measured(&*parse, count);
    unsafe {
        ubnfc_profile_stop();
    }
    println!(
        "文法,mode,bytes,暖機,反復,ns/parse\n{},{},{},{},{},{}",
        args[1],
        args[2],
        text.len(),
        warmup,
        count,
        start.elapsed().as_nanos() / count as u128
    );
}
