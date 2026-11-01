use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn round2_generated_contracts_and_m3() {
    let output = Command::new("mktemp").arg("-d").output().unwrap();
    assert!(output.status.success());
    let dir = PathBuf::from(String::from_utf8(output.stdout).unwrap().trim());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut sources = Vec::new();
    for name in [
        "catalog",
        "typeof",
        "error",
        "ci-unicode",
        "enum-collision",
        "enum-plain",
        "eval-literal",
        "eval-passthrough",
        "eval-variable_ref",
        "eval-conditional",
        "eval-binary_arithmetic",
        "eval-manual",
        "eval-unknown",
        "recovery-sync",
        "recovery-auto",
        "recovery-root",
        "recovery-skip",
    ] {
        let value: Value = serde_json::from_str(
            &fs::read_to_string(root.join(format!("corpus/oracle/{name}.json"))).unwrap(),
        )
        .unwrap();
        sources.push((
            if name == "typeof" {
                "typeof_".into()
            } else {
                name.replace('-', "_")
            },
            value["grammar"].as_str().unwrap().to_owned(),
        ));
    }
    let external = std::env::var_os("UNLAXER_PARSER_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("../unlaxer-parser"));
    for (family, names) in [
        (
            "scope-annotations",
            vec![
                "quantified-capture-occurrences",
                "repeated-capture-occurrences",
                "trimmed-group-non-bmp",
                "nested-same-name-captures",
            ],
        ),
        ("rule-trivia", vec!["right-associative-local-interleave"]),
    ] {
        let rows: Value = serde_json::from_str(
            &fs::read_to_string(external.join(format!(
                "unlaxer-dsl/src/test/resources/{family}/corpus.json"
            )))
            .unwrap(),
        )
        .unwrap();
        for name in names {
            let row = rows
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["name"] == name)
                .unwrap();
            sources.push((
                name.replace('-', "_"),
                row["grammar"].as_str().unwrap().to_owned(),
            ));
        }
    }
    let mut dag = "grammar Dag { @root Root ::= D28; D0 ::= 'x';".to_owned();
    for n in 1..=28 {
        dag.push_str(&format!("D{n} ::= D{} | D{};", n - 1, n - 1));
    }
    dag.push('}');
    sources.push(("dag".into(), dag));
    sources.push((
        "lexical".into(),
        "grammar G { @root Root ::= 'a' 'b' 'x'; }".into(),
    ));
    // memo の資源上限は「memo が付く文法」でしか踏めない（D-058 で rule ごとに決まるようになった）。
    // 2 候補が同じ位置から A を試すので A は再入し得る。さらに D-065 で「本体が終端走査だけ」の
    // 規則は memo を入れなくなったので、A の本体は別の規則を呼ぶ形にしてある。
    sources.push((
        "memo_budget".into(),
        "grammar G { @root Root ::= ( A 'b' ) | ( A 'c' ); A ::= B; B ::= 'a'; }".into(),
    ));
    for (name, field, body) in [
        ("scalar", "item", "Stmt @item"),
        ("optional", "item", "[Stmt @item]"),
        ("rollback", "item", "(Stmt @item 'x') | 'bad;'"),
        ("memo_recovery", "item", "(Stmt @item 'x') | Stmt @item"),
    ] {
        sources.push((name.into(),format!("grammar G {{ @root @mapping(Root, params=[{field}]) Root ::= {body}; @recovery(sync=';') @mapping(Stmt, params=[value]) Stmt ::= 'ok' @value ';'; }}")));
    }
    sources.push((
        "dfa".into(),
        "grammar Dfa { token T=REGEX('😀+') @root Root ::= T; }".into(),
    ));
    let mut lib = "#![allow(dead_code)]\n".to_owned();
    for (name, source) in sources {
        let path = dir.join(format!("{name}.ubnf"));
        fs::write(&path, &source).unwrap();
        let mut ir = ubnfc_front::pipeline::compile(&path, &|p| fs::read_to_string(p))
            .unwrap_or_else(|e| panic!("{name}: {e}"))
            .ir;
        if name == "dfa" {
            use ubnfc_front::ir::*;
            // front は正則 REGEX に DFA 計画を供給する（leftmost-first）。
            // DFA が無い（javaPattern fallback）IR は Rust backend が明示拒否することを確認する。
            let mut without_dfa = ir.clone();
            without_dfa.plan.lexical[0].strategy = LexicalPlanStrategy::JavaPattern;
            without_dfa.plan.lexical[0].dfa = None;
            assert!(ubnfc_rust::emit(&without_dfa, &Default::default()).is_err());
            // 実行は front が供給した leftmost-first DFA をそのまま使う（token spec と plan の DFA は一致していなければならない）。
            assert!(matches!(
                ir.plan.lexical[0].strategy,
                LexicalPlanStrategy::Dfa
            ));
            assert!(ir.plan.lexical[0].dfa.is_some());
        }
        if name == "enum_plain" {
            for ty in &mut ir.semantics.ast_types {
                if let ubnfc_front::ir::AstType::Enum { variants, .. } = ty {
                    variants.push(variants[0].clone());
                }
            }
        }
        let generated = ubnfc_rust::emit(
            &ir,
            &ubnfc_rust::EmitOptions {
                emit_driver: false,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        generated.write_to(dir.join("src").join(&name)).unwrap();
        lib.push_str(&format!("pub mod {name};\n"));
    }
    let p4=ubnfc_front::ir::from_json(include_str!("../../../ir/frozen/tinyexpression-p4-f86ce8a5.ir.json")).unwrap();
    ubnfc_rust::emit(
        &p4,
        &ubnfc_rust::EmitOptions {
            emit_driver: false,
            ..Default::default()
        },
    )
    .unwrap()
    .write_to(dir.join("src/p4"))
    .unwrap();
    lib.push_str("pub mod p4;\n");
    lib.push_str(include_str!("support/round2_assertions.rs"));
    fs::write(dir.join("src/lib.rs"), lib).unwrap();
    fs::write(dir.join("Cargo.toml"),"[package]\nname=\"round2-contracts\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[features]\ndefault=[\"json\"]\njson=[]\n[dependencies]\nserde=\"1\"\nserde_json=\"1\"\n").unwrap();
    for args in [
        vec!["test", "--offline"],
        vec!["test", "--offline", "--release"],
        vec![
            "clippy",
            "--offline",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    ] {
        let out = Command::new(env!("CARGO"))
            .args(&args)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{args:?} {}\n{}\n{}",
            dir.display(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn generator_checks_recursive_value_types_before_reference_validation() {
    use ubnfc_front::ir::ValueType;
    let mut ir = ubnfc_front::ir::from_json(include_str!("ir/operators.ir.json")).unwrap();
    let mut value = ValueType::Text {};
    for _ in 0..65 {
        value = ValueType::Mixed {
            members: vec![value],
        };
    }
    ir.semantics.captures[0].value_type = value;
    assert!(matches!(
        ubnfc_rust::emit(&ir, &Default::default()),
        Err(ubnfc_rust::EmitError::ResourceLimit {
            resource: "value type depth",
            limit: 64
        })
    ));
}
