use serde::Deserialize;
use serde_json::{json, Value};
use std::{path::PathBuf, sync::OnceLock};
use ubnfc_target_rust::{canonical, operators, power, Ast, AstKind, ParseOptions, ParseResult};

#[derive(Deserialize)]
struct Row {
    input: String,
    value: Option<f64>,
    #[serde(default)]
    modes: Vec<String>,
}

struct Fixture {
    rows: Vec<Row>,
    golden: Value,
}
fn load(family: &str, golden: &str) -> Fixture {
    let root = std::env::var_os("UNLAXER_PARSER_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../unlaxer-parser")
        });
    let path = root
        .join("unlaxer-dsl/src/test/resources")
        .join(family)
        .join("corpus.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "{}: {error}; UNLAXER_PARSER_ROOT で指定可能",
            path.display()
        )
    });
    Fixture {
        rows: serde_json::from_str(&text).unwrap(),
        golden: serde_json::from_str(golden).unwrap(),
    }
}
fn operators_fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| load("associative", include_str!("golden/operators.json")))
}
fn power_fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| load("right-associative", include_str!("golden/power.json")))
}

fn snapshot(result: &ParseResult) -> Value {
    json!({ "ok": result.ok, "consumed_cp": result.consumed_cp, "matched_cp": result.matched_cp,
        "ast": result.ast.as_ref().map(canonical::value) })
}

fn verify(result: &ParseResult, row: &Row, accepted: bool) {
    assert_eq!(result.ok, accepted, "{}", row.input);
    assert_eq!(result.ast.is_some(), accepted, "{}", row.input);
    if accepted {
        assert_eq!(result.consumed_cp, row.input.chars().count());
        assert_eq!(result.matched_cp, result.consumed_cp);
        let ast = result.ast.as_ref().unwrap();
        assert_eq!(ast.evaluate(), row.value.unwrap(), "{}", row.input);
        assert_eq!(ast.span, [0, row.input.chars().count()]);
        verify_spans(ast, result, ast.span);
        for (order, capture) in result.captures.iter().enumerate() {
            assert_eq!(capture.completion_order, order);
            assert!(
                capture.span[0] <= capture.span[1] && capture.span[1] <= row.input.chars().count()
            );
        }
        assert_eq!(
            serde_json::from_str::<Value>(&canonical::json(ast)).unwrap(),
            canonical::value(ast)
        );
    } else {
        assert!(result.node_spans.is_empty());
        assert!(result.captures.is_empty());
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.statistics.mapped_numbers, 0);
    }
}

fn verify_spans(ast: &Ast, result: &ParseResult, parent: [usize; 2]) {
    assert!(parent[0] <= ast.span[0] && ast.span[0] <= ast.span[1] && ast.span[1] <= parent[1]);
    if let Some(id) = ast.node_id {
        assert_eq!(result.node_spans[id].span, ast.span);
        assert_eq!(result.node_spans[id].node_type, ast.type_name());
    }
    match &ast.kind {
        AstKind::Binary { left, op, right } | AstKind::Power { left, op, right } => {
            assert_eq!(op.len(), right.len());
            if matches!(ast.kind, AstKind::Power { .. }) {
                assert!(right.len() <= 1);
            }
            verify_spans(left, result, ast.span);
            for right in right {
                verify_spans(right, result, ast.span);
            }
        }
        _ => {}
    }
}

fn operators_case(index: usize) {
    let fixture = operators_fixture();
    assert_eq!(fixture.rows.len(), 24);
    let row = &fixture.rows[index];
    let result = operators::parse(&row.input);
    verify(&result, row, row.value.is_some());
    assert_eq!(fixture.golden[index]["input"], row.input);
    assert_eq!(
        snapshot(&result),
        fixture.golden[index]["result"],
        "{}",
        row.input
    );
    let recognize = operators::parse_with_options(
        &row.input,
        ParseOptions {
            build_ast: false,
            require_eof: true,
        },
    );
    verify_recognition(&result, &recognize);
}

fn power_case(index: usize) {
    let fixture = power_fixture();
    assert_eq!(fixture.rows.len(), 28);
    let row = &fixture.rows[index];
    for mode in power::Mode::ALL {
        let accepted = row.value.is_some()
            && (row.modes.is_empty() || row.modes.iter().any(|name| name == mode.name()));
        let result = power::parse_in_mode(&row.input, mode);
        verify(&result, row, accepted);
        assert_eq!(fixture.golden[mode.name()][index]["input"], row.input);
        assert_eq!(
            snapshot(&result),
            fixture.golden[mode.name()][index]["result"],
            "{} {}",
            mode.name(),
            row.input
        );
        let recognize = power::parse_with_options(
            &row.input,
            mode,
            ParseOptions {
                build_ast: false,
                require_eof: true,
            },
        );
        verify_recognition(&result, &recognize);
    }
}

fn verify_recognition(mapped: &ParseResult, recognize: &ParseResult) {
    assert_eq!(
        (mapped.ok, mapped.consumed_cp, mapped.matched_cp),
        (recognize.ok, recognize.consumed_cp, recognize.matched_cp)
    );
    assert_eq!(mapped.diagnostics, recognize.diagnostics);
    assert!(
        recognize.ast.is_none() && recognize.node_spans.is_empty() && recognize.captures.is_empty()
    );
    assert_eq!(recognize.statistics.recipes, 0);
    assert_eq!(recognize.statistics.mapped_numbers, 0);
}

macro_rules! cases {
    ($run:ident; $($name:ident: $index:literal),* $(,)?) => { $(#[test] fn $name() { $run($index); })* };
}
cases!(operators_case;
    operators_00:0, operators_01:1, operators_02:2, operators_03:3, operators_04:4, operators_05:5,
    operators_06:6, operators_07:7, operators_08:8, operators_09:9, operators_10:10, operators_11:11,
    operators_12:12, operators_13:13, operators_14:14, operators_15:15, operators_16:16, operators_17:17,
    operators_18:18, operators_19:19, operators_20:20, operators_21:21, operators_22:22, operators_23:23);
cases!(power_case;
    power_00:0, power_01:1, power_02:2, power_03:3, power_04:4, power_05:5, power_06:6,
    power_07:7, power_08:8, power_09:9, power_10:10, power_11:11, power_12:12, power_13:13,
    power_14:14, power_15:15, power_16:16, power_17:17, power_18:18, power_19:19, power_20:20,
    power_21:21, power_22:22, power_23:23, power_24:24, power_25:25, power_26:26, power_27:27);
