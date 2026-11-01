#[cfg(test)]
mod review {
    use super::*;
    macro_rules! parse {
        ($m:ident,$input:expr,$memo:expr) => {
            $m::parse_with_options(
                $input,
                $m::ParseOptions {
                    build_ast: false,
                    require_eof: false,
                    memo: $memo,
                    ..Default::default()
                },
            )
        };
    }
    #[test]
    fn empty_word_and_iteration_boundaries() {
        for memo in [false, true] {
            for input in ["", "ab"] {
                let r = parse!(empty_look, input, memo);
                assert!(!r.ok);
                assert_eq!((r.consumed_cp, r.matched_cp), (0, 0));
                let r = parse!(empty_not, input, memo);
                assert!(r.ok);
                assert_eq!((r.consumed_cp, r.matched_cp), (0, 0));
                assert!(r.diagnostics.is_empty());
            }
            let r = parse!(repeat_two, "abc", memo);
            assert!(r.ok);
            assert_eq!((r.consumed_cp, r.matched_cp), (0, 2));
            let r = parse!(repeat_three, "abc", memo);
            assert!(r.ok);
            assert_eq!((r.consumed_cp, r.matched_cp), (0, 3));
            let r = parse!(optional_reset, "ab", memo);
            assert!(r.ok);
            assert_eq!((r.consumed_cp, r.matched_cp), (0, 0));
            let r = parse!(empty_zero, "ab", memo);
            assert!(r.ok);
            assert_eq!((r.consumed_cp, r.matched_cp), (0, 0));
            assert!(r.diagnostics.is_empty());
            let r = parse!(empty_two, "ab", memo);
            assert!(!r.ok);
            assert_eq!(r.diagnostics[0].offset_cp, 0);
            assert!(!r.diagnostics[0].expected.is_empty());
            for input in ["a", "😀"] {
                let r = parse!(eof, input, memo);
                assert!(!r.ok);
                assert_eq!(r.diagnostics[0].offset_cp, 0);
                assert!(!r.diagnostics[0].expected.is_empty());
            }
        }
    }
    #[test]
    fn not_diagnostics_and_replay() {
        for memo in [false, true] {
            let r = parse!(not, "ab", memo);
            assert!(!r.ok);
            let d = &r.diagnostics[0];
            assert_eq!(d.offset_cp, 1);
            assert_eq!(d.expected, vec!["'x'"]);
            assert_eq!(d.farthest_expected, d.expected);
            let r = parse!(not, "b", memo);
            assert!(r.ok);
            assert!(r.diagnostics.is_empty());
            let r = parse!(not_replay, "b", memo);
            assert!(!r.ok);
            assert_eq!(r.diagnostics[0].expected, vec!["'x'"]);
            assert_eq!(r.statistics.memo_hits > 0, memo);
        }
    }
    #[test]
    fn rust_labels_and_primary_vs_farthest() {
        for memo in [false, true] {
            let r = parse!(ci, "le", memo);
            assert!(!r.ok);
            assert_eq!(r.diagnostics[0].offset_cp, 0);
            assert_eq!(r.diagnostics[0].expected, vec!["CUSTOM_CI"]);
            for input in ["", "b"] {
                let r = parse!(regex, input, memo);
                assert!(!r.ok);
                assert_eq!(r.diagnostics[0].offset_cp, 0);
                assert_eq!(r.diagnostics[0].expected, vec!["REGEX('a*')"]);
            }
            let r = trailing::parse_with_options(
                "abq",
                trailing::ParseOptions {
                    memo,
                    ..Default::default()
                },
            );
            assert!(!r.ok);
            assert_eq!(r.diagnostics.len(), 1);
            let d = &r.diagnostics[0];
            assert_eq!(d.kind, trailing::DiagnosticKind::TrailingInput);
            assert_eq!((d.offset_cp, d.farthest_cp), (1, 2));
            assert_eq!(d.expected, vec!["end of input"]);
            assert_eq!(d.farthest_expected, vec!["'x'"]);
            // 現 pin の Classic は仮数への rollback 後も不完全指数の失敗位置を保つ。
            let r = parse!(number, "neg(1e+)", memo);
            assert!(!r.ok);
            assert_eq!(r.diagnostics[0].offset_cp, 7);
            for (input, offset) in [
                ("a//x\nb", 5),
                ("a//x\r\nb", 6),
                ("a#x\nb", 1),
                ("a//😀\nb", 5),
            ] {
                let r = parse!(comment, input, memo);
                assert!(!r.ok);
                assert_eq!(r.diagnostics[0].offset_cp, offset, "{input:?}");
            }
        }
    }
    #[test]
    fn recursion_aborts_optional_and_recovers_next_call() {
        assert_eq!(recursive::ParseOptions::default().max_depth, 10_000);
        for memo in [false, true] {
            let options = recursive::ParseOptions {
                build_ast: false,
                require_eof: false,
                memo,
                max_depth: 8,
                ..Default::default()
            };
            let r = recursive::parse_with_options(
                &format!("{}a{}", "(".repeat(7), ")".repeat(7)),
                options,
            );
            assert!(r.ok, "{r:?}");
            let r = recursive::parse_with_options(
                &format!("{}a{}", "(".repeat(8), ")".repeat(8)),
                options,
            );
            assert!(!r.ok);
            assert_eq!((r.consumed_cp, r.matched_cp), (0, 0));
            assert_eq!(r.diagnostics[0].kind, recursive::DiagnosticKind::Syntax);
            assert_eq!(r.diagnostics[0].offset_cp, 8);
            assert_eq!(
                r.diagnostics[0].expected,
                vec!["maximum parse depth exceeded"]
            );
            assert!(r.scope.events.is_empty());
            assert!(parse!(recursive, "a", memo).ok);
        }
        // The child process running this generated test must survive native stack pressure.
        // issue #35 / D-070: 2,000 段は `max_depth`（既定 10,000）の範囲内なので、native stack
        // 番兵に呼出し元 thread（ここではわざと 1–2 MiB に絞っている）の上で当たっても、
        // 専用の大きい thread へ 1 度だけエスカレーションして通る（以前はここで
        // 誤って `maximum parse depth exceeded` になっていた＝本 issue の再現条件）。
        // 20,000 段は `max_depth` を超えるので、エスカレーション後の thread でも
        // 論理上限にちょうど当たり、クラッシュせず同じ診断になる。
        for stack_size in [1024 * 1024, 2 * 1024 * 1024] {
            std::thread::Builder::new()
                .stack_size(stack_size)
                .spawn(|| {
                    let input = format!("{}a{}", "(".repeat(2000), ")".repeat(2000));
                    let r = parse!(recursive, &input, true);
                    assert!(r.ok, "{r:?}");
                    let input = format!("{}a{}", "(".repeat(20_000), ")".repeat(20_000));
                    let r = parse!(recursive, &input, true);
                    assert!(!r.ok);
                    assert_eq!(
                        r.diagnostics[0].expected,
                        vec!["maximum parse depth exceeded"]
                    );
                    assert!(parse!(recursive, "a", true).ok);
                })
                .unwrap()
                .join()
                .unwrap();
        }
    }
    macro_rules! scope_check {
        ($m:ident,$mode:expr) => {{
            let mut results = Vec::new();
            for memo in [false, true] {
                let r = parse!($m, "a", memo);
                assert!(r.ok);
                assert_eq!(r.statistics.memo_hits > 0, memo);
                assert_eq!(r.scope.mode, vec![$mode]);
                let events = &r.scope.events;
                assert_eq!(
                    events.iter().map(|e| e.action).collect::<Vec<_>>(),
                    vec!["enter", "leave", "declare", "use"]
                );
                for (i, event) in events.iter().enumerate() {
                    assert_eq!(event.order, i);
                    assert_eq!(event.mode, $mode);
                }
                assert_eq!(
                    events.iter().map(|e| e.offset_cp).collect::<Vec<_>>(),
                    vec![0, 1, 0, 0]
                );
                assert_eq!(
                    events.iter().map(|e| e.length_cp).collect::<Vec<_>>(),
                    vec![0, 0, 1, 1]
                );
                assert_eq!(r.declarations.len(), 1);
                assert_eq!(r.references.len(), 1);
                assert!(parse!($m, "b", memo).scope.events.is_empty());
                let peek = $m::parse_entry_with_options(
                    "Operators",
                    Some("Peek"),
                    "a",
                    $m::ParseOptions {
                        build_ast: false,
                        require_eof: false,
                        memo,
                        ..Default::default()
                    },
                )
                .unwrap();
                assert!(peek.ok);
                assert_eq!((peek.consumed_cp, peek.matched_cp), (0, 1));
                assert!(peek.scope.events.is_empty());
                results.push(r.scope);
            }
            assert_eq!(results[0], results[1]);
        }};
    }
    #[test]
    fn scope_order_mode_rollback_and_memo_replay() {
        scope_check!(lexical, "lexical");
        scope_check!(dynamic, "dynamic");
    }
    #[test]
    fn scanner_receives_five_state_fields() {
        struct Scanner(usize);
        impl scanner::TokenScanner for Scanner {
            fn scan(
                &mut self,
                id: &str,
                _: &str,
                state: scanner::State,
                mode: bool,
                _: &scanner::rt::scope::ScopeStore,
            ) -> scanner::ScanResult {
                self.0 += 1;
                assert_eq!(id, "scan");
                assert_eq!(
                    (
                        state.consumed,
                        state.matched,
                        mode,
                        state.invert,
                        state.reset
                    ),
                    (0, 1, true, false, false)
                );
                scanner::ScanResult {
                    ok: true,
                    consumed_end: 0,
                    matched_end: 2,
                    value_span: [1, 2],
                    diagnostics: vec![],
                    effects: vec![scanner::ScanEffect::Enter],
                }
            }
        }
        for memo in [false, true] {
            let mut scanner = Scanner(0);
            let r = scanner::parse_with_scanner(
                "ab",
                scanner::ParseOptions {
                    build_ast: false,
                    require_eof: false,
                    memo,
                    // scanner の呼出し回数を数えるので 1 回の解析に固定する（既定の fast mode は
                    // 入力が残った結果を診断モードで解析し直すため、scanner が 2 回呼ばれる）。
                    diagnostics: true,
                    ..Default::default()
                },
                &mut scanner,
            );
            assert!(r.ok);
            assert_eq!(scanner.0, 1);
            assert_eq!((r.consumed_cp, r.matched_cp), (0, 2));
            assert!(r.scope.events.is_empty());
        }
    }
    #[test]
    fn expression_memo_success_failure_effects_and_diagnostics() {
        struct Scanner {
            count: usize,
            ok: bool,
        }
        impl memo::TokenScanner for Scanner {
            fn scan(
                &mut self,
                _: &str,
                _: &str,
                state: memo::State,
                mode: bool,
                _: &memo::rt::scope::ScopeStore,
            ) -> memo::ScanResult {
                self.count += 1;
                assert_eq!(
                    (
                        state.consumed,
                        state.matched,
                        mode,
                        state.invert,
                        state.reset
                    ),
                    (0, 0, false, false, false)
                );
                let end = usize::from(self.ok);
                memo::ScanResult {
                    ok: self.ok,
                    consumed_end: end,
                    matched_end: end,
                    value_span: [0, end],
                    diagnostics: vec![memo::ScanDiagnostic {
                        offset: 0,
                        expected: "SCAN",
                    }],
                    effects: if self.ok {
                        vec![
                            memo::ScanEffect::Enter,
                            memo::ScanEffect::Leave,
                            memo::ScanEffect::Declare {
                                name: "a".into(),
                                offset_cp: 0,
                            },
                        ]
                    } else {
                        vec![]
                    },
                }
            }
        }
        for ok in [false, true] {
            let mut results = Vec::new();
            for enabled in [false, true] {
                let mut scanner = Scanner { count: 0, ok };
                let r = memo::parse_with_scanner(
                    "ay",
                    memo::ParseOptions {
                        build_ast: false,
                        memo: enabled,
                        // 同上。診断の比較もこのテストの主題なので診断モードで回す。
                        diagnostics: true,
                        ..Default::default()
                    },
                    &mut scanner,
                );
                assert_eq!(scanner.count, if enabled { 1 } else { 2 });
                assert_eq!(r.ok, ok);
                assert_eq!(r.declarations.len(), usize::from(ok));
                results.push(r);
            }
            assert_eq!(results[0].scope, results[1].scope);
            assert_eq!(results[0].diagnostics, results[1].diagnostics);
            assert_eq!(results[0].declarations, results[1].declarations);
        }
    }
    #[test]
    fn mixed_lists_keep_empty_and_all_occurrences() {
        for memo in [false, true] {
            for (input, expected) in [
                ("grammar G { R ::= 'a'; }", vec![]),
                (
                    "grammar G { @root @mapping(V) R ::= 'a'; }",
                    vec!["RootAnnotation", "MappingAnnotation"],
                ),
            ] {
                let r = selfhost::parse_with_options(
                    input,
                    selfhost::ParseOptions {
                        memo,
                        ..Default::default()
                    },
                );
                assert!(r.ok, "{r:?}");
                let ast = r.ast.unwrap().canonical_value();
                let annotations = ast["fields"]["grammars"][0]["fields"]["rules"][0]["fields"]
                    ["annotations"]
                    .as_array()
                    .unwrap();
                assert_eq!(
                    annotations
                        .iter()
                        .map(|a| a["type"].as_str().unwrap())
                        .collect::<Vec<_>>(),
                    expected
                );
                for a in annotations {
                    let span = a["span"].as_array().unwrap();
                    let text = &input
                        [span[0].as_u64().unwrap() as usize..span[1].as_u64().unwrap() as usize];
                    assert_eq!(
                        text.trim(),
                        if a["type"] == "RootAnnotation" {
                            "@root"
                        } else {
                            "@mapping(V)"
                        }
                    );
                }
            }
        }
    }
    #[test]
    fn identifiers_compile_and_preserve_public_names() {
        macro_rules! check {
            ($m:ident,$name:expr) => {{
                let r = $m::parse("a");
                assert!(r.ok);
                let ast = r.ast.unwrap().canonical_value();
                assert_eq!(ast["type"], $name);
                assert_eq!(ast["fields"][$name], serde_json::json!(["a"]));
            }};
        }
        check!(ident_keyword, "type");
        check!(ident_self, "Self");
        check!(ident_punctuation, "a-b");
        check!(ident_unicode, "変数");
        check!(ident_ast, "Ast");
    }
}
