use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};
use ubnfc_front::{ir, pipeline};

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let out = Command::new("mktemp").arg("-d").output().unwrap();
        assert!(out.status.success());
        Self(PathBuf::from(String::from_utf8(out.stdout).unwrap().trim()))
    }
    fn write(&self, path: &str, source: &str) {
        let path = self.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ubnfc"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn code(out: &Output, expected: i32) {
    assert_eq!(
        out.status.code(),
        Some(expected),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
const VALID: &str = "grammar G { @root Root ::= '日本語😀' | 'x'; }";
const WARNING: &str = "grammar G { Root ::= 'x'; }";
const INVALID: &str = "grammar G { @root @mapping(Node, params=[missing]) Root ::= 'x'; }";

#[test]
fn help_version_and_argument_errors() {
    let dir = Sandbox::new();
    for args in [
        vec![],
        vec!["--help"],
        vec!["-h"],
        vec!["ir", "--help"],
        vec!["validate", "-h"],
        vec!["gen", "--help"],
    ] {
        let out = dir.run(&args);
        code(&out, 0);
        assert!(String::from_utf8(out.stdout).unwrap().contains("--grammar"));
        assert!(out.stderr.is_empty());
    }
    let out = dir.run(&["--version"]);
    code(&out, 0);
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("ubnfc {}\n", env!("CARGO_PKG_VERSION"))
    );
    for args in [
        vec!["unknown"],
        vec!["ir"],
        vec!["ir", "--grammar"],
        vec!["ir", "--grammar", "--strict"],
        vec!["ir", "--grammar", "x", "--grammar", "y"],
        vec!["ir", "--grammar", "x", "--strict", "--strict"],
        vec!["ir", "--grammar", "x", "--format", "json"],
        vec!["validate", "--grammar", "x", "--format", "xml"],
        vec!["validate", "--grammar", "x", "--out", "-"],
        vec!["gen", "--grammar", "x", "--target", "go", "--out", "out"],
        vec!["gen", "--grammar", "x", "--target", "rust"],
        vec!["gen", "--grammar", "x", "--target", "java", "--out", "-"],
        vec!["--version", "extra"],
        vec!["--help", "extra"],
        vec!["ir", "--grammar", "x", "extra"],
    ] {
        let out = dir.run(&args);
        code(&out, 2);
        assert!(out.stdout.is_empty(), "{args:?}");
        assert!(!out.stderr.is_empty(), "{args:?}");
    }
}

#[test]
fn ir_stdout_dash_file_and_api_are_byte_identical() {
    let dir = Sandbox::new();
    dir.write("main.ubnf", VALID);
    let default = dir.run(&["ir", "--grammar", "main.ubnf"]);
    code(&default, 0);
    assert!(default.stderr.is_empty());
    let dash = dir.run(&["ir", "--grammar", "./main.ubnf", "--out", "-", "--strict"]);
    code(&dash, 0);
    assert_eq!(dash.stdout, default.stdout);
    let file = dir.run(&["ir", "--grammar", "main.ubnf", "--out", "out.json"]);
    code(&file, 0);
    assert!(file.stdout.is_empty());
    assert_eq!(
        std::fs::read(dir.0.join("out.json")).unwrap(),
        default.stdout
    );
    let compiled =
        pipeline::compile(dir.0.join("main.ubnf"), &|p| std::fs::read_to_string(p)).unwrap();
    assert_eq!(
        ir::to_canonical_json(&compiled.ir).as_bytes(),
        default.stdout
    );
    assert!(ir::validate_references(&compiled.ir).is_empty());
    let output = String::from_utf8(default.stdout).unwrap();
    assert!(output.contains("日本語😀"));
    assert_eq!(ir::from_json(&output).unwrap(), compiled.ir);
}

#[test]
fn validation_text_json_counts_order_and_strict_warnings() {
    let dir = Sandbox::new();
    dir.write("main.ubnf", WARNING);
    let text = dir.run(&["validate", "--grammar", "main.ubnf"]);
    code(&text, 0);
    let issues =
        pipeline::validate_file(dir.0.join("main.ubnf"), &|p| std::fs::read_to_string(p)).unwrap();
    assert_eq!(
        String::from_utf8(text.stdout.clone()).unwrap(),
        format!("{}\n", issues[0].format())
    );
    let strict = dir.run(&["validate", "--grammar", "main.ubnf", "--strict"]);
    code(&strict, 5);
    assert_eq!(text.stdout, strict.stdout);
    let args = [
        "validate",
        "--grammar",
        "main.ubnf",
        "--format",
        "json",
        "--strict",
    ];
    let out = dir.run(&args);
    code(&out, 5);
    assert!(out.stderr.is_empty());
    assert_eq!(out.stdout, dir.run(&args).stdout);
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["severityCounts"], json!({"ERROR":0,"WARNING":1}));
    assert_eq!(v["categoryCounts"], json!({"GENERAL":1}));
    assert_eq!(v["issues"][0]["severity"], "WARNING");
    assert_eq!(v["issues"][0]["code"], "W-GENERAL-NO-ROOT");
    let ir = dir.run(&["ir", "--grammar", "main.ubnf"]);
    code(&ir, 0);
    assert_eq!(ir.stderr, text.stdout);
    let strict = dir.run(&[
        "ir",
        "--grammar",
        "main.ubnf",
        "--strict",
        "--out",
        "strict.json",
    ]);
    code(&strict, 5);
    assert!(strict.stdout.is_empty());
    assert!(!dir.0.join("strict.json").exists());
}

#[test]
fn no_issues_has_empty_text_and_zero_json_counts() {
    let dir = Sandbox::new();
    dir.write("main.ubnf", VALID);
    let out = dir.run(&["validate", "--grammar", "main.ubnf", "--strict"]);
    code(&out, 0);
    assert!(out.stdout.is_empty());
    let out = dir.run(&["validate", "--grammar", "main.ubnf", "--format", "json"]);
    code(&out, 0);
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap(),
        json!({
            "issues": [], "categoryCounts": {}, "severityCounts":{"ERROR":0,"WARNING":0}
        })
    );
}

#[test]
fn validation_errors_precede_strict_warnings_and_do_not_overwrite_ir() {
    let dir = Sandbox::new();
    dir.write(
        "main.ubnf",
        "grammar G { @mapping(Node, params=[missing]) Root ::= 'x'; }",
    );
    dir.write("existing.json", "keep me");
    let out = dir.run(&[
        "validate",
        "--grammar",
        "main.ubnf",
        "--strict",
        "--format",
        "json",
    ]);
    code(&out, 1);
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["severityCounts"], json!({"ERROR":1,"WARNING":1}));
    assert_eq!(v["categoryCounts"], json!({"GENERAL":1,"MAPPING":1}));
    assert_eq!(v["issues"][0]["code"], "W-GENERAL-NO-ROOT");
    assert_eq!(v["issues"][1]["code"], "E-MAPPING-MISSING-CAPTURE");
    let out = dir.run(&[
        "ir",
        "--grammar",
        "main.ubnf",
        "--strict",
        "--out",
        "existing.json",
    ]);
    code(&out, 1);
    assert!(out.stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(dir.0.join("existing.json")).unwrap(),
        "keep me"
    );
}

#[test]
fn syntax_resolve_and_io_errors_use_the_expected_exit_codes() {
    let dir = Sandbox::new();
    for source in [
        "grammar G {",
        "grammar G { @root Root ::= Missing; }",
        "grammar G { @import x from 'missing.ubnf' @root Root ::= x.R; }",
    ] {
        dir.write("main.ubnf", source);
        let out = dir.run(&["validate", "--grammar", "main.ubnf", "--format", "json"]);
        code(&out, 1);
        let value: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(value["severityCounts"]["ERROR"], 1);
        let ir = dir.run(&["ir", "--grammar", "main.ubnf"]);
        code(&ir, 1);
        assert!(ir.stdout.is_empty());
    }
    for cmd in ["ir", "validate"] {
        code(&dir.run(&[cmd, "--grammar", "absent.ubnf"]), 2);
    }
    dir.write("main.ubnf", VALID);
    code(
        &dir.run(&["ir", "--grammar", "main.ubnf", "--out", "absent/out.json"]),
        2,
    );
}

#[test]
fn gen_writes_ir_before_backend_exit_for_both_targets() {
    let dir = Sandbox::new();
    dir.write("main.ubnf", VALID);
    let expected = dir.run(&["ir", "--grammar", "main.ubnf"]);
    code(&expected, 0);
    for target in ["rust", "java"] {
        let missing = dir.0.join("missing backend");
        let variable = if target == "rust" {
            "UBNFC_RUST_BIN"
        } else {
            "UBNFC_JAVA_JAR"
        };
        let out = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
            .current_dir(&dir.0)
            .env(variable, &missing)
            .args([
                "gen",
                "--grammar",
                "main.ubnf",
                "--target",
                target,
                "--out",
                target,
            ])
            .output()
            .unwrap();
        code(&out, 3);
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8(out.stderr)
            .unwrap()
            .contains(missing.to_str().unwrap()));
        assert_eq!(
            std::fs::read(dir.0.join(target).join("grammar.ir.json")).unwrap(),
            expected.stdout
        );
    }
    dir.write("main.ubnf", INVALID);
    code(
        &dir.run(&[
            "gen",
            "--grammar",
            "main.ubnf",
            "--target",
            "rust",
            "--out",
            "invalid",
        ]),
        1,
    );
    assert!(!dir.0.join("invalid").exists());
}

#[test]
fn relative_imports_parent_paths_relocation_and_cwd_do_not_change_bytes() {
    let first = Sandbox::new();
    let relocated = Sandbox::new();
    for dir in [&first, &relocated] {
        dir.write("story/code/main.ubnf", "grammar M { @import a from '../../lib/./one.ubnf' @import b from '../../lib/sub/../one.ubnf' @root Root ::= a.R b.R; }");
        dir.write(
            "lib/one.ubnf",
            "grammar L { @import t from './two.ubnf' R ::= t.T; }",
        );
        dir.write("lib/two.ubnf", "grammar T { token T = NumberParser }");
    }
    let relative = first.run(&["ir", "--grammar", "story/code/main.ubnf", "--out", "-"]);
    code(&relative, 0);
    let absolute = relocated.run(&[
        "ir",
        "--grammar",
        first.0.join("story/code/main.ubnf").to_str().unwrap(),
    ]);
    code(&absolute, 0);
    let moved = relocated.run(&["ir", "--grammar", "story/code/main.ubnf"]);
    code(&moved, 0);
    assert_eq!(relative.stdout, absolute.stdout);
    assert_eq!(relative.stdout, moved.stdout);
    let ir = ir::from_json(std::str::from_utf8(&relative.stdout).unwrap()).unwrap();
    assert_eq!(
        ir.sources
            .iter()
            .map(|s| s.path.as_str())
            .collect::<Vec<_>>(),
        ["story/code/main.ubnf", "lib/one.ubnf", "lib/two.ubnf"]
    );
    assert_eq!(ir.import_edges.len(), 4);
    assert!(ir.sources.iter().all(|s| !Path::new(&s.path).is_absolute()));
}

#[test]
fn repository_markers_preserve_nested_logical_paths_after_relocation() {
    let first = Sandbox::new();
    let moved = Sandbox::new();
    first.write(".git/marker", "");
    moved.write(".git", "gitdir: unused-for-source-identity\n");
    for dir in [&first, &moved] {
        dir.write("grammar/main.ubnf", VALID);
    }
    let a = first.run(&["ir", "--grammar", "grammar/main.ubnf"]);
    let b = moved.run(&["ir", "--grammar", "grammar/main.ubnf"]);
    code(&a, 0);
    code(&b, 0);
    assert_eq!(a.stdout, b.stdout);
    let typed = ir::from_json(std::str::from_utf8(&a.stdout).unwrap()).unwrap();
    assert_eq!(typed.sources[0].path, "grammar/main.ubnf");
}

#[cfg(unix)]
#[test]
fn gen_invokes_both_backends_and_propagates_failure() {
    use std::os::unix::fs::PermissionsExt;
    let dir = Sandbox::new();
    dir.write("main.ubnf", VALID);
    dir.write(
        "fake backend",
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > arguments\nexit \"${BACKEND_STATUS:-0}\"\n",
    );
    let fake = dir.0.join("fake backend");
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    dir.write("generator jar.jar", "placeholder");
    std::fs::copy(&fake, dir.0.join("java")).unwrap();
    for target in ["rust", "java"] {
        for status in [0, 7] {
            let output = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
                .current_dir(&dir.0)
                .env("UBNFC_RUST_BIN", "fake backend")
                .env("UBNFC_JAVA_JAR", dir.0.join("generator jar.jar"))
                .env("PATH", &dir.0)
                .env("BACKEND_STATUS", status.to_string())
                .args([
                    "gen",
                    "--grammar",
                    "main.ubnf",
                    "--target",
                    target,
                    "--out",
                    "generated sources",
                ])
                .output()
                .unwrap();
            code(&output, status);
            let arguments = std::fs::read_to_string(dir.0.join("arguments")).unwrap();
            let mut expected = Vec::new();
            let jar = dir.0.join("generator jar.jar");
            if target == "java" {
                expected.extend(["-cp", jar.to_str().unwrap(), "org.ubnfc.java.Main"]);
            }
            expected.extend(["--ir", "generated sources/grammar.ir.json", "--out"]);
            if target == "java" {
                expected.extend(["generated sources/src/main/java", "--package", "target"]);
            } else {
                expected.push("generated sources");
            }
            expected.push("--emit-driver");
            assert_eq!(arguments.lines().collect::<Vec<_>>(), expected);
            assert!(dir.0.join("generated sources/grammar.ir.json").is_file());
        }
    }
}

#[test]
fn gen_rust_discovers_adjacent_generator_and_builds_driver() {
    let dir = Sandbox::new();
    dir.write("main.ubnf", VALID);
    let out = Command::new(env!("CARGO_BIN_EXE_ubnfc"))
        .current_dir(&dir.0)
        .env_remove("UBNFC_RUST_BIN")
        .args([
            "gen",
            "--grammar",
            "main.ubnf",
            "--target",
            "rust",
            "--out",
            "generated",
        ])
        .output()
        .unwrap();
    code(&out, 0);
    assert!(dir.0.join("generated/src/bin/driver.rs").is_file());
    let build = Command::new(env!("CARGO"))
        .current_dir(dir.0.join("generated"))
        .args(["build", "--offline", "--release"])
        .output()
        .unwrap();
    code(&build, 0);
}
