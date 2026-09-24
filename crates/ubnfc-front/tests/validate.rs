use serde_json::{json, Value};
use ubnfc_front::resolve::resolve;
use ubnfc_front::syntax::parse_file;
use ubnfc_front::validate::{validate, ValidationIssue};

fn issue_json(issue: &ValidationIssue) -> Value {
    json!({ "grammar": issue.grammar, "rule": issue.rule, "code": issue.code,
        "message": issue.message, "hint": issue.hint, "severity": issue.severity(), "category": issue.category() })
}

fn sorted(mut issues: Vec<Value>) -> Vec<Value> {
    issues.sort_by_key(|v| {
        ["grammar", "rule", "code", "message"].map(|k| v[k].as_str().unwrap_or("").to_owned())
    });
    issues
}

fn check(source: &str) -> Vec<ValidationIssue> {
    let ast = parse_file(source).unwrap();
    validate(&resolve(&[("test.ubnf", ast)], &|_| panic!("unexpected import")).unwrap())
}

/// validation-java.json は現在の Java GrammarValidator を一時領域でコンパイルし、
/// GrammarValidatorTest の49文法 + 条件補足34文法から採取した独立した期待値。
/// Java 実行やクラスパスは cargo test の依存にはしない。
/// upstream: ed4178e673f93e719b62a7bd181f3a219abf8ed1
/// GrammarValidator.java SHA-256:
/// 6d46368872ba18bbc071b6b8254a8b704beebbce22c657ba6f943f55f5507338
#[test]
fn java_validator_cases_match_full_diagnostics() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/validation-java.json")).unwrap();
    assert_eq!(cases.len(), 83);
    for case in cases {
        let name = case["case"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let ast = parse_file(source).unwrap_or_else(|e| panic!("{name}: {e}"));
        let result = resolve(&[("test.ubnf", ast)], &|_| panic!("unexpected import"));
        let mut expected = case["issues"].as_array().unwrap().clone();
        // OQ-DSL-001: 既知の NumberParser 単純名は Builtin として解決する。
        // この差分の対象を1ケースに固定し、一般的な警告の除去はしない。
        if name == "testTokenUnresolvedHintSuggestsFullyQualifiedName:0" {
            assert_eq!(expected.len(), 1);
            assert_eq!(expected[0]["code"], "W-TOKEN-UNRESOLVED");
            expected.clear();
        }
        let actual: Vec<_> = match result {
            Ok(grammar) => validate(&grammar).iter().map(issue_json).collect(),
            Err(errors) => {
                // 未定義参照は validate より前に拒否する API 契約。
                assert!(
                    errors.iter().all(|e| e.code == "E-RULE-UNDEFINED"),
                    "{name}: {errors:?}"
                );
                expected.retain(|e| e["code"] == "E-RULE-UNDEFINED");
                errors
                    .into_iter()
                    .map(|e| {
                        issue_json(&ValidationIssue {
                            grammar: e.grammar,
                            rule: e.rule,
                            code: e.code,
                            message: e.message,
                            hint: e.hint,
                        })
                    })
                    .collect()
            }
        };
        let expected = sorted(expected);
        assert_eq!(actual, expected, "{name}\n{source}");
    }
}

#[test]
fn issues_sort_by_grammar_rule_code_and_message() {
    let source = "grammar Z { @mapping(X, params=[z,a,z]) Start ::= 'a' @extra; } grammar A { @root @declares(symbol=b) @declares(symbol=a) Start ::= 'a'; }";
    let issues = check(source);
    let actual: Vec<_> = issues.iter().map(issue_json).collect();
    assert_eq!(actual, sorted(actual.clone()));
    assert_eq!(issues[0].grammar, "A");
    assert_eq!(issues.iter().find(|i| i.grammar == "Z").unwrap().rule, None);
    assert_eq!(
        issues[0].format(),
        format!(
            "{} [code: {}] [hint: {}]",
            issues[0].message, issues[0].code, issues[0].hint
        )
    );
    assert_eq!(issues, check(source));
}

#[test]
fn eval_auto_on_explicit_and_implicit_sums_is_rejected() {
    for source in [
        "grammar G { @root @mapping(Node) @eval(kind='literal') Start ::= A | B; @mapping(A) A ::= 'a'; @mapping(B) B ::= 'b'; }",
        "grammar G { @root @mapping(Node) @eval(kind='unknown', strategy='auto') Start ::= 'x'; @mapping(Node.A) A ::= 'a'; }",
    ] {
        let issues = check(source);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "E-EVAL-SUM-TYPE");
        assert_eq!(issues[0].message, "Automatic @eval requires a concrete mapping, not sum Node; annotate its variants instead");
        assert_eq!(issues[0].severity(), "ERROR");
    }
}

#[test]
fn eval_manual_concrete_enum_and_skip_are_accepted() {
    for source in [
        "grammar G { @root @mapping(Node) @eval(kind='literal', strategy='manual') Start ::= A | B; @mapping(A) A ::= 'a'; @mapping(B) B ::= 'b'; }",
        "grammar G { @root @mapping(Node) @eval(kind='literal') Start ::= 'x'; }",
        "grammar G { @root @mapping(Node) @eval(kind='unknown') Start ::= 'x'; }",
        "grammar G { @root @enum @mapping(Node) @eval(kind='literal') Start ::= 'x'; @mapping(Node.A) A ::= 'a'; }",
        "grammar G { @root @skip @mapping(Node) @eval(kind='literal') Start ::= A | B; @mapping(A) A ::= 'a'; @mapping(B) B ::= 'b'; }",
        // UBNFMapper の positional strategy。2番目の値が manual なら自動拒否しない。
        "grammar G { @root @mapping(Node) @eval(kind='literal', custom='manual') Start ::= A | B; @mapping(A) A ::= 'a'; @mapping(B) B ::= 'b'; }",
    ] { assert!(check(source).is_empty(), "{source}"); }
}

#[test]
fn assoc_mapping_parameters_have_java_messages() {
    let issues = check("grammar G { @root @leftAssoc @precedence(level=1) @mapping(Node, params=[left]) Start ::= 'a' @left { '+' @op 'b' @right }; }");
    let params: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "E-ASSOC-MAPPING-PARAM")
        .collect();
    assert_eq!(params.len(), 2);
    assert_eq!(
        params[0].message,
        "rule Start uses @leftAssoc but @mapping(Node) params does not contain 'op'"
    );
    assert_eq!(
        params[1].message,
        "rule Start uses @leftAssoc but @mapping(Node) params does not contain 'right'"
    );
    assert_eq!(params[0].hint, "Include left/op/right in @mapping params.");
}

#[test]
fn negative_precedence_is_rejected_by_syntax_before_validation() {
    // Annotation::Precedence.level は u64。構文層を変更しないため、Java AST で
    // 手動構築できる負の level に E-PRECEDENCE-NEGATIVE を付ける API は未提供。
    assert!(parse_file("grammar G { @precedence(level=-1) Start ::= 'a'; }").is_err());
}

/// 2026-09-23 のクロスチェック（`docs/reports/2026-09-23-parser-feature-correctness-comparison.md`
/// §3.1 ①）が見つけた「ubnfc だけが受理する不正宣言」7 件の回帰。unlaxer-ubnf と
/// Java bootstrap はいずれも拒否する。正本は
/// `unlaxer-parser/unlaxer-dsl/specs/ubnf-syntax.md`（CHAR_RANGE の境界）と
/// `rust/unlaxer-ubnf/src/parser.rs`（整数は Java int の範囲、recovery mode は 3 種）。
#[test]
fn crosscheck_malformed_declarations_are_rejected() {
    for (source, code) in [
        // 以前はここで lower/tokens.rs が unwrap して panic していた。
        (
            "grammar G { token T=CHAR_RANGE('','a') R ::= T; }",
            "E-TOKEN-CHAR-RANGE-BOUNDARY",
        ),
        (
            "grammar G { token T=CHAR_RANGE('a','') R ::= T; }",
            "E-TOKEN-CHAR-RANGE-BOUNDARY",
        ),
        // 先頭 1 文字を黙って採らない。
        (
            "grammar G { token T=CHAR_RANGE('ab','z') R ::= T; }",
            "E-TOKEN-CHAR-RANGE-BOUNDARY",
        ),
        (
            "grammar G { token T=CHAR_RANGE('\u{1f600}','\u{1f607}') R ::= T; }",
            "E-TOKEN-CHAR-RANGE-SUPPLEMENTARY",
        ),
        (
            "grammar G { token T=CHAR_RANGE('z','a') R ::= T; }",
            "E-TOKEN-CHAR-RANGE-ORDER",
        ),
        (
            "grammar G { @root R ::= 'x'{2147483648}; }",
            "E-BOUNDED-OVERFLOW",
        ),
        (
            "grammar G { @root R ::= 'x'{2147483648,}; }",
            "E-BOUNDED-OVERFLOW",
        ),
        (
            "grammar G { @root @leftAssoc @precedence(level=2147483648) @mapping(N, params=[left,op,right]) R ::= 'a' @left { '+' @op 'b' @right }; }",
            "E-PRECEDENCE-OVERFLOW",
        ),
        (
            "grammar G { @root @recovery(magic) R ::= 'x'; }",
            "E-ANNOTATION-RECOVERY-MODE",
        ),
    ] {
        let issues = check(source);
        let issue = issues
            .iter()
            .find(|i| i.code == code)
            .unwrap_or_else(|| panic!("{code} が出ません: {source}\n{issues:?}"));
        assert_eq!(issue.severity(), "ERROR", "{source}");
        // ERROR がある限り lower へは進めない（pipeline と lower の両方が門を持つ）。
        let ast = parse_file(source).unwrap();
        let resolved = resolve(&[("test.ubnf", ast)], &|_| panic!("unexpected import")).unwrap();
        let errors = ubnfc_front::lower::lower(&resolved, &[]).unwrap_err();
        assert!(
            errors.iter().any(|e| e.to_string().starts_with(code)),
            "{source}: {errors:?}"
        );
    }
}

/// 境界が正しい CHAR_RANGE と、3 種の recovery mode、32 bit に収まる量指定子は通る。
#[test]
fn crosscheck_wellformed_declarations_stay_accepted() {
    for source in [
        "grammar G { token T=CHAR_RANGE('a','z') @root R ::= T; }",
        "grammar G { token T=CHAR_RANGE('a','a') @root R ::= T; }",
        // エスケープ展開後に 1 文字であればよい（`\t` < `\n` なので順序も正しい）。
        "grammar G { token T=CHAR_RANGE('\\t','\\n') @root R ::= T; }",
        "grammar G { @root R ::= 'x'{2147483647}; }",
        "grammar G { @root @recovery(sync) R ::= 'x'; }",
        "grammar G { @root @recovery(AUTO) R ::= 'x'; }",
        "grammar G { @root @recovery(skip) R ::= 'x'; }",
        "grammar G { @root @recovery(sync=';') R ::= 'x'; }",
    ] {
        assert!(
            check(source).iter().all(|i| i.severity() != "ERROR"),
            "{source}: {:?}",
            check(source)
        );
    }
}

/// D-063: v2 の code point クラス。`\uXXXX` / `\u{X…}` は v2 でのみエスケープで、
/// v1 では `E-TOKEN-ESCAPE-V2-ONLY` として拒否する（v1 で黙って別の意味に読まない）。
/// 補助面の境界は v2 でのみ許す（`E-TOKEN-CHAR-RANGE-SUPPLEMENTARY` は v1 専用になった）。
#[test]
fn v2_code_point_classes_are_gated_and_validated() {
    for (source, code) in [
        // v1 の門。CHAR_RANGE も NEGATION も同じ理由で止める。
        (
            "grammar G { token T=CHAR_RANGE('\\u0041','\\u005A') @root R ::= T; }",
            "E-TOKEN-ESCAPE-V2-ONLY",
        ),
        (
            "grammar G { token T=NEGATION('\\u0041') @root R ::= T; }",
            "E-TOKEN-ESCAPE-V2-ONLY",
        ),
        // v2 でも綴りの誤りは拒否する。
        (
            "grammar G { @ubnf: 2 token T=CHAR_RANGE('\\u00','z') @root R ::= T; }",
            "E-TOKEN-ESCAPE-UNICODE",
        ),
        (
            "grammar G { @ubnf: 2 token T=CHAR_RANGE('a','\\u{110000}') @root R ::= T; }",
            "E-TOKEN-ESCAPE-UNICODE",
        ),
        (
            "grammar G { @ubnf: 2 token T=NEGATION('\\u{}') @root R ::= T; }",
            "E-TOKEN-ESCAPE-UNICODE",
        ),
        // 単独 surrogate は code point ではない。
        (
            "grammar G { @ubnf: 2 token T=CHAR_RANGE('\\uD800','\\uDFFF') @root R ::= T; }",
            "E-TOKEN-ESCAPE-SURROGATE",
        ),
        // D-050 の 3 条件は v2 でもそのまま。
        (
            "grammar G { @ubnf: 2 token T=CHAR_RANGE('\\u{1F607}','\\u{1F600}') @root R ::= T; }",
            "E-TOKEN-CHAR-RANGE-ORDER",
        ),
        (
            "grammar G { @ubnf: 2 token T=CHAR_RANGE('\\u0041\\u0042','z') @root R ::= T; }",
            "E-TOKEN-CHAR-RANGE-BOUNDARY",
        ),
        // 補助面は v1 では従来どおり拒否。
        (
            "grammar G { token T=CHAR_RANGE('\u{1f600}','\u{1f607}') @root R ::= T; }",
            "E-TOKEN-CHAR-RANGE-SUPPLEMENTARY",
        ),
    ] {
        let issues = check(source);
        let issue = issues
            .iter()
            .find(|i| i.code == code)
            .unwrap_or_else(|| panic!("{code} が出ません: {source}\n{issues:?}"));
        assert_eq!(issue.severity(), "ERROR", "{source}");
        assert_eq!(issue.category(), "TOKEN", "{source}");
    }
}

/// v2 で受理される新しい書き方。生の補助面文字も、エスケープも通る。
#[test]
fn v2_code_point_classes_accept_escapes_and_supplementary_boundaries() {
    for source in [
        "grammar G { @ubnf: 2 token T=CHAR_RANGE('\\u0020','\\u0021') @root R ::= T; }",
        "grammar G { @ubnf: 2 token T=CHAR_RANGE('\\u{5D}','\\u{D7FF}') @root R ::= T; }",
        "grammar G { @ubnf: 2 token T=CHAR_RANGE('\\u{E000}','\\u{10FFFF}') @root R ::= T; }",
        "grammar G { @ubnf: 2 token T=CHAR_RANGE('\u{1f600}','\u{1f607}') @root R ::= T; }",
        "grammar G { @ubnf: 2 token T=NEGATION('\"\\u0000\\u{1F600}') @root R ::= T; }",
        // `\u` を書かない v2 文法は v1 と同じ扱い（未知のエスケープも v1 のまま）。
        "grammar G { @ubnf: 2 token T=NEGATION('a\\qb') @root R ::= T; }",
    ] {
        assert!(
            check(source).iter().all(|i| i.severity() != "ERROR"),
            "{source}: {:?}",
            check(source)
        );
    }
}
