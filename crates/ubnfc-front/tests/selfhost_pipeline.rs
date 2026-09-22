use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ubnfc_front::ir;
use ubnfc_front::pipeline::{self, ParsedSource};
use ubnfc_front::syntax;

#[test]
fn parsed_main_and_imports_produce_the_existing_compile_result() {
    let files = BTreeMap::from([
        (
            PathBuf::from("/virtual/main.ubnf"),
            "grammar M { @import lib from 'lib.ubnf' @root R ::= lib.R; }",
        ),
        (
            PathBuf::from("/virtual/lib.ubnf"),
            "grammar L { R ::= 'x'; }",
        ),
    ]);
    let expected = pipeline::compile("/virtual/main.ubnf", &|path| Ok(files[path].into())).unwrap();
    let main = parsed(files[Path::new("/virtual/main.ubnf")]);
    let actual =
        pipeline::compile_parsed("/virtual/main.ubnf", main, &|path| Ok(parsed(files[path])))
            .unwrap();
    assert_eq!(
        ir::to_canonical_json(&actual.ir),
        ir::to_canonical_json(&expected.ir)
    );
    assert_eq!(actual.warnings, expected.warnings);
}

fn parsed(source: &str) -> ParsedSource {
    ParsedSource::new(source, syntax::parse_file(source).unwrap())
}
