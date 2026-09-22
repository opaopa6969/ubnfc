use std::{env, path::PathBuf, process::ExitCode};
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1).peekable();
    let (mut ir, mut out) = (None, None);
    let mut emit_driver = true;
    let mut driver_seen = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ir" if ir.is_none() => ir = args.next().map(PathBuf::from),
            "--out" if out.is_none() => out = args.next().map(PathBuf::from),
            "--emit-driver" | "--no-emit-driver" if !driver_seen => {
                driver_seen = true;
                emit_driver = arg == "--emit-driver";
                if emit_driver && args.peek().is_some_and(|s| !s.starts_with("--")) {
                    emit_driver = match args.next().as_deref() {
                        Some("true") => true,
                        Some("false") => false,
                        _ => return Err("--emit-driver requires true or false".into()),
                    };
                }
            }
            "--help" | "-h" => {
                println!("ubnfc-rust --ir grammar.ir.json --out DIR [--emit-driver [true|false]] [--no-emit-driver]");
                return Ok(());
            }
            _ => return Err(format!("unknown or duplicate argument: {arg}").into()),
        }
    }
    let ir = ir.ok_or("--ir is required")?;
    let out = out.ok_or("--out is required")?;
    let grammar = ubnfc_front::ir::from_json(&std::fs::read_to_string(ir)?)?;
    ubnfc_rust::emit(
        &grammar,
        &ubnfc_rust::EmitOptions {
            emit_driver,
            ..Default::default()
        },
    )?
    .write_to(out)?;
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
