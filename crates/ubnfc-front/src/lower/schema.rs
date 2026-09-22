use super::*;
use r::ResolvedAtom as A;
use std::collections::BTreeSet;

pub(super) fn prepare(cx: &mut Context<'_>) {
    for rule in cx.grammar.grammars.iter().flat_map(|g| &g.rules) {
        if skipped(rule) {
            continue;
        }
        let name = if rule
            .annotations
            .iter()
            .any(|a| matches!(a, a::Annotation::Enum { .. }))
        {
            Some(rule.name.as_str())
        } else {
            mapping(rule).map(|(n, _)| n)
        };
        if let Some(name) = name {
            cx.types
                .insert(rule.id, format!("{}.{name}", cx.ast_root(rule.id)));
        }
    }
}
fn variants(cx: &Context<'_>, rule: &r::ResolvedRule) -> Vec<String> {
    let Some((name, params)) = mapping(rule) else {
        return vec![];
    };
    if !params.is_empty() || name.contains('.') {
        return vec![];
    }
    let mut out = vec![];
    for seq in &rule.body.alternatives {
        let [e] = seq.as_slice() else {
            return vec![];
        };
        let A::Ref { symbol, .. } = &e.atom else {
            return vec![];
        };
        let Some(target) = cx.rule(*symbol) else {
            return vec![];
        };
        let Some((target_name, _)) = mapping(target) else {
            return vec![];
        };
        if skipped(target)
            || target_name == name
            || (target_name.contains('.') && !target_name.starts_with(&format!("{name}.")))
        {
            return vec![];
        }
        let id = cx.types[symbol].clone();
        if !out.contains(&id) {
            out.push(id);
        }
    }
    out
}
pub(super) fn finish(cx: &mut Context<'_>) {
    for grammar in &cx.grammar.grammars {
        let mut types = vec![];
        let root = format!("{}AST", grammar.name);
        let mut done = BTreeSet::new();
        for rule in &grammar.rules {
            let Some(id) = cx.types.get(&rule.id).cloned() else {
                continue;
            };
            if !done.insert(id.clone()) {
                continue;
            }
            if rule
                .annotations
                .iter()
                .any(|a| matches!(a, a::Annotation::Enum { .. }))
            {
                let literals: Vec<_> = rule
                    .body
                    .alternatives
                    .iter()
                    .flatten()
                    .filter_map(|e| {
                        if let A::Literal { value, .. } = &e.atom {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect();
                types.push(AstType::Enum {
                    enclosing_type_id: None,
                    id,
                    name: rule.name.clone(),
                    rule_id: cx.sid(rule.id),
                    variants: literals
                        .into_iter()
                        .map(|literal| {
                            let name = enum_name(literal);
                            EnumVariant {
                                literal: literal.clone(),
                                variant: name.clone(),
                                java_name: name.clone(),
                                rust_name: name,
                            }
                        })
                        .collect(),
                });
                continue;
            }
            let (name, params) = mapping(rule).unwrap();
            let family: Vec<_> = grammar
                .rules
                .iter()
                .filter(|r| !skipped(r) && mapping(r).is_some_and(|(n, _)| n == name))
                .collect();
            let union = variants(cx, rule);
            if family.iter().any(|r| variants(cx, r) != union) {
                cx.errors.push(error(
                    "E-LOWER-SHARED-SCHEMA",
                    format!("{name}: 共有型の record/union または variant が不整合"),
                ));
            }
            let enclosing = name
                .rsplit_once('.')
                .map(|(outer, _)| format!("{root}.{outer}"));
            if !union.is_empty() {
                for r in &family {
                    if let Some(eval) = cx
                        .out
                        .semantics
                        .rules
                        .iter()
                        .find(|x| x.symbol_id == cx.sid(r.id))
                        .and_then(|x| x.eval.as_ref())
                    {
                        let manual=r.annotations.iter().any(|a|matches!(a,a::Annotation::Eval {entries,..} if entries.iter().any(|e|e.key=="strategy"&&e.value=="manual")));
                        if !manual {
                            cx.errors.push(error(
                                "E-LOWER-SUM-EVAL",
                                format!("{name}: 自動 @eval({}) は具象型が必要", eval.kind),
                            ));
                        }
                    }
                }
                types.push(AstType::TransparentUnion {
                    enclosing_type_id: enclosing,
                    id,
                    name: name.into(),
                    variants: union,
                });
                continue;
            }
            let mut fields = vec![];
            let mut shared_spine = None;
            if assoc(rule).is_some() {
                shared_spine = assoc_schema(cx, &family, &id, &root);
            }
            for r in &family {
                if mapping(r).unwrap().1 != params || assoc(r).is_some() != assoc(rule).is_some() {
                    cx.errors.push(error(
                        "E-LOWER-SHARED-SCHEMA",
                        format!("{name}: params の名前/順序または結合種別が不整合"),
                    ));
                }
            }
            for param in params {
                let mut shape = None::<captures::Shape>;
                for r in &family {
                    let Some(next) = cx.fields[&r.id].get(param).cloned() else {
                        cx.errors.push(error(
                            "E-LOWER-MAPPING-CAPTURE",
                            format!("{}: capture {param} がありません", r.name),
                        ));
                        continue;
                    };
                    if let Some(old) = shape.as_mut() {
                        if old.count != next.count {
                            cx.errors.push(error(
                                "E-LOWER-SHARED-SCHEMA",
                                format!("{name}.{param}: cardinality が不整合"),
                            ));
                        }
                        old.value = captures::join(old.value.clone(), next.value);
                    } else {
                        shape = Some(next);
                    }
                }
                let Some(mut shape) = shape else {
                    continue;
                };
                if let Some(operand) = &shared_spine {
                    if param == "left" {
                        shape.value = captures::join(ValueType::Null {}, operand.clone());
                        shape.count = Cardinality::Scalar;
                    }
                    if param == "right" {
                        shape.value = operand.clone();
                        shape.count = Cardinality::List;
                    }
                    // 元の capture の変換も shared assoc レシピで上書きする。
                    if param == "left" || param == "right" {
                        let ids: Vec<_> = family.iter().map(|r| cx.sid(r.id)).collect();
                        for c in &mut cx.out.semantics.captures {
                            if ids.contains(&c.rule_id) && c.name == *param {
                                c.value_type = operand.clone();
                                c.conversion = captures::conversion(operand);
                            }
                        }
                    }
                }
                fields.push(AstField {
                    name: param.clone(),
                    cardinality: shape.count,
                    value_type: shape.value,
                });
            }
            types.push(AstType::Record {
                enclosing_type_id: enclosing,
                id,
                name: name.rsplit('.').next().unwrap().into(),
                fields,
                implements: vec![],
            });
        }
        // dotted mapping の暗黙 owner を宣言順で作る。
        let mut parents: Vec<(String, Vec<String>)> = vec![];
        for t in &types {
            if let AstType::Record {
                id,
                enclosing_type_id: Some(parent),
                ..
            } = t
            {
                if let Some((_, variants)) = parents.iter_mut().find(|(p, _)| p == parent) {
                    variants.push(id.clone());
                } else {
                    parents.push((parent.clone(), vec![id.clone()]));
                }
            }
        }
        for (parent, children) in parents {
            if let Some(AstType::TransparentUnion { variants, .. }) =
                types.iter_mut().find(|t| type_id(t) == parent)
            {
                for c in children {
                    if !variants.contains(&c) {
                        variants.push(c);
                    }
                }
            } else {
                types.push(AstType::TransparentUnion {
                    id: parent.clone(),
                    name: parent
                        .strip_prefix(&format!("{root}."))
                        .unwrap_or(&parent)
                        .into(),
                    enclosing_type_id: None,
                    variants: children,
                });
            }
        }
        let mut members = vec![];
        for t in &types {
            if matches!(t, AstType::Enum { .. }) {
                continue;
            }
            let id = type_id(t);
            if !types.iter().any(|t|matches!(t,AstType::TransparentUnion {variants,..} if variants.iter().any(|v|v==id))){members.push(id.into());}
        }
        let unions: Vec<_> = types
            .iter()
            .filter_map(|t| {
                if let AstType::TransparentUnion { id, variants, .. } = t {
                    Some((id.clone(), variants.clone()))
                } else {
                    None
                }
            })
            .collect();
        for t in &mut types {
            if let AstType::Record { id, implements, .. } = t {
                *implements = unions
                    .iter()
                    .filter(|(_, v)| v.contains(id))
                    .map(|(u, _)| u.clone())
                    .collect();
                if implements.is_empty() {
                    implements.push(root.clone());
                }
            }
        }
        if !members.is_empty() {
            cx.out.semantics.ast_types.push(AstType::TransparentUnion {
                id: root.clone(),
                name: root,
                enclosing_type_id: None,
                variants: members,
            });
        }
        cx.out.semantics.ast_types.extend(types);
        for rule in &grammar.rules {
            recipe(cx, rule);
            common(cx, rule);
        }
    }
}
fn assoc_schema(
    cx: &mut Context<'_>,
    family: &[&r::ResolvedRule],
    id: &str,
    root: &str,
) -> Option<ValueType> {
    let mut mapped = vec![];
    let mut left = vec![];
    let mut right = vec![];
    for rule in family {
        let (l, op, r) = annotations::operator_elements(rule)?;
        for operand in [&l.atom, &r.atom] {
            for t in captures::mapped_types(cx, operand) {
                if !mapped.contains(&t) {
                    mapped.push(t);
                }
            }
        }
        left.push(captures::captured(cx, &l.atom).value);
        right.push(captures::captured(cx, &r.atom).value);
        if family.len() > 1
            && (mapping(rule).unwrap().1 != ["left", "op", "right"]
                || captures::captured(cx, &op.atom).value != ValueType::Text {})
        {
            cx.errors.push(error(
                "E-LOWER-SHARED-ASSOC",
                format!("{id}: params=[left,op,right] と text operator が必要"),
            ));
        }
    }
    let right_heterogeneous = assoc(family[0]) == Some(OperatorAssociativity::Right)
        && mapped.contains(&id.to_string())
        && mapped.len() > 1;
    if family.len() < 2 && !right_heterogeneous {
        return None;
    }
    if mapped.contains(&id.to_string()) {
        return Some(ValueType::Node {
            type_id: if mapped.len() > 1 {
                root.into()
            } else {
                id.into()
            },
        });
    }
    if !mapped.is_empty()
        && (mapped.len() > 1
            || left
                .iter()
                .chain(&right)
                .any(|v| !matches!(v, ValueType::Node { .. })))
    {
        return Some(ValueType::Node {
            type_id: root.into(),
        });
    }
    if left.windows(2).any(|p| p[0] != p[1]) || right.windows(2).any(|p| p[0] != p[1]) {
        cx.errors.push(error(
            "E-LOWER-SHARED-ASSOC",
            format!("{id}: scalar operand 型が不整合"),
        ));
    }
    None
}
fn recipe(cx: &mut Context<'_>, rule: &r::ResolvedRule) {
    let id = cx.sid(rule.id);
    let result = cx.types.get(&rule.id).cloned();
    let union = result.as_ref().is_some_and(|id| {
        cx.out
            .semantics
            .ast_types
            .iter()
            .any(|t| matches!(t,AstType::TransparentUnion {id:t,..} if t==id))
    });
    let kind = if skipped(rule) {
        RecipeKind::Skip
    } else if rule
        .annotations
        .iter()
        .any(|a| matches!(a, a::Annotation::Enum { .. }))
    {
        RecipeKind::Enum
    } else if assoc(rule).is_some() {
        RecipeKind::AssocLists
    } else if union {
        RecipeKind::Transparent
    } else if result.is_some() {
        RecipeKind::Construct
    } else {
        return;
    };
    let fields = mapping(rule).map_or(vec![], |(_, params)| {
        params
            .iter()
            .map(|p| {
                let captures: Vec<_> = cx
                    .out
                    .semantics
                    .captures
                    .iter()
                    .filter(|c| c.rule_id == id && &c.name == p)
                    .collect();
                let value = captures
                    .iter()
                    .map(|c| c.value_type.clone())
                    .reduce(captures::join)
                    .unwrap_or(ValueType::Text {});
                RecipeField {
                    field: p.clone(),
                    cardinality: cx.fields[&rule.id]
                        .get(p)
                        .map_or(Cardinality::Scalar, |s| s.count),
                    conversion: captures::conversion(&value),
                    site_ids: captures.iter().map(|c| c.site_id.clone()).collect(),
                }
            })
            .collect()
    });
    let leaf=result.as_ref().filter(|t|kind==RecipeKind::AssocLists&&cx.out.semantics.ast_types.iter().any(|a|matches!(a,AstType::Record {id,fields,..} if id==*t&&fields.iter().any(|f|f.name=="left"&&matches!(&f.value_type,ValueType::Mixed {members} if members.contains(&ValueType::Null {})))))).map(|t|LeafFallback {type_id:t.clone(),left:RecipeLeafFallbackLeft,op:RecipeLeafFallbackOp,right:RecipeLeafFallbackRight,span:RecipeLeafFallbackSpan});
    cx.out.semantics.recipes.push(Recipe {
        id: if cx.grammar.grammars.len() == 1 {
            format!("recipe:{}", rule.name)
        } else {
            format!("recipe:{id}")
        },
        rule_id: id,
        kind,
        fields,
        result_type_id: result,
        leaf_fallback: leaf,
        on_failure: RecipeOnFailure,
        phase: RecipePhase,
        transform_id: None,
    });
}
fn common(cx: &mut Context<'_>, rule: &r::ResolvedRule) {
    let Some(id) = cx.types.get(&rule.id) else {
        return;
    };
    let Some(AstType::TransparentUnion { variants, .. }) =
        cx.out.semantics.ast_types.iter().find(|t| type_id(t) == id)
    else {
        return;
    };
    let variants = variants.clone();
    for name in rule.annotations.iter().flat_map(|a| {
        if let a::Annotation::CommonField { fields, .. } = a {
            fields.as_slice()
        } else {
            &[]
        }
    }) {
        let fields: Vec<_> = variants
            .iter()
            .filter_map(|v| {
                cx.out.semantics.ast_types.iter().find_map(|t| match t {
                    AstType::Record { id, fields, .. } if id == v => {
                        fields.iter().find(|f| &f.name == name).cloned()
                    }
                    _ => None,
                })
            })
            .collect();
        if fields.len() != variants.len() || fields.windows(2).any(|p| p[0] != p[1]) {
            cx.errors.push(error(
                "E-LOWER-COMMON-FIELD",
                format!("{id}.{name}: variants の field 型/cardinality が不整合"),
            ));
        } else if let Some(field) = fields.first() {
            cx.out.semantics.common_fields.push(CommonField {
                field: field.clone(),
                interface_type_id: id.clone(),
                variant_type_ids: variants.clone(),
            });
        }
    }
}
fn type_id(t: &AstType) -> &str {
    match t {
        AstType::Record { id, .. }
        | AstType::TransparentUnion { id, .. }
        | AstType::Enum { id, .. } => id,
    }
}
fn enum_name(literal: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for c in literal.chars() {
        if ['-', '_', ' '].contains(&c) {
            out.push('_');
            prev_lower = false;
        } else if c.is_uppercase() && prev_lower {
            out.push('_');
            out.push(c);
            prev_lower = false;
        } else {
            out.extend(c.to_uppercase());
            prev_lower = c.is_lowercase();
        }
    }
    out
}
