// JSON 専用ライブラリ（Rust 側 = serde_json）の判定と計測（issue #68）。
//
//   family_jsonlib check <file>...
//       1 行 1 ファイルで TSV: path<TAB>accept|reject<TAB>理由
//   family_jsonlib bench <ignored|value> <file>...
//       CSV は bench_util.rs の共通形式（ubnfc Rust 側と同じ計測部を使う）
//
// モード:
//   ignored … `serde_json::from_str::<serde::de::IgnoredAny>`。**何も作らない**（認識相当）。
//   value   … `serde_json::from_str::<serde_json::Value>`。**木（span なし）**。
//
// 判定の作法:
//   - 入力は byte 列を**厳密な** UTF-8 として復号する（ubnfc Rust 側の driver と同じ）。
//     復号できないものは `not-utf8` として拒否する。
//   - `from_str` は末尾に空白以外が残っていれば `trailing characters` で落ちる（＝
//     「1 つの JSON 値を読み切る」判定になっていて、他の実装と揃う）。
//   - serde_json の既定は寛容 feature を持たない（`arbitrary_precision` 等の Cargo feature は
//     使っていない）ので、Jackson / Gson のような「既定 / 厳格 / 寛容」の 3 本立ては無い。
//
// simd-json は `~/.cargo/registry` に無く crates.io も遮断されているので**未計測**。
include!("bench_util.rs");

#[global_allocator]
static ALLOC: Counting = Counting;

fn read(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read-error:{e}"))?;
    String::from_utf8(bytes).map_err(|_| "not-utf8".to_owned())
}

type Parse = fn(&str) -> Result<(), String>;

fn parse_ignored(text: &str) -> Result<(), String> {
    serde_json::from_str::<serde::de::IgnoredAny>(text)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn parse_value(text: &str) -> Result<(), String> {
    serde_json::from_str::<serde_json::Value>(text)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: family_jsonlib check <file>... | family_jsonlib bench <ignored|value> <file>...");
        std::process::exit(2);
    }
    if args[0] == "check" {
        let mut out = String::new();
        for path in &args[1..] {
            let (verdict, reason) = match read(path) {
                Err(reason) => ("reject", reason),
                Ok(text) => match parse_ignored(&text) {
                    Ok(()) => ("accept", String::new()),
                    Err(e) => ("reject", e.replace(['\n', '\t'], " ")),
                },
            };
            out.push_str(&format!("{path}\t{verdict}\t{reason}\n"));
        }
        print!("{out}");
        return;
    }
    if args[0] != "bench" || args.len() < 3 {
        eprintln!("bad arguments");
        std::process::exit(2);
    }
    let value_mode = args[1] == "value";
    let (parse, mode): (Parse, &str) = if value_mode {
        (parse_value, "serde_json::Value(木・span なし)")
    } else {
        (parse_ignored, "IgnoredAny(何も作らない)")
    };
    header();
    for path in &args[2..] {
        let text = read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let bytes = text.len();
        parse(&text).unwrap_or_else(|e| panic!("{path} を受理できない: {e}"));
        let name = std::path::Path::new(path)
            .file_name()
            .map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
        row("jsonlib", "serde_json", &name, bytes, mode, || {
            let r = parse(std::hint::black_box(text.as_str()));
            std::hint::black_box(r).ok();
        });
    }
}
