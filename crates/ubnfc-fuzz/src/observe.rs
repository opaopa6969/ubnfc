//! 2 バックエンドの観測比較と、バックエンドによらず成り立つべき内部不変条件。
//!
//! 比較の弱め方は corpus と同じ契約に限る:
//! - D-011 / D-014 (`docs/design/04-cursor-and-diagnostics-contract.md` §2):
//!   `syntax` 診断の主 offset と hints の表記は言語別 oracle。`farthest*` と kind は共有契約。
//! - D-019 / D-020: `nativeFailure` / `rulePath` / `deepestRule` は契約対象外。
//! - 数値 token の直接 capture は**射影しない**（D-074 が D-022 の言語別契約を置換）。
//!   両 backend とも原文の文字列なので厳密比較する。以前は Java の int を字句へ射影して
//!   いたため、`NUMBER @x` の backend 差（#75）をこの比較が消していた。

use serde_json::{json, Map, Value};
use ubnfc_harness::canonical::{diff_values, AstValue, Difference};

/// 両バックエンドで同じでなければならない観測。
const COMPARED: &[&str] = &[
    "accepted",
    "allConsumed",
    "consumed",
    "matched",
    "ast",
    "captures",
    "prefixCaptures",
    "declarations",
    "references",
    "scope",
    "scopeDepth",
    "parsedStatus",
    "diagnostics",
    "mappingError",
    "recoveries",
    "catalogs",
    "value",
];

fn get(value: &Value, key: &str) -> Value {
    value.get(key).cloned().unwrap_or(Value::Null)
}

/// 評価器の 1.0 と 1 は同じ数（harness の comparison と同じ扱い）。
fn numeric(value: Value) -> Value {
    match value {
        Value::Number(ref number) if number.is_f64() => {
            let f = number.as_f64().unwrap();
            if f.fract() == 0.0 && f.abs() < 9e15 {
                json!(f as i64)
            } else {
                value
            }
        }
        Value::Array(items) => Value::Array(items.into_iter().map(numeric).collect()),
        Value::Object(items) => {
            Value::Object(items.into_iter().map(|(k, v)| (k, numeric(v))).collect())
        }
        _ => value,
    }
}

/// D-027: 文法に現れない合成 parser の表示名は言語別（旧 `ParseContext.java:1113-1143` の
/// 「2 段 BFS が空のときだけクラス単純名」フォールバックに由来し、helper の作り方が言語で違う）。
/// corpus は Java 採取 oracle を両 backend で厳密比較したままなので、緩めるのは oracle の無い
/// ランダム文法だけ。`...Parser` で終わる token 表示名まで落ちる点は report に記録する。
fn grammar_visible(hint: &str) -> bool {
    !hint.ends_with("Parser")
        && !matches!(
            hint,
            "Repeat" | "OneOrMore" | "Bounded" | "Optional" | "__CaptureSite"
        )
}

fn project_hints(value: &Value) -> Value {
    match value.as_array() {
        None => value.clone(),
        Some(items) => Value::Array(
            items
                .iter()
                .filter(|hint| hint.as_str().is_none_or(grammar_visible))
                .cloned()
                .collect(),
        ),
    }
}

fn project_diagnostic(diagnostic: &Value) -> Value {
    let kind = get(diagnostic, "kind");
    let mut out = Map::new();
    out.insert("kind".into(), kind.clone());
    // D-011: syntax の主 offset は言語別 oracle。他の kind は共有契約。
    if kind.as_str() != Some("syntax") {
        out.insert("offset".into(), get(diagnostic, "offset"));
        out.insert("expected".into(), get(diagnostic, "expected"));
    } else {
        // D-014: hints の表記は言語別。非空契約だけを比較する。
        out.insert(
            "expectedNonEmpty".into(),
            json!(diagnostic
                .get("expected")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty())),
        );
    }
    out.insert("farthestOffset".into(), get(diagnostic, "farthestOffset"));
    out.insert(
        "farthestExpected".into(),
        project_hints(&get(diagnostic, "farthestExpected")),
    );
    for key in ["message", "length", "severity"] {
        out.insert(key.into(), get(diagnostic, key));
    }
    Value::Object(out)
}

fn project_diagnostics(value: &Value) -> Value {
    match value.as_array() {
        None => value.clone(),
        Some(items) => Value::Array(items.iter().map(project_diagnostic).collect()),
    }
}

fn project_capture(capture: &Value) -> Value {
    let mut out = Map::new();
    for key in ["name", "binding", "span", "text"] {
        out.insert(key.into(), get(capture, key));
    }
    Value::Object(out)
}

fn project_captures(value: &Value) -> Value {
    match value.as_array() {
        None => value.clone(),
        Some(items) => Value::Array(items.iter().map(project_capture).collect()),
    }
}

/// 比較に使う形へ正規化する。両 backend に同じ規則を当てる。
fn projection(response: &Value) -> Map<String, Value> {
    let mut out = Map::new();
    for &key in COMPARED {
        let present = response.get(key).is_some();
        let value = match key {
            "captures" | "prefixCaptures" => project_captures(&get(response, key)),
            "diagnostics" => project_diagnostics(&get(response, key)),
            "scope" => {
                let mut scope = get(response, key);
                if let Some(object) = scope.as_object_mut() {
                    if let Some(diagnostics) = object.get("diagnostics") {
                        let projected = project_diagnostics(diagnostics);
                        object.insert("diagnostics".into(), projected);
                    }
                }
                scope
            }
            // 例外クラス名は言語で異なる。有無だけを契約とする。
            "mappingError" => json!({ "present": !get(response, key).is_null() }),
            "value" => numeric(get(response, key)),
            _ => get(response, key),
        };
        // 明示 null と欠落を区別する（protocol の契約）。
        out.insert(
            key.into(),
            if present {
                value
            } else {
                json!({ "<unobserved>": key })
            },
        );
    }
    out
}

/// 両バックエンドの応答差。空なら観測上は一致。
pub fn differences(java: &Value, rust: &Value) -> Vec<Difference> {
    // 片側だけが error（停止・異常終了・未対応）を返したらそれ自体が差。
    // 両側とも error なら、その入力の観測は無いので比較しない。
    let (java_error, rust_error) = (java.get("error"), rust.get("error"));
    if java_error.is_some() || rust_error.is_some() {
        if java_error.is_some() && rust_error.is_some() {
            return Vec::new();
        }
        return vec![Difference {
            path: "/error".into(),
            expected: java_error.cloned(),
            actual: rust_error.cloned(),
        }];
    }
    let expected = projection(java);
    let actual = projection(rust);
    diff_values(&Value::Object(expected), &Value::Object(actual))
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Violation {
    pub id: &'static str,
    pub detail: String,
}

fn violation(out: &mut Vec<Violation>, id: &'static str, detail: impl Into<String>) {
    let detail = detail.into();
    if !out.iter().any(|v| v.id == id && v.detail == detail) {
        out.push(Violation { id, detail });
    }
}

fn span(value: &Value) -> Option<(i64, i64)> {
    let items = value.as_array()?;
    if items.len() != 2 {
        return None;
    }
    Some((items[0].as_i64()?, items[1].as_i64()?))
}

fn check_span(out: &mut Vec<Violation>, id: &'static str, label: &str, value: &Value, len: i64) {
    match span(value) {
        None => violation(
            out,
            id,
            format!("{label}: span が 2 要素の整数ではない {value}"),
        ),
        Some((start, end)) => {
            if start > end {
                violation(out, id, format!("{label}: span 逆転 [{start},{end}]"));
            }
            if start < 0 || end > len {
                violation(
                    out,
                    id,
                    format!("{label}: span が入力長 {len} の外 [{start},{end}]"),
                );
            }
        }
    }
}

fn check_ast(
    out: &mut Vec<Violation>,
    node: &Value,
    parent: Option<(i64, i64)>,
    len: i64,
    path: &str,
) {
    let Some(object) = node.as_object() else {
        return;
    };
    let Some(current) = span(&get(node, "span")) else {
        violation(out, "ast-span", format!("{path}: span がない"));
        return;
    };
    check_span(out, "ast-span", path, &get(node, "span"), len);
    if let Some((start, end)) = parent {
        if current.0 < start || current.1 > end {
            violation(
                out,
                "ast-span-nesting",
                format!(
                    "{path}: 子 span [{},{}] が親 [{start},{end}] の外",
                    current.0, current.1
                ),
            );
        }
    }
    let Some(fields) = object.get("fields").and_then(Value::as_object) else {
        return;
    };
    for (name, value) in fields {
        match value {
            Value::Object(_) => {
                check_ast(out, value, Some(current), len, &format!("{path}/{name}"))
            }
            Value::Array(items) => {
                let mut previous: Option<(i64, i64)> = None;
                for (index, item) in items.iter().enumerate() {
                    let child = format!("{path}/{name}/{index}");
                    if item.is_object() {
                        check_ast(out, item, Some(current), len, &child);
                        if let (Some(previous), Some(now)) = (previous, span(&get(item, "span"))) {
                            if now.0 < previous.0 {
                                violation(
                                    out,
                                    "ast-list-order",
                                    format!("{child}: list の span が文書順に並んでいない"),
                                );
                            }
                        }
                        previous = span(&get(item, "span"));
                    }
                }
            }
            _ => {}
        }
    }
}

/// バックエンドによらず成り立つべき条件。破れは片側の実装バグか契約の不足。
pub fn invariants(input: &str, response: &Value) -> Vec<Violation> {
    let mut out = Vec::new();
    if response.get("error").is_some() {
        return out;
    }
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len() as i64;
    let consumed = response.get("consumed").and_then(Value::as_i64);
    let matched = response.get("matched").and_then(Value::as_i64);
    if let (Some(consumed), Some(matched)) = (consumed, matched) {
        if consumed > matched {
            violation(
                &mut out,
                "cursor-order",
                format!("consumed {consumed} > matched {matched}"),
            );
        }
        if consumed < 0 || consumed > len || matched < 0 || matched > len {
            violation(
                &mut out,
                "cursor-range",
                format!("consumed {consumed} / matched {matched} が入力長 {len} の外"),
            );
        }
    }
    let accepted = response.get("accepted").and_then(Value::as_bool);
    let all_consumed = response.get("allConsumed").and_then(Value::as_bool);
    // D-024: `allConsumed` は全入力解析の成否であって、prefix 解析の `consumed` からは導けない。
    // 全入力解析が成功したなら、その解析のカーソルは入力末尾にある。
    let full_consumed = response.get("fullConsumed").and_then(Value::as_i64);
    let full_matched = response.get("fullMatched").and_then(Value::as_i64);
    if let (Some(true), Some(full_consumed)) = (all_consumed, full_consumed) {
        if full_consumed != len {
            violation(
                &mut out,
                "allconsumed-consistency",
                format!("allConsumed=true だが fullConsumed={full_consumed} / 入力長={len}"),
            );
        }
    }
    if let (Some(full_consumed), Some(full_matched)) = (full_consumed, full_matched) {
        if full_consumed > full_matched {
            violation(
                &mut out,
                "cursor-order",
                format!("fullConsumed {full_consumed} > fullMatched {full_matched}"),
            );
        }
        if full_consumed < 0 || full_consumed > len || full_matched < 0 || full_matched > len {
            violation(
                &mut out,
                "cursor-range",
                format!(
                    "fullConsumed {full_consumed} / fullMatched {full_matched} が入力長 {len} の外"
                ),
            );
        }
    }
    if all_consumed == Some(true) && accepted == Some(false) {
        violation(
            &mut out,
            "allconsumed-implies-accepted",
            "allConsumed=true なのに accepted=false".to_string(),
        );
    }
    match response.get("ast") {
        None | Some(Value::Null) => {}
        Some(ast) => {
            match AstValue::try_from(ast) {
                Err(error) => violation(
                    &mut out,
                    "ast-canonical",
                    format!("canonical AST ではない: {error}"),
                ),
                Ok(parsed) => {
                    if &Value::from(&parsed) != ast {
                        violation(
                            &mut out,
                            "ast-canonical",
                            "canonical AST の JSON 往復が一致しない".to_string(),
                        );
                    }
                }
            }
            check_ast(&mut out, ast, None, len, "/ast");
        }
    }
    for key in ["captures", "prefixCaptures"] {
        for (index, capture) in response
            .get(key)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let label = format!("/{key}/{index}");
            check_span(&mut out, "capture-span", &label, &get(capture, "span"), len);
            if let (Some((start, end)), Some(text)) = (
                span(&get(capture, "span")),
                capture.get("text").and_then(Value::as_str),
            ) {
                if start >= 0 && end <= len && start <= end {
                    let slice: String = chars[start as usize..end as usize].iter().collect();
                    if slice != text {
                        violation(
                            &mut out,
                            "capture-text",
                            format!("{label}: text {text:?} が入力の span 部分 {slice:?} と違う"),
                        );
                    }
                }
            }
        }
    }
    for key in ["declarations", "references"] {
        for (index, item) in response
            .get(key)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let offset = item
                .get("sourceOffset")
                .or_else(|| item.get("offset"))
                .and_then(Value::as_i64);
            if let Some(offset) = offset {
                if offset < 0 || offset > len {
                    violation(
                        &mut out,
                        "scope-offset-range",
                        format!("/{key}/{index}: offset {offset} が入力長 {len} の外"),
                    );
                }
            }
        }
    }
    for (index, diagnostic) in response
        .get("diagnostics")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        for key in ["offset", "farthestOffset"] {
            if let Some(offset) = diagnostic.get(key).and_then(Value::as_i64) {
                // 未到達は -1 で表す（Java Driver の farthestCp）。
                if offset < -1 || offset > len {
                    violation(
                        &mut out,
                        "diagnostic-offset-range",
                        format!("/diagnostics/{index}/{key}: {offset} が入力長 {len} の外"),
                    );
                }
            }
        }
    }
    for (index, recovery) in response
        .get("recoveries")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        for key in ["span", "skippedSpan", "syncSpan"] {
            if let Some(value) = recovery.get(key).filter(|v| !v.is_null()) {
                check_span(
                    &mut out,
                    "recovery-span",
                    &format!("/recoveries/{index}/{key}"),
                    value,
                    len,
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(extra: Value) -> Value {
        let mut base = json!({"accepted": true, "allConsumed": true, "consumed": 3, "matched": 3,
            "fullConsumed": 3, "fullMatched": 3,
            "ast": null, "captures": [], "prefixCaptures": [], "declarations": [], "references": [],
            "scope": {}, "scopeDepth": 0, "parsedStatus": "succeeded", "diagnostics": [],
            "mappingError": null, "recoveries": []});
        for (key, value) in extra.as_object().unwrap() {
            base[key] = value.clone();
        }
        base
    }

    #[test]
    fn identical_responses_have_no_difference_and_ignored_metadata_is_not_compared() {
        let java = response(
            json!({"nativeFailure": {"offset": 1}, "mappingErrorKind": "X",
            "lexical": [{"occurrenceId": 1}], "id": 0}),
        );
        let rust = response(json!({"id": 1, "nodes": []}));
        assert_eq!(differences(&java, &rust), vec![]);
    }

    #[test]
    fn language_specific_diagnostic_contracts_are_honoured_but_shared_ones_are_not() {
        let at = |offset, expected: Value, farthest| {
            response(json!({"diagnostics": [{"kind": "syntax", "offset": offset,
                "expected": expected, "farthestOffset": farthest, "farthestExpected": ["a"],
                "origin": "Mapper.diagnose", "rulePath": ["R"], "deepestRule": "R"}]}))
        };
        // D-011: syntax の主 offset と hints の表記は言語別。
        assert_eq!(
            differences(&at(7, json!(["DigitParser"]), 7), &at(5, json!(["';'"]), 7)),
            vec![]
        );
        // D-014: farthest は共有契約。
        assert_eq!(
            differences(&at(7, json!(["a"]), 7), &at(7, json!(["a"]), 5)).len(),
            1
        );
        // 非空契約は弱めない。
        assert_eq!(
            differences(&at(7, json!(["a"]), 7), &at(7, json!([]), 7)).len(),
            1
        );
        // trailing_input は offset も expected も厳密。
        let trailing = |offset| {
            response(
                json!({"diagnostics": [{"kind": "trailing_input", "offset": offset,
                "expected": ["end of input"], "farthestOffset": 4, "farthestExpected": []}]}),
            )
        };
        assert_eq!(differences(&trailing(2), &trailing(3)).len(), 1);
    }

    /// #75 / D-074: 数値 capture の int と字句は**差**として出る（射影で消さない）。
    /// 旧 Java の NumberFormatException も、もう免除しない。
    #[test]
    fn d074_numeric_captures_are_compared_without_projection() {
        let java = response(json!({
            "ast": {"type": "V", "span": [0, 3], "fields": {"value": 7}},
            "captures": [{"name": "value", "span": [1, 2], "text": "7"}]}));
        let rust = response(json!({
            "ast": {"type": "V", "span": [0, 3], "fields": {"value": "7"}},
            "captures": [{"name": "value", "span": [1, 2], "text": "7"}]}));
        let differences_found = differences(&java, &rust);
        assert_eq!(differences_found.len(), 1, "{differences_found:?}");
        assert_eq!(differences_found[0].path, "/ast/fields/value");
        assert_eq!(differences(&rust, &rust), vec![]);
        let java_error = response(
            json!({"mappingError": "java.lang.NumberFormatException: For input string: \"1.5\""}),
        );
        assert_eq!(differences(&java_error, &response(json!({}))).len(), 1);
        // mapping 失敗は有無を比較する。
        let other = response(json!({"mappingError": "java.lang.IllegalArgumentException: x"}));
        assert_eq!(differences(&other, &response(json!({}))).len(), 1);
    }

    #[test]
    fn a_one_sided_driver_error_is_itself_a_difference_but_a_two_sided_one_is_not() {
        let error = json!({"id": 0, "error": "timeout: 90s を超えました"});
        assert_eq!(differences(&error, &error), vec![]);
        let differences = differences(&error, &response(json!({})));
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].path, "/error");
    }

    #[test]
    fn missing_keys_are_reported_and_not_confused_with_explicit_null() {
        let mut rust = response(json!({}));
        rust.as_object_mut().unwrap().remove("captures");
        let differences = differences(&response(json!({})), &rust);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].path, "/captures");
    }

    #[test]
    fn invariants_catch_cursors_spans_capture_text_and_canonical_shape() {
        assert_eq!(invariants("abc", &response(json!({}))), vec![]);
        let ids = |input: &str, extra: Value| {
            invariants(input, &response(extra))
                .into_iter()
                .map(|v| v.id)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            ids("abc", json!({"consumed": 3, "matched": 2})),
            ["cursor-order"]
        );
        assert_eq!(
            ids("ab", json!({"consumed": 3, "matched": 3})),
            ["cursor-range", "allconsumed-consistency", "cursor-range"]
        );
        // D-024: allConsumed は全入力解析の成否。prefix の consumed とは別の解析なので、
        // 両者が食い違っても違反ではない。破れるのは全入力解析のカーソルとの関係だけ。
        assert_eq!(
            ids("abc", json!({"consumed": 1, "matched": 1})),
            Vec::<&str>::new()
        );
        assert_eq!(
            ids("abc", json!({"fullConsumed": 2, "fullMatched": 2})),
            ["allconsumed-consistency"]
        );
        assert_eq!(
            ids(
                "abc",
                json!({"captures": [{"name": "v", "span": [0, 1], "text": "b"}]})
            ),
            ["capture-text"]
        );
        assert_eq!(
            ids(
                "abc",
                json!({"captures": [{"name": "v", "span": [0, 9], "text": "abc"}]})
            ),
            ["capture-span"]
        );
        assert_eq!(
            ids(
                "abc",
                json!({"ast": {"type": "R", "span": [0, 3], "fields": {"x": true}}})
            ),
            ["ast-canonical"]
        );
        assert_eq!(
            ids(
                "abc",
                json!({"ast": {"type": "R", "span": [0, 1], "fields":
                {"x": {"type": "C", "span": [1, 3], "fields": {}}}}})
            ),
            ["ast-span-nesting"]
        );
        assert_eq!(
            ids(
                "abc",
                json!({"ast": {"type": "R", "span": [0, 3], "fields": {"x": [
                {"type": "C", "span": [2, 3], "fields": {}},
                {"type": "C", "span": [0, 1], "fields": {}}]}}})
            ),
            ["ast-list-order"]
        );
        assert_eq!(
            ids("abc", json!({"recoveries": [{"span": [0, 8]}]})),
            ["recovery-span"]
        );
    }
}
