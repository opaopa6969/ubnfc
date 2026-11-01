use std::{env, fs, path::PathBuf};

fn main() {
    let grammar = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../stax.ubnf");
    println!("cargo:rerun-if-changed={}", grammar.display());
    let compiled = ubnfc_front::pipeline::compile_with_strict(
        &grammar,
        &|path| fs::read_to_string(path),
        true,
    )
    .expect("stax 文法の strict 検証");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let generated = out.join("generated");
    ubnfc_rust::emit(
        &compiled.ir,
        &ubnfc_rust::EmitOptions {
            documentation: None,
            emit_driver: false,
        },
    )
    .expect("Rust パーサ生成")
    .write_to(&generated)
    .expect("生成物の保存");
    fs::write(
        out.join("parser.rs"),
        format!(
            "#[path = {:?}] pub mod generated;\n",
            generated.join("mod.rs")
        ),
    )
    .unwrap();
}
