// 外部比較（issue #26）の ubnfc Rust 側ドライバ。
//
//   family_ext check <json|operators|power> <file>...
//       1 行 1 ファイルで TSV: path<TAB>accept|reject<TAB>理由
//   family_ext bench <json|operators|power> <認識|AST> <file>...
//       CSV は bench_util.rs の共通形式
//
// 判定の作法:
//   - 入力は byte 列を**厳密な** UTF-8 として復号する。生成 Rust の入口は `&str` なので、
//     不正 UTF-8 はそもそも渡せない（`not-utf8` として拒否する）。
//   - `require_eof: true`（既定）。末尾に余りがあれば ok=false。
//   - 認識のみは `build_ast: false`、typed AST は `build_ast: true`。
//     どちらも既定の最適化（memo / predict / factor / buffer 再利用）のままで、
//     JSON 向けの特別扱いはしていない。
include!("bench_util.rs");

#[global_allocator]
static ALLOC: Counting = Counting;

fn read(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read-error:{e}"))?;
    String::from_utf8(bytes).map_err(|_| "not-utf8".to_owned())
}

/// `UBNFC_ARENA_ENTRIES` で `ResourceLimits.arena_entries` を上書きする。
///
/// issue #35 / D-071: 既定値（2,000,000）は `Session::new` が入力長に応じて自動的に
/// 引き上げる（未設定のときだけ）ので、1 MB の JSON も既定のままで通るようになった。
/// この env var は速度比較で入力サイズごとに設定を変えず**固定値**で揃えたいときの
/// 明示的な上書き用に残してある（明示すると自動スケーリングの対象外になる）。
/// 認識のみの経路はこの上限に触れない。
macro_rules! limits {
    ($ns:ident) => {{
        let mut limits = $ns::ResourceLimits::default();
        if let Ok(value) = std::env::var("UBNFC_ARENA_ENTRIES") {
            if let Ok(parsed) = value.parse::<usize>() {
                limits.arena_entries = parsed;
            }
        }
        limits
    }};
}

/// 文法ごとの入口を 1 つの型に揃える。
struct Entry {
    recognize: fn(&str) -> (bool, usize),
    ast: fn(&str) -> (bool, usize),
}

fn json_entry() -> Entry {
    fn recognize(text: &str) -> (bool, usize) {
        let r = gen_json::parse_with_options(
            text,
            gen_json::ParseOptions { build_ast: false, limits: limits!(gen_json), ..Default::default() },
        );
        (r.ok, r.consumed_cp)
    }
    fn ast(text: &str) -> (bool, usize) {
        let r = gen_json::parse_with_options(
            text,
            gen_json::ParseOptions { limits: limits!(gen_json), ..Default::default() },
        );
        (r.ok && r.ast.is_some(), r.consumed_cp)
    }
    Entry { recognize, ast }
}

fn operators_entry() -> Entry {
    fn recognize(text: &str) -> (bool, usize) {
        let r = gen_operators::parse_with_options(
            text,
            gen_operators::ParseOptions { build_ast: false, limits: limits!(gen_operators), ..Default::default() },
        );
        (r.ok, r.consumed_cp)
    }
    fn ast(text: &str) -> (bool, usize) {
        let r = gen_operators::parse_with_options(
            text,
            gen_operators::ParseOptions { limits: limits!(gen_operators), ..Default::default() },
        );
        (r.ok && r.ast.is_some(), r.consumed_cp)
    }
    Entry { recognize, ast }
}

fn power_entry() -> Entry {
    fn recognize(text: &str) -> (bool, usize) {
        let r = gen_power::parse_with_options(
            text,
            gen_power::ParseOptions { build_ast: false, limits: limits!(gen_power), ..Default::default() },
        );
        (r.ok, r.consumed_cp)
    }
    fn ast(text: &str) -> (bool, usize) {
        let r = gen_power::parse_with_options(
            text,
            gen_power::ParseOptions { limits: limits!(gen_power), ..Default::default() },
        );
        (r.ok && r.ast.is_some(), r.consumed_cp)
    }
    Entry { recognize, ast }
}

fn entry(grammar: &str) -> Entry {
    match grammar {
        "json" => json_entry(),
        "operators" => operators_entry(),
        "power" => power_entry(),
        other => {
            eprintln!("unknown grammar: {other}");
            std::process::exit(2);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: family_ext check <grammar> <file>... | family_ext bench <grammar> <認識|AST> <file>...");
        std::process::exit(2);
    }
    let entry = entry(&args[1]);
    if args[0] == "check" {
        let mut out = String::new();
        for path in &args[2..] {
            let (verdict, reason) = match read(path) {
                Err(reason) => ("reject", reason),
                Ok(text) => {
                    let (ok, consumed) = (entry.recognize)(&text);
                    if ok {
                        ("accept", String::new())
                    } else {
                        ("reject", format!("failed at cp {consumed}"))
                    }
                }
            };
            out.push_str(&format!("{path}\t{verdict}\t{reason}\n"));
        }
        print!("{out}");
        return;
    }
    if args[0] != "bench" || args.len() < 4 {
        eprintln!("bad arguments");
        std::process::exit(2);
    }
    let build_ast = args[2] == "AST";
    let parse = if build_ast { entry.ast } else { entry.recognize };
    let mode = if build_ast { "typed AST" } else { "認識のみ" };
    header();
    for path in &args[3..] {
        let text = read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let bytes = text.len();
        let (ok, _) = parse(&text);
        assert!(ok, "{path} を受理できない");
        let name = std::path::Path::new(path)
            .file_name()
            .map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
        row("ext", "ubnfc-rust(生成)", &name, bytes, mode, || {
            let r = parse(std::hint::black_box(text.as_str()));
            std::hint::black_box(r);
        });
    }
}
