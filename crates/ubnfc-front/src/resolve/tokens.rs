use crate::syntax::ast::TokenValue;

use super::{BuiltinKind, ResolvedTokenValue};

const POSIX: &[&str] = &[
    "AlphabetParser",
    "AlphabetNumericParser",
    "AlphabetUnderScoreParser",
    "AlphabetNumericUnderScoreParser",
    "AsciiParser",
    "BlankParser",
    "ColonParser",
    "CommaParser",
    "ControlParser",
    "DotParser",
    "GraphParser",
    "HashParser",
    "LowerParser",
    "PrintParser",
    "PunctuationParser",
    "SemiColonParser",
    "SpaceParser",
    "UpperParser",
    "WordParser",
    "XDigitParser",
];
const ASCII: &[&str] = &[
    "BackSlashParser",
    "DivisionParser",
    "DoubleQuoteParser",
    "EqualParser",
    "GreaterThanParser",
    "LeftParenthesisParser",
    "LessThanParser",
    "MinusParser",
    "PlusParser",
    "PointParser",
    "RightParenthesisParser",
    "SlashParser",
];
// クラスパス探索は行わない。既知だが §A の内蔵スキャナ対象外のクラスは Extern。
const ELEMENTARY_EXTERN: &[&str] = &[
    "AbstractTokenParser",
    "DoubleQuotedParser",
    "EParser",
    "EmptyLineParser",
    "EmptyParser",
    "EndOfLineParser",
    "EscapeInQuotedParser",
    "ExponentParser",
    "IgnoreCaseWordParser",
    "LineTerminatorParser",
    "MappedSingleCharacterParser",
    "MultipleParser",
    "NamedParenthesesParser",
    "ParenthesesParser",
    "SignParser",
    "SingleCharacterParser",
    "SingleQuoteParser",
    "SingleStringParser",
    "SpaceDelimitor",
    "StartOfLineParser",
    "StartOfSourceParser",
    "WildCardCharacterParser",
    "WildCardInterleaveParser",
    "WildCardLineParser",
    "WildCardStringParser",
    "WildCardStringTerminatorParser",
    "WordParser",
];
const CLANG_EXTERN: &[&str] = &[
    "BlockComment",
    "CPPComment",
    "CStyleDelimitedLazyChain",
    "CStyleDelimitor",
    "CStyleDelimitorElements",
];

/// FQN は実在するパッケージと単純名の組を照合する。
/// `com.example.NumberParser` を内蔵 Number に誤分類しない。
pub fn builtin_class(class_name: &str) -> Option<BuiltinKind> {
    let (package, name) = class_name
        .rsplit_once('.')
        .map_or((None, class_name), |(p, n)| (Some(p), n));
    let in_package = |p: &str| package.is_none_or(|actual| actual == p);
    if in_package("org.unlaxer.parser.elementary") {
        let kind = match name {
            "NumberParser" => Some(BuiltinKind::Number),
            "SingleQuotedParser" => Some(BuiltinKind::SingleQuoted),
            "QuotedParser" => Some(BuiltinKind::Quoted),
            "EndOfSourceParser" => Some(BuiltinKind::EndOfSource),
            _ => None,
        };
        if kind.is_some() {
            return kind;
        }
        // 単純名 WordParser は探索順により elementary（引数付き）を指す。
        if name == "WordParser" {
            return None;
        }
    }
    if in_package("org.unlaxer.parser.clang") && name == "IdentifierParser" {
        return Some(BuiltinKind::Identifier);
    }
    if in_package("org.unlaxer.parser.posix") {
        if name == "DigitParser" {
            return Some(BuiltinKind::Digit);
        }
        if let Some(&name) = POSIX.iter().find(|&&n| n == name) {
            return Some(BuiltinKind::CharacterSet(name));
        }
    }
    if in_package("org.unlaxer.parser.ascii") {
        if let Some(&name) = ASCII.iter().find(|&&n| n == name) {
            return Some(BuiltinKind::CharacterSet(name));
        }
    }
    None
}

/// `v2` は `@ubnf: 2` 以上を宣言した grammar かどうか（D-038）。v2 のときだけ
/// `CHAR_RANGE` の境界と `NEGATION` の除外集合で `\uXXXX` / `\u{X…}` を展開する
/// （D-062、`crate::escape`）。展開に失敗した場合は v1 の字句値をそのまま残し、
/// validate の `E-TOKEN-ESCAPE-*` が理由を述べる（lower には入らない）。
pub(super) fn classify(value: &TokenValue, v2: bool) -> (ResolvedTokenValue, bool) {
    use ResolvedTokenValue as R;
    let expand = |literal: &String, raw: &str| -> String {
        if v2 {
            crate::escape::expand(raw).unwrap_or_else(|_| literal.clone())
        } else {
            literal.clone()
        }
    };
    let value = match value {
        // 中立名は「同じ FQN を書いたとき」と同じ分類器を通す。これで受理範囲が
        // 02-facts §A の表と一致することが構成的に保証される（D-041）。
        TokenValue::Builtin { name, .. } => {
            let kind = crate::vocabulary::builtin_fqn(name)
                .and_then(builtin_class)
                .expect("語彙表の項目は組込みに分類される（vocabulary のテストが固定）");
            return (R::Builtin { kind }, false);
        }
        // 明示された host 実装。未解決警告（W-TOKEN-UNRESOLVED）は綴り間違いを
        // 拾うためのもので、明示宣言には出さない。registry の有無は ir/gen が見る。
        TokenValue::Extern { name, .. } => {
            return (
                R::Extern {
                    class_name: name.clone(),
                },
                false,
            )
        }
        TokenValue::ClassRef { dotted, .. } => {
            if let Some(kind) = builtin_class(dotted) {
                return (R::Builtin { kind }, false);
            }
            if !dotted.contains('.') {
                for (package, names) in [
                    ("org.unlaxer.parser.elementary", ELEMENTARY_EXTERN),
                    ("org.unlaxer.parser.clang", CLANG_EXTERN),
                ] {
                    if names.contains(&dotted.as_str()) {
                        return (
                            R::Extern {
                                class_name: format!("{package}.{dotted}"),
                            },
                            false,
                        );
                    }
                }
            }
            return (
                R::Extern {
                    class_name: dotted.clone(),
                },
                !dotted.is_empty() && !dotted.contains('.'),
            );
        }
        TokenValue::Until { terminator, .. } => R::Until {
            terminator: terminator.clone(),
        },
        TokenValue::Negation {
            chars, chars_raw, ..
        } => R::Negation {
            chars: expand(chars, chars_raw),
        },
        TokenValue::Lookahead { pattern, .. } => R::Lookahead {
            pattern: pattern.clone(),
        },
        TokenValue::NegativeLookahead { pattern, .. } => R::NegativeLookahead {
            pattern: pattern.clone(),
        },
        TokenValue::CharRange {
            min,
            max,
            min_raw,
            max_raw,
            ..
        } => R::CharRange {
            min: expand(min, min_raw),
            max: expand(max, max_raw),
        },
        TokenValue::CaseInsensitive { word, .. } => R::CaseInsensitive { word: word.clone() },
        TokenValue::Regex { pattern, .. } => R::Regex {
            pattern: pattern.clone(),
        },
        TokenValue::Any { .. } => R::Any,
        TokenValue::Eof { .. } => R::Eof,
        TokenValue::Empty { .. } => R::Empty,
    };
    (value, false)
}
