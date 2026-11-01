// issue #3: SWAR の前後を同一 binary 内で交互に測る。
include!("bench_util.rs");
#[allow(dead_code)]
mod sc_after;
#[allow(dead_code)]
mod sc_before;


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
type Parse = Box<dyn Fn(&str) -> (bool, usize)>;

macro_rules! entry {
    ($p4:ident, $json:ident, $ops:ident, $sc:ident, $grammar:expr, $build:expr, $tree:expr, $reuse:expr) => {{
        macro_rules! options {
            ($k:ident) => {{
                let mut o = $k::ParseOptions {
                    build_ast: $build,
                    reuse_buffers: $reuse,
                    ..Default::default()
                };
                o.limits.arena_entries = 1 << 30;
                set_tree!(o, $tree);
                o
            }};
        }
        let parse: Parse = match $grammar {
            "p4" => {
                let o = options!($p4);
                Box::new(move |t: &str| {
                    let mut sc = $sc::Registry::new(EXTERN).unwrap();
                    let r = $p4::parse_with_scanner(t, o, &mut sc);
                    (r.ok, r.statistics.ast_nodes)
                })
            }
            "json" => {
                let o = options!($json);
                Box::new(move |t: &str| {
                    let r = $json::parse_with_options(t, o);
                    (r.ok, r.statistics.ast_nodes)
                })
            }
            _ => {
                let o = options!($ops);
                Box::new(move |t: &str| {
                    let r = $ops::parse_with_options(t, o);
                    (r.ok, r.statistics.ast_nodes)
                })
            }
        };
        parse
    }};
}
fn before(grammar: &str, build_ast: bool, reuse: bool) -> Parse {
    macro_rules! set_tree {
        ($o:ident, $t:expr) => {
            let _ = $t;
        };
    }
    entry!(
        p4_before,
        json_before,
        ops_before,
        sc_before,
        grammar,
        build_ast,
        false,
        reuse
    )
}
fn after(grammar: &str, build_ast: bool, tree: bool, reuse: bool) -> Parse {
    macro_rules! set_tree {
        ($o:ident, $t:expr) => {
            $o.ast_tree = $t;
        };
    }
    entry!(p4_after, json_after, ops_after, sc_after, grammar, build_ast, tree, reuse)
}
// 同じ IR・オプションで認識のみを交互に実行する。
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(args.len() >= 3 && args[0] == "time");
    let grammar = &args[1];
    let runs = env_usize("AB_RUNS", 7).max(3);
    let seconds = env_f64("AB_RUN_SECONDS", 0.5);
    println!("input,bytes,variant,iterations,round,ms,load");
    for path in &args[2..] {
        let text = std::fs::read_to_string(path).unwrap();
        let name = path.rsplit('/').next().unwrap();
        let parses = [before(grammar, false, true), after(grammar, false, false, true)];
        for parse in &parses { assert!(parse(&text).0, "受理: {path}"); }
        let start = Instant::now();
        parses[0](&text);
        let iterations = ((seconds / start.elapsed().as_secs_f64().max(1e-9)) as usize).clamp(3, 200_000);
        for parse in &parses {
            for _ in 0..(iterations/2).clamp(3, 2_000) { black_box(parse(black_box(&text))); }
        }
        for round in 0..runs {
            for k in 0..2 {
                let v = (round+k)%2;
                let start = Instant::now();
                for _ in 0..iterations { black_box(parses[v](black_box(&text))); }
                let ms = start.elapsed().as_secs_f64()*1e3/iterations as f64;
                let load = std::fs::read_to_string("/proc/loadavg").unwrap().split_whitespace().take(3).collect::<Vec<_>>().join("/");
                println!("{name},{},{},{iterations},{round},{ms:.6},{load}",text.len(),["before", "after"][v]);
            }
        }
    }
}
