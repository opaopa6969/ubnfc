//! Shared UBNF corpus -> IR -> standalone Java 17 / Rust programs (no scanner bindings).
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn java_files(path: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            java_files(&path, out);
        } else if path.extension().is_some_and(|s| s == "java") {
            out.push(path);
        }
    }
}

#[test]
fn shared_lexical_corpus_runs_in_both_standalone_backends() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let java_classes = repo.join("ubnfc-java/target/classes");
    assert!(
        java_classes
            .join("org/unlaxer/ubnfc/java/Main.class")
            .exists(),
        "build the Java generator first: mvn -q -f ubnfc-java/pom.xml -DskipTests package"
    );
    let created = success(
        Command::new("mktemp")
            .args(["-d", "/tmp/ubnfc-declarative.XXXXXX"])
            .output()
            .unwrap(),
    );
    let work = PathBuf::from(String::from_utf8(created.stdout).unwrap().trim());
    // Retain failures for diagnosis. Every generated output is isolated in this unique directory.
    eprintln!("declarative conformance artifacts: {}", work.display());
    let corpus = repo.join("fixtures/spec-corpus");
    let source = fs::read_to_string(corpus.join("declarative-tokens/tokens.ubnf")).unwrap();
    let fixtures: Value = serde_json::from_str(
        &fs::read_to_string(corpus.join("declarative-tokens/runtime.json")).unwrap(),
    )
    .unwrap();
    let mut inputs = vec![];
    for fixture in fixtures.as_array().unwrap() {
        let mut text = source.replace(
            "Root ::= NUMBER @value;",
            &format!("Root ::= {};", fixture["body"].as_str().unwrap()),
        );
        if fixture.get("whitespace").is_some() {
            text = text.replace(
                "@package: lexical.probe",
                "@package: lexical.probe\n @whitespace: javaStyle",
            );
        }
        let path = work.join(format!("{}.ubnf", fixture["name"].as_str().unwrap()));
        fs::write(&path, text).unwrap();
        inputs.push((path, fixture["cases"].clone()));
    }
    inputs.push((
        corpus.join("lexical-modules/root.ubnf"),
        serde_json::from_str(
            &fs::read_to_string(corpus.join("lexical-modules/runtime.json")).unwrap(),
        )
        .unwrap(),
    ));
    let mut rust = String::from("#![allow(dead_code,unused_imports)]\n");
    let mut rust_body = String::from("fn main() {\n");
    let mut java = String::from(
        "public class Probe {public static void main(String[] args) throws Exception {\n",
    );
    for (i, (path, cases)) in inputs.iter().enumerate() {
        let compiled = ubnfc_front::pipeline::compile(path, &|p: &Path| fs::read_to_string(p))
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let ir_path = work.join(format!("g{i}.json"));
        fs::write(&ir_path, ubnfc_front::ir::to_canonical_json(&compiled.ir)).unwrap();
        let generated = ubnfc_rust::emit(
            &compiled.ir,
            &ubnfc_rust::EmitOptions {
                emit_driver: false,
                ..Default::default()
            },
        )
        .unwrap();
        generated.write_to(work.join(format!("g{i}"))).unwrap();
        rust.push_str(&format!("mod g{i};\n"));
        success(
            Command::new("java")
                .arg("-cp")
                .arg(&java_classes)
                .args(["org.unlaxer.ubnfc.java.Main", "--ir"])
                .arg(&ir_path)
                .arg("--out")
                .arg(work.join("java"))
                .args(["--package", &format!("probe.g{i}"), "--no-emit-driver"])
                .output()
                .unwrap(),
        );
        for row in cases.as_array().unwrap() {
            let input = row["input"].as_str().unwrap();
            let quoted = serde_json::to_string(input).unwrap();
            let p = row["prefix"].as_array().unwrap();
            let (ok, c, m) = (
                p[0].as_bool().unwrap(),
                p[1].as_u64().unwrap(),
                p[2].as_u64().unwrap(),
            );
            rust_body.push_str(&format!("{{let r=g{i}::parse_with_options({input:?},g{i}::ParseOptions{{require_eof:false,..Default::default()}});assert_eq!((r.ok,r.consumed_cp,r.matched_cp),({ok},{c},{m}),\"case g{i}: {{:?}}\",{input:?});}}\n"));
            java.push_str(&format!("{{var r=probe.g{i}.LexicalProbeParser.parse({quoted},new probe.g{i}.api.ParseOptions(true,false,false,true,java.util.Map.of()));if(r.ok()!={ok}||r.consumedCp()!={c}||r.matchedCp()!={m})throw new AssertionError(\"g{i}: \"+{quoted}+r);}}\n"));
            let accepted = row.get("value").is_some();
            rust_body.push_str(&format!("{{let r=g{i}::parse({input:?});assert_eq!(r.ok,{accepted},\"full g{i}: {{:?}}\",{input:?});"));
            java.push_str(&format!("{{var r=probe.g{i}.LexicalProbeParser.parse({quoted});if(r.ok()!={accepted})throw new AssertionError(\"full g{i}: \"+{quoted}+r);"));
            if row.get("value").is_some() {
                // Recognition is shared. Keep ubnfc's existing text-field trim,
                // but do not give a declarative token the old quoted scanner's unquote effect.
                let value = input.trim_matches(|c: char| c <= '\u{20}');
                let end = input.chars().count();
                rust_body.push_str(&format!("let g{i}::Ast::g_LexicalProbeAST_2e_Value(v)=r.ast.unwrap() else {{panic!(\"Value\")}};assert_eq!(v.g_value,{value:?});assert_eq!(v.span,[0,{end}]);"));
                let value = serde_json::to_string(value).unwrap();
                java.push_str(&format!("var ast=r.ast().orElseThrow();if(!ast.getClass().getMethod(\"value\").invoke(ast).equals({value}))throw new AssertionError(ast);var span=r.nodeSpans().get(ast);if(span.start()!=0||span.end()!={end})throw new AssertionError(span);"));
            }
            rust_body.push_str("}\n");
            java.push_str("}\n");
        }
    }
    rust.push_str(&rust_body);
    rust.push_str("}\n");
    // Rust str cannot contain an isolated UTF-16 surrogate. Java String can,
    // but ANY / ranges / complements must still recognize Unicode scalars only.
    java.push_str("for(char c:new char[]{(char)0xd800,(char)0xdfff}){String s=String.valueOf(c);for(var op:new probe.g0.internal.LexicalExpression.Op[]{probe.g0.internal.LexicalExpression.Op.ANY,probe.g0.internal.LexicalExpression.Op.RANGE,probe.g0.internal.LexicalExpression.Op.EXCEPT}){var p=new probe.g0.internal.LexicalExpression(op,\"\",0,0x10ffff,java.util.List.of());if(p.match(s,0)!=-1)throw new AssertionError(op);}}\n");
    java.push_str("}}\n");
    fs::write(work.join("main.rs"), rust).unwrap();
    fs::write(work.join("java/Probe.java"), java).unwrap();
    success(
        Command::new("rustc")
            .args(["--edition=2021", "-Awarnings"])
            .arg(work.join("main.rs"))
            .arg("-o")
            .arg(work.join("probe"))
            .output()
            .unwrap(),
    );
    success(Command::new(work.join("probe")).output().unwrap());
    let mut sources = vec![];
    java_files(&work.join("java"), &mut sources);
    sources.sort();
    fs::create_dir(work.join("classes")).unwrap();
    success(
        Command::new("javac")
            .args(["--release", "17", "-Xlint:all", "-Werror", "-d"])
            .arg(work.join("classes"))
            .args(sources)
            .output()
            .unwrap(),
    );
    success(
        Command::new("java")
            .arg("-cp")
            .arg(work.join("classes"))
            .arg("Probe")
            .output()
            .unwrap(),
    );
    fs::remove_dir_all(&work).unwrap();
}
