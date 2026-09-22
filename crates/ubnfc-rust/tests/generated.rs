use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use ubnfc_front::ir::{from_json, GrammarIr};
use ubnfc_rust::{emit, EmitOptions};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let o = Command::new("mktemp")
            .args(["-d", "/tmp/ubnfc-generated.XXXXXX"])
            .output()
            .unwrap();
        assert!(o.status.success());
        Self(PathBuf::from(String::from_utf8(o.stdout).unwrap().trim()))
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            fs::remove_dir_all(&self.0).unwrap();
        } else {
            eprintln!("generated fixture retained: {}", self.0.display());
        }
    }
}
fn cargo(dir: &Path, args: &[&str]) {
    let out = Command::new(env!("CARGO"))
        .args(args)
        .arg("--offline")
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "cargo {args:?}:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
fn fixture(name: &str) -> GrammarIr {
    from_json(
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/ir")
                .join(format!("{name}.ir.json")),
        )
        .unwrap(),
    )
    .unwrap()
}
fn tiny() -> GrammarIr {
    from_json(include_str!("../../../ir/examples/tinycalc.ir.json")).unwrap()
}

#[test]
fn generated_crates_build_offline_and_match_all_golden() {
    let temp = Temp::new();
    let dir = &temp.0;
    fs::create_dir(dir.join("src")).unwrap();
    let mut grammars = vec![
        ("tiny".to_owned(), tiny()),
        ("operators".to_owned(), fixture("operators")),
    ];
    for mode in ["Text", "Mapped", "Unicode", "Grouped"] {
        let mut ir = fixture("power");
        ir.entry.rule_id = if mode == "Text" {
            "Power::Expr".into()
        } else {
            format!("Power::{mode}Expr")
        };
        grammars.push((mode.to_lowercase(), ir));
    }
    for (name, ir) in &grammars {
        let generated = emit(ir, &EmitOptions::default()).unwrap();
        assert_eq!(generated, emit(ir, &EmitOptions::default()).unwrap());
        generated.write_to(dir.join("src").join(name)).unwrap();
    }
    fs::write(dir.join("Cargo.toml"),"[package]\nname=\"generated-check\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[features]\ndefault=[]\njson=[\"dep:serde\",\"dep:serde_json\"]\n[dependencies]\nserde={version=\"1\",optional=true}\nserde_json={version=\"1\",optional=true}\n").unwrap();
    let mut main = String::from("#![allow(dead_code)]\n");
    for (name, _) in &grammars {
        main.push_str(&format!("mod {name};\n"));
    }
    main.push_str("#[cfg(not(feature=\"json\"))] fn main(){assert!(tiny::parse(\"1+2\").ok);}\n#[cfg(feature=\"json\")] fn main(){use std::io::{self,BufRead};for line in io::stdin().lock().lines(){let value:serde_json::Value=serde_json::from_str(&line.unwrap()).unwrap();let input=value[\"input\"].as_str().unwrap();let recognize=value[\"recognize\"].as_bool().unwrap_or(false);let result=match value[\"grammar\"].as_str().unwrap(){\n");
    for (name, _) in &grammars {
        main.push_str(&format!("{name:?}=>{{let r={name}::parse_with_options(input,{name}::ParseOptions{{build_ast:!recognize,require_eof:true,lexical:false,..Default::default()}});serde_json::json!({{\"ok\":r.ok,\"consumed_cp\":r.consumed_cp,\"matched_cp\":r.matched_cp,\"ast\":r.ast.as_ref().map(|a|a.canonical_value())}})}},\n"));
    }
    main.push_str("_=>panic!(\"grammar\")};println!(\"{}\",result);}}\n");
    fs::write(dir.join("src/main.rs"), main).unwrap();
    cargo(dir, &["build"]);
    cargo(dir, &["run", "--quiet"]);
    cargo(dir, &["build", "--features", "json"]);
    for feature in [false, true] {
        let mut c = Command::new(env!("CARGO"));
        c.args(["clippy", "--offline"]);
        if feature {
            c.args(["--features", "json"]);
        }
        let out = c
            .args(["--", "-D", "warnings"])
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let operators: Value = serde_json::from_str(include_str!(
        "../../../examples/handwritten-target/rust/tests/golden/operators.json"
    ))
    .unwrap();
    let power: Value = serde_json::from_str(include_str!(
        "../../../examples/handwritten-target/rust/tests/golden/power.json"
    ))
    .unwrap();
    let mut cases = Vec::new();
    for row in operators.as_array().unwrap() {
        cases.push(("operators", row.clone()));
    }
    for mode in ["text", "mapped", "unicode", "grouped"] {
        for row in power[mode].as_array().unwrap() {
            cases.push((mode, row.clone()));
        }
    }
    let corpus_root = std::env::var_os("UNLAXER_PARSER_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../unlaxer-parser")
        });
    for (family, rows) in [
        ("associative", operators.as_array().unwrap()),
        ("right-associative", power["text"].as_array().unwrap()),
    ] {
        let corpus: Value = serde_json::from_str(
            &fs::read_to_string(
                corpus_root
                    .join("unlaxer-dsl/src/test/resources")
                    .join(family)
                    .join("corpus.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(corpus.as_array().unwrap().len(), rows.len());
        for (source, golden) in corpus.as_array().unwrap().iter().zip(rows) {
            assert_eq!(source["input"], golden["input"]);
        }
    }
    let mut requests = String::new();
    for (grammar, row) in &cases {
        requests.push_str(&format!(
            "{}\n",
            json!({"grammar":grammar,"input":row["input"]})
        ));
    }
    for (grammar, row) in &cases {
        requests.push_str(&format!(
            "{}\n",
            json!({"grammar":grammar,"input":row["input"],"recognize":true})
        ));
    }
    requests.push_str(&format!("{}\n", json!({"grammar":"tiny","input":"1+2*3"})));
    let mut child = Command::new(dir.join("target/debug/generated-check"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(requests.as_bytes()).unwrap());
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<Value> = result
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), cases.len() * 2 + 1);
    for (i, (grammar, row)) in cases.iter().enumerate() {
        assert_eq!(lines[i], row["result"], "{grammar}: {}", row["input"]);
        let mut expected = row["result"].clone();
        expected["ast"] = Value::Null;
        assert_eq!(
            lines[i + cases.len()],
            expected,
            "recognize {grammar}: {}",
            row["input"]
        );
    }
    let tiny_golden: Value = serde_json::from_str(include_str!("golden/tinycalc.json")).unwrap();
    assert_eq!(lines.last().unwrap(), &tiny_golden);
}

#[test]
fn unsupported_features_are_named_and_invalid_ir_is_rejected() {
    use ubnfc_front::ir::RequiredFeature::*;
    for f in [Recovery, Eval, Catalog, Typeof, LeftRecursionRewrite] {
        let mut ir = tiny();
        ir.required_features.push(f);
        let error = emit(&ir, &Default::default()).unwrap_err().to_string();
        let name = serde_json::to_value(f).unwrap();
        assert!(error.contains(name.as_str().unwrap()), "{error}");
    }
    let mut ir = tiny();
    ir.entry.rule_id = "missing".into();
    assert!(emit(&ir, &Default::default())
        .unwrap_err()
        .to_string()
        .contains("missing"));
}

#[test]
fn cli_writes_the_same_modules_and_names_errors() {
    let temp = Temp::new();
    let path = temp.0.join("input.json");
    fs::write(&path, include_str!("../../../ir/examples/tinycalc.ir.json")).unwrap();
    let output = temp.0.join("generated");
    let result = Command::new(env!("CARGO_BIN_EXE_ubnfc-rust"))
        .arg("--ir")
        .arg(&path)
        .arg("--out")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    for (name, expected) in emit(&tiny(), &Default::default()).unwrap().files {
        assert_eq!(fs::read_to_string(output.join(name)).unwrap(), expected);
    }
    let mut ir = tiny();
    ir.required_features
        .push(ubnfc_front::ir::RequiredFeature::Recovery);
    fs::write(&path, serde_json::to_string(&ir).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_ubnfc-rust"))
        .arg("--ir")
        .arg(&path)
        .arg("--out")
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("recovery"));
}
