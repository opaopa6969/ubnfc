use std::{ffi::OsStr, path::PathBuf, process::ExitCode};
use ubnfc_harness::{
    default_root, load_all,
    runner::{self, Artifact, Backend, Report, Runner, Scratch},
    summary,
};

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "run") {
        return execute(&args[1..]);
    }
    let root = match args.as_slice() {
        [cmd] if cmd == "list" => default_root(),
        [cmd, flag, path] if cmd == "list" && flag == "--root" => PathBuf::from(path),
        [flag] if flag == OsStr::new("--help") || flag == OsStr::new("-h") => {
            println!("使い方: ubnfc-harness list [--root DIR]\nubnfc-harness run --backend rust|java (--artifact DIR | --ir FILE) --families associative,right-associative [--root DIR] [--variant NAME] [--report out.md|out.json]");
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

fn execute(args: &[std::ffi::OsString]) -> Result<(), String> {
    let mut flags = std::collections::BTreeMap::new();
    if args.len() % 2 != 0 {
        return Err("各オプションには値が必要です".into());
    }
    for pair in args.chunks_exact(2) {
        let key = pair[0].to_str().ok_or("不正なオプション")?;
        if ![
            "--backend",
            "--artifact",
            "--ir",
            "--families",
            "--root",
            "--variant",
            "--report",
        ]
        .contains(&key)
        {
            return Err(format!("未知のオプション: {key}"));
        }
        if flags.insert(key, &pair[1]).is_some() {
            return Err(format!("重複したオプション: {key}"));
        }
    }
    let backend = match flags.get("--backend").and_then(|v| v.to_str()) {
        Some("rust") => Backend::Rust,
        Some("java") => Backend::Java,
        _ => return Err("--backend rust|java が必要です".into()),
    };
    let artifact = match (flags.get("--artifact"), flags.get("--ir")) {
        (Some(dir), None) => Artifact::Prebuilt {
            dir: PathBuf::from(dir),
        },
        (None, Some(path)) => Artifact::FromIr {
            ir_path: PathBuf::from(path),
            backend,
        },
        _ => return Err("--artifact DIR または --ir FILE のいずれか一つが必要です".into()),
    };
    let families = flags
        .get("--families")
        .and_then(|v| v.to_str())
        .ok_or("--families NAME,... が必要です")?
        .split(',')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let root = flags
        .get("--root")
        .map(PathBuf::from)
        .unwrap_or_else(default_root);
    let files = load_all(root).map_err(|error| error.to_string())?;
    let cases = runner::select(
        &files,
        &families,
        flags.get("--variant").and_then(|v| v.to_str()),
    )
    .map_err(|error| error.to_string())?;
    let scratch = Scratch::new().map_err(|error| error.to_string())?;
    let runner = Runner {
        backend,
        build_dir: scratch.0.clone(),
    };
    let executable = runner.build(artifact).map_err(|error| error.to_string())?;
    let actual = runner.run(&executable, &cases);
    let report = Report::from_actual(&cases, &actual).map_err(|error| error.to_string())?;
    if let Some(path) = flags.get("--report") {
        let path = PathBuf::from(path);
        let content = if path.extension().is_some_and(|ext| ext == "json") {
            report.json()
        } else {
            report.markdown()
        };
        std::fs::write(path, content).map_err(|error| error.to_string())?;
    }
    print!("{}", report.markdown());
    if report.fail.is_empty() {
        Ok(())
    } else {
        Err(format!("{} ケースが Fail", report.fail.len()))
    }
}
