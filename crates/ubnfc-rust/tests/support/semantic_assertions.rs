#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! prefix {
        ($m:ident,$text:expr) => {
            $m::parse_with_options(
                $text,
                $m::ParseOptions {
                    build_ast: true,
                    require_eof: false,
                    lexical: true,
                    ..Default::default()
                },
            )
        };
    }
    #[test]
    fn cursors_and_progress() {
        let r = prefix!(empty, "ab");
        assert!(r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (0, 1));
        assert!(!prefix!(empty_two, "ab").ok);
        assert!(prefix!(empty_once, "ab").ok);
        assert!(!prefix!(zero_max, "a").ok);
        assert!(zero_max::parse("").ok);
        let r = prefix!(look, "😀x");
        assert!(r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (0, 1));
        let r = prefix!(look_consume, "ab");
        assert!(r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (1, 1));
        let r = prefix!(repeat_matched, "aa");
        assert!(r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (0, 2));
        let r = prefix!(terminator, "a#");
        assert!(r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (1, 2));
        let r = prefix!(until, "a#");
        assert!(r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (0, 2));
    }
    #[test]
    fn choice_capture_and_diagnostics() {
        let r = prefix!(negative, "b");
        assert!(r.ok);
        assert!(r.diagnostics.is_empty());
        assert!(!prefix!(negative, "a").ok);
        // 出現表は opt-in（D-035）。既定は空で、AST はそのまま作られる。
        let plain = nested::parse("");
        assert!(plain.captures.is_empty() && plain.ast.is_some());
        macro_rules! occ {
            ($m:ident,$text:expr) => {
                $m::parse_with_options(
                    $text,
                    $m::ParseOptions {
                        occurrences: true,
                        ..Default::default()
                    },
                )
            };
        }
        let r = occ!(nested, "");
        assert_eq!(
            r.captures.iter().map(|c| c.name).collect::<Vec<_>>(),
            vec!["inner", "outer"]
        );
        assert_eq!(r.captures[0].span, [0, 0]);
        let r = occ!(longest, "ab");
        assert!(r.ok);
        assert_eq!(r.captures.len(), 1);
        assert_eq!(r.captures[0].name, "long");
        let r = occ!(longest_tie, "a");
        assert!(r.ok);
        assert_eq!(r.captures[0].name, "first");
        assert!(predict::parse("😀").ok);
        assert!(!predict::parse("z").ok);
        // 成功した parse の投機的失敗（hints）は診断モードでだけ残る（fast mode は記録しない）。
        let predicted = predict::parse_with_options(
            "😀",
            predict::ParseOptions {
                diagnostics: true,
                ..Default::default()
            },
        );
        assert!(predicted.ok);
        assert_eq!(predicted.hints[0].expected, vec!["literal a"]);
        assert!(predict::parse("😀").hints.is_empty());
        assert!(!prefix::parse("variable").ok);
        assert_eq!(prefix::parse("variable").consumed_cp, 3);
        let r = prefix!(separated, "a,");
        assert!(r.ok);
        assert_eq!(r.consumed_cp, 1);
        let r = memo::parse_with_options(
            "a",
            memo::ParseOptions {
                lexical: true,
                diagnostics: true,
                ..Default::default()
            },
        );
        assert!(r.ok);
        assert!(r.statistics.memo_hits > 0);
        assert_eq!(r.captures.len(), 1);
        assert_eq!(r.captures[0].span, [0, 1]);
        assert_eq!(r.hints[0].offset_cp, 1);
        assert_eq!(r.lexical.len(), 2);
        assert_eq!(r.tokens.len(), 1);
        assert_eq!(r.lexical[1].parent_occurrence_id, Some(0));
    }
    #[test]
    fn scope_rollback_and_replay() {
        // `Decl '!' | Decl Use` は既定で左因数分解され Decl は 1 回だけ評価される。
        // memo hit での効果の巻戻し・再生は参照経路（factor off）で検証する。
        let r = scope::parse("aa");
        assert!(r.ok);
        assert_eq!(r.declarations.len(), 1);
        assert_eq!(r.references.len(), 1);
        assert!(r.scope_diagnostics.is_empty());
        let factored = r;
        let r = scope::parse_with_options(
            "aa",
            scope::ParseOptions {
                factor: false,
                ..Default::default()
            },
        );
        assert!(r.ok);
        assert!(r.statistics.memo_hits > 0);
        assert_eq!(r.declarations, factored.declarations);
        assert_eq!(r.references, factored.references);
        assert_eq!(r.scope, factored.scope);
        assert!(r.scope_diagnostics.is_empty());
        let r = scope::parse_with_options(
            "aa",
            scope::ParseOptions {
                build_ast: false,
                ..Default::default()
            },
        );
        assert_eq!(r.declarations.len(), 1);
        assert_eq!(r.references.len(), 1);
    }
    #[test]
    fn number_and_dfa() {
        let r = prefix!(number, "1e+");
        assert!(r.ok);
        assert_eq!(r.consumed_cp, 1);
        assert!(r.diagnostics.is_empty());
        assert!(number::parse("-.5e+2").ok);
        assert!(!number::parse(".").ok);
        assert!(dfa::parse("abc").ok);
        assert!(!dfa::parse("").ok);
    }
    #[test]
    fn scanner_boundaries() {
        struct Scanner;
        impl external::TokenScanner for Scanner {
            fn scan(
                &mut self,
                _: &str,
                _: &str,
                s: external::State,
                _: bool,
                _: &external::rt::scope::ScopeStore,
            ) -> external::ScanResult {
                external::ScanResult {
                    ok: true,
                    consumed_end: s.consumed + 4,
                    matched_end: s.consumed + 4,
                    value_span: [s.consumed, s.consumed + 4],
                    diagnostics: vec![],
                    effects: vec![external::ScanEffect::Declare {
                        name: "x".into(),
                        offset_cp: 0,
                    }],
                }
            }
        }
        let r = external::parse_with_scanner("😀", Default::default(), &mut Scanner);
        assert!(r.ok);
        assert_eq!(r.consumed_cp, 1);
        assert_eq!(r.declarations.len(), 1);
        assert!(!external::parse_with_scanner("a", Default::default(), &mut Scanner).ok);
    }
    #[test]
    fn mapping_types_and_failure() {
        let r = mapped_record::parse("a");
        assert!(r.ok);
        let ast = r.ast.unwrap();
        assert_eq!(ast.type_name(), "Item");
        assert_eq!(ast.common_0_g_value().map(String::as_str), Some("a"));
        let r = mapping_failure::parse("aa");
        assert!(r.ok);
        assert!(r.ast.is_none());
        assert_eq!(
            r.diagnostics.last().unwrap().kind,
            mapping_failure::DiagnosticKind::Mapping
        );
        let r = enumeration::parse("red");
        assert!(r.ok);
        assert_eq!(r.ast.unwrap().type_name(), "Color");
        assert!(cached_trivia::parse("a /* c */ b").ok);
    }
    #[cfg(feature = "json")]
    #[test]
    fn enum_canonical_json() {
        assert_eq!(
            enumeration::parse("red").ast.unwrap().canonical_json(),
            "\"RED\""
        );
        let ast = mapped_record::parse("a").ast.unwrap();
        assert_eq!(
            ast.canonical_value(),
            serde_json::json!({"type":"Item","span":[0,1],"fields":{"value":"a"}})
        );
        assert_eq!(serde_json::to_string(&ast).unwrap(), ast.canonical_json());
    }
    #[test]
    fn builtin_tokens() {
        assert!(space_token::parse(" ").ok);
        assert!(!space_token::parse("").ok);
        assert_eq!(prefix!(space_token, "  ").consumed_cp, 1);
        assert!(digit::parse("3").ok);
        assert!(!digit::parse("٣").ok);
        assert!(alphabet::parse("A").ok);
        assert!(underscore::parse("_").ok);
        assert!(alnum::parse("9").ok);
        assert!(cp_set::parse("😀").ok);
        assert!(identifier::parse("a_7").ok);
        assert!(!identifier::parse("é").ok);
        assert!(quoted::parse("\"a\\😀b\"").ok);
        assert!(single_quoted::parse("'a'").ok);
        assert!(eof_token::parse("").ok);
        assert!(!eof_token::parse("x").ok);
        assert!(case_insensitive::parse("KİſS").ok);
        assert!(char_range::parse("a").ok);
        assert!(!char_range::parse("0").ok);
        // D-063: 補助面（U+1F600）は 1 文字として通り、BMP の U+D7FF 未満は落ちる。
        assert!(char_range_supplementary::parse("\u{1f600}").ok);
        assert!(char_range_supplementary::parse("\u{e000}").ok);
        assert!(!char_range_supplementary::parse("a").ok);
        assert!(!char_range_supplementary::parse("\u{d7ff}").ok);
        assert!(negation::parse("😀").ok);
        assert!(!negation::parse("a").ok);
    }
    /// D-040: 語境界。境界あり／なしの差が「直後が識別子構成文字か」だけであること、
    /// 境界違反の失敗記録がリテラル不一致と同じ（同じ位置・同じ候補）であることを固定する。
    #[test]
    fn literal_word_boundary() {
        // 境界なし（v1）は従来どおり接頭辞一致。
        assert!(prefix!(boundary_off, "letter").ok);
        assert!(prefix!(boundary_off, "let_").ok);
        assert!(prefix!(boundary_off, "let9").ok);
        // 境界あり（v2 既定）は直後が ASCII [A-Za-z0-9_] なら不一致。
        assert!(!prefix!(boundary_on, "letter").ok);
        assert!(!prefix!(boundary_on, "let_").ok);
        assert!(!prefix!(boundary_on, "let9").ok);
        assert!(boundary_on::parse("let").ok);
        assert!(prefix!(boundary_on, "let+x").ok);
        assert!(prefix!(boundary_on, "let あ").ok);
        assert!(prefix!(boundary_on, "let😀").ok, "非 ASCII は識別子構成文字でない");
        // 境界違反の観測は「一致しなかった」と同じ: 同じ位置、同じ expected。
        let violated = boundary_on::parse("letter");
        let mismatched = boundary_on::parse("lex");
        assert!(!violated.ok && !mismatched.ok);
        assert_eq!(violated.consumed_cp, mismatched.consumed_cp);
        let shape = |r: &boundary_on::ParseResult| {
            r.diagnostics
                .iter()
                .map(|d| (d.kind, d.offset_cp, d.expected.clone(), d.farthest_cp))
                .collect::<Vec<_>>()
        };
        assert_eq!(shape(&violated), shape(&mismatched));
        // backend は flag をそのまま実行する（どのリテラルに立てるかは front の責務）。
        // 境界は「リテラル全体の直後」を見るので、記号を含む `'let+'` でも位置は末尾の次。
        assert!(boundary_symbol::parse("let+").ok);
        assert!(!prefix!(boundary_symbol, "let+x").ok);
        // 語境界があると `'var' | 'variable'` の後続候補が生きる（D-037 / W-CHOICE-PREFIX）。
        assert_eq!(prefix!(boundary_prefix_choice, "variable").consumed_cp, 8);
        assert_eq!(prefix!(boundary_prefix_choice, "var;").consumed_cp, 3);
        // CI token も同じ規則。
        assert!(prefix!(case_insensitive, "KISSx").ok);
        assert!(!prefix!(case_insensitive_boundary, "KISSx").ok);
        assert!(case_insensitive_boundary::parse("KISS").ok);
    }

    #[test]
    fn local_backreference() {
        assert!(backref::parse("aa").scope_diagnostics.is_empty());
        let r = backref::parse("ab");
        assert!(r.ok);
        assert!(r.declarations.is_empty());
        assert_eq!(r.scope_diagnostics.len(), 1);
        assert_eq!(r.scope_diagnostics[0].severity, backref::Severity::Error);
    }
}
