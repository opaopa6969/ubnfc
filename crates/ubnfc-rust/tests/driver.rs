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
        json!({"kind":"trailing_input","offset":1,"expected":["end of input"],"farthestOffset":2,"farthestExpected":["'x'"],"origin":"Mapper.diagnose","deepestRule":"Operators::Root","rulePath":["Operators::Root"]})
    );
}

#[test]
fn round3_evaluation_and_retained_observations_follow_mapped_values() {
    let temp = Temp::new();
    let grammar = temp.0.join("round3.ubnf");
    fs::write(&grammar, r#"
        grammar Power {
            token NUMBER=NumberParser
            @root @mapping(Power, params=[left,op,right]) Root ::= NumericLiteral @left ['^' @op Root @right];
            @mapping(Number, params=[value]) NumericLiteral ::= NUMBER @value;
        }
        grammar Mixed {
            @root @mapping(Root, params=[head,maybe,items]) Document ::= Factor @head ':' [Factor @maybe] ':' {Factor @items};
            Factor ::= 'a' | LeafRule;
            @mapping(Leaf, params=[text]) LeafRule ::= 'x' @text;
        }
        grammar Semantic {
            @root @mapping(Box, params=[values]) Root ::= {Factor @values};
            Factor ::= 'a' | LeafRule;
            @mapping(Leaf, params=[text]) LeafRule ::= 'x' @text;
        }
        grammar Cardinality {
            token NUMBER=NumberParser
            @root @mapping(Container, params=[head,values,tail,flags]) Root ::= [Item] @head ':' {Item} @values ':' ['!'] @tail ':' {'x'} @flags;
            @mapping(Item, params=[value]) Item ::= NUMBER @value ';';
        }
        grammar Quoted {
            token TEXT=org.unlaxer.tinyexpression.parser.StringLiteralParser
            @root @mapping(Value, params=[value]) Root ::= TEXT @value;
        }
        grammar Alias {
            @root @mapping(Box, params=[values]) Root ::= Helper @values;
            Helper ::= '(' [Factor] ')';
            Factor ::= 'a' | Leaf;
            @mapping(Leaf, params=[text]) Leaf ::= 'x' @text;
        }
        grammar RepeatAlias {
            @root @mapping(Box, params=[values]) Root ::= Helper @values;
            Helper ::= { '(' Factor ')' };
            Factor ::= 'a' | Leaf;
            @mapping(Leaf, params=[text]) Leaf ::= 'x' @text;
        }
        grammar Look {
            token T=SingleQuotedParser token L=LOOKAHEAD('a') token N=LOOKAHEAD('b')
            @root @mapping(Value, params=[value]) Root ::= L [T] N 'ab' @value;
        }
        grammar Backref {
            token ID=IdentifierParser
            @root @backref(name=name) @mapping(Pair, params=[name]) Root ::= ID @name ':' ID @name;
        }
        grammar Separated {
            @root @mapping(Box, params=[values]) Root ::= '[' Items @values ']';
            Items ::= Factor % ',';
            Factor ::= 'a' | '😀' | Leaf;
            @mapping(Leaf, params=[text]) Leaf ::= 'x' @text;
        }
        grammar GroupValue {
            @root @mapping(Root, params=[value]) Root ::= ('a' | Leaf) @value;
            @mapping(Leaf, params=[text]) Leaf ::= 'x' @text;
        }
        grammar TokenList {
            @whitespace: javaStyle
            token T=ANY
            @root @mapping(Value, params=[value]) Root ::= (T | 'z')+ @value;
        }
        grammar GroupList {
            @whitespace: javaStyle
            token T=ANY
            @root @mapping(Value, params=[value]) Root ::= ((T | 'z'))+ @value;
        }
        grammar Nested {
            @root @mapping(Root, params=[first,second]) Root ::= Leaf @first Leaf @second;
            @mapping(Leaf, params=[text]) Leaf ::= 'x' @text;
        }
    "#).unwrap();
    let ir = ubnfc_front::pipeline::compile(&grammar, &|p| fs::read_to_string(p))
        .unwrap()
        .ir;
    let generated = temp.0.join("generated");
    ubnfc_rust::emit(&ir, &Default::default())
        .unwrap()
        .write_to(&generated)
        .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for file in ["scanners.rs", "scanners_tests.rs"] {
        fs::copy(
            root.join("scanners/rust").join(file),
            generated.join("src/bin").join(file),
        )
        .unwrap();
    }
    cargo(
        &generated,
        &["build", "--offline", "--features", "scanners"],
    );
    let requests = [
        ("Power", "2^3^2", json!({"family":"right-associative"})),
        (
            "Mixed",
            "a::aa",
            json!({"family":"mixed-values", "valueSpanField":"head", "valueSpanListField":"items"}),
        ),
        ("Mixed", "x:a:ax", json!({"family":"mixed-values"})),
        ("Semantic", "axa", json!({"family":"semantic-cardinality"})),
        ("Cardinality", ":::", json!({"family":"cardinality"})),
        (
            "Cardinality",
            "1;:2;3;:!:xxx",
            json!({"family":"cardinality"}),
        ),
        ("Quoted", "'😀'", json!({"family":"oracle"})),
        ("Quoted", "'😀'!", json!({"family":"oracle"})),
        (
            "Nested",
            "xx",
            json!({"captureNames":["text","first","second"]}),
        ),
        ("Alias", "(a)", json!({"family":"semantic-cardinality"})),
        ("Alias", "()", json!({"family":"semantic-cardinality"})),
        (
            "RepeatAlias",
            "(a)(x)(a)",
            json!({"family":"semantic-cardinality"}),
        ),
        ("RepeatAlias", "", json!({"family":"semantic-cardinality"})),
        ("Look", "ab", json!({})),
        ("Quoted", "' \0 '", json!({})),
        ("Backref", "x:x", json!({})),
        ("Backref", "x:y", json!({})),
        (
            "Separated",
            "[a,x,😀]",
            json!({"family":"semantic-cardinality"}),
        ),
        ("GroupValue", "a", json!({"family":"mixed-values"})),
        ("TokenList", "a /*c*/b", json!({})),
        ("GroupList", "a /*c*/b", json!({})),
    ];
    let lines: String = requests.iter().enumerate().map(|(id, (grammar, input, observations))|
        format!("{}\n", json!({"id":id,"grammar":grammar,"entry":null,"input":input,"options":{"lexical":true,"scope":false},"observations":observations}))
    ).collect();
    let rows = exchange(&generated.join("target/debug/driver"), &lines);
    assert_eq!(rows[0]["value"], 512.0, "{rows:?}");
    assert_eq!(rows[1]["value"], "T:a|-|T:a,T:a");
    assert_eq!(
        rows[1]["texts"],
        json!([[0, 1, "a"], [3, 4, "a"], [4, 5, "a"]])
    );
    assert_eq!(rows[1]["valueSpans"], json!([[0, 1], [3, 4], [4, 5]]));
    assert_eq!(rows[1]["retainedValueSpans"], rows[1]["valueSpans"]);
    assert_eq!(rows[2]["value"], "L:x|T:a|T:a,L:x");
    assert_eq!(rows[2]["nodes"], json!([[0, 1, "x"], [5, 6, "x"]]));
    assert_eq!(rows[3]["value"], "T:a,L:x,T:a");
    assert_eq!(rows[4]["value"], 0.0);
    assert_eq!(rows[4]["ast"]["fields"]["tail"], Value::Null);
    assert_eq!(rows[4]["ast"]["fields"]["flags"], json!([]));
    assert_eq!(rows[5]["value"], 10.0);
    assert_eq!(rows[5]["ast"]["fields"]["flags"], json!(["x", "x", "x"]));
    assert_eq!(rows[6]["ast"]["fields"]["value"], "😀");
    assert_eq!(rows[6]["texts"], json!([]));
    assert_eq!(
        rows[6]["captures"][0],
        json!({"binding":"Root:0","name":"value","span":[0,3],"text":"'😀'"})
    );
    assert_eq!(rows[7]["accepted"], true);
    assert_eq!(rows[7]["allConsumed"], false);
    assert_eq!(rows[7]["prefixCaptures"], rows[6]["captures"]);
    assert_eq!(
        rows[7]["prefixCaptures"][0],
        json!({
            "binding":"Root:0","name":"value","span":[0,3],"text":"'😀'"
        })
    );
    let names: Vec<_> = rows[8]["captures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["text", "text", "first", "second"]);
    assert_eq!(rows[9]["value"], "T:(a)");
    assert_eq!(rows[9]["texts"], json!([[0, 3, "(a)"]]));
    assert_eq!(rows[10]["value"], "");
    assert_eq!(rows[10]["texts"], json!([]));
    assert_eq!(rows[11]["value"], "T:(a),L:x,T:(a)");
    assert_eq!(rows[11]["texts"], json!([[0, 3, "(a)"], [6, 9, "(a)"]]));
    assert_eq!(rows[12]["value"], "");
    assert_eq!(rows[13]["accepted"], true);
    assert_eq!(rows[13]["matched"], 2);
    assert_eq!(rows[14]["ast"]["fields"]["value"], " \0 ");
    assert_eq!(rows[15]["diagnostics"], json!([]));
    assert_eq!(
        rows[16]["diagnostics"],
        json!([{"message":"back-reference mismatch: expected 'x' but got 'y'", "offset":2,"length":1,"severity":"ERROR","kind":"ERROR","expected":[],"origin":"ScopeStore"}])
    );
    assert_eq!(rows[17]["value"], "T:a,L:x,T:😀");
    assert_eq!(rows[17]["texts"], json!([[1, 2, "a"], [5, 6, "😀"]]));
    assert_eq!(rows[18]["value"], "T:a");
    assert_eq!(rows[19]["ast"]["fields"]["value"], json!(["a", "b"]));
    assert_eq!(rows[20]["ast"]["fields"]["value"], json!(["a /*c*/", "b"]));
    cargo(
        &generated,
        &[
            "clippy",
            "--offline",
            "--all-targets",
            "--features",
            "scanners",
            "--",
            "-D",
            "warnings",
        ],
    );
}

#[test]
fn farthest_candidates_match_independent_evolution_oracle_with_and_without_memo() {
    // Java r3 と同じ独立 oracle。主 offset は D-014 の Rust fail_at を維持する。
    let oracle: Value =
        serde_json::from_str(include_str!("../../../corpus/oracle/evolution-v3.json")).unwrap();
    let temp = Temp::new();
    let source = temp.0.join("Evolution.ubnf");
    fs::write(&source, oracle["grammar"].as_str().unwrap()).unwrap();
    let ir = ubnfc_front::pipeline::compile(&source, &|p| fs::read_to_string(p))
        .unwrap()
        .ir;
    let generated = temp.0.join("generated");
    ubnfc_rust::emit(
        &ir,
        &ubnfc_rust::EmitOptions {
            emit_driver: false,
            ..Default::default()
        },
    )
    .unwrap()
    .write_to(&generated)
    .unwrap();
    let mut assertions = String::from("#[cfg(test)] mod round5 { use super::*; #[test] fn independent_oracle() { for memo in [false,true] {\n");
    for row in oracle["cases"].as_array().unwrap() {
        let Some(d) = row["diagnostics"].get(0) else {
            continue;
        };
        let input = row["input"].as_str().unwrap();
        let expected: Vec<_> = d["farthestExpected"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let offset = d["farthestOffset"].as_u64().unwrap();
        assertions.push_str(&format!("let r=parse_with_options({input:?},ParseOptions{{memo,..Default::default()}});assert!(!r.ok);assert_eq!(r.diagnostics[0].farthest_cp,{offset},{input:?});assert_eq!(r.diagnostics[0].farthest_expected,vec!{expected:?},{input:?});\n"));
    }
    assertions.push_str("}}}\n");
    let mut root = fs::read_to_string(generated.join("mod.rs")).unwrap();
    root.push_str(&assertions);
    fs::write(generated.join("mod.rs"), root).unwrap();
    cargo(&generated, &["test", "--offline"]);
}
