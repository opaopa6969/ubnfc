use crate::ast::{ident, type_ident};
use std::fmt::Write;
use ubnfc_front::ir::{AstType, GrammarIr};

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
json = ["dep:serde", "dep:serde_json"]
[dependencies]
serde = { version = "1", optional = true }
serde_json = { version = "1", optional = true }
"#;
pub(crate) const BIN: &str = "\n[[bin]]\nname = \"driver\"\npath = \"src/bin/driver.rs\"\n";

pub(crate) fn generate(ir: &GrammarIr) -> String {
    let mut s = include_str!("../templates/driver.rs").to_owned();
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
                for v in variants {
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
    s
}
