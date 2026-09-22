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
        // D-019 (docs/design/03-design-v1.md §8): 内部診断は保存するが Verdict 契約には含めない。
        if name == "nativeFailure" {
            return;
        }
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
                // NestedCaptureConformanceTest.java:283-284,320-327:
                // fixture.captureNames で絞り、完了順と重複はそのまま比較する。
                let observed = response.get("captures").map(|value| {
                    match (
                        case.fixture.get("captureNames").and_then(Value::as_array),
                        value.as_array(),
                    ) {
                        (Some(names), Some(items)) => json!(items
                            .iter()
                            .filter(|item| item.get("name").is_none_or(|name| names.contains(name)))
                            .collect::<Vec<_>>()),
                        _ => value.clone(),
                    }
                });
                p.field("captures", json!(captures), observed);
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
            if e.parent_rollback.is_some() {
                // ScopeAnnotationConformanceTest.java:162-169,238-257:
                // キーの存在は probe 起動条件で、期待値は外側 rollback 後の状態。
                let queries = case.fixture.get("queries").and_then(Value::as_array);
                if let Some(queries) = queries {
                    p.wire("rollback", json!({"cursor":[0,0],"scope":{
                        "depth":0,"resolved":queries.iter().map(|query| json!({"query":query,"symbol":null})).collect::<Vec<_>>(),
                        "declarations":[],"references":[],"diagnostics":[]
                    }}), response);
                } else {
                    p.missing.push("/fixture/queries".into());
                }
            }
            for (key, value) in &e.extra {
                p.wire(key, value.clone(), response);
            }
        }
        Expectation::TokenValue(e) => {
            p.full(e.value.is_some(), response);
            if let Some(value) = &e.value {
                // RustConformanceTest.java:561-575: value は評価値でなく mapped AST field。
                p.wire(
                    "ast",
                    json!({"type":"Value", "span":[0,case.case.input.chars().count()],
                    "fields":{"value":value}}),
                    response,
                );
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
            // RustConformanceTest.java:680-690 / SemanticCardinalityConformance.java:154-177:
            // 不受理 variant では AST/value/span oracle を検証しない。
            if accepted {
                if let Some(texts) = &e.texts {
                    // Java は canonical AST、Rust は owned Text の span も検証する。
                    // RustConformanceTest.java:147-153、SemanticCardinalityConformance.java:175。
                    if actual.backend == Backend::Rust
                        || !["semantic-cardinality", "mixed-values"].contains(&case.family.as_str())
                    {
                        p.wire("texts", json!(texts), response);
                    }
                }
                if let Some(nodes) = &e.nodes {
                    p.wire("nodes", json!(nodes), response);
                }
                if let Some(expected) = &e.expected {
                    if case.family == "cardinality" {
                        p.wire("ast", cardinality_ast(expected, &case.case.input), response);
                    } else {
                        p.wire("expected", expected.clone(), response);
                    }
                }
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
                // TinyStringTokenConformanceTest.java:230-237 は AST の value を読む。
                if case.family == "tiny-string-token" {
                    p.wire(
                        "ast",
                        json!({"type":"Value","span":e.span,"fields":{"value":value}}),
                        response,
                    );
                } else {
                    p.value(value, response);
                }
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
                diagnostic
                    .as_ref()
                    .map(|d| d.get("offset").cloned().unwrap_or(Value::Null)),
            );
            // RustConformanceTest.java:756-765: syntax は非空、trailing_input は厳密比較。
            p.field(
                "expected",
                if e.kind == "trailing_input" {
                    json!(["end of input"])
                } else {
                    json!(true)
                },
                diagnostic.map(|d| {
                    if e.kind == "trailing_input" {
                        d.get("expected").cloned().unwrap_or(Value::Null)
                    } else {
                        json!(d
                            .get("expected")
                            .and_then(Value::as_array)
                            .is_some_and(|v| !v.is_empty()))
                    }
                }),
            );
            for (key, value) in &e.extra {
                p.wire(key, value.clone(), response);
            }
        }
        Expectation::Numeric(e) => {
            p.full(true, response);
            match actual.backend {
                // NumericCaptureRuntimeTest.java:31-49,81-86: AST field の int/lexeme。
                Backend::Rust => p.field("value", json!(e.rust_lexeme), mapped_value(response)),
                Backend::Java => {
                    if let Some(value) = e.java_int {
                        p.field("value", json!(value), mapped_value(response));
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
        Expectation::Oracle { .. }
        | Expectation::GenerationFailure { .. }
        | Expectation::ExpectedRejection { .. } => {}
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
fn mapped_value(response: &Value) -> Option<Value> {
    response
        .get("ast")
        .map(|ast| ast["fields"]["value"].clone())
}

// RustConformanceTest.java:698-721: 簡略 oracle を canonical AST に展開する。
// driver に fixture 固有の /expected キーを要求しない。
fn cardinality_ast(expected: &Value, input: &str) -> Value {
    fn item(value: &Value) -> Value {
        if value.is_null() {
            return Value::Null;
        }
        json!({"type":"Item","span":[value[1],value[2]],"fields":{"value":value[0]}})
    }
    json!({"type":"Container","span":[0,input.chars().count()],"fields":{
        "head":item(&expected["head"]),
        "values":expected["items"].as_array().unwrap().iter().map(item).collect::<Vec<_>>(),
        "tail":expected["tail"],
        "flags":expected["flags"].as_str().unwrap().chars().map(|c| c.to_string()).collect::<Vec<_>>()
    }})
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
            // ScopeAnnotationConformanceTest.java:138-141,222-249 は ScopeStore の record 列。
            // 全診断 wire から scope のみ射影。message/severity/offset/length は厳密。
            let observed = if key == "resolved" {
                let mut observed = observed.clone();
                if let (Some(expected), Some(actual)) = (value.as_array(), observed.as_array_mut())
                {
                    for (expected, actual) in expected.iter().zip(actual) {
                        if let Some(actual) = actual.as_object_mut() {
                            // ScopeAnnotationConformanceTest.java:219-235 は query/symbol のみ。
                            // 未指定 declared は比較せず、明示 null は null または欠落を許す。
                            match expected.get("declared") {
                                None => {
                                    actual.remove("declared");
                                }
                                Some(Value::Null) => {
                                    actual.entry("declared").or_insert(Value::Null);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                observed
            } else if key == "diagnostics"
                && response
                    .get("scope")
                    .and_then(|scope| scope.get(key))
                    .is_none()
            {
                observed
                    .as_array()
                    .map(|items| {
                        json!(items
                            .iter()
                            .filter(|d| d["kind"] != "syntax" && d["kind"] != "trailing_input")
                            .map(|d| {
                                let mut d = d.clone();
                                if let Some(d) = d.as_object_mut() {
                                    for field in ["kind", "origin", "expected"] {
                                        d.remove(field);
                                    }
                                }
                                d
                            })
                            .collect::<Vec<_>>())
                    })
                    .unwrap_or_else(|| observed.clone())
            } else {
                observed.clone()
            };
            a.insert(key.clone(), observed);
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
    let mut response = actual.response.clone();
    // OracleCollector.java:175 / corpus/oracle/README.md:84-85:
    // oracle の allConsumed は構文成否と独立な prefix cursor。wire は全入力構文受理。
    if let Some(consumed) = response.get("consumed").and_then(Value::as_u64) {
        response["allConsumed"] = json!(consumed == case.case.input.chars().count() as u64);
    } else if let Some(object) = response.as_object_mut() {
        object.remove("allConsumed");
    }
    // RustConformanceTest.java:563-575 / OracleCollector.java:198-203:
    // diagnose は全入力成功なら空。成功枝の投機的 syntax 診断は nativeFailure 側。
    if actual.response["allConsumed"] == true {
        if let Some(items) = response
            .get_mut("diagnostics")
            .and_then(Value::as_array_mut)
        {
            items.retain(|d| d["kind"] != "syntax" && d["kind"] != "trailing_input");
        }
    }
    for (key, value) in expected["Oracle"].as_object_mut().unwrap() {
        if matches!(
            key.as_str(),
            "accepted" | "allConsumed" | "consumed" | "matched"
        ) && value.is_null()
        {
            continue;
        }
        // OracleCollector.java:171-174 / SemanticsTest.java:143-151:
        // oracle の capture は prefix 解析から採取され、全入力失敗でも残る。
        // 未提供時に全入力 captures へ fallback しない（Rust driver は未出力）。
        let wire_key = if key == "captures" {
            "prefixCaptures"
        } else {
            key
        };
        match response.get(wire_key) {
            None => missing.push(format!("/{wire_key}")),
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
                                // RustConformanceTest.java:756-765,840-844。
                                // primary syntax hints の表記は比較せず非空契約を守る。
                                // trailing_input / farthestExpected は弱めない。
                                if key == "diagnostics"
                                    && e.get("kind") == Some(&json!("syntax"))
                                    && a.get("kind") == Some(&json!("syntax"))
                                    && e.get("expected")
                                        .and_then(Value::as_array)
                                        .is_some_and(|v| !v.is_empty())
                                    && a.get("expected")
                                        .and_then(Value::as_array)
                                        .is_some_and(|v| !v.is_empty())
                                {
                                    a.insert("expected".into(), e["expected"].clone());
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
    if let Expectation::ExpectedRejection { stage, code, .. } = &case.case.expect {
        return Verdict::Fail(vec![Difference {
            path: "/rejection".into(),
            expected: Some(json!({"stage":stage,"code":code})),
            actual: Some(json!("runtime reached")),
        }]);
    }
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
        // parsedStatus も明示登録された観測だけを既知扱いにする。
        // references/scopeDepth 等の未登録・未知の観測は引き続き Fail。
        let extra = divergence::compare_oracle_values(
            &Value::Object(p.expected),
            &Value::Object(p.actual),
            known,
        );
        let failed = matches!(verdict, Verdict::Fail(_)) || matches!(extra, Verdict::Fail(_));
        let mut differences = Vec::new();
        for result in [extra, verdict] {
            if let Verdict::Fail(d) | Verdict::KnownDivergence(d) = result {
                differences.extend(d);
            }
        }
        if failed {
            Verdict::Fail(differences)
        } else if differences.is_empty() {
            Verdict::Pass
        } else {
            Verdict::KnownDivergence(differences)
        }
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
