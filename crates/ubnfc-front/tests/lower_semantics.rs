use std::collections::BTreeSet;
use std::path::PathBuf;
use ubnfc_front::ir::*;
use ubnfc_front::lower::{lower, LowerError, SemanticsIr, SourceInput};
use ubnfc_front::resolve::resolve;
use ubnfc_front::syntax::parse_file;

fn compile(text: &str) -> Result<SemanticsIr, Vec<LowerError>> {
    let path = PathBuf::from("test.ubnf");
    let grammar = resolve(&[(path.clone(), parse_file(text).unwrap())], &|_| {
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    })
    .unwrap();
    lower(&grammar, &[SourceInput::new(path, "test.ubnf", text)])
}
fn field<'a>(out: &'a SemanticsIr, ty: &str, name: &str) -> &'a AstField {
    out.semantics
        .ast_types
        .iter()
        .find_map(|t| match t {
            AstType::Record { id, fields, .. } if id == ty => {
                fields.iter().find(|f| f.name == name)
            }
            _ => None,
        })
        .unwrap()
}
fn body<'a>(out: &'a SemanticsIr, name: &str) -> &'a Expr {
    let rule = out
        .semantics
        .rules
        .iter()
        .find(|r| r.symbol_id.ends_with(&format!("::{name}")))
        .unwrap();
    out.semantics
        .expressions
        .iter()
        .find(|e| e.id == rule.body)
        .unwrap()
}
#[test]
fn nested_capture_cardinality_table_and_completion() {
    for (grammar, expected, sites) in [
        ("('a' @value 'b') @value", Cardinality::List, 2),
        ("('a' @value) @value", Cardinality::List, 2),
        ("'a' @value 'b' @value", Cardinality::List, 2),
        ("'a' @value | 'b' @value", Cardinality::Scalar, 2),
        ("'a' @value | 'b'", Cardinality::Optional, 1),
        ("[ 'a' @value ] @value", Cardinality::List, 2),
        ("{ 'a' @value } @value", Cardinality::List, 2),
    ] {
        let out = compile(&format!(
            "grammar G {{ @root @mapping(Node, params=[value]) Root ::= {grammar}; }}"
        ))
        .unwrap();
        assert_eq!(
            field(&out, "GAST.Node", "value").cardinality,
            expected,
            "{grammar}"
        );
        assert_eq!(
            field(&out, "GAST.Node", "value").value_type,
            ValueType::Text {},
            "{grammar}"
        );
        assert_eq!(out.semantics.captures.len(), sites);
        if grammar.starts_with(['(', '[', '{']) {
            assert_eq!(
                out.capture_bindings[1].site_id, out.semantics.captures[0].site_id,
                "outer capture completes last: {grammar}"
            );
            assert_eq!(out.capture_bindings[0].completion_order, 0);
            if grammar.starts_with(['[', '{']) {
                assert!(out.capture_bindings[0].inside_quantifier);
                assert!(!out.capture_bindings[1].inside_quantifier);
                assert_eq!(
                    out.capture_bindings[0].value_expr_ids,
                    out.capture_bindings[1].value_expr_ids
                );
            }
        }
    }
}
#[test]
fn numeric_capture_is_direct_text_and_quantified_values_are_bound() {
    let out=compile("grammar G { token N = NumberParser token D = DigitParser @root @mapping(Node,params=[a,b,c,d,e]) Root ::= N @a D? @b (N) @c N+ @d Leaf @e; @mapping(Leaf,params=[x]) Leaf ::= 'x' @x; }").unwrap();
    // D-074: 数値 token の直接 capture も group を挟んだ capture と同じく原文の文字列。
    for name in ["a", "c"] {
        assert_eq!(
            field(&out, "GAST.Node", name).value_type,
            ValueType::Text {}
        );
    }
    assert!(out
        .semantics
        .captures
        .iter()
        .all(|c| !matches!(c.value_type, ValueType::Number { .. })));
    assert_eq!(
        field(&out, "GAST.Node", "b").cardinality,
        Cardinality::Optional
    );
    assert_eq!(field(&out, "GAST.Node", "c").value_type, ValueType::Text {});
    assert_eq!(field(&out, "GAST.Node", "d").cardinality, Cardinality::List);
    assert_eq!(
        field(&out, "GAST.Node", "e").value_type,
        ValueType::Node {
            type_id: "GAST.Leaf".into()
        }
    );
    let d = out
        .semantics
        .captures
        .iter()
        .find(|c| c.name == "d")
        .unwrap();
    let binding = out
        .capture_bindings
        .iter()
        .find(|b| b.site_id == d.site_id)
        .unwrap();
    assert!(binding.value_expr_ids.iter().all(|id| out
        .semantics
        .expressions
        .iter()
        .any(|e| &e.id == id && matches!(e.node, ExprNode::TokenRef { .. }))));
}
#[test]
fn d015_suffix_priority_and_separated_loop() {
    for (source, min, max) in [
        ("'a'+{2,3} % ','", 1, None),
        ("'a'?{2,3} % ','", 0, Some(1)),
        ("'a'{2,3} % ','", 2, Some(3)),
    ] {
        let out = compile(&format!("grammar G {{ @root Root ::= {source} @v; }}")).unwrap();
        assert_eq!(out.warnings.len(), 1);
        assert!(
            matches!(&out.warnings[0],LowerError::Warning {code,..} if code=="W-LOWER-SUFFIX-PRIORITY")
        );
        let capture = &out.semantics.captures[0];
        let e = out
            .semantics
            .expressions
            .iter()
            .find(|e| e.id == capture.expr_id)
            .unwrap();
        assert!(matches!(e.node,ExprNode::Repeat {min:m,max:n,..} if m==min&&n==max));
        assert!(!out
            .semantics
            .expressions
            .iter()
            .any(|e| matches!(&e.node,ExprNode::Literal {value,..} if value==",")));
    }
    let out = compile("grammar G { @root Root ::= ('a' @v) % ',' @list; }").unwrap();
    assert!(out.warnings.is_empty());
    let capture = out
        .semantics
        .captures
        .iter()
        .find(|c| c.name == "list")
        .unwrap();
    let separated = out
        .semantics
        .expressions
        .iter()
        .find(|e| e.id == capture.expr_id)
        .unwrap();
    let ExprNode::Seq { items } = &separated.node else {
        panic!("separated must lower to a sequence");
    };
    assert_eq!(items.len(), 2);
    let repeat = out
        .semantics
        .expressions
        .iter()
        .find(|e| e.id == items[1])
        .unwrap();
    let ExprNode::Repeat {
        child,
        min: 0,
        max: None,
        ..
    } = &repeat.node
    else {
        panic!("zero or more pairs");
    };
    let pair = out
        .semantics
        .expressions
        .iter()
        .find(|e| &e.id == child)
        .unwrap();
    assert!(matches!(&pair.node,ExprNode::Seq {items:p} if p.len()==2&&p[1]==items[0]));
    assert!(pair.id.as_str().contains("/lower/separated/"));
    assert_eq!(out.semantics.captures.len(), 2);
}
#[test]
fn source_code_points_paths_and_crlf() {
    let text = "// 😀\r\ngrammar G { @root R ::= 'é😀'; }\r\n";
    let path = PathBuf::from("/arbitrary/location/g.ubnf");
    let resolved = resolve(
        &[(path.clone(), parse_file(text).unwrap())],
        &|_| unreachable!(),
    )
    .unwrap();
    let out = lower(
        &resolved,
        &[SourceInput::new(path.clone(), "文法/./sub/../g.ubnf", text)],
    )
    .unwrap();
    assert_eq!(out.sources[0].id, "%E6%96%87%E6%B3%95/g.ubnf");
    assert_eq!(
        out.sources[0].code_point_length,
        text.chars().count() as u64
    );
    let literal = out
        .semantics
        .expressions
        .iter()
        .find(|e| matches!(e.node, ExprNode::Literal { .. }))
        .unwrap();
    let start = text.find("'é").unwrap();
    assert_eq!(literal.source.start, text[..start].chars().count() as u64);
    assert_eq!(literal.source.end - literal.source.start, 4);
    assert!(ExprId::try_from(literal.id.0.clone()).is_ok());
    assert!(lower(
        &resolved,
        &[SourceInput::new(path.clone(), "../g.ubnf", text)]
    )
    .is_err());
    assert!(lower(
        &resolved,
        &[SourceInput::new(path.clone(), "/g.ubnf", text)]
    )
    .is_err());
    assert!(lower(
        &resolved,
        &[SourceInput::new(path, "g.ubnf", text.replace('é', "e"))]
    )
    .is_err());
    assert!(lower(&resolved, &[]).is_err());
}
#[test]
fn ast_sum_nested_enum_common_skip_and_shared_join() {
    let out=compile("grammar G { @root @mapping(Node) @commonField(name) Root ::= A | B; @mapping(Node.Item,params=[name]) A ::= 'a' @name; @mapping(Node.Other,params=[name]) B ::= 'b' @name; @enum Kind ::= 'camelCase' | 'kebab-case' | 'sync'; @skip @mapping(Ignored) Ignored ::= 'x'; }").unwrap();
    assert!(out.semantics.ast_types.iter().any(|t|matches!(t,AstType::TransparentUnion {id,variants,..} if id=="GAST.Node"&&variants==&["GAST.Node.Item","GAST.Node.Other"])));
    assert_eq!(out.semantics.common_fields.len(), 1);
    assert!(!out
        .semantics
        .ast_types
        .iter()
        .any(|t| matches!(t,AstType::Record {id,..} if id=="GAST.Ignored")));
    assert!(out
        .semantics
        .recipes
        .iter()
        .any(|r| r.kind == RecipeKind::Skip));
    let variants = out
        .semantics
        .ast_types
        .iter()
        .find_map(|t| {
            if let AstType::Enum { variants, .. } = t {
                Some(variants)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        variants
            .iter()
            .map(|v| v.java_name.as_str())
            .collect::<Vec<_>>(),
        ["CAMEL_CASE", "KEBAB_CASE", "SYNC"]
    );
    // D-074: 数値 token の直接 capture は text なので、text との共有は Mixed にならない。
    let shared=compile("grammar G { token N = NumberParser @root @mapping(Node,params=[v]) A ::= N @v; @mapping(Node,params=[v]) B ::= 'x' @v; }").unwrap();
    assert_eq!(
        field(&shared, "GAST.Node", "v").value_type,
        ValueType::Text {}
    );
    let mixed=compile("grammar G { @root @mapping(Node,params=[v]) A ::= Leaf @v; @mapping(Node,params=[v]) B ::= 'x' @v; @mapping(Leaf) Leaf ::= 'l'; }").unwrap();
    assert!(matches!(
        field(&mixed, "GAST.Node", "v").value_type,
        ValueType::Mixed { .. }
    ));
    assert!(compile("grammar G { @root @mapping(Node,params=[v]) A ::= 'a' @v; @mapping(Node,params=[v]) B ::= {'b' @v}; }").is_err());
    assert!(compile("grammar G { @root @mapping(Node,params=[a,b]) A ::= 'a' @a 'b' @b; @mapping(Node,params=[b,a]) B ::= 'a' @a 'b' @b; }").is_err());
}
#[test]
fn trivia_has_chain_and_helper_boundaries_with_precedence() {
    let out=compile("grammar G { @whitespace: javaStyle @root Root ::= Enabled Disabled; @whitespace(none) @interleave(profile=javaStyle) Disabled ::= 'a' 'b'; Enabled ::= ('c') 'd'+; Choice ::= 'x' | 'y'; }").unwrap();
    let disabled = out
        .semantics
        .rules
        .iter()
        .find(|r| r.symbol_id == "G::Disabled")
        .unwrap();
    let policy = out
        .semantics
        .trivia
        .policies
        .iter()
        .find(|p| p.id == disabled.trivia_policy_id)
        .unwrap();
    assert!(policy.effective_delimiter_set.is_empty());
    assert!(!out.semantics.trivia.insertion_plan.iter().any(|p| out
        .semantics
        .expressions
        .iter()
        .any(|e| e.id == p.boundary.owner_expr_id && e.rule_id == "G::Choice")));
    let root = body(&out, "Root");
    let boundaries: Vec<_> = out
        .semantics
        .trivia
        .insertion_plan
        .iter()
        .filter(|p| p.boundary.owner_expr_id == root.id)
        .collect();
    assert_eq!(boundaries.len(), 3);
    assert_eq!(boundaries[0].boundary.at, BoundaryAt::Entry);
    assert_eq!(boundaries[1].boundary.child_index, Some(0));
    assert!(out
        .semantics
        .trivia
        .insertion_plan
        .iter()
        .any(|p| p.boundary.at == BoundaryAt::HelperEntry));
    let comments = compile(
        "grammar G { @comment: {line: '#' blockOpen: '<!--' blockClose: '-->'} @root R ::= 'x'; }",
    )
    .unwrap();
    let p = &comments.semantics.trivia.policies[0];
    // D-018 comment-trivia: javaStyle なしの明示 @comment は宣言した区切りだけを有効にする
    // （旧 Java の「line の値を無視して // を登録」は再現しない）。
    assert!(p
        .effective_delimiter_set
        .iter()
        .any(|d| matches!(d,Delimiter::LineComment {open,..} if open=="#")));
    assert!(!p
        .effective_delimiter_set
        .iter()
        .any(|d| matches!(d,Delimiter::LineComment {open,..} if open=="//")));
    assert!(p
        .effective_delimiter_set
        .iter()
        .any(|d| matches!(d,Delimiter::BlockComment {open,close} if open=="<!--"&&close=="-->")));
    assert!(!p
        .effective_delimiter_set
        .iter()
        .any(|d| matches!(d, Delimiter::Characters { .. })));
    // javaStyle と併用すると既定集合へ追加し、同一の区切りは重複除去する。`block: '(* *)'`
    // は blockOpen/blockClose と同じ表現へ写す（corpus/oracle/comment-block.json）。
    let merged = compile(
        "grammar G { @whitespace: javaStyle @comment: {line: '//' block: '(* *)'} @root R ::= 'x'; }",
    )
    .unwrap();
    let p = &merged.semantics.trivia.policies[0];
    assert_eq!(
        p.raw_settings
            .comment
            .as_ref()
            .and_then(|c| c.block_open.as_deref()),
        Some("(*")
    );
    assert_eq!(
        p.raw_settings
            .comment
            .as_ref()
            .and_then(|c| c.block_close.as_deref()),
        Some("*)")
    );
    assert_eq!(p.effective_delimiter_set.len(), 4);
    assert!(matches!(
        &p.effective_delimiter_set[0],
        Delimiter::Characters { .. }
    ));
    assert!(
        matches!(&p.effective_delimiter_set[1], Delimiter::LineComment { open, .. } if open == "//")
    );
    assert!(
        matches!(&p.effective_delimiter_set[2], Delimiter::BlockComment { open, close } if open == "/*" && close == "*/")
    );
    assert!(
        matches!(&p.effective_delimiter_set[3], Delimiter::BlockComment { open, close } if open == "(*" && close == "*)")
    );
    // rule @whitespace(none) はその rule の自動 trivia 全体を無効にする。
    let none =
        compile("grammar G { @comment: {line: '#'} @root R ::= S; @whitespace(none) S ::= 'x'; }")
            .unwrap();
    let s_rule = none
        .semantics
        .rules
        .iter()
        .find(|r| r.symbol_id == "G::S")
        .unwrap();
    let s_policy = none
        .semantics
        .trivia
        .policies
        .iter()
        .find(|p| p.id == s_rule.trivia_policy_id)
        .unwrap();
    assert!(s_policy.effective_delimiter_set.is_empty());
}
#[test]
fn structured_annotations_and_recovery_application_sites() {
    let out=compile("grammar G { @memoSafeToken: N token N = NumberParser @root @scopeTree(mode=lexical) Root ::= Decl Use Auto; @declares(symbol=n) @scopeTree(mode=dynamic) Decl ::= N @n; @backref(name=n) @catalog(context='vars') @mapping(Use,params=[n]) @eval(kind='literal',strategy='auto') @doc('use') Use ::= N @n; @recovery(AUTO) Auto ::= 'a'; }").unwrap();
    let scope = out
        .semantics
        .scope
        .effects
        .iter()
        .find(|e| e.rule_id == "G::Decl")
        .unwrap();
    assert_eq!(
        scope.events.iter().map(|e| e.action).collect::<Vec<_>>(),
        [
            ScopeEffectEventsAction::Enter,
            ScopeEffectEventsAction::Leave,
            ScopeEffectEventsAction::Declare
        ]
    );
    assert_eq!(scope.mode, ScopeEffectMode::Dynamic);
    assert_eq!(out.semantics.catalogs.len(), 1);
    assert_eq!(
        out.semantics.tokens[0].memo_safety.provenance,
        MemoSafetyProvenance::UserAliasAssertion
    );
    let rule = out
        .semantics
        .rules
        .iter()
        .find(|r| r.symbol_id == "G::Use")
        .unwrap();
    assert_eq!(rule.eval.as_ref().unwrap().strategy, EvalStrategy::Auto);
    assert!(rule.annotations.iter().any(|a| a.name == "doc"));
    assert_eq!(out.recovery_follow_sync_patterns["G::Auto"], None);
    assert_eq!(out.semantics.recovery[0].application_sites.len(), 1);
    let backref = compile("grammar G { @root @backref(name=x) R ::= 'a' @x 'a' @x; }").unwrap();
    assert_eq!(
        backref.semantics.scope.effects[0].mode,
        ScopeEffectMode::SameRuleText
    );
    let sync = compile("grammar G { @root @recovery(sync=';') R ::= 'a'; }").unwrap();
    assert_eq!(sync.semantics.recovery[0].resolved_sync_patterns, [";"]);
    assert!(sync.semantics.recovery[0].application_sites.is_empty());
}
#[test]
fn token_specs_and_choice_annotations() {
    let out=compile("grammar G { token N = NumberParser token ID = IdentifierParser token S = SingleQuotedParser token Q = QuotedParser token A = AlphabetParser token X = org.example.NumberParser token U = UNTIL('end') token NEG = NEGATION('x') token L = LOOKAHEAD('x') token NL = NEGATIVE_LOOKAHEAD('x') token C = CHAR_RANGE('a','z') token CI = CI('yes') token RE = REGEX('[a-z]+') token ANY = ANY token EOF = EOF token EMPTY = EMPTY @root @longestChoice R ::= 'a' | 'ab'; @predictiveChoice P ::= 'x' | 'y'; }").unwrap();
    assert!(matches!(
        out.semantics.tokens[5].spec,
        TokenSpec::Extern { .. }
    ));
    assert!(!out.semantics.tokens[5].memo_safety.success);
    assert!(matches!(
        out.semantics.tokens[6].spec,
        TokenSpec::Until { .. }
    ));
    assert!(matches!(
        body(&out, "R").node,
        ExprNode::Choice {
            selection: ChoiceSelection::Longest,
            ..
        }
    ));
    assert!(matches!(
        body(&out, "P").node,
        ExprNode::Choice {
            selection: ChoiceSelection::Predictive,
            ..
        }
    ));
}
#[test]
fn d016_rejections_are_explicit() {
    let supported =
        compile("grammar G { @root E ::= E Op T | T; Op ::= '+'; T ::= 'a'; }").unwrap();
    assert_eq!(supported.semantics.normalizations.len(), 1);
    let rewrite = &supported.semantics.normalizations[0];
    assert_eq!(body(&supported, "E").id, rewrite.rewritten_body);
    assert!(supported
        .semantics
        .expressions
        .iter()
        .any(|e| e.id == rewrite.original_body));
    for text in [
        "grammar G { @root E ::= T | E Op T; Op ::= '+'; T ::= 'a'; }",
        "grammar G { @root E ::= E Op T | T; Op ::= ['+']; T ::= ['a']; }",
        "grammar G { @root E ::= E Op @op T | T; Op ::= '+'; T ::= 'a'; }",
        "grammar G { @root @recovery(sync=';') E ::= E Op T | T; Op ::= '+'; T ::= 'a'; }",
        "grammar G { @root E ::= (E) '+' | 'a'; }",
        "grammar G { @root E ::= E E T | T; T ::= 'a'; }",
        "grammar G { @root E ::= E Op T | T; Op ::= '+'; @mapping(Node,params=[v]) T ::= 'a' @v; }",
    ] {
        let errors = compile(text).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e,LowerError::Error {code,..} if code=="E-LOWER-D016")),
            "{errors:?}"
        );
    }
    let errors = compile("grammar G { @root E ::= T 'x' | 'a'; T ::= E; }").unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e,LowerError::Error {code,..} if code=="E-LOWER-INDIRECT-LEFT-RECURSION")
    ));
}
#[test]
fn imported_aliases_keep_unique_expr_and_site_ids() {
    let root="grammar G { @import a from 'a.ubnf' @import b from 'a.ubnf' @root Root ::= a.Item b.Item; }";
    let imported = "grammar Items { Item ::= 'a' @v; }";
    let grammar = resolve(
        &[(PathBuf::from("root.ubnf"), parse_file(root).unwrap())],
        &|_| Ok(imported.into()),
    )
    .unwrap();
    let out = lower(
        &grammar,
        &[
            SourceInput::new("root.ubnf", "root.ubnf", root),
            SourceInput::new("a.ubnf", "a.ubnf", imported),
        ],
    )
    .unwrap();
    assert_eq!(out.sources.len(), 2);
    assert_eq!(out.grammar_units.len(), 2);
    assert_eq!(out.import_edges.len(), 2);
    assert_eq!(
        out.semantics
            .expressions
            .iter()
            .map(|e| &e.id)
            .collect::<BTreeSet<_>>()
            .len(),
        out.semantics.expressions.len()
    );
    assert_eq!(out.semantics.captures.len(), 2);
    assert_ne!(
        out.semantics.captures[0].site_id,
        out.semantics.captures[1].site_id
    );
}

#[test]
fn typeof_links_capture_sites_and_eval_rejects_automatic_sum() {
    let out =
        compile("grammar G { @root @mapping(Node,params=[a,b]) R ::= @typeof(a) 'b' @b 'a' @a; }")
            .unwrap();
    let constraint = &out.semantics.typeof_constraints[0];
    assert_eq!(
        out.semantics
            .captures
            .iter()
            .find(|c| c.site_id == constraint.own_capture)
            .unwrap()
            .name,
        "b"
    );
    assert_eq!(
        out.semantics
            .captures
            .iter()
            .find(|c| c.site_id == constraint.referenced_capture)
            .unwrap()
            .name,
        "a"
    );
    assert_eq!(constraint.phase, TypeofPhase);
    let source="grammar G { @root @mapping(Node) @eval(kind='literal',strategy='auto') R ::= A | B; @mapping(A) A ::= 'a'; @mapping(B) B ::= 'b'; }";
    assert!(compile(source)
        .unwrap_err()
        .iter()
        .any(|e| matches!(e,LowerError::Error {code,..} if code=="E-EVAL-SUM-TYPE")));
    assert!(compile(&source.replace("strategy='auto'", "strategy='manual'")).is_ok());
    let source="grammar G { @root @mapping(Node) R ::= A; @mapping(Node) S ::= 's'; @mapping(A) A ::= 'a'; }";
    assert!(compile(source)
        .unwrap_err()
        .iter()
        .any(|e| matches!(e,LowerError::Error {code,..} if code=="E-LOWER-SHARED-SCHEMA")));
}

#[test]
fn empty_alternatives_keep_original_source_intervals() {
    let text = "// 😀\r\ngrammar G { @root R ::= | 'x' | ; Empty ::= ; Helpers ::= [] {} (|); }";
    let out = compile(text).unwrap();
    let choices: Vec<_> = out
        .semantics
        .expressions
        .iter()
        .filter(|e| matches!(e.node, ExprNode::Choice { .. }))
        .collect();
    assert_eq!(choices.len(), 2);
    for e in &out.semantics.expressions {
        assert!(e.source.start > 0, "source-derived interval: {e:?}");
        if matches!(&e.node,ExprNode::Seq {items} if items.is_empty()) {
            assert_eq!(e.source.start, e.source.end);
        }
        assert!(ExprId::try_from(e.id.0.clone()).is_ok());
    }
    let at = text.find("| 'x'").unwrap();
    assert_eq!(choices[0].source.start, text[..at].chars().count() as u64);
    assert_eq!(
        out.semantics
            .expressions
            .iter()
            .map(|e| &e.id)
            .collect::<BTreeSet<_>>()
            .len(),
        out.semantics.expressions.len()
    );
}
