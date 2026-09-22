//! golden の目視確認用: cargo run -p ubnfc-target-rust --example inspect -- operators '1+2'
use ubnfc_target_rust::{canonical, operators, power};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert!(
        args.len() >= 2,
        "inspect operators INPUT | inspect power MODE INPUT"
    );
    let result = match args[0].as_str() {
        "operators" => operators::parse(&args[1]),
        "power" => {
            let mode = power::Mode::ALL
                .into_iter()
                .find(|mode| mode.name() == args[1])
                .expect("mode");
            power::parse_in_mode(&args[2], mode)
        }
        _ => panic!("grammar"),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": result.ok, "consumed_cp": result.consumed_cp, "matched_cp": result.matched_cp,
            "ast": result.ast.as_ref().map(canonical::value),
        }))
        .unwrap()
    );
    eprintln!(
        "diagnostics: {:?}\ncaptures: {:?}\nstatistics: {:?}",
        result.diagnostics, result.captures, result.statistics
    );
}
