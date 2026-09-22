use crate::{
    ast::{field_type, ident, nullable, text_type},
    emit::Generator,
    validate::unsupported,
    EmitError,
};
use std::fmt::Write;
use ubnfc_front::ir::*;
pub(crate) fn generate(g: &Generator<'_>) -> Result<String, EmitError> {
    let mut s = String::from("\nuse super::ast::*;\n");
    writeln!(
        s,
        "const HAS_SCOPE:bool={};",
        !g.ir.semantics.scope.effects.is_empty()
    )
    .unwrap();
    writeln!(
        s,
        "const RULES:&[&str]=&{:?};",
        g.ir.semantics
            .rules
            .iter()
            .map(|r| r.symbol_id.as_str())
            .collect::<Vec<_>>()
    )
    .unwrap();
    writeln!(
        s,
        "const BODIES:&[&str]=&{:?};",
        g.ir.semantics
            .rules
            .iter()
            .map(|r| r.body.as_str())
            .collect::<Vec<_>>()
    )
    .unwrap();
    writeln!(
        s,
        "const SITES:&[(&str,&str)]=&{:?};",
        g.ir.semantics
            .captures
            .iter()
            .map(|c| (c.site_id.as_str(), c.name.as_str()))
            .collect::<Vec<_>>()
    )
    .unwrap();
    s.push_str("impl Session<'_> {\nfn build_values(&mut self,root:EventId)->Result<Vec<Ast>,String> {let mut values=Vec::new();let mut stack=vec![root];while let Some(id)=stack.pop() {match self.arena[id.0] {Event::Join(a,b)=>{stack.push(b);stack.push(a);},Event::Capture{child,..}=>stack.push(child),Event::Rule{rule,span,child}=>values.extend(self.build_rule(rule,span,child)?),_=>{}}}Ok(values)}\nfn field_nodes(&mut self,root:EventId,sites:&[usize],fallback:Option<usize>)->Result<Vec<Ast>,String> {let mut values=Vec::new();for (span,child) in self.owned_captures(root,sites) {let mut found=self.build_values(child)?;if found.is_empty() && span[0]!=span[1] {found.push(Ast::Text(self.text(span)));}for value in found {values.push(if let (Some(ty),Ast::Text(text))=(fallback,&value) {self.leaf(ty,span,text.clone())?} else {value});}}Ok(values)}\nfn leaf(&mut self,ty:usize,span:Span,text:String)->Result<Ast,String> {let span=self.span(span);match ty {\n");
    let mut leaves = std::collections::BTreeSet::new();
    for recipe in &g.ir.semantics.recipes {
        if let Some(leaf) = &recipe.leaf_fallback {
            let index =
                g.ir.semantics
                    .ast_types
                    .iter()
                    .position(|t| t.id() == leaf.type_id)
                    .unwrap();
            if leaves.insert(index) {
                let name = match &g.ir.semantics.ast_types[index] {
                    AstType::Record { name, fields, .. }
                        if fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>()
                            == ["left", "op", "right"] =>
                    {
                        name
                    }
                    _ => {
                        return Err(unsupported(
                            "deferredMapping.leafFallback",
                            "requires left/op/right record",
                        ))
                    }
                };
                let ty = ident(&leaf.type_id);
                let rule = g.rules[recipe.rule_id.as_str()];
                writeln!(s,"{index}=>{{let node_id=self.node(RULES[{rule}],{name:?},span);Ok(Ast::{ty}({ty}{{span,node_id,{}:None,{}:vec![text],{}:vec![]}}))}},",ident("left"),ident("op"),ident("right")).unwrap();
            }
        }
    }
    if leaves.is_empty() {
        s = s.strip_suffix("match ty {\n").unwrap().to_owned();
        s.push_str("let _=(span,text);Err(format!(\"unknown leaf type {ty}\"))}\nfn build_rule(&mut self,rule:usize,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);match rule {\n");
    } else {
        s.push_str("_=>{let _=(span,text);Err(format!(\"unknown leaf type {ty}\"))}}}\nfn build_rule(&mut self,rule:usize,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);match rule {\n");
    }
    for (i, rule) in g.ir.semantics.rules.iter().enumerate() {
        let recipe =
            g.ir.semantics
                .recipes
                .iter()
                .find(|r| r.rule_id == rule.symbol_id);
        if rule.skip || recipe.is_some_and(|r| r.kind == RecipeKind::Skip) {
            writeln!(s, "{i}=>Ok(vec![]),").unwrap();
            continue;
        }
        let Some(recipe) = recipe else {
            writeln!(s,"{i}=>{{let mut values=self.build_values(child)?;if values.is_empty() {{values.push(Ast::Text(self.text(span)));}}Ok(values)}},").unwrap();
            continue;
        };
        if recipe.kind == RecipeKind::Transparent {
            writeln!(s, "{i}=>self.build_values(child),").unwrap();
            continue;
        }
        if recipe.kind == RecipeKind::Text {
            writeln!(s, "{i}=>Ok(vec![Ast::Text(self.text(span))]),").unwrap();
            continue;
        }
        let Some(type_id) = &recipe.result_type_id else {
            return Err(unsupported(
                "deferredMapping",
                "construct/enum needs resultTypeId",
            ));
        };
        let ty =
            g.ir.semantics
                .ast_types
                .iter()
                .find(|t| t.id() == type_id)
                .unwrap();
        let ty_id = ident(type_id);
        match ty {
            AstType::Record { fields, name, .. } => {
                if recipe.fields.len() != fields.len() {
                    return Err(EmitError::InvalidIr(format!(
                        "{}: recipe fields must match record fields",
                        recipe.id
                    )));
                }
                writeln!(s, "{i}=>{{").unwrap();
                for (n, f) in fields.iter().enumerate() {
                    let Some(rf) = recipe.fields.iter().find(|r| r.field == f.name) else {
                        return Err(EmitError::InvalidIr(format!(
                            "{}: missing recipe field {}",
                            recipe.id, f.name
                        )));
                    };
                    if rf.cardinality != f.cardinality
                        || (text_type(&f.value_type)
                            && !matches!(rf.conversion, Conversion::Text | Conversion::Number))
                    {
                        return Err(EmitError::InvalidIr(format!(
                            "{}: incompatible recipe field {}",
                            recipe.id, f.name
                        )));
                    }
                    let sites = rf
                        .site_ids
                        .iter()
                        .map(|id| g.sites[id.as_str()])
                        .collect::<Vec<_>>();
                    let fty = field_type(f);
                    let mutable = if f.cardinality == Cardinality::List {
                        ""
                    } else {
                        "mut "
                    };
                    if text_type(&f.value_type) {
                        writeln!(s, "let {mutable}f{n}=self.field_text(child,&{sites:?});")
                            .unwrap();
                        match f.cardinality {
                            Cardinality::Scalar => {
                                writeln!(s,"if f{n}.len()!=1 {{return Err({:?}.into());}}let f{n}=f{n}.remove(0);",format!("{} requires one value",f.name)).unwrap();
                            }
                            Cardinality::Optional => {
                                writeln!(s,"if f{n}.len()>1 {{return Err({:?}.into());}}let f{n}=f{n}.pop();",format!("{} requires at most one value",f.name)).unwrap();
                            }
                            _ => {}
                        }
                    } else {
                        let fallback = recipe.leaf_fallback.as_ref().map(|l| {
                            g.ir.semantics
                                .ast_types
                                .iter()
                                .position(|t| t.id() == l.type_id)
                                .unwrap()
                        });
                        if matches!(rf.conversion, Conversion::Text | Conversion::Number) {
                            writeln!(s,"let {mutable}f{n}=self.field_text(child,&{sites:?}).into_iter().map(Ast::Text).collect::<Vec<_>>();").unwrap();
                        } else {
                            writeln!(s,"let {mutable}f{n}=self.field_nodes(child,&{sites:?},{fallback:?})?;").unwrap();
                        }
                        let check = accepts(g, &f.value_type, "value", &mut Vec::new());
                        writeln!(
                            s,
                            "if !f{n}.iter().all(|value|{check}) {{return Err({:?}.into());}}",
                            format!("{}: mapped value type mismatch", f.name)
                        )
                        .unwrap();
                        match f.cardinality {
                            Cardinality::List => {}
                            Cardinality::Optional => {
                                writeln!(s,"if f{n}.len()>1 {{return Err({:?}.into());}}let f{n}=f{n}.pop().map(Box::new);",format!("{} requires at most one node",f.name)).unwrap();
                            }
                            Cardinality::Scalar if nullable(&f.value_type) => {
                                writeln!(s,"if f{n}.len()>1 {{return Err({:?}.into());}}let f{n}=f{n}.pop().map(Box::new);",format!("{} requires at most one node",f.name)).unwrap();
                            }
                            _ => {
                                writeln!(s,"if f{n}.len()!=1 {{return Err({:?}.into());}}let f{n}=Box::new(f{n}.remove(0));",format!("{} requires one node",f.name)).unwrap();
                            }
                        }
                    }
                    writeln!(s, "let f{n}:{fty}=f{n};").unwrap();
                }
                writeln!(s,"let span=self.span(span);let node_id=self.node(RULES[{i}],{name:?},span);Ok(vec![Ast::{ty_id}({ty_id}{{span,node_id,").unwrap();
                for (n, f) in fields.iter().enumerate() {
                    writeln!(s, "{}:f{n},", ident(&f.name)).unwrap();
                }
                s.push_str("})])},\n");
            }
            AstType::Enum { variants, name, .. } => {
                writeln!(s, "{i}=>{{let value=match self.text(span).as_str() {{").unwrap();
                for variant in variants {
                    writeln!(
                        s,
                        "{:?}=>{ty_id}Value::{},",
                        variant.literal,
                        ident(&variant.rust_name)
                    )
                    .unwrap();
                }
                writeln!(s,"_=>return Err(\"enum literal mismatch\".into()),}};let span=self.span(span);let node_id=self.node(RULES[{i}],{name:?},span);Ok(vec![Ast::{ty_id}({ty_id}{{span,node_id,value}})])}},").unwrap();
            }
            AstType::TransparentUnion { .. } => {
                return Err(unsupported("deferredMapping.constructUnion", type_id))
            }
        }
    }
    s.push_str("_=>Err(format!(\"unknown rule {rule}\"))}}\n");
    s.push_str(include_str!("../templates/finish.rs"));
    s.push_str("}\n");
    Ok(s)
}

fn accepts(g: &Generator<'_>, ty: &ValueType, value: &str, visited: &mut Vec<String>) -> String {
    match ty {
        ValueType::Text {} | ValueType::Number { .. } => format!("matches!({value},Ast::Text(_))"),
        ValueType::Null {} => format!("matches!({value},Ast::Null)"),
        ValueType::Mixed { members } => format!(
            "({})",
            members
                .iter()
                .map(|m| accepts(g, m, value, visited))
                .collect::<Vec<_>>()
                .join(" || ")
        ),
        ValueType::Node { type_id } => {
            if visited.contains(type_id) {
                return "false".into();
            }
            visited.push(type_id.clone());
            let ty =
                g.ir.semantics
                    .ast_types
                    .iter()
                    .find(|t| t.id() == type_id)
                    .unwrap();
            let result = if let AstType::TransparentUnion { variants, .. } = ty {
                format!(
                    "({})",
                    variants
                        .iter()
                        .map(|id| accepts(
                            g,
                            &ValueType::Node {
                                type_id: id.clone()
                            },
                            value,
                            visited
                        ))
                        .collect::<Vec<_>>()
                        .join(" || ")
                )
            } else {
                format!("matches!({value},Ast::{}(_))", ident(type_id))
            };
            visited.pop();
            result
        }
        _ => "false".into(),
    }
}
