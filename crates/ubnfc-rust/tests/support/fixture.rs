use serde_json::{json, Value};
use ubnfc_front::ir::{from_json, GrammarIr};

pub fn literal(value: &str) -> Value {
    json!({"kind":"literal","value":value,"caseSensitive":true,"wordBoundary":false})
}
/// D-040 の語境界つきリテラル。
pub fn bounded_literal(value: &str) -> Value {
    json!({"kind":"literal","value":value,"caseSensitive":true,"wordBoundary":true})
}
pub fn seq(items: Vec<Value>) -> Value {
    json!({"kind":"seq","items":items})
}
pub fn choice(alternatives: Vec<Value>, selection: &str) -> Value {
    json!({"kind":"choice","alternatives":alternatives,"selection":selection,"tieBreak":"declarationOrder"})
}
pub fn unary(kind: &str, child: Value) -> Value {
    json!({"kind":kind,"child":child})
}
pub fn repeat(child: Value, min: u64, max: u64) -> Value {
    json!({"kind":"repeat","child":child,"min":min,"max":max,"terminator":null})
}
pub fn reference(name: &str) -> Value {
    json!({"kind":"ruleRef","symbolId":format!("Operators::{name}")})
}
pub fn token() -> Value {
    json!({"kind":"tokenRef","symbolId":"Operators::NUMBER"})
}
pub fn capture(name: &str, body: Value) -> Value {
    json!({"capture":name,"body":body})
}

pub fn flatten(j: &mut Value, mut node: Value, rule: &str, role: &str) -> String {
    let mut captured = Vec::new();
    while let Some(name) = node.get("capture").and_then(Value::as_str) {
        captured.push(name.to_owned());
        node = node["body"].take();
    }
    let source = j["sources"][0].clone();
    let id = format!(
        "expr:{}:0:{}:{rule}/{role}",
        source["id"].as_str().unwrap(),
        source["codePointLength"]
    );
    let kind = node["kind"].as_str().unwrap().to_owned();
    let transaction = match kind.as_str() {
        "seq" | "lookahead" | "negativeLookahead" | "separated" => "once",
        "choice" => "perAlternative",
        "repeat" | "optional" => "occurs",
        "group" | "ruleRef" | "tokenRef" => "delegate",
        _ => "none",
    };
    let e = json!({"id":id,"node":{},"ruleId":format!("Operators::{rule}"),"source":{"sourceId":source["id"],"start":0,"end":source["codePointLength"]},"captureSiteIds":[],"diagnosticLabel":null,"execution":{"childMode":"inherit","entryFlag":if kind=="seq"{"setFalse"}else{"preserve"},"failureCursor":if transaction=="none"{"primitiveMayResetMatched"}else if transaction=="delegate"{"delegate"}else{"restoreTransaction"},"primitiveProfile":null,"transaction":transaction}});
    let index = j["semantics"]["expressions"].as_array().unwrap().len();
    j["semantics"]["expressions"]
        .as_array_mut()
        .unwrap()
        .push(e);
    let mut analysis: Value =
        serde_json::from_str::<Value>(include_str!("../ir/operators.ir.json")).unwrap()["analysis"]
            ["expressions"][0]
            .clone();
    analysis["exprId"] = json!(id);
    j["analysis"]["expressions"]
        .as_array_mut()
        .unwrap()
        .push(analysis);
    for key in ["items", "alternatives"] {
        if let Some(items) = node.get_mut(key) {
            let children = items.take();
            let mut ids = vec![];
            for (i, child) in children.as_array().unwrap().iter().enumerate() {
                ids.push(flatten(j, child.clone(), rule, &format!("{role}/{i}")));
            }
            node[key] = json!(ids);
        }
    }
    for (i, key) in ["child", "element", "separator", "terminator"]
        .iter()
        .enumerate()
    {
        if node.get(*key).is_some_and(Value::is_object) {
            let child = node[*key].take();
            node[*key] = json!(flatten(j, child, rule, &format!("{role}/child{i}")));
        }
    }
    j["semantics"]["expressions"][index]["node"] = node;
    for (capture_index, name) in captured.into_iter().rev().enumerate() {
        let site = format!("{id}/capture/{capture_index}");
        j["semantics"]["expressions"][index]["captureSiteIds"]
            .as_array_mut()
            .unwrap()
            .push(json!(site));
        j["semantics"]["captures"].as_array_mut().unwrap().push(json!({"siteId":site,"exprId":id,"ruleId":format!("Operators::{rule}"),"name":name,"cardinality":"scalar","conversion":"text","occurrenceOrder":"completionInnerBeforeOuter","skip":false,"spanRule":"consumedExtent","valueType":{"kind":"text"}}));
    }
    id
}
pub fn grammar(rules: Vec<(&str, Value)>, spec: Option<Value>) -> GrammarIr {
    let mut j: Value = serde_json::from_str(include_str!("../ir/operators.ir.json")).unwrap();
    let symbol = j["semantics"]["symbols"][0].clone();
    let rule_template = j["semantics"]["rules"][0].clone();
    let number = j["semantics"]["tokens"][0].clone();
    for key in [
        "astTypes",
        "captures",
        "expressions",
        "recipes",
        "rules",
        "symbols",
    ] {
        j["semantics"][key] = json!([]);
    }
    for key in ["expressions", "rules", "prefixTries", "sccs"] {
        j["analysis"][key] = json!([]);
    }
    j["semantics"]["trivia"]["insertionPlan"] = json!([]);
    j["plan"]["memo"]["entries"] = json!([]);
    j["plan"]["choices"] = json!([]);
    j["plan"]["operators"] = json!([]);
    j["requiredFeatures"] = json!([
        "backendDiagnostics",
        "captures",
        "commonField",
        "cursorFrameFlag",
        "deferredMapping",
        "dualCursor",
        "enum",
        "externTokens",
        "longestChoice",
        "memoReplay",
        "orderedChoice",
        "predictiveChoice",
        "regex",
        "scopeUndo",
        "structuredOccurrences",
        "triviaInsertion"
    ]);
    j["entry"]["ruleId"] = json!(format!("Operators::{}", rules[0].0));
    let mut ids = vec![];
    for (name, body) in rules {
        let id = format!("Operators::{name}");
        ids.push(id.clone());
        let body = flatten(&mut j, body, name, "body");
        let mut r = rule_template.clone();
        r["symbolId"] = json!(id);
        r["body"] = json!(body);
        r["operator"] = Value::Null;
        r["mappingTypeId"] = Value::Null;
        r["mappingParams"] = json!([]);
        j["semantics"]["rules"].as_array_mut().unwrap().push(r);
        let mut sym = symbol.clone();
        sym["id"] = json!(id);
        sym["qualifiedSymbolId"] = json!(id);
        sym["name"] = json!(name);
        sym["originalName"] = json!(name);
        sym["kind"] = json!("rule");
        j["semantics"]["symbols"].as_array_mut().unwrap().push(sym);
        j["analysis"]["rules"].as_array_mut().unwrap().push(json!({"ruleId":id,"sccId":"scc:all","followSyncPatterns":null,"leftRecursion":"none","progressProven":false}));
    }
    j["analysis"]["sccs"] =
        json!([{"id":"scc:all","ruleIds":ids,"recursive":false,"memoPolicy":"allEligible"}]);
    let mut sym = symbol;
    sym["id"] = json!("Operators::NUMBER");
    sym["qualifiedSymbolId"] = json!("Operators::NUMBER");
    sym["name"] = json!("NUMBER");
    sym["kind"] = json!("token");
    j["semantics"]["symbols"].as_array_mut().unwrap().push(sym);
    j["semantics"]["tokens"] = json!([number]);
    if let Some(spec) = spec {
        j["semantics"]["tokens"][0]["spec"] = spec;
    }
    from_json(&j.to_string()).unwrap()
}
