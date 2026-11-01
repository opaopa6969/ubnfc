//! スキーマそのものから全 oneOf/enum を列挙する。手書きの variant リストと
//! 実装が同じ項目を落としてしまうことを防ぐ。
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::fmt::Debug;
use ubnfc_front::declarative::LexicalExpression;
use ubnfc_front::ir::*;

fn resolve<'a>(schema: &'a Value, defs: &'a Value) -> &'a Value {
    schema.get("$ref").map_or(schema, |r| {
        &defs[r.as_str().unwrap().rsplit('/').next().unwrap()]
    })
}

fn minimal(schema: &Value, defs: &Value) -> Value {
    let schema = resolve(schema, defs);
    if let Some(v) = schema.get("const") {
        return v.clone();
    }
    if let Some(v) = schema.get("enum") {
        return v[0].clone();
    }
    if let Some(v) = schema.get("oneOf").or_else(|| schema.get("anyOf")) {
        return minimal(&v[0], defs);
    }
    if schema
        .get("pattern")
        .is_some_and(|p| p.as_str().unwrap().starts_with("^expr:"))
    {
        return json!("expr:test.ubnf:0:1:body");
    }
    let kind = schema["type"]
        .as_str()
        .unwrap_or_else(|| schema["type"][0].as_str().unwrap());
    match kind {
        "string" => {
            if schema.get("pattern").is_some() {
                json!("a".repeat(64))
            } else {
                json!("example")
            }
        }
        "integer" | "number" => schema.get("minimum").cloned().unwrap_or(json!(0)),
        "boolean" => json!(false),
        "null" => Value::Null,
        "array" => Value::Array(
            (0..schema["minItems"].as_u64().unwrap_or(0))
                .map(|_| minimal(&schema["items"], defs))
                .collect(),
        ),
        "object" => {
            let mut object = serde_json::Map::new();
            if let Some(required) = schema["required"].as_array() {
                for key in required {
                    object.insert(
                        key.as_str().unwrap().to_owned(),
                        minimal(&schema["properties"][key.as_str().unwrap()], defs),
                    );
                }
            }
            Value::Object(object)
        }
        other => panic!("unexpected schema type {other}"),
    }
}

// $ref 先は各 $defs のテストが網羅するため最小値のみ。それ以外の inline 型は再帰列挙。
fn cases(schema: &Value, defs: &Value) -> Vec<Value> {
    if schema.get("$ref").is_some() {
        return vec![minimal(schema, defs)];
    }
    if let Some(values) = schema["enum"].as_array() {
        return values.clone();
    }
    if schema.get("const").is_some() {
        return vec![minimal(schema, defs)];
    }
    if let Some(branches) = schema["oneOf"]
        .as_array()
        .or_else(|| schema["anyOf"].as_array())
    {
        return branches.iter().flat_map(|s| cases(s, defs)).collect();
    }
    if let Some(types) = schema["type"].as_array() {
        return types
            .iter()
            .flat_map(|t| {
                let mut s = schema.clone();
                s["type"] = t.clone();
                cases(&s, defs)
            })
            .collect();
    }
    let base = minimal(schema, defs);
    let mut values = vec![base.clone()];
    if let Some(properties) = schema["properties"].as_object() {
        for (key, prop) in properties {
            for sample in cases(prop, defs) {
                let mut value = base.clone();
                value[key] = sample;
                values.push(value);
            }
        }
    }
    if schema["type"] == "array" {
        for sample in cases(&schema["items"], defs) {
            let mut value = base.clone();
            if value.as_array().unwrap().is_empty() {
                value.as_array_mut().unwrap().push(sample);
            } else {
                value[0] = sample;
            }
            values.push(value);
        }
    }
    if schema["additionalProperties"].is_object() {
        let mut value = base;
        value["payload"] = minimal(&schema["additionalProperties"], defs);
        values.push(value);
    }
    values
}

fn invalid_cases(schema: &Value, value: &Value, defs: &Value) -> Vec<Value> {
    let schema = resolve(schema, defs);
    if schema.get("const").is_some() || schema.get("enum").is_some() {
        return vec![json!({"invalid": true})];
    }
    if let Some(branches) = schema["oneOf"].as_array() {
        if let Some(kind) = value.get("kind") {
            let branch = branches
                .iter()
                .find(|s| &s["properties"]["kind"]["const"] == kind)
                .unwrap();
            let mut result = invalid_cases(branch, value, defs);
            let mut invalid = value.clone();
            invalid["kind"] = json!("unknownVariant");
            result.push(invalid);
            return result;
        }
        return Vec::new(); // JsonValue は任意の JSON。
    }
    if let Some(branches) = schema["anyOf"].as_array() {
        return if value.is_null() {
            Vec::new()
        } else {
            invalid_cases(&branches[0], value, defs)
        };
    }
    let mut result = Vec::new();
    if schema["additionalProperties"] == false && value.is_object() {
        let mut unknown = value.clone();
        unknown["unknownField"] = json!(true);
        result.push(unknown);
        for key in schema["required"].as_array().unwrap() {
            let mut missing = value.clone();
            missing
                .as_object_mut()
                .unwrap()
                .remove(key.as_str().unwrap());
            result.push(missing);
        }
        for (key, field) in value.as_object().unwrap() {
            for invalid in invalid_cases(&schema["properties"][key], field, defs) {
                let mut nested = value.clone();
                nested[key] = invalid;
                result.push(nested);
            }
        }
    }
    if schema["type"] == "array" {
        for (i, item) in value.as_array().unwrap().iter().enumerate() {
            for invalid in invalid_cases(&schema["items"], item, defs) {
                let mut nested = value.clone();
                nested[i] = invalid;
                result.push(nested);
            }
        }
    }
    result
}

fn check<T: DeserializeOwned + Serialize + Debug>(name: &str, schema: &Value, defs: &Value) {
    for sample in cases(schema, defs) {
        let parsed: T = serde_json::from_value(sample.clone())
            .unwrap_or_else(|e| panic!("{name}: {e}: {sample}"));
        assert_eq!(
            serde_json::to_value(&parsed).unwrap(),
            sample,
            "round trip {name}"
        );
        for invalid in invalid_cases(schema, &sample, defs) {
            assert!(
                serde_json::from_value::<T>(invalid.clone()).is_err(),
                "{name} accepted {invalid}"
            );
        }
    }
}

#[test]
fn every_definition_and_inline_enum_round_trips_and_rejects_invalid_structure() {
    let schema: Value =
        serde_json::from_str(include_str!("../../../ir/schema/grammar-ir.v1.schema.json")).unwrap();
    let defs = &schema["$defs"];
    let mut names = Vec::new();
    macro_rules! check_defs {
        ($($ty:ident),+ $(,)?) => { $(
            names.push(stringify!($ty));
            check::<$ty>(stringify!($ty), &defs[stringify!($ty)], defs);
        )+ };
    }
    check_defs!(
        Analysis,
        Annotation,
        AstField,
        AstType,
        Boundary,
        Capture,
        Catalog,
        CharRange,
        ChoicePlan,
        CommonField,
        Contracts,
        CursorContract,
        CursorTransition,
        Delimiter,
        Diagnostics,
        Eval,
        Execution,
        Expr,
        ExprAnalysis,
        ExprId,
        ExprNode,
        GrammarUnit,
        ImportEdge,
        JsonValue,
        LexicalExpression,
        LexicalPlan,
        Ll1Operation,
        Ll1State,
        MayStart,
        MemoEntry,
        MemoPlan,
        MemoSafety,
        OccurrenceContract,
        Operator,
        OperatorPlan,
        Plan,
        PrefixTrie,
        Proof,
        Recipe,
        Recovery,
        RegexDfa,
        RegexDfaTransition,
        Rewrite,
        Rule,
        RuleAnalysis,
        ScanClass,
        ScanPlan,
        Scc,
        Scope,
        ScopeEffect,
        Semantics,
        Settings,
        Source,
        Span,
        Symbol,
        Token,
        TokenSpec,
        Trivia,
        TriviaPolicy,
        Typeof,
        ValueType,
    );
    names.sort();
    let expected: Vec<_> = defs
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(names, expected, "a schema definition has no Rust coverage");
    check::<GrammarIr>("GrammarIr", &schema, defs);
}

#[test]
fn json_value_covers_every_rust_variant_and_nested_payloads() {
    for value in [
        Value::Null,
        json!(true),
        json!(-42),
        json!(1.25),
        json!("text"),
        json!([null, {"x": 1}]),
        json!({"array": [true, 2.5]}),
    ] {
        let typed: JsonValue = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), value);
    }
}
