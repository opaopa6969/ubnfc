use super::Context;
use crate::ir::*;

pub(super) fn transition(cx: &Context<'_>, i: usize) -> CursorTransition {
    let e = &cx.sem.expressions[i];
    let f = &cx.facts[i];
    let progress = |possible: bool, min: u64| {
        if !possible {
            CursorProgress::Preserve
        } else if min > 0 && min != u64::MAX {
            CursorProgress::Advance
        } else {
            CursorProgress::MayAdvance
        }
    };
    let token = if let ExprNode::TokenRef { symbol_id } = &e.node {
        Some(&cx.sem.tokens[cx.tokens[symbol_id]].spec)
    } else {
        None
    };
    let primitive = matches!(e.node, ExprNode::Literal { .. } | ExprNode::Any { .. })
        || matches!(
            token,
            Some(
                TokenSpec::Any {}
                    | TokenSpec::Negation { .. }
                    | TokenSpec::CharRange { .. }
                    | TokenSpec::CaseInsensitive { .. }
            )
        );
    let transaction = matches!(
        token,
        Some(
            TokenSpec::Lookahead { .. }
                | TokenSpec::NegativeLookahead { .. }
                | TokenSpec::Empty {}
                | TokenSpec::Eof {}
                | TokenSpec::Until { .. }
                | TokenSpec::Builtin {
                    which: BuiltinToken::EndOfSource,
                    ..
                }
        )
    ) || matches!(
        e.node,
        ExprNode::Seq { .. }
            | ExprNode::Choice { .. }
            | ExprNode::Optional { .. }
            | ExprNode::Repeat { .. }
            | ExprNode::Separated { .. }
            | ExprNode::Lookahead { .. }
            | ExprNode::NegativeLookahead { .. }
            | ExprNode::Empty { .. }
            | ExprNode::Eof { .. }
    );
    CursorTransition {
        consumed_mode: progress(f.consumes, f.min),
        matched_mode: progress(f.matches, f.matched_min),
        matched_on_consume: CursorTransitionMatchedOnConsume,
        flag_effect: if matches!(e.node, ExprNode::Seq { .. })
            || e.execution.entry_flag == ExecutionEntryFlag::SetFalse
        {
            CursorTransitionFlagEffect::SetFalse
        } else if matches!(
            e.node,
            ExprNode::RuleRef { .. } | ExprNode::TokenRef { .. } | ExprNode::Group { .. }
        ) {
            CursorTransitionFlagEffect::Contextual
        } else {
            CursorTransitionFlagEffect::Preserve
        },
        failure: if primitive
            || e.execution.failure_cursor == ExecutionFailureCursor::PrimitiveMayResetMatched
        {
            CursorTransitionFailure::MayResetMatched
        } else if transaction
            || e.execution.failure_cursor == ExecutionFailureCursor::RestoreTransaction
        {
            CursorTransitionFailure::Restore
        } else {
            CursorTransitionFailure::Contextual
        },
    }
}
