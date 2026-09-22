use ubnfc_front::syntax::ast::*;
use ubnfc_front::syntax::{parse_file, SourceMap};

fn rule(source_body: &str) -> RuleDecl {
    let file = parse_file(&format!("grammar G {{ R ::= {source_body}; }}")).unwrap();
    file.grammars
        .into_iter()
        .next()
        .unwrap()
        .rules
        .into_iter()
        .next()
        .unwrap()
}

#[test]
fn empty_file_and_grammar_are_allowed_by_self_description() {
    assert!(parse_file("// コメント😀\n").unwrap().grammars.is_empty());
    assert!(parse_file("grammar G {}").unwrap().grammars[0]
        .rules
        .is_empty());
}

#[test]
fn empty_sequences_are_kept_in_all_positions() {
    let rule = rule(" | 'x' | ");
    let alternatives = rule.body.alternatives();
    assert_eq!(alternatives.len(), 3);
    assert!(alternatives[0].elements.is_empty());
    assert_eq!(alternatives[1].elements.len(), 1);
    assert!(alternatives[2].elements.is_empty());
    for sequence in alternatives {
        assert!(rule.body.span().start <= sequence.span.start);
        assert!(sequence.span.end <= rule.body.span().end);
    }
    assert!(self::rule("").body.alternatives()[0].elements.is_empty());
}

#[test]
fn grouping_optional_and_repeat_are_distinct_even_when_empty() {
    let rule = rule("('a' | 'b') [] {} (|)");
    let elements = &rule.body.alternatives()[0].elements;
    assert!(
        matches!(&elements[0].atom, Atom::Group { body, .. } if body.alternatives().len() == 2)
    );
    assert!(
        matches!(&elements[1].atom, Atom::Optional { body, .. } if body.alternatives()[0].elements.is_empty())
    );
    assert!(
        matches!(&elements[2].atom, Atom::Repeat { body, .. } if body.alternatives()[0].elements.is_empty())
    );
    assert!(matches!(&elements[3].atom, Atom::Group { body, .. }
        if body.alternatives().len() == 2 && body.alternatives().iter().all(|seq| seq.elements.is_empty())));
}

#[test]
fn postfix_and_bounded_quantifiers_have_explicit_limits() {
    let rule = rule("A+ B? C* D{2} E{1,3} F{2,} G{0}");
    let quantifiers: Vec<_> = rule.body.alternatives()[0]
        .elements
        .iter()
        .map(|element| element.quantifier.as_ref().unwrap())
        .collect();
    assert!(matches!(
        quantifiers.as_slice(),
        [
            Quantifier::Plus { .. },
            Quantifier::Question { .. },
            Quantifier::Star { .. },
            Quantifier::Bounded {
                min: 2,
                max: Some(2),
                ..
            },
            Quantifier::Bounded {
                min: 1,
                max: Some(3),
                ..
            },
            Quantifier::Bounded {
                min: 2,
                max: None,
                ..
            },
            Quantifier::Bounded {
                min: 0,
                max: Some(0),
                ..
            },
        ]
    ));
}

#[test]
fn repeat_and_bounded_quantifiers_are_disambiguated() {
    let rule = rule("A { B } A{3} { C }{2} A {} { 'd' }+");
    let elements = &rule.body.alternatives()[0].elements;
    assert_eq!(elements.len(), 7);
    assert!(elements[0].quantifier.is_none());
    assert!(matches!(elements[1].atom, Atom::Repeat { .. }));
    assert!(matches!(
        elements[2].quantifier,
        Some(Quantifier::Bounded {
            min: 3,
            max: Some(3),
            ..
        })
    ));
    assert!(matches!(elements[3].atom, Atom::Repeat { .. }));
    assert!(matches!(
        elements[3].quantifier,
        Some(Quantifier::Bounded {
            min: 2,
            max: Some(2),
            ..
        })
    ));
    assert!(elements[4].quantifier.is_none());
    assert!(matches!(elements[5].atom, Atom::Repeat { .. }));
    assert!(matches!(
        elements[6].quantifier,
        Some(Quantifier::Plus { .. })
    ));
}

#[test]
fn self_description_allows_quantifiers_followed_by_separator_and_capture() {
    let rule = rule("@typeof(seed) Item+{2,3} % (',' | ';') @values");
    let element = &rule.body.alternatives()[0].elements[0];
    assert_eq!(element.typeof_constraint.as_deref(), Some("seed"));
    assert!(matches!(
        element.postfix_quantifier,
        Some(Quantifier::Plus { .. })
    ));
    assert!(matches!(
        element.quantifier,
        Some(Quantifier::Bounded {
            min: 2,
            max: Some(3),
            ..
        })
    ));
    assert!(
        matches!(&element.separator, Some(Atom::Group { body, .. }) if body.alternatives().len() == 2)
    );
    assert_eq!(element.capture.as_deref(), Some("values"));
}

#[test]
fn separators_can_be_any_atomic_element() {
    let rule = rule("A % ',' B % ns.Comma C % ['x'] D % {'y'} E % ERROR('separator')");
    let separators: Vec<_> = rule.body.alternatives()[0]
        .elements
        .iter()
        .map(|element| element.separator.as_ref().unwrap())
        .collect();
    assert!(matches!(
        separators.as_slice(),
        [
            Atom::Literal { .. },
            Atom::Ref { .. },
            Atom::Optional { .. },
            Atom::Repeat { .. },
            Atom::Error { .. }
        ]
    ));
}

#[test]
fn typeof_is_a_prefix_on_the_next_element_unless_it_is_a_capture() {
    let rule = rule("A @a @typeof(a) B @b C @typeof D");
    let elements = &rule.body.alternatives()[0].elements;
    assert_eq!(elements.len(), 4);
    assert_eq!(elements[0].capture.as_deref(), Some("a"));
    assert_eq!(elements[1].typeof_constraint.as_deref(), Some("a"));
    assert_eq!(elements[1].capture.as_deref(), Some("b"));
    assert_eq!(elements[2].capture.as_deref(), Some("typeof"));
    assert!(elements[3].typeof_constraint.is_none());
    let rule = self::rule("A @typeof(x) B");
    assert_eq!(rule.body.alternatives()[0].elements.len(), 2);
    assert!(rule.body.alternatives()[0].elements[0].capture.is_none());
}

#[test]
fn identifier_boundaries_and_dotted_references_are_preserved() {
    let file = parse_file(
        "grammar G { @rooted tokenRule ::= T // boundary\r\n Root @value a.b.Value _x9; }",
    )
    .unwrap();
    let rule = &file.grammars[0].rules[0];
    assert!(matches!(&rule.annotations[0], Annotation::Simple { name, .. } if name == "rooted"));
    let elements = &rule.body.alternatives()[0].elements;
    assert_eq!(elements.len(), 4);
    assert!(matches!(&elements[0].atom, Atom::Ref { name, .. } if name == "T"));
    assert!(matches!(&elements[1].atom, Atom::Ref { name, .. } if name == "Root"));
    assert_eq!(elements[1].capture.as_deref(), Some("value"));
    assert!(
        matches!(&elements[2].atom, Atom::Ref { namespace: Some(namespace), name, .. }
        if namespace == "a.b" && name == "Value")
    );
    assert!(parse_file("grammarG {}").is_err());
}

#[test]
fn error_is_special_only_for_the_complete_error_expression() {
    let rule = rule("ERROR('message') ERROR (Item) ERROR ('a' 'b') ERRORMore");
    let atoms: Vec<_> = rule.body.alternatives()[0]
        .elements
        .iter()
        .map(|e| &e.atom)
        .collect();
    assert!(
        matches!(atoms.as_slice(), [Atom::Error { message, .. }, Atom::Ref { name, .. },
        Atom::Group { .. }, Atom::Ref { .. }, Atom::Group { .. }, Atom::Ref { name: last, .. }]
        if message == "message" && name == "ERROR" && last == "ERRORMore")
    );
}

#[test]
fn token_keywords_do_not_consume_identifier_prefixes() {
    let file = parse_file(
        "grammar G { token T = ANYthing token U = UNTILParser token V = pkg.EOF R ::= T; }",
    )
    .unwrap();
    for (token, expected) in
        file.grammars[0]
            .tokens
            .iter()
            .zip(["ANYthing", "UNTILParser", "pkg.EOF"])
    {
        assert!(matches!(&token.value, TokenValue::ClassRef { dotted, .. } if dotted == expected));
    }
}

#[test]
fn annotations_from_self_description_accept_colons_and_quoted_description() {
    let file = parse_file("grammar G { @longestChoice @predictiveChoice @eval(kind: 'literal', strategy: 'manual') @declares(symbol=name, description='説明') @mapping(Node) @recovery(sync) R ::= ; }").unwrap();
    assert!(matches!(file.grammars[0].rules[0].annotations.as_slice(), [
        Annotation::LongestChoice { .. }, Annotation::PredictiveChoice { .. },
        Annotation::Eval { entries, .. }, Annotation::Declares { description: Some(description), .. },
        Annotation::Mapping { params, .. }, Annotation::Recovery { mode: Some(mode), sync: None, .. },
    ] if entries.len() == 2 && entries[0].value == "literal" && entries[1].value == "manual"
        && description == "説明" && params.is_empty() && mode == "sync"));
}

#[test]
fn syntax_keeps_semantically_invalid_or_duplicate_values_for_later_validation() {
    let file = parse_file("grammar G { token T = CHAR_RANGE('ab','') @eval(x='a', x='b') @precedence(level=4294967296) R ::= T{9,2}; R ::= Missing; }").unwrap();
    let grammar = &file.grammars[0];
    assert_eq!(grammar.rules.len(), 2);
    assert!(
        matches!(&grammar.tokens[0].value, TokenValue::CharRange { min, max, .. }
        if min == "ab" && max.is_empty())
    );
    assert!(
        matches!(&grammar.rules[0].annotations[0], Annotation::Eval { entries, .. }
        if entries.len() == 2 && entries[0].key == entries[1].key)
    );
}

#[test]
fn legacy_double_quoted_strings_are_limited_to_setting_blocks() {
    let file =
        parse_file(r#"grammar G { @comment: { line: "//" quote: "\"" } R ::= 'x'; }"#).unwrap();
    assert!(
        matches!(&file.grammars[0].settings[0], GlobalSetting::Block { entries, .. }
        if entries[0].value == "//" && entries[1].value == "\"")
    );
    for source in [
        r#"grammar G { R ::= "x"; }"#,
        r#"grammar G { token T = UNTIL("x") }"#,
        r#"grammar G { @doc("x") R ::= ; }"#,
        r#"grammar G { @eval(kind="x") R ::= ; }"#,
        r#"grammar G { @package: "p" }"#,
    ] {
        assert!(parse_file(source).is_err(), "{source}");
    }
}

#[test]
fn known_and_unknown_escapes_and_literal_comments_are_preserved() {
    let rule = rule(r"'\n\t\r\\\'\q\d///*😀'");
    assert!(matches!(&rule.body.alternatives()[0].elements[0].atom,
        Atom::Literal { value, raw, .. } if value == "\n\t\r\\'\\q\\d///*😀" && raw == r"'\n\t\r\\\'\q\d///*😀'"));
}

#[test]
fn node_spans_use_bytes_and_exclude_surrounding_trivia() {
    let source = "//😀\ngrammar G { @doc('説明') R ::= @typeof(x) 'é😀'+ % ',' @値; }";
    // 識別子は ASCII。非 ASCII の不正 capture を byte 単位で指す。
    let error = parse_file(source).unwrap_err();
    assert_eq!(&source[error.span.start..error.span.end], "値");
    let source = source.replace("@値", "@value");
    let file = parse_file(&source).unwrap();
    assert_eq!(
        file.span,
        Span {
            start: 0,
            end: source.len()
        }
    );
    let grammar = &file.grammars[0];
    assert_eq!(
        &source[grammar.span.start..grammar.span.end],
        source.trim_start_matches("//😀\n")
    );
    let rule = &grammar.rules[0];
    assert_eq!(
        &source[rule.annotations[0].span().start..rule.annotations[0].span().end],
        "@doc('説明')"
    );
    let element = &rule.body.alternatives()[0].elements[0];
    assert_eq!(
        &source[element.span.start..element.span.end],
        "@typeof(x) 'é😀'+ % ',' @value"
    );
    let span = element.atom.span();
    assert_eq!(span.end - span.start, 8); // quotes 2 + é 2 + 😀 4 bytes
    assert_eq!(&source[span.start..span.end], "'é😀'");
}

#[test]
fn source_map_uses_codepoints_for_columns_and_handles_all_newlines() {
    let source = "a😀é\r\n\t文\rZ\n";
    let map = SourceMap::new(source);
    assert_eq!(map.line_col(0), (1, 1));
    assert_eq!(map.line_col(1), (1, 2));
    assert_eq!(map.line_col(3), (1, 2)); // UTF-8 の途中
    assert_eq!(map.line_col(5), (1, 3));
    assert_eq!(map.line_col(source.find('\t').unwrap()), (2, 1));
    assert_eq!(map.line_col(source.find('文').unwrap()), (2, 2));
    assert_eq!(map.line_col(source.find('Z').unwrap()), (3, 1));
    assert_eq!(map.line_col(source.len()), (4, 1));
    assert_eq!(map.line_col(usize::MAX), (4, 1));
    assert_eq!(SourceMap::new("").line_col(0), (1, 1));
}

#[test]
fn missing_semicolon_is_a_positioned_error() {
    let source = "grammar G {\n R ::= '😀'\n}";
    let error = parse_file(source).unwrap_err();
    assert_eq!(&source[error.span.start..error.span.end], "}");
    assert_eq!(error.expected, [";"]);
    assert_eq!(SourceMap::new(source).line_col(error.span.start), (3, 1));
    assert!(!error.message.is_empty());
}

#[test]
fn missing_grammar_brace_is_an_eof_error() {
    let source = "grammar G { R ::= 'x';\n";
    let error = parse_file(source).unwrap_err();
    assert_eq!(
        error.span,
        Span {
            start: source.len(),
            end: source.len()
        }
    );
    assert_eq!(error.expected, ["}"]);
    assert_eq!(SourceMap::new(source).line_col(error.span.start), (2, 1));
}

#[test]
fn unterminated_strings_report_the_opening_quote_through_eof() {
    for source in ["grammar G { R ::= '😀", "grammar G { R ::= 'abc\\"] {
        let error = parse_file(source).unwrap_err();
        assert_eq!(
            error.span,
            Span {
                start: source.find('\'').unwrap(),
                end: source.len()
            }
        );
        assert_eq!(error.expected, ["'"]);
    }
}

#[test]
fn block_comments_and_unexpected_unicode_are_positioned_lex_errors() {
    for (source, invalid) in [("/* comment */ grammar G {}", "/"), ("grammar 😀 {}", "😀")] {
        let error = parse_file(source).unwrap_err();
        assert_eq!(&source[error.span.start..error.span.end], invalid);
        assert!(!error.expected.is_empty());
    }
}

#[test]
fn malformed_constructs_and_declaration_order_are_rejected() {
    for source in [
        "grammar G { R ::= ('x'; }",
        "grammar G { R ::= [ 'x' ); }",
        "grammar G { R ::= { 'x' ]; }",
        "grammar G { R ::= A{1,2,3}; }",
        "grammar G { R ::= A{,2}; }",
        "grammar G { R ::= A{1 2}; }",
        "grammar G { R ::= A % ; }",
        "grammar G { R ::= A @; }",
        "grammar G { R ::= @typeof() A; }",
        "grammar G { R ::= a.; }",
        "grammar G { @unknown(x) R ::= ; }",
        "grammar G { @mapping(N, params=[]) R ::= ; }",
        "grammar G { @eval() R ::= ; }",
        "grammar G { @commonField() R ::= ; }",
        "grammar G { @precedence(level=-1) R ::= ; }",
        "grammar G { token T = UNKNOWN('x') R ::= ; }",
        "grammar G { @package: 'quoted' }",
        "grammar G { token T = ANY; }",
        "grammar G { token T = ANY @package: p }",
        "grammar G { R ::= ; token T = ANY }",
        "grammar G { @package: p @import a from 'a' }",
        "grammar G {} trailing",
    ] {
        let error = parse_file(source).expect_err(source);
        assert!(!error.message.is_empty(), "{source}");
        assert!(!error.expected.is_empty(), "{source}");
        assert!(
            source.get(error.span.start..error.span.end).is_some(),
            "{source}"
        );
    }
}

#[test]
fn integer_overflow_is_an_error_instead_of_wrapping() {
    for source in [
        "grammar G { R ::= A{18446744073709551616}; }",
        "grammar G { @precedence(level=18446744073709551616) R ::= ; }",
    ] {
        let error = parse_file(source).unwrap_err();
        assert_eq!(
            &source[error.span.start..error.span.end],
            "18446744073709551616"
        );
    }
}

#[test]
fn excessive_nesting_reports_an_error_instead_of_overflowing_the_stack() {
    let source = format!(
        "grammar G {{ R ::= {} 'x' {}; }}",
        "(".repeat(129),
        ")".repeat(129)
    );
    let error = parse_file(&source).unwrap_err();
    assert_eq!(error.expected, ["shallower rule body"]);
    assert!(source.get(error.span.start..error.span.end).is_some());
}
