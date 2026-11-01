use std::{env, process::ExitCode};
use ubnfc_stax::{Limits, Program};

fn main() -> ExitCode {
    let mut check = false;
    let mut expand = false;
    let mut limits = Limits::default();
    let mut file = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" => check = true,
            "--expand" => expand = true,
            "--max-steps" | "--max-depth" => {
                let Some(Ok(n)) = args.next().map(|s| s.parse::<u64>()) else {
                    return usage();
                };
                if arg == "--max-steps" {
                    limits.steps = n;
                } else {
                    let Ok(n) = usize::try_from(n) else {
                        return usage();
                    };
                    limits.depth = n;
                }
            }
            _ if !arg.starts_with('-') && file.is_none() => file = Some(arg),
            _ => return usage(),
        }
    }
    if check && expand {
        return usage();
    }
    if limits.steps == 0 || limits.steps > 1_000_000 || limits.depth == 0 || limits.depth > 4096 {
        return usage();
    }
    let Some(file) = file else {
        return usage();
    };
    let result = Program::load(file).and_then(|p| {
        if expand {
            Ok(p.expansion())
        } else if check {
            Ok(String::new())
        } else {
            p.run(limits)
        }
    });
    match result {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("stax: {e}");
            ExitCode::from(1)
        }
    }
}
fn usage() -> ExitCode {
    eprintln!("使用法: stax [--check | --expand] [--max-steps N] [--max-depth N] FILE");
    ExitCode::from(2)
}
