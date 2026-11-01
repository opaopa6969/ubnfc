use std::path::{Path, PathBuf};

use serde_json::Value;
use ubnfc_front::syntax::ast::*;
use ubnfc_front::syntax::{parse_file, SourceMap};

fn fixture_root(variable: &str, default: &str) -> PathBuf {
    let path = std::env::var_os(variable).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join(default),
        PathBuf::from,
    );
    assert!(
        path.is_dir(),
        "fixture ディレクトリがありません: {} ({variable})",
        path.display()
    );
    path
}

fn unlaxer_root() -> PathBuf {
    fixture_root("UNLAXER_PARSER_DIR", "../../../unlaxer-parser")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn parse_fixture(source: &str, path: &Path) -> UbnfFile {
    let file = parse_file(source).unwrap_or_else(|error| {
        let (line, col) = SourceMap::new(source).line_col(error.span.start);
        panic!("{}:{line}:{col}: {error}", path.display());
    });
    // 全 fixture の全ノードについて UTF-8 境界と親の span への包含を確認する。
    check_spans(&serde_json::to_value(&file).unwrap(), source, file.span);
    file
}

fn check_spans(value: &Value, source: &str, parent: Span) {
    match value {
        Value::Object(fields) => {
            let span = if let Some(span) = fields.get("span") {
                let span = Span {
                    start: span["start"].as_u64().unwrap() as usize,
                    end: span["end"].as_u64().unwrap() as usize,
                };
                assert!(
                    span.start <= span.end && parent.start <= span.start && span.end <= parent.end,
                    "親 {parent:?} の範囲外: {span:?}"
                );
                assert!(
                    source.get(span.start..span.end).is_some(),
                    "UTF-8 境界でない: {span:?}"
                );
                span
            } else {
                parent
            };
            for (key, child) in fields {
                if key != "span" {
                    check_spans(child, source, span);
                }
            }
        }
        Value::Array(children) => {
            for child in children {
                check_spans(child, source, parent);
            }
        }
        _ => {}
    }
}

fn collect_ubnf(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            if entry.file_name() != "target" {
                collect_ubnf(&path, paths);
            }
        } else if path.extension().is_some_and(|ext| ext == "ubnf") {
            paths.push(path);
        }
    }
}

#[test]
fn all_unlaxer_ubnf_files_parse() {
    let root = unlaxer_root();
    let mut paths = Vec::new();
    collect_ubnf(&root, &mut paths);
    paths.sort();
    // 依頼時の表記は25だが、実際の find 結果は24。既知ファイルの欠落も失敗させ、
    // 将来追加された .ubnf も再帰探索で対象にする。.ubnf.txt と target は対象外。
    for required in [
        "story/code/calc-var.ubnf",
        "story/code/calc.ubnf",
        "story/code/calc-leftrec.ubnf",
        "rust/unlaxer-ubnf/tests/fixtures/positive/boundaries.ubnf",
        "rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf",
        "rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf",
        "rust/unlaxer-ubnf/tests/fixtures/positive/elements.ubnf",
        "rust/unlaxer-ubnf/tests/fixtures/known/eval-default.ubnf",
        "rust/unlaxer-ubnf/tests/fixtures/known/keyword-boundary.ubnf",
        "rust/unlaxer-ubnf/tests/fixtures/known/namespace.ubnf",
        "rust/unlaxer-ubnf/tests/fixtures/known/common-field-space.ubnf",
        "unlaxer-dsl/src/test/resources/right-associative/Power.ubnf",
        "unlaxer-dsl/src/test/resources/mixed-values/Delimited.ubnf",
        "unlaxer-dsl/src/test/resources/mixed-values/Mixed.ubnf",
        "unlaxer-dsl/src/test/resources/mixed-values/Shared.ubnf",
        "unlaxer-dsl/src/test/resources/evolution/0/Evolution.ubnf",
        "unlaxer-dsl/src/test/resources/evolution/1/Evolution.ubnf",
        "unlaxer-dsl/src/test/resources/evolution/2/Evolution.ubnf",
        "unlaxer-dsl/src/test/resources/evolution/3/Evolution.ubnf",
        "unlaxer-dsl/src/test/resources/associative/Operators.ubnf",
        "unlaxer-dsl/src/test/resources/cardinality/Cardinality.ubnf",
        "unlaxer-dsl/grammar/ubnf.ubnf",
        "unlaxer-dsl/tinycalc-vscode/grammar/tinycalc.ubnf",
        "unlaxer-dsl/examples/tinycalc.ubnf",
    ] {
        assert!(
            paths.contains(&root.join(required)),
            "fixture がありません: {required}"
        );
    }
    for path in &paths {
        if path.ends_with("rust/unlaxer-ubnf/tests/fixtures/positive/adapter.ubnf")
            || (path.starts_with(root.join("spec-corpus/token-adapters"))
                && read(path).lines().any(|line| {
                    line.trim_start().starts_with("token ") && line.contains("ADAPTER(")
                }))
        {
            // core's versioned ADAPTER registry is distinct from ubnfc's extern
            // registry and is deliberately not silently reinterpreted here.
            // The spelling parses (Classic is the canonical spelling, D-084) and resolve
            // rejects it at its position (E-TOKEN-ADAPTER-UNSUPPORTED, D-085).
            let file = parse_fixture(&read(path), path);
            assert!(
                file.grammars
                    .iter()
                    .flat_map(|g| &g.tokens)
                    .any(|t| matches!(t.value, TokenValue::Adapter { .. })),
                "{}",
                path.display()
            );
            continue;
        }
        if path.ends_with("rust/unlaxer-generator/tests/fixtures/portability/syntax.ubnf") {
            assert!(
                parse_file(&read(path)).is_err(),
                "既知の構文エラー: {}",
                path.display()
            );
            continue;
        }
        parse_fixture(&read(path), path);
    }
    eprintln!("unlaxer-parser: {} 個の .ubnf が成功", paths.len());
}

#[test]
fn self_description_has_seven_tokens_and_75_rules() {
    let path = unlaxer_root().join("unlaxer-dsl/grammar/ubnf.ubnf");
    let source = read(&path);
    let file = parse_fixture(&source, &path);
    assert_eq!(file.grammars.len(), 1);
    let grammar = &file.grammars[0];
    assert_eq!(grammar.name, "UBNF");
    // 2026-10 の更新（REGEX 廃止、CAPTURE( などの関数形式、NOT_OWNED_CALL / XID_IDENTIFIER、
    // @lexicalContext などの注釈）で 6 token / 66 規則から 7 token / 75 規則になった。
    assert_eq!(grammar.tokens.len(), 7);
    assert_eq!(grammar.rules.len(), 75);
    // 行頭の独立した宣言を数える。文字列内の '::=' は数えない。
    let declarations = source
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with(|c: char| c.is_ascii_alphabetic())
                && !trimmed.starts_with("token ")
                && trimmed.contains("::=")
        })
        .count();
    assert_eq!(grammar.rules.len(), declarations);
    assert_eq!(grammar.rules.first().unwrap().name, "UBNFFile");
    assert_eq!(grammar.rules.last().unwrap().name, "RuleBody");
}

#[test]
fn all_token_variants_import_settings_and_escapes_are_preserved() {
    let path = unlaxer_root().join("rust/unlaxer-ubnf/tests/fixtures/positive/all-tokens.ubnf");
    let source = read(&path);
    let file = parse_fixture(&source, &path);
    assert_eq!(file.grammars.len(), 2);
    let grammar = &file.grammars[0];
    assert_eq!(grammar.imports[0].alias, "json");
    assert_eq!(grammar.imports[0].path, "json.ubnf");
    // 2026-10 に先頭へ `@ubnf: v2` が入った。設定の位置ではなく key で引く。
    let setting = |name: &str| {
        grammar
            .settings
            .iter()
            .find(|s| matches!(s, GlobalSetting::String { key, .. } | GlobalSetting::Block { key, .. } if key == name))
            .unwrap_or_else(|| panic!("@{name} がありません"))
    };
    assert!(
        matches!(setting("package"), GlobalSetting::String { key, value, .. }
        if key == "package" && value == "org.example.tokens")
    );
    assert!(
        matches!(setting("comment"), GlobalSetting::Block { key, entries, .. }
        if key == "comment" && entries.iter().map(|e| (e.key.as_str(), e.value.as_str())).collect::<Vec<_>>()
            == [("line", "//"), ("blockOpen", "/*"), ("blockClose", "*/")])
    );
    let kinds: Vec<_> = grammar.tokens.iter().map(|token| &token.value).collect();
    assert!(matches!(kinds.as_slice(), [
        TokenValue::ClassRef { dotted, .. }, TokenValue::Until { terminator, .. },
        TokenValue::Negation { chars, .. }, TokenValue::Lookahead { pattern: lookahead, .. },
        TokenValue::NegativeLookahead { pattern: negative, .. }, TokenValue::Any { .. },
        TokenValue::Eof { .. }, TokenValue::Empty { .. }, TokenValue::CharRange { min, max, .. },
        // REGEX は Classic で廃止され（unlaxer-parser #612）、fixture も宣言的 token になった。
        TokenValue::CaseInsensitive { word, .. }, TokenValue::Declarative { .. },
    ] if dotted == "org.unlaxer.parser.elementary.NumberParser" && terminator == "*/"
        && chars == "😀\n\t" && lookahead == ":" && negative == "//" && min == "a" && max == "z"
        && word == "select"));
    let second = &file.grammars[1];
    assert_eq!(second.name, "Second");
    let Atom::Literal { value, raw, span } =
        &second.rules[0].body.alternatives()[0].elements[0].atom
    else {
        panic!("Second は literal のはず");
    };
    // 既存 rust/unlaxer-ubnf/src/lexer.rs は未知の \q を2文字のまま保持する。
    // fixture は \\ の直後に \q があるため、結果は backslash 2個 + q。
    // 同 repo tests/frontend.rs の期待値とも一致する。
    assert_eq!(value, "😀\n\t\r'\\\\q");
    assert_eq!(raw, &source[span.start..span.end]);
    assert_eq!(raw, r"'😀\n\t\r\'\\\q'");
    assert!(
        matches!(&grammar.rules[0].body.alternatives()[0].elements[0].atom,
        Atom::Ref { namespace: Some(namespace), name, .. } if namespace == "json" && name == "Value")
    );
}

#[test]
fn annotations_fixture_is_structured_without_inventing_defaults() {
    let path = unlaxer_root().join("rust/unlaxer-ubnf/tests/fixtures/positive/annotations.ubnf");
    let file = parse_fixture(&read(&path), &path);
    let rules = &file.grammars[0].rules;
    assert_eq!(rules.len(), 4);
    assert!(matches!(rules[0].annotations.as_slice(), [
        Annotation::Root { .. }, Annotation::Mapping { class_name, params, .. },
        Annotation::Eval { entries, .. }, Annotation::Whitespace { style: Some(style), .. },
        Annotation::Interleave { profile, .. }, Annotation::Backref { name, .. },
        Annotation::ScopeTree { mode, .. }, Annotation::Declares { symbol, description: Some(description), .. },
        Annotation::Catalog { context, .. }, Annotation::LeftAssoc { .. },
        Annotation::Precedence { level: 10, .. }, Annotation::Doc { text, .. },
        Annotation::Recovery { mode: None, sync: Some(sync), .. },
        Annotation::CommonField { fields, .. }, Annotation::Simple { name: custom, .. },
    ] if class_name == "Node.Item" && params == &["left", "right"]
        && entries.iter().map(|e| (e.key.as_str(), e.value.as_str())).collect::<Vec<_>>() == [
            ("kind", "binary"), ("strategy", "manual"), ("operation", "+"), ("result", "number")]
        && style == "javaStyle" && profile == "comments" && name == "value" && mode == "lexical"
        && symbol == "left" && description == "right" && context == "variables" && text == "文法😀"
        && sync == ";, }" && fields == &["left", "right"] && custom == "custom"));
    assert!(matches!(rules[1].annotations.as_slice(), [
        Annotation::Whitespace { style: None, .. }, Annotation::RightAssoc { .. },
        Annotation::Recovery { mode: Some(mode), sync: None, .. }, Annotation::Skip { .. },
    ] if mode == "auto"));
    assert!(matches!(rules[2].annotations.as_slice(), [
        Annotation::Recovery { mode: Some(mode), sync: None, .. }, Annotation::Enum { .. },
    ] if mode == "skip"));
    assert!(matches!(rules[3].annotations.as_slice(), [
        Annotation::Declares { symbol, description: None, .. }, Annotation::Eval { entries, .. },
    ] if symbol == "name" && entries.len() == 1 && entries[0].key == "kind" && entries[0].value == "literal"));
}

#[test]
fn tinyexpression_p4_has_130_rules() {
    let path = fixture_root("TINYEXPRESSION_DIR", "../../../tinyexpression")
        .join("tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf");
    let file = parse_fixture(&read(&path), &path);
    assert_eq!(file.grammars.len(), 1);
    assert_eq!(file.grammars[0].rules.len(), 130);
}
