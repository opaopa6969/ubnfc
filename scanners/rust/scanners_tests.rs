use super::*;

#[test]
fn long_fences_keep_width_line_boundaries_raw_span_and_atomic_failure() {
    let class = "org.unlaxer.tinyexpression.parser.javalang.LongCodeBlockParser";
    let prefix = "😀\n";
    for nl in ["\n", "\r\n", "\r"] {
        let block = format!(
            "`````rust:p.Demo{nl}// 😀 ```{nl}````{nl}``````{nl}````` {nl}x `````{nl}`````{nl}"
        );
        for match_only in [false, true] {
            for reset in [false, true] {
                let (c, m) = if match_only {
                    (0, prefix.len())
                } else {
                    (prefix.len(), 0)
                };
                let end = prefix.len() + block.len();
                let r = scan(
                    class,
                    &format!("{prefix}{block}tail"),
                    c,
                    m,
                    match_only,
                    false,
                    reset,
                );
                assert!(r.diagnostics.is_empty());
                result(
                    r,
                    true,
                    if match_only { c } else { end },
                    end,
                    [prefix.len(), end - nl.len()],
                );
                let r = scan(
                    class,
                    &format!("{prefix}{block}"),
                    c,
                    m,
                    match_only,
                    true,
                    reset,
                );
                assert_eq!(
                    r.diagnostics
                        .iter()
                        .map(|d| (d.offset, d.expected))
                        .collect::<Vec<_>>(),
                    [(prefix.len(), "long code block")]
                );
                result(r, false, c, m, [0, 0]);
                for bad in [
                    format!("````rust:A{nl}"),
                    format!("```rust:A{nl}```"),
                    format!("````型:A{nl}````"),
                    format!("````rust:A.{nl}````"),
                    "````rust:A".into(),
                    format!("````rust:A{nl}`````{nl}"),
                    format!("````rust:A{nl}```` {nl}"),
                    format!("````rust:A{nl}x ````{nl}"),
                ] {
                    let r = scan(
                        class,
                        &format!("{prefix}{bad}"),
                        c,
                        m,
                        match_only,
                        false,
                        reset,
                    );
                    assert_eq!(
                        r.diagnostics
                            .iter()
                            .map(|d| (d.offset, d.expected))
                            .collect::<Vec<_>>(),
                        [(prefix.len(), "long code block")]
                    );
                    result(r, false, c, m, [0, 0]);
                }
            }
        }
    }
    let empty = "````rust:A\n````";
    result(
        scan(class, empty, 0, 0, false, false, true),
        true,
        empty.len(),
        empty.len(),
        [0, empty.len()],
    );
    result(
        scan(class, &format!("x{empty}"), 1, 0, false, false, true),
        false,
        1,
        0,
        [0, 0],
    );
    // host 文字列の途中でも同幅の単独行で終わる。
    let opaque = "````rust:A\n\"string\n````\n";
    result(
        scan(class, &format!("{opaque}rest\""), 0, 0, false, false, true),
        true,
        opaque.len(),
        opaque.len(),
        [0, opaque.len() - 1],
    );
    assert!(Registry::new(&[("T", "other.LongCodeBlockParser")]).is_err());
}

fn scan(
    class: &'static str,
    input: &str,
    consumed: usize,
    matched: usize,
    match_only: bool,
    invert: bool,
    reset: bool,
) -> ScanResult {
    let scan = Scan {
        input,
        state: State {
            consumed,
            matched,
            invert,
            reset,
        },
        match_only,
        p: if match_only { matched } else { consumed },
        value_start: 0,
        value_end: 0,
        failure: 0,
        expected: "token",
    };
    // 実 registry と同じクラス解決。未知クラスは登録時に拒否する。
    scan.run(kind(class).unwrap())
}
fn result(r: ScanResult, ok: bool, consumed: usize, matched: usize, value: [usize; 2]) {
    assert_eq!(
        (r.ok, r.consumed_end, r.matched_end),
        (ok, consumed, matched),
        "{r:?}"
    );
    assert!(r.effects.is_empty());
    if ok {
        assert_eq!(r.value_span, value);
    } else {
        assert!(!r.diagnostics.is_empty());
    }
}
#[test]
fn external_failure_keeps_following_match_only_expectation() {
    let external = "oracle.ExternalCursorParser";
    for match_only in [false, true] {
        for (input, expected) in [
            ("a", vec![(0, "'😀'"), (0, "'x'")]),
            ("", vec![(0, "'😀'"), (0, "'x'")]),
            ("😀y", vec![(4, "'x'")]),
        ] {
            let r = scan(external, input, 0, 0, match_only, false, true);
            assert_eq!(
                r.diagnostics
                    .iter()
                    .map(|d| (d.offset, d.expected))
                    .collect::<Vec<_>>(),
                expected
            );
            result(r, false, 0, 0, [0, 0]);
        }
        assert!(scan(external, "😀x", 0, 0, match_only, false, true)
            .diagnostics
            .is_empty());
    }
}
#[test]
fn five_states_unicode_spans_and_transactional_failure() {
    let string = "org.unlaxer.tinyexpression.parser.StringLiteralParser";
    let start = "org.unlaxer.tinyexpression.parser.javalang.CodeStartParser";
    let end = "org.unlaxer.tinyexpression.parser.javalang.CodeEndParser";
    let external = "oracle.ExternalCursorParser";
    result(
        scan(string, "'😀'", 0, 0, false, false, true),
        true,
        6,
        6,
        [1, 5],
    );
    result(
        scan("DoubleQuotedParser", "\"\\😀\"", 0, 0, true, false, false),
        true,
        0,
        7,
        [0, 7],
    );
    result(
        scan(string, "''\"b\"", 0, 2, true, false, true),
        true,
        0,
        2,
        [1, 1],
    );
    result(
        scan(string, "''\"b\"", 0, 2, true, false, false),
        true,
        0,
        5,
        [2, 5],
    );
    result(
        scan(string, "'x", 0, 0, false, false, true),
        false,
        0,
        0,
        [0, 0],
    );
    result(
        scan(string, "'x'", 0, 0, false, true, true),
        false,
        0,
        0,
        [0, 0],
    );
    result(
        scan(external, "😀x", 0, 0, false, false, true),
        true,
        4,
        5,
        [0, 4],
    );
    result(
        scan(external, "😀x", 0, 0, true, false, false),
        true,
        0,
        5,
        [0, 4],
    );
    result(
        scan(external, "😀y", 0, 0, false, false, true),
        false,
        0,
        0,
        [0, 0],
    );
    result(
        scan(external, "ay", 0, 0, false, true, false),
        true,
        1,
        2,
        [0, 1],
    );
    result(
        scan(start, "x\r\n```java:a.B\r\n", 3, 3, false, false, true),
        true,
        16,
        16,
        [3, 14],
    );
    result(
        scan(end, "```\n", 0, 0, true, false, true),
        true,
        0,
        4,
        [0, 3],
    );
    for input in [" ```", "```x", "``", "````"] {
        result(
            scan(end, input, 0, 0, false, false, true),
            false,
            0,
            0,
            [0, 0],
        );
    }
    for input in [
        "```λ:a.B\n",
        "```java:a.$B\n",
        "```java:a..B\n",
        "```java:a.B.\n",
    ] {
        result(
            scan(start, input, 0, 0, false, false, true),
            false,
            0,
            0,
            [0, 0],
        );
    }
    assert!(Registry::new(&[("T", "other.StringLiteralParser")]).is_err());
}
