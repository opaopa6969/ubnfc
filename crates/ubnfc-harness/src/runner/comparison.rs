use super::{Actual, Backend, RunCase};
use crate::{
    canonical::Difference,
    divergence::{self, Observation, Verdict},
    Expectation,
};
use serde_json::{json, Map, Value};

struct Projection {
    expected: Map<String, Value>,
    actual: Map<String, Value>,
    missing: Vec<String>,
}
impl Projection {
    fn new() -> Self {
        Self {
            expected: Map::new(),
            actual: Map::new(),
            missing: vec![],
        }
    }
    fn field(&mut self, name: &str, expected: Value, actual: Option<Value>) {
        if let Some(actual) = actual {
            self.expected.insert(name.into(), expected);
            self.actual.insert(name.into(), actual);
        } else {
            self.missing.push(format!("/{name}"));
        }
    }
    fn wire(&mut self, name: &str, expected: Value, response: &Value) {
        self.field(name, expected, response.get(name).cloned());
    }
    fn prefix(&mut self, prefix: &crate::corpus::Prefix, response: &Value) {
        self.wire("accepted", json!(prefix.0), response);
        self.wire("consumed", json!(prefix.1), response);
        self.wire("matched", json!(prefix.2), response);
    }
    fn full(&mut self, accepted: bool, response: &Value) {
        self.wire("allConsumed", json!(accepted), response);
    }
    fn value(&mut self, expected: &Value, response: &Value) {
        self.field(
            "value",
            numeric(expected.clone()),
            response.get("value").cloned().map(numeric),
        );
    }
}
// 評価器の 1.0 と JSON fixture の 1 は同じ数。巨大整数や AST の型は丸めない。
fn numeric(value: Value) -> Value {
    match value {
        Value::Number(ref number) if number.is_f64() => {
            let f = number.as_f64().unwrap();
            if f.fract() == 0.0 && f.abs() <= 9_007_199_254_740_992.0 {
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
fn projection(case: &RunCase, actual: &Actual) -> Projection {
    let response = &actual.response;
    let mut p = Projection::new();
    match &case.case.expect {
        Expectation::Ast(e) => {
            p.full(e.accepted.unwrap_or(true), response);
            if let Some(prefix) = &e.prefix {
                p.prefix(prefix, response);
            }
            p.wire("ast", json!(e.ast), response);
            if let Some(captures) = &e.captures {
                p.wire("captures", json!(captures), response);
            }
            if e.declarations.is_some() || case.family == "nested-captures" {
                p.wire(
                    "declarations",
                    json!(e.declarations.as_deref().unwrap_or_default()),
                    response,
                );
            }
            if e.value_spans.is_some() || case.family == "nested-captures" {
                p.wire(
                    "valueSpans",
                    json!(e.value_spans.as_deref().unwrap_or_default()),
                    response,
                );
            }
            if let Some(scope) = &e.scope {
                scope_fields(&mut p, scope, response);
            }
            if let Some(rollback) = e.parent_rollback {
                p.wire("parentRollback", json!(rollback), response);
            }
            for (key, value) in &e.extra {
                p.wire(key, value.clone(), response);
            }
        }
        Expectation::TokenValue(e) => {
            p.full(e.value.is_some(), response);
            if let Some(value) = &e.value {
                p.value(value, response);
            }
            if let Some(prefix) = &e.prefix {
                p.prefix(prefix, response);
            }
            for (key, value) in &e.extra {
                p.wire(key, value.clone(), response);
            }
        }
        Expectation::RuleValue(e) => {
            let accepted = e.accepted_in(&actual.variant);
            p.full(accepted, response);
            if accepted {
                p.value(e.value.as_ref().unwrap(), response);
            }
            if let Some(prefix) = &e.prefix {
                p.prefix(prefix, response);
            }
            if let Some(texts) = &e.texts {
                p.wire("texts", json!(texts), response);
            }
            if let Some(nodes) = &e.nodes {
                p.wire("nodes", json!(nodes), response);
            }
            if let Some(expected) = &e.expected {
                p.wire("expected", expected.clone(), response);
            }
            for (key, value) in &e.extra {
                p.wire(key, value.clone(), response);
            }
        }
        Expectation::Accept(e) => {
            p.full(e.accepted, response);
            p.prefix(&e.prefix, response);
            if let Some(span) = e.span {
                p.field("span", json!(span), rule_field(response, "span"));
            }
            if let Some(fields) = &e.fields {
                p.field("fields", json!(fields), rule_field(response, "fields"));
            }
            if let Some(value) = &e.value {
                p.value(value, response);
            }
            for (key, value) in &e.extra {
                p.wire(key, value.clone(), response);
            }
        }
        Expectation::Diagnostics(e) => {
            // 空の診断配列は観測済みの不一致。未観測と混同しない。
            let diagnostic = response
                .get("diagnostics")
                .map(|d| d.get(0).cloned().unwrap_or(Value::Null));
            p.field(
                "kind",
                json!(e.kind),
                diagnostic
                    .as_ref()
                    .map(|d| d.get("kind").cloned().unwrap_or(Value::Null)),
            );
            p.field(
                "offset",
                json!(match actual.backend {
                    Backend::Java => e.java_offset,
                    Backend::Rust => e.rust_offset,
                }),
                diagnostic.map(|d| d.get("offset").cloned().unwrap_or(Value::Null)),
            );
            for (key, value) in &e.extra {
                p.wire(key, value.clone(), response);
            }
        }
        Expectation::Numeric(e) => {
            p.full(true, response);
            match actual.backend {
                Backend::Rust => p.value(&json!(e.rust_lexeme), response),
                Backend::Java => {
                    if let Some(value) = e.java_int {
                        p.value(&json!(value), response);
                        p.wire("mappingError", Value::Null, response);
                    } else {
                        p.field(
                            "mappingError",
                            json!(true),
                            response
                                .get("mappingError")
                                .map(|value| json!(value.is_string())),
                        );
                    }
                }
            }
            for (key, value) in &e.extra {
                p.wire(key, value.clone(), response);
            }
        }
        Expectation::Oracle { .. } | Expectation::GenerationFailure { .. } => {}
        Expectation::ScopeOps(_) | Expectation::Catalog(_) => {
            let encoded = serde_json::to_value(&case.case.expect).unwrap();
            let fields = encoded.as_object().unwrap().values().next().unwrap();
            for (key, value) in fields.as_object().unwrap() {
                p.wire(key, value.clone(), response);
            }
        }
        Expectation::Raw(_) => p.missing.push("/expectation".into()),
    }
    for (key, value) in &case.extra_expectations {
        p.wire(key, value.clone(), response);
    }
    p
}
// AST 自体が明示 null なら span/fields も観測済みの欠落。キー未提供とは異なる。
fn rule_field(response: &Value, key: &str) -> Option<Value> {
    response.get(key).cloned().or_else(|| {
        response
            .get("ast")
            .map(|ast| ast.get(key).cloned().unwrap_or(Value::Null))
    })
}
fn scope_fields(
    p: &mut Projection,
    scope: &crate::corpus::operations::ScopeState,
    response: &Value,
) {
    let mut expected = serde_json::to_value(scope).unwrap();
    // ScopeState の optional None は期待値未指定。resolved 内の symbol:null は保持する。
    expected
        .as_object_mut()
        .unwrap()
        .retain(|_, value| !value.is_null());
    if response
        .get("scope")
        .is_some_and(|scope| !scope.is_object())
    {
        p.wire("scope", expected, response);
        return;
    }
    let mut e = Map::new();
    let mut a = Map::new();
    for (key, value) in expected.as_object().unwrap() {
        let observed = response
            .get("scope")
            .and_then(|scope| scope.get(key))
            .or_else(|| match key.as_str() {
                "declarations" | "references" | "diagnostics" => response.get(key),
                "depth" => response.get("scopeDepth"),
                _ => None,
            });
        if let Some(observed) = observed {
            e.insert(key.clone(), value.clone());
            a.insert(key.clone(), observed.clone());
        } else {
            p.missing.push(format!("/scope/{key}"));
        }
    }
    if !e.is_empty() {
        p.field("scope", Value::Object(e), Some(Value::Object(a)));
    }
}

/// Oracle は同じ Expectation へ投影して既存 D-017 判定を使う。
/// 欠落した観測は比較用コピーでのみ一致させ、受信値自体には書き足さない。
fn oracle(case: &RunCase, actual: &Actual) -> (Expectation, Expectation, Vec<String>) {
    let mut expected = serde_json::to_value(&case.case.expect).unwrap();
    let mut observed = expected.clone();
    let mut missing = Vec::new();
    for (key, value) in expected["Oracle"].as_object_mut().unwrap() {
        if matches!(
            key.as_str(),
            "accepted" | "allConsumed" | "consumed" | "matched"
        ) && value.is_null()
        {
            continue;
        }
        match actual.response.get(key) {
            None => missing.push(format!("/{key}")),
            Some(row) => {
                let mut row = row.clone();
                if matches!(key.as_str(), "captures" | "diagnostics") {
                    // binding/origin 等は protocol 拡張。要素数や必須項目の欠落は通常の差分。
                    if let (Some(e), Some(a)) = (value.as_array(), row.as_array_mut()) {
                        for (i, (e, a)) in e.iter().zip(a).enumerate() {
                            if let (Some(e), Some(a)) = (e.as_object(), a.as_object_mut()) {
                                for (field, expected) in e {
                                    let mandatory = if key == "captures" {
                                        ["name", "span", "text"].contains(&field.as_str())
                                    } else {
                                        ["kind", "offset", "expected"].contains(&field.as_str())
                                    };
                                    if !mandatory && !a.contains_key(field) {
                                        missing.push(format!("/{key}/{i}/{field}"));
                                        a.insert(field.clone(), expected.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                observed["Oracle"][key] = row;
            }
        }
    }
    // 型違い・必須フィールド欠落も Fail。deserialize 失敗を未観測にしない。
    match serde_json::from_value(observed.clone()) {
        Ok(observed) => (case.case.expect.clone(), observed, missing),
        Err(_) => (
            Expectation::Raw(expected["Oracle"].clone()),
            Expectation::Raw(observed["Oracle"].clone()),
            missing,
        ),
    }
}
fn raw_compare(p: &Projection) -> Verdict {
    clean(divergence::compare(
        &Expectation::Raw(Value::Object(p.expected.clone())),
        &Expectation::Raw(Value::Object(p.actual.clone())),
        &[],
    ))
}
fn clean(verdict: Verdict) -> Verdict {
    fn paths(mut diffs: Vec<Difference>) -> Vec<Difference> {
        for diff in &mut diffs {
            if let Some(path) = diff.path.strip_prefix("/Raw") {
                diff.path = path.to_owned();
            }
        }
        diffs
    }
    match verdict {
        Verdict::Pass => Verdict::Pass,
        Verdict::Fail(d) => Verdict::Fail(paths(d)),
        Verdict::KnownDivergence(d) => Verdict::KnownDivergence(paths(d)),
    }
}
pub fn compare(case: &RunCase, actual: &Actual, known: &[Observation]) -> Verdict {
    if let Some(error) = actual.response.get("error") {
        return Verdict::Fail(vec![Difference {
            path: "/error".into(),
            expected: None,
            actual: Some(error.clone()),
        }]);
    }
    if let Expectation::GenerationFailure { .. } = case.case.expect {
        // build 成功後の runtime 応答は generation failure ではない。
        let runtime: Expectation = serde_json::from_value(json!({"Oracle": {
            "accepted":null,"allConsumed":null,"consumed":null,"matched":null,
            "ast":null,"captures":[],"declarations":[],"diagnostics":[],"mappingError":null
        }}))
        .unwrap();
        return divergence::compare(&case.case.expect, &runtime, known);
    }
    let p = projection(case, actual);
    if matches!(case.case.expect, Expectation::Oracle { .. }) {
        let (expected, observed, _) = oracle(case, actual);
        let verdict = clean(divergence::compare(&expected, &observed, known));
        // raw にのみ残る references/scopeDepth 等も未登録差分なら必ず Fail。
        if let Verdict::Fail(mut extra) = raw_compare(&p) {
            if let Verdict::KnownDivergence(d) | Verdict::Fail(d) = verdict {
                extra.extend(d);
            }
            return Verdict::Fail(extra);
        }
        verdict
    } else {
        raw_compare(&p)
    }
}
pub fn unobserved(case: &RunCase, actual: &Actual) -> Vec<String> {
    if actual.response.get("error").is_some() {
        return vec![];
    }
    let mut missing = projection(case, actual).missing;
    if matches!(case.case.expect, Expectation::Oracle { .. }) {
        missing.extend(oracle(case, actual).2);
    }
    missing
}
