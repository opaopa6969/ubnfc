use std::{ffi::OsStr, path::PathBuf, process::ExitCode};
use ubnfc_harness::{default_root, load_all, summary};

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let root = match args.as_slice() {
        [cmd] if cmd == "list" => default_root(),
        [cmd, flag, path] if cmd == "list" && flag == "--root" => PathBuf::from(path),
        [flag] if flag == OsStr::new("--help") || flag == OsStr::new("-h") => {
            println!("使い方: ubnfc-harness list [--root DIR]");
            return Ok(());
        }
        _ => return Err("使い方: ubnfc-harness list [--root DIR]".into()),
    };
    let files = load_all(root).map_err(|e| e.to_string())?;
    println!("| 系統 | ファイル数 | ケース数 |\n|---|---:|---:|");
    for (family, count) in summary(&files) {
        println!("| {family} | {} | {} |", count.files, count.cases);
    }
    println!(
        "| 合計 | {} | {} |",
        files.len(),
        files.iter().map(|f| f.cases.len()).sum::<usize>()
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
