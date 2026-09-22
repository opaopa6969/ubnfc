//! lower/analysis が生成した型契約を、backend に渡す前に検査する。
use super::*;
use std::collections::BTreeSet;

pub(super) fn check(ir: &GrammarIr) -> Result<(), PipelineError> {
    let mut errors = ir::validate_references(ir);
    check_semantics(&ir.semantics, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(PipelineError::Internal(errors))
    }
}

fn error(errors: &mut Vec<IrError>, path: &str, message: &str) {
    errors.push(IrError::Validation {
        path: path.into(),
        message: message.into(),
    });
}

// front の通常出力は要素型 + cardinality。ValueType に外側の container も
// 明記されている場合、二つの指定が矛盾してはならない。
fn element_type(value: &ValueType) -> &ValueType {
    match value {
        ValueType::Optional { element } | ValueType::List { element } => element,
        _ => value,
    }
}

fn check_cardinality(
    cardinality: Cardinality,
    value: &ValueType,
    path: &str,
    errors: &mut Vec<IrError>,
) {
    let declared = match value {
        ValueType::Optional { .. } => Cardinality::Optional,
        ValueType::List { .. } => Cardinality::List,
        _ => return,
    };
    if cardinality != declared {
        error(
            errors,
            path,
            "cardinality と ValueType の container が不一致",
        );
    }
}

fn conversion_matches(conversion: Conversion, value: &ValueType) -> bool {
    match (conversion, element_type(value)) {
        (Conversion::Text, ValueType::Text {})
        | (Conversion::Number, ValueType::Number { .. })
        | (Conversion::Node, ValueType::Node { .. })
        | (Conversion::Mixed, ValueType::Mixed { .. }) => true,
        // 共有結合の leafFallback は null、通常の operand は node。
        (Conversion::Node, ValueType::Mixed { members }) => {
            !members.is_empty()
                && members
                    .iter()
                    .all(|m| matches!(m, ValueType::Node { .. } | ValueType::Null {}))
        }
        _ => false,
    }
}

fn assignable(source: &ValueType, target: &ValueType, semantics: &Semantics) -> bool {
    fn node_member(source: &str, target: &str, s: &Semantics, seen: &mut BTreeSet<String>) -> bool {
        if source == target {
            return true;
        }
        if !seen.insert(target.into()) {
            return false;
        }
        s.ast_types.iter().any(|t| match t {
            AstType::TransparentUnion { id, variants, .. } if id == target => {
                variants.iter().any(|v| node_member(source, v, s, seen))
            }
            _ => false,
        })
    }
    if source == target {
        return true;
    }
    match (source, target) {
        (ValueType::Mixed { members }, _) => {
            members.iter().all(|m| assignable(m, target, semantics))
        }
        (_, ValueType::Mixed { members }) => {
            members.iter().any(|m| assignable(source, m, semantics))
        }
        (ValueType::Node { type_id: source }, ValueType::Node { type_id: target }) => {
            node_member(source, target, semantics, &mut BTreeSet::new())
        }
        _ => false,
    }
}

fn check_semantics(s: &Semantics, errors: &mut Vec<IrError>) {
    for (i, t) in s.ast_types.iter().enumerate() {
        if let AstType::Record { fields, .. } = t {
            for (j, field) in fields.iter().enumerate() {
                check_cardinality(
                    field.cardinality,
                    &field.value_type,
                    &format!("/semantics/astTypes/{i}/fields/{j}/valueType"),
                    errors,
                );
            }
        }
    }
    for (i, common) in s.common_fields.iter().enumerate() {
        check_cardinality(
            common.field.cardinality,
            &common.field.value_type,
            &format!("/semantics/commonFields/{i}/field/valueType"),
            errors,
        );
    }
    for (i, capture) in s.captures.iter().enumerate() {
        let path = format!("/semantics/captures/{i}");
        check_cardinality(
            capture.cardinality,
            &capture.value_type,
            &format!("{path}/valueType"),
            errors,
        );
        if !conversion_matches(capture.conversion, &capture.value_type) {
            error(
                errors,
                &format!("{path}/conversion"),
                "capture の変換と型が不一致",
            );
        }
    }
    for (i, recipe) in s.recipes.iter().enumerate() {
        let path = format!("/semantics/recipes/{i}");
        let result = s.ast_types.iter().find(|t| {
            let (AstType::Record { id, .. }
            | AstType::TransparentUnion { id, .. }
            | AstType::Enum { id, .. }) = t;
            Some(id) == recipe.result_type_id.as_ref()
        });
        let fields = match (recipe.kind, result) {
            (
                RecipeKind::Construct | RecipeKind::AssocLists | RecipeKind::FrontendTransform,
                Some(AstType::Record { fields, .. }),
            ) => fields.as_slice(),
            (RecipeKind::Enum, Some(AstType::Enum { .. }))
            | (RecipeKind::Transparent, Some(AstType::TransparentUnion { .. }))
            | (RecipeKind::Text, None)
            | (RecipeKind::Skip, None)
            | (RecipeKind::FrontendTransform, _) => &[],
            _ => {
                error(
                    errors,
                    &format!("{path}/resultTypeId"),
                    "recipe と結果型の種類が不一致",
                );
                continue;
            }
        };
        // frontendTransform の結果は独立の版管理された変換契約で規定する。
        // field があれば site 対応検査は共通に適用する。
        if recipe.kind != RecipeKind::FrontendTransform || !fields.is_empty() {
            let expected: BTreeSet<_> = fields.iter().map(|f| &f.name).collect();
            let actual: BTreeSet<_> = recipe.fields.iter().map(|f| &f.field).collect();
            if expected != actual || actual.len() != recipe.fields.len() {
                error(
                    errors,
                    &format!("{path}/fields"),
                    "recipe の field が AST schema と不一致",
                );
            }
        }
        for (j, field) in recipe.fields.iter().enumerate() {
            let path = format!("{path}/fields/{j}");
            let schema = fields.iter().find(|f| f.name == field.field);
            if let Some(schema) = schema {
                if field.cardinality != schema.cardinality {
                    error(
                        errors,
                        &format!("{path}/cardinality"),
                        "recipe と field の cardinality が不一致",
                    );
                }
                if !conversion_matches(field.conversion, &schema.value_type) {
                    error(
                        errors,
                        &format!("{path}/conversion"),
                        "recipe の変換と field 型が不一致",
                    );
                }
            }
            let captures: Vec<_> = s
                .captures
                .iter()
                .filter(|c| c.rule_id == recipe.rule_id && c.name == field.field)
                .collect();
            let expected: BTreeSet<_> = captures.iter().map(|c| &c.site_id).collect();
            let actual: BTreeSet<_> = field.site_ids.iter().collect();
            if actual.is_empty() || expected != actual || actual.len() != field.site_ids.len() {
                error(
                    errors,
                    &format!("{path}/siteIds"),
                    "recipe と capture site の対応が不一致",
                );
            }
            for capture in captures {
                if capture.cardinality != field.cardinality {
                    error(
                        errors,
                        &format!("{path}/cardinality"),
                        "recipe と capture の cardinality が不一致",
                    );
                }
                if schema.is_some_and(|f| {
                    !assignable(
                        element_type(&capture.value_type),
                        element_type(&f.value_type),
                        s,
                    )
                }) {
                    error(
                        errors,
                        &format!("{path}/siteIds"),
                        "capture 型を field 型へ代入できません",
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> GrammarIr {
        compile("contract.ubnf", &|_| Ok(
            "grammar G { token N = NumberParser @root @mapping(Value,params=[value]) First ::= 'f' Factor @value; @mapping(Value,params=[value]) Second ::= 's' Leaf @value; Factor ::= 'a' | Leaf; @mapping(Leaf,params=[n]) Leaf ::= N @n; }".into()
        )).unwrap().ir
    }

    fn rejected(mut ir: GrammarIr, mutate: impl FnOnce(&mut GrammarIr), suffix: &str) {
        mutate(&mut ir);
        let Err(PipelineError::Internal(errors)) = check(&ir) else {
            panic!("不整合 IR が pipeline の内部エラーになりません");
        };
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, IrError::Validation {path, ..} if path.ends_with(suffix))),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_conversion_cardinality_and_site_mismatches() {
        let ir = fixture();
        assert!(check(&ir).is_ok());
        rejected(
            ir.clone(),
            |ir| ir.semantics.recipes[1].fields[0].conversion = Conversion::Node,
            "/conversion",
        );
        rejected(
            ir.clone(),
            |ir| ir.semantics.captures[0].conversion = Conversion::Number,
            "/conversion",
        );
        rejected(
            ir.clone(),
            |ir| ir.semantics.recipes[0].fields[0].cardinality = Cardinality::List,
            "/cardinality",
        );
        rejected(
            ir.clone(),
            |ir| ir.semantics.captures[0].cardinality = Cardinality::Optional,
            "/cardinality",
        );
        rejected(
            ir.clone(),
            |ir| ir.semantics.recipes[0].fields[0].site_ids.clear(),
            "/siteIds",
        );
        rejected(
            ir.clone(),
            |ir| {
                let wrong = ir.semantics.recipes[2].fields[0].site_ids[0].clone();
                ir.semantics.recipes[0].fields[0].site_ids = vec![wrong];
            },
            "/siteIds",
        );
        rejected(
            ir.clone(),
            |ir| {
                let fields = &mut ir.semantics.recipes[0].fields;
                let duplicate = fields[0].site_ids[0].clone();
                fields[0].site_ids.push(duplicate);
            },
            "/siteIds",
        );
        rejected(
            ir.clone(),
            |ir| ir.semantics.recipes[0].fields.clear(),
            "/fields",
        );
        rejected(
            ir,
            |ir| {
                // ID は存在していても、局所 number は共有 text|node 型に代入不可。
                ir.semantics.captures[0].value_type = ValueType::Number {
                    java: ValueTypeNumberJava,
                    rust: ValueTypeNumberRust,
                };
                ir.semantics.captures[0].conversion = Conversion::Number;
            },
            "/siteIds",
        );
    }

    #[test]
    fn rejects_contradictory_container_annotations() {
        for (cardinality, value) in [
            (
                Cardinality::Scalar,
                ValueType::List {
                    element: Box::new(ValueType::Text {}),
                },
            ),
            (
                Cardinality::List,
                ValueType::Optional {
                    element: Box::new(ValueType::Text {}),
                },
            ),
            (
                Cardinality::Optional,
                ValueType::List {
                    element: Box::new(ValueType::Text {}),
                },
            ),
        ] {
            rejected(
                fixture(),
                |ir| {
                    let fields = ir
                        .semantics
                        .ast_types
                        .iter_mut()
                        .find_map(|t| match t {
                            AstType::Record { fields, .. } => Some(fields),
                            _ => None,
                        })
                        .unwrap();
                    fields[0].cardinality = cardinality;
                    fields[0].value_type = value.clone();
                },
                "/valueType",
            );
            rejected(
                fixture(),
                |ir| {
                    ir.semantics.captures[0].cardinality = cardinality;
                    ir.semantics.captures[0].value_type = value;
                },
                "/valueType",
            );
        }
    }
}
