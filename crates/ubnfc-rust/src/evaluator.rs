use crate::{
    ast::{ident, nullable, text_type, type_ident},
    validate::unsupported,
    EmitError,
};
use std::{collections::BTreeMap, fmt::Write};
use ubnfc_front::ir::*;

pub(crate) fn generate(ir: &GrammarIr) -> Result<String, EmitError> {
    let mut s = String::from("//! Typed evaluator dispatch. Rust numeric captures remain lexemes.\n#![allow(non_snake_case,unused_imports)]\nuse super::ast::*;\n#[derive(Clone,Debug,PartialEq)] pub enum Value {Text(String),Number(f64),Bool(bool),Node(Box<Ast>),Null}\n#[derive(Clone,Debug,PartialEq,Eq)] pub struct EvalError(pub String);\nimpl std::fmt::Display for EvalError {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {self.0.fmt(f)}}\nimpl std::error::Error for EvalError {}\npub type EvalResult=Result<Value,EvalError>;\npub trait Evaluator {\nfn eval(&mut self,node:&Ast)->EvalResult {match node {Ast::Text(text)=>Ok(Value::Text(text.clone())),Ast::Null=>Ok(Value::Null),\n");
    for ty in &ir.semantics.ast_types {
        if !matches!(ty, AstType::TransparentUnion { .. }) {
            let n = type_ident(ty);
            writeln!(s, "Ast::{n}(node)=>self.eval_{n}(node),").unwrap();
        }
    }
    s.push_str("}}\n");
    let mut required = BTreeMap::new();
    let mut binary = false;
    let mut variable = false;
    for ty in &ir.semantics.ast_types {
        if matches!(ty, AstType::TransparentUnion { .. }) {
            continue;
        }
        let n = type_ident(ty);
        let rule = ir
            .semantics
            .rules
            .iter()
            .find(|r| r.mapping_type_id.as_deref() == Some(ty.id()));
        let eval = rule.and_then(|r| r.eval.as_ref());
        let auto = eval.filter(|e| e.strategy == EvalStrategy::Auto);
        let Some(eval) = auto else {
            writeln!(s, "fn eval_{n}(&mut self,node:&{n})->EvalResult;").unwrap();
            required.insert(format!("eval_{n}"), format!("node:&{n}"));
            continue;
        };
        let AstType::Record { fields, .. } = ty else {
            return Err(unsupported(
                "eval.auto",
                "automatic evaluation requires a record",
            ));
        };
        let field = |name: &str| -> Result<&AstField, EmitError> {
            fields
                .iter()
                .find(|f| f.name == name && f.cardinality == Cardinality::Scalar)
                .ok_or_else(|| unsupported("eval.auto", format!("{} needs scalar {name}", ty.id())))
        };
        let value = |f: &AstField, recurse: bool| {
            let v = format!("node.{}", ident(&f.name));
            if text_type(&f.value_type) {
                format!("Ok(Value::Text({v}.clone()))")
            } else if nullable(&f.value_type) {
                if recurse {
                    format!("{v}.as_deref().map_or(Ok(Value::Null),|v|self.eval(v))")
                } else {
                    format!("Ok({v}.clone().map_or(Value::Null,Value::Node))")
                }
            } else if recurse {
                format!("self.eval(&{v})")
            } else {
                format!("Ok(Value::Node({v}.clone()))")
            }
        };
        writeln!(s, "fn eval_{n}(&mut self,node:&{n})->EvalResult {{").unwrap();
        match eval.kind.as_str() {
            "literal" | "passthrough" => {
                writeln!(s, "{}", value(field("value")?, eval.kind == "passthrough")).unwrap();
            }
            "variable_ref" => {
                let f = field("name")?;
                if !text_type(&f.value_type) {
                    return Err(unsupported("eval.variable_ref", "name must be text"));
                }
                let strip = eval
                    .params
                    .get("strip_prefix")
                    .and_then(|v| {
                        if let JsonValue::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("");
                writeln!(s,"let name=&node.{};self.resolve_variable(name.strip_prefix({strip:?}).unwrap_or(name))",ident("name")).unwrap();
                variable = true;
            }
            "conditional" => {
                let condition = value(field("condition")?, true);
                let yes = value(field("thenExpr")?, true);
                let no = value(field("elseExpr")?, true);
                writeln!(s,"let condition=({condition})?;let truth=match condition {{Value::Bool(b)=>b,Value::Text(s)=>s.eq_ignore_ascii_case(\"true\"),_=>false}};if truth {{{yes}}} else {{{no}}}").unwrap();
            }
            "binary_arithmetic" => {
                let left = field("left")?;
                let op = fields.iter().find(|f| {
                    f.name == "op" && f.cardinality == Cardinality::List && text_type(&f.value_type)
                });
                let right = fields.iter().find(|f| {
                    f.name == "right"
                        && f.cardinality == Cardinality::List
                        && !text_type(&f.value_type)
                });
                if op.is_none() || right.is_none() || text_type(&left.value_type) {
                    return Err(unsupported(
                        "eval.binary_arithmetic",
                        "requires node left / text[] op / node[] right",
                    ));
                }
                let (l, o, r) = (ident("left"), ident("op"), ident("right"));
                if nullable(&left.value_type) {
                    writeln!(s,"if node.{l}.is_none() && node.{r}.is_empty() && node.{o}.len()==1 {{return self.eval_leaf(&node.{o}[0]);}}let left=node.{l}.as_deref().ok_or_else(||EvalError(\"missing binary left\".into()))?;").unwrap();
                } else {
                    writeln!(s, "let left=&node.{l};").unwrap();
                }
                writeln!(s,"let mut current=self.eval(left)?;for (op,right) in node.{o}.iter().zip(&node.{r}) {{let right=self.eval(right)?;current=self.apply_binary(op,current,right)?;}}Ok(current)").unwrap();
                binary = true;
            }
            _ => return Err(unsupported("eval.auto", &eval.kind)),
        }
        s.push_str("}\n");
    }
    if binary {
        s.push_str("fn eval_leaf(&mut self,text:&str)->EvalResult;\nfn apply_binary(&mut self,op:&str,left:Value,right:Value)->EvalResult;\n");
        required.insert("eval_leaf".into(), "text:&str".into());
        required.insert(
            "apply_binary".into(),
            "op:&str,left:Value,right:Value".into(),
        );
    }
    if variable {
        s.push_str("fn resolve_variable(&mut self,name:&str)->EvalResult;\n");
        required.insert("resolve_variable".into(), "name:&str".into());
    }
    s.push_str("}\n/// Automatic kinds work without overrides; semantic callbacks report a missing implementation.\npub struct DefaultEvaluator;\nimpl Evaluator for DefaultEvaluator {\n");
    for (name, args) in required {
        let unused = args
            .split(',')
            .map(|a| a.split(':').next().unwrap())
            .collect::<Vec<_>>()
            .join(",");
        writeln!(s,"fn {name}(&mut self,{args})->EvalResult {{let _=({unused},);Err(EvalError({:?}.into()))}}",format!("callback required: {name}")).unwrap();
    }
    s.push_str("}\npub fn evaluate<E:Evaluator+?Sized>(node:&Ast,evaluator:&mut E)->EvalResult {evaluator.eval(node)}\n");
    Ok(s)
}
