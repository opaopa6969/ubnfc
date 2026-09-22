use std::fmt::Write;
use ubnfc_front::ir::*;

pub(crate) fn ident(name: &str) -> String {
    let mut s = String::from("g_");
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c);
        } else {
            write!(s, "_{:x}_", c as u32).unwrap();
        }
    }
    s
}
pub(crate) fn type_ident(ty: &AstType) -> String {
    ident(ty.id())
}
pub(crate) fn nullable(t: &ValueType) -> bool {
    matches!(t, ValueType::Null {} | ValueType::Optional { .. })
        || matches!(t,ValueType::Mixed{members} if members.iter().any(nullable))
}
pub(crate) fn text_type(t: &ValueType) -> bool {
    matches!(t, ValueType::Text {} | ValueType::Number { .. })
}
pub(crate) fn field_type(f: &AstField) -> String {
    let base = if text_type(&f.value_type) {
        "String"
    } else {
        "Ast"
    };
    match f.cardinality {
        Cardinality::List => format!("Vec<{base}>"),
        Cardinality::Optional => {
            if base == "Ast" {
                "Option<Box<Ast>>".into()
            } else {
                "Option<String>".into()
            }
        }
        Cardinality::Scalar if nullable(&f.value_type) => "Option<Box<Ast>>".into(),
        Cardinality::Scalar if base == "Ast" => "Box<Ast>".into(),
        _ => base.into(),
    }
}
pub(crate) fn generate(ir: &GrammarIr) -> String {
    let mut s=String::from("//! 採用結果だけから構築する AST。数値 capture は字句 String。\n#![allow(non_camel_case_types, non_snake_case)]\nuse super::api::Span;\n#[derive(Clone,Debug,PartialEq)]\npub enum Ast { Text(String), Null,\n");
    for ty in &ir.semantics.ast_types {
        if !matches!(ty, AstType::TransparentUnion { .. }) {
            let n = type_ident(ty);
            writeln!(s, "{n}({n}),").unwrap();
        }
    }
    s.push_str("}\n");
    for ty in &ir.semantics.ast_types {
        let n = type_ident(ty);
        match ty {
            AstType::Record { fields, .. } => {
                writeln!(s,"#[derive(Clone,Debug,PartialEq)] pub struct {n} {{ pub span:Span, pub node_id:usize,").unwrap();
                for f in fields {
                    writeln!(s, "pub {}: {},", ident(&f.name), field_type(f)).unwrap();
                }
                s.push_str("}\n");
            }
            AstType::Enum { variants, .. } => {
                writeln!(s,"#[derive(Clone,Debug,PartialEq)] pub struct {n} {{ pub span:Span,pub node_id:usize,pub value:{n}Value }}\n#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum {n}Value {{").unwrap();
                for v in variants {
                    writeln!(s, "{},", ident(&v.rust_name)).unwrap();
                }
                s.push_str("}\n");
            }
            AstType::TransparentUnion { .. } => {
                writeln!(s, "pub type {n} = Ast;").unwrap();
            }
        }
    }
    // Rust は型内型を持たないので、enclosing type の名前空間を module で表現する。
    for outer in &ir.semantics.ast_types {
        let children: Vec<_> = ir
            .semantics
            .ast_types
            .iter()
            .filter(|t| t.enclosing_type_id().as_deref() == Some(outer.id()))
            .collect();
        if !children.is_empty() {
            writeln!(s, "pub mod {}_nested {{", type_ident(outer)).unwrap();
            for child in children {
                let name = match child {
                    AstType::Record { name, .. }
                    | AstType::Enum { name, .. }
                    | AstType::TransparentUnion { name, .. } => name,
                };
                writeln!(
                    s,
                    "pub use super::{} as {};",
                    type_ident(child),
                    ident(name)
                )
                .unwrap();
            }
            s.push_str("}\n");
        }
    }
    s.push_str("impl Ast { pub fn span(&self)->Option<Span> { match self {Self::Text(_)|Self::Null=>None,\n");
    for ty in &ir.semantics.ast_types {
        if !matches!(ty, AstType::TransparentUnion { .. }) {
            writeln!(s, "Self::{}(node)=>Some(node.span),", type_ident(ty)).unwrap();
        }
    }
    s.push_str("}}\n");
    s.push_str("pub fn type_name(&self)->&'static str { match self {Self::Text(_)=>\"text\",Self::Null=>\"null\",\n");
    for ty in &ir.semantics.ast_types {
        match ty {
            AstType::Record { name, .. } | AstType::Enum { name, .. } => {
                writeln!(s, "Self::{}(_)=>{name:?},", type_ident(ty)).unwrap();
            }
            _ => {}
        }
    }
    s.push_str("}}\n");
    for (i, common) in ir.semantics.common_fields.iter().enumerate() {
        let field = &common.field;
        writeln!(
            s,
            "pub fn common_{i}_{}(&self)->Option<&{}> {{ match self {{",
            ident(&field.name),
            field_type(field)
        )
        .unwrap();
        for id in &common.variant_type_ids {
            writeln!(
                s,
                "Self::{}(node)=>Some(&node.{}),",
                ident(id),
                ident(&field.name)
            )
            .unwrap();
        }
        s.push_str("_=>None,}}\n");
    }
    s.push_str("#[cfg(feature=\"json\")] pub fn canonical_value(&self)->serde_json::Value { match self { Self::Text(value)=>serde_json::Value::String(value.clone()),Self::Null=>serde_json::Value::Null,\n");
    for ty in &ir.semantics.ast_types {
        match ty {
            AstType::Record { fields, name, .. } => {
                writeln!(
                    s,
                    "Self::{}(node)=>serde_json::json!({{\"fields\":{{",
                    type_ident(ty)
                )
                .unwrap();
                let mut canonical_fields = fields.iter().collect::<Vec<_>>();
                canonical_fields.sort_by(|a, b| a.name.cmp(&b.name));
                for f in canonical_fields {
                    let field = format!("node.{}", ident(&f.name));
                    let v = if text_type(&f.value_type) {
                        field
                    } else {
                        match f.cardinality {
                            Cardinality::List => format!(
                                "{field}.iter().map(Ast::canonical_value).collect::<Vec<_>>()"
                            ),
                            Cardinality::Optional => {
                                format!("{field}.as_ref().map(|v|v.canonical_value())")
                            }
                            Cardinality::Scalar if nullable(&f.value_type) => {
                                format!("{field}.as_ref().map(|v|v.canonical_value())")
                            }
                            _ => format!("{field}.canonical_value()"),
                        }
                    };
                    writeln!(s, "{:?}:{v},", f.name).unwrap();
                }
                writeln!(s, "}},\"span\":node.span,\"type\":{name:?}}}),").unwrap();
            }
            AstType::Enum { variants, .. } => {
                let n = type_ident(ty);
                writeln!(
                    s,
                    "Self::{n}(node)=>serde_json::Value::String(match node.value {{"
                )
                .unwrap();
                for v in variants {
                    writeln!(s, "{n}Value::{}=>{:?},", ident(&v.rust_name), v.variant).unwrap();
                }
                s.push_str("}.into()),\n");
            }
            _ => {}
        }
    }
    s.push_str("}}\n#[cfg(feature=\"json\")] pub fn canonical_json(&self)->String {serde_json::to_string(&self.canonical_value()).expect(\"finite canonical AST\")}\n}\n#[cfg(feature=\"json\")] impl serde::Serialize for Ast {fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error> {self.canonical_value().serialize(serializer)}}\n");
    s
}
