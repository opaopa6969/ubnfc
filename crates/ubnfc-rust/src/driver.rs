use crate::ast::{ident, type_ident};
use std::fmt::Write;
use ubnfc_front::ir::{AstType, GrammarIr, TokenSpec};

pub(crate) const MANIFEST: &str = r#"[package]
name = "ubnfc-generated"
version = "0.0.0"
edition = "2021"
autobins = false
[workspace]
[lib]
path = "mod.rs"
[features]
default = []
scanners = []
json = ["dep:serde", "dep:serde_json"]
[dependencies]
serde = { version = "1", optional = true }
serde_json = { version = "1", optional = true }
"#;
pub(crate) const BIN: &str = "\n[[bin]]\nname = \"driver\"\npath = \"src/bin/driver.rs\"\n";

pub(crate) fn generate(ir: &GrammarIr) -> String {
    let mut s = include_str!("../templates/driver.rs").to_owned();
    scanner_hook(ir, &mut s);
    s.push_str("\nimpl ToJson for Ast { fn to_json(&self)->Json { match self { Self::Text(s)=>s.to_json(),Self::Null=>Json::Null,\n");
    for ty in &ir.semantics.ast_types {
        let n = type_ident(ty);
        match ty {
            AstType::Record { fields, name, .. } => {
                writeln!(s, "Self::{n}(node)=>Json::object([(\"type\", {name:?}.to_json()),(\"span\",node.span.to_json()),(\"fields\",Json::object(vec![").unwrap();
                for field in fields {
                    writeln!(
                        s,
                        "({:?},node.{}.to_json()),",
                        field.name,
                        ident(&field.name)
                    )
                    .unwrap();
                }
                // 空 record でも key 型が確定するようにする。
                s.push_str("] as Vec<(&str,Json)>))]),\n");
            }
            AstType::Enum { variants, .. } => {
                writeln!(s, "Self::{n}(node)=>match node.value {{").unwrap();
                let mut seen = std::collections::BTreeSet::new();
                for v in variants.iter().filter(|v| seen.insert(&v.rust_name)) {
                    writeln!(
                        s,
                        "{n}Value::{}=>{:?}.to_json(),",
                        ident(&v.rust_name),
                        v.variant
                    )
                    .unwrap();
                }
                s.push_str("},\n");
            }
            AstType::TransparentUnion { .. } => {}
        }
    }
    s.push_str("}}}\nfn operator_ast(ast:&Ast)->bool {\n");
    let mut types = std::collections::BTreeSet::new();
    for rule in &ir.semantics.rules {
        if rule.operator.is_some() {
            for recipe in &ir.semantics.recipes {
                if recipe.rule_id == rule.symbol_id {
                    if let Some(id) = &recipe.result_type_id {
                        types.insert(id);
                    }
                }
            }
        }
    }
    if types.is_empty() {
        s.push_str("let _=ast;false\n}\n");
    } else {
        let patterns = types
            .iter()
            .map(|id| format!("Ast::{}(_)", ident(id)))
            .collect::<Vec<_>>();
        writeln!(s, "matches!(ast,{})\n}}", patterns.join("|")).unwrap();
    }
    s.push_str("const CAPTURE_BINDINGS: &[(&str,&str)] = &[\n");
    let mut counts = std::collections::HashMap::<&str, usize>::new();
    for capture in &ir.semantics.captures {
        let rule = ir
            .semantics
            .symbols
            .iter()
            .find(|s| s.id == capture.rule_id)
            .unwrap();
        let count = counts.entry(&capture.rule_id).or_default();
        writeln!(
            s,
            "({:?},{:?}),",
            capture.site_id,
            format!("{}:{}", rule.name, count)
        )
        .unwrap();
        *count += 1;
    }
    s.push_str("];\n");
    s
}

// driver だけに scanner 対応の entry を追加する。公開 parser API と実行意味論は変更しない。
// 同じ emit 関数の parser を利用し、entry/root 選択を別実装にしない。
fn scanner_hook(ir: &GrammarIr, out: &mut String) {
    let generated = crate::emit::generate(
        ir,
        &crate::EmitOptions {
            documentation: None,
            emit_driver: false,
        },
    )
    .expect("検証済み IR の driver scanner hook");
    let parser = &generated.files["parser.rs"];
    let start = parser
        .find("pub fn parse_entry_with_options(")
        .expect("entry API");
    let end = start + parser[start..].find("\nimpl Session").expect("entry 終端");
    let entry = parser[start..end]
        .replacen("parse_entry_with_options(", "parse_entry_with_scanner(", 1)
        .replacen(
            "options:ParseOptions)",
            "options:ParseOptions,scanner:&mut dyn TokenScanner)",
            1,
        )
        .replacen("let mut scanner=RejectExtern;", "", 1)
        .replacen("&mut scanner", "scanner", 1);
    out.push_str("\n#[cfg(feature=\"scanners\")] use ubnfc_generated::{api, ast, rt};\n#[cfg(feature=\"scanners\")] #[path=\"../../runtime.rs\"] #[allow(dead_code)] mod runtime;\n#[cfg(feature=\"scanners\")] mod scanner_parser {\n");
    out.push_str(parser);
    out.push_str(&entry);
    out.push_str("\n}\nconst EXTERN_CLASSES: &[(&str,&str)] = &[\n");
    for token in &ir.semantics.tokens {
        if let TokenSpec::Extern {
            logical_id,
            original_class_name,
            ..
        } = &token.spec
        {
            writeln!(out, "({logical_id:?},{original_class_name:?}),").unwrap();
        }
    }
    out.push_str("];\n");
}
