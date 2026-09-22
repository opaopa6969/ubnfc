use super::{Delimiter, GrammarIr, IrError, TokenSpec};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::BTreeMap;

/// JSON の構造を検査して読み込む。参照検証は `validate_references` を使う。
/// 非 canonical な空白やキー順も受け付け、配列順・null・Unicode を保持する。
pub fn from_json(input: &str) -> Result<GrammarIr, IrError> {
    serde_json::from_str(input).map_err(IrError::from)
}

/// ubnfc-json.v1: 再帰的キー順、2 空白、LF、末尾 LF 1 個。
///
/// 意味順の配列・Unicode は正規化しない。集合の feature/ranges/codePoints は昇順。
/// source path/ID の整合は参照検証の責務。
/// JsonValue 内の浮動小数は serde_json の有限数表現を使用する（実行数値は整数型）。
pub fn to_canonical_json(ir: &GrammarIr) -> String {
    let mut ordered = ir.clone();
    ordered.required_features.sort_by_cached_key(|feature| {
        serde_json::to_string(feature).expect("feature serialization")
    });
    for analysis in &mut ordered.analysis.expressions {
        analysis
            .may_start
            .ranges
            .sort_by_key(|range| (range.start, range.end));
    }
    for token in &mut ordered.semantics.tokens {
        if let TokenSpec::Builtin { code_points, .. } = &mut token.spec {
            code_points.sort_unstable();
        }
    }
    for policy in &mut ordered.semantics.trivia.policies {
        for delimiter in &mut policy.effective_delimiter_set {
            if let Delimiter::Characters { code_points } = delimiter {
                code_points.sort_unstable();
            }
        }
    }
    let value =
        serde_json::to_value(ordered).expect("GrammarIr contains only JSON-serializable values");
    let mut output = String::new();
    write_value(&value, 0, &mut output);
    output.push('\n');
    output
}

fn indent(depth: usize, output: &mut String) {
    for _ in 0..depth {
        output.push_str("  ");
    }
}

fn write_value(value: &Value, depth: usize, output: &mut String) {
    match value {
        Value::Object(fields) => {
            output.push('{');
            // UTF-8 の辞書順は Unicode scalar 順と一致する。
            // serde_json の preserve_order feature の有無に依存しない。
            let sorted: BTreeMap<_, _> = fields.iter().collect();
            for (index, (key, value)) in sorted.into_iter().enumerate() {
                output.push('\n');
                indent(depth + 1, output);
                output.push_str(&serde_json::to_string(key).expect("string serialization"));
                output.push_str(": ");
                write_value(value, depth + 1, output);
                if index + 1 < fields.len() {
                    output.push(',');
                }
            }
            if !fields.is_empty() {
                output.push('\n');
                indent(depth, output);
            }
            output.push('}');
        }
        Value::Array(items) => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                output.push('\n');
                indent(depth + 1, output);
                write_value(item, depth + 1, output);
                if index + 1 < items.len() {
                    output.push(',');
                }
            }
            if !items.is_empty() {
                output.push('\n');
                indent(depth, output);
            }
            output.push(']');
        }
        _ => output.push_str(&serde_json::to_string(value).expect("JSON value serialization")),
    }
}

// Option の通常の derive は欠落キーも None とするため、必須 nullable に使用する。
pub(super) fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

// comment のキーは省略できるが、存在する場合 null を許さない。
pub(super) fn present_value<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}
