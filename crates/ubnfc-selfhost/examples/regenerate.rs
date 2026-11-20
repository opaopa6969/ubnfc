//! upstream UBNF から IR、自己ホスト用の明示的な語境界、capture metadata を再生成する。
use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let upstream = std::env::var_os("UNLAXER_PARSER_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repository.join("../unlaxer-parser"));
    let path = upstream.join("unlaxer-dsl/grammar/ubnf.ubnf");
    let compiled =
        ubnfc_front::pipeline::compile(&path, &|p: &std::path::Path| fs::read_to_string(p))?;
    for diagnostic in &compiled.warnings {
        eprintln!("{diagnostic:?}");
    }
    let ir = &compiled.ir;
    fs::write(
        repository.join("ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json"),
        ubnfc_front::ir::to_canonical_json(ir),
    )?;
    let parser_ir = ubnfc_selfhost::with_keyword_boundaries(ir).map_err(|e| format!("{e:?}"))?;
    let mut generated = ubnfc_rust::emit(
        &parser_ir,
        &ubnfc_rust::EmitOptions {
            emit_driver: false,
            ..Default::default()
        },
    )?;
    generated.files.remove("Cargo.toml");
    generated.write_to(repository.join("crates/ubnfc-selfhost/src/generated"))?;
    let metadata_path = repository.join("crates/ubnfc-selfhost/src/capture_sites.rs");
    let old = fs::read_to_string(&metadata_path)?;
    let start = old
        .find("pub(crate) const CAPTURE_RULES:")
        .ok_or("capture table header")?;
    let end = start + old[start..].find("\n];").ok_or("capture table footer")? + 3;
    let mut updated = old[..start].to_owned();
    updated.push_str("pub(crate) const CAPTURE_RULES: &[(&str, &str)] = &[\n");
    for capture in &ir.semantics.captures {
        updated.push_str(&format!(
            "    (\n        {:?},\n        {:?},\n    ),\n",
            capture.site_id, capture.rule_id
        ));
    }
    updated.push_str("];\n");
    updated.push_str(old[end..].trim_start_matches('\n'));
    fs::write(metadata_path, updated)?;
    Ok(())
}
