//! `ubnfc-fuzz`: 差分 fuzzing の campaign を回す CLI。
//!
//! ```sh
//! cargo run -p ubnfc-fuzz --release --offline -- --seeds 300 --minutes 60 --jobs 3
//! ```

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;
use ubnfc_fuzz::campaign::{self, Config};

const USAGE: &str = "\
ubnfc-fuzz — Java / Rust バックエンドの差分 fuzzing

    ubnfc-fuzz [--seeds N] [--start-seed N] [--inputs N] [--jobs N]
               [--minutes M] [--shrink-builds N] [--driver-timeout SEC]
               [--findings DIR] [--report FILE] [--json FILE]

  --seeds N           生成する文法の数（既定 64）
  --start-seed N      最初の seed（既定 1）。seed だけで campaign は再現する
  --inputs N          1 文法あたりの入力数（既定 32）
  --jobs N            並列 worker 数（既定 2）
  --minutes M         全体の制限時間。超えたら残り seed を打ち切る
  --shrink-builds N   1 所見の縮小で許す build 回数（既定 30）
  --driver-timeout S  driver 1 バッチの制限時間（秒、既定 90）
  --findings DIR      最小再現例の出力先（既定 crates/ubnfc-fuzz/findings）
  --report FILE       Markdown 要約の出力先
  --json FILE         集計 JSON の出力先
  --dump N            seed の文法と判定を表示して終了（triage 用）
  --front-only 1      --dump で backend の build を行わず front の判定だけを見る
  --minimize-front N  seed N の文法が front に拒否されるなら、同じ理由のまま最小化して表示
";

fn main() -> ExitCode {
    ubnfc_fuzz::tune_generated_builds();
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("ubnfc-fuzz: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(arguments: &[String]) -> Result<ExitCode, String> {
    let mut config = Config {
        seeds: 64,
        inputs: 32,
        findings: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("findings")),
        ..Config::default()
    };
    let mut report = None;
    let mut json = None;
    let mut dump = None;
    let mut front_only = false;
    let mut minimize_front = None;
    let mut index = 0;
    while index < arguments.len() {
        let key = arguments[index].as_str();
        if matches!(key, "--help" | "-h") {
            print!("{USAGE}");
            return Ok(ExitCode::SUCCESS);
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| format!("{key} に値がありません\n{USAGE}"))?;
        let number = |what: &str| -> Result<u64, String> {
            value
                .parse::<u64>()
                .map_err(|_| format!("{what} は整数で指定してください: {value}"))
        };
        match key {
            "--seeds" => config.seeds = number("--seeds")? as usize,
            "--start-seed" => config.start_seed = number("--start-seed")?,
            "--inputs" => config.inputs = number("--inputs")? as usize,
            "--jobs" => config.jobs = number("--jobs")?.max(1) as usize,
            "--shrink-builds" => config.shrink_builds = number("--shrink-builds")? as usize,
            "--driver-timeout" => {
                config.driver_timeout = Duration::from_secs(number("--driver-timeout")?)
            }
            "--minutes" => {
                let minutes: f64 = value
                    .parse()
                    .map_err(|_| format!("--minutes は数値で指定してください: {value}"))?;
                config.budget = Some(Duration::from_secs_f64(minutes * 60.0));
            }
            "--dump" => dump = Some(number("--dump")? as usize),
            "--front-only" => front_only = number("--front-only")? != 0,
            "--minimize-front" => minimize_front = Some(number("--minimize-front")?),
            "--findings" => config.findings = Some(PathBuf::from(value)),
            "--report" => report = Some(PathBuf::from(value)),
            "--json" => json = Some(PathBuf::from(value)),
            _ => return Err(format!("不明な引数: {key}\n{USAGE}")),
        }
        index += 2;
    }

    let scratch = scratch_dir()?;
    config.workdir = scratch.0.clone();
    if let Some(seed) = minimize_front {
        let mut engine = ubnfc_fuzz::engine::Engine::new(scratch.0.join("minimize"));
        let grammar = ubnfc_fuzz::gen::generate(seed);
        let Err(discard) = engine.compile_front(&grammar) else {
            return Err(format!("seed {seed} は front を通ります"));
        };
        let minimal = ubnfc_fuzz::campaign::minimize_front(&mut engine, &grammar, &discard.code);
        println!("{}", minimal.render());
        if let Some((file, source)) = minimal.imported_source() {
            println!("----- {file} -----\n{source}");
        }
        match engine.compile_front(&minimal) {
            Ok(_) => println!("-> ok（最小化で理由が消えました）"),
            Err(discard) => println!("-> [{}] {}", discard.code, discard.message),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if let Some(count) = dump {
        return dump_seeds(&config, &scratch.0, count, front_only).map(|()| ExitCode::SUCCESS);
    }
    if let Some(directory) = &config.findings {
        std::fs::create_dir_all(directory).map_err(|e| format!("{}: {e}", directory.display()))?;
    }
    eprintln!(
        "ubnfc-fuzz: seeds={} inputs={} jobs={} workdir={}",
        config.seeds,
        config.inputs,
        config.jobs,
        config.workdir.display()
    );
    let summary = campaign::run(&config);
    let markdown = campaign::markdown(&summary);
    print!("{markdown}");
    if let Some(path) = report {
        std::fs::write(&path, &markdown).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if let Some(path) = json {
        let text = serde_json::to_string_pretty(&summary).expect("summary json") + "\n";
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(ExitCode::SUCCESS)
}

/// seed ごとの文法原文と front の判定を出す。生成器の被覆を直接確かめるために使う。
fn dump_seeds(
    config: &Config,
    workdir: &std::path::Path,
    count: usize,
    front_only: bool,
) -> Result<(), String> {
    let mut engine = ubnfc_fuzz::engine::Engine::new(workdir.join("dump"));
    for index in 0..count {
        let seed = config.start_seed.wrapping_add(index as u64);
        let grammar = ubnfc_fuzz::gen::generate(seed);
        println!("===== seed {seed} =====\n{}", grammar.render());
        if let Some((file, source)) = grammar.imported_source() {
            println!("----- {file} -----\n{source}");
        }
        let result = if front_only {
            engine.compile_front(&grammar).map(|_| ())
        } else {
            engine.prepare(&grammar).map(|_| ())
        };
        match result {
            Ok(()) => println!("-> ok"),
            Err(discard) => println!("-> discard [{}] {}", discard.code, discard.message),
        }
    }
    Ok(())
}

/// この呼出し専用の一時ディレクトリ。終了時に削除する。
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch_dir() -> Result<Scratch, String> {
    let output = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(Scratch(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    )))
}
