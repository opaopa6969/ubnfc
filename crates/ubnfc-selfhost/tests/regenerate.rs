use std::path::{Path, PathBuf};
use std::process::Command;

struct Temporary(PathBuf);
impl Temporary {
    fn new() -> Self {
        let output = Command::new("mktemp").arg("-d").output().unwrap();
        assert!(output.status.success());
        Self(PathBuf::from(
            String::from_utf8(output.stdout).unwrap().trim(),
        ))
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn files(root: &Path, current: &Path, result: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(current).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(root, &path, result);
        } else {
            result.push(path.strip_prefix(root).unwrap().to_path_buf());
        }
    }
}

fn assert_generated_bytes(ir: &ubnfc_front::ir::GrammarIr) {
    // self-host はライブラリとして埋め込むので driver は生成しない（生成物一覧の比較対象外）。
    let options = ubnfc_rust::EmitOptions {
        emit_driver: false,
        ..Default::default()
    };
    let ir = ubnfc_selfhost::with_keyword_boundaries(ir).unwrap();
    let generated = ubnfc_rust::emit(&ir, &options).unwrap();
    let temp = Temporary::new();
    generated.write_to(&temp.0).unwrap();
    let committed = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/generated");
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    files(&temp.0, &temp.0, &mut actual);
    files(&committed, &committed, &mut expected);
    // 生成 crate の Cargo.toml は単体 crate 用。self-host は module として埋め込むので比較対象外。
    actual.retain(|p| p != Path::new("Cargo.toml"));
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected, "余分な生成物・欠落も検出する");
    for file in actual {
        assert_eq!(
            std::fs::read(committed.join(&file)).unwrap(),
            std::fs::read(temp.0.join(&file)).unwrap(),
            "生成物が不一致: {}",
            file.display()
        );
    }
}

#[test]
fn unmodified_golden_regenerates_every_committed_byte() {
    let ir = ubnfc_front::ir::from_json(include_str!(
        "../../../ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json"
    ))
    .unwrap();
    assert_generated_bytes(&ir);
}

#[test]
fn generated_frontend_reaches_the_ir_and_rust_fixed_point() {
    let root = std::env::var_os("UNLAXER_PARSER_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../unlaxer-parser"),
        PathBuf::from,
    );
    let grammar = root.join("unlaxer-dsl/grammar/ubnf.ubnf");
    let source = std::fs::read_to_string(&grammar).unwrap();
    let ast = ubnfc_selfhost::parse_file(&source).unwrap();
    let compiled = ubnfc_front::pipeline::compile_parsed(
        &grammar,
        ubnfc_front::pipeline::ParsedSource::new(source, ast),
        &|path| {
            panic!(
                "ubnf.ubnf に予期しない import があります: {}",
                path.display()
            )
        },
    )
    .unwrap();
    let actual = ubnfc_front::ir::to_canonical_json(&compiled.ir);
    let expected = include_str!("../../../ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json");
    assert_eq!(actual.as_bytes(), expected.as_bytes(), "self-host IR");
    assert_generated_bytes(&compiled.ir);
}
