//! D-077: arena 形の AST（`ast::tree`）の生成。
//!
//! 型付き節点・Text 節点を `AstTree` の `Vec` に積み、所有 `Ast` はこの木から写す
//! （`project`）。汎用部分は `templates/tree.rs`、文法ごとの部分（節点の種類・型付きの
//! 参照型・写像・canonical JSON・value span の走査）をここで作る。
use crate::{
    ast::{ident, nullable, text_type, type_ident},
    validate::unsupported,
    EmitError,
};
use std::fmt::Write;
use ubnfc_front::ir::*;

/// record field の表現（所有 `Ast` の `field_type` と 1 対 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Repr {
    /// `String`（1 slot、Text 節点）
    Text,
    /// `Option<String>`（1 slot、空は NONE）
    OptText,
    /// `Vec<String>`（2 slot）
    TextList,
    /// `Box<Ast>`（1 slot）
    Node,
    /// `Option<Box<Ast>>`（1 slot、空は NONE）
    OptNode,
    /// `Vec<Ast>`（2 slot）
    NodeList,
}
impl Repr {
    pub(crate) fn of(f: &AstField) -> Self {
        let text = text_type(&f.value_type);
        match f.cardinality {
            Cardinality::List if text => Self::TextList,
            Cardinality::List => Self::NodeList,
            Cardinality::Optional if text => Self::OptText,
            Cardinality::Optional => Self::OptNode,
            Cardinality::Scalar if nullable(&f.value_type) => Self::OptNode,
            Cardinality::Scalar if text => Self::Text,
            Cardinality::Scalar => Self::Node,
        }
    }
    pub(crate) fn slots(self) -> usize {
        match self {
            Self::TextList | Self::NodeList => 2,
            _ => 1,
        }
    }
}

/// 型付き節点の種類番号（`KIND_TEXT` / `KIND_NULL` の後ろ、透過 union を除く宣言順）。
pub(crate) fn kinds(ir: &GrammarIr) -> Result<Vec<(usize, u16)>, EmitError> {
    let mut out = Vec::new();
    let mut next = 2usize;
    for (index, ty) in ir.semantics.ast_types.iter().enumerate() {
        if matches!(ty, AstType::TransparentUnion { .. }) {
            continue;
        }
        let kind = u16::try_from(next)
            .map_err(|_| unsupported("ast.tree", "more than 65533 AST node types"))?;
        out.push((index, kind));
        next += 1;
    }
    if ir.semantics.rules.len() >= u16::MAX as usize {
        return Err(unsupported("ast.tree", "more than 65535 rules"));
    }
    Ok(out)
}
/// 種類番号の定数名。
pub(crate) fn kind_const(ty: &AstType) -> String {
    format!("K_{}", type_ident(ty))
}
/// enum 型の variant（Rust 名で重複を除いた宣言順。番号が `TreeNode::a` に入る）。
pub(crate) fn enum_variants(variants: &[EnumVariant]) -> Vec<&EnumVariant> {
    let mut seen = std::collections::BTreeSet::new();
    variants
        .iter()
        .filter(|v| seen.insert(&v.rust_name))
        .collect()
}

pub(crate) fn generate(ir: &GrammarIr) -> Result<String, EmitError> {
    let kinds = kinds(ir)?;
    let types = &ir.semantics.ast_types;
    let mut s = String::from("/// D-077: arena 形の AST。所有 `Ast` はこの木から写す。\npub mod tree {\n#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]\n");
    s.push_str(include_str!("../templates/tree.rs"));
    for &(index, kind) in &kinds {
        writeln!(s, "pub const {}: u16 = {kind};", kind_const(&types[index])).unwrap();
    }
    let mut ids = vec!["\"text\"".to_owned(), "\"null\"".to_owned()];
    let mut names = ids.clone();
    for &(index, _) in &kinds {
        let ty = &types[index];
        ids.push(format!("{:?}", ty.id()));
        names.push(match ty {
            AstType::Record { name, .. } | AstType::Enum { name, .. } => format!("{name:?}"),
            AstType::TransparentUnion { .. } => unreachable!(),
        });
    }
    writeln!(
        s,
        "pub(crate) const TYPE_IDS: &[&str] = &[{}];",
        ids.join(",")
    )
    .unwrap();
    writeln!(
        s,
        "pub(crate) const TYPE_NAMES: &[&str] = &[{}];",
        names.join(",")
    )
    .unwrap();
    writeln!(
        s,
        "pub(crate) const RULE_IDS: &[&str] = &{:?};",
        ir.semantics
            .rules
            .iter()
            .map(|r| r.symbol_id.as_str())
            .collect::<Vec<_>>()
    )
    .unwrap();
    // 型付きの参照（所有 `Ast` と同じ形の列挙）。
    s.push_str("/// `super::Ast` と同じ形の、木への参照。\n#[derive(Clone, Copy, Debug)]\npub enum Node<'t> { Text(&'t str), Null,\n");
    for &(index, _) in &kinds {
        let n = type_ident(&types[index]);
        writeln!(s, "{n}({n}<'t>),").unwrap();
    }
    s.push_str("}\nimpl<'t> NodeRef<'t> {\n/// 型ごとの参照へ分ける。\npub fn get(&self) -> Node<'t> { let (tree, id) = (self.tree, self.id); match tree.kind(id) { KIND_TEXT => Node::Text(tree.text_in(&tree.source, id)), KIND_NULL => Node::Null,\n");
    for &(index, _) in &kinds {
        let ty = &types[index];
        let n = type_ident(ty);
        writeln!(s, "{} => Node::{n}({n} {{ tree, id }}),", kind_const(ty)).unwrap();
    }
    s.push_str("_ => unreachable!(\"unknown AST node kind\"), } }\n}\n");
    for &(index, _) in &kinds {
        view(&mut s, &types[index]);
    }
    projection(&mut s, types, &kinds);
    value_spans(&mut s, types, &kinds);
    canonical(&mut s, types, &kinds);
    s.push_str("}\n");
    Ok(s)
}

fn view(s: &mut String, ty: &AstType) {
    let n = type_ident(ty);
    writeln!(
        s,
        "#[derive(Clone, Copy)] pub struct {n}<'t> {{ tree: &'t AstTree, id: u32 }}"
    )
    .unwrap();
    writeln!(s, "impl std::fmt::Debug for {n}<'_> {{ fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{ f.debug_struct({:?}).field(\"id\", &self.id).finish() }} }}", n).unwrap();
    writeln!(s, "impl<'t> {n}<'t> {{\npub fn node(&self) -> NodeRef<'t> {{ NodeRef {{ tree: self.tree, id: self.id }} }}\npub fn span(&self) -> Span {{ span_of(&self.tree.nodes[self.id as usize]) }}\npub fn node_id(&self) -> usize {{ self.tree.nodes[self.id as usize].node_id as usize }}\npub fn to_ast(&self) -> super::{n} {{ self.tree.project_{n}(&self.tree.source, &self.tree.nodes[self.id as usize]) }}").unwrap();
    match ty {
        AstType::Record { fields, .. } => {
            let mut offset = 0;
            for f in fields {
                let name = ident(&f.name);
                let repr = Repr::of(f);
                let body = match repr {
                    Repr::Text => format!("-> &'t str {{ let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, {offset})) }}"),
                    Repr::OptText => format!("-> Option<&'t str> {{ let tree = self.tree; let v = tree.slot(self.id, {offset}); (v != NONE).then(|| tree.text_in(&tree.source, v)) }}"),
                    Repr::TextList => format!("-> TextList<'t> {{ TextList {{ tree: self.tree, ids: self.tree.items(self.id, {offset}) }} }}"),
                    Repr::Node => format!("-> NodeRef<'t> {{ NodeRef {{ tree: self.tree, id: self.tree.slot(self.id, {offset}) }} }}"),
                    Repr::OptNode => format!("-> Option<NodeRef<'t>> {{ let v = self.tree.slot(self.id, {offset}); (v != NONE).then_some(NodeRef {{ tree: self.tree, id: v }}) }}"),
                    Repr::NodeList => format!("-> NodeList<'t> {{ NodeList {{ tree: self.tree, ids: self.tree.items(self.id, {offset}) }} }}"),
                };
                writeln!(s, "pub fn {name}(&self) {body}").unwrap();
                offset += repr.slots();
            }
        }
        AstType::Enum { .. } => {
            writeln!(s, "pub fn value(&self) -> super::{n}Value {{ {n}_VALUES[self.tree.nodes[self.id as usize].a as usize] }}").unwrap();
        }
        AstType::TransparentUnion { .. } => unreachable!(),
    }
    s.push_str("}\n");
    if let AstType::Enum { variants, .. } = ty {
        let values = enum_variants(variants)
            .iter()
            .map(|v| format!("super::{n}Value::{}", ident(&v.rust_name)))
            .collect::<Vec<_>>();
        writeln!(
            s,
            "pub(crate) const {n}_VALUES: &[super::{n}Value] = &[{}];",
            values.join(",")
        )
        .unwrap();
    }
}

/// 所有 `Ast` への写像。構築の唯一の経路（`ParseResult::ast` もこれで作る）。
fn projection(s: &mut String, types: &[AstType], kinds: &[(usize, u16)]) {
    s.push_str("impl AstTree {\n/// 節点を所有 `Ast` へ写す。`source` は Text の byte 範囲を読む入力。\n#[doc(hidden)] pub fn project(&self, source: &str, id: u32) -> super::Ast { let node = self.nodes[id as usize]; match node.kind { KIND_TEXT => super::Ast::Text(source[node.a as usize..node.b as usize].to_owned()), KIND_NULL => super::Ast::Null,\n");
    for &(index, _) in kinds {
        let ty = &types[index];
        let n = type_ident(ty);
        writeln!(
            s,
            "{} => super::Ast::{n}(self.project_{n}(source, &node)),",
            kind_const(ty)
        )
        .unwrap();
    }
    s.push_str("_ => unreachable!(\"unknown AST node kind\"), } }\n");
    for &(index, _) in kinds {
        let ty = &types[index];
        let n = type_ident(ty);
        // record の slot は 1 回の範囲検査で切り出し、定数添字で読む。
        let slots = match ty {
            AstType::Record { fields, .. } => fields.iter().map(|f| Repr::of(f).slots()).sum(),
            _ => 0,
        };
        let head = if slots > 0 {
            format!("let f = &self.slots[node.a as usize..node.a as usize + {slots}];")
        } else {
            "let _ = source;".to_owned()
        };
        writeln!(s, "fn project_{n}(&self, source: &str, node: &TreeNode) -> super::{n} {{ {head} super::{n} {{ span: span_of(node), node_id: node.node_id as usize,").unwrap();
        match ty {
            AstType::Record { fields, .. } => {
                let mut offset = 0;
                for f in fields {
                    let repr = Repr::of(f);
                    let next = offset + 1;
                    let value = match repr {
                        Repr::Text => format!("self.text_in(source, f[{offset}]).to_owned()"),
                        Repr::OptText => format!("{{ let v = f[{offset}]; (v != NONE).then(|| self.text_in(source, v).to_owned()) }}"),
                        Repr::TextList => format!("self.list_at(f[{offset}], f[{next}]).iter().map(|&v| self.text_in(source, v).to_owned()).collect()"),
                        Repr::Node => format!("Box::new(self.project(source, f[{offset}]))"),
                        Repr::OptNode => format!("{{ let v = f[{offset}]; (v != NONE).then(|| Box::new(self.project(source, v))) }}"),
                        Repr::NodeList => format!("self.list_at(f[{offset}], f[{next}]).iter().map(|&v| self.project(source, v)).collect()"),
                    };
                    writeln!(s, "{}: {value},", ident(&f.name)).unwrap();
                    offset += repr.slots();
                }
            }
            AstType::Enum { .. } => {
                writeln!(s, "value: {n}_VALUES[node.a as usize],").unwrap();
            }
            AstType::TransparentUnion { .. } => unreachable!(),
        }
        s.push_str("} }\n");
    }
    s.push_str("}\n");
}

/// value span（`ParseOptions::value_spans`）。所有 `Ast` の JSON pointer と同じ順で出す。
fn value_spans(s: &mut String, types: &[AstType], kinds: &[(usize, u16)]) {
    s.push_str("impl AstTree {\n#[doc(hidden)] pub fn collect_value_spans(&self, source: &str, id: u32, path: &mut String, out: &mut Vec<ValueSpan>) { let _ = &source; let node = self.nodes[id as usize]; match node.kind { KIND_TEXT => out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: Some(self.text_in(source, id).to_owned()) }), KIND_NULL => {},\n");
    for &(index, _) in kinds {
        let ty = &types[index];
        writeln!(
            s,
            "{} => {{ out.push(ValueSpan {{ path: path.clone(), span: span_of(&node), text: None }});",
            kind_const(ty)
        )
        .unwrap();
        if let AstType::Record { fields, .. } = ty {
            let mut offset = 0;
            for f in fields {
                let field = f.name.replace('~', "~0").replace('/', "~1");
                let repr = Repr::of(f);
                let walk = match repr {
                    Repr::Text => format!("self.text_value_span(self.slot(id, {offset}), path, out);"),
                    Repr::OptText => format!("let v = self.slot(id, {offset}); if v != NONE {{ self.text_value_span(v, path, out); }}"),
                    Repr::TextList => format!("for (i, &v) in self.items(id, {offset}).iter().enumerate() {{ let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!(\"/{{i}}\")); self.text_value_span(v, path, out); path.truncate(len); }}"),
                    Repr::Node => format!("self.collect_value_spans(source, self.slot(id, {offset}), path, out);"),
                    Repr::OptNode => format!("let v = self.slot(id, {offset}); if v != NONE {{ self.collect_value_spans(source, v, path, out); }}"),
                    Repr::NodeList => format!("for (i, &v) in self.items(id, {offset}).iter().enumerate() {{ let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!(\"/{{i}}\")); self.collect_value_spans(source, v, path, out); path.truncate(len); }}"),
                };
                writeln!(s, "{{ let len = path.len(); path.push_str(\"/fields/{field}\"); {walk} path.truncate(len); }}").unwrap();
                offset += repr.slots();
            }
        }
        s.push_str("},\n");
    }
    s.push_str("_ => unreachable!(\"unknown AST node kind\"), } }\n}\n");
}

/// canonical JSON（`json` feature）。所有 `Ast::canonical_value` と同じ値を木から直接作る。
fn canonical(s: &mut String, types: &[AstType], kinds: &[(usize, u16)]) {
    s.push_str("#[cfg(feature=\"json\")] impl AstTree {\n/// 根の canonical JSON（所有 `Ast::canonical_json` と byte 一致）。\npub fn canonical_json(&self) -> Option<String> { self.root().map(|root| serde_json::to_string(&root.canonical_value()).expect(\"finite canonical AST\")) }\n}\n#[cfg(feature=\"json\")] impl<'t> NodeRef<'t> {\npub fn canonical_value(&self) -> serde_json::Value { let (tree, id) = (self.tree, self.id); let node = tree.nodes[id as usize]; let text = |v: u32| tree.text_in(&tree.source, v); let value = |v: u32| NodeRef { tree, id: v }.canonical_value(); let _ = (&text, &value); match node.kind { KIND_TEXT => serde_json::Value::String(text(id).to_owned()), KIND_NULL => serde_json::Value::Null,\n");
    for &(index, _) in kinds {
        let ty = &types[index];
        let n = type_ident(ty);
        match ty {
            AstType::Record { fields, name, .. } => {
                writeln!(s, "{} => serde_json::json!({{\"fields\":{{", kind_const(ty)).unwrap();
                let mut offsets = Vec::new();
                let mut offset = 0;
                for f in fields {
                    offsets.push(offset);
                    offset += Repr::of(f).slots();
                }
                let mut sorted = fields.iter().zip(offsets).collect::<Vec<_>>();
                sorted.sort_by(|a, b| a.0.name.cmp(&b.0.name));
                for (f, offset) in sorted {
                    let v = match Repr::of(f) {
                        Repr::Text => format!("text(tree.slot(id, {offset}))"),
                        Repr::OptText => format!("{{ let v = tree.slot(id, {offset}); (v != NONE).then(|| text(v)) }}"),
                        Repr::TextList => format!("tree.items(id, {offset}).iter().map(|&v| text(v)).collect::<Vec<_>>()"),
                        Repr::Node => format!("value(tree.slot(id, {offset}))"),
                        Repr::OptNode => format!("{{ let v = tree.slot(id, {offset}); (v != NONE).then(|| value(v)) }}"),
                        Repr::NodeList => format!("tree.items(id, {offset}).iter().map(|&v| value(v)).collect::<Vec<_>>()"),
                    };
                    // json! は `{` で始まる値を object と読むので括弧で包む。
                    writeln!(s, "{:?}:({v}),", f.name).unwrap();
                }
                writeln!(s, "}},\"span\":span_of(&node),\"type\":{name:?}}}),").unwrap();
            }
            AstType::Enum { variants, .. } => {
                writeln!(
                    s,
                    "{} => serde_json::Value::String(match {n}_VALUES[node.a as usize] {{",
                    kind_const(ty)
                )
                .unwrap();
                for v in enum_variants(variants) {
                    writeln!(
                        s,
                        "super::{n}Value::{}=>{:?},",
                        ident(&v.rust_name),
                        v.variant
                    )
                    .unwrap();
                }
                s.push_str("}.into()),\n");
            }
            AstType::TransparentUnion { .. } => unreachable!(),
        }
    }
    s.push_str("_ => unreachable!(\"unknown AST node kind\"), } }\n}\n");
}
