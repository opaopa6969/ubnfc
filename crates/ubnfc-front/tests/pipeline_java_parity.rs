use ubnfc_front::ir::{AstType, Cardinality, Conversion, GrammarIr};
use ubnfc_front::pipeline::{compile, PipelineError};

fn source(text: &str) -> Result<GrammarIr, PipelineError> {
    compile("parity.ubnf", &|_| Ok(text.into())).map(|c| c.ir)
}

// 2026-09-22 の corpus IR 失敗101件（17文法）。同じ文法を既存 Java の
// UBNFMapper.parse → GrammarValidator.validate に渡し、ERROR がないことを確認。
// GrammarValidator.java:444-484 は capture 名と mapping params を検証する。
// RustConformanceTest の SemanticCardinalityConformance も Optional/List を扱う。
#[test]
fn semantic_cardinality_preserves_values_separately_from_capture_occurrences() {
    use Cardinality::{List, Optional, Scalar};
    for (name, captured, helper, extra, expected, conversion) in [
        ("node-pair", "Helper", "Leaf Leaf", "", List, Conversion::Node),
        ("mixed-pair", "Helper", "Factor Factor", "", List, Conversion::Mixed),
        ("mixed-group", "( Factor Factor )", "Factor", "", List, Conversion::Mixed),
        ("mixed-alias", "Helper", "Alias", "Alias ::= '(' Pair ')'; Pair ::= Factor Factor;", List, Conversion::Mixed),
        ("node-optional", "Helper", "'(' [ Leaf ] ')'", "", Optional, Conversion::Node),
        ("mixed-optional", "Helper", "'(' [ Factor ] ')'", "", Optional, Conversion::Mixed),
        ("node-repeat", "Helper", "'[' { Leaf } ']'", "", List, Conversion::Node),
        ("mixed-repeat", "Helper", "'[' { Factor } ']'", "", List, Conversion::Mixed),
        ("mixed-bounded", "Helper", "'[' (Factor){1,2} ']'", "", List, Conversion::Mixed),
        ("node-separated", "Helper", "'[' Leaf % ',' ']'", "", List, Conversion::Node),
        ("mixed-separated", "Helper", "'[' Factor % ',' ']'", "", List, Conversion::Mixed),
        ("optional-many-alias", "Helper", "'(' [ Items ] ')'", "Items ::= Factor % ',';", List, Conversion::Mixed),
        ("mixed-repeat-delimited-alias", "Helper", "{ Outer }", "Outer ::= '(' Factor ')';", List, Conversion::Mixed),
        ("mixed-repeat-delimited-inline", "Helper", "{ '(' Factor ')' }", "", List, Conversion::Mixed),
        ("pure-node-alias-optional", "Helper", "'(' [Alias] ')'", "Alias ::= Leaf;", Optional, Conversion::Node),
        ("pure-node-alias-many", "Helper", "{ Outer }", "Outer ::= '(' Alias ')'; Alias ::= Leaf;", List, Conversion::Node),
        ("pure-node-multi-alias-many", "Helper", "{ Outer }", "Outer ::= '(' Alias ')'; Alias ::= Leaf | Other; @mapping(Other,params=[text]) Other ::= ('z' | '🚀') @text;", List, Conversion::Mixed),
    ] {
        let text = format!(
            "grammar Semantic {{
                @package: org.example.semantic
                @whitespace: javaStyle
                @root @mapping(Box, params=[values]) Document ::= {captured} @values;
                Helper ::= {helper};
                {extra}
                Factor ::= 'a' | '😀' | Leaf;
                @mapping(Leaf, params=[text]) Leaf ::= ('x' | 'y') @text;
            }}"
        );
        let ir = source(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        let capture = ir.semantics.captures.iter().find(|c| c.name == "values").unwrap();
        assert_eq!(capture.cardinality, Scalar, "{name}: 一回の capture");
        assert_eq!(capture.conversion, conversion, "{name}");
        assert_values_cardinality(&ir, expected);
        // IR 単独の analysis でも出現回数の検査を回避しない。
        ubnfc_front::analysis::analyze(&ir.semantics).unwrap();
    }
}

fn assert_values_cardinality(ir: &GrammarIr, expected: Cardinality) {
    let field = ir
        .semantics
        .ast_types
        .iter()
        .find_map(|t| match t {
            AstType::Record { name, fields, .. } if name == "Box" => {
                fields.iter().find(|f| f.name == "values")
            }
            _ => None,
        })
        .unwrap();
    let recipe = ir
        .semantics
        .recipes
        .iter()
        .find(|r| r.id == "recipe:Document")
        .unwrap();
    assert_eq!(field.cardinality, expected);
    assert_eq!(recipe.fields[0].cardinality, expected);
}

#[test]
fn semantic_values_compose_with_syntax_and_stop_at_mapped_rules() {
    use Cardinality::{List, Optional, Scalar};
    for (body, helper, annotation, occurrences, values) in [
        ("[ Helper @values ]", "Leaf Leaf", "", Optional, List),
        ("{ Helper @values }", "[ Leaf ]", "", List, List),
        (
            "Helper @values | Leaf @values",
            "Leaf Leaf",
            "",
            Scalar,
            List,
        ),
        ("Helper @values", "'(' [ 'a' ] ')'", "", Scalar, Scalar),
        ("Helper @values", "{ 'a' }", "", Scalar, Scalar),
        (
            "Helper @values",
            "Leaf Leaf",
            "@mapping(Pair)",
            Scalar,
            Scalar,
        ),
    ] {
        let ir = source(&format!(
            "grammar Semantic {{
                @root @mapping(Box,params=[values]) Document ::= {body};
                {annotation} Helper ::= {helper};
                @mapping(Leaf) Leaf ::= 'x';
            }}"
        ))
        .unwrap();
        for c in ir.semantics.captures.iter().filter(|c| c.name == "values") {
            assert_eq!(c.cardinality, occurrences, "{body} / {helper}");
        }
        assert_values_cardinality(&ir, values);
    }
}

// GrammarValidator.java:917-945: 自己参照を除き、本文内の参照先が結合規則で
// 双方に precedence があるとき、参照先 <= 参照元を拒否する。推移閉包は取らない。
// RustConformanceTest.java:391-455,873-877 は検証器を呼ばず生成器を直接実行する。
#[test]
fn reversed_and_equal_precedence_remain_java_validator_errors() {
    for level in [10, 20, 30] {
        let text = format!(
            "grammar Operators {{
                token NUMBER = NumberParser
                @root @mapping(Binary,params=[left,op,right]) @leftAssoc @precedence(level={level})
                Expression ::= Term @left {{ AddOp @op Term @right }};
                @mapping(Binary,params=[left,op,right]) @leftAssoc @precedence(level=20)
                Term ::= Factor @left {{ MulOp @op Factor @right }};
                AddOp ::= '+' | '-'; MulOp ::= '*' | '/';
                Factor ::= NumericLiteral | '(' Expression ')';
                @mapping(Number,params=[value]) NumericLiteral ::= (NUMBER) @value;
            }}"
        );
        match (level, source(&text)) {
            (10, Ok(_)) => {}
            (20 | 30, Err(PipelineError::Validation(issues))) => {
                let errors: Vec<_> = issues.iter().filter(|i| i.severity() == "ERROR").collect();
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].code, "E-PRECEDENCE-ORDER");
                assert_eq!(errors[0].rule.as_deref(), Some("Expression"));
            }
            (_, result) => panic!("level={level}: {result:?}"),
        }
    }
}

#[test]
fn java_rejected_mapping_and_common_field_contracts_stay_rejected() {
    // GrammarValidator.java:464-483,1006-1048。
    for (body, code) in [
        ("@root @mapping(Box,params=[values]) Document ::= Helper; Helper ::= 'x';", "E-MAPPING-MISSING-CAPTURE"),
        ("@root @mapping(Box) Document ::= Helper @values; Helper ::= 'x';", "E-MAPPING-UNLISTED-CAPTURE"),
        ("@root @mapping(Node) @commonField(name) Root ::= A | B; @mapping(Node.A,params=[name]) A ::= 'a' @name; @mapping(Node.B) B ::= 'b';", "E-COMMONFIELD-MISMATCH"),
    ] {
        let Err(PipelineError::Validation(issues)) = source(&format!("grammar G {{ {body} }}")) else {
            panic!("{code} が必要");
        };
        assert!(issues.iter().any(|i| i.code == code), "{issues:?}");
    }
}

#[test]
fn oracle_generation_and_compilation_failures_remain_explicit_lower_errors() {
    // Validator は受理するが、旧 common-field-{missing,type} oracle は javac が拒否。
    // eval-sum は EvaluatorGenerator.java:35-41 が明示拒否する。
    for (body, code) in [
        ("token ID=IdentifierParser @root @mapping(Node) @commonField(name) Root ::= A | B; @mapping(A,params=[name]) A ::= 'a:' ID @name; @mapping(B) B ::= 'b';", "E-LOWER-COMMON-FIELD"),
        ("token NUM=NumberParser token ID=IdentifierParser @root @mapping(Node) @commonField(name) Root ::= A | B; @mapping(A,params=[name]) A ::= 'a:' ID @name; @mapping(B,params=[name]) B ::= 'b:' NUM @name;", "E-LOWER-COMMON-FIELD"),
        ("@root @eval(kind='literal',strategy='default') @mapping(Node) Root ::= Leaf; @mapping(Item) Leaf ::= 'a';", "E-LOWER-SUM-EVAL"),
    ] {
        let Err(PipelineError::Lower(issues)) = source(&format!("grammar Probe {{ {body} }}")) else {
            panic!("{code} が必要");
        };
        assert!(issues.iter().any(|i| matches!(i,
            ubnfc_front::lower::LowerError::Error { code: actual, .. } if actual == code
        )), "{issues:?}");
    }
}
