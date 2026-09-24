//! UBNF v2 の**言語中立な組込み token 語彙**（D-041）。
//!
//! v1 の token 宣言は host 言語のクラス名を書く:
//!
//! ```ubnf
//! token NUMBER = org.unlaxer.parser.elementary.NumberParser
//! ```
//!
//! これは「Java と Rust の両方を出す言語中立な文法形式」を名乗る以上、層の破れである。
//! IR の側は既に組込み種別へ正規化しているので、直すべきは表層の語彙だけだった。
//!
//! この表が語彙の**唯一の定義**で、右列は `resolve::builtin_class` が分類する
//! 旧実装の完全クラス名である。中立名は「その FQN を書いたときと同じ述語」を指す、
//! という契約を**構成的に**満たす（`builtin_fqn` の結果をそのまま既存の分類器へ渡す）。
//! したがって受理範囲は `docs/design/02-facts-tokens-bench-corpus.md` §A の表そのままで、
//! 中立名の導入で 1 文字も変わらない。
//!
//! 表に載せていない組込みは、**FQN が違っても IR が 1 バイトも変わらない別名**だけ:
//!
//! | 省いた FQN | 同じ IR を作る中立名 | 理由 |
//! |---|---|---|
//! | `posix.PointParser` | `DOT`（`posix.DotParser`） | どちらも `CodePointSet [46]` |
//! | `ascii.DivisionParser` | `SLASH`（`ascii.SlashParser`） | どちらも `CodePointSet [47]` |
//!
//! `ANY` / `EOF` / `EMPTY` と `UNTIL(…)` / `CI(…)` / `NEGATION(…)` /
//! `CHAR_RANGE(…)` / `LOOKAHEAD(…)` / `NEGATIVE_LOOKAHEAD(…)` / `REGEX(…)` は
//! v1 から既に中立な綴りなので、この表には入らない（直すものが無い）。
//!
//! v2 の code point クラス（D-062）も**この表を動かさない**。新しい構成子を足さず、
//! 既にある `CHAR_RANGE(…)` / `NEGATION(…)` の引数に `\uXXXX` / `\u{X…}` を許し、
//! `CHAR_RANGE` の境界から BMP 制限を外しただけだからである（`crate::escape`）。
//! 「host 言語の綴りを表層から消す」という D-041 の目的から見ても、
//! `\u` は Java にも Rust にも寄らない Unicode の綴りなので中立性は保たれる。

/// 中立名 → 旧実装の完全クラス名。`resolve::builtin_class` がこの FQN を分類する。
///
/// 名前は **SCREAMING_SNAKE_CASE**。host のクラス名は慣習上 `UpperCamelCase` なので、
/// この形の綴りが実在の host クラスと衝突することは事実上なく、衝突した場合も
/// `extern '…'` が任意の名前を綴れる逃げ道になる（D-041）。
pub const BUILTIN_VOCABULARY: &[(&str, &str)] = &[
    ("ALNUM", "org.unlaxer.parser.posix.AlphabetNumericParser"),
    (
        "ALNUM_UNDERSCORE",
        "org.unlaxer.parser.posix.AlphabetNumericUnderScoreParser",
    ),
    ("ALPHA", "org.unlaxer.parser.posix.AlphabetParser"),
    (
        "ALPHA_UNDERSCORE",
        "org.unlaxer.parser.posix.AlphabetUnderScoreParser",
    ),
    ("ASCII", "org.unlaxer.parser.posix.AsciiParser"),
    ("BACKSLASH", "org.unlaxer.parser.ascii.BackSlashParser"),
    ("BLANK", "org.unlaxer.parser.posix.BlankParser"),
    ("COLON", "org.unlaxer.parser.posix.ColonParser"),
    ("COMMA", "org.unlaxer.parser.posix.CommaParser"),
    ("CONTROL", "org.unlaxer.parser.posix.ControlParser"),
    ("DIGIT", "org.unlaxer.parser.posix.DigitParser"),
    ("DOT", "org.unlaxer.parser.posix.DotParser"),
    ("DOUBLE_QUOTE", "org.unlaxer.parser.ascii.DoubleQuoteParser"),
    (
        "DOUBLE_QUOTED",
        "org.unlaxer.parser.elementary.QuotedParser",
    ),
    (
        "END_OF_INPUT",
        "org.unlaxer.parser.elementary.EndOfSourceParser",
    ),
    ("EQUAL", "org.unlaxer.parser.ascii.EqualParser"),
    ("GRAPH", "org.unlaxer.parser.posix.GraphParser"),
    ("GREATER_THAN", "org.unlaxer.parser.ascii.GreaterThanParser"),
    ("HASH", "org.unlaxer.parser.posix.HashParser"),
    ("IDENTIFIER", "org.unlaxer.parser.clang.IdentifierParser"),
    (
        "LEFT_PAREN",
        "org.unlaxer.parser.ascii.LeftParenthesisParser",
    ),
    ("LESS_THAN", "org.unlaxer.parser.ascii.LessThanParser"),
    ("LOWER", "org.unlaxer.parser.posix.LowerParser"),
    ("MINUS", "org.unlaxer.parser.ascii.MinusParser"),
    ("NUMBER", "org.unlaxer.parser.elementary.NumberParser"),
    ("PLUS", "org.unlaxer.parser.ascii.PlusParser"),
    ("PRINT", "org.unlaxer.parser.posix.PrintParser"),
    ("PUNCT", "org.unlaxer.parser.posix.PunctuationParser"),
    (
        "RIGHT_PAREN",
        "org.unlaxer.parser.ascii.RightParenthesisParser",
    ),
    ("SEMICOLON", "org.unlaxer.parser.posix.SemiColonParser"),
    (
        "SINGLE_QUOTED",
        "org.unlaxer.parser.elementary.SingleQuotedParser",
    ),
    ("SLASH", "org.unlaxer.parser.ascii.SlashParser"),
    ("SPACE", "org.unlaxer.parser.posix.SpaceParser"),
    ("UPPER", "org.unlaxer.parser.posix.UpperParser"),
    ("WORD_CHAR", "org.unlaxer.parser.posix.WordParser"),
    ("XDIGIT", "org.unlaxer.parser.posix.XDigitParser"),
];

/// 中立名なら、それが指す旧実装の完全クラス名を返す。
pub fn builtin_fqn(name: &str) -> Option<&'static str> {
    BUILTIN_VOCABULARY
        .iter()
        .find(|(neutral, _)| *neutral == name)
        .map(|(_, class)| *class)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::builtin_class;
    use std::collections::BTreeSet;

    /// 語彙の全項目が**実在の組込み述語**を指すこと。中立名が FQN と同じ述語を
    /// 指す保証は「同じ分類器に同じ FQN を渡す」ことで構成的に得ている。
    #[test]
    fn every_neutral_name_denotes_an_existing_builtin() {
        for (neutral, class) in BUILTIN_VOCABULARY {
            assert!(
                builtin_class(class).is_some(),
                "{neutral} -> {class} が組込みに分類されない"
            );
        }
    }

    /// 名前は SCREAMING_SNAKE_CASE で一意、表は名前順。
    #[test]
    fn names_are_unique_sorted_and_screaming_snake() {
        let names: Vec<&str> = BUILTIN_VOCABULARY.iter().map(|(n, _)| *n).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted, "語彙表は名前順に保つ");
        assert_eq!(names.len(), names.iter().collect::<BTreeSet<_>>().len());
        for name in names {
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()),
                "{name} は SCREAMING_SNAKE_CASE ではない"
            );
        }
    }

    /// 既に中立な綴りを持つ組込み（v1 から変わらない）は語彙表に入れない。
    #[test]
    fn already_neutral_spellings_are_not_duplicated() {
        for existing in [
            "ANY",
            "EOF",
            "EMPTY",
            "UNTIL",
            "CI",
            "NEGATION",
            "CHAR_RANGE",
            "LOOKAHEAD",
            "NEGATIVE_LOOKAHEAD",
            "REGEX",
        ] {
            assert!(builtin_fqn(existing).is_none(), "{existing}");
        }
    }
}
