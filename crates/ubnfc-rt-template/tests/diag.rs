use ubnfc_rt_template::diag::{Diagnostics, Snapshot, NO_RULE};

#[test]
fn zero_offset_failure_is_distinct_from_no_failure() {
    let mut diag = Diagnostics::new();
    assert_eq!(diag.snapshot(), Snapshot::default());
    diag.fail_at(0, 7);
    assert_eq!(diag.farthest, 0);
    assert_eq!(diag.expected, [7]);
    assert_eq!(diag.deepest_rule, NO_RULE);
}

#[test]
fn farthest_replaces_ties_union_and_nearer_failures_are_ignored() {
    let mut diag = Diagnostics::new();
    diag.fail_at(4, 1);
    diag.fail_at(4, 2);
    diag.fail_at(4, 1);
    diag.fail_at(3, 9);
    assert_eq!(diag.expected, [1, 2]);
    diag.fail_at(5, 3);
    assert_eq!(diag.farthest, 5);
    assert_eq!(diag.expected, [3]);
}

#[test]
fn deepest_rule_is_selected_at_the_farthest_position() {
    let mut diag = Diagnostics::new();
    diag.enter_rule(10);
    diag.fail_at(4, 1);
    diag.enter_rule(20);
    diag.fail_at(4, 2);
    diag.leave_rule();
    diag.enter_rule(30);
    diag.fail_at(4, 3);
    assert_eq!(diag.deepest_rule, 20); // 同深度は初回を保持する。
    diag.leave_rule();
    diag.fail_at(5, 4);
    assert_eq!(diag.deepest_rule, 10); // より遠い位置なら浅い stack に置き換わる。
    diag.leave_rule();
    assert_eq!(diag.snapshot().rule_path, [10]);
    assert!(diag.leave_rule().is_none());
}

#[test]
fn successful_rule_failures_survive_leave() {
    let mut diag = Diagnostics::new();
    diag.enter_rule(1);
    diag.enter_rule(2);
    diag.fail_at(8, 9);
    let child = diag.leave_rule().unwrap();
    assert_eq!(child.rule_path, [2]);
    assert_eq!(diag.snapshot().rule_path, [1, 2]);
    diag.leave_rule();
    assert_eq!(diag.farthest, 8);
    assert_eq!(diag.expected, [9]);
}

#[test]
fn negative_lookahead_suppresses_nested_failures_and_memo_replay() {
    let mut saved = Diagnostics::new();
    saved.enter_rule(99);
    saved.fail_at(100, 99);
    let saved = saved.snapshot();

    let mut diag = Diagnostics::new();
    diag.enter_rule(1);
    diag.fail_at(2, 3);
    let before = diag.snapshot();
    diag.suppress_begin();
    diag.enter_rule(2);
    diag.fail_at(50, 4);
    diag.suppress_begin();
    diag.merge(&saved);
    diag.suppress_end();
    diag.fail_at(60, 5);
    assert!(diag.leave_rule().unwrap().expected.is_empty());
    assert_eq!(diag.snapshot(), before);
    diag.suppress_end();
    diag.fail_at(3, 6);
    assert_eq!(diag.expected, [6]);
    assert_eq!(diag.farthest, 3);
}

#[test]
fn local_snapshot_excludes_farther_sibling_and_caller_failures() {
    let mut diag = Diagnostics::new();
    diag.enter_rule(1);
    diag.fail_at(100, 1);
    diag.enter_rule(2);
    diag.fail_at(200, 2);
    diag.leave_rule();
    diag.enter_rule(3);
    assert!(diag.snapshot().expected.is_empty());
    diag.enter_rule(4);
    diag.fail_at(5, 7);
    diag.leave_rule();
    let local = diag.snapshot();
    assert_eq!(local.farthest, 5);
    assert_eq!(local.expected, [7]);
    assert_eq!(local.rule_path, [3, 4]);
    assert_eq!(diag.farthest, 200);
    assert_eq!(diag.deepest_rule, 2);
}

#[test]
fn memo_replay_rebases_stack_and_reaches_all_active_accumulators() {
    let mut first = Diagnostics::new();
    first.enter_rule(1);
    first.enter_rule(2);
    first.enter_rule(3);
    first.fail_at(8, 10);
    first.leave_rule();
    let memo = first.snapshot();

    let mut replay = Diagnostics::new();
    replay.enter_rule(40);
    replay.enter_rule(50);
    replay.merge(&memo);
    assert_eq!(replay.deepest_rule, 3);
    assert_eq!(replay.snapshot().rule_path, [50, 2, 3]);
    replay.leave_rule();
    assert_eq!(replay.snapshot().rule_path, [40, 50, 2, 3]);
    replay.leave_rule();

    let mut direct = Diagnostics::new();
    for rule in [40, 50, 2, 3] {
        direct.enter_rule(rule);
    }
    direct.fail_at(8, 10);
    for _ in 0..4 {
        direct.leave_rule();
    }
    assert_eq!(replay.snapshot(), direct.snapshot());
}

#[test]
fn nearer_memo_failure_still_reaches_local_frame() {
    let mut source = Diagnostics::new();
    source.enter_rule(7);
    source.fail_at(5, 8);
    let saved = source.snapshot();
    let mut target = Diagnostics::new();
    target.fail_at(100, 1);
    target.enter_rule(6);
    target.merge(&saved);
    assert_eq!(target.farthest, 100);
    assert_eq!(target.expected, [1]);
    assert_eq!(target.snapshot().farthest, 5);
    assert_eq!(target.snapshot().rule_path, [6, 7]);
}

#[test]
fn empty_and_repeated_merges_are_harmless() {
    let mut source = Diagnostics::new();
    source.fail_at(8, 4);
    source.fail_at(8, 5);
    let saved = source.snapshot();
    let mut target = Diagnostics::new();
    target.merge(&Snapshot::default());
    assert_eq!(target.snapshot(), Snapshot::default());
    target.fail_at(8, 3);
    target.merge(&saved);
    target.merge(&saved);
    assert_eq!(target.expected, [3, 4, 5]);
    target.merge(&Snapshot::default());
    assert_eq!(target.expected, [3, 4, 5]);
}

#[test]
fn snapshots_are_owned_and_independent_of_later_failures() {
    let mut diag = Diagnostics::new();
    diag.enter_rule(1);
    diag.fail_at(2, 3);
    let saved = diag.snapshot();
    diag.fail_at(9, 4);
    drop(diag);
    assert_eq!(saved.farthest, 2);
    assert_eq!(saved.expected, [3]);
    assert_eq!(saved.rule_path, [1]);
}
