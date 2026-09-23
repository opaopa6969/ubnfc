//! リテラルの語境界（D-040、`06-ubnf-v2.md` §3）。
//!
//! `'let'` が `letter` の先頭 3 文字に一致してしまう弱点（D-002）を消す。
//! **末尾が識別子構成文字のリテラルは、直後も識別子構成文字なら一致しない。**
//!
//! - 適用は「消費するリテラル形の終端」だけ: rule 本体のリテラルと `CI('word')`。
//!   `LOOKAHEAD` / `NEGATIVE_LOOKAHEAD` は消費しないゼロ幅述語、`UNTIL` の終端は
//!   区切り記号であって語ではないので対象外（`docs/reports/2026-09-23-ubnf-v2-word-boundary.md`）。
//! - 識別子構成文字は ASCII `[A-Za-z0-9_]`。組込み `IdentifierParser` の継続文字集合と
//!   厳密に同じで、生成物に Unicode 表を持ち込まない（依存ゼロ契約）。
//! - 開始側の境界は課さない（`10px` のような接尾キーワードを壊すため）。
//!
//! 設定 `@literalBoundary: word|none`。既定は `@ubnf: 1` で `none`、`@ubnf: 2` で `word`。

use crate::syntax::ast::{GlobalSetting, GrammarDecl};
use crate::validate::ValidationIssue;

/// 互換影響の実測用に既定を差し替える環境変数（`word` / `none`）。
/// 文法が `@literalBoundary` を明示していれば、そちらが常に勝つ。
pub const OVERRIDE_ENV: &str = "UBNFC_LITERAL_BOUNDARY_DEFAULT";

pub const SETTING_KEY: &str = "literalBoundary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    None,
    Word,
}

impl Mode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "word" => Some(Self::Word),
            _ => None,
        }
    }
    /// 版ごとの既定。v1 は凍結なので `none`。
    pub fn default_for(version: u64) -> Self {
        if version >= 2 {
            Self::Word
        } else {
            Self::None
        }
    }
}

/// ASCII `[A-Za-z0-9_]`。組込み `IdentifierParser` の継続文字集合。
#[must_use]
pub fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// そのリテラル値に語境界を課すか。末尾が識別子構成文字のときだけ。
#[must_use]
pub fn applies_to(value: &str, mode: Mode) -> bool {
    mode == Mode::Word && value.chars().next_back().is_some_and(is_identifier_char)
}

fn setting_value(setting: &GlobalSetting) -> Option<&str> {
    match setting {
        GlobalSetting::String { value, .. } => Some(value.as_str()),
        GlobalSetting::Block { .. } => None,
    }
}

/// 文法の実効モード。値の検査は `validate` の責務で、ここは最後の宣言を読むだけ。
#[must_use]
pub fn mode_of(settings: &[GlobalSetting], version: u64) -> Mode {
    let declared = settings.iter().rev().find(|s| {
        let (GlobalSetting::String { key, .. } | GlobalSetting::Block { key, .. }) = s;
        key == SETTING_KEY
    });
    match declared {
        Some(setting) => setting_value(setting)
            .and_then(Mode::parse)
            .unwrap_or_else(|| Mode::default_for(version)),
        None => std::env::var(OVERRIDE_ENV)
            .ok()
            .and_then(|v| Mode::parse(&v))
            .unwrap_or_else(|| Mode::default_for(version)),
    }
}

/// `@literalBoundary` の値と重複を検査する（desugar 段階、resolve より前）。
/// 実効モードそのものは lower が `mode_of` で読み直す。
pub fn validate(decl: &GrammarDecl, version: u64, issues: &mut Vec<ValidationIssue>) -> Mode {
    let mut seen = false;
    for setting in &decl.settings {
        let (GlobalSetting::String { key, .. } | GlobalSetting::Block { key, .. }) = setting;
        if key != SETTING_KEY {
            continue;
        }
        if seen {
            issues.push(ValidationIssue {
                code: "E-LITERAL-BOUNDARY-DUPLICATE".into(),
                message: "@literalBoundary は grammar ごとに一度だけ宣言できます".into(),
                hint: "余分な @literalBoundary 設定を削除してください。".into(),
                rule: None,
                grammar: decl.name.clone(),
            });
            continue;
        }
        seen = true;
        if setting_value(setting).and_then(Mode::parse).is_none() {
            issues.push(ValidationIssue {
                code: "E-LITERAL-BOUNDARY-VALUE".into(),
                message: format!(
                    "@literalBoundary は word か none です: {}",
                    setting_value(setting).unwrap_or("{...}")
                ),
                hint: "'@literalBoundary: word' か '@literalBoundary: none' を宣言してください。"
                    .into(),
                rule: None,
                grammar: decl.name.clone(),
            });
        }
    }
    mode_of(&decl.settings, version)
}
