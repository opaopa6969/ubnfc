//! UBNF v2: パラメータ化ルール（D-043）。
//!
//! 中心の主張は 1 つ:「雛形を実体化した文法の IR は、同じ形を手で書いた文法の IR と
//! **文法ソース上の範囲を除いて完全に一致する**」。雛形は規則を作らないので、
//! 規則数・AST クラス・capture・rulePath・expected ラベルは 1 つも変わらない。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ubnfc_front::pipeline::{compile, Compiled, PipelineError};

fn source(text: &str) -> Result<Compiled, PipelineError> {
    let files: BTreeMap<PathBuf, String> =
        BTreeMap::from([("/virtual/main.ubnf".into(), text.into())]);
    compile("/virtual/main.ubnf", &|path: &Path| Ok(files[path].clone()))
}

fn codes(text: &str) -> Vec<String> {
    match source(text) {
        Ok(compiled) => compiled.warnings.iter().map(|i| i.code.clone()).collect(),
        Err(PipelineError::Validation(issues) | PipelineError::Strict(issues)) => {
            issues.iter().map(|i| i.code.clone()).collect()
        }
        Err(PipelineError::Resolve(errors)) => vec![format!("resolve: {errors:?}")],
        Err(other) => panic!("想定外のエラー: {other}"),
    }
}

fn grammar(version: u64, body: &str) -> String {
    format!(
        "grammar G {{\n  @package: org.example\n  @ubnf: {version}\n  @whitespace: javaStyle\n  \
         token NUMBER = NUMBER\n  token IDENTIFIER = IDENTIFIER\n{body}}}"
    )
}

/// IR を「文法ソース上の範囲」だけ潰して比べるための正規形。
fn shape(compiled: &Compiled) -> String {
    let mut text = serde_json::to_string_pretty(&compiled.ir).expect("IR");
    // ExprId は expr:<source>:<start>:<end>:<role>。範囲は実体化の位置なので潰す。
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let line = if let Some(rest) = line.find("expr:") {
            let (head, tail) = line.split_at(rest);
            let parts: Vec<&str> = tail.splitn(5, ':').collect();
            if parts.len() == 5 {
                format!("{head}expr:*:*:{}", parts[4])
            } else {
                line.to_owned()
            }
        } else {
            line.to_owned()
        };
        let trimmed = line.trim_start();
        let skip = [
            "\"start\":",
            "\"end\":",
            "\"sha256\":",
            "\"codePointLength\":",
        ]
        .iter()
        .any(|k| trimmed.starts_with(k));
        if !skip {
            out.push_str(&line);
            out.push('\n');
        }
    }
    text.clear();
    out
}

fn names(compiled: &Compiled) -> Vec<String> {
    compiled
        .ir
        .semantics
        .rules
        .iter()
        .map(|r| r.symbol_id.rsplit("::").next().unwrap().to_owned())
        .collect()
}

/// 雛形版と手書き版が同じ IR になることを確かめる。
fn equivalent(templated: &str, handwritten: &str) {
    let a = source(&grammar(2, templated)).expect("雛形版");
    let b = source(&grammar(2, handwritten)).expect("手書き版");
    assert_eq!(names(&a), names(&b), "規則の名前と個数");
    assert_eq!(shape(&a), shape(&b), "範囲を除いた IR");
}

// ── 等価性 ──────────────────────────────────────────────────────────────────

/// 中心の主張。capture は実体化した規則のフィールドになるので、`@mapping` の
/// params も AST も手書きと同じになる。
#[test]
fn an_instantiated_template_lowers_to_exactly_the_handwritten_shape() {
    equivalent(
        "  SepList<X, Sep> ::= X @values { Sep X @values } ;\n\
         \x20 @root\n\
         \x20 @mapping(Args, params=[values])\n\
         \x20 Args ::= SepList<NUMBER, ','> ;\n\
         \x20 @mapping(Names, params=[values])\n\
         \x20 Names ::= SepList<IDENTIFIER, ';'> ;\n",
        "  @root\n\
         \x20 @mapping(Args, params=[values])\n\
         \x20 Args ::= NUMBER @values { ',' NUMBER @values } ;\n\
         \x20 @mapping(Names, params=[values])\n\
         \x20 Names ::= IDENTIFIER @values { ';' IDENTIFIER @values } ;\n",
    );
}

/// 実体化が要素の途中にあるときは、要素列がその場へ**差し込まれる**。
#[test]
fn a_mid_sequence_instantiation_splices_its_elements_in_place() {
    equivalent(
        "  Pair<A, B> ::= A @left B @right ;\n\
         \x20 @root\n\
         \x20 @mapping(R, params=[left, right])\n\
         \x20 Root ::= '(' Pair<NUMBER, IDENTIFIER> ')' ;\n",
        "  @root\n\
         \x20 @mapping(R, params=[left, right])\n\
         \x20 Root ::= '(' NUMBER @left IDENTIFIER @right ')' ;\n",
    );
}

/// 候補が複数ある雛形でも、規則本体まるごとなら括弧が 1 段増えない。
#[test]
fn a_whole_body_instantiation_keeps_the_choice_at_the_top_level() {
    equivalent(
        "  Alt<A, B> ::= A @value | B @value | '?' ;\n\
         \x20 @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= Alt<NUMBER, IDENTIFIER> ;\n",
        "  @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= NUMBER @value | IDENTIFIER @value | '?' ;\n",
    );
}

/// `[ ... ]` は雛形の中でも `[ ... ]` のまま（`X?` の Repeat にならない）。
#[test]
fn a_bracketed_optional_inside_a_template_stays_a_bracketed_optional() {
    equivalent(
        "  Opt<A> ::= '(' [ A @value ] ')' ;\n\
         \x20 @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= Opt<NUMBER> ;\n",
        "  @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= '(' [ NUMBER @value ] ')' ;\n",
    );
}

/// 実引数は atom なので、括弧つきの選択も渡せる。
#[test]
fn an_argument_may_be_a_group_a_literal_or_a_reference() {
    equivalent(
        "  Wrap<A> ::= '<' A @value '>' ;\n\
         \x20 @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= Wrap<( NUMBER | IDENTIFIER )> ;\n",
        "  @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= '<' ( NUMBER | IDENTIFIER ) @value '>' ;\n",
    );
}

/// 雛形は雛形を使える（依存グラフが DAG なら展開は必ず止まる）。
#[test]
fn a_template_may_instantiate_another_template() {
    equivalent(
        "  Inner<A> ::= A @value ;\n\
         \x20 Outer<A, B> ::= Inner<A> ',' Inner<B> ;\n\
         \x20 @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= Outer<NUMBER, IDENTIFIER> ;\n",
        "  @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= NUMBER @value ',' IDENTIFIER @value ;\n",
    );
}

/// 実体化に量指定子や capture が付いているときは、手で書くときと同じく括弧が付く。
#[test]
fn a_quantified_instantiation_is_wrapped_in_a_group() {
    equivalent(
        "  Item<A> ::= A ',' ;\n\
         \x20 @root\n\
         \x20 Root ::= Item<NUMBER>* IDENTIFIER ;\n",
        "  @root\n  Root ::= ( NUMBER ',' )* IDENTIFIER ;\n",
    );
}

/// 同じ雛形を二度実体化しても ExprId は衝突しない（範囲が実体化の位置になる）。
#[test]
fn two_instantiations_of_the_same_template_get_distinct_expression_ids() {
    let compiled = source(&grammar(
        2,
        "  One<A> ::= A ;\n  @root\n  Root ::= One<NUMBER> One<IDENTIFIER> ;\n",
    ))
    .expect("compile");
    let ids: Vec<&str> = compiled
        .ir
        .semantics
        .expressions
        .iter()
        .map(|e| e.id.as_str())
        .collect();
    let unique: std::collections::BTreeSet<&&str> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len(), "ExprId が重複している: {ids:?}");
}

// ── 拒否するもの ────────────────────────────────────────────────────────────

/// v1 は凍結。雛形は `@ubnf: 2` でだけ書ける。
#[test]
fn templates_are_rejected_under_v1() {
    let text = grammar(1, "  One<A> ::= A ;\n  @root\n  Root ::= One<NUMBER> ;\n");
    let found = codes(&text);
    assert!(
        found.contains(&"E-TEMPLATE-UNSUPPORTED".to_owned()),
        "{found:?}"
    );
}

/// 雛形の再帰は**必ず**無限展開になるので、循環は例外なく拒否する。
#[test]
fn a_recursive_template_is_rejected() {
    let text = grammar(
        2,
        "  L<A> ::= A [ ',' L<A> ] ;\n  @root\n  Root ::= L<NUMBER> ;\n",
    );
    let found = codes(&text);
    assert!(
        found.contains(&"E-TEMPLATE-RECURSION".to_owned()),
        "{found:?}"
    );
}

#[test]
fn a_mutually_recursive_pair_of_templates_is_rejected() {
    let text = grammar(
        2,
        "  A1<X> ::= X A2<X> ;\n  A2<X> ::= X [ A1<X> ] ;\n  @root\n  Root ::= A1<NUMBER> ;\n",
    );
    let found = codes(&text);
    assert!(
        found.contains(&"E-TEMPLATE-RECURSION".to_owned()),
        "{found:?}"
    );
}

/// 再帰が要る形は通常の規則で書ける（雛形の引数に規則を渡すのは循環ではない）。
#[test]
fn recursion_through_an_ordinary_rule_is_allowed() {
    equivalent(
        "  Paren<A> ::= '(' A @value ')' ;\n\
         \x20 @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= Paren<Root> | NUMBER ;\n",
        "  @root\n\
         \x20 @mapping(R, params=[value])\n\
         \x20 Root ::= '(' Root @value ')' | NUMBER ;\n",
    );
}

#[test]
fn an_arity_mismatch_is_rejected() {
    let text = grammar(2, "  P<A, B> ::= A B ;\n  @root\n  Root ::= P<NUMBER> ;\n");
    let found = codes(&text);
    assert!(found.contains(&"E-TEMPLATE-ARITY".to_owned()), "{found:?}");
}

#[test]
fn an_unknown_template_is_rejected() {
    let text = grammar(2, "  @root\n  Root ::= Missing<NUMBER> ;\n");
    let found = codes(&text);
    assert!(
        found.contains(&"E-TEMPLATE-UNKNOWN".to_owned()),
        "{found:?}"
    );
}

/// パラメータを雛形として実体化する（高階）形は未対応。
#[test]
fn a_higher_order_parameter_is_rejected() {
    let text = grammar(
        2,
        "  Apply<F, A> ::= F<A> ;\n  Id<A> ::= A ;\n  @root\n  Root ::= Apply<Id, NUMBER> ;\n",
    );
    let found = codes(&text);
    assert!(
        found.contains(&"E-TEMPLATE-HIGHER-ORDER".to_owned()),
        "{found:?}"
    );
}

/// パラメータ名が規則 / token を隠すと、名前による代入が健全でなくなる。
#[test]
fn a_parameter_shadowing_a_rule_or_token_is_rejected() {
    let text = grammar(
        2,
        "  P<NUMBER> ::= NUMBER ;\n  @root\n  Root ::= P<IDENTIFIER> ;\n",
    );
    let found = codes(&text);
    assert!(
        found.contains(&"E-TEMPLATE-PARAM-SHADOW".to_owned()),
        "{found:?}"
    );
}

#[test]
fn a_duplicate_parameter_is_rejected() {
    let text = grammar(
        2,
        "  P<A, A> ::= A ;\n  @root\n  Root ::= P<NUMBER, NUMBER> ;\n",
    );
    let found = codes(&text);
    assert!(
        found.contains(&"E-TEMPLATE-PARAM-DUPLICATE".to_owned()),
        "{found:?}"
    );
}

#[test]
fn a_template_named_like_a_rule_is_rejected() {
    let text = grammar(2, "  Root<A> ::= A ;\n  @root\n  Root ::= Root<NUMBER> ;\n");
    let found = codes(&text);
    assert!(
        found.contains(&"E-TEMPLATE-NAME-COLLISION".to_owned()),
        "{found:?}"
    );
}

/// 注釈は実体化する規則の側に書く（AST は実体化した規則のもの）。
#[test]
fn an_annotation_on_a_template_is_rejected() {
    let text = grammar(
        2,
        "  @mapping(X)\n  P<A> ::= A ;\n  @root\n  Root ::= P<NUMBER> ;\n",
    );
    let found = codes(&text);
    assert!(
        found.contains(&"E-TEMPLATE-ANNOTATION".to_owned()),
        "{found:?}"
    );
}

// ── 警告 ────────────────────────────────────────────────────────────────────

#[test]
fn an_unused_parameter_and_an_uninstantiated_template_are_warned_about() {
    let text = grammar(
        2,
        "  P<A, B> ::= A ;\n  Q<A> ::= A ;\n  @root\n  Root ::= P<NUMBER, IDENTIFIER> ;\n",
    );
    let found = codes(&text);
    assert!(
        found.contains(&"W-TEMPLATE-PARAM-UNUSED".to_owned()),
        "{found:?}"
    );
    assert!(found.contains(&"W-TEMPLATE-UNUSED".to_owned()), "{found:?}");
}

// ── v1 の凍結 ───────────────────────────────────────────────────────────────

/// `<` は v1 でも字句として読めるようになったが、構文解析が必ず拒否する。
#[test]
fn an_angle_bracket_is_still_a_syntax_error_wherever_v1_allowed_no_template() {
    let text = grammar(2, "  @root\n  Root ::= NUMBER < IDENTIFIER ;\n");
    assert!(matches!(source(&text), Err(PipelineError::Syntax { .. })));
}
