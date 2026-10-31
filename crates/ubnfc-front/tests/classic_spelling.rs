//! UBNF の綴りの正本は Classic（unlaxer-parser）である（issue #11、D-084）。
//! front が受け付ける Classic の綴りと、IR に落とせないものの位置付きの拒否を固定する。
//!
//! - 引数付き規則 `F(p) ::=` と呼び出し `F(a)`（#701）: `<...>` の雛形（D-043）と同じ展開。
//! - 値渡しの引数 `F(=p)`（#704）: `CAPTURE_SCOPE` / `CAPTURE` / `SAME_AS` に展開し、拒否する。
//! - 規則の中の `CAPTURE(` `SAME_AS(` `CAPTURE_SCOPE(` `COMMIT(` `LABEL(` `CATCH(`: 拒否する。
//! - `@autoCommit`（#702）: 受理と AST を変えない注釈なので、受け付けて何も挿入しない。

use serde_json::Value;
use ubnfc_front::pipeline::{compile, validate_file, PipelineError};
use ubnfc_front::syntax::ast::{Atom, GrammarDecl};
use ubnfc_front::syntax::parse_file;

fn grammar(source: &str) -> GrammarDecl {
    parse_file(source).unwrap().grammars.pop().unwrap()
}

/// 脱糖（雛形の展開を含む）後の規則を、位置を除いた JSON にする。
fn expanded_rules(source: &str) -> Value {
    let mut file = parse_file(source).unwrap();
    let mut issues = Vec::new();
    ubnfc_front::desugar::file(&mut file, &mut issues);
    assert!(issues.iter().all(|i| i.severity() != "ERROR"), "{issues:?}");
    let mut value = serde_json::to_value(&file.grammars.last().unwrap().rules).unwrap();
    fn strip(value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.remove("span");
                map.values_mut().for_each(strip);
            }
            Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    strip(&mut value);
    value
}

/// 拒否の (code, 原文の切り出し)。
fn rejection(source: &str) -> Vec<(String, String)> {
    let text = source.to_owned();
    match validate_file("/virtual/main.ubnf", &move |_| Ok(text.clone())) {
        Err(PipelineError::Resolve(errors)) => errors
            .into_iter()
            .map(|e| (e.code, source[e.span.start..e.span.end].to_owned()))
            .collect(),
        Err(PipelineError::Syntax { error, .. }) => {
            vec![(
                error.message,
                source[error.span.start..error.span.end].to_owned(),
            )]
        }
        Ok(issues) => issues
            .into_iter()
            .filter(|i| i.severity() == "ERROR")
            .map(|i| (i.code, String::new()))
            .collect(),
        Err(other) => vec![(other.to_string(), String::new())],
    }
}

#[test]
fn a_call_is_only_a_call_when_the_grammar_defines_the_name_and_the_parenthesis_is_adjacent() {
    // 定義が無ければ、隣接した `Foo('a')` は従来どおり参照と group。
    let g = grammar("grammar G { Root ::= Foo('a') ; Foo ::= 'b' ; }");
    let atoms: Vec<_> = g.rules[0].body.alternatives()[0]
        .elements
        .iter()
        .map(|e| &e.atom)
        .collect();
    assert!(matches!(
        atoms.as_slice(),
        [Atom::Ref { .. }, Atom::Group { .. }]
    ));
    // 定義があれば呼び出し。空白を挟めば参照と group（引数の数 0 で E-PARAM-ARITY）。
    let g = grammar("grammar G { Root ::= F('a') F ('a') ; F(x) ::= x x ; }");
    assert_eq!(g.templates.len(), 1);
    assert!(g.templates[0].classic);
    let atoms: Vec<_> = g.rules[0].body.alternatives()[0]
        .elements
        .iter()
        .map(|e| &e.atom)
        .collect();
    assert!(matches!(
        atoms.as_slice(),
        [
            Atom::Instantiate { classic: true, .. },
            Atom::Ref { .. },
            Atom::Group { .. }
        ]
    ));
    // 定義は grammar ブロックごと（unlaxer-parser parameterized-rules-two-grammars.ubnf）。
    let file = parse_file("grammar A { Root ::= F('a') ; F(x) ::= x { x } ; } grammar B { Root ::= F('b') ; F ::= 'c' ; }")
        .unwrap();
    assert_eq!(file.grammars[0].templates.len(), 1);
    assert!(file.grammars[1].templates.is_empty());
    assert!(matches!(
        file.grammars[1].rules[0].body.alternatives()[0].elements[0].atom,
        Atom::Ref { .. }
    ));
}

#[test]
fn classic_calls_expand_like_the_hand_written_grammar_without_a_version_gate() {
    // `@ubnf: 2` が無くても使える（Classic は版を問わない）。
    let calls = expanded_rules(
        "grammar G { @root Root ::= Kw('let') Sep('a', ',') Opt('b' 'c')? ; \
         Spacing ::= { ' ' } ; Token(p) ::= p Spacing ; Kw(w) ::= Token(w) ; \
         Sep(item, sep) ::= item { sep item } ; Opt(x) ::= [ x ] ; }",
    );
    let by_hand = expanded_rules(
        "grammar G { @root Root ::= 'let' Spacing 'a' { ',' 'a' } ( [ ( 'b' 'c' ) ] )? ; \
         Spacing ::= { ' ' } ; }",
    );
    assert_eq!(calls, by_hand);
}

#[test]
fn malformed_classic_definitions_and_calls_are_syntax_errors() {
    for source in [
        "grammar G { F() ::= 'a' ; }",
        "grammar G { F(x,) ::= x ; }",
        "grammar G { F('a') ::= 'a' ; }",
        "grammar G { F(==x) ::= x ; }",
        "grammar G { F(=) ::= 'a' ; }",
        "grammar G { F(x=) ::= x ; }",
        "grammar G { F (x) ::= x ; }",
        "grammar G { Root ::= F() ; F(x) ::= x ; }",
        "grammar G { Root ::= F('a',) ; F(x) ::= x ; }",
        "grammar G { Root ::= F(='a') ; F(x) ::= x ; }",
    ] {
        assert!(parse_file(source).is_err(), "{source}");
    }
    let error = parse_file("grammar G { CAPTURE(x) ::= x ; }").unwrap_err();
    assert!(
        error.message.starts_with("E-PARAM-RESERVED"),
        "{}",
        error.message
    );
}

#[test]
fn classic_checks_use_the_classic_codes() {
    let cases = [
        (
            "grammar G { Root ::= F 'a' ; F(x) ::= x ; }",
            "E-PARAM-ARITY",
        ),
        (
            "grammar G { Root ::= F('a', 'b') ; F(x) ::= x ; }",
            "E-PARAM-ARITY",
        ),
        (
            "grammar G { Root ::= F('a') ; F(x) ::= G(x) ; G(y) ::= F(y) ; }",
            "E-PARAM-CYCLE",
        ),
        (
            "grammar G { Root ::= F('a') ; @root F(x) ::= x ; }",
            "E-PARAM-ANNOTATION",
        ),
        (
            "grammar G { Root ::= F('a' @t) ; F(x) ::= x ; }",
            "E-PARAM-CAPTURE",
        ),
        (
            "grammar G { Root ::= F('a') ; F(x) ::= x @t ; }",
            "E-PARAM-CAPTURE",
        ),
        (
            "grammar G { Root ::= F('a') ; R ::= 'r' ; F(R) ::= R ; }",
            "E-PARAM-SHADOW",
        ),
        (
            "grammar G { Root ::= F('a', 'b') ; F(x, x) ::= x ; }",
            "E-PARAM-DUPLICATE",
        ),
    ];
    for (source, code) in cases {
        let codes: Vec<_> = rejection(source).into_iter().map(|(c, _)| c).collect();
        assert!(
            codes.iter().any(|c| c == code),
            "{source}: {code} を期待したが {codes:?}"
        );
    }
}

#[test]
fn owned_captures_and_cuts_in_rules_are_rejected_at_their_position() {
    let source = "grammar G { @root Root ::= CAPTURE(w, 'x') ':' SAME_AS(w) Rest ; \
                  Rest ::= CAPTURE_SCOPE('y') COMMIT('z', 'msg') LABEL(l, 'a') CATCH('b', 'c', l) ; }";
    let mut got = rejection(source);
    got.sort();
    let mut want: Vec<(String, String)> = [
        ("E-CUT-UNSUPPORTED", "CATCH('b', 'c', l)"),
        ("E-CUT-UNSUPPORTED", "COMMIT('z', 'msg')"),
        ("E-CUT-UNSUPPORTED", "LABEL(l, 'a')"),
        ("E-OWNED-CAPTURE-UNSUPPORTED", "CAPTURE(w, 'x')"),
        ("E-OWNED-CAPTURE-UNSUPPORTED", "CAPTURE_SCOPE('y')"),
        ("E-OWNED-CAPTURE-UNSUPPORTED", "SAME_AS(w)"),
    ]
    .into_iter()
    .map(|(c, s)| (c.to_owned(), s.to_owned()))
    .collect();
    want.sort();
    assert_eq!(got, want);
    // 名前と `(` の間に空白があれば従来どおり参照と group。
    let source = "grammar G { @root Root ::= CAPTURE ('x') SAME_AS ('y') COMMIT ('z') ; \
                  CAPTURE ::= 'c' ; SAME_AS ::= 's' ; COMMIT ::= 'm' ; }";
    assert!(rejection(source).is_empty(), "{:?}", rejection(source));
}

#[test]
fn value_parameters_expand_to_owned_captures_and_are_rejected_at_the_call() {
    let source = "grammar G { @whitespace: none @root Root ::= '<' Body(Name) ; \
                  Name ::= 'a' { 'a' } ; Body(= tag) ::= '>' '</' tag '>' ; }";
    let got = rejection(source);
    assert!(
        got.iter()
            .any(|(code, text)| code == "E-OWNED-CAPTURE-UNSUPPORTED" && text == "Body(Name)"),
        "{got:?}"
    );
    // 名前渡しだけなら受理する（同じ形の #701）。
    let source = "grammar G { @whitespace: none @root Root ::= '<' Body(Name) ; \
                  Name ::= 'a' { 'a' } ; Body(tag) ::= '>' '</' tag '>' ; }";
    assert!(rejection(source).is_empty());
}

#[test]
fn auto_commit_is_accepted_without_changing_the_ir() {
    let with = "grammar G { @root @autoCommit @mapping(S, params=[n]) S ::= 'let' N @n ';' | 'print' N @n ';' ; N ::= 'n' ; }";
    // 位置（ExprId に入る）をそろえるため、注釈の分だけ空白にする。
    let without = "grammar G { @root             @mapping(S, params=[n]) S ::= 'let' N @n ';' | 'print' N @n ';' ; N ::= 'n' ; }";
    assert_eq!(with.len(), without.len());
    let issues = validate_file("/virtual/main.ubnf", &|_| Ok(with.to_owned())).unwrap();
    assert!(issues.is_empty(), "{issues:?}");
    let ir = |text: &str| {
        let text = text.to_owned();
        let mut value = serde_json::to_value(
            compile("/virtual/main.ubnf", &move |_| Ok(text.clone()))
                .unwrap()
                .ir,
        )
        .unwrap();
        // 原文の SHA-256 と長さ、注釈の metadata は綴りが違うので比べない。
        value["sources"] = Value::Null;
        value
    };
    let (a, b) = (ir(with), ir(without));
    assert_eq!(a["plan"], b["plan"]);
    assert_eq!(a["analysis"], b["analysis"]);
    assert_eq!(
        a["semantics"]["rules"][0]["body"],
        b["semantics"]["rules"][0]["body"]
    );
}

#[test]
fn angle_templates_still_need_ubnf_2() {
    let codes: Vec<_> = rejection("grammar G { Root ::= Q<'a'> ; Q<x> ::= x ; }")
        .into_iter()
        .map(|(c, _)| c)
        .collect();
    assert!(
        codes.contains(&"E-TEMPLATE-UNSUPPORTED".to_owned()),
        "{codes:?}"
    );
}
