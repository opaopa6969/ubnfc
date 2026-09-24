// Family D: P4 文法を ANTLR4 / tree-sitter へ移植したものとの比較（issue #27）の Rust 側。
//
// ここで測るのは ubnfc の Rust 生成 parser だけで、ANTLR（Java）と tree-sitter（C）は
// それぞれの driver が別プロセスで測る。1 入力ごとに 3 実装を順に回すのは
// `benchmarks/scripts/run-p4-external.sh` の役目。
//
// mode:
// - `認識`  … `build_ast=false`。木も capture も作らない。
// - `AST`   … `build_ast=true`。型付き AST を作る（ANTLR の parse tree / tree-sitter の
//              CST とは作るものが違う。報告の列見出しを参照）。
//
// scope: P4 の `@scopeTree(mode=lexical)` / `@declares` / `@backref` の効果を切る
// `ParseOptions` は存在しない（api.rs の ParseOptions に該当フィールドが無い）。
// scope 無しの数字は、IR の `semantics.scope.effects` と `semantics.catalogs` を空にして
// 生成し直した parser を同じ binary 構成で測って採る。どちらを測っているかは
// `ParseResult::scope.mode` が空かどうかで自動判定する。
include!("bench_util.rs");

#[global_allocator]
static ALLOC: Counting = Counting;

fn options(build_ast: bool) -> ubnfc_p4_rust::ParseOptions {
    let mut options = ubnfc_p4_rust::ParseOptions {
        build_ast,
        ..Default::default()
    };
    options.limits.diagnostics = 1 << 30;
    options.limits.arena_entries = 1 << 30;
    options.limits.memo_entries = 1 << 30;
    options
}

fn run(
    text: &str,
    entry: Option<&str>,
    options: ubnfc_p4_rust::ParseOptions,
) -> ubnfc_p4_rust::ParseResult {
    match entry {
        None => ubnfc_p4_rust::parse_formula(black_box(text), options),
        Some(e) => ubnfc_p4_rust::parse_entry(Some(e), black_box(text), options).unwrap(),
    }
}

/// `accept <dir> <name>...`: 各 fixture を Formula 入口で、comparison-heavy 系は
/// BooleanExpression 入口でも解析し、受理と消費 code point を出す。
fn accept(dir: &str, names: &[String]) {
    println!("fixture\tbytes\tentry\tok\tconsumed_cp\ttotal_cp");
    for name in names {
        let path = format!("{dir}/{name}.tiny");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let total = text.chars().count();
        let mut entries: Vec<Option<&str>> = vec![None];
        if name.starts_with("comparison-heavy") {
            entries.push(Some("BooleanExpression"));
        }
        for entry in entries {
            let mut o = options(true);
            o.limits.diagnostics = 1 << 30;
            let r = run(&text, entry, o);
            println!(
                "{name}\t{}\t{}\t{}\t{}\t{total}",
                text.len(),
                entry.unwrap_or("Formula"),
                r.ok,
                r.consumed_cp
            );
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next().expect("accept|bench");
    let (command, dir) = if first == "accept" || first == "bench" {
        (first, args.next().expect("fixture dir"))
    } else {
        ("bench".to_owned(), first)
    };
    let names: Vec<String> = args.collect();
    let probe = run("1 + 2", None, options(false));
    assert!(probe.ok);
    let label = if probe.scope.mode.is_empty() {
        "ubnfc-rust(生成, scope 無効化 IR)"
    } else {
        "ubnfc-rust(生成, scope あり)"
    };
    if command == "accept" {
        eprintln!("# {label}");
        accept(&dir, &names);
        return;
    }
    eprintln!("# {label}");
    header();
    for name in &names {
        let path = format!("{dir}/{name}.tiny");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let bytes = text.len();
        let entry = name
            .starts_with("comparison-heavy")
            .then_some("BooleanExpression");
        for build_ast in [false, true] {
            let options = options(build_ast);
            let mode = if build_ast { "AST" } else { "認識" };
            row("D", label, name, bytes, mode, || {
                let r = run(&text, entry, options);
                assert!(r.ok);
                black_box(r);
            });
        }
    }
}
