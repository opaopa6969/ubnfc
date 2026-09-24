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
    // D-025: 自分の AST ノードを作る（または捨てる）規則。この境界より外側の capture は
    // 回復領域そのものではなくノードを値に持つので `captureOccurrences` に数えない
    // （Java `Session.rule` の nodes 置換条件と同じ）。
    writeln!(
        s,
        "const RULE_NODE:&[bool]=&{:?};",
        g.ir.semantics
            .rules
            .iter()
            .map(|r| {
                r.skip
                    || g.ir
                        .semantics
                        .recipes
                        .iter()
                        .find(|recipe| recipe.rule_id == r.symbol_id)
                        .is_some_and(|recipe| {
                            !matches!(recipe.kind, RecipeKind::Transparent | RecipeKind::Text)
                        })
            })
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
    s.push_str("impl<const DIAG: bool> Session<'_, DIAG> {\nfn build_values(&mut self,root:EventId)->Result<Vec<Ast>,String> {let mut out=Vec::new();self.build_values_into(root,&mut out)?;Ok(out)}\n#[inline(never)] fn build_values_into(&mut self,root:EventId,out:&mut Vec<Ast>)->Result<(),String> {let frame=0u8;let address=std::ptr::addr_of!(frame) as usize;let anchor=*self.mapping_anchor.get_or_insert(address);if self.mapping_depth>=self.options.limits.mapping_depth || anchor.abs_diff(address)>256*1024 {return Err(\"maximum mapping depth exceeded\".into());}self.mapping_depth+=1;let result=self.build_values_inner(root,out);self.mapping_depth-=1;result}\nfn build_values_inner(&mut self,mut root:EventId,out:&mut Vec<Ast>)->Result<(),String> {
// 大半の呼出しは Join を含まない 1 本の枝（P4 実測: complex-x64 で 10,883 回すべてが 1 event）。
// その場合は pool から stack を借りずに降りる。
loop {match self.arena[root.0] {
Event::Capture{child,..}=>root=child,
Event::Values{child,span,wrap}=>{let start=out.len();self.build_values_into(child,out)?;if wrap && (matches!(&out[start..],[Ast::Text(_)]) || (out.len()==start && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {let text=self.semantic_text(child,span);out.truncate(start);out.push(Ast::Text(text));}return Ok(());},
Event::Rule{rule,span,child,caps}=>return self.build_rule(rule,caps,span,child,out),
Event::Join(..)=>break,
_=>return Ok(()),}}
let mut stack=self.take_stack();stack.push((root,false));while let Some((id,_))=stack.pop() {match self.arena[id.0] {Event::Join(a,b)=>{stack.push((b,false));stack.push((a,false));},Event::Capture{child,..}=>stack.push((child,false)),Event::Values{child,span,wrap}=>{let start=out.len();self.build_values_into(child,out)?;if wrap && (matches!(&out[start..],[Ast::Text(_)]) || (out.len()==start && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {let text=self.semantic_text(child,span);out.truncate(start);out.push(Ast::Text(text));}},Event::Rule{rule,span,child,caps}=>self.build_rule(rule,caps,span,child,out)?,_=>{}}}self.give_stack(stack);Ok(())}\nfn leaf(&mut self,ty:usize,span:Span,text:String)->Result<Ast,String> {let text=self.remember_text(text,span);let span=self.span(span);match ty {\n");
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
        s.push_str("let _=(span,text);Err(format!(\"unknown leaf type {ty}\"))}\nfn build_rule(&mut self,rule:usize,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);match rule {\n");
    } else {
        s.push_str("_=>{let _=(span,text);Err(format!(\"unknown leaf type {ty}\"))}}}\nfn build_rule(&mut self,rule:usize,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);match rule {\n");
    }
    for i in 0..g.ir.semantics.rules.len() {
        writeln!(s, "{i}=>self.map_rule_{i}(caps,span,child,out),").unwrap();
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
        writeln!(s,"fn map_rule_{i}(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {{let _=(caps,span,child,&out);{body}}}").unwrap();
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
        writeln!(s, "{i}=>Ok(()),").unwrap();
        return Ok(s);
    }
    let Some(recipe) = recipe else {
        writeln!(s,"{i}=>{{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {{let text=self.semantic_text(child,span);out.push(Ast::Text(text));}}Ok(())}},").unwrap();
        return Ok(s);
    };
    if recipe.kind == RecipeKind::Transparent {
        writeln!(s, "{i}=>self.build_values_into(child,out),").unwrap();
        return Ok(s);
    }
    if recipe.kind == RecipeKind::Text {
        writeln!(s, "{i}=>if self.directly_recovered(child) {{Ok(())}} else {{let text=self.semantic_text(child,span);out.push(Ast::Text(text));Ok(())}},").unwrap();
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
            writeln!(
                s,
                "{i}=>{{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;"
            )
            .unwrap();
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
                if text_type(&f.value_type) {
                    match f.cardinality {
                        Cardinality::Scalar => {
                            writeln!(s,"let mut t{n}=self.take_texts();self.text_values(&caps,&{sites:?},&mut t{n});if t{n}.is_empty() && self.missing_field(&caps,&{sites:?}) {{return Ok(());}}if t{n}.len()!=1 {{return Err({:?}.into());}}let f{n}=t{n}.pop().unwrap();self.give_texts(t{n});",format!("{} requires one value",f.name)).unwrap();
                        }
                        Cardinality::Optional => {
                            writeln!(s,"let mut t{n}=self.take_texts();self.text_values(&caps,&{sites:?},&mut t{n});if t{n}.len()>1 {{return Err({:?}.into());}}let f{n}=t{n}.pop();self.give_texts(t{n});",format!("{} requires at most one value", f.name)).unwrap();
                        }
                        _ => {
                            writeln!(s, "let mut f{n}=Vec::new();self.text_values(&caps,&{sites:?},&mut f{n});").unwrap();
                        }
                    }
                } else {
                    let fallback = recipe.leaf_fallback.as_ref().map(|l| {
                        g.ir.semantics
                            .ast_types
                            .iter()
                            .position(|t| t.id() == l.type_id)
                            .unwrap()
                    });
                    // List は node へ移すので新しい Vec、scalar / optional は pool の作業 Vec。
                    let (var, alloc) = if f.cardinality == Cardinality::List {
                        (format!("f{n}"), format!("let mut f{n}=Vec::new();"))
                    } else {
                        (format!("v{n}"), format!("let mut v{n}=self.take_values();"))
                    };
                    s.push_str(&alloc);
                    if matches!(rf.conversion, Conversion::Text | Conversion::Number) {
                        writeln!(s, "self.text_nodes(&caps,&{sites:?},&mut {var});").unwrap();
                    } else {
                        writeln!(
                            s,
                            "self.node_values(&caps,&{sites:?},{fallback:?},&mut {var})?;"
                        )
                        .unwrap();
                    }
                    let check = accepts(g, &f.value_type, "value");
                    writeln!(
                        s,
                        "if !{var}.iter().all(|value|{check}) {{return Err({:?}.into());}}",
                        format!("{}: mapped value type mismatch", f.name)
                    )
                    .unwrap();
                    match f.cardinality {
                        Cardinality::List => {}
                        Cardinality::Optional => {
                            writeln!(s,"if {var}.len()>1 {{return Err({:?}.into());}}let f{n}={var}.pop().map(Box::new);self.give_values({var});",format!("{} requires at most one node",f.name)).unwrap();
                        }
                        Cardinality::Scalar if nullable(&f.value_type) => {
                            writeln!(s,"if {var}.len()>1 {{return Err({:?}.into());}}let f{n}={var}.pop().map(Box::new);self.give_values({var});",format!("{} requires at most one node",f.name)).unwrap();
                        }
                        _ => {
                            writeln!(s,"if {var}.is_empty() && self.missing_field(&caps,&{sites:?}) {{return Ok(());}}if {var}.len()!=1 {{return Err({:?}.into());}}let f{n}=Box::new({var}.pop().unwrap());self.give_values({var});",format!("{} requires one node",f.name)).unwrap();
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
                // D-028: `@typeof` 違反の mappingError 文字列は旧 Java 実装の観測を固定した
                // 契約値（`corpus/oracle/typeof.json`）。互換のためここでだけ例外クラス名を持つ。
                // runtime 内部で作る他の mapping 失敗には付けない。
                writeln!(s,"if let (Some(expected),Some(actual))=({left},{right}) {{if expected.0!=actual.0 {{let (expected,actual)=(expected.1,actual.1);return Err(format!({:?}));}}}}",format!("java.lang.IllegalArgumentException: @typeof constraint violated: {} must be same type as {}, expected {{expected}} but got {{actual}}",own.name,reference.name)).unwrap();
            }
            writeln!(s,"self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[{i}],{name:?},span);out.push(Ast::{ty_id}({ty_id}{{span,node_id,").unwrap();
            for (n, f) in fields.iter().enumerate() {
                writeln!(s, "{}:f{n},", ident(&f.name)).unwrap();
            }
            s.push_str("}));Ok(())},\n");
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
            writeln!(s,"_=>return Err(\"enum literal mismatch\".into()),}};let span=self.span(span);let node_id=self.node(RULES[{i}],{name:?},span);out.push(Ast::{ty_id}({ty_id}{{span,node_id,value}}));Ok(())}},").unwrap();
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
    s.push_str("impl CollectValueSpans for Ast { fn collect(&self,path:&mut String,texts:&TextSpans,out:&mut Vec<ValueSpan>) { match self { Self::Text(text)=>{if let Some(span)=texts.get(&(text.as_ptr() as usize)) {out.push(ValueSpan{path:path.clone(),span:*span,text:Some(text.clone())});}},Self::Null=>{},\n");
    for ty in &g.ir.semantics.ast_types {
        if matches!(ty, AstType::TransparentUnion { .. }) {
            continue;
        }
        let id = ident(ty.id());
        writeln!(
            s,
            "Self::{id}(node)=>{{out.push(ValueSpan{{path:path.clone(),span:node.span,text:None}});"
        )
        .unwrap();
        if let AstType::Record { fields, .. } = ty {
            for f in fields {
                let field = f.name.replace('~', "~0").replace('/', "~1");
                writeln!(
                    s,
                    "{{let len=path.len();path.push_str(\"/fields/{field}\");node.{}.collect(path,texts,out);path.truncate(len);}}",
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
