use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let out = Command::new("mktemp").arg("-d").output().unwrap();
        assert!(out.status.success());
        Self(PathBuf::from(String::from_utf8(out.stdout).unwrap().trim()))
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn cargo(dir: &Path, args: &[&str]) {
    let out = Command::new(env!("CARGO"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
fn exchange(binary: &Path, lines: &str) -> Vec<Value> {
    let mut child = Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let output = std::thread::scope(|scope| {
        scope.spawn(move || input.write_all(lines.as_bytes()).unwrap());
        child.wait_with_output().unwrap()
    });
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
#[test]
fn generated_driver_is_dependency_free_and_preserves_protocol_and_golden_ast() {
    let temp = Temp::new();
    let ir = ubnfc_front::ir::from_json(include_str!("ir/operators.ir.json")).unwrap();
    ubnfc_rust::emit(&ir, &Default::default())
        .unwrap()
        .write_to(&temp.0)
        .unwrap();
    cargo(&temp.0, &["build", "--release", "--offline"]);
    let metadata = Command::new(env!("CARGO"))
        .current_dir(&temp.0)
        .args(["metadata", "--offline", "--format-version=1"])
        .output()
        .unwrap();
    assert!(metadata.status.success());
    let metadata: Value = serde_json::from_slice(&metadata.stdout).unwrap();
    assert_eq!(metadata["resolve"]["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(metadata["resolve"]["nodes"][0]["deps"], json!([]));
    let golden: Value = serde_json::from_str(include_str!(
        "../../../examples/handwritten-target/rust/tests/golden/operators.json"
    ))
    .unwrap();
    let request = |id: Value, input: &str| json!({"id":id,"grammar":"Operators","entry":"Expression","input":input,"options":{"lexical":true,"scope":true}});
    let mut lines = String::new();
    for (i, row) in golden.as_array().unwrap().iter().enumerate() {
        lines.push_str(&format!(
            "{}\n",
            request(json!(i), row["input"].as_str().unwrap())
        ));
    }
    lines.push_str("not json\n");
    lines.push_str(&format!(
        "{}\n",
        request(json!([9007199254740993u64,"😀",{"x":null}]), "2+3")
    ));
    lines.push_str(&format!("{}\n", request(json!("nonfinite"), "1/0")));
    lines.push_str(&format!("{}\n", request(Value::Null, "2/*😀*/+3\n*4")));
    let rows = exchange(&temp.0.join("target/release/driver"), &lines);
    for (i, row) in golden.as_array().unwrap().iter().enumerate() {
        assert_eq!(rows[i]["ast"], row["result"]["ast"], "{}", row["input"]);
        assert_eq!(rows[i]["allConsumed"], row["result"]["ok"]);
        assert_eq!(rows[i]["consumed"], row["result"]["consumed_cp"]);
        assert_eq!(rows[i]["matched"], row["result"]["matched_cp"]);
    }
    let i = golden.as_array().unwrap().len();
    assert_eq!(rows[i]["id"], Value::Null);
    assert!(rows[i]["error"].is_string());
    assert_eq!(
        rows[i + 1]["id"],
        json!([9007199254740993u64,"😀",{"x":null}])
    );
    assert_eq!(rows[i + 1]["value"], 5.0);
    assert!(rows[i + 2]["error"]
        .as_str()
        .unwrap()
        .contains("non-finite"));
    assert_eq!(rows[i + 3]["value"], 14.0);
    for feature in [false, true] {
        let mut args = vec!["clippy", "--offline", "--all-targets"];
        if feature {
            args.extend(["--features", "json"]);
        }
        args.extend(["--", "-D", "warnings"]);
        cargo(&temp.0, &args);
    }
}
#[test]
fn cli_driver_flags_and_explicit_disable() {
    let temp = Temp::new();
    for (name, flags, enabled) in [
        ("default", vec![], true),
        ("on", vec!["--emit-driver"], true),
        ("false", vec!["--emit-driver", "false"], false),
        ("off", vec!["--no-emit-driver"], false),
    ] {
        let dir = temp.0.join(name);
        let out = Command::new(env!("CARGO_BIN_EXE_ubnfc-rust"))
            .args([
                "--ir",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ir/operators.ir.json"),
                "--out",
            ])
            .arg(&dir)
            .args(flags)
            .output()
            .unwrap();
        assert!(out.status.success());
        assert_eq!(dir.join("src/bin/driver.rs").exists(), enabled);
        assert_eq!(
            fs::read_to_string(dir.join("Cargo.toml"))
                .unwrap()
                .contains("[[bin]]"),
            enabled
        );
        if !enabled {
            cargo(&dir, &["check", "--offline"]);
        }
    }
    let out = Command::new(env!("CARGO_BIN_EXE_ubnfc-rust"))
        .args(["--emit-driver", "maybe"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn dispatches_each_grammar_root_and_reports_mapping_without_losing_acceptance() {
    let temp = Temp::new();
    let grammar = temp.0.join("multi.ubnf");
    fs::write(
        &grammar,
        "grammar First { @root One ::= 'a'; } grammar Second { @root @skip Two ::= 'b'; }",
    )
    .unwrap();
    let ir = ubnfc_front::pipeline::compile(&grammar, &|p| fs::read_to_string(p))
        .unwrap()
        .ir;
    let generated = temp.0.join("generated");
    ubnfc_rust::emit(&ir, &Default::default())
        .unwrap()
        .write_to(&generated)
        .unwrap();
    cargo(&generated, &["build", "--offline"]);
    let mut lines = String::new();
    for (grammar, entry, input) in [
        ("First", Value::Null, "a"),
        ("Second", Value::Null, "b"),
        ("First", json!("One"), "a"),
        ("Second", json!("One"), "a"),
        ("Unknown", Value::Null, "b"),
    ] {
        lines.push_str(&format!("{}\n", json!({"id":grammar,"grammar":grammar,"entry":entry,"input":input,"options":{"lexical":false,"scope":false}})));
    }
    let rows = exchange(&generated.join("target/debug/driver"), &lines);
    for row in &rows[..3] {
        assert_eq!(row["accepted"], true);
        assert_eq!(row["allConsumed"], true);
        if row["id"] == "Second" {
            assert_eq!(row["ast"], Value::Null);
            assert!(row["mappingError"].is_string());
        } else {
            assert_eq!(row["ast"], "a");
            assert_eq!(row["mappingError"], Value::Null);
        }
        assert!(row.get("captures").is_none());
        assert!(row.get("declarations").is_none());
        assert!(row.get("references").is_none());
    }
    for row in &rows[3..] {
        assert!(row["error"].is_string());
    }
}

#[path = "support/fixture.rs"]
#[allow(dead_code)]
mod fixture;

#[test]
fn scope_events_and_farthest_diagnostics_are_public_protocol_fields() {
    use ubnfc_front::ir::*;
    let temp = Temp::new();
    let mut ir = fixture::grammar(
        vec![(
            "Root",
            fixture::choice(
                vec![
                    fixture::seq(vec![
                        fixture::literal("a"),
                        fixture::literal("b"),
                        fixture::literal("x"),
                    ]),
                    fixture::literal("a"),
                ],
                "ordered",
            ),
        )],
        None,
    );
    ir.semantics.scope.effects = vec![ScopeEffect {
        rule_id: "Operators::Root".into(),
        mode: ScopeEffectMode::Dynamic,
        events: vec![
            ScopeEvent {
                action: ScopeEffectEventsAction::Enter,
                when: ScopeEffectEventsWhen::Entry,
                capture_site_id: None,
                name: None,
            },
            ScopeEvent {
                action: ScopeEffectEventsAction::Leave,
                when: ScopeEffectEventsWhen::SuccessBeforeLeave,
                capture_site_id: None,
                name: None,
            },
        ],
    }];
    ubnfc_rust::emit(&ir, &Default::default())
        .unwrap()
        .write_to(&temp.0)
        .unwrap();
    cargo(&temp.0, &["build", "--offline"]);
    let request = |input, scope| {
        json!({"id":0,"grammar":"Operators","entry":"Root","input":input,"options":{"lexical":false,"scope":scope}}).to_string()+"\n"
    };
    let rows = exchange(
        &temp.0.join("target/debug/driver"),
        &(request("a", true) + &request("a", false) + &request("abq", true)),
    );
    assert_eq!(
        rows[0]["scope"],
        json!({"mode":["dynamic"],"events":[
            {"order":0,"action":"enter","mode":"dynamic","name":null,"offset":0,"length":0},
            {"order":1,"action":"leave","mode":"dynamic","name":null,"offset":1,"length":0}
        ]})
    );
    assert!(rows[1].get("scope").is_none());
    assert_eq!(
        rows[2]["diagnostics"][0],
        json!({"kind":"trailing_input","offset":1,"expected":["end of input"],"farthestOffset":2,"farthestExpected":["x"],"deepestRule":"Operators::Root","rulePath":["Operators::Root"]})
    );
}
