use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use ubnfc_front::{ir::TokenSpec, pipeline::compile};

fn source(
    text: &str,
) -> Result<ubnfc_front::pipeline::Compiled, ubnfc_front::pipeline::PipelineError> {
    compile("/virtual/main.ubnf", &|_| Ok(text.into()))
}
#[test]
fn lexical_programs_are_versioned_closed_and_validated() {
    for version in ["2", "v2"] {
        let text=format!("grammar G {{ @ubnf: {version} token D ::= CHAR_RANGE('0','9'); token N ::= D+; @root R ::= N; }}");
        let compiled = source(&text).unwrap();
        assert_eq!(compiled.ir.language_version, 2);
        for token in &compiled.ir.semantics.tokens {
            let TokenSpec::Declarative { expression } = &token.spec else {
                panic!("declarative token")
            };
            expression.validate().unwrap();
        }
    }
    for expression in [
        "MISSING",
        "T",
        "SAME_AS(x)",
        "[CAPTURE(x,'a')] SAME_AS(x)",
        "LOOKAHEAD(CAPTURE(x,'a')) SAME_AS(x)",
        "{EOF}",
        "(CAPTURE(x,'a') | 'b') SAME_AS(x)",
    ] {
        let text = format!("grammar G {{ @ubnf: v2 token T ::= {expression}; @root R ::= T; }}");
        let error = source(&text).unwrap_err().to_string();
        assert!(error.contains("E-LEXICAL"), "{expression}: {error}");
    }
    assert!(source("grammar G { token T ::= 'a'; @root R ::= T; }")
        .unwrap_err()
        .to_string()
        .contains("E-LEXICAL"));
}

#[test]
fn invalid_shared_module_graphs_do_not_generate_ir() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/spec-corpus/lexical-modules/invalid.json"
    ))
    .unwrap();
    for row in fixtures.as_array().unwrap() {
        // The established ubnfc general-rule import is broader than the new lexical-only core loader.
        if matches!(
            row["name"].as_str().unwrap(),
            "rule-module" | "host-parser-module"
        ) {
            continue;
        }
        let files: BTreeMap<PathBuf, String> = row["files"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, text)| {
                (
                    Path::new("/virtual").join(name),
                    text.as_str().unwrap().into(),
                )
            })
            .collect();
        let result = compile("/virtual/root.ubnf", &|p: &Path| {
            files
                .get(p)
                .cloned()
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
        });
        assert!(result.is_err(), "{}", row["name"]);
    }
}

#[test]
fn imported_reference_resolution_is_lexical_not_caller_based() {
    let files: BTreeMap<PathBuf, String> = BTreeMap::from([
        (
            "/virtual/main.ubnf".into(),
            "grammar G { @import n from 'lib.ubnf' @ubnf: v2 token D ::= 'x'; @root R ::= n.N; }"
                .into(),
        ),
        (
            "/virtual/lib.ubnf".into(),
            "grammar N { @ubnf: v2 token D ::= '0'; token N ::= D+; }".into(),
        ),
    ]);
    let compiled = compile("/virtual/main.ubnf", &|p: &Path| Ok(files[p].clone())).unwrap();
    let n = compiled
        .ir
        .semantics
        .tokens
        .iter()
        .find(|t| t.symbol_id.ends_with("::n.N"))
        .unwrap();
    let json = serde_json::to_string(&n.spec).unwrap();
    assert!(json.contains("\"text\":\"0\""), "{json}");
    assert!(!json.contains("\"text\":\"x\""), "{json}");
}
