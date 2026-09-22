use std::{ffi::OsStr, path::PathBuf, process::ExitCode};
use ubnfc_harness::{
    default_root, load_all,
    runner::{self, Artifact, Backend, Report, Runner, Scratch},
    summary,
};

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "run-all") {
        return execute_all(&args[1..]);
    }
    if args.first().is_some_and(|arg| arg == "run") {
        return execute(&args[1..]);
    }
    let root = match args.as_slice() {
        [cmd] if cmd == "list" => default_root(),
        [cmd, flag, path] if cmd == "list" && flag == "--root" => PathBuf::from(path),
        [flag] if flag == OsStr::new("--help") || flag == OsStr::new("-h") => {
            println!("使い方: ubnfc-harness list [--root DIR]\nubnfc-harness run --backend rust|java (--artifact DIR | --ir FILE) --families associative,right-associative [--root DIR] [--variant NAME] [--report out.md|out.json]\n{RUN_ALL_USAGE}");
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

const RUN_ALL_USAGE: &str = "ubnfc-harness run-all --backend java|rust [--families a,b] [--report out.md] [--json out.json] [--jobs N] [--root DIR]";

fn execute_all(args: &[std::ffi::OsString]) -> Result<(), String> {
    use ubnfc_harness::runall::{self, Config};
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!("{RUN_ALL_USAGE}\nJava の既定 jobs=4。Rust の --jobs は Cargo に渡します（未指定時は Cargo の既定値）。UBNFC_BIN / UBNFC_JAVA_GENERATOR / UBNFC_RUST_GENERATOR で実行ファイルを指定できます。");
        return Ok(());
    }
    if args.len() % 2 != 0 {
        return Err(format!("各オプションには値が必要です\n{RUN_ALL_USAGE}"));
    }
    let mut flags = std::collections::BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let key = pair[0].to_str().ok_or("不正なオプション")?;
        if ![
            "--backend",
            "--families",
            "--report",
            "--json",
            "--jobs",
            "--root",
        ]
        .contains(&key)
        {
            return Err(format!("未知のオプション: {key}"));
        }
        if pair[1].is_empty() {
            return Err(format!("{key} の値が空です"));
        }
        if flags.insert(key, &pair[1]).is_some() {
            return Err(format!("重複したオプション: {key}"));
        }
    }
    let backend = match flags.get("--backend").and_then(|s| s.to_str()) {
        Some("java") => Backend::Java,
        Some("rust") => Backend::Rust,
        _ => return Err("--backend java|rust が必要です".into()),
    };
    let mut config = Config {
        backend,
        ..Config::default()
    };
    if let Some(jobs) = flags.get("--jobs") {
        config.jobs = jobs
            .to_str()
            .and_then(|s| s.parse().ok())
            .filter(|n| *n > 0)
            .ok_or("--jobs は1以上の整数が必要です")?;
        config.cargo_jobs = Some(config.jobs);
    }
    if let Some(families) = flags.get("--families") {
        config.families = families
            .to_str()
            .ok_or("不正な系統名")?
            .split(',')
            .map(str::to_owned)
            .collect();
        if config.families.iter().any(String::is_empty) {
            return Err("--families の系統名が空です".into());
        }
    }
    if flags.contains_key("--report") && flags.get("--report") == flags.get("--json") {
        return Err("--report と --json は別の出力先が必要です".into());
    }
    let root = flags
        .get("--root")
        .map(PathBuf::from)
        .unwrap_or_else(default_root);
    let files = load_all(root).map_err(|e| e.to_string())?;
    let report = runall::run(&files, &config).map_err(|e| e.to_string())?;
    for (flag, content) in [("--report", report.markdown()), ("--json", report.json())] {
        if let Some(path) = flags.get(flag) {
            let path = PathBuf::from(path);
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(path, content).map_err(|e| e.to_string())?;
        }
    }
    print!("{}", report.summary_markdown());
    let c = &report.totals;
    let failures = c.ir_failure
        + c.generation_rejected
        + c.javac_failure
        + c.build_failure
        + c.driver_error
        + c.fail;
    if failures == 0 {
        Ok(())
    } else {
        Err(format!(
            "{failures} ケースに段階失敗または Fail（レポート出力済み）"
        ))
    }
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
