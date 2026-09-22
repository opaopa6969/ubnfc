use serde_json::json;
use ubnfc_target_rust::{canonical, operators, power, AstKind, DiagnosticKind, ParseOptions};

fn prefix() -> ParseOptions {
    ParseOptions {
        build_ast: true,
        require_eof: false,
    }
}
fn strict_int(text: &str) -> Option<f64> {
    text.parse::<i32>().ok().map(f64::from)
}

#[test]
fn trivia_comments_and_java_space_set() {
    for text in [
        "1 + 2",
        " 1+2 ",
        "1/*😀*/+//line\r\n2",
        "1\t\x0b\x0c\r\n + 2",
        "1+2//😀",
        "1+2/*a/*b*/",
    ] {
        let r = operators::parse(text);
        assert!(r.ok, "{text:?}");
        assert_eq!(r.ast.unwrap().evaluate(), 3.0);
        assert_eq!(
            (r.consumed_cp, r.matched_cp),
            (text.chars().count(), text.chars().count())
        );
    }
    for text in ["1\u{a0}+2", "1\u{2003}+2", "1\0+2", "1/*open", "1+/*open"] {
        assert!(!operators::parse(text).ok, "{text:?}");
    }
}

#[test]
fn exact_chain_and_group_spans_from_java_runtime_test() {
    let r = operators::parse(" 1 + 2*3 ");
    let ast = canonical::value(r.ast.as_ref().unwrap());
    assert_eq!(ast["span"], json!([0, 9]));
    assert_eq!(ast["fields"]["left"]["span"], json!([1, 3]));
    assert_eq!(
        ast["fields"]["left"]["fields"]["left"]["span"],
        json!([1, 3])
    );
    let product = &ast["fields"]["right"][0];
    assert_eq!(product["span"], json!([5, 9]));
    assert_eq!(product["fields"]["left"]["span"], json!([5, 6]));
    assert_eq!(product["fields"]["right"][0]["span"], json!([7, 9]));
    let r = operators::parse("1 /*inner*/ +2*3");
    let ast = canonical::value(r.ast.as_ref().unwrap());
    assert_eq!(
        ast["fields"]["left"]["fields"]["left"]["fields"]["value"],
        "1 /*inner*/"
    );
    assert_eq!(r.ast.unwrap().evaluate(), 7.0);
    assert_eq!(r.captures[0].span, [0, 12]);
}

#[test]
fn all_number_forms_keep_the_lexeme() {
    for (text, expected) in [
        ("+1", 1.0),
        ("-0", -0.0),
        ("001", 1.0),
        ("12.", 12.0),
        (".5", 0.5),
        ("-.5", -0.5),
        ("+12.50E-1", 1.25),
        ("1.e+2", 100.0),
        (".25e2", 25.0),
        ("2147483648", 2147483648.0),
    ] {
        let r = operators::parse(text);
        assert!(r.ok, "{text}");
        let ast = r.ast.unwrap();
        assert_eq!(ast.evaluate(), expected);
        assert_eq!(
            canonical::value(&ast)["fields"]["left"]["fields"]["left"]["fields"]["value"],
            text
        );
    }
    for text in [
        ".", "+.", "--1", "１２", "NaN", "Infinity", "1e", "1e+", "1E-",
    ] {
        assert!(!operators::parse(text).ok, "{text}");
    }
}

#[test]
fn prefix_success_and_incomplete_exponent_remain_separate_from_eof() {
    for (input, consumed) in [("1abc", 1), ("1e+", 1), ("1+", 1), ("1+2)", 3)] {
        let r = operators::parse_with_options(input, prefix());
        assert!(r.ok && r.ast.is_some());
        assert_eq!((r.consumed_cp, r.matched_cp), (consumed, consumed));
        assert!(!operators::parse(input).ok);
    }
    let r = operators::parse("1e+");
    assert_eq!(r.diagnostics[0].offset_cp, 3);
    assert_eq!(r.diagnostics[0].expected, ["digit"]);
}

#[test]
fn syntax_success_mapping_failure_is_explicit_and_delayed() {
    for text in ["2147483648", ".5", "1e2"] {
        let r = operators::parse_with_number_mapper(text, ParseOptions::default(), strict_int);
        assert!(r.ok && r.ast.is_none());
        assert_eq!(r.consumed_cp, text.len());
        assert!(r.node_spans.is_empty() && !r.captures.is_empty());
        assert_eq!(r.statistics.mapped_numbers, 1);
        assert!(r
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Mapping && d.offset_cp == 0));
    }
    // recursive 枝で大きな数を認識しても ')' 欠落なら、その枝を mapping しない。
    let r = power::parse_with_number_mapper(
        "2^(2147483648",
        power::Mode::Grouped,
        prefix(),
        strict_int,
    );
    assert!(r.ok && r.ast.is_some());
    assert_eq!(r.ast.unwrap().evaluate(), 2.0);
    assert_eq!(r.statistics.mapped_numbers, 1);
    assert!(r
        .diagnostics
        .iter()
        .all(|d| d.kind != DiagnosticKind::Mapping));
    assert!(r.captures.iter().all(|c| c.span[1] <= 1));
}

#[test]
fn recognition_and_syntax_failure_never_call_mapping() {
    fn forbidden(_: &str) -> Option<f64> {
        panic!("認識中に変換した")
    }
    let r = operators::parse_with_number_mapper(
        "2147483648",
        ParseOptions {
            build_ast: false,
            require_eof: true,
        },
        forbidden,
    );
    assert!(r.ok && r.ast.is_none());
    assert_eq!(r.statistics.recipes, 0);
    let r = operators::parse_with_number_mapper("2147483648+", ParseOptions::default(), forbidden);
    assert!(!r.ok);
    assert_eq!(r.statistics.mapped_numbers, 0);
}

#[test]
fn missing_number_fails_without_empty_nodes_or_stalled_repetition() {
    for input in ["", " ", "+", "()", "1+", "1*", "1+( )"] {
        let r = operators::parse(input);
        assert!(!r.ok, "{input}");
        assert!(r.ast.is_none() && r.node_spans.is_empty() && r.captures.is_empty());
        assert!(r.statistics.rule_evaluations < 100);
    }
    for mode in power::Mode::ALL {
        for input in ["", "^", "2^", "2^^"] {
            assert!(!power::parse_in_mode(input, mode).ok);
        }
    }
}

#[test]
fn failure_diagnostics_use_farthest_code_point_and_deepest_rule() {
    for (input, offset, expected, deepest) in [
        ("", 0, "NUMBER", "NUMBER"),
        ("1+", 2, "NUMBER", "NUMBER"),
        ("1**2", 2, "NUMBER", "NUMBER"),
        ("(1+2", 4, "')'", "MulOp"),
        ("1/*😀*/+", 7, "NUMBER", "NUMBER"),
        ("1+2)", 3, "EOF", "MulOp"),
    ] {
        let r = operators::parse(input);
        let d = &r.diagnostics[0];
        assert_eq!(d.offset_cp, offset, "{input}");
        assert!(d.expected.contains(&expected), "{input}: {d:?}");
        assert_eq!(d.deepest_rule, deepest, "{input}");
        assert_eq!(d.rule_path.last(), Some(&deepest));
    }
    let r = power::parse("2^3^");
    assert_eq!(r.diagnostics[0].offset_cp, 4);
    assert_eq!(r.diagnostics[0].deepest_rule, "NUMBER");
}

#[test]
fn right_recursion_retains_singleton_lists_and_completion_order() {
    let r = power::parse_in_mode("2^3^2", power::Mode::Mapped);
    assert!(
        r.statistics.memo_hits > 0,
        "最深段の Atom fallback は memo hit"
    );
    let mut ast = r.ast.as_ref().unwrap();
    for expected_left in ["2", "3", "2"] {
        let AstKind::Power { left, op, right } = &ast.kind else {
            panic!("Power")
        };
        let AstKind::Number { value, .. } = &left.kind else {
            panic!("Number")
        };
        assert_eq!(value, expected_left);
        assert_eq!(op.len(), right.len());
        assert!(right.len() <= 1);
        if let Some(next) = right.first() {
            ast = next;
        }
    }
    let right: Vec<_> = r
        .captures
        .iter()
        .filter(|c| c.name == "right")
        .map(|c| c.span)
        .collect();
    assert_eq!(right, [[4, 5], [2, 5]]);
    assert_eq!(
        r.statistics.mapped_numbers, 3,
        "memo replay で変換を重複しない"
    );
    let r = power::parse_with_options("2^", power::Mode::Mapped, prefix());
    assert!(r
        .captures
        .iter()
        .all(|c| c.name != "op" && c.name != "right"));
    assert_eq!(r.captures.iter().filter(|c| c.name == "value").count(), 1);
}

#[test]
fn right_fallback_and_unicode_atom_keep_their_own_trivia_boundaries() {
    let single = power::parse_in_mode(" 2 ", power::Mode::Mapped);
    let chain = power::parse_in_mode(" 2 ^ 3 ", power::Mode::Mapped);
    let left_span = |r: &ubnfc_target_rust::ParseResult| {
        canonical::value(r.ast.as_ref().unwrap())["fields"]["left"]["span"].clone()
    };
    assert_eq!(left_span(&single), json!([0, 3]));
    assert_eq!(left_span(&chain), json!([1, 3]));
    let r = power::parse_in_mode(" 😀 2 ^ 3 ^ 😀2 ", power::Mode::Unicode);
    assert_eq!(r.consumed_cp, 14);
    assert_eq!(left_span(&r), json!([1, 5]));
    assert_eq!(r.captures[0].span, [3, 5]);
    assert_eq!(r.ast.unwrap().evaluate(), 512.0);
}

#[test]
fn left_fold_handles_512_atoms_without_recursive_list_nodes() {
    let r = operators::parse(&vec!["1"; 512].join("+"));
    let ast = r.ast.unwrap();
    assert_eq!(ast.evaluate(), 512.0);
    let AstKind::Binary { op, right, .. } = ast.kind else {
        panic!("Binary")
    };
    assert_eq!((op.len(), right.len()), (511, 511));
}
