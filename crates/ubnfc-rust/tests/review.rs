use serde_json::json;
use std::{fs, path::PathBuf, process::Command};
use ubnfc_front::ir::*;
#[path = "support/fixture.rs"]
#[allow(dead_code)]
mod fixture;
use fixture::*;

fn display(ir: &mut GrammarIr, index: usize, label: &str) {
    ir.semantics
        .diagnostics
        .expected_display
        .push(ExpectedDisplay {
            expr_id: ir.semantics.expressions[index].id.clone(),
            java: "JAVA_LABEL_MUST_NOT_LEAK".into(),
            rust: label.into(),
        });
}
fn external() -> serde_json::Value {
    json!({"kind":"extern","logicalId":"scan","originalClassName":"Scanner","javaBinding":null,"rustBinding":null,"inputState":["consumed","matched","mode","invertMatch","resetMatchedWithConsumed"],"resultFields":["ok","consumedEnd","matchedEnd","valueSpan","diagnostics","effects"]})
}

#[test]
fn java_review_regressions_in_generated_rust() {
    let mut grammars = Vec::new();
    for (name, kind) in [
        ("empty_look", "lookahead"),
        ("empty_not", "negativeLookahead"),
    ] {
        grammars.push((
            name,
            grammar(
                vec![("Root", token())],
                Some(json!({"kind":kind,"pattern":""})),
            ),
        ));
    }
    for (name, bound) in [("repeat_two", 2), ("repeat_three", 3)] {
        grammars.push((
            name,
            grammar(
                vec![(
                    "Root",
                    unary("lookahead", repeat(json!({"kind":"any"}), bound, bound)),
                )],
                None,
            ),
        ));
    }
    grammars.push((
        "optional_reset",
        grammar(
            vec![("Root", seq(vec![token(), unary("optional", literal("x"))]))],
            Some(json!({"kind":"lookahead","pattern":"a"})),
        ),
    ));
    let mut not = grammar(
        vec![(
            "Root",
            unary(
                "negativeLookahead",
                choice(
                    vec![seq(vec![literal("a"), literal("x")]), literal("a")],
                    "ordered",
                ),
            ),
        )],
        None,
    );
    for i in 0..not.semantics.expressions.len() {
        if matches!(&not.semantics.expressions[i].node,ExprNode::Literal{value,..} if value=="x") {
            display(&mut not, i, "'x'");
        }
    }
    grammars.push(("not", not));
    // Both references target the very same non-rule expression and memo key.
    let mut replay = grammar(
        vec![(
            "Root",
            seq(vec![
                unary("negativeLookahead", literal("x")),
                unary("lookahead", literal("x")),
            ]),
        )],
        None,
    );
    let word = replay.semantics.expressions[2].id.clone();
    if let ExprNode::Lookahead { child } = &mut replay.semantics.expressions[3].node {
        *child = word;
    }
    display(&mut replay, 2, "'x'");
    grammars.push(("not_replay", replay));
    let mut ci = grammar(
        vec![("Root", token())],
        Some(json!({"kind":"caseInsensitive","word":"let","comparison":"javaEqualsIgnoreCase"})),
    );
    display(&mut ci, 0, "CUSTOM_CI");
    grammars.push(("ci", ci));
    let mut regex = grammar(
        vec![("Root", token())],
        Some(
            json!({"kind":"regex","pattern":"a*","regularSubset":true,"matching":"lookingAt","zeroWidth":"fail","javaFallback":"java.util.regex.Pattern","rustFallback":"generationError"}),
        ),
    );
    regex.plan.lexical[0].strategy = LexicalPlanStrategy::Dfa;
    regex.plan.lexical[0].dfa = Some(Dfa {
        start_state: 0,
        states: vec![DfaState {
            id: 0,
            accepting: true,
            transitions: vec![DfaTransition {
                range: CharRange { start: 97, end: 97 },
                target: 0,
            }],
        }],
    });
    display(&mut regex, 0, "REGEX('a*')");
    grammars.push(("regex", regex));
    let mut trailing = grammar(
        vec![(
            "Root",
            choice(
                vec![
                    seq(vec![literal("a"), literal("b"), literal("x")]),
                    literal("a"),
                ],
                "ordered",
            ),
        )],
        None,
    );
    display(&mut trailing, 4, "'x'");
    grammars.push(("trailing", trailing));
    let mut comment = grammar(vec![("Root", seq(vec![literal("a"), literal("b")]))], None);
    comment.semantics.trivia.policies[0].effective_delimiter_set = vec![Delimiter::LineComment {
        open: "//".into(),
        line_end_profile: Default::default(),
    }];
    comment.semantics.trivia.insertion_plan = vec![TriviaInsertion {
        boundary: Boundary {
            owner_expr_id: comment.semantics.expressions[0].id.clone(),
            at: BoundaryAt::AfterChild,
            child_index: Some(0),
        },
        policy_id: comment.semantics.trivia.policies[0].id.clone(),
    }];
    grammars.push(("comment", comment));
    for (name, bound) in [("empty_zero", 0), ("empty_two", 2)] {
        grammars.push((
            name,
            grammar(
                vec![("Root", repeat(json!({"kind":"empty"}), bound, bound))],
                None,
            ),
        ));
    }
    grammars.push(("eof", grammar(vec![("Root", json!({"kind":"eof"}))], None)));
    grammars.push((
        "number",
        grammar(
            vec![("Root", seq(vec![literal("neg("), token(), literal(")")]))],
            None,
        ),
    ));
    let body = unary(
        "optional",
        choice(
            vec![
                seq(vec![literal("("), reference("Root"), literal(")")]),
                literal("a"),
            ],
            "ordered",
        ),
    );
    grammars.push(("recursive", grammar(vec![("Root", body)], None)));
    grammars.push((
        "scanner",
        grammar(
            vec![(
                "Root",
                seq(vec![
                    unary("lookahead", literal("a")),
                    unary("lookahead", token()),
                ]),
            )],
            Some(external()),
        ),
    ));
    let mut memo = grammar(
        vec![(
            "Root",
            choice(
                vec![
                    seq(vec![token(), literal("x")]),
                    seq(vec![token(), literal("y")]),
                ],
                "ordered",
            ),
        )],
        Some(external()),
    );
    let shared = memo.semantics.expressions[2].id.clone();
    if let ExprNode::Seq { items } = &mut memo.semantics.expressions[4].node {
        items[0] = shared;
    }
    grammars.push(("memo", memo));
    for (name, mode) in [
        ("lexical", ScopeEffectMode::Lexical),
        ("dynamic", ScopeEffectMode::Dynamic),
    ] {
        let mut ir = grammar(
            vec![
                (
                    "Root",
                    choice(
                        vec![
                            seq(vec![reference("Scoped"), literal("x")]),
                            seq(vec![reference("Scoped")]),
                        ],
                        "ordered",
                    ),
                ),
                ("Scoped", capture("value", literal("a"))),
                ("Peek", unary("lookahead", reference("Scoped"))),
            ],
            None,
        );
        let shared = ir.semantics.expressions[2].id.clone();
        if let ExprNode::Seq { items } = &mut ir.semantics.expressions[4].node {
            items[0] = shared;
        }
        let site = ir.semantics.captures[0].site_id.clone();
        ir.semantics.scope.effects = vec![ScopeEffect {
            rule_id: "Operators::Scoped".into(),
            mode,
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
                ScopeEvent {
                    action: ScopeEffectEventsAction::Declare,
                    when: ScopeEffectEventsWhen::SuccessAfterLeave,
                    capture_site_id: Some(site.clone()),
                    name: Some("var".into()),
                },
                ScopeEvent {
                    action: ScopeEffectEventsAction::Use,
                    when: ScopeEffectEventsWhen::AfterDeclarations,
                    capture_site_id: Some(site),
                    name: Some("var".into()),
                },
            ],
        }];
        grammars.push((name, ir));
    }
    let selfhost = from_json(include_str!(
        "../../../ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json"
    ))
    .unwrap();
    grammars.push(("selfhost", selfhost));
    // Legal Rust keywords, punctuation, Unicode and reserved generated type names.
    for (module, name) in [
        ("ident_keyword", "type"),
        ("ident_self", "Self"),
        ("ident_punctuation", "a-b"),
        ("ident_unicode", "変数"),
        ("ident_ast", "Ast"),
    ] {
        let mut ir = grammar(vec![("Root", capture(name, literal("a")))], None);
        let site = ir.semantics.captures[0].site_id.clone();
        ir.semantics.captures[0].cardinality = Cardinality::List;
        ir.semantics.ast_types = vec![AstType::Record {
            id: name.into(),
            name: name.into(),
            enclosing_type_id: None,
            implements: vec![],
            fields: vec![AstField {
                name: name.into(),
                value_type: ValueType::Text {},
                cardinality: Cardinality::List,
            }],
        }];
        ir.semantics.recipes = vec![Recipe {
            id: "recipe:Root".into(),
            rule_id: "Operators::Root".into(),
            kind: RecipeKind::Construct,
            result_type_id: Some(name.into()),
            fields: vec![RecipeField {
                field: name.into(),
                site_ids: vec![site],
                conversion: Conversion::Text,
                cardinality: Cardinality::List,
            }],
            leaf_fallback: None,
            on_failure: Default::default(),
            phase: Default::default(),
            transform_id: None,
        }];
        grammars.push((module, ir));
    }
    let output = Command::new("mktemp").arg("-d").output().unwrap();
    assert!(output.status.success());
    let dir = PathBuf::from(String::from_utf8(output.stdout).unwrap().trim());
    let mut lib = String::from("#![allow(dead_code)]\n");
    for (name, ir) in grammars {
        let mut generated = ubnfc_rust::emit(
            &ir,
            &ubnfc_rust::EmitOptions {
                emit_driver: false,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        if name == "scanner" {
            generated
                .files
                .get_mut("parser.rs")
                .unwrap()
                .push_str(include_str!("support/scanner_state_assertions.rs"));
        }
        generated.write_to(dir.join("src").join(name)).unwrap();
        lib.push_str(&format!("pub mod {name};\n"));
    }
    lib.push_str(include_str!("support/review_assertions.rs"));
    fs::write(dir.join("src/lib.rs"), lib).unwrap();
    fs::write(dir.join("Cargo.toml"),"[package]\nname=\"review-regressions\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[features]\ndefault=[\"json\"]\njson=[]\n[dependencies]\nserde=\"1\"\nserde_json=\"1\"\n").unwrap();
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
        let output = Command::new(env!("CARGO"))
            .args(&args)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?} {}\n{}\n{}",
            dir.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::remove_dir_all(dir).unwrap();
}
