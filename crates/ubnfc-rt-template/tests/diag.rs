use ubnfc_rt_template::diag::{Diag, Diagnostics};
#[test]
fn zero_offset_failure_is_distinct_from_no_failure() {
    let mut d = Diagnostics::default();
    assert_eq!(d.summary(Diag(0)), None);
    let leaf = d.fail(0, "number");
    assert_eq!(d.summary(leaf), Some((0, vec!["number"], vec![])));
}
#[test]
fn farthest_replaces_ties_union_and_nearer_failures_are_ignored() {
    let mut d = Diagnostics::new();
    let a = d.fail(4, "a");
    let b = d.fail(4, "b");
    let c = d.fail(3, "c");
    let joined = d.join(a, b);
    let joined = d.join(joined, c);
    let joined = d.join(joined, a);
    assert_eq!(d.summary(joined), Some((4, vec!["a", "b"], vec![])));
    let far = d.fail(5, "far");
    let joined = d.join(joined, far);
    assert_eq!(d.summary(joined), Some((5, vec!["far"], vec![])));
}
#[test]
fn deepest_rule_ties_preserve_first_path() {
    let mut d = Diagnostics::new();
    let leaf = d.fail(4, "x");
    let a = d.rule(20, leaf);
    let b = d.rule(30, leaf);
    let both = d.join(a, b);
    let root = d.rule(10, both);
    assert_eq!(d.summary(root).unwrap().2, vec![10, 20]);
    let far = d.fail(5, "far");
    let far = d.rule(10, far);
    let root = d.join(root, far);
    assert_eq!(d.summary(root).unwrap().2, vec![10]);
}
#[test]
fn successful_rule_failures_remain_in_parent_dag() {
    let mut d = Diagnostics::new();
    let leaf = d.fail(8, "x");
    let child = d.rule(2, leaf);
    let parent = d.rule(1, child);
    assert_eq!(d.summary(child).unwrap().2, vec![2]);
    assert_eq!(d.summary(parent), Some((8, vec!["x"], vec![1, 2])));
}
#[test]
fn negative_lookahead_suppresses_by_not_joining_child_diagnostics() {
    let mut d = Diagnostics::new();
    let kept = d.fail(2, "kept");
    let ignored = d.fail(100, "ignored");
    let ignored = d.rule(99, ignored);
    assert_eq!(d.summary(ignored).unwrap().0, 100);
    let parent = d.rule(1, kept);
    assert_eq!(d.summary(parent), Some((2, vec!["kept"], vec![1])));
}
#[test]
fn local_dag_excludes_farther_sibling_and_caller_failures() {
    let mut d = Diagnostics::new();
    let far = d.fail(200, "far");
    let far = d.rule(2, far);
    let local = d.fail(5, "local");
    let local = d.rule(4, local);
    let local = d.rule(3, local);
    let root = d.join(far, local);
    assert_eq!(d.summary(root).unwrap().0, 200);
    assert_eq!(d.summary(local), Some((5, vec!["local"], vec![3, 4])));
}
#[test]
fn memo_replay_rebases_without_copying_local_dag() {
    let mut d = Diagnostics::new();
    let leaf = d.fail(8, "x");
    let leaf = d.rule(3, leaf);
    let memo = d.rule(2, leaf);
    let first = d.rule(1, memo);
    let replay = d.rule(50, memo);
    let replay = d.rule(40, replay);
    assert_eq!(d.summary(first).unwrap().2, vec![1, 2, 3]);
    assert_eq!(d.summary(replay), Some((8, vec!["x"], vec![40, 50, 2, 3])));
}
#[test]
fn nearer_memo_failure_remains_available_to_local_frame() {
    let mut d = Diagnostics::new();
    let local = d.fail(5, "local");
    let local = d.rule(7, local);
    let local = d.rule(6, local);
    let far = d.fail(100, "far");
    let root = d.join(far, local);
    assert_eq!(d.summary(root).unwrap().0, 100);
    assert_eq!(d.summary(local), Some((5, vec!["local"], vec![6, 7])));
}
#[test]
fn empty_and_repeated_joins_are_harmless() {
    let mut d = Diagnostics::new();
    let a = d.fail(8, "a");
    let b = d.fail(8, "b");
    assert_eq!(d.join(Diag(0), a), a);
    assert_eq!(d.join(a, Diag(0)), a);
    assert_eq!(d.join(a, a), a);
    let both = d.join(a, b);
    let repeated = d.join(both, a);
    assert_eq!(d.summary(repeated).unwrap().1, vec!["a", "b"]);
}
#[test]
fn saved_handles_are_immutable_and_summaries_are_owned() {
    let mut d = Diagnostics::new();
    let saved = d.fail(2, "saved");
    let saved = d.rule(1, saved);
    let later = d.fail(9, "later");
    let _ = d.join(saved, later);
    let summary = d.summary(saved);
    drop(d);
    assert_eq!(summary, Some((2, vec!["saved"], vec![1])));
}
