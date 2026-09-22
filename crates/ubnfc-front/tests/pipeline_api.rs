use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use ubnfc_front::ir::{self, Cardinality, ExprNode, RequiredFeature};
use ubnfc_front::pipeline::{compile, compile_with_strict, Compiled, PipelineError};

fn source(text: &str) -> Result<Compiled, PipelineError> {
    compile("/virtual/main.ubnf", &|path| {
        assert_eq!(path, Path::new("/virtual/main.ubnf"));
        Ok(text.into())
    })
}

#[test]
fn loader_reads_each_normalized_file_once_and_preserves_import_order() {
    let files = BTreeMap::from([
        (PathBuf::from("/virtual/main.ubnf"), "grammar M { @import a from './lib.ubnf' @import b from 'sub/../lib.ubnf' @root Root ::= a.R b.R; }"),
        (PathBuf::from("/virtual/lib.ubnf"), "grammar L { R ::= 'x'; }"),
    ]);
    let calls = RefCell::new(Vec::new());
    let compiled = compile("/virtual/./main.ubnf", &|path| {
        calls.borrow_mut().push(path.to_path_buf());
        Ok(files[path].into())
    })
    .unwrap();
    assert_eq!(
        *calls.borrow(),
        [
            PathBuf::from("/virtual/main.ubnf"),
            PathBuf::from("/virtual/lib.ubnf")
        ]
    );
    assert_eq!(
        compiled
            .ir
            .import_edges
            .iter()
            .map(|e| e.alias.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(
        compiled
            .ir
            .semantics
            .rules
            .iter()
            .map(|r| r.symbol_id.as_str())
            .collect::<Vec<_>>(),
        ["M::Root", "M::a.R", "M::b.R"]
    );
    assert!(compiled
        .ir
        .required_features
        .contains(&RequiredFeature::Imports));
    assert!(ir::validate_references(&compiled.ir).is_empty());
}

#[test]
fn syntax_resolve_validation_and_lower_failures_are_distinct() {
    assert!(matches!(
        compile("/virtual/missing.ubnf", &|_| Err(
            io::ErrorKind::NotFound.into()
        )),
        Err(PipelineError::Io { .. })
    ));
    assert!(matches!(
        source("grammar G {"),
        Err(PipelineError::Syntax { .. })
    ));
    assert!(matches!(
        source("grammar G { @root R ::= Missing; }"),
        Err(PipelineError::Resolve(_))
    ));
    let Err(PipelineError::Validation(issues)) =
        source("grammar G { @mapping(Node, params=[missing]) R ::= 'x'; }")
    else {
        panic!("validation errors must stop before lower");
    };
    assert!(issues.iter().any(|i| i.severity() == "ERROR"));
    assert!(issues.iter().any(|i| i.severity() == "WARNING"));
    assert!(matches!(
        source("grammar G { @root A ::= B; B ::= A | 'x'; }"),
        Err(PipelineError::Lower(_))
    ));
    let error = compile("/virtual/main.ubnf", &|path| {
        Ok(if path.ends_with("main.ubnf") {
            "grammar G { @import x from 'lib.ubnf' @root R ::= x.R; }"
        } else {
            "grammar L {"
        }
        .into())
    })
    .unwrap_err();
    let PipelineError::Resolve(issues) = error else {
        panic!("import syntax is a resolver diagnostic");
    };
    assert_eq!(issues[0].code, "E-IMPORT-SYNTAX");
}

#[test]
fn warning_collection_and_strict_include_lower_suffix_diagnostics() {
    let text = "grammar G { @root R ::= 'a'+{2,3} % ',' @v; }";
    let compiled = source(text).unwrap();
    assert_eq!(compiled.warnings.len(), 1);
    assert_eq!(compiled.warnings[0].code, "W-LOWER-SUFFIX-PRIORITY");
    assert!(compiled
        .ir
        .semantics
        .expressions
        .iter()
        .all(|e| !matches!(&e.node, ExprNode::Literal { value, .. } if value == ",")));
    assert!(matches!(
        compile_with_strict("/virtual/main.ubnf", &|_| Ok(text.into()), true),
        Err(PipelineError::Strict(_))
    ));
}

#[test]
fn auto_recovery_follow_is_integrated_without_changing_sync_or_skip() {
    let compiled = source("grammar G { @root R ::= Auto 'end' Sync Skip; @recovery(AUTO) Auto ::= 'a'; @recovery(sync='!') Sync ::= 'b'; @recovery(SKIP) Skip ::= 'c'; }").unwrap();
    let auto = compiled
        .ir
        .semantics
        .recovery
        .iter()
        .find(|r| r.rule_id == "G::Auto")
        .unwrap();
    assert_eq!(auto.resolved_sync_patterns, ["end"]);
    let analyzed = compiled
        .ir
        .analysis
        .rules
        .iter()
        .find(|r| r.rule_id == "G::Auto")
        .unwrap();
    assert_eq!(
        analyzed.follow_sync_patterns.as_ref().unwrap(),
        &auto.resolved_sync_patterns
    );
    let sync = compiled
        .ir
        .semantics
        .recovery
        .iter()
        .find(|r| r.rule_id == "G::Sync")
        .unwrap();
    assert_eq!(sync.resolved_sync_patterns, ["!"]);
    let skip = compiled
        .ir
        .semantics
        .recovery
        .iter()
        .find(|r| r.rule_id == "G::Skip")
        .unwrap();
    assert!(skip.resolved_sync_patterns.is_empty());
    let fallback = source("grammar G { @root R ::= A; @recovery(AUTO) A ::= 'a'; }").unwrap();
    assert_eq!(
        fallback.ir.semantics.recovery[0].resolved_sync_patterns,
        [";"]
    );
}

#[test]
fn separated_capture_bindings_survive_serialization_and_keep_mapping_references() {
    let compiled =
        source("grammar G { @root @mapping(Node,params=[list,v]) Root ::= ('a' @v) % ',' @list; }")
            .unwrap();
    let s = &compiled.ir.semantics;
    assert!(s
        .captures
        .iter()
        .all(|c| c.cardinality == Cardinality::List));
    let list = s.captures.iter().find(|c| c.name == "list").unwrap();
    let owner = s.expressions.iter().find(|e| e.id == list.expr_id).unwrap();
    assert!(matches!(owner.node, ExprNode::Group { .. }));
    // 同じ element が先頭と pair の両方から参照され、各完了時に一度 capture する。
    assert_eq!(
        s.expressions
            .iter()
            .filter(|e| e.node.children().contains(&&list.expr_id))
            .count(),
        2
    );
    assert!(s
        .recipes
        .iter()
        .flat_map(|r| &r.fields)
        .any(|f| f.site_ids.contains(&list.site_id)));
    assert!(ir::validate_references(&compiled.ir).is_empty());
    let json = ir::to_canonical_json(&compiled.ir);
    assert_eq!(ir::from_json(&json).unwrap(), compiled.ir);
}

#[test]
fn last_grammar_is_entry_and_features_are_unique_and_complete() {
    let compiled =
        source("grammar A { @root R ::= 'a'; } grammar B { @root R ::= 'b' | 'c'; }").unwrap();
    assert_eq!(compiled.ir.entry.grammar_id, "grammar:B");
    assert_eq!(compiled.ir.entry.rule_id, "B::R");
    assert_eq!(compiled.ir.grammar_units.len(), 2);
    assert!(compiled
        .ir
        .required_features
        .contains(&RequiredFeature::PredictiveChoice));
    assert!(compiled
        .ir
        .required_features
        .contains(&RequiredFeature::OrderedChoice));
    let names: Vec<_> = compiled
        .ir
        .required_features
        .iter()
        .map(|f| serde_json::to_string(f).unwrap())
        .collect();
    assert!(names.windows(2).all(|w| w[0] < w[1]));
    assert!(ir::validate_references(&compiled.ir).is_empty());
}
