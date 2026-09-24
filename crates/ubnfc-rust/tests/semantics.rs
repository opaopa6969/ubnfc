use serde_json::json;
use std::{fs, path::PathBuf, process::Command};
use ubnfc_rust::emit;

#[path = "support/fixture.rs"]
mod fixture;
use fixture::*;

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
    // D-040: 語境界。同じ文法を境界あり／なしで並べ、差が境界だけであることを固定する。
    add("boundary_off", literal("let"));
    add("boundary_on", bounded_literal("let"));
    add("boundary_symbol", bounded_literal("let+"));
    add(
        "boundary_prefix_choice",
        choice(
            vec![bounded_literal("var"), bounded_literal("variable")],
            "ordered",
        ),
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
                json!({"kind":"caseInsensitive","word":"kiss","comparison":"javaEqualsIgnoreCase","wordBoundary":false}),
            ),
        ),
    ));
    grammars.push((
        "case_insensitive_boundary",
        grammar(
            vec![("Root", token())],
            Some(
                json!({"kind":"caseInsensitive","word":"kiss","comparison":"javaEqualsIgnoreCase","wordBoundary":true}),
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
    // D-063: 補助面まで届く charRange。Rust の照合は UTF-8 を 1 scalar 復号して
    // code point で比べるので、surrogate 対の概念なしに 1 文字として読む。
    grammars.push((
        "char_range_supplementary",
        grammar(
            vec![("Root", token())],
            Some(json!({"kind":"charRange","min":0xe000,"max":0x10ffff})),
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
