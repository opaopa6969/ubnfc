//! Classic docs/zero-repetition.md: 上限ゼロは leading edge と FIRST を子から取らない。
use ubnfc_front::{ir::*, pipeline::compile};

#[test]
fn a_zero_count_self_reference_is_empty_but_a_following_reference_still_recurses() {
    let text = "grammar Zero { @root Root ::= Root{0} 'x'; }";
    let ir = compile("/virtual/zero.ubnf", &|_| Ok(text.into()))
        .unwrap()
        .ir;
    let zero = ir
        .semantics
        .expressions
        .iter()
        .find(|e| matches!(e.node, ExprNode::Repeat { max: Some(0), .. }))
        .unwrap();
    let analysis = ir
        .analysis
        .expressions
        .iter()
        .find(|a| a.expr_id == zero.id)
        .unwrap();
    assert_eq!(
        analysis.may_start.zero_consume_condition,
        MayStartZeroConsumeCondition::Always
    );
    assert!(analysis.may_start.ranges.is_empty());
    assert_eq!(
        analysis.cursor_transition.consumed_mode,
        CursorProgress::Preserve
    );
    assert_eq!(
        analysis.cursor_transition.matched_mode,
        CursorProgress::Preserve
    );
    assert_eq!(
        analysis.cursor_transition.flag_effect,
        CursorTransitionFlagEffect::Preserve
    );
    let text = "grammar Zero { @root Root ::= Root{0} Root | 'x'; }";
    assert!(compile("/virtual/zero.ubnf", &|_| Ok(text.into())).is_err());
}

#[test]
fn invalid_nested_bounds_are_validated_even_below_a_zero_count() {
    let text = "grammar Zero { @root Root ::= ('x'{2,1}){0}; }";
    assert!(compile("/virtual/zero.ubnf", &|_| Ok(text.into())).is_err());
}
