//! serde を使う出力アダプター。生成 parser / AST 本体から独立させる。
use crate::{Ast, AstKind};
use serde::Serialize;
use serde_json::{json, Value};

pub fn value(ast: &Ast) -> Value {
    let fields = match &ast.kind {
        AstKind::Text { value, .. } => return Value::String(value.clone()),
        AstKind::Number { value, .. } => json!({ "value": value }),
        AstKind::Binary { left, op, right } | AstKind::Power { left, op, right } => {
            json!({ "left": value(left), "op": op, "right": right.iter().map(value).collect::<Vec<_>>() })
        }
    };
    // serde_json の既定 map はキー順。harness canonical.rs と同じ node / text / list。
    json!({ "type": ast.type_name(), "span": ast.span, "fields": fields })
}

pub struct Canonical<'a>(pub &'a Ast);
impl Serialize for Canonical<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        value(self.0).serialize(serializer)
    }
}

pub fn json(ast: &Ast) -> String {
    serde_json::to_string(&Canonical(ast)).expect("canonical は有限の JSON 値だけを含む")
}
