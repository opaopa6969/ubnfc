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
