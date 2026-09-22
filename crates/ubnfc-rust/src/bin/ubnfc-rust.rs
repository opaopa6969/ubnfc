use std::{env, path::PathBuf, process::ExitCode};
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let (mut ir, mut out) = (None, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ir" if ir.is_none() => ir = args.next().map(PathBuf::from),
            "--out" if out.is_none() => out = args.next().map(PathBuf::from),
            "--help" | "-h" => {
                println!("ubnfc-rust --ir grammar.ir.json --out DIR");
                return Ok(());
            }
            _ => return Err(format!("unknown or duplicate argument: {arg}").into()),
        }
    }
    let ir = ir.ok_or("--ir is required")?;
    let out = out.ok_or("--out is required")?;
    let grammar = ubnfc_front::ir::from_json(&std::fs::read_to_string(ir)?)?;
    ubnfc_rust::emit(&grammar, &Default::default())?.write_to(out)?;
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
