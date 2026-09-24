// issue #59（D-077）の A/B ドライバ: 基準 revision の生成器（before）と arena AST の生成器
// （after）が出した P4 / JSON / Operators の parser を 1 プロセスに並べて、確保と時間を測る。
//
//   family_arena alloc <p4|json|ops> <file>...
//   family_arena time  <p4|json|ops> <file>...
//   family_arena tree  <p4|json|ops> <file>...   （after の木の節点・slot・容量）
//
// 変種は 5 つ: before 認識 / after 認識 / before 所有 AST / after 所有 AST / after 木だけ
// （`build_ast: false, ast_tree: true`）。
// - alloc: 定常状態（同じ入力を 3 回解析して pool の容量を落ち着かせた後の 1 回）と、
//   cold（`reuse_buffers: false`、pool なし = プロセスで最初の解析と同じ）を数える。
// - time: 1 run は `AB_RUN_SECONDS`（既定 0.5 秒）ぶんの反復、`AB_RUNS`（既定 7）周。
//   **周ごとに 5 変種を順番を回しながら交互に**走らせる（ホスト負荷の傾きを片側へ寄せない）。
//   中央値と最小値を出す。CPU は呼出し側が `taskset` で固定する。
include!("bench_util.rs");
#[allow(dead_code)]
mod sc_after;
#[allow(dead_code)]
mod sc_before;
#[global_allocator]
static ALLOC: Counting = Counting;

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
/// (名前, before か, build_ast, ast_tree)
const VARIANTS: &[(&str, bool, bool, bool)] = &[
    ("before 認識", true, false, false),
    ("after 認識", false, false, false),
    ("before AST", true, true, false),
    ("after AST", false, true, false),
    ("after 木", false, false, true),
];
fn make(grammar: &str, variant: usize, reuse: bool) -> Parse {
    let (_, is_before, build_ast, tree) = VARIANTS[variant];
    if is_before {
        before(grammar, build_ast, reuse)
    } else {
        after(grammar, build_ast, tree, reuse)
    }
}
fn counted(parse: &Parse, text: &str, warm: bool) -> (usize, usize, usize) {
    if warm {
        // 定常状態: 1 回目は伸長、2 回目は pool 上限での縮小の後始末が入り得るので 3 回回す。
        for _ in 0..3 {
            let _ = parse(text);
        }
    }
    COUNT.store(0, Relaxed);
    BYTES.store(0, Relaxed);
    ACTIVE.store(true, Relaxed);
    let (ok, nodes) = parse(text);
    ACTIVE.store(false, Relaxed);
    assert!(ok, "受理できない");
    (COUNT.load(Relaxed), BYTES.load(Relaxed), nodes)
}
fn time(parse: &Parse, text: &str, iterations: usize) -> f64 {
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(parse(black_box(text)));
    }
    start.elapsed().as_secs_f64() * 1e3 / iterations as f64
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: family_arena <alloc|time> <p4|json|ops> <file>...");
        std::process::exit(2);
    }
    let (what, grammar) = (args[0].as_str(), args[1].as_str());
    let runs = env_usize("AB_RUNS", 7).max(3);
    let run_seconds = env_f64("AB_RUN_SECONDS", 0.5);
    if what == "tree" {
        // 木の大きさ（節点・slot・容量）。bytes/節点の表に使う。
        println!("input,bytes,nodes,typed_nodes,slots,heap_bytes,source_bytes");
        for path in &args[2..] {
            let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let name = path.rsplit('/').next().unwrap();
            macro_rules! stats {
                ($r:expr) => {{
                    let r = $r;
                    let t = r.tree.as_ref().expect("tree");
                    println!(
                        "{name},{},{},{},{},{},{}",
                        text.len(),
                        t.node_count(),
                        t.typed_node_count(),
                        t.slot_count(),
                        t.heap_bytes(),
                        t.source().len()
                    );
                }};
            }
            match grammar {
                "p4" => {
                    let mut sc = sc_after::Registry::new(EXTERN).unwrap();
                    let o = p4_after::ParseOptions {
                        build_ast: false,
                        ast_tree: true,
                        ..Default::default()
                    };
                    stats!(p4_after::parse_with_scanner(&text, o, &mut sc))
                }
                "json" => stats!(json_after::parse_with_options(
                    &text,
                    json_after::ParseOptions {
                        build_ast: false,
                        ast_tree: true,
                        ..Default::default()
                    }
                )),
                _ => stats!(ops_after::parse_with_options(
                    &text,
                    ops_after::ParseOptions {
                        build_ast: false,
                        ast_tree: true,
                        ..Default::default()
                    }
                )),
            }
        }
        return;
    }
    if what == "alloc" {
        println!("input,bytes,variant,allocs,alloc_bytes,cold_allocs,cold_alloc_bytes,ast_nodes");
    } else {
        println!("input,bytes,variant,iterations,runs,median_ms,min_ms,load_start,load_end");
    }
    for path in &args[2..] {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let name = path.rsplit('/').next().unwrap();
        if what == "alloc" {
            for (v, (label, ..)) in VARIANTS.iter().enumerate() {
                let (n, b, nodes) = counted(&make(grammar, v, true), &text, true);
                let (cn, cb, _) = counted(&make(grammar, v, false), &text, false);
                println!("{name},{},{label},{n},{b},{cn},{cb},{nodes}", text.len());
            }
            continue;
        }
        let parses: Vec<Parse> = (0..VARIANTS.len())
            .map(|v| make(grammar, v, true))
            .collect();
        // 反復数は最も遅い変種（before AST）で決め、全変種で揃える。
        let once = Instant::now();
        parses[2](&text);
        let per = once.elapsed().as_secs_f64().max(1e-9);
        let iterations = ((run_seconds / per) as usize).clamp(3, 200_000);
        for parse in &parses {
            for _ in 0..(iterations / 2).clamp(3, 2_000) {
                black_box(parse(black_box(&text)));
            }
        }
        let load = || {
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .split(' ')
                .next()
                .unwrap_or("")
                .to_owned()
        };
        let load_start = load();
        let mut samples = vec![Vec::new(); parses.len()];
        for round in 0..runs {
            for k in 0..parses.len() {
                let v = (round + k) % parses.len();
                samples[v].push(time(&parses[v], &text, iterations));
            }
        }
        let load_end = load();
        for (v, (label, ..)) in VARIANTS.iter().enumerate() {
            let s = &mut samples[v];
            s.sort_by(f64::total_cmp);
            println!(
                "{name},{},{label},{iterations},{runs},{:.4},{:.4},{load_start},{load_end}",
                text.len(),
                s[s.len() / 2],
                s[0]
            );
        }
    }
}
