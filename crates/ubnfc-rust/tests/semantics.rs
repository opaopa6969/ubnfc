use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::Command};
use ubnfc_front::ir::{from_json, GrammarIr};
use ubnfc_rust::emit;

fn literal(value: &str) -> Value {
    json!({"kind":"literal","value":value,"caseSensitive":true,"wordBoundary":false})
}
fn seq(items: Vec<Value>) -> Value {
    json!({"kind":"seq","items":items})
}
fn choice(alternatives: Vec<Value>, selection: &str) -> Value {
    json!({"kind":"choice","alternatives":alternatives,"selection":selection,"tieBreak":"declarationOrder"})
}
fn unary(kind: &str, child: Value) -> Value {
    json!({"kind":kind,"child":child})
}
fn repeat(child: Value, min: u64, max: u64) -> Value {
    json!({"kind":"repeat","child":child,"min":min,"max":max,"terminator":null})
}
fn reference(name: &str) -> Value {
    json!({"kind":"ruleRef","symbolId":format!("Operators::{name}")})
}
fn token() -> Value {
    json!({"kind":"tokenRef","symbolId":"Operators::NUMBER"})
}
fn capture(name: &str, body: Value) -> Value {
    json!({"capture":name,"body":body})
}

fn flatten(j: &mut Value, mut node: Value, rule: &str, role: &str) -> String {
    let mut captured = Vec::new();
    while let Some(name) = node.get("capture").and_then(Value::as_str) {
        captured.push(name.to_owned());
        node = node["body"].take();
    }
    let source = j["sources"][0].clone();
    let id = format!(
        "expr:{}:0:{}:{rule}/{role}",
        source["id"].as_str().unwrap(),
        source["codePointLength"]
    );
    let kind = node["kind"].as_str().unwrap().to_owned();
    let transaction = match kind.as_str() {
        "seq" | "lookahead" | "negativeLookahead" | "separated" => "once",
        "choice" => "perAlternative",
        "repeat" | "optional" => "occurs",
        "group" | "ruleRef" | "tokenRef" => "delegate",
        _ => "none",
    };
    let e = json!({"id":id,"node":{},"ruleId":format!("Operators::{rule}"),"source":{"sourceId":source["id"],"start":0,"end":source["codePointLength"]},"captureSiteIds":[],"diagnosticLabel":null,"execution":{"childMode":"inherit","entryFlag":if kind=="seq"{"setFalse"}else{"preserve"},"failureCursor":if transaction=="none"{"primitiveMayResetMatched"}else if transaction=="delegate"{"delegate"}else{"restoreTransaction"},"primitiveProfile":null,"transaction":transaction}});
    let index = j["semantics"]["expressions"].as_array().unwrap().len();
    j["semantics"]["expressions"]
        .as_array_mut()
        .unwrap()
        .push(e);
    let mut analysis: Value = serde_json::from_str::<Value>(include_str!("ir/operators.ir.json"))
        .unwrap()["analysis"]["expressions"][0]
        .clone();
    analysis["exprId"] = json!(id);
    j["analysis"]["expressions"]
        .as_array_mut()
        .unwrap()
        .push(analysis);
    for key in ["items", "alternatives"] {
        if let Some(items) = node.get_mut(key) {
            let children = items.take();
            let mut ids = vec![];
            for (i, child) in children.as_array().unwrap().iter().enumerate() {
                ids.push(flatten(j, child.clone(), rule, &format!("{role}/{i}")));
            }
            node[key] = json!(ids);
        }
    }
    for (i, key) in ["child", "element", "separator", "terminator"]
        .iter()
        .enumerate()
    {
        if node.get(*key).is_some_and(Value::is_object) {
            let child = node[*key].take();
            node[*key] = json!(flatten(j, child, rule, &format!("{role}/child{i}")));
        }
    }
    j["semantics"]["expressions"][index]["node"] = node;
    for (capture_index, name) in captured.into_iter().rev().enumerate() {
        let site = format!("{id}/capture/{capture_index}");
        j["semantics"]["expressions"][index]["captureSiteIds"]
            .as_array_mut()
            .unwrap()
            .push(json!(site));
        j["semantics"]["captures"].as_array_mut().unwrap().push(json!({"siteId":site,"exprId":id,"ruleId":format!("Operators::{rule}"),"name":name,"cardinality":"scalar","conversion":"text","occurrenceOrder":"completionInnerBeforeOuter","skip":false,"spanRule":"consumedExtent","valueType":{"kind":"text"}}));
    }
    id
}
fn grammar(rules: Vec<(&str, Value)>, spec: Option<Value>) -> GrammarIr {
    let mut j: Value = serde_json::from_str(include_str!("ir/operators.ir.json")).unwrap();
    let symbol = j["semantics"]["symbols"][0].clone();
    let rule_template = j["semantics"]["rules"][0].clone();
    let number = j["semantics"]["tokens"][0].clone();
    for key in [
        "astTypes",
        "captures",
        "expressions",
        "recipes",
        "rules",
        "symbols",
    ] {
        j["semantics"][key] = json!([]);
    }
    for key in ["expressions", "rules", "prefixTries", "sccs"] {
        j["analysis"][key] = json!([]);
    }
    j["semantics"]["trivia"]["insertionPlan"] = json!([]);
    j["plan"]["memo"]["entries"] = json!([]);
    j["plan"]["choices"] = json!([]);
    j["plan"]["operators"] = json!([]);
    j["requiredFeatures"] = json!([
        "backendDiagnostics",
        "captures",
        "commonField",
        "cursorFrameFlag",
        "deferredMapping",
        "dualCursor",
        "enum",
        "externTokens",
        "longestChoice",
        "memoReplay",
        "orderedChoice",
        "predictiveChoice",
        "regex",
        "scopeUndo",
        "structuredOccurrences",
        "triviaInsertion"
    ]);
    j["entry"]["ruleId"] = json!(format!("Operators::{}", rules[0].0));
    let mut ids = vec![];
    for (name, body) in rules {
        let id = format!("Operators::{name}");
        ids.push(id.clone());
        let body = flatten(&mut j, body, name, "body");
        let mut r = rule_template.clone();
        r["symbolId"] = json!(id);
        r["body"] = json!(body);
        r["operator"] = Value::Null;
        r["mappingTypeId"] = Value::Null;
        r["mappingParams"] = json!([]);
        j["semantics"]["rules"].as_array_mut().unwrap().push(r);
        let mut sym = symbol.clone();
        sym["id"] = json!(id);
        sym["qualifiedSymbolId"] = json!(id);
        sym["name"] = json!(name);
        sym["originalName"] = json!(name);
        sym["kind"] = json!("rule");
        j["semantics"]["symbols"].as_array_mut().unwrap().push(sym);
        j["analysis"]["rules"].as_array_mut().unwrap().push(json!({"ruleId":id,"sccId":"scc:all","followSyncPatterns":null,"leftRecursion":"none","progressProven":false}));
    }
    j["analysis"]["sccs"] =
        json!([{"id":"scc:all","ruleIds":ids,"recursive":false,"memoPolicy":"allEligible"}]);
    let mut sym = symbol;
    sym["id"] = json!("Operators::NUMBER");
    sym["qualifiedSymbolId"] = json!("Operators::NUMBER");
    sym["name"] = json!("NUMBER");
    sym["kind"] = json!("token");
    j["semantics"]["symbols"].as_array_mut().unwrap().push(sym);
    j["semantics"]["tokens"] = json!([number]);
    if let Some(spec) = spec {
        j["semantics"]["tokens"][0]["spec"] = spec;
    }
    from_json(&j.to_string()).unwrap()
}

#[test]
fn generated_semantic_contracts() {
    let mut grammars: Vec<(&str, GrammarIr)> = vec![];
    let mut add = |name, body| grammars.push((name, grammar(vec![("Root", body)], None)));
    add("empty", json!({"kind":"empty"}));
    add("empty_two", repeat(json!({"kind":"empty"}), 2, 2));
    add("empty_once", repeat(json!({"kind":"empty"}), 1, 2));
    add("zero_max", repeat(json!({"kind":"any"}), 0, 0));
    add("look", unary("lookahead", literal("😀")));
    add(
        "look_consume",
        seq(vec![unary("lookahead", literal("ab")), literal("a")]),
    );
    add("negative", unary("negativeLookahead", literal("a")));
    add("nested", capture("outer", capture("inner", literal(""))));
    add(
        "longest",
        choice(
            vec![
                capture("short", literal("a")),
                capture("long", literal("ab")),
            ],
            "longest",
        ),
    );
    add(
        "longest_tie",
        choice(
            vec![
                capture("first", literal("a")),
                capture("second", literal("a")),
            ],
            "longest",
        ),
    );
    add(
        "predict",
        choice(vec![literal("a"), literal("😀")], "predictive"),
    );
    add(
        "prefix",
        choice(vec![literal("var"), literal("variable")], "predictive"),
    );
    add(
        "separated",
        json!({"kind":"separated","element":literal("a"),"separator":literal(","),"min":1,"pairRollback":true}),
    );
    add(
        "repeat_matched",
        unary("lookahead", repeat(literal("a"), 2, 2)),
    );
    add(
        "terminator",
        json!({"kind":"repeat","child":{"kind":"any"},"min":0,"max":null,"terminator":unary("lookahead",literal("#"))}),
    );
    add("number", token());
    let predictive = &mut grammars
        .iter_mut()
        .find(|(name, _)| *name == "predict")
        .unwrap()
        .1;
    predictive
        .semantics
        .diagnostics
        .expected_display
        .push(ubnfc_front::ir::ExpectedDisplay {
            expr_id: predictive.semantics.expressions[1].id.clone(),
            java: "a".into(),
            rust: "literal a".into(),
        });
    grammars.push(("until",grammar(vec![("Root",seq(vec![unary("lookahead",literal("a")),token()]))],Some(json!({"kind":"until","terminator":"#","emptyTerminator":"consumeToEof","missingTerminator":"consumeToEofAndSucceed","preserveMatched":true})))));
    grammars.push((
        "memo",
        grammar(
            vec![
                (
                    "Root",
                    seq(vec![choice(
                        vec![
                            seq(vec![reference("Atom"), literal("!")]),
                            reference("Atom"),
                        ],
                        "ordered",
                    )]),
                ),
                ("Atom", capture("value", literal("a"))),
            ],
            None,
        ),
    ));
    let mut scope = grammar(
        vec![
            (
                "Root",
                choice(
                    vec![
                        seq(vec![reference("Decl"), literal("!")]),
                        seq(vec![reference("Decl"), reference("Use")]),
                    ],
                    "ordered",
                ),
            ),
            ("Decl", capture("decl", literal("a"))),
            ("Use", capture("use", literal("a"))),
        ],
        None,
    );
    use ubnfc_front::ir::*;
    let decl = scope
        .semantics
        .captures
        .iter()
        .find(|c| c.name == "decl")
        .unwrap()
        .site_id
        .clone();
    let usage = scope
        .semantics
        .captures
        .iter()
        .find(|c| c.name == "use")
        .unwrap()
        .site_id
        .clone();
    scope.semantics.scope.effects = vec![
        ScopeEffect {
            rule_id: "Operators::Decl".into(),
            mode: ScopeEffectMode::Lexical,
            events: vec![ScopeEvent {
                action: ScopeEffectEventsAction::Declare,
                capture_site_id: Some(decl),
                name: None,
                when: ScopeEffectEventsWhen::SuccessAfterLeave,
            }],
        },
        ScopeEffect {
            rule_id: "Operators::Use".into(),
            mode: ScopeEffectMode::Lexical,
            events: vec![ScopeEvent {
                action: ScopeEffectEventsAction::Use,
                capture_site_id: Some(usage),
                name: None,
                when: ScopeEffectEventsWhen::AfterDeclarations,
            }],
        },
    ];
    grammars.push(("scope", scope));
    let ext = json!({"kind":"extern","logicalId":"custom","originalClassName":"Custom","javaBinding":null,"rustBinding":null,"inputState":["consumed","matched","mode","invertMatch","resetMatchedWithConsumed","stateVersion"],"resultFields":["ok","consumedEnd","matchedEnd","valueSpan","diagnostics","effects"]});
    grammars.push(("external", grammar(vec![("Root", token())], Some(ext))));
    let mut dfa = grammar(
        vec![("Root", token())],
        Some(
            json!({"kind":"regex","pattern":"[a-z]+","regularSubset":true,"javaFallback":"none","matching":"lookingAt","rustFallback":"generationError","zeroWidth":"fail"}),
        ),
    );
    dfa.plan.lexical[0].strategy = LexicalPlanStrategy::Dfa;
    dfa.plan.lexical[0].dfa = Some(Dfa {
        start_state: 0,
        states: vec![
            DfaState {
                id: 0,
                accepting: false,
                transitions: vec![DfaTransition {
                    range: CharRange {
                        start: 97,
                        end: 122,
                    },
                    target: 1,
                }],
            },
            DfaState {
                id: 1,
                accepting: true,
                transitions: vec![DfaTransition {
                    range: CharRange {
                        start: 97,
                        end: 122,
                    },
                    target: 1,
                }],
            },
        ],
    });
    grammars.push(("dfa", dfa));
    for (name, duplicate) in [("mapped_record", false), ("mapping_failure", true)] {
        let body = if duplicate {
            seq(vec![
                capture("value", literal("a")),
                capture("value", literal("a")),
            ])
        } else {
            capture("value", literal("a"))
        };
        let mut ir = grammar(vec![("Root", body)], None);
        let field = AstField {
            cardinality: Cardinality::Scalar,
            name: "value".into(),
            value_type: ValueType::Text {},
        };
        ir.semantics.ast_types = vec![
            AstType::TransparentUnion {
                id: "Node".into(),
                name: "Node".into(),
                enclosing_type_id: None,
                variants: vec!["Node.Item".into()],
            },
            AstType::Record {
                id: "Node.Item".into(),
                name: "Item".into(),
                enclosing_type_id: Some("Node".into()),
                fields: vec![field.clone()],
                implements: vec!["Node".into()],
            },
        ];
        ir.semantics.common_fields = vec![CommonField {
            field,
            interface_type_id: "Node".into(),
            variant_type_ids: vec!["Node.Item".into()],
        }];
        ir.semantics.rules[0].mapping_type_id = Some("Node.Item".into());
        ir.semantics.rules[0].mapping_params = vec!["value".into()];
        ir.semantics.recipes = vec![Recipe {
            id: "recipe:Root".into(),
            rule_id: "Operators::Root".into(),
            kind: RecipeKind::Construct,
            result_type_id: Some("Node.Item".into()),
            fields: vec![RecipeField {
                field: "value".into(),
                cardinality: Cardinality::Scalar,
                conversion: Conversion::Text,
                site_ids: ir
                    .semantics
                    .captures
                    .iter()
                    .map(|c| c.site_id.clone())
                    .collect(),
            }],
            leaf_fallback: None,
            on_failure: Default::default(),
            phase: Default::default(),
            transform_id: None,
        }];
        grammars.push((name, ir));
    }
    let mut enumeration = grammar(
        vec![(
            "Root",
            choice(vec![literal("red"), literal("blue")], "ordered"),
        )],
        None,
    );
    enumeration.semantics.ast_types = vec![AstType::Enum {
        id: "Color".into(),
        name: "Color".into(),
        enclosing_type_id: None,
        rule_id: "Operators::Root".into(),
        variants: vec![
            EnumVariant {
                java_name: "RED".into(),
                literal: "red".into(),
                rust_name: "Red".into(),
                variant: "RED".into(),
            },
            EnumVariant {
                java_name: "BLUE".into(),
                literal: "blue".into(),
                rust_name: "Blue".into(),
                variant: "BLUE".into(),
            },
        ],
    }];
    enumeration.semantics.rules[0].mapping_type_id = Some("Color".into());
    enumeration.semantics.recipes = vec![Recipe {
        id: "recipe:Root".into(),
        rule_id: "Operators::Root".into(),
        kind: RecipeKind::Enum,
        result_type_id: Some("Color".into()),
        fields: vec![],
        leaf_fallback: None,
        on_failure: Default::default(),
        phase: Default::default(),
        transform_id: None,
    }];
    grammars.push(("enumeration", enumeration));
    // Cache 有無は trivia の consumed(0) による matched reset も同じ。
    let mut cached = grammar(vec![("Root", seq(vec![literal("a"), literal("b")]))], None);
    cached.plan.trivia_cache.enabled = true;
    let body = cached.semantics.rules[0].body.clone();
    cached.semantics.trivia.insertion_plan = vec![TriviaInsertion {
        boundary: Boundary {
            owner_expr_id: body,
            at: BoundaryAt::AfterChild,
            child_index: Some(0),
        },
        policy_id: "trivia:javaStyle".into(),
    }];
    grammars.push(("cached_trivia", cached));
    for (module, which) in [
        ("space_token", "Space"),
        ("digit", "Digit"),
        ("alphabet", "Alphabet"),
        ("underscore", "AlphabetUnderScore"),
        ("alnum", "AlphabetNumericUnderScore"),
        ("cp_set", "CodePointSet"),
        ("identifier", "Identifier"),
        ("quoted", "Quoted"),
        ("single_quoted", "SingleQuoted"),
        ("eof_token", "EndOfSource"),
    ] {
        grammars.push((module,grammar(vec![("Root",token())],Some(json!({"kind":"builtin","which":which,"codePoints":if which=="CodePointSet"{vec![0x1f600]}else{vec![]}})))));
    }
    grammars.push((
        "case_insensitive",
        grammar(
            vec![("Root", token())],
            Some(
                json!({"kind":"caseInsensitive","word":"kiss","comparison":"javaEqualsIgnoreCase"}),
            ),
        ),
    ));
    grammars.push((
        "char_range",
        grammar(
            vec![("Root", token())],
            Some(json!({"kind":"charRange","min":97,"max":122})),
        ),
    ));
    grammars.push((
        "negation",
        grammar(
            vec![("Root", token())],
            Some(json!({"kind":"negation","excluded":"ab"})),
        ),
    ));
    let mut backref = grammar(
        vec![(
            "Root",
            seq(vec![
                capture("decl", literal("a")),
                capture("use", json!({"kind":"any"})),
            ]),
        )],
        None,
    );
    let decl = backref
        .semantics
        .captures
        .iter()
        .find(|c| c.name == "decl")
        .unwrap()
        .site_id
        .clone();
    let usage = backref
        .semantics
        .captures
        .iter()
        .find(|c| c.name == "use")
        .unwrap()
        .site_id
        .clone();
    backref.semantics.scope.effects = vec![ScopeEffect {
        rule_id: "Operators::Root".into(),
        mode: ScopeEffectMode::SameRuleText,
        events: vec![
            ScopeEvent {
                action: ScopeEffectEventsAction::Declare,
                capture_site_id: Some(decl),
                name: None,
                when: ScopeEffectEventsWhen::AfterDeclarations,
            },
            ScopeEvent {
                action: ScopeEffectEventsAction::Use,
                capture_site_id: Some(usage),
                name: None,
                when: ScopeEffectEventsWhen::AfterDeclarations,
            },
        ],
    }];
    grammars.push(("backref", backref));
    let out = Command::new("mktemp")
        .args(["-d", "/tmp/ubnfc-semantics.XXXXXX"])
        .output()
        .unwrap();
    let dir = PathBuf::from(String::from_utf8(out.stdout).unwrap().trim());
    let mut lib = String::from("#![allow(dead_code)]\n");
    for (name, ir) in grammars {
        let output = emit(&ir, &Default::default()).unwrap_or_else(|e| panic!("{name}: {e}"));
        if name == "predict" {
            assert!(output.files["parser.rs"].contains("let predicted=match"));
        }
        output.write_to(dir.join("src").join(name)).unwrap();
        lib.push_str(&format!("pub mod {name};\n"));
    }
    lib.push_str(include_str!("support/semantic_assertions.rs"));
    fs::write(dir.join("src/lib.rs"), lib).unwrap();
    fs::write(dir.join("Cargo.toml"),"[package]\nname=\"semantic-check\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[features]\njson=[\"dep:serde\",\"dep:serde_json\"]\n[dependencies]\nserde={version=\"1\",optional=true}\nserde_json={version=\"1\",optional=true}\n").unwrap();
    for args in [
        vec!["test", "--offline"],
        vec!["test", "--offline", "--all-features"],
        vec![
            "clippy",
            "--offline",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    ] {
        let output = Command::new(env!("CARGO"))
            .args(&args)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?} {}:\n{}\n{}",
            dir.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::remove_dir_all(dir).unwrap();
}
