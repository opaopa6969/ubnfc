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
        // D-020: 旧 Java の rule 経路も情報のみ。受信値・report は変更しない。
        if matches!(name, "nativeFailure" | "rulePath" | "deepestRule") {
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
            // D-074: token の直接 capture は両 backend とも原文の文字列。旧 Java の
            // `javaInt`（NumericCaptureRuntimeTest.java:31-49）は旧実装の観測として残し、
            // 比較は `rustLexeme`（= 入力字句）で行う。数値変換の失敗（旧 Java の
            // NumberFormatException）も起きないので mappingError は null。
            p.field("value", json!(e.rust_lexeme), mapped_value(response));
            p.wire("mappingError", Value::Null, response);
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
    // D-022 / D-074: 数値 token の直接 capture は両 backend とも字句（原文の文字列）。
    // 旧 Java 採取 oracle の int を字句へ投影し、字句そのものは capture 原文と照合する。
    numeric_lexeme_contract(&mut expected["Oracle"], actual.response.get("ast"));
    if actual.backend == Backend::Rust {
        // D-011: 診断の主 offset は言語別 oracle。Rust 実走の採取値で置換する（farthestOffset は D-014 の共有契約）。
        if let Some(rust) = &case.case.rust_oracle {
            crate::rust_oracle::project_diagnostics(&mut expected["Oracle"]["diagnostics"], rust);
        }
    }
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
        // 未提供時に全入力 captures へ fallback しない。
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
                                    // D-020: 内部 rule 経路は欠落していても未観測に数えない。
                                    if key == "diagnostics"
                                        && matches!(field.as_str(), "rulePath" | "deepestRule")
                                    {
                                        continue;
                                    }
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
    // expected は backend 別投影（D-022 / D-011）後の値を返す。
    match (
        serde_json::from_value(expected.clone()),
        serde_json::from_value(observed.clone()),
    ) {
        (Ok(expected), Ok(observed)) => (expected, observed, missing),
        _ => (
            Expectation::Raw(expected["Oracle"].clone()),
            Expectation::Raw(observed["Oracle"].clone()),
            missing,
        ),
    }
}
/// `Integer.parseInt` の受理範囲（NumericCaptureRuntimeTest.java:31-49、generators.md「数値 token の型境界」）。
fn java_int(text: &str) -> Option<i64> {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse::<i32>().ok().map(i64::from)
}
fn within(inner: &Value, outer: &Value) -> bool {
    let span = |v: &Value| match v.as_array() {
        Some(a) if a.len() == 2 => Some((a[0].as_u64()?, a[1].as_u64()?)),
        _ => None,
    };
    match (span(inner), span(outer)) {
        (Some(i), Some(o)) => i.0 >= o.0 && i.1 <= o.1,
        _ => false,
    }
}
/// Java 採取 AST の int field を、同名・同範囲・同値へ変換できる capture 原文へ置換する。
fn lexeme_fields(node: &mut Value, captures: &[Value]) {
    let Some(span) = node.get("span").cloned() else {
        return;
    };
    let Some(fields) = node.get_mut("fields").and_then(Value::as_object_mut) else {
        return;
    };
    for (name, value) in fields.iter_mut() {
        let lexeme = |n: i64| {
            captures
                .iter()
                .find(|c| {
                    c["name"] == *name
                        && within(&c["span"], &span)
                        && c["text"].as_str().and_then(java_int) == Some(n)
                })
                .map(|c| c["text"].clone())
        };
        match value {
            Value::Number(number) => {
                if let Some(text) = number.as_i64().and_then(lexeme) {
                    *value = text;
                }
            }
            Value::Array(items) => {
                for item in items {
                    match item {
                        Value::Number(number) => {
                            if let Some(text) = number.as_i64().and_then(lexeme) {
                                *item = text;
                            }
                        }
                        Value::Object(_) => lexeme_fields(item, captures),
                        _ => {}
                    }
                }
            }
            Value::Object(_) => lexeme_fields(value, captures),
            _ => {}
        }
    }
}
fn has_text_field(node: &Value, name: &str, text: &str) -> bool {
    match node {
        Value::Object(map) => map
            .iter()
            .any(|(k, v)| (k == name && v.as_str() == Some(text)) || has_text_field(v, name, text)),
        Value::Array(items) => items.iter().any(|item| has_text_field(item, name, text)),
        _ => false,
    }
}
/// D-022 (docs/design/03-design-v1.md §8): oracle 系統の数値 capture を Rust 契約で比較する。
/// - AST の int は capture 原文（字句）へ置換して厳密比較する（RustConformanceTest.java:561-575、
///   NumericCaptureRuntimeTest.java:55-89 の「fields.value == 入力字句」）。
/// - Java の NumberFormatException（mappingError）は Rust では起きない。Rust の AST に
///   同名 capture の字句がそのまま入っていることを確認し、mappingError は null を期待する。
fn numeric_lexeme_contract(oracle: &mut Value, observed_ast: Option<&Value>) {
    let captures = oracle["captures"].as_array().cloned().unwrap_or_default();
    if let Some(ast) = oracle.get_mut("ast") {
        lexeme_fields(ast, &captures);
    }
    let lexeme = oracle["mappingError"]
        .as_str()
        .and_then(|m| m.strip_prefix("java.lang.NumberFormatException: For input string: \""))
        .and_then(|m| m.strip_suffix('"'))
        .map(str::to_owned);
    let Some(lexeme) = lexeme else {
        return;
    };
    let Some(capture) = captures
        .iter()
        .find(|c| c["text"].as_str() == Some(&lexeme))
    else {
        return;
    };
    let name = capture["name"].as_str().unwrap_or_default().to_owned();
    oracle["mappingError"] = Value::Null;
    oracle["ast"] = match observed_ast {
        Some(ast) if ast.is_object() && has_text_field(ast, &name, &lexeme) => ast.clone(),
        _ => {
            json!({"type":"<D-022 numeric lexeme>","span":capture["span"].clone(),"fields":{name:lexeme}})
        }
    };
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
        let (expected, observed, unobserved) = oracle(case, actual);
        let verdict = clean(divergence::compare(&expected, &observed, known));
        // parsedStatus も明示登録された観測だけを既知扱いにする。
        // references/scopeDepth 等の未登録・未知の観測は引き続き Fail。
        let extra = divergence::compare_oracle_values(
            &Value::Object(p.expected),
            &Value::Object(p.actual),
            known,
        );
        let mut failed = matches!(verdict, Verdict::Fail(_)) || matches!(extra, Verdict::Fail(_));
        let mut differences = Vec::new();
        for result in [extra, verdict] {
            if let Verdict::Fail(d) | Verdict::KnownDivergence(d) = result {
                differences.extend(d);
            }
        }
        // D-021 (docs/design/03-design-v1.md §8): follow-spec は「旧 oracle の値を再現しない」決定。
        // 登録済み観測群が実測と一致し、かつ観測済み（欠落・null oracle でない）なら既知バグ再現として Fail。
        let reproduced = reproduced_follow_spec(
            case,
            &differences,
            unobserved.iter().chain(&p.missing).map(String::as_str),
        );
        if !reproduced.is_empty() {
            failed = true;
            differences.extend(reproduced);
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
fn reproduced_follow_spec<'a>(
    case: &RunCase,
    differences: &[Difference],
    missing: impl Iterator<Item = &'a str>,
) -> Vec<Difference> {
    use std::collections::BTreeSet;
    let differing: BTreeSet<_> = differences
        .iter()
        .filter_map(|d| divergence::observation(&d.path))
        .collect();
    let unobserved: BTreeSet<_> = missing.filter_map(divergence::observation).collect();
    let oracle = serde_json::to_value(&case.case.expect).unwrap();
    let oracle = &oracle["Oracle"];
    let null_oracle = |o: Observation| {
        let keys: &[&str] = match o {
            Observation::Accepted => &["accepted", "allConsumed"],
            Observation::Consumed => &["consumed"],
            Observation::Matched => &["matched"],
            _ => &[],
        };
        keys.iter().any(|k| oracle[k].is_null())
    };
    case.case
        .follow_spec
        .iter()
        .filter(|o| {
            !matches!(o, Observation::Generation)
                && !differing.contains(o)
                && !unobserved.contains(o)
                && !null_oracle(**o)
        })
        .map(|o| Difference {
            path: format!("/{}", serde_json::to_value(o).unwrap().as_str().unwrap()),
            expected: Some(json!("follow-spec: 旧 oracle と異なる値（D-017）")),
            actual: Some(json!("旧 oracle と一致（既知バグ再現、D-021）")),
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{corpus::Case, runner::Options, rust_oracle::RustOracle};
    use serde_json::json;

    fn run_case(oracle: Value) -> RunCase {
        RunCase {
            case: Case {
                id: "numeric/1".into(),
                grammar_ref: vec![0],
                entry_rule: None,
                input: oracle["input"].as_str().unwrap_or("n7;").to_owned(),
                expect: serde_json::from_value(json!({"Oracle": {
                    "accepted": oracle["accepted"], "allConsumed": oracle["allConsumed"],
                    "consumed": oracle["consumed"], "matched": oracle["matched"],
                    "ast": oracle["ast"], "captures": oracle["captures"], "declarations": [],
                    "diagnostics": oracle["diagnostics"], "mappingError": oracle["mappingError"]
                }}))
                .unwrap(),
                known_divergences: vec![],
                follow_spec: vec![],
                rust_oracle: None,
            },
            grammar: "Probe".into(),
            variant: "numeric-capture".into(),
            family: "oracle".into(),
            options: Options::default(),
            fixture: Value::Null,
            extra_expectations: Default::default(),
        }
    }
    fn actual(backend: Backend, response: Value) -> Actual {
        Actual {
            backend,
            variant: "numeric-capture".into(),
            response,
        }
    }
    fn response(ast: Value, mapping_error: Value) -> Value {
        json!({"accepted": true, "allConsumed": true, "consumed": 3, "matched": 3, "ast": ast,
            "prefixCaptures": [{"name":"value","binding":"Root:0","span":[1,2],"text":"7"}],
            "declarations": [], "diagnostics": [], "mappingError": mapping_error})
    }
    fn int_oracle() -> Value {
        json!({"input":"n7;","accepted": true, "allConsumed": true, "consumed": 3, "matched": 3,
            "ast": {"type":"Value","span":[0,3],"fields":{"value":7}},
            "captures": [{"name":"value","binding":"Root:0","span":[1,2],"text":"7"}],
            "diagnostics": [], "mappingError": null})
    }

    /// D-074（D-022 を置換）: 両 backend とも字句で比較する。旧 Java oracle の int は
    /// 同名・同範囲・同値の capture 原文へ投影され、int を返す backend は Fail。
    #[test]
    fn d074_both_backends_compare_int_oracle_captures_as_lexemes() {
        let case = run_case(int_oracle());
        let lexeme = response(
            json!({"type":"Value","span":[0,3],"fields":{"value":"7"}}),
            Value::Null,
        );
        let int = response(
            json!({"type":"Value","span":[0,3],"fields":{"value":7}}),
            Value::Null,
        );
        for backend in [Backend::Rust, Backend::Java] {
            assert_eq!(
                compare(&case, &actual(backend, lexeme.clone()), &[]),
                Verdict::Pass,
                "{backend:?}"
            );
            assert!(
                matches!(
                    compare(&case, &actual(backend, int.clone()), &[]),
                    Verdict::Fail(_)
                ),
                "{backend:?}"
            );
        }
        // 字句そのものを照合する: 同値でも綴りが違えば Fail。
        let other = response(
            json!({"type":"Value","span":[0,3],"fields":{"value":"07"}}),
            Value::Null,
        );
        let Verdict::Fail(diffs) = compare(&case, &actual(Backend::Rust, other), &[]) else {
            panic!()
        };
        assert_eq!(diffs[0].path, "/ast/fields/value");
        assert_eq!(diffs[0].expected, Some(json!("7")));
    }

    #[test]
    fn d022_rust_has_no_number_format_error_but_must_keep_the_lexeme() {
        let mut oracle = int_oracle();
        oracle["input"] = json!("n1.5;");
        oracle["consumed"] = json!(5);
        oracle["matched"] = json!(5);
        oracle["ast"] = Value::Null;
        oracle["captures"] = json!([{"name":"value","binding":"Root:0","span":[1,4],"text":"1.5"}]);
        oracle["mappingError"] =
            json!("java.lang.NumberFormatException: For input string: \"1.5\"");
        let case = run_case(oracle);
        let mut ok = response(
            json!({"type":"Value","span":[0,5],"fields":{"value":"1.5"}}),
            Value::Null,
        );
        ok["consumed"] = json!(5);
        ok["matched"] = json!(5);
        ok["prefixCaptures"] =
            json!([{"name":"value","binding":"Root:0","span":[1,4],"text":"1.5"}]);
        assert_eq!(
            compare(&case, &actual(Backend::Rust, ok.clone()), &[]),
            Verdict::Pass
        );
        // D-074: Java も字句を保つ。旧 oracle どおりの mappingError はもう Fail。
        assert_eq!(
            compare(&case, &actual(Backend::Java, ok.clone()), &[]),
            Verdict::Pass
        );
        let mut java = ok.clone();
        java["ast"] = Value::Null;
        java["mappingError"] = json!("java.lang.NumberFormatException: For input string: \"1.5\"");
        assert!(matches!(
            compare(&case, &actual(Backend::Java, java), &[]),
            Verdict::Fail(_)
        ));
        // Rust が字句を落とした／別の値にした／AST を作らなかった場合は Fail。
        for ast in [
            json!({"type":"Value","span":[0,5],"fields":{"value":"1"}}),
            json!({"type":"Value","span":[0,5],"fields":{"other":"1.5"}}),
            Value::Null,
        ] {
            let mut bad = ok.clone();
            bad["ast"] = ast;
            let Verdict::Fail(diffs) = compare(&case, &actual(Backend::Rust, bad), &[]) else {
                panic!()
            };
            assert!(
                diffs.iter().any(|d| d.path.starts_with("/ast")),
                "{diffs:?}"
            );
        }
        let mut bad = ok;
        bad["mappingError"] = json!("x");
        assert!(matches!(
            compare(&case, &actual(Backend::Rust, bad), &[]),
            Verdict::Fail(_)
        ));
    }

    #[test]
    fn d011_rust_oracle_replaces_diagnostic_offsets_for_rust_only() {
        let mut oracle = int_oracle();
        oracle["input"] = json!("n1e+;");
        oracle["accepted"] = json!(false);
        oracle["allConsumed"] = json!(false);
        oracle["consumed"] = json!(0);
        oracle["matched"] = json!(0);
        oracle["ast"] = Value::Null;
        oracle["captures"] = json!([]);
        oracle["diagnostics"] = json!([{"kind":"syntax","offset":4,"expected":["DigitParser"],
            "farthestOffset":4,"farthestExpected":["DigitParser"],"origin":"Mapper.diagnose"}]);
        let mut case = run_case(oracle);
        case.case.rust_oracle = Some(RustOracle {
            diagnostics: vec![
                json!({"kind":"syntax","offset":2,"expected":[";"],"farthestOffset":2,"farthestExpected":[";"]}),
            ],
        });
        let at = |offset: u64| {
            json!({"accepted": false, "allConsumed": false, "consumed": 0, "matched": 0, "ast": null,
                "prefixCaptures": [], "declarations": [], "mappingError": null,
                "diagnostics": [{"kind":"syntax","offset":offset,"expected":["';'"],
                    "farthestOffset":4,"farthestExpected":["DigitParser"],"origin":"Mapper.diagnose"}]})
        };
        assert_eq!(
            compare(&case, &actual(Backend::Rust, at(2)), &[]),
            Verdict::Pass
        );
        assert!(matches!(
            compare(&case, &actual(Backend::Rust, at(4)), &[]),
            Verdict::Fail(_)
        ));
        assert_eq!(
            compare(&case, &actual(Backend::Java, at(4)), &[]),
            Verdict::Pass
        );
        assert!(matches!(
            compare(&case, &actual(Backend::Java, at(2)), &[]),
            Verdict::Fail(_)
        ));
    }

    #[test]
    fn d021_follow_spec_equal_to_the_old_oracle_is_a_reproduced_known_bug() {
        let mut case = run_case(int_oracle());
        case.case.known_divergences = vec![
            Observation::Consumed,
            Observation::Ast,
            Observation::Accepted,
        ];
        case.case.follow_spec = vec![Observation::Consumed];
        // D-074: 数値 capture は両 backend とも字句で返る。
        let same = response(
            json!({"type":"Value","span":[0,3],"fields":{"value":"7"}}),
            Value::Null,
        );
        let Verdict::Fail(diffs) = compare(
            &case,
            &actual(Backend::Java, same.clone()),
            &case.case.known_divergences,
        ) else {
            panic!("follow-spec と一致した consumed は既知バグ再現")
        };
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "/consumed");
        // 値が変わっていれば従来どおり既知乖離。
        let mut fixed = same.clone();
        fixed["consumed"] = json!(2);
        assert!(matches!(
            compare(
                &case,
                &actual(Backend::Java, fixed),
                &case.case.known_divergences
            ),
            Verdict::KnownDivergence(_)
        ));
        // new-contract だけの登録は一致しても Pass。
        case.case.follow_spec = vec![];
        assert_eq!(
            compare(
                &case,
                &actual(Backend::Java, same.clone()),
                &case.case.known_divergences
            ),
            Verdict::Pass
        );
        // 未観測（wire 欠落。allConsumed は consumed から再計算されるので両群とも未観測）や
        // 旧 oracle が null の観測群は再現判定しない。
        case.case.follow_spec = vec![Observation::Consumed, Observation::Accepted];
        let mut missing = same.clone();
        missing.as_object_mut().unwrap().remove("consumed");
        assert_eq!(
            compare(
                &case,
                &actual(Backend::Java, missing.clone()),
                &case.case.known_divergences
            ),
            Verdict::Pass
        );
        assert!(
            unobserved(&case, &actual(Backend::Java, missing)).contains(&"/consumed".to_string())
        );
        let mut null_oracle = run_case(int_oracle());
        let Expectation::Oracle { accepted, .. } = &mut null_oracle.case.expect else {
            panic!()
        };
        *accepted = None;
        null_oracle.case.known_divergences = vec![Observation::Accepted];
        null_oracle.case.follow_spec = vec![Observation::Accepted];
        assert_eq!(
            compare(
                &null_oracle,
                &actual(Backend::Java, same),
                &null_oracle.case.known_divergences
            ),
            Verdict::Pass
        );
    }
}
