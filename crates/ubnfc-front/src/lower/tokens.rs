use super::*;
use r::{BuiltinKind as B, ResolvedTokenValue as T};

pub(super) fn lower(cx: &mut Context<'_>, grammar: &r::Grammar) {
    for token in &grammar.tokens {
        let assertion = grammar.settings.iter().find_map(|s| match s {
            a::GlobalSetting::String { key, value, .. }
                if key == "memoSafeToken" && value == &token.name =>
            {
                Some((value.clone(), s.span()))
            }
            _ => None,
        });
        let asserted = assertion.as_ref().map(|(name, _)| name.clone());
        let original = Some(cx.text(token.unit)[token.span.start..token.span.end].to_owned());
        let external = matches!(token.value, T::Extern { .. });
        let safe = !external || asserted.is_some();
        let mut annotations = vec![];
        if let Some(alias) = &asserted {
            annotations.push(Annotation {
                name: "memoSafeToken".into(),
                arguments: BTreeMap::from([("alias".into(), JsonValue::String(alias.clone()))]),
                source: cx.span(grammar.unit, assertion.as_ref().unwrap().1),
            });
        }
        let spec = match &token.value {
            T::Builtin { kind } => {
                let (which, code_points) = match kind {
                    B::Number => (BuiltinToken::Number, vec![]),
                    B::Identifier => (BuiltinToken::Identifier, vec![]),
                    B::SingleQuoted => (BuiltinToken::SingleQuoted, vec![]),
                    B::Quoted => (BuiltinToken::Quoted, vec![]),
                    B::EndOfSource => (BuiltinToken::EndOfSource, vec![]),
                    B::Digit => (BuiltinToken::Digit, vec![]),
                    B::CharacterSet(name) => (
                        match *name {
                            "AlphabetParser" => BuiltinToken::Alphabet,
                            "AlphabetUnderScoreParser" => BuiltinToken::AlphabetUnderScore,
                            "AlphabetNumericUnderScoreParser" => {
                                BuiltinToken::AlphabetNumericUnderScore
                            }
                            "SpaceParser" => BuiltinToken::Space,
                            _ => BuiltinToken::CodePointSet,
                        },
                        code_points(name),
                    ),
                };
                TokenSpec::Builtin { which, code_points }
            }
            T::Extern { class_name } => TokenSpec::Extern {
                logical_id: cx.sid(token.id),
                original_class_name: class_name.clone(),
                java_binding: Some(class_name.clone()),
                rust_binding: None,
                input_state: vec![
                    "consumed".into(),
                    "matched".into(),
                    "mode".into(),
                    "invertMatch".into(),
                    "resetMatchedWithConsumed".into(),
                ],
                result_fields: TokenSpecExternResultFields,
            },
            T::Until { terminator } => TokenSpec::Until {
                terminator: terminator.clone(),
                empty_terminator: TokenSpecUntilEmptyTerminator,
                missing_terminator: TokenSpecUntilMissingTerminator,
                preserve_matched: TokenSpecUntilPreserveMatched,
            },
            T::Negation { chars } => TokenSpec::Negation {
                excluded: chars.clone(),
            },
            T::Lookahead { pattern } => TokenSpec::Lookahead {
                pattern: pattern.clone(),
            },
            T::NegativeLookahead { pattern } => TokenSpec::NegativeLookahead {
                pattern: pattern.clone(),
            },
            T::CharRange { min, max } => TokenSpec::CharRange {
                min: min.chars().next().unwrap() as u64,
                max: max.chars().next().unwrap() as u64,
            },
            T::CaseInsensitive { word } => TokenSpec::CaseInsensitive {
                word: word.clone(),
                comparison: TokenSpecCaseInsensitiveComparison,
            },
            // 正規部分集合の証明は字句 analysis の責務。未証明を DFA 対応と偽らない。
            T::Regex { pattern } => TokenSpec::Regex {
                dfa: None,
                fallback: None,
                pattern: pattern.clone(),
                regular_subset: false,
                java_fallback: TokenSpecRegexJavaFallback::JavaUtilRegexPattern,
                matching: TokenSpecRegexMatching,
                rust_fallback: TokenSpecRegexRustFallback,
                zero_width: TokenSpecRegexZeroWidth,
            },
            T::Any => TokenSpec::Any {},
            T::Eof => TokenSpec::Eof {},
            T::Empty => TokenSpec::Empty {},
        };
        let profile = match &spec {
            TokenSpec::Builtin { which, .. } => format!("{which:?}"),
            _ => "Token".into(),
        };
        let profiles = TokenDiagnosticProfiles {
            java: format!("legacyJava{profile}"),
            rust: format!("legacyRust{profile}"),
        };
        let reason = match &token.declaration {
            a::TokenValue::ClassRef { dotted, .. } if !external => {
                format!("既知の {dotted} の専用スキャナ。受理と診断を組込み契約で再現する。")
            }
            _ if external => "extern token: alias assertion required".into(),
            _ => "known deterministic builtin".into(),
        };
        cx.out.semantics.tokens.push(Token {
            symbol_id: cx.sid(token.id),
            spec,
            annotations,
            diagnostic_profiles: profiles,
            memo_safety: MemoSafety {
                asserted_alias: asserted.clone(),
                original_declaration: original,
                success: safe,
                failure: safe,
                provenance: if asserted.is_some() {
                    MemoSafetyProvenance::UserAliasAssertion
                } else if external {
                    MemoSafetyProvenance::None
                } else {
                    MemoSafetyProvenance::BuiltinProof
                },
                purity: if safe {
                    MemoSafetyPurity::Pure
                } else {
                    MemoSafetyPurity::Unknown
                },
                reason,
            },
        });
    }
}
fn code_points(name: &str) -> Vec<u64> {
    (0u8..=127)
        .filter(|c| match name {
            "AlphabetParser" => c.is_ascii_alphabetic(),
            "AlphabetNumericParser" => c.is_ascii_alphanumeric(),
            "AlphabetUnderScoreParser" => c.is_ascii_alphabetic() || *c == b'_',
            "AlphabetNumericUnderScoreParser" | "WordParser" => {
                c.is_ascii_alphanumeric() || *c == b'_'
            }
            "AsciiParser" => true,
            "BlankParser" => b" \t".contains(c),
            "ControlParser" => c.is_ascii_control(),
            "GraphParser" => (33..=126).contains(c),
            "PrintParser" => (32..=126).contains(c),
            "LowerParser" => c.is_ascii_lowercase(),
            "UpperParser" => c.is_ascii_uppercase(),
            "PunctuationParser" => c.is_ascii_punctuation(),
            "SpaceParser" => [9, 10, 11, 12, 13, 32].contains(c),
            "XDigitParser" => c.is_ascii_hexdigit(),
            _ => {
                (match name {
                    "ColonParser" => b':',
                    "CommaParser" => b',',
                    "DotParser" | "PointParser" => b'.',
                    "HashParser" => b'#',
                    "SemiColonParser" => b';',
                    "BackSlashParser" => b'\\',
                    "DivisionParser" | "SlashParser" => b'/',
                    "DoubleQuoteParser" => b'"',
                    "EqualParser" => b'=',
                    "GreaterThanParser" => b'>',
                    "LeftParenthesisParser" => b'(',
                    "LessThanParser" => b'<',
                    "MinusParser" => b'-',
                    "PlusParser" => b'+',
                    "RightParenthesisParser" => b')',
                    _ => 255,
                }) == *c
            }
        })
        .map(u64::from)
        .collect()
}
