//! Extern token の先頭文字宣言（`scanners/first-chars.json`）。
//!
//! scanner 実装の registry が「成功し得る先頭 code point の集合」を元クラス名ごとに宣言し、
//! front は `--extern-first-chars FILE` で読み込んで `TokenSpec::Extern.firstChars` に写す。
//! 宣言は必要条件（集合外の先頭文字では scanner が消費なしで失敗する）で、健全性は
//! scanner 側の責任。宣言の無いクラスは unknown のまま（backend は候補除外に使わない）。
use super::{CharRange, GrammarIr, TokenSpec};
use serde_json::Value;
use std::collections::BTreeMap;

/// 元クラス名 → 先頭文字集合（正規化済み: 昇順・互いに素）。`null` 宣言は unknown。
pub type ExternFirstChars = BTreeMap<String, Option<Vec<CharRange>>>;

fn code_point(v: &Value, path: &str) -> Result<u64, String> {
    match v {
        Value::Number(n) => n
            .as_u64()
            .filter(|&c| c <= 0x10ffff)
            .ok_or_else(|| format!("{path}: code point は 0..=0x10FFFF の整数")),
        Value::String(s) => {
            let mut chars = s.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => Ok(c as u64),
                _ => Err(format!(
                    "{path}: 1 code point の文字列か整数を指定する: {s:?}"
                )),
            }
        }
        _ => Err(format!("{path}: 1 code point の文字列か整数を指定する")),
    }
}

/// 昇順・互いに素な閉区間に正規化する（隣接区間は連結）。
pub fn normalize_ranges(mut ranges: Vec<CharRange>) -> Vec<CharRange> {
    ranges.sort_by_key(|r| (r.start, r.end));
    let mut out: Vec<CharRange> = Vec::with_capacity(ranges.len());
    for r in ranges {
        if let Some(last) = out.last_mut() {
            if r.start <= last.end.saturating_add(1) {
                last.end = last.end.max(r.end);
                continue;
            }
        }
        out.push(r);
    }
    out
}

/// 宣言ファイルを読む。形式:
/// `{"scanners": {"<originalClassName>": ["'", "\"", {"start": "0", "end": "9"}] | null, ...}}`
/// 要素は 1 code point の文字列（または整数）か、`start`/`end` を持つ閉区間。
pub fn parse_extern_first_chars(text: &str) -> Result<ExternFirstChars, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| format!("first-chars: {e}"))?;
    let Some(scanners) = root.get("scanners").and_then(Value::as_object) else {
        return Err("first-chars: 最上位に \"scanners\" オブジェクトが必要".into());
    };
    let mut out = ExternFirstChars::new();
    for (class, decl) in scanners {
        let path = format!("scanners/{class}");
        let ranges = match decl {
            Value::Null => None,
            Value::Array(items) => {
                let mut ranges = Vec::with_capacity(items.len());
                for (i, item) in items.iter().enumerate() {
                    let p = format!("{path}/{i}");
                    let range = match item {
                        Value::Object(o) => {
                            let start =
                                o.get("start").ok_or_else(|| format!("{p}: start が必要"))?;
                            let end = o.get("end").ok_or_else(|| format!("{p}: end が必要"))?;
                            if o.len() != 2 {
                                return Err(format!("{p}: start と end だけを指定する"));
                            }
                            let (start, end) = (
                                code_point(start, &format!("{p}/start"))?,
                                code_point(end, &format!("{p}/end"))?,
                            );
                            if start > end {
                                return Err(format!("{p}: start <= end"));
                            }
                            CharRange { start, end }
                        }
                        other => {
                            let c = code_point(other, &p)?;
                            CharRange { start: c, end: c }
                        }
                    };
                    ranges.push(range);
                }
                if ranges.is_empty() {
                    return Err(format!("{path}: 空集合は宣言できない（unknown は null）"));
                }
                Some(normalize_ranges(ranges))
            }
            _ => return Err(format!("{path}: 配列か null を指定する")),
        };
        out.insert(class.clone(), ranges);
    }
    Ok(out)
}

/// 宣言を IR の Extern token に写す。宣言の無いクラスは変更しない。
/// 返り値は (埋めた token 数, 宣言が無かった元クラス名)。
pub fn apply_extern_first_chars(
    ir: &mut GrammarIr,
    declarations: &ExternFirstChars,
) -> (usize, Vec<String>) {
    let mut filled = 0;
    let mut missing = Vec::new();
    for token in &mut ir.semantics.tokens {
        if let TokenSpec::Extern {
            first_chars,
            original_class_name,
            ..
        } = &mut token.spec
        {
            match declarations.get(original_class_name.as_str()) {
                Some(Some(ranges)) => {
                    *first_chars = Some(ranges.clone());
                    filled += 1;
                }
                Some(None) => *first_chars = None,
                None => missing.push(original_class_name.clone()),
            }
        }
    }
    (filled, missing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chars_ranges_and_null() {
        let d = parse_extern_first_chars(
            r#"{"scanners":{"a.B":["'", 34, {"start":"0","end":"9"}, "\"" ],"c.D":null}}"#,
        )
        .unwrap();
        assert_eq!(
            d["a.B"],
            Some(vec![
                CharRange { start: 34, end: 34 },
                CharRange { start: 39, end: 39 },
                CharRange { start: 48, end: 57 }
            ])
        );
        assert_eq!(d["c.D"], None);
        assert!(parse_extern_first_chars(r#"{"scanners":{"a":[]}}"#).is_err());
        assert!(parse_extern_first_chars(r#"{"scanners":{"a":["ab"]}}"#).is_err());
        assert!(
            parse_extern_first_chars(r#"{"scanners":{"a":[{"start":"b","end":"a"}]}}"#).is_err()
        );
        assert!(parse_extern_first_chars(r#"{"x":{}}"#).is_err());
    }

    #[test]
    fn normalizes_adjacent_and_overlapping() {
        let r = normalize_ranges(vec![
            CharRange { start: 5, end: 7 },
            CharRange { start: 1, end: 2 },
            CharRange { start: 3, end: 4 },
            CharRange { start: 6, end: 9 },
        ]);
        assert_eq!(r, vec![CharRange { start: 1, end: 9 }]);
        let r = normalize_ranges(vec![
            CharRange { start: 10, end: 12 },
            CharRange { start: 1, end: 2 },
        ]);
        assert_eq!(
            r,
            vec![
                CharRange { start: 1, end: 2 },
                CharRange { start: 10, end: 12 }
            ]
        );
    }
}
