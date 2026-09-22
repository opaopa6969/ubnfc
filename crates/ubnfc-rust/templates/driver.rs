use std::io::{self, BufRead, Write};
use ubnfc_generated::{ast::*, parse_entry_with_options, DiagnosticKind, ParseOptions, Severity};
#[path = "driver_json.rs"]
mod json;
use json::{Json, ToJson};

fn required<'a>(value: &'a Json, key: &str) -> Result<&'a Json, &'static str> {
    value.get(key).ok_or("missing request field")
}
fn execute(request: &Json) -> Result<Json, &'static str> {
    let id = required(request, "id")?.clone();
    let grammar = required(request, "grammar")?.string()?;
    let entry = match required(request, "entry")? {
        Json::Null => None,
        other => Some(other.string()?),
    };
    let input = required(request, "input")?.string()?;
    let options = required(request, "options")?;
    let lexical = required(options, "lexical")?.boolean()?;
    let scope = required(options, "scope")?.boolean()?;
    let prefix = parse_entry_with_options(
        grammar,
        entry,
        input,
        ParseOptions {
            build_ast: false,
            require_eof: false,
            lexical: false,
            ..Default::default()
        },
    )?;
    let result = parse_entry_with_options(
        grammar,
        entry,
        input,
        ParseOptions {
            build_ast: true,
            require_eof: true,
            lexical,
            ..Default::default()
        },
    )?;
    let ast = result.ast.as_ref().map_or(Json::Null, ToJson::to_json);
    let value = if result.ast.as_ref().is_some_and(operator_ast) {
        numeric(&ast)?.map_or(Json::Null, |v| Json::Number(v.to_string()))
    } else {
        Json::Null
    };
    let mut mapping = Json::Null;
    let mut diagnostics = Vec::new();
    for d in &result.diagnostics {
        let kind = match d.kind {
            DiagnosticKind::Syntax => "syntax",
            DiagnosticKind::TrailingInput => "trailing_input",
            DiagnosticKind::Mapping => {
                mapping = d.message.to_json();
                "mapping"
            }
        };
        diagnostics.push(Json::object([
            ("kind", kind.to_json()),
            ("offset", d.offset_cp.to_json()),
            ("expected", d.expected.to_json()),
            ("farthestOffset", d.farthest_cp.to_json()),
            ("farthestExpected", d.farthest_expected.to_json()),
        ]));
    }
    for d in &result.scope_diagnostics {
        let severity = match d.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARNING",
            Severity::Info => "INFO",
            Severity::Hint => "HINT",
        };
        diagnostics.push(Json::object([
            ("kind", "scope".to_json()),
            ("offset", d.offset.to_json()),
            ("expected", Json::Array(vec![])),
            ("message", d.msg.to_json()),
            ("length", d.len.to_json()),
            ("severity", severity.to_json()),
        ]));
    }
    let mut response = Json::object([
        ("id", id),
        ("accepted", Json::Bool(prefix.ok)),
        ("allConsumed", Json::Bool(result.ok)),
        ("consumed", prefix.consumed_cp.to_json()),
        ("matched", prefix.matched_cp.to_json()),
        ("ast", ast),
        ("diagnostics", Json::Array(diagnostics)),
        ("mappingError", mapping),
        ("value", value),
    ]);
    let Json::Object(fields) = &mut response else {
        unreachable!()
    };
    if lexical {
        fields.insert(
            "captures".into(),
            Json::Array(
                result
                    .captures
                    .iter()
                    .map(|c| {
                        let text: String = input
                            .chars()
                            .skip(c.span[0])
                            .take(c.span[1] - c.span[0])
                            .collect();
                        Json::object([
                            ("name", c.name.to_json()),
                            ("span", c.span.to_json()),
                            ("text", text.to_json()),
                        ])
                    })
                    .collect(),
            ),
        );
    }
    if scope {
        fields.insert(
            "scope".into(),
            Json::object([
                ("mode", result.scope.mode.to_json()),
                (
                    "events",
                    Json::Array(
                        result
                            .scope
                            .events
                            .iter()
                            .map(|event| {
                                Json::object([
                                    ("order", event.order.to_json()),
                                    ("action", event.action.to_json()),
                                    ("mode", event.mode.to_json()),
                                    ("name", event.name.to_json()),
                                    ("offset", event.offset_cp.to_json()),
                                    ("length", event.length_cp.to_json()),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]),
        );
        fields.insert(
            "declarations".into(),
            Json::Array(
                result
                    .declarations
                    .iter()
                    .map(|d| {
                        Json::object([
                            ("name", d.name.to_json()),
                            ("sourceOffset", d.offset_cp.to_json()),
                        ])
                    })
                    .collect(),
            ),
        );
        fields.insert(
            "references".into(),
            Json::Array(
                result
                    .references
                    .iter()
                    .map(|r| {
                        Json::object([
                            ("name", r.name.to_json()),
                            ("offset", r.offset.to_json()),
                            ("length", r.len.to_json()),
                        ])
                    })
                    .collect(),
            ),
        );
    }
    Ok(response)
}
fn finite(value: f64) -> Result<Option<f64>, &'static str> {
    if value.is_finite() {
        Ok(Some(value))
    } else {
        Err("non-finite evaluation value")
    }
}
// Java driver と同じ範囲。入力ではなく AST の数値葉と left/op/right を評価する。
fn numeric(value: &Json) -> Result<Option<f64>, &'static str> {
    match value {
        Json::Number(n) => return finite(n.parse().map_err(|_| "invalid number")?),
        Json::String(s) => {
            let s = s.trim();
            let end = s
                .find(|c: char| !c.is_ascii_digit() && !matches!(c, '+' | '-' | '.' | 'e' | 'E'))
                .unwrap_or(s.len());
            let mut rest = s[end..].trim_start();
            while !rest.is_empty() {
                if let Some(comment) = rest.strip_prefix("//") {
                    rest = comment
                        .find('\n')
                        .map_or("", |i| comment[i + 1..].trim_start());
                } else if let Some(comment) = rest.strip_prefix("/*") {
                    let Some(i) = comment.find("*/") else {
                        return Ok(None);
                    };
                    rest = comment[i + 2..].trim_start();
                } else {
                    return Ok(None);
                }
            }
            return match s[..end].parse::<f64>() {
                Ok(n) => finite(n),
                Err(_) => Ok(None),
            };
        }
        _ => {}
    }
    let Some(Json::Object(fields)) = value.get("fields") else {
        return Ok(None);
    };
    if fields.len() == 1 {
        return fields.get("value").map_or(Ok(None), numeric);
    }
    let (Some(left), Some(Json::Array(ops)), Some(Json::Array(rights))) =
        (fields.get("left"), fields.get("op"), fields.get("right"))
    else {
        return Ok(None);
    };
    if fields.len() != 3 {
        return Ok(None);
    }
    if *left == Json::Null && rights.is_empty() && ops.len() == 1 {
        return numeric(&ops[0]);
    }
    if ops.len() != rights.len() {
        return Ok(None);
    }
    let Some(mut left) = numeric(left)? else {
        return Ok(None);
    };
    for (op, right) in ops.iter().zip(rights) {
        let (Ok(op), Some(right)) = (op.string(), numeric(right)?) else {
            return Ok(None);
        };
        left = match op.trim() {
            "+" => left + right,
            "-" => left - right,
            "*" => left * right,
            "/" => left / right,
            "^" => left.powf(right),
            _ => return Ok(None),
        };
        finite(left)?;
    }
    Ok(Some(left))
}
fn main() -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        let line = line?;
        let request = json::read(&line);
        let id = request
            .as_ref()
            .ok()
            .and_then(|r| r.get("id"))
            .cloned()
            .unwrap_or(Json::Null);
        let response = request
            .and_then(|r| execute(&r))
            .unwrap_or_else(|error| Json::object([("id", id), ("error", error.to_json())]));
        let mut text = String::new();
        response.write(&mut text);
        writeln!(stdout, "{text}")?;
        stdout.flush()?;
    }
    Ok(())
}
