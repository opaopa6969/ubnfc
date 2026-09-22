use ubnfc_rt_template::cursor::State;

#[test]
fn consuming_resets_matched_to_consumed() {
    let mut state = State::default();
    state.advance::<false>(3);
    state.advance::<true>(4);
    assert_eq!((state.consumed, state.matched), (3, 7));
    state.advance::<false>(2);
    assert_eq!((state.consumed, state.matched), (5, 5));
}
#[test]
fn copied_checkpoints_restore_both_positions_and_flags() {
    let mut state = State {
        consumed: 2,
        matched: 6,
        invert: true,
        reset: false,
    };
    let outer = state;
    state.advance::<false>(3);
    let inner = state;
    state.advance::<true>(4);
    assert_eq!(state.matched, 9);
    state = inner;
    assert_eq!((state.consumed, state.matched), (5, 5));
    state = outer;
    assert_eq!((state.consumed, state.matched), (2, 6));
    assert!(state.invert);
    assert!(!state.reset);
}
#[test]
fn advance_zero_sets_matched_to_consumed_only_in_consume_mode() {
    let mut state = State {
        consumed: 1,
        matched: 3,
        ..State::default()
    };
    state.advance::<true>(0);
    assert_eq!((state.consumed, state.matched), (1, 3));
    state.advance::<false>(0);
    assert_eq!((state.consumed, state.matched), (1, 1));
}
#[test]
fn begin_and_commit_preserve_parent_flags() {
    let parent = State {
        consumed: 2,
        matched: 7,
        invert: true,
        reset: false,
    };
    assert_eq!(parent.begin().matched, 7);
    let child = State {
        consumed: 3,
        matched: 9,
        invert: false,
        reset: true,
    };
    assert_eq!(child.begin().matched, 3);
    let result = parent.commit(child);
    assert_eq!(result.position::<false>(), 3);
    assert_eq!(result.position::<true>(), 9);
    assert!(result.invert);
    assert!(!result.reset);
}
