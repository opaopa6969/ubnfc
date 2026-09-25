use crate::{
    ast::{nullable, text_type},
    emit::Generator,
    tree::{enum_variants, kind_const, Repr},
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
    // D-077: 規則が作る record の slot 数（木の slot の見積りに使う。record を作らない規則は 0）。
    writeln!(
        s,
        "const RULE_SLOTS:&[u32]=&{:?};",
        g.ir.semantics
            .rules
            .iter()
            .map(|r| {
                let recipe =
                    g.ir.semantics
                        .recipes
                        .iter()
                        .find(|recipe| recipe.rule_id == r.symbol_id);
                let ty = recipe
                    .and_then(|recipe| recipe.result_type_id.as_deref())
                    .and_then(|id| g.ir.semantics.ast_types.iter().find(|t| t.id() == id));
                match ty {
                    Some(AstType::Record { fields, .. }) if !r.skip => fields
                        .iter()
                        .map(|f| Repr::of(f).slots() as u32)
                        .sum::<u32>(),
                    _ => 0,
                }
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
    s.push_str("impl<const DIAG: bool> Session<'_, DIAG> {\nfn build_values(&mut self,root:EventId)->Result<Vec<u32>,String> {let mut out=Vec::new();self.build_values_into(root,&mut out)?;Ok(out)}\n#[inline(never)] fn build_values_into(&mut self,root:EventId,out:&mut Vec<u32>)->Result<(),String> {let frame=0u8;let address=std::ptr::addr_of!(frame) as usize;let anchor=*self.mapping_anchor.get_or_insert(address);if self.mapping_depth>=self.options.limits.mapping_depth || anchor.abs_diff(address)>256*1024 {return Err(\"maximum mapping depth exceeded\".into());}self.mapping_depth+=1;let result=self.build_values_inner(root,out);self.mapping_depth-=1;result}\nfn build_values_inner(&mut self,mut root:EventId,out:&mut Vec<u32>)->Result<(),String> {
// 大半の呼出しは Join を含まない 1 本の枝（P4 実測: complex-x64 で 10,883 回すべてが 1 event）。
// その場合は pool から stack を借りずに降りる。
loop {match self.ev(root) {
Event::Capture{child,..}=>root=child,
Event::Values{child,span,wrap,text}=>{let start=out.len();if !text {self.value_region+=1;}let built=self.build_values_into(child,out);if !text {self.value_region-=1;}built?;if text || wrap && ((out.len()==start+1 && self.tree.kind(out[start])==tree::KIND_TEXT) || (out.len()==start && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {let text=self.semantic_text(child,span);out.truncate(start);out.push(text);}return Ok(());},
Event::Rule{rule,span,child,caps}=>return self.build_rule(rule,caps,span,child,out),
Event::Join(..)=>break,
_=>return Ok(()),}}
let mut stack=self.take_stack();stack.push((root,false));while let Some((id,_))=stack.pop() {match self.ev(id) {Event::Join(a,b)=>{stack.push((b,false));stack.push((a,false));},Event::Capture{child,..}=>stack.push((child,false)),Event::Values{child,span,wrap,text}=>{let start=out.len();if !text {self.value_region+=1;}let built=self.build_values_into(child,out);if !text {self.value_region-=1;}built?;if text || wrap && ((out.len()==start+1 && self.tree.kind(out[start])==tree::KIND_TEXT) || (out.len()==start && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {let text=self.semantic_text(child,span);out.truncate(start);out.push(text);}},Event::Rule{rule,span,child,caps}=>self.build_rule(rule,caps,span,child,out)?,_=>{}}}self.give_stack(stack);Ok(())}\nfn leaf(&mut self,ty:usize,span:Span,text:u32)->Result<u32,String> {let span=self.span(span);self.tree.set_extent(text,span);match ty {\n");
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
                let ty = &g.ir.semantics.ast_types[index];
                let fields = match ty {
                    AstType::Record { fields, .. }
                        if fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>()
                            == ["left", "op", "right"] =>
                    {
                        fields
                    }
                    _ => {
                        return Err(unsupported(
                            "deferredMapping.leafFallback",
                            "requires left/op/right record",
                        ))
                    }
                };
                // 所有形では `left: None, op: vec![text], right: vec![]`。
                let reprs = fields.iter().map(Repr::of).collect::<Vec<_>>();
                if !matches!(reprs[0], Repr::OptNode | Repr::OptText)
                    || !matches!(reprs[1], Repr::TextList | Repr::NodeList)
                    || !matches!(reprs[2], Repr::TextList | Repr::NodeList)
                {
                    return Err(unsupported(
                        "deferredMapping.leafFallback",
                        "requires optional left and list op/right",
                    ));
                }
                let rule = g.rules[recipe.rule_id.as_str()];
                writeln!(s,"{index}=>{{let op=self.tree.list(&[text]);let right=self.tree.list(&[]);Ok(self.tree.record(tree::{},{rule},span,&[tree::NONE,op[0],op[1],right[0],right[1]]))}},",kind_const(ty)).unwrap();
            }
        }
    }
    if leaves.is_empty() {
        s = s.strip_suffix("match ty {\n").unwrap().to_owned();
        s.push_str("let _=(span,text);Err(format!(\"unknown leaf type {ty}\"))}\nfn build_rule(&mut self,rule:usize,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);match rule {\n");
    } else {
        s.push_str("_=>{let _=(span,text);Err(format!(\"unknown leaf type {ty}\"))}}}\nfn build_rule(&mut self,rule:usize,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);match rule {\n");
    }
    for (i, rule) in g.ir.semantics.rules.iter().enumerate() {
        if opens_value_scope(g, rule) {
            // #86: 写像される規則の本体は新しい値の文脈。外側の値域の中にいても、
            // 自分の field（capture の子）は text 規則の字句を値にしてよい。
            writeln!(s, "{i}=>{{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_{i}(caps,span,child,out);self.value_region=outer;built}},").unwrap();
        } else {
            writeln!(s, "{i}=>self.map_rule_{i}(caps,span,child,out),").unwrap();
        }
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
        writeln!(s,"fn map_rule_{i}(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {{let _=(caps,span,child,&out);{body}}}").unwrap();
    }
    s.push_str(include_str!("../templates/finish.rs"));
    s.push_str("}\n");
    Ok(s)
}

/// #86: 意味値を持たない（text だけの）写像されない規則。`semantic_shape` が `None` のもの。
fn text_only_rule(g: &Generator<'_>, rule: &Rule) -> bool {
    !rule.skip
        && rule.mapping_type_id.is_none()
        && crate::expressions::semantic_shape_of(g, &rule.body).is_none()
}

/// #89: capture site の 1 出現の中身が意味値を持たない（text だけの）とき真。
/// Optional / Repeat に付いた site は要素ごとに出現するので、要素の式で判定する。
fn text_only_site(g: &Generator<'_>, site: usize) -> bool {
    let capture = &g.ir.semantics.captures[site];
    let expr = &g.ir.semantics.expressions[g.expr[capture.expr_id.as_str()]];
    let occurrence = match &expr.node {
        ExprNode::Optional { child } | ExprNode::Repeat { child, .. } => child,
        _ => &expr.id,
    };
    crate::expressions::semantic_shape_of(g, occurrence).is_none()
}

/// #86: 本体の写像が新しい値の文脈を開く規則（recipe を持ち、透過でも text でもないもの）。
fn opens_value_scope(g: &Generator<'_>, rule: &Rule) -> bool {
    if rule.skip {
        return false;
    }
    g.ir.semantics
        .recipes
        .iter()
        .find(|r| r.rule_id == rule.symbol_id)
        .is_some_and(|r| {
            !matches!(
                r.kind,
                RecipeKind::Skip | RecipeKind::Transparent | RecipeKind::Text
            )
        })
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
        if text_only_rule(g, rule) {
            // #86: 意味値を持たない（text だけの）規則への参照は、意味値の領域
            // （`Values{text:false}`）の中では区切りで値を作らない（D-032）。Java の
            // `combine` は null（plain text）の項目を値の列へ数えない。領域の外（capture の
            // 直下など）では従来どおり一致範囲の字句を値にする。
            writeln!(s,"{i}=>{{if self.value_region>0 {{return Ok(());}}let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {{let text=self.semantic_text(child,span);out.push(text);}}Ok(())}},").unwrap();
            return Ok(s);
        }
        writeln!(s,"{i}=>{{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {{let text=self.semantic_text(child,span);out.push(text);}}Ok(())}},").unwrap();
        return Ok(s);
    };
    if recipe.kind == RecipeKind::Transparent {
        writeln!(s, "{i}=>self.build_values_into(child,out),").unwrap();
        return Ok(s);
    }
    if recipe.kind == RecipeKind::Text {
        writeln!(s, "{i}=>if self.directly_recovered(child) {{Ok(())}} else {{let text=self.semantic_text(child,span);out.push(text);Ok(())}},").unwrap();
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
    match ty {
        AstType::Record { fields, .. } => {
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
                let repr = Repr::of(f);
                if text_type(&f.value_type) {
                    match f.cardinality {
                        Cardinality::Scalar => {
                            writeln!(s,"let mut t{n}=self.take_values();self.text_values(&caps,&{sites:?},&mut t{n});if t{n}.is_empty() && self.missing_field(&caps,&{sites:?}) {{return Ok(());}}if t{n}.len()!=1 {{return Err({:?}.into());}}let f{n}=t{n}.pop().unwrap();self.give_values(t{n});",format!("{} requires one value",f.name)).unwrap();
                        }
                        Cardinality::Optional => {
                            writeln!(s,"let mut t{n}=self.take_values();self.text_values(&caps,&{sites:?},&mut t{n});if t{n}.len()>1 {{return Err({:?}.into());}}let f{n}=t{n}.pop();self.give_values(t{n});",format!("{} requires at most one value", f.name)).unwrap();
                        }
                        _ => {
                            writeln!(s, "let mut f{n}=self.take_values();self.text_values(&caps,&{sites:?},&mut f{n});").unwrap();
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
                    // 作業 Vec はどれも pool から借りる（list は木の slot へ写してから返す）。
                    let (var, alloc) = if f.cardinality == Cardinality::List {
                        (format!("f{n}"), format!("let mut f{n}=self.take_values();"))
                    } else {
                        (format!("v{n}"), format!("let mut v{n}=self.take_values();"))
                    };
                    s.push_str(&alloc);
                    if matches!(rf.conversion, Conversion::Text | Conversion::Number) {
                        writeln!(s, "self.text_nodes(&caps,&{sites:?},&mut {var});").unwrap();
                    } else {
                        // #89: 出現の中身が text だけ（意味形 None）の site は、出現 1 つにつき
                        // 一致範囲の字句 1 つを値にする（D-032）。
                        let text_sites = rf
                            .site_ids
                            .iter()
                            .map(|id| g.sites[id.as_str()])
                            .filter(|&i| text_only_site(g, i))
                            .collect::<Vec<_>>();
                        writeln!(
                            s,
                            "self.node_values(&caps,&{sites:?},&{text_sites:?},{fallback:?},&mut {var})?;"
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
                            writeln!(s,"if {var}.len()>1 {{return Err({:?}.into());}}let f{n}={var}.pop();self.give_values({var});",format!("{} requires at most one node",f.name)).unwrap();
                        }
                        Cardinality::Scalar if nullable(&f.value_type) => {
                            writeln!(s,"if {var}.len()>1 {{return Err({:?}.into());}}let f{n}={var}.pop();self.give_values({var});",format!("{} requires at most one node",f.name)).unwrap();
                        }
                        _ => {
                            writeln!(s,"if {var}.is_empty() && self.missing_field(&caps,&{sites:?}) {{return Ok(());}}if {var}.len()!=1 {{return Err({:?}.into());}}let f{n}={var}.pop().unwrap();self.give_values({var});",format!("{} requires one node",f.name)).unwrap();
                        }
                    }
                }
                let fty = match repr {
                    Repr::Text | Repr::Node => "u32",
                    Repr::OptText | Repr::OptNode => "Option<u32>",
                    Repr::TextList | Repr::NodeList => "Vec<u32>",
                };
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
                            format!("f{n}.map(|_|(\"text\",\"text\"))")
                        } else {
                            format!(
                                "f{n}.map(|v|(self.tree.type_id_of(v),self.tree.type_name_of(v)))"
                            )
                        }
                    } else if text_type(&f.value_type) {
                        "Some((\"text\",\"text\"))".into()
                    } else {
                        format!("Some((self.tree.type_id_of(f{n}),self.tree.type_name_of(f{n})))")
                    })
                };
                let left = key(&reference.name)?;
                let right = key(&own.name)?;
                // D-028: `@typeof` 違反の mappingError 文字列は旧 Java 実装の観測を固定した
                // 契約値（`corpus/oracle/typeof.json`）。互換のためここでだけ例外クラス名を持つ。
                // runtime 内部で作る他の mapping 失敗には付けない。
                writeln!(s,"if let (Some(expected),Some(actual))=({left},{right}) {{if expected.0!=actual.0 {{let (expected,actual)=(expected.1,actual.1);return Err(format!({:?}));}}}}",format!("java.lang.IllegalArgumentException: @typeof constraint violated: {} must be same type as {}, expected {{expected}} but got {{actual}}",own.name,reference.name)).unwrap();
            }
            s.push_str("self.give_captures(caps);let span=self.span(span);");
            let mut slots = Vec::new();
            for (n, f) in fields.iter().enumerate() {
                match Repr::of(f) {
                    Repr::Text | Repr::Node => slots.push(format!("f{n}")),
                    Repr::OptText | Repr::OptNode => {
                        slots.push(format!("f{n}.unwrap_or(tree::NONE)"))
                    }
                    Repr::TextList | Repr::NodeList => {
                        write!(s, "let l{n}=self.tree.list(&f{n});self.give_values(f{n});")
                            .unwrap();
                        slots.push(format!("l{n}[0]"));
                        slots.push(format!("l{n}[1]"));
                    }
                }
            }
            writeln!(
                s,
                "let node=self.tree.record(tree::{},{i},span,&[{}]);out.push(node);Ok(())}},",
                kind_const(ty),
                slots.join(",")
            )
            .unwrap();
        }
        AstType::Enum { variants, .. } => {
            writeln!(s, "{i}=>{{let value:u32=match self.trimmed(span) {{").unwrap();
            let order = enum_variants(variants);
            let mut seen = std::collections::BTreeSet::new();
            for variant in variants.iter().filter(|v| seen.insert(&v.literal)) {
                let index = order
                    .iter()
                    .position(|v| v.rust_name == variant.rust_name)
                    .unwrap();
                writeln!(s, "{:?}=>{index},", variant.literal).unwrap();
            }
            writeln!(s,"_=>return Err(\"enum literal mismatch\".into()),}};let span=self.span(span);let node=self.tree.enumeration(tree::{},{i},span,value);out.push(node);Ok(())}},",kind_const(ty)).unwrap();
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
                clauses.insert("tree::KIND_TEXT".to_owned());
                continue;
            }
            Pending::Value(ValueType::Null {}) => {
                clauses.insert("tree::KIND_NULL".to_owned());
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
            clauses.insert(format!("tree::{}", kind_const(ty)));
        }
    }
    if clauses.is_empty() {
        "false".into()
    } else {
        let kinds = clauses
            .into_iter()
            .map(|k| format!("k=={k}"))
            .collect::<Vec<_>>()
            .join(" || ");
        format!("{{let k=self.tree.kind(*{value});{kinds}}}")
    }
}
