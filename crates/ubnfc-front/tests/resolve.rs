use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use ubnfc_front::resolve::*;
use ubnfc_front::syntax::parse_file;
use ubnfc_front::validate::validate;

fn memory(files: &[(&str, &str)]) -> impl Fn(&Path) -> io::Result<String> {
    let files: BTreeMap<_, _> = files
        .iter()
        .map(|(p, s)| (PathBuf::from(p), s.to_string()))
        .collect();
    move |path| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing fixture"))
    }
}

fn run(source: &str) -> Result<ResolvedGrammar, Vec<ResolveError>> {
    resolve(&[("main.ubnf", parse_file(source).unwrap())], &memory(&[]))
}

#[test]
fn nested_imports_rebind_local_rule_and_token_refs_and_keep_origins() {
    let source = "grammar Main { @import lib from './lib/library.ubnf' @root Start ::= lib.Value; Value ::= 'main'; }";
    let loader = memory(&[
        ("lib/library.ubnf", "grammar Library { @import leaf from '../tokens.ubnf' Value ::= Local leaf.T; Local ::= 'library'; }"),
        ("tokens.ubnf", "grammar Tokens { token T = NumberParser }"),
    ]);
    let resolved = resolve(&[("main.ubnf", parse_file(source).unwrap())], &loader).unwrap();
    let grammar = &resolved.grammars[0];
    assert_eq!(resolved.units.len(), 3);
    assert_eq!(resolved.imports.len(), 2);
    assert_eq!(resolved.imports[1].namespace, "lib.leaf");
    assert_eq!(
        grammar
            .rules
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>(),
        ["Start", "Value", "lib.Value", "lib.Local"]
    );
    let body = &grammar.rules[2].body.alternatives[0];
    for (element, expected, kind) in [
        (&body[0], "lib.Local", SymbolKind::Rule),
        (&body[1], "lib.leaf.T", SymbolKind::Token),
    ] {
        let ResolvedAtom::Ref { symbol, .. } = element.atom else {
            panic!("expected reference");
        };
        let symbol = resolved.symbol(symbol);
        assert_eq!(symbol.name, expected);
        assert_eq!(symbol.kind, kind);
    }
    assert_eq!(resolved.units[grammar.rules[2].unit.0].ast.name, "Library");
    assert_eq!(
        resolved.units[grammar.rules[2].unit.0].ast.rules[0].name,
        "Value"
    );
    assert!(validate(&resolved).is_empty());
}

#[test]
fn imports_are_relative_cached_and_reusable_under_distinct_aliases() {
    let calls = RefCell::new(Vec::new());
    let loader = |path: &Path| {
        calls.borrow_mut().push(path.to_path_buf());
        assert_eq!(path, Path::new("root/lib.ubnf"));
        Ok("grammar L { @root Rule ::= 'x'; }".into())
    };
    let source = "grammar M { @import a from './lib.ubnf' @import b from 'dir/../lib.ubnf' @root Start ::= a.Rule b.Rule; }";
    let resolved = resolve(&[("root/main.ubnf", parse_file(source).unwrap())], &loader).unwrap();
    assert_eq!(calls.borrow().len(), 1);
    assert_eq!(resolved.units.len(), 2);
    assert_eq!(
        resolved.grammars[0].root,
        resolved.lookup(GrammarId(0), "Start")
    );
    assert_ne!(
        resolved.lookup(GrammarId(0), "a.Rule"),
        resolved.lookup(GrammarId(0), "b.Rule")
    );
}

#[test]
fn imported_file_last_grammar_matches_java_export_contract() {
    let resolved = resolve(
        &[(
            "main.ubnf",
            parse_file("grammar M { @import lib from 'lib.ubnf' @root Start ::= lib.Value; }")
                .unwrap(),
        )],
        &memory(&[(
            "lib.ubnf",
            "grammar First { Value ::= 'first'; } grammar Last { Value ::= 'last'; }",
        )]),
    )
    .unwrap();
    assert_eq!(resolved.units[resolved.imports[0].to.0].ast.name, "Last");
}

#[test]
fn supplied_files_bypass_loader_and_grammars_have_separate_namespaces() {
    let main =
        parse_file("grammar M { @import lib from 'lib.ubnf' @root Start ::= lib.Start; }").unwrap();
    let lib = parse_file("grammar L { @root Start ::= 'x'; }").unwrap();
    let resolved = resolve(&[("main.ubnf", main), ("lib.ubnf", lib)], &|_| {
        panic!("already supplied")
    })
    .unwrap();
    assert_eq!(resolved.grammars.len(), 2);
    assert_ne!(
        resolved.lookup(GrammarId(0), "Start"),
        resolved.lookup(GrammarId(1), "Start")
    );
    assert!(validate(&resolved).is_empty());
}

#[test]
fn diamond_import_is_not_a_cycle() {
    let loader = memory(&[
        (
            "a.ubnf",
            "grammar A { @import c from 'c.ubnf' Value ::= c.Value; }",
        ),
        (
            "b.ubnf",
            "grammar B { @import c from 'c.ubnf' Value ::= c.Value; }",
        ),
        ("c.ubnf", "grammar C { Value ::= 'x'; }"),
    ]);
    let resolved = resolve(&[("main.ubnf", parse_file("grammar M { @import a from 'a.ubnf' @import b from 'b.ubnf' @root Start ::= a.Value b.Value; }").unwrap())], &loader).unwrap();
    assert_eq!(resolved.units.len(), 4);
    assert!(resolved.lookup(GrammarId(0), "a.c.Value").is_some());
    assert!(resolved.lookup(GrammarId(0), "b.c.Value").is_some());
}

#[test]
fn normalized_cycles_missing_imports_and_import_syntax_are_errors() {
    let file =
        parse_file("grammar G { @import a from 'a.ubnf' @root Start ::= a.Value; }").unwrap();
    let errors = resolve(
        &[("main.ubnf", file.clone())],
        &memory(&[(
            "a.ubnf",
            "grammar A { @import back from './dir/../main.ubnf' Value ::= 'x'; }",
        )]),
    )
    .unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "E-IMPORT-CYCLE");
    assert!(errors[0].message.contains("main.ubnf"));
    let errors = resolve(&[("main.ubnf", file.clone())], &memory(&[])).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "E-IMPORT-UNRESOLVED");
    assert_eq!(errors[0].message, "import が解決できません: 'a.ubnf'");
    let errors = resolve(&[("main.ubnf", file)], &memory(&[("a.ubnf", "broken")])).unwrap_err();
    assert_eq!(errors[0].code, "E-IMPORT-SYNTAX");
    assert_eq!(errors[0].path, Path::new("a.ubnf"));
}

#[test]
fn duplicate_symbols_aliases_grammars_and_roots_are_rejected() {
    for (source, code) in [
        (
            "grammar G { @root A ::= 'a'; A ::= 'b'; }",
            "E-SYMBOL-DUPLICATE",
        ),
        (
            "grammar G { token A = ANY @root A ::= 'a'; }",
            "E-SYMBOL-DUPLICATE",
        ),
        (
            "grammar G { @root A ::= 'a'; @root B ::= 'b'; }",
            "E-ROOT-MULTIPLE",
        ),
        (
            "grammar G { A ::= 'a'; } grammar G { B ::= 'b'; }",
            "E-GRAMMAR-DUPLICATE",
        ),
    ] {
        assert!(
            run(source).unwrap_err().iter().any(|e| e.code == code),
            "{source}"
        );
    }
    let file = parse_file("grammar G { @import a from 'lib.ubnf' @import a from 'lib.ubnf' @root Start ::= a.Value; }").unwrap();
    let errors = resolve(
        &[("main.ubnf", file)],
        &memory(&[("lib.ubnf", "grammar L { Value ::= 'x'; }")]),
    )
    .unwrap_err();
    assert_eq!(errors[0].code, "E-IMPORT-DUPLICATE-ALIAS");
    let file = parse_file("grammar G { A ::= 'a'; }").unwrap();
    assert!(resolve(
        &[("main.ubnf", file.clone()), ("./main.ubnf", file)],
        &memory(&[])
    )
    .unwrap_err()
    .iter()
    .any(|e| e.code == "E-SOURCE-DUPLICATE"));
}

#[test]
fn undefined_references_under_every_wrapper_and_separator_are_checked() {
    for body in [
        "Missing",
        "Missing+",
        "Missing?",
        "Missing*",
        "Missing{1,2}",
        "('a' Missing)",
        "['a' Missing]",
        "{'a' Missing}",
        "'a' % Missing",
        "'a' % (Missing @x)",
    ] {
        let source = format!("grammar G {{ @root Start ::= {body}; }}");
        let errors = run(&source).unwrap_err();
        assert_eq!(errors[0].code, "E-RULE-UNDEFINED", "{body}");
        assert_eq!(
            errors[0].message,
            "rule Start references undefined rule or token 'Missing'"
        );
    }
    let errors = resolve(
        &[(
            "main.ubnf",
            parse_file("grammar G { @import a from 'lib.ubnf' @root Start ::= a.Missing; }")
                .unwrap(),
        )],
        &memory(&[("lib.ubnf", "grammar L { Value ::= 'x'; }")]),
    )
    .unwrap_err();
    assert_eq!(errors[0].code, "E-RULE-UNDEFINED");
    let errors = run("grammar G { @root Start ::= Missing % Separator; }").unwrap_err();
    assert_eq!(errors.len(), 2);
    assert!(errors[1].message.contains("'Separator'"));
}

#[test]
fn resolved_quantifier_spans_include_the_wrapped_atom() {
    let source = "grammar G { @root Start ::= 'a'+{1,2} 'b'? 'c'* 'd' % ','; }";
    let resolved = run(source).unwrap();
    let elements = &resolved.grammars[0].rules[0].body.alternatives[0];
    let spans: Vec<_> = elements
        .iter()
        .map(|e| {
            let span = e.atom.span();
            &source[span.start..span.end]
        })
        .collect();
    assert_eq!(spans, ["'a'+{1,2}", "'b'?", "'c'*", "'d' % ','"]);
    let ResolvedAtom::Optional { body, .. } = &elements[1].atom else {
        panic!("optional");
    };
    assert_eq!(&source[body.span.start..body.span.end], "'b'");
}

#[test]
fn token_classification_does_not_confuse_custom_classes_with_builtins() {
    let resolved = run("grammar G { token N = NumberParser token F = org.unlaxer.parser.elementary.NumberParser token I = IdentifierParser token D = DigitParser token A = AlphabetParser token X = com.example.NumberParser token U = MissingParser token W = WordParser token P = org.unlaxer.parser.posix.WordParser @root Start ::= N; }").unwrap();
    let tokens = &resolved.grammars[0].tokens;
    assert_eq!(
        tokens[0].value,
        ResolvedTokenValue::Builtin {
            kind: BuiltinKind::Number
        }
    );
    assert_eq!(tokens[0].value, tokens[1].value);
    assert_eq!(
        tokens[2].value,
        ResolvedTokenValue::Builtin {
            kind: BuiltinKind::Identifier
        }
    );
    assert_eq!(
        tokens[3].value,
        ResolvedTokenValue::Builtin {
            kind: BuiltinKind::Digit
        }
    );
    assert_eq!(
        tokens[4].value,
        ResolvedTokenValue::Builtin {
            kind: BuiltinKind::CharacterSet("AlphabetParser")
        }
    );
    assert_eq!(
        tokens[5].value,
        ResolvedTokenValue::Extern {
            class_name: "com.example.NumberParser".into()
        }
    );
    assert!(tokens[6].unresolved);
    assert_eq!(
        tokens[7].value,
        ResolvedTokenValue::Extern {
            class_name: "org.unlaxer.parser.elementary.WordParser".into()
        }
    );
    assert!(matches!(
        tokens[8].value,
        ResolvedTokenValue::Builtin { .. }
    ));
    let issues = validate(&resolved);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "W-TOKEN-UNRESOLVED");
}

#[test]
fn all_token_operators_keep_their_values() {
    let resolved = run("grammar G { token A = ANY token B = EOF token C = EMPTY token D = UNTIL('*/') token E = NEGATION('abc') token F = LOOKAHEAD(':') token G = NEGATIVE_LOOKAHEAD('//') token H = CHAR_RANGE('a','z') token I = CI('select') token J = REGEX('[a-z]+') @root Start ::= A; }").unwrap();
    let values: Vec<_> = resolved.grammars[0]
        .tokens
        .iter()
        .map(|t| t.value.clone())
        .collect();
    assert_eq!(
        values,
        vec![
            ResolvedTokenValue::Any,
            ResolvedTokenValue::Eof,
            ResolvedTokenValue::Empty,
            ResolvedTokenValue::Until {
                terminator: "*/".into()
            },
            ResolvedTokenValue::Negation {
                chars: "abc".into()
            },
            ResolvedTokenValue::Lookahead {
                pattern: ":".into()
            },
            ResolvedTokenValue::NegativeLookahead {
                pattern: "//".into()
            },
            ResolvedTokenValue::CharRange {
                min: "a".into(),
                max: "z".into()
            },
            ResolvedTokenValue::CaseInsensitive {
                word: "select".into()
            },
            ResolvedTokenValue::Regex {
                pattern: "[a-z]+".into()
            }
        ]
    );
}

#[test]
fn captures_keep_sites_multiplicity_and_quantifier_ownership() {
    let source = "grammar G { @root Start ::= ('a' @x ('b' @x) @x)+ @items 'c'+ @whole 'd' % ('e' @sep) [ 'f' @opt ] ('g' @both)+{2,3}; }";
    let resolved = run(source).unwrap();
    let captures = &resolved.grammars[0].rules[0].captures;
    assert_eq!(
        captures.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        ["items", "x", "x", "x", "whole", "sep", "opt", "both"]
    );
    assert!(!captures[0].inside_quantifier);
    assert!(captures[0].quantified);
    assert!(captures[1..4]
        .iter()
        .all(|c| c.inside_quantifier && c.occurrences == 3));
    assert!(!captures[4].inside_quantifier);
    assert!(captures[4].quantified);
    assert!(captures[5].path.contains(&PathStep::Separator));
    assert!(captures[6].path.contains(&PathStep::Optional));
    assert_eq!(captures[7].quantifier_depth, 2);
    assert!(captures[7].path.contains(&PathStep::Bounded));
    for (i, capture) in captures.iter().enumerate() {
        assert_eq!(capture.id, i);
        assert!(
            source[capture.span.start..capture.span.end].contains(&format!("@{}", capture.name))
        );
        for other in &captures[..i] {
            assert_ne!(capture.path, other.path);
        }
    }
}

#[test]
fn absent_root_is_a_warning_and_repeated_annotation_is_one_root_rule() {
    let resolved = run("grammar G { Start ::= 'a'; }").unwrap();
    let issues = validate(&resolved);
    assert_eq!(issues[0].code, "W-GENERAL-NO-ROOT");
    assert_eq!(issues[0].severity(), "WARNING");
    assert!(validate(&run("grammar G { @root @root Start ::= 'a'; }").unwrap()).is_empty());
}

#[test]
fn suggestions_use_java_distance_and_declaration_order() {
    assert_eq!(suggest_similar_name("Tern", ["Term", "Expr"]), Some("Term"));
    assert_eq!(suggest_similar_name("X", ["B", "A"]), Some("B"));
    assert_eq!(suggest_similar_name("Missing", ["A", "B"]), None);
    assert_eq!(suggest_similar_name("", std::iter::empty()), None);
    assert_eq!(suggest_similar_name("😀", ["ab", "x"]), Some("ab"));
}
