//! harness JSON Lines アダプター。parser 本体の意味論は変更しない。
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use ubnfc_target_rust::{api::DiagnosticKind, canonical, operators, power, ParseOptions};

#[derive(Deserialize)]
struct Options {
    lexical: bool,
    scope: bool,
}
#[derive(Deserialize)]
struct Request {
    id: Value,
    grammar: String,
    entry: Option<String>,
    input: String,
    options: Options,
}
fn execute(request: Request) -> Result<Value, String> {
    let root = match request.grammar.as_str() {
        "Operators" => "Expression",
        "Power" => "Expr",
        grammar => return Err(format!("未対応の grammar: {grammar}")),
    };
    if request.entry.as_deref().is_some_and(|entry| entry != root) {
        return Err(format!("未対応の entry: {:?}", request.entry));
    }
    let parse = |options| match request.grammar.as_str() {
        "Operators" => operators::parse_with_options(&request.input, options),
        _ => power::parse_with_options(&request.input, power::Mode::Text, options),
    };
    let prefix = parse(ParseOptions {
        build_ast: false,
        require_eof: false,
    });
    let result = parse(ParseOptions::default());
    let value = result.ast.as_ref().map(|ast| ast.evaluate());
    if value.is_some_and(|value| !value.is_finite()) {
        return Err("評価値が有限の JSON 数値ではありません".into());
    }
    let diagnostics: Vec<_> = result.diagnostics.iter().map(|d| json!({
        "kind": match d.kind { DiagnosticKind::Syntax => "syntax", DiagnosticKind::Mapping => "mapping" },
        "offset": d.offset_cp, "expected": d.expected,
    })).collect();
    let mapping = result
        .diagnostics
        .iter()
        .find(|d| d.kind == DiagnosticKind::Mapping)
        .and_then(|d| d.message.as_ref());
    let mut response = json!({"id":request.id, "accepted":prefix.ok, "allConsumed":result.ok,
        "consumed":prefix.consumed_cp, "matched":prefix.matched_cp,
        "ast":result.ast.as_ref().map(canonical::value), "diagnostics":diagnostics,
        "mappingError":mapping, "value":value});
    if request.options.lexical {
        response["captures"] = json!(result.captures.iter().map(|c| json!({"name":c.name, "span":c.span,
            "text":request.input.chars().skip(c.span[0]).take(c.span[1]-c.span[0]).collect::<String>()})).collect::<Vec<_>>());
    }
    if request.options.scope {
        response["declarations"] = json!([]);
        response["references"] = json!([]);
    }
    Ok(response)
}
fn main() -> io::Result<()> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line?;
        let raw = serde_json::from_str::<Value>(&line);
        let id = raw
            .as_ref()
            .ok()
            .and_then(|raw| raw.get("id"))
            .cloned()
            .unwrap_or(Value::Null);
        let result = raw
            .map_err(|error| error.to_string())
            .and_then(|raw| serde_json::from_value(raw).map_err(|error| error.to_string()))
            .and_then(execute)
            .unwrap_or_else(|error| json!({"id":id, "error":error}));
        writeln!(output, "{result}")?;
        output.flush()?;
    }
    Ok(())
}
