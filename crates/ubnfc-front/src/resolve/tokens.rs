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

pub(super) fn classify(value: &TokenValue) -> (ResolvedTokenValue, bool) {
    use ResolvedTokenValue as R;
    let value = match value {
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
        TokenValue::Negation { chars, .. } => R::Negation {
            chars: chars.clone(),
        },
        TokenValue::Lookahead { pattern, .. } => R::Lookahead {
            pattern: pattern.clone(),
        },
        TokenValue::NegativeLookahead { pattern, .. } => R::NegativeLookahead {
            pattern: pattern.clone(),
        },
        TokenValue::CharRange { min, max, .. } => R::CharRange {
            min: min.clone(),
            max: max.clone(),
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
