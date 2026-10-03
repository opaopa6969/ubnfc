use crate::EmitError;
use std::collections::{BTreeMap, BTreeSet};
use ubnfc_front::ir::*;

pub(crate) fn unsupported(feature: &str, detail: impl Into<String>) -> EmitError {
    EmitError::Unsupported {
        feature: feature.into(),
        detail: detail.into(),
    }
}
/// UBNF 表層言語のバージョン上限（D-038）。表層の新機能は front が v1 の正規形へ
/// 脱糖するので backend の実装は変わらないが、**未知のバージョンは明示拒否する**。
pub(crate) const MAX_LANGUAGE_VERSION: u64 = 2;

pub(crate) fn validate(ir: &GrammarIr) -> Result<(), EmitError> {
    if ir.language_version < 1 || ir.language_version > MAX_LANGUAGE_VERSION {
        return Err(unsupported(
            "languageVersion",
            format!("unknown UBNF language version {}", ir.language_version),
        ));
    }
    for feature in &ir.required_features {
        if matches!(feature, RequiredFeature::LeftRecursionRewrite) {
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
            ExprNode::Seq { .. } | ExprNode::Error { .. } => ExecutionTransaction::Once,
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
    }
    for token in &ir.semantics.tokens {
        if matches!(token.spec, TokenSpec::Regex { .. })
            && !ir.plan.lexical.iter().any(|p| {
                p.token_id == token.symbol_id
                    && p.strategy == LexicalPlanStrategy::Dfa
                    && p.dfa.is_some()
            })
        {
            return Err(unsupported(
                "regex",
                format!(
                    "{} requires an analyzed DFA; Java Pattern fallback is unavailable",
                    token.symbol_id
                ),
            ));
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
    validate_left_recursion(ir)?;
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

// directRewritten describes the original grammar, not the executable rule body.
// Start at current bodies so retained pre-lowering expressions are not rejected.
fn validate_left_recursion(ir: &GrammarIr) -> Result<(), EmitError> {
    let expressions: BTreeMap<_, _> = ir
        .semantics
        .expressions
        .iter()
        .map(|e| (&e.id, &e.node))
        .collect();
    let rules: BTreeMap<_, _> = ir
        .semantics
        .rules
        .iter()
        .map(|r| (&r.symbol_id, &r.body))
        .collect();
    let tokens: BTreeMap<_, _> = ir
        .semantics
        .tokens
        .iter()
        .map(|t| (&t.symbol_id, &t.spec))
        .collect();
    // Analysis may conservatively mark consuming expressions nullable. Recompute
    // from semantics, also avoiding stale analysis on manually constructed IR.
    let mut nullable = BTreeSet::new();
    loop {
        let before = nullable.len();
        for (id, node) in &expressions {
            let empty = match node {
                ExprNode::Seq { items } => items.iter().all(|id| nullable.contains(id)),
                ExprNode::Choice { alternatives, .. } => {
                    alternatives.iter().any(|id| nullable.contains(id))
                }
                ExprNode::RuleRef { symbol_id } => nullable.contains(rules[symbol_id]),
                ExprNode::Group { child } => nullable.contains(child),
                ExprNode::Repeat { child, min, .. } => *min == 0 || nullable.contains(child),
                ExprNode::Separated { element, min, .. } => *min == 0 || nullable.contains(element),
                ExprNode::Literal { value, .. } => value.is_empty(),
                ExprNode::TokenRef { symbol_id } => match tokens[symbol_id] {
                    TokenSpec::Declarative { expression } => expression.nullable(),
                    TokenSpec::Builtin { which, .. } => *which == BuiltinToken::EndOfSource,
                    TokenSpec::CaseInsensitive { word, .. } => word.is_empty(),
                    TokenSpec::Eof {}
                    | TokenSpec::Empty {}
                    | TokenSpec::Until { .. }
                    | TokenSpec::Lookahead { .. }
                    | TokenSpec::NegativeLookahead { .. }
                    | TokenSpec::Extern { .. } => true,
                    _ => false,
                },
                ExprNode::Optional { .. }
                | ExprNode::Lookahead { .. }
                | ExprNode::NegativeLookahead { .. }
                | ExprNode::Eof {}
                | ExprNode::Empty {} => true,
                ExprNode::Any {} | ExprNode::Error { .. } => false,
            };
            if empty {
                nullable.insert(*id);
            }
        }
        if nullable.len() == before {
            break;
        }
    }
    for rule in &ir.semantics.rules {
        let mut pending = vec![&rule.body];
        let mut seen = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !seen.insert(id) {
                continue;
            }
            let node = expressions[id];
            if let ExprNode::RuleRef { symbol_id } = node {
                if *symbol_id == rule.symbol_id {
                    return Err(unsupported("unloweredLeftRecursion", symbol_id));
                }
                pending.push(rules[symbol_id]);
            } else {
                for child in node.children() {
                    pending.push(child);
                    if matches!(node, ExprNode::Seq { .. } | ExprNode::Separated { .. })
                        && !nullable.contains(child)
                    {
                        break;
                    }
                }
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

// Check nested value types before the shared reference validator recurses into
// them. Expression reachability itself is iterative in Generator::visit.
pub(crate) fn budget(ir: &GrammarIr) -> Result<(), EmitError> {
    let limit = |resource, limit| EmitError::ResourceLimit { resource, limit };
    if ir.semantics.expressions.len() > 100_000 {
        return Err(limit("expressions", 100_000));
    }
    if ir.semantics.rules.len() > 10_000 {
        return Err(limit("rules", 10_000));
    }
    if ir.semantics.ast_types.len() > 256 {
        return Err(limit("AST types", 256));
    }
    for e in &ir.semantics.expressions {
        if e.node.children().len() > 256 {
            return Err(limit("expression children", 256));
        }
    }
    let mut pending = Vec::new();
    for ty in &ir.semantics.ast_types {
        if let AstType::Record { fields, .. } = ty {
            if fields.len() > 256 {
                return Err(limit("record fields", 256));
            }
            pending.extend(fields.iter().map(|f| (&f.value_type, 0)));
        }
    }
    pending.extend(ir.semantics.captures.iter().map(|c| (&c.value_type, 0)));
    pending.extend(
        ir.semantics
            .common_fields
            .iter()
            .map(|c| (&c.field.value_type, 0)),
    );
    while let Some((ty, depth)) = pending.pop() {
        if depth >= 64 {
            return Err(limit("value type depth", 64));
        }
        match ty {
            ValueType::Mixed { members } => pending.extend(members.iter().map(|t| (t, depth + 1))),
            ValueType::Optional { element } | ValueType::List { element } => {
                pending.push((element, depth + 1))
            }
            _ => {}
        }
    }
    Ok(())
}
