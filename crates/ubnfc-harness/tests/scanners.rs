use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use ubnfc_harness::{
    default_root, load_all,
    runner::{select, Artifact, Backend, Report, Runner, Scratch},
};

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn compile_ir(scratch: &Scratch, source: &str) -> PathBuf {
    let grammar = scratch.0.join("scanner.ubnf");
    let ir = scratch.0.join("scanner.ir.json");
    fs::write(&grammar, source).unwrap();
    let output = Command::new(workspace().join("target/debug/ubnfc"))
        .args(["ir", "--grammar"])
        .arg(grammar)
        .arg("--out")
        .arg(&ir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    ir
}

#[test]
fn tiny_string_from_ir_registers_both_backends_without_oracle_rewriting() {
    let scratch = Scratch::new().unwrap();
    let ir = compile_ir(&scratch, "grammar TinyString { token STRING = org.unlaxer.tinyexpression.parser.StringLiteralParser token END = EOF @root @mapping(Value, params=[value]) Root ::= STRING @value END; @mapping(Value, params=[value]) Other ::= STRING @value END; }");
    let cases = select(
        &load_all(default_root()).unwrap(),
        &["tiny-string-token".into()],
        None,
    )
    .unwrap();
    let cases: Vec<_> = cases
        .into_iter()
        // single quote の内容抽出は mapper に残る既知の差分。scanner 登録の E2E は double と拒否を検証。
        .filter(|case| case.variant == "direct-eof" &&
            (case.case.input.starts_with('"') || matches!(&case.case.expect, ubnfc_harness::Expectation::Accept(e) if !e.accepted)))
        .collect();
    assert!(!cases.is_empty());
    for backend in [Backend::Java, Backend::Rust] {
        let runner = Runner {
            backend,
            build_dir: scratch.0.join(format!("{backend:?}")),
        };
        let mut executable = runner
            .build(Artifact::FromIr {
                ir_path: ir.clone(),
                backend,
            })
            .unwrap();
        if backend == Backend::Java {
            let contract = scratch.0.join("RegistryTest.java");
            fs::write(
                &contract,
                include_str!("../../../scanners/java/RegistryTest.java")
                    .replace("generated.", "target."),
            )
            .unwrap();
            let classes = &executable.args[1];
            let output = Command::new("javac")
                .arg("-cp")
                .arg(classes)
                .arg("-d")
                .arg(classes)
                .arg(contract)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let output = Command::new("java")
                .args(["-ea", "-cp"])
                .arg(classes)
                .arg("ubnfc.scanners.RegistryTest")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            // registry を driver の classpath から分離し、指定 directory/jar からのロードを検証する。
            let plugin = scratch.0.join("plugin-classes");
            fs::create_dir(&plugin).unwrap();
            fs::rename(Path::new(classes).join("ubnfc"), plugin.join("ubnfc")).unwrap();
            *executable.args.last_mut().unwrap() = plugin.clone().into_os_string();
            let report = Report::from_actual(&cases, &runner.run(&executable, &cases)).unwrap();
            assert_eq!(report.pass, cases.len(), "{}", report.markdown());
            let jar = scratch.0.join("scanners.jar");
            let output = Command::new("jar")
                .args(["--create", "--file"])
                .arg(&jar)
                .arg("-C")
                .arg(&plugin)
                .arg(".")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            *executable.args.last_mut().unwrap() = jar.clone().into_os_string();
            let output = Command::new("java")
                .args(&executable.args[..3])
                .env("UBNFC_SCANNERS", &jar)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            let source = executable.program.parent().unwrap().join("sources");
            let output = Command::new("cargo")
                .current_dir(source)
                .args(["test", "--offline", "--bin", "driver", "--target-dir"])
                .arg(scratch.0.join("scanner-tests"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let named: Vec<_> = cases
            .iter()
            .cloned()
            .map(|mut case| {
                case.case.entry_rule = Some("Other".into());
                case
            })
            .collect();
        let report = Report::from_actual(&named, &runner.run(&executable, &named)).unwrap();
        assert_eq!(
            report.pass,
            named.len(),
            "{backend:?}: {}",
            report.markdown()
        );
        let actual = runner.run(&executable, &cases);
        let report = Report::from_actual(&cases, &actual).unwrap();
        assert_eq!(
            report.pass,
            cases.len(),
            "{backend:?}: {}",
            report.markdown()
        );
    }
}
