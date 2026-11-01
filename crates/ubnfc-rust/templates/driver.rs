use std::io::{self, BufRead, Write};
#[cfg(not(feature = "scanners"))]
use ubnfc_generated::parse_entry_with_options;
use ubnfc_generated::{ast::*, DiagnosticKind, ParseOptions, Severity};
#[path = "driver_json.rs"]
mod json;
use json::{Json, ToJson};

fn required<'a>(value: &'a Json, key: &str) -> Result<&'a Json, &'static str> {
    value.get(key).ok_or("missing request field")
}
#[cfg(feature = "scanners")]
#[path = "scanners.rs"]
mod scanners;

fn parse_registered(
    grammar: &str,
    entry: Option<&str>,
    input: &str,
    options: ParseOptions,
) -> Result<ubnfc_generated::ParseResult, &'static str> {
    #[cfg(feature = "scanners")]
    {
        let mut registry = scanners::Registry::new(EXTERN_CLASSES)?;
        scanner_parser::parse_entry_with_scanner(grammar, entry, input, options, &mut registry)
    }
    #[cfg(not(feature = "scanners"))]
    {
        if !EXTERN_CLASSES.is_empty() {
            return Err("Extern scanner 未登録: scanners feature が必要です");
        }
        parse_entry_with_options(grammar, entry, input, options)
    }
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
    // D-072 の A/B 用の任意キー。`options.scan` があるときは「fast mode で走らせて
    // 一括走査だけを切り替える」という意味にする（走査は fast mode でしか効かず、
    // 出現表を要求すると畳まないため、診断と出現表を外して同じ条件で 2 度測る）。
    // 既定（キーが無い）は従来どおり診断ありで、走査経路には入らない。
    let scan_ab = match options.get("scan") {
        Some(value) => Some(value.boolean()?),
        None => None,
    };
    let ll1_ab = match options.get("ll1") {
        Some(value) => Some(value.boolean()?),
        None => None,
    };
    let scan = scan_ab.unwrap_or(true);
    // 各最適化を独立に比較する。scan A/B の両側では LL(1) を外す。
    let ll1 = ll1_ab.unwrap_or(scan_ab.is_none());
    let ab = scan_ab.is_some() || ll1_ab.is_some();
    let diagnostics = !ab;
    let lexical = lexical && !ab;
    let observations = request.get("observations").unwrap_or(&Json::Null);
    let family = observations.get("family").and_then(|v| v.string().ok());
    let max_depth = match options.get("maxDepth") {
        Some(Json::Number(n)) => n.parse().map_err(|_| "invalid maxDepth")?,
        None => ParseOptions::default().max_depth,
        _ => return Err("invalid maxDepth"),
    };
    // 位置ごとの字句 cache は既定 off。corpus 全体を cache 有効で回すための脱出口。
    let lexical_cache = std::env::var_os("UBNFC_LEXICAL_CACHE").is_some_and(|v| v != "0");
    let prefix = parse_registered(
        grammar,
        entry,
        input,
        ParseOptions {
            build_ast: false,
            require_eof: false,
            lexical,
            lexical_cache,
            max_depth,
            scan,
            ll1,
            // driver は成功時も hints（投機的失敗）を出すので、診断を明示的に要求する。
            diagnostics,
            // prefix parse の出現は `lexical` のときだけ観測に出る。`lexical: true` が
            // 出現表の要求を含むので（D-035）、ここで別に要求する必要はない。value span は使わない。
            ..Default::default()
        },
    )?;
    let result = parse_registered(
        grammar,
        entry,
        input,
        ParseOptions {
            build_ast: true,
            require_eof: true,
            lexical,
            lexical_cache,
            max_depth,
            scan,
            ll1,
            diagnostics,
            // AST が出たときは必ず texts / nodes / valueSpans を出すので value span を要求する。
            // 出現表は `lexical` のときだけ観測に出る（D-035。`lexical: true` が要求を含む）。
            value_spans: true,
            ..Default::default()
        },
    )?;
    let ast = result.ast.as_ref().map_or(Json::Null, ToJson::to_json);
    let (value, evaluation_error) =
        match evaluate(&ast, family, result.ast.as_ref().is_some_and(operator_ast)) {
            Ok(value) => (value, false),
            Err(_) if family.is_some() => (None, true),
            Err(error) => return Err(error),
        };
    let mut mapping = Json::Null;
    let mut diagnostics = Vec::new();
    for d in &result.diagnostics {
        let kind = match d.kind {
            DiagnosticKind::Syntax => "syntax",
            DiagnosticKind::Resource => "resource",
            DiagnosticKind::Recovery => "recovery",
            DiagnosticKind::TrailingInput => "trailing_input",
            DiagnosticKind::Mapping => {
                // D-028: メッセージは生成側が組み立てたものをそのまま返す（driver は他言語の例外名を足さない）。
                mapping = d.message.as_deref().to_json();
                continue;
            }
        };
        let mut row = Json::object([
            ("kind", kind.to_json()),
            ("origin", "Mapper.diagnose".to_json()),
            ("offset", d.offset_cp.to_json()),
            ("expected", d.expected.to_json()),
            ("farthestOffset", d.farthest_cp.to_json()),
            ("farthestExpected", d.farthest_expected.to_json()),
            ("deepestRule", d.deepest_rule.to_json()),
            ("rulePath", d.rule_path.to_json()),
        ]);
        if let Some(id) = d.recovery_id {
            if let Json::Object(fields) = &mut row {
                fields.insert("length".into(), d.length_cp.to_json());
                fields.insert("severity".into(), "ERROR".to_json());
                fields.insert("origin".into(), "recovery".to_json());
                fields.insert("ruleId".into(), result.recoveries[id].rule_id.to_json());
                fields.insert("recoveryId".into(), id.to_json());
            }
        }
        diagnostics.push(row);
    }
    for d in &result.scope_diagnostics {
        let severity = severity_name(d.severity);
        diagnostics.push(Json::object([
            ("kind", severity.to_json()),
            ("origin", "ScopeStore".to_json()),
            ("offset", d.offset.to_json()),
            ("expected", Json::Array(vec![])),
            ("message", d.msg.to_json()),
            ("length", d.len.to_json()),
            ("severity", severity.to_json()),
        ]));
    }
    let mapping_failed = mapping != Json::Null;
    let mut response = Json::object([
        ("id", id),
        ("accepted", Json::Bool(prefix.ok)),
        ("allConsumed", Json::Bool(result.ok)),
        ("consumed", prefix.consumed_cp.to_json()),
        ("matched", prefix.matched_cp.to_json()),
        // D-024: allConsumed は全入力解析の成否。prefix 側の consumed からは導けないので、
        // 全入力解析のカーソルも別キーで公開する。
        ("fullConsumed", result.consumed_cp.to_json()),
        ("fullMatched", result.matched_cp.to_json()),
        ("ast", ast.clone()),
        ("diagnostics", Json::Array(diagnostics)),
        ("mappingError", mapping),
        (
            // D-025: wire のキーと要素は両 driver で同一。skippedSpan / syncPattern は
            // span・syncSpan・mode から導ける（Session.java:630-632 と同じ定義）。
            "recoveries",
            Json::Array(
                result
                    .recoveries
                    .iter()
                    .map(|r| {
                        let skipped = match r.sync_span {
                            Some(sync) if r.mode != "SKIP" => [r.span[0], r.span[1].min(sync[0])],
                            _ => r.span,
                        };
                        Json::object([
                            ("ruleId", r.rule_id.to_json()),
                            ("mode", r.mode.to_json()),
                            ("span", r.span.to_json()),
                            (
                                "syncPattern",
                                r.sync_span.map(|s| slice(input, s)).to_json(),
                            ),
                            ("skippedSpan", skipped.to_json()),
                            ("syncSpan", r.sync_span.map_or(Json::Null, |s| s.to_json())),
                            ("captureOccurrences", r.capture_occurrence_ids.to_json()),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "parsedStatus",
            if prefix.ok { "succeeded" } else { "failed" }.to_json(),
        ),
        (
            "catalogs",
            Json::Array(
                ubnfc_generated::metadata::CATALOGS
                    .iter()
                    .map(|c| {
                        Json::object([
                            ("rule", c.rule.to_json()),
                            ("context", c.context.to_json()),
                            ("captures", c.captures.to_json()),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "hints",
            Json::Array(
                result
                    .hints
                    .iter()
                    .map(|d| {
                        Json::object([
                            ("offset", d.offset_cp.to_json()),
                            ("expected", d.expected.to_json()),
                            ("deepestRule", d.deepest_rule.to_json()),
                            ("rulePath", d.rule_path.to_json()),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    let Json::Object(fields) = &mut response else {
        unreachable!()
    };
    // D-028: entry が AST 値を作らないことは mapping 失敗ではない。評価器も値を持たないので
    // `value` はキーごと出さない（Driver.java:115 と同じ規則。観測済み null と区別する）。
    let unevaluated = ast == Json::Null && result.ok && !mapping_failed;
    if let Some(value) = value {
        if !unevaluated {
            fields.insert("value".into(), value);
        }
    }
    if evaluation_error {
        fields.insert("evaluationError".into(), Json::Bool(true));
    }
    if ast != Json::Null {
        fields.insert(
            "texts".into(),
            Json::Array(
                result
                    .value_spans
                    .iter()
                    .filter_map(|v| {
                        v.text.as_ref().map(|text| {
                            Json::Array(vec![
                                v.span[0].to_json(),
                                v.span[1].to_json(),
                                text.to_json(),
                            ])
                        })
                    })
                    .collect(),
            ),
        );
        fields.insert(
            "nodes".into(),
            Json::Array(
                result
                    .value_spans
                    .iter()
                    .filter_map(|v| {
                        let node = at_path(&ast, &v.path)?;
                        if !matches!(
                            node.get("type").and_then(|v| v.string().ok()),
                            Some("Leaf" | "Other" | "OtherLeaf")
                        ) {
                            return None;
                        }
                        let fields = node.get("fields")?;
                        let text = fields.get("text").or_else(|| fields.get("value"))?;
                        text.string().ok()?;
                        Some(Json::Array(vec![
                            v.span[0].to_json(),
                            v.span[1].to_json(),
                            text.clone(),
                        ]))
                    })
                    .collect(),
            ),
        );
        let mut selected = Vec::new();
        for key in ["valueSpanField", "valueSpanListField"] {
            if let Some(name) = observations.get(key).and_then(|v| v.string().ok()) {
                let path = format!("/fields/{}", name.replace('~', "~0").replace('/', "~1"));
                selected.extend(
                    result
                        .value_spans
                        .iter()
                        .filter(|v| {
                            v.path == path
                                || v.path
                                    .strip_prefix(&(path.clone() + "/"))
                                    .is_some_and(|suffix| suffix.parse::<usize>().is_ok())
                        })
                        .map(|v| v.span.to_json()),
                );
            }
        }
        if observations.get("captureNames").is_some()
            || observations.get("valueSpanField").is_some()
        {
            fields.insert("valueSpans".into(), Json::Array(selected.clone()));
            fields.insert("retainedValueSpans".into(), Json::Array(selected));
        }
    }
    if lexical {
        // OracleCollector.java:171-174: full parse が失敗しても prefix の
        // 完了順・重複・binding と code point span/text をそのまま観測する。
        fields.insert(
            "prefixCaptures".into(),
            captures(&prefix, input, false, true),
        );
        fields.insert(
            "lexical".into(),
            Json::Array(
                result
                    .lexical
                    .iter()
                    .chain(&result.tokens)
                    .map(|l| {
                        Json::object([
                            ("occurrenceId", l.occurrence_id.to_json()),
                            ("parentOccurrenceId", l.parent_occurrence_id.to_json()),
                            ("ruleId", l.rule_id.to_json()),
                            ("exprId", l.expr_id.to_json()),
                            ("span", l.span.to_json()),
                            ("completionOrder", l.completion_order.to_json()),
                        ])
                    })
                    .collect(),
            ),
        );
        fields.insert(
            "captures".into(),
            captures(
                &result,
                input,
                observations.get("captureNames").is_some(),
                matches!(family, Some("oracle" | "evolution")),
            ),
        );
    }
    if scope {
        fields.insert("scopeDepth".into(), result.scope.depth.to_json());
        let scope_declarations = Json::Array(
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
        );
        let scope_references = Json::Array(
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
        );
        let scope_diagnostics = Json::Array(
            result
                .scope_diagnostics
                .iter()
                .map(|d| {
                    Json::object([
                        ("message", d.msg.to_json()),
                        ("offset", d.offset.to_json()),
                        ("length", d.len.to_json()),
                        ("severity", severity_name(d.severity).to_json()),
                    ])
                })
                .collect(),
        );
        fields.insert(
            "scope".into(),
            Json::object([
                ("depth", result.scope.depth.to_json()),
                // D-025: mode は文法が宣言した scope mode の集合（入力に依存しない静的な値）。
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
                                    (
                                        "action",
                                        if event.action == "use" {
                                            "reference"
                                        } else {
                                            event.action
                                        }
                                        .to_json(),
                                    ),
                                    ("mode", event.mode.to_json()),
                                    ("name", event.name.to_json()),
                                    ("offset", event.offset_cp.to_json()),
                                    ("length", event.length_cp.to_json()),
                                ])
                            })
                            .collect(),
                    ),
                ),
                ("declarations", scope_declarations.clone()),
                ("references", scope_references.clone()),
                ("diagnostics", scope_diagnostics),
            ]),
        );
        fields.insert("declarations".into(), scope_declarations);
        fields.insert("references".into(), scope_references);
    }
    Ok(response)
}
fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "ERROR",
        Severity::Warning => "WARNING",
        Severity::Info => "INFO",
        Severity::Hint => "HINT",
    }
}
/// code point の半開区間を入力から切り出す。
fn slice(input: &str, span: [usize; 2]) -> String {
    input
        .chars()
        .skip(span[0])
        .take(span[1] - span[0])
        .collect()
}
fn at_path<'a>(mut value: &'a Json, path: &str) -> Option<&'a Json> {
    for part in path.split('/').skip(1) {
        value = match value {
            Json::Array(items) => items.get(part.parse::<usize>().ok()?)?,
            _ => value.get(&part.replace("~1", "/").replace("~0", "~"))?,
        };
    }
    Some(value)
}
fn captures(
    result: &ubnfc_generated::ParseResult,
    input: &str,
    rule_order: bool,
    bindings: bool,
) -> Json {
    let mut captures: Vec<_> = result.captures.iter().collect();
    if rule_order {
        captures.sort_by_key(|c| c.rule_completion_order);
    }
    Json::Array(
        captures
            .into_iter()
            .map(|c| {
                let text: String = input
                    .chars()
                    .skip(c.span[0])
                    .take(c.span[1] - c.span[0])
                    .collect();
                let mut row = Json::object([
                    ("name", c.name.to_json()),
                    ("span", c.span.to_json()),
                    ("text", text.to_json()),
                ]);
                if bindings {
                    if let Json::Object(fields) = &mut row {
                        fields.insert(
                            "binding".into(),
                            CAPTURE_BINDINGS
                                .iter()
                                .find(|(id, _)| *id == c.site_id)
                                .map_or(Json::Null, |(_, binding)| binding.to_json()),
                        );
                    }
                }
                row
            })
            .collect(),
    )
}
fn values(value: &Json) -> &[Json] {
    match value {
        Json::Null => &[],
        Json::Array(items) => items,
        _ => std::slice::from_ref(value),
    }
}
fn tagged(value: &Json) -> Result<String, &'static str> {
    if let Json::String(text) = value {
        return Ok(format!("T:{text}"));
    }
    let fields = value.get("fields").ok_or("unsupported semantic value")?;
    let text = fields
        .get("text")
        .or_else(|| fields.get("value"))
        .ok_or("missing semantic text")?;
    match value.get("type").and_then(|v| v.string().ok()) {
        Some("Leaf") => Ok(format!("L:{}", text.string()?)),
        Some("Other" | "OtherLeaf") => Ok(format!("O:{}", text.string()?)),
        Some("Value" | "Shared") => tagged(text),
        _ => Err("unsupported semantic value"),
    }
}
fn tags(value: &Json) -> Result<String, &'static str> {
    values(value)
        .iter()
        .map(tagged)
        .collect::<Result<Vec<_>, _>>()
        .map(|v| v.join(","))
}
fn evaluate(
    ast: &Json,
    family: Option<&str>,
    operator: bool,
) -> Result<Option<Json>, &'static str> {
    if *ast == Json::Null {
        return Ok(Some(Json::Null));
    }
    let Some(fields) = ast.get("fields") else {
        return Ok(None);
    };
    let field = |name| fields.get(name).unwrap_or(&Json::Null);
    if family == Some("semantic-cardinality") && fields.get("values").is_some() {
        return Ok(Some(tags(field("values"))?.to_json()));
    }
    if family == Some("mixed-values") {
        if fields.get("head").is_some()
            && fields.get("maybe").is_some()
            && fields.get("items").is_some()
        {
            return Ok(Some(
                format!(
                    "{}|{}|{}",
                    tagged(field("head"))?,
                    if *field("maybe") == Json::Null {
                        "-".into()
                    } else {
                        tagged(field("maybe"))?
                    },
                    tags(field("items"))?
                )
                .to_json(),
            ));
        }
        if fields.get("value").is_some() {
            return Ok(Some(tagged(field("value"))?.to_json()));
        }
    }
    if family == Some("cardinality") && fields.get("flags").is_some() {
        let mut total = if *field("head") == Json::Null {
            0.0
        } else {
            numeric(field("head"))?.ok_or("invalid head")?
        };
        for item in values(field("values")) {
            total += numeric(item)?.ok_or("invalid item")?;
        }
        total += if *field("tail") == Json::Null {
            0.0
        } else {
            1.0
        };
        total += values(field("flags")).len() as f64;
        return Ok(finite(total)?.map(|n| Json::Number(format!("{n:?}"))));
    }
    if operator
        || matches!(
            ast.get("type").and_then(|v| v.string().ok()),
            Some("Number" | "Literal" | "Binary" | "Power" | "Negation" | "Conditional")
        )
        || matches!(
            family,
            Some("right-associative" | "associative" | "evolution")
        )
    {
        return Ok(numeric(ast)?.map(|n| Json::Number(format!("{n:?}"))));
    }
    Ok(None)
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
    if value.get("type").and_then(|v| v.string().ok()) == Some("Negation") {
        return fields
            .get("value")
            .map_or(Ok(None), numeric)
            .map(|v| v.map(|v| -v));
    }
    if value.get("type").and_then(|v| v.string().ok()) == Some("Conditional") {
        let Some(condition) = fields.get("condition").map_or(Ok(None), numeric)? else {
            return Ok(None);
        };
        return fields
            .get(if condition != 0.0 {
                "thenExpr"
            } else {
                "elseExpr"
            })
            .map_or(Ok(None), numeric);
    }
    if fields.len() == 1 {
        return fields.get("value").map_or(Ok(None), numeric);
    }
    let (Some(left), Some(ops), Some(rights)) =
        (fields.get("left"), fields.get("op"), fields.get("right"))
    else {
        return Ok(None);
    };
    let (ops, rights) = (values(ops), values(rights));
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
    #[cfg(feature = "scanners")]
    scanners::Registry::new(EXTERN_CLASSES).map_err(io::Error::other)?;
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
