//! `ubnfc` CLI。サブコマンド: `ir` / `validate` / `gen`（後続マイルストーンで実装）。

pub fn run(args: &[String]) -> Result<i32, String> {
    match args.first().map(String::as_str) {
        Some("--version") | Some("-v") => {
            println!("ubnfc {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        Some("--help") | Some("-h") | None => {
            print!("{}", USAGE);
            Ok(0)
        }
        Some(other) => Err(format!("unknown subcommand: {other}\n{USAGE}")),
    }
}

const USAGE: &str = "usage: ubnfc <ir|validate|gen> [options]\n";
