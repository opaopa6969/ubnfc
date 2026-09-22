use crate::EmitError;
use std::collections::BTreeSet;
use ubnfc_front::ir::*;

pub(crate) fn unsupported(feature: &str, detail: impl Into<String>) -> EmitError {
    EmitError::Unsupported {
        feature: feature.into(),
        detail: detail.into(),
    }
}
pub(crate) fn validate(ir: &GrammarIr) -> Result<(), EmitError> {
    for feature in &ir.required_features {
        if matches!(
            feature,
            RequiredFeature::Recovery
                | RequiredFeature::Eval
                | RequiredFeature::Catalog
                | RequiredFeature::Typeof
                | RequiredFeature::LeftRecursionRewrite
        ) {
            return Err(unsupported(
                serde_json::to_value(feature).unwrap().as_str().unwrap(),
                "not implemented in this backend",
            ));
        }
    }
    let errors = validate_references(ir);
    if !errors.is_empty() {
        return Err(EmitError::InvalidIr(
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }
    if ir
        .semantics
        .recipes
        .iter()
        .any(|r| r.kind == RecipeKind::FrontendTransform)
    {
        return Err(unsupported(
            "structuredOccurrences.frontendTransform",
            "versioned frontend transforms are not implemented",
        ));
    }
    for expr in &ir.semantics.expressions {
        let expected = match expr.node {
            ExprNode::Seq { .. } => ExecutionTransaction::Once,
            ExprNode::Choice { .. } => ExecutionTransaction::PerAlternative,
            ExprNode::Repeat { .. } | ExprNode::Optional { .. } => ExecutionTransaction::Occurs,
            ExprNode::Separated { .. } => ExecutionTransaction::Once,
            ExprNode::Group { .. } | ExprNode::RuleRef { .. } | ExprNode::TokenRef { .. } => {
                ExecutionTransaction::Delegate
            }
            ExprNode::Lookahead { .. } | ExprNode::NegativeLookahead { .. } => {
                ExecutionTransaction::Once
            }
            _ => ExecutionTransaction::None,
        };
        if expr.execution.transaction != expected {
            return Err(unsupported(
                "execution.transaction",
                format!(
                    "{}: {:?} for {:?}",
                    expr.id, expr.execution.transaction, expr.node
                ),
            ));
        }
        if let ExprNode::Literal {
            case_sensitive: false,
            value,
            ..
        } = &expr.node
        {
            if !value.is_ascii() {
                return Err(unsupported(
                    "caseInsensitive.unicode",
                    "Java simple case folding requires a versioned Unicode table",
                ));
            }
        }
    }
    for token in &ir.semantics.tokens {
        match &token.spec {
            TokenSpec::Regex { .. } => {
                if !ir.plan.lexical.iter().any(|p| {
                    p.token_id == token.symbol_id
                        && p.strategy == LexicalPlanStrategy::Dfa
                        && p.dfa.is_some()
                }) {
                    return Err(unsupported(
                        "regex",
                        format!(
                            "{} requires an analyzed DFA; Java Pattern fallback is unavailable",
                            token.symbol_id
                        ),
                    ));
                }
            }
            TokenSpec::CaseInsensitive { word, .. } if !word.is_ascii() => {
                return Err(unsupported("caseInsensitive.unicode", &token.symbol_id))
            }
            _ => {}
        }
    }
    for plan in &ir.plan.lexical {
        if plan.strategy == LexicalPlanStrategy::JavaPattern {
            return Err(unsupported("regex.javaPattern", &plan.token_id));
        }
        if plan.strategy == LexicalPlanStrategy::Dfa && plan.dfa.is_none() {
            return Err(EmitError::InvalidIr(format!(
                "missing DFA: {}",
                plan.token_id
            )));
        }
        if let Some(dfa) = &plan.dfa {
            for state in &dfa.states {
                let mut ranges = state
                    .transitions
                    .iter()
                    .map(|t| (t.range.start, t.range.end))
                    .collect::<Vec<_>>();
                ranges.sort_unstable();
                if ranges.iter().any(|(a, b)| a > b || *b > 0x10ffff)
                    || ranges.windows(2).any(|w| w[0].1 >= w[1].0)
                {
                    return Err(EmitError::InvalidIr(format!(
                        "overlapping/invalid DFA transitions: {}",
                        plan.token_id
                    )));
                }
            }
        }
    }
    let mut memo_ids = BTreeSet::new();
    for p in &ir.plan.operators {
        if p.strategy == OperatorPlanStrategy::PrecedenceClimbing {
            return Err(unsupported(
                "operators.precedenceClimbing",
                "tierFunctions preserves rule/trivia boundaries",
            ));
        }
    }
    for a in &ir.analysis.rules {
        if a.left_recursion != RuleAnalysisLeftRecursion::None {
            return Err(unsupported("leftRecursionRewrite", &a.rule_id));
        }
    }
    for e in &ir.plan.memo.entries {
        if !memo_ids.insert(&e.expr_id) {
            return Err(EmitError::InvalidIr(format!(
                "duplicate memo entry: {}",
                e.expr_id
            )));
        }
        let a = ir
            .analysis
            .expressions
            .iter()
            .find(|a| a.expr_id == e.expr_id)
            .unwrap();
        if (e.cache_success && !a.memo_safety.success)
            || (e.cache_failure && !a.memo_safety.failure)
        {
            return Err(EmitError::InvalidIr(format!("unsafe memo: {}", e.expr_id)));
        }
        if ((!e.cache_success && a.memo_safety.success)
            || (!e.cache_failure && a.memo_safety.failure))
            && e.reason != MemoEntryReason::UnsafeExternClosure
            && e.omission_proof
                .as_ref()
                .is_none_or(|p| p.kind == ProofKind::None)
        {
            return Err(EmitError::InvalidIr(format!(
                "missing memo omission proof: {}",
                e.expr_id
            )));
        }
    }
    let mut names = BTreeSet::new();
    for ty in &ir.semantics.ast_types {
        let name = crate::ast::type_ident(ty);
        if !names.insert(name.clone()) {
            return Err(EmitError::InvalidIr(format!(
                "Rust type name collision: {name}"
            )));
        }
        if let AstType::Record { fields, .. } = ty {
            let mut fields_seen = BTreeSet::from(["span".to_owned(), "node_id".to_owned()]);
            for f in fields {
                validate_value_type(&f.value_type)?;
                if !fields_seen.insert(crate::ast::ident(&f.name)) {
                    return Err(EmitError::InvalidIr(format!(
                        "Rust field collision: {}",
                        f.name
                    )));
                }
            }
        }
        if let AstType::Enum { variants, .. } = ty {
            let mut variants_seen = BTreeSet::new();
            let mut literals_seen = BTreeSet::new();
            for variant in variants {
                if !variants_seen.insert(crate::ast::ident(&variant.rust_name))
                    || !literals_seen.insert(&variant.literal)
                {
                    return Err(EmitError::InvalidIr(format!(
                        "duplicate enum name/literal: {}",
                        ty.id()
                    )));
                }
            }
        }
    }
    for common in &ir.semantics.common_fields {
        for id in &common.variant_type_ids {
            if !ir.semantics.ast_types.iter().any(|t|matches!(t,AstType::Record{id:found,fields,..} if found==id && fields.contains(&common.field))) {
                return Err(EmitError::InvalidIr(format!("commonField {} missing or incompatible in {id}",common.field.name)));
            }
        }
    }
    for p in &ir.plan.choices {
        let expr = ir
            .semantics
            .expressions
            .iter()
            .find(|e| e.id == p.expr_id)
            .unwrap();
        if !matches!(&expr.node,ExprNode::Choice{alternatives,..} if p.candidate_order==*alternatives)
        {
            return Err(EmitError::InvalidIr(format!(
                "choice plan must retain all candidates in declaration order: {}",
                p.expr_id
            )));
        }
    }
    for p in &ir.semantics.trivia.policies {
        for d in &p.effective_delimiter_set {
            if matches!(d,Delimiter::LineComment{open,..} if open.is_empty())
                || matches!(d,Delimiter::BlockComment{open,close} if open.is_empty()||close.is_empty())
            {
                return Err(EmitError::InvalidIr(format!(
                    "empty trivia delimiter: {}",
                    p.id
                )));
            }
        }
    }
    Ok(())
}

fn validate_value_type(ty: &ValueType) -> Result<(), EmitError> {
    match ty {
        ValueType::Optional { .. } | ValueType::List { .. } => Err(unsupported(
            "deferredMapping.valueType",
            "use field cardinality for Option/List; nested value containers are not implemented",
        )),
        ValueType::Mixed { members } => {
            if members.is_empty() {
                return Err(EmitError::InvalidIr("empty mixed value type".into()));
            }
            for member in members {
                validate_value_type(member)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
