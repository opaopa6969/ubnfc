use serde_json::Value;
use ubnfc_front::{analysis, ir::*, pipeline};

fn grammar(source: &str) -> GrammarIr {
    pipeline::compile("/virtual/regex.ubnf", &|_| Ok(source.into()))
        .unwrap()
        .ir
}
fn pattern(pattern: &str) -> GrammarIr {
    let escaped = pattern.replace('\\', "\\\\").replace('\'', "\\'");
    grammar(&format!(
        "grammar G {{ token T=REGEX('{escaped}') @root Root ::= T; }}"
    ))
}
fn dfa(ir: &GrammarIr) -> &RegexDfa {
    let TokenSpec::Regex {
        dfa: Some(Some(dfa)),
        regular_subset: true,
        ..
    } = &ir.semantics.tokens[0].spec
    else {
        panic!("DFA が必要: {:?}", ir.semantics.tokens[0].spec);
    };
    assert_eq!(ir.plan.lexical[0].strategy, LexicalPlanStrategy::Dfa);
    dfa
}

// backend とは独立に追加 IR の閉区間遷移を解釈する。戻り値は code point 消費長。
// 受理後も高優先の経路だけを走査し、最後の候補を採用する。開始状態の受理は
// 長さ 0 の候補なので zeroWidth=fail により返さない（その枝刈り自体は DFA に残る）。
fn run(dfa: &RegexDfa, input: &str) -> Option<usize> {
    let mut state = dfa.start;
    let mut accepted = None;
    for (i, c) in input.chars().enumerate() {
        let next = dfa.transitions.iter().find(|t| {
            t.from == state
                && t.ranges
                    .iter()
                    .any(|[lo, hi]| *lo <= c as u64 && c as u64 <= *hi)
        });
        let Some(next) = next else {
            break;
        };
        state = next.to;
        if dfa.accepting.contains(&state) {
            accepted = Some(i + 1);
        }
    }
    accepted
}

#[test]
fn regular_subset_leftmost_first_and_zero_width() {
    type CaseTable<'a> = &'a [(&'a str, &'a [(&'a str, Option<usize>)])];
    let cases: CaseTable<'_> = &[
        ("[a-z]+", &[("abc!", Some(3)), ("1a", None), ("", None)]),
        (
            r"\d+\.\d+",
            &[("12.345!", Some(6)), (".5", None), ("12.", None)],
        ),
        // Java Pattern と同じ左枝優先。右枝の方が長くても左枝を採用する。
        ("a|ab", &[("ab!", Some(1)), ("ax", Some(1)), ("b", None)]),
        ("ab|a", &[("ab!", Some(2)), ("ax", Some(1))]),
        ("[^a-c]+", &[("deあ!b", Some(4)), ("abc", None)]),
        (
            "[a-c]{2,3}",
            &[
                ("abcd", Some(3)),
                ("ab!", Some(2)),
                ("a!", None),
                ("aaaa", Some(3)),
            ],
        ),
        (
            "(ab|c)+d?",
            &[("abcabd!", Some(6)), ("ccd", Some(3)), ("d", None)],
        ),
        ("(?:a|bc){2,}", &[("abcbc!", Some(5)), ("a", None)]),
        ("a{2}", &[("aaa", Some(2)), ("a", None)]),
        ("a{0}", &[("a", None), ("", None)]),
        ("a*", &[("aaaa!", Some(4)), ("b", None), ("", None)]),
        ("a?", &[("a", Some(1)), ("b", None)]),
        ("a|", &[("a", Some(1)), ("", None)]),
        ("", &[("", None), ("x", None)]),
        ("^a+", &[("aa", Some(2)), ("ba", None)]),
        ("a^b", &[("ab", None)]),
        ("(^a)+", &[("aa", Some(1))]),
        ("a|^b", &[("ab", Some(1)), ("b", Some(1))]),
        (
            ".+",
            &[
                ("a😀b", Some(3)),
                ("a\nb", Some(1)),
                ("\r", None),
                ("\u{85}", None),
                ("\u{2028}", None),
                ("\u{2029}", None),
            ],
        ),
        (r"[\w\s]+", &[("Az_90 \t!", Some(7)), ("あ", None)]),
        (r"\D+", &[("ab_1", Some(3)), ("1", None)]),
        (r"\W+", &[("! あa", Some(3)), ("a", None)]),
        (r"\S+", &[("abc d", Some(3)), (" ", None)]),
        (r"[\d]+", &[("12a", Some(2)), ("١", None)]),
        (r"[\]\-\\]+", &[("]-\\x", Some(3))]),
        ("[-a]+", &[("-a!", Some(2))]),
    ];
    for &(regex, inputs) in cases {
        let ir = pattern(regex);
        let automaton = dfa(&ir);
        for &(input, expected) in inputs {
            assert_eq!(run(automaton, input), expected, "{regex:?} / {input:?}");
        }
        assert!(validate_references(&ir).is_empty());
        // 既存 lexical 表現と追加表現は常に同一の状態/遷移から派生する。
        assert_eq!(
            RegexDfa::from(ir.plan.lexical[0].dfa.as_ref().unwrap()),
            *automaton
        );
        assert_eq!(analysis::analyze(&ir.semantics).unwrap().1, ir.plan);
        let json = to_canonical_json(&ir);
        assert_eq!(to_canonical_json(&from_json(&json).unwrap()), json);
    }
}

#[test]
fn java_21_looking_at_oracle_matches_leftmost_first() {
    // JDK 21.0.9+7-LTS-338 の java.util.regex.Pattern で実測した固定 oracle。
    // var m = Pattern.compile(regex).matcher(input);
    // m.lookingAt() ? input.codePointCount(0, m.end()) : -1
    // Matcher.end() は UTF-16 offset。😀+ / 😀😀x は end=4、code point 長=2。
    // Some(0) は Pattern のゼロ幅成功であり、UBNF の zeroWidth=fail で失敗にする。
    let cases = [
        ("a|ab", "ab", Some(1)),
        ("ab|a", "ab", Some(2)),
        ("(a|ab)(c|bcd)", "abcd", Some(4)),
        ("a*ab", "aaab", Some(4)),
        ("(a|b)*a", "abab", Some(3)),
        (r"[0-9]+\.[0-9]+", "1.5", Some(3)),
        (r"\d+", "12x", Some(2)),
        ("x*", "y", Some(0)),
        ("(?:ab)+", "ababx", Some(4)),
        ("a{2,3}", "aaaa", Some(3)),
        ("[^a]+", "bbba", Some(3)),
        ("😀+", "😀😀x", Some(2)),
        ("(a|ab)c", "abc", Some(3)),
        ("(ab|a)b", "ab", Some(2)),
        ("(ab|a)b", "abb", Some(3)),
        ("a|", "a", Some(1)),
        ("|a", "a", Some(0)),
        ("(a|ab)*", "ab", Some(1)),
        ("(ab|a)*", "ab", Some(2)),
        ("a?ab", "ab", Some(2)),
        ("a{2,3}a", "aaa", Some(3)),
        ("ab|a", "ac", Some(1)),
        ("a*ab", "aaaa", None),
        ("^a+", "baa", None),
        ("(^a)+", "aa", Some(1)),
        ("(ab*c|a)", "abbbx", Some(1)),
        ("(ab*c|a)", "abbbc", Some(5)),
        ("(a|ab)(c|bcd)", "abc", Some(3)),
        ("😀|😀😀", "😀😀x", Some(1)),
        ("(ab|a)*b", "abab", Some(4)),
        ("a{0}", "a", Some(0)),
    ];
    for (regex, input, java_length) in cases {
        let ir = pattern(regex);
        let json = serde_json::to_value(dfa(&ir)).unwrap();
        assert_eq!(json["semantics"], "leftmostFirst");
        assert!(json.get("longestMatch").is_none());
        assert_eq!(json["anchoredAtStart"], true);
        assert_eq!(json["zeroWidth"], "fail");
        assert_eq!(
            run(dfa(&ir), input),
            java_length.filter(|&length| length > 0),
            "Java lookingAt: {regex:?} / {input:?}"
        );
    }
}

#[test]
fn java_nullable_repetition_oracle_requires_fallback() {
    // 同じ JDK 21 実走。通常の Thompson NFA の ε-cycle 除去だけでは Java の
    // ゼロ幅進捗停止を再現できない。差分の有無にかかわらず本体が nullable なら委譲。
    let cases = [
        ("(|a)*", "a", 0),
        ("(a?|b)*", "b", 0),
        ("(a*|b){0,2}a", "baa", 3),
        ("(a?)*", "aaab", 3),
        ("(a?)*", "b", 0),
        ("(a?)+", "aaab", 3),
        ("(a?){2,3}", "aaab", 3),
    ];
    for (regex, input, java_length) in cases {
        let ir = pattern(regex);
        assert!(
            matches!(
                &ir.semantics.tokens[0].spec,
                TokenSpec::Regex {
                    dfa: Some(None),
                    fallback: Some(_),
                    regular_subset: false,
                    java_fallback: TokenSpecRegexJavaFallback::JavaUtilRegexPattern,
                    ..
                }
            ),
            "Java lookingAt: {regex:?} / {input:?} = {java_length}"
        );
        assert_eq!(
            ir.plan.lexical[0].strategy,
            LexicalPlanStrategy::JavaPattern
        );
        assert!(ir.plan.lexical[0].dfa.is_none());
        assert!(validate_references(&ir).is_empty());
    }
}

#[test]
fn unsupported_or_unproven_patterns_explicitly_fall_back() {
    for regex in [
        r"(a+)\1",
        "(?=a)a",
        "(?!b)a",
        "(?<=a)b",
        r"\bword\b",
        "a*?",
        "a+?",
        "a??",
        "a{2,3}?",
        "a++a",
        r"\p{L}+",
        r"\P{L}",
        "(?i)a",
        "[a-z&&[^b]]",
        "[a[b]]",
        "[z-a]",
        "(",
        "[",
        "a{3,2}",
        "a{10001}",
        "a{9999}",
        "a$",
        r"\q",
    ] {
        let ir = pattern(regex);
        assert!(
            matches!(
                &ir.semantics.tokens[0].spec,
                TokenSpec::Regex {
                    dfa: Some(None),
                    fallback: Some(_),
                    regular_subset: false,
                    java_fallback: TokenSpecRegexJavaFallback::JavaUtilRegexPattern,
                    ..
                }
            ),
            "{regex}"
        );
        assert_eq!(
            ir.plan.lexical[0].strategy,
            LexicalPlanStrategy::JavaPattern
        );
        assert!(ir.plan.lexical[0].dfa.is_none());
        assert!(to_canonical_json(&ir).contains("\"dfa\": null"));
        assert!(validate_references(&ir).is_empty());
    }
}

fn oracle(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../corpus/oracle/regex-{name}.json"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn java_oracle_ascii_and_zero_agree_on_acceptance_and_length() {
    for name in ["ascii", "zero"] {
        let oracle = oracle(name);
        let ir = grammar(oracle["grammar"].as_str().unwrap());
        for case in oracle["cases"].as_array().unwrap() {
            let result = run(dfa(&ir), case["input"].as_str().unwrap());
            assert_eq!(
                result.is_some(),
                case["accepted"].as_bool().unwrap(),
                "{}",
                case["id"]
            );
            assert_eq!(
                result.unwrap_or(0) as u64,
                case["consumed"].as_u64().unwrap()
            );
            assert_eq!(
                result.unwrap_or(0) as u64,
                case["matched"].as_u64().unwrap()
            );
        }
    }
    let zero = pattern("a*");
    assert!(dfa(&zero).accepting.contains(&dfa(&zero).start));
    assert_eq!(run(dfa(&zero), ""), None);
}

#[test]
fn c15_non_bmp_oracle_differences_remain_explicit() {
    // Java oracle は UTF-16 Matcher.end と code point のカーソルを混在する。
    // regex-prefix 自体も非 BMP 接頭辞を含むので、一致ケースとして偽装しない。
    let prefix = oracle("prefix");
    let ir = grammar(prefix["grammar"].as_str().unwrap());
    for case in prefix["cases"].as_array().unwrap() {
        let input = case["input"].as_str().unwrap().strip_prefix('😀').unwrap();
        assert_eq!(run(dfa(&ir), input), Some(1));
        assert_eq!(case["accepted"], false);
        assert_eq!(case["consumed"], 0);
        assert_eq!(case["nativeFailure"]["consumed"], 1);
    }
    let emoji = oracle("emoji");
    let ir = grammar(emoji["grammar"].as_str().unwrap());
    for case in emoji["cases"].as_array().unwrap() {
        assert_eq!(run(dfa(&ir), case["input"].as_str().unwrap()), Some(1));
        assert_ne!(case["consumed"], 1);
    }
    assert_eq!(emoji["cases"][0]["accepted"], false);
    assert_eq!(emoji["cases"][1]["consumed"], 2);
    let backref = oracle("backref");
    let ir = grammar(backref["grammar"].as_str().unwrap());
    assert!(ir.plan.lexical[0].dfa.is_none());
    assert_eq!(backref["cases"][0]["consumed"], 4);
}

#[test]
fn optional_dfa_is_backward_compatible_and_inconsistent_plans_are_rejected() {
    let ir = pattern("[a-z]+");
    let mut json = serde_json::to_value(&ir).unwrap();
    json["semantics"]["tokens"][0]["spec"]
        .as_object_mut()
        .unwrap()
        .remove("dfa");
    let old = from_json(&json.to_string()).unwrap();
    assert_eq!(serde_json::to_value(&old).unwrap(), json);
    assert!(validate_references(&old).is_empty());
    let mut invalid = ir.clone();
    invalid.plan.lexical[0].dfa.as_mut().unwrap().states[0].accepting = true;
    assert!(validate_references(&invalid)
        .iter()
        .any(|e| e.to_string().contains("differs")));
    let mut invalid = ir;
    if let TokenSpec::Regex { fallback, .. } = &mut invalid.semantics.tokens[0].spec {
        *fallback = Some(RegexFallback);
    }
    assert!(validate_references(&invalid)
        .iter()
        .any(|e| e.to_string().contains("inconsistent")));
}
