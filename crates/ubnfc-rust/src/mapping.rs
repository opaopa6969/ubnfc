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
        "const HAS_RECOVERY:bool={};",
        !g.ir.semantics.recovery.is_empty()
    )
    .unwrap();
    let mut modes = Vec::new();
    for effect in &g.ir.semantics.scope.effects {
        let mode = match effect.mode {
            ScopeEffectMode::Lexical => "lexical",
            ScopeEffectMode::Dynamic => "dynamic",
            ScopeEffectMode::SameRuleText => "sameRuleText",
        };
        if !modes.contains(&mode) {
            modes.push(mode);
        }
    }
    writeln!(s, "const SCOPE_MODES:&[&str]=&{modes:?};").unwrap();
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
        "const EXPRESSIONS:&[&str]=&{:?};",
        g.ir.semantics
            .expressions
            .iter()
            .map(|e| e.id.as_str())
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
    s.push_str("impl Session<'_> {\n#[inline(never)] fn build_values(&mut self,root:EventId)->Result<Vec<Ast>,String> {let frame=0u8;let address=std::ptr::addr_of!(frame) as usize;let anchor=*self.mapping_anchor.get_or_insert(address);if self.mapping_depth>=self.options.limits.mapping_depth || anchor.abs_diff(address)>256*1024 {return Err(\"maximum mapping depth exceeded\".into());}self.mapping_depth+=1;let result=self.build_values_inner(root);self.mapping_depth-=1;result}\nfn build_values_inner(&mut self,root:EventId)->Result<Vec<Ast>,String> {let mut values=Vec::new();let mut stack=vec![root];while let Some(id)=stack.pop() {match self.arena[id.0] {Event::Join(a,b)=>{stack.push(b);stack.push(a);},Event::Capture{child,..}=>stack.push(child),Event::Values{child,span,wrap}=>{let mut found=self.build_values(child)?;if wrap && (matches!(found.as_slice(),[Ast::Text(_)]) || (found.is_empty() && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {found=vec![Ast::Text(self.semantic_text(child,span))];}values.extend(found);},Event::Rule{rule,span,child}=>values.extend(self.build_rule(rule,span,child)?),_=>{}}}Ok(values)}\nfn field_nodes(&mut self,root:EventId,sites:&[usize],fallback:Option<usize>)->Result<Vec<Ast>,String> {let mut values=Vec::new();for (span,child) in self.owned_captures(root,sites) {let mut found=self.build_values(child)?;if found.is_empty() && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child) {found.push(Ast::Text(self.semantic_text(child,span)));}for value in found {values.push(if let (Some(ty),Ast::Text(text))=(fallback,&value) {self.leaf(ty,span,text.clone())?} else {value});}}Ok(values)}\nfn leaf(&mut self,ty:usize,span:Span,text:String)->Result<Ast,String> {let text=self.remember_text(text,span);let span=self.span(span);match ty {\n");
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
    for i in 0..g.ir.semantics.rules.len() {
        writeln!(s, "{i}=>self.map_rule_{i}(span,child),").unwrap();
    }
    s.push_str("_=>Err(format!(\"unknown rule {rule}\"))}}\n");
    for (i, rule) in g.ir.semantics.rules.iter().enumerate() {
        let arm = rule_mapping(g, i, rule)?;
        let body = arm
            .strip_prefix(&format!("{i}=>"))
            .unwrap()
            .trim_end()
            .strip_suffix(',')
            .unwrap();
        writeln!(s,"fn map_rule_{i}(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {{let _=(span,child);{body}}}").unwrap();
    }
    s.push_str(include_str!("../templates/finish.rs"));
    s.push_str("}\n");
    s.push_str(&value_spans(g));
    Ok(s)
}

fn rule_mapping(g: &Generator<'_>, i: usize, rule: &Rule) -> Result<String, EmitError> {
    let mut s = String::new();
    let recipe =
        g.ir.semantics
            .recipes
            .iter()
            .find(|r| r.rule_id == rule.symbol_id);
    if rule.skip || recipe.is_some_and(|r| r.kind == RecipeKind::Skip) {
        writeln!(s, "{i}=>Ok(vec![]),").unwrap();
        return Ok(s);
    }
    let Some(recipe) = recipe else {
        writeln!(s,"{i}=>{{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {{values.push(Ast::Text(self.semantic_text(child,span)));}}Ok(values)}},").unwrap();
        return Ok(s);
    };
    if recipe.kind == RecipeKind::Transparent {
        writeln!(s, "{i}=>self.build_values(child),").unwrap();
        return Ok(s);
    }
    if recipe.kind == RecipeKind::Text {
        writeln!(s, "{i}=>if self.directly_recovered(child) {{Ok(vec![])}} else {{Ok(vec![Ast::Text(self.semantic_text(child,span))])}},").unwrap();
        return Ok(s);
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
                    writeln!(s, "let {mutable}f{n}=self.field_text(child,&{sites:?});").unwrap();
                    match f.cardinality {
                        Cardinality::Scalar => {
                            writeln!(s,"if f{n}.is_empty() && self.field_missing(child,&{sites:?}) {{return Ok(vec![]);}}if f{n}.len()!=1 {{return Err({:?}.into());}}let f{n}=f{n}.remove(0);",format!("{} requires one value",f.name)).unwrap();
                        }
                        Cardinality::Optional => {
                            writeln!(
                                s,
                                "if f{n}.len()>1 {{return Err({:?}.into());}}let f{n}=f{n}.pop();",
                                format!("{} requires at most one value", f.name)
                            )
                            .unwrap();
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
                        writeln!(
                            s,
                            "let {mutable}f{n}=self.field_nodes(child,&{sites:?},{fallback:?})?;"
                        )
                        .unwrap();
                    }
                    let check = accepts(g, &f.value_type, "value");
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
                            writeln!(s,"if f{n}.is_empty() && self.field_missing(child,&{sites:?}) {{return Ok(vec![]);}}if f{n}.len()!=1 {{return Err({:?}.into());}}let f{n}=Box::new(f{n}.remove(0));",format!("{} requires one node",f.name)).unwrap();
                        }
                    }
                }
                writeln!(s, "let f{n}:{fty}=f{n};").unwrap();
            }
            for constraint in &g.ir.semantics.typeof_constraints {
                let own = &g.ir.semantics.captures[g.sites[constraint.own_capture.as_str()]];
                if own.rule_id != rule.symbol_id {
                    continue;
                }
                let reference =
                    &g.ir.semantics.captures[g.sites[constraint.referenced_capture.as_str()]];
                let key = |name: &str| -> Result<String, EmitError> {
                    let n = fields
                        .iter()
                        .position(|f| f.name == name)
                        .ok_or_else(|| unsupported("typeof", format!("unmapped field {name}")))?;
                    let f = &fields[n];
                    Ok(if f.cardinality == Cardinality::List {
                        "Some((\"list\",\"list\"))".into()
                    } else if f.cardinality == Cardinality::Optional || nullable(&f.value_type) {
                        if text_type(&f.value_type) {
                            format!("f{n}.as_ref().map(|_|(\"text\",\"text\"))")
                        } else {
                            format!("f{n}.as_ref().map(|v|(v.type_id(),v.type_name()))")
                        }
                    } else if text_type(&f.value_type) {
                        "Some((\"text\",\"text\"))".into()
                    } else {
                        format!("Some((f{n}.type_id(),f{n}.type_name()))")
                    })
                };
                let left = key(&reference.name)?;
                let right = key(&own.name)?;
                writeln!(s,"if let (Some(expected),Some(actual))=({left},{right}) {{if expected.0!=actual.0 {{let (expected,actual)=(expected.1,actual.1);return Err(format!({:?}));}}}}",format!("@typeof constraint violated: {} must be same type as {}, expected {{expected}} but got {{actual}}",own.name,reference.name)).unwrap();
            }
            writeln!(s,"let span=self.span(span);let node_id=self.node(RULES[{i}],{name:?},span);Ok(vec![Ast::{ty_id}({ty_id}{{span,node_id,").unwrap();
            for (n, f) in fields.iter().enumerate() {
                writeln!(s, "{}:f{n},", ident(&f.name)).unwrap();
            }
            s.push_str("})])},\n");
        }
        AstType::Enum { variants, name, .. } => {
            writeln!(s, "{i}=>{{let value=match self.text(span).as_str() {{").unwrap();
            let mut seen = std::collections::BTreeSet::new();
            for variant in variants.iter().filter(|v| seen.insert(&v.literal)) {
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
    Ok(s)
}

fn accepts(g: &Generator<'_>, ty: &ValueType, value: &str) -> String {
    enum Pending<'a> {
        Value(&'a ValueType),
        Node(&'a str),
    }
    let mut pending = vec![Pending::Value(ty)];
    let mut seen = std::collections::BTreeSet::new();
    let mut clauses = std::collections::BTreeSet::new();
    while let Some(item) = pending.pop() {
        let id = match item {
            Pending::Value(ValueType::Mixed { members }) => {
                pending.extend(members.iter().map(Pending::Value));
                continue;
            }
            Pending::Value(ValueType::Text {} | ValueType::Number { .. }) => {
                clauses.insert(format!("matches!({value},Ast::Text(_))"));
                continue;
            }
            Pending::Value(ValueType::Null {}) => {
                clauses.insert(format!("matches!({value},Ast::Null)"));
                continue;
            }
            Pending::Value(ValueType::Node { type_id }) => type_id.as_str(),
            Pending::Node(id) => id,
            _ => continue,
        };
        if !seen.insert(id) {
            continue;
        }
        let ty =
            g.ir.semantics
                .ast_types
                .iter()
                .find(|t| t.id() == id)
                .unwrap();
        if let AstType::TransparentUnion { variants, .. } = ty {
            pending.extend(variants.iter().map(|s| Pending::Node(s)));
        } else {
            clauses.insert(format!("matches!({value},Ast::{}(_))", ident(id)));
        }
    }
    if clauses.is_empty() {
        "false".into()
    } else {
        clauses.into_iter().collect::<Vec<_>>().join(" || ")
    }
}

fn value_spans(g: &Generator<'_>) -> String {
    let mut s = include_str!("../templates/value_spans.rs").to_owned();
    s.push_str("impl CollectValueSpans for Ast { fn collect(&self,path:&str,texts:&std::collections::HashMap<usize,Span>,out:&mut Vec<ValueSpan>) { match self { Self::Text(text)=>{if let Some(span)=texts.get(&(text.as_ptr() as usize)) {out.push(ValueSpan{path:path.into(),span:*span,text:Some(text.clone())});}},Self::Null=>{},\n");
    for ty in &g.ir.semantics.ast_types {
        if matches!(ty, AstType::TransparentUnion { .. }) {
            continue;
        }
        let id = ident(ty.id());
        writeln!(
            s,
            "Self::{id}(node)=>{{out.push(ValueSpan{{path:path.into(),span:node.span,text:None}});"
        )
        .unwrap();
        if let AstType::Record { fields, .. } = ty {
            for f in fields {
                let field = f.name.replace('~', "~0").replace('/', "~1");
                writeln!(
                    s,
                    "node.{}.collect(&format!(\"{{path}}/fields/{field}\"),texts,out);",
                    ident(&f.name)
                )
                .unwrap();
            }
        }
        s.push_str("},\n");
    }
    s.push_str("}}}\n");
    s
}
