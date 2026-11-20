//! `@import alias from 'pkg:ID'` と `@whitespace: alias.SPACES_AND_COMMENTS`（Classic の明示
//! package 依存、unlaxer-parser docs/ubnf-packages.md）。tinyexpression P4 がこの綴りを使う。

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;
use ubnfc_front::pipeline::{compile, validate_file, PipelineError};

const STD_SHA: &str = "2701f6884ea6c712d28c76ec3d340c3b27fca4575846239bd0854933205f4b67";

const MANIFEST: &str = r#"{"schemaVersion":1,"dependencies":{"std/layout":{"version":"1.0.0","source":"builtin:std/layout@1.0.0"}}}"#;

fn lock() -> String {
    format!(
        r#"{{"packages":{{"std/layout":{{"dependencies":{{}},"id":"std/layout","sha256":"{STD_SHA}","version":"1.0.0"}}}},"roots":{{"std/layout":"1.0.0"}},"schemaVersion":1}}"#
    )
}

fn loader(files: BTreeMap<PathBuf, String>) -> impl Fn(&Path) -> io::Result<String> {
    move |path| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.display().to_string()))
    }
}

fn project(main: &str) -> BTreeMap<PathBuf, String> {
    BTreeMap::from([
        (PathBuf::from("/p/main.ubnf"), main.to_owned()),
        (PathBuf::from("/p/ubnf.json"), MANIFEST.to_owned()),
        (PathBuf::from("/p/ubnf.lock.json"), lock()),
    ])
}

/// IR 全体から `effectiveDelimiterSet` を集める（規則ごとの trivia の集合）。
fn delimiter_sets(value: &Value, out: &mut Vec<Value>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if key == "effectiveDelimiterSet" {
                    out.push(child.clone());
                }
                delimiter_sets(child, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|i| delimiter_sets(i, out)),
        _ => {}
    }
}

fn sets(main: &str) -> Vec<Value> {
    let compiled = compile("/p/main.ubnf", &loader(project(main))).unwrap();
    let mut out = Vec::new();
    delimiter_sets(&serde_json::to_value(&compiled.ir).unwrap(), &mut out);
    out
}

const BODY: &str = "@root @mapping(Root, params=[a, b]) Root ::= 'x' @a R @b ; @whitespace(none) R ::= 'y' 'z' ; }";

#[test]
fn std_layout_spaces_and_comments_lowers_to_the_java_style_delimiter_set() {
    let packaged = sets(&format!(
        "grammar G {{ @import layout from 'pkg:std/layout' @whitespace: layout.SPACES_AND_COMMENTS {BODY}"
    ));
    let java = sets(&format!("grammar G {{ @whitespace: javaStyle {BODY}"));
    assert!(!packaged.is_empty());
    assert_eq!(packaged, java);
    // 規則の注釈でも同じ。
    let rule = sets(
        "grammar G { @import layout from 'pkg:std/layout' \
         @root @whitespace(layout.SPACES_AND_COMMENTS) @mapping(Root, params=[a]) Root ::= 'x' @a 'y' ; }",
    );
    let java = sets("grammar G { @root @whitespace(javaStyle) @mapping(Root, params=[a]) Root ::= 'x' @a 'y' ; }");
    assert_eq!(rule, java);
}

#[test]
fn the_package_entry_is_recorded_as_a_hash_checked_source() {
    let compiled = compile(
        "/p/main.ubnf",
        &loader(project(&format!(
            "grammar G {{ @import layout from 'pkg:std/layout' @whitespace: layout.SPACES_AND_COMMENTS {BODY}"
        ))),
    )
    .unwrap();
    let paths: Vec<_> = compiled
        .ir
        .sources
        .iter()
        .map(|s| s.path.as_str())
        .collect();
    assert_eq!(paths, ["main.ubnf", ".ubnf-packages/std/layout"]);
}

#[test]
fn other_declarative_tokens_are_lexical_profiles_without_java_style_substitution() {
    for (import, token, module) in [
        ("pkg:std/layout", "SPACES", None),
        (
            "layout.ubnf",
            "SPACES_AND_COMMENTS",
            Some("grammar L { @ubnf: v2 token SPACES_AND_COMMENTS ::= ' '+; }"),
        ),
    ] {
        let mut files = project(&format!(
            "grammar G {{ @import layout from '{import}' @whitespace: layout.{token} {BODY}"
        ));
        if let Some(module) = module {
            files.insert(PathBuf::from("/p/layout.ubnf"), module.into());
        }
        let ir = compile("/p/main.ubnf", &loader(files)).unwrap().ir;
        let delimiters = &ir.semantics.trivia.policies[0].effective_delimiter_set;
        assert!(matches!(
            &delimiters[..],
            [ubnfc_front::ir::Delimiter::Lexical { .. }]
        ));
    }
}

#[test]
fn a_missing_lock_is_a_positioned_unresolved_import() {
    let mut files = project("grammar G { @import layout from 'pkg:std/layout' @root R ::= 'x' ; }");
    files.remove(Path::new("/p/ubnf.lock.json"));
    let Err(PipelineError::Resolve(errors)) = validate_file("/p/main.ubnf", &loader(files)) else {
        panic!("lock が無ければ解決できない");
    };
    assert_eq!(errors[0].code, "E-IMPORT-UNRESOLVED");
    assert!(
        errors[0].hint.contains("unlaxer deps resolve"),
        "{}",
        errors[0].hint
    );
    assert_eq!(errors[0].span.start, "grammar G { ".len());
}
