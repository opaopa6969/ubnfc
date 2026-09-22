// transactional-scopes.md / generated-scope-effects.md の store 層の契約。
// capture 選択・trim・注釈 lowering は生成側の責務で、末尾のケースはその呼出し列を再現する。
use ubnfc_rt_template::cursor::State;
use ubnfc_rt_template::diag::Diagnostics;
use ubnfc_rt_template::scope::{Decl, ScopeStore, Severity};

fn decl(name: &str, offset_cp: usize) -> Decl {
    Decl {
        name: name.into(),
        offset_cp,
    }
}

fn assert_same_state(actual: &ScopeStore, expected: &ScopeStore) {
    assert_eq!(actual.depth(), expected.depth());
    assert_eq!(actual.state_version(), expected.state_version());
    assert_eq!(actual.all_declarations(), expected.all_declarations());
    assert_eq!(
        actual.declared_in_current_scope(),
        expected.declared_in_current_scope()
    );
    assert_eq!(actual.references(), expected.references());
    assert_eq!(actual.diagnostics(), expected.diagnostics());
    for name in actual
        .all_declarations()
        .iter()
        .chain(expected.all_declarations())
        .map(|d| d.name.as_str())
        .chain(["absent"])
    {
        assert_eq!(actual.resolve(name), expected.resolve(name));
    }
}

#[test]
fn same_scope_redeclaration_is_last_wins_but_history_keeps_both() {
    for nested in [false, true] {
        let mut store = ScopeStore::new();
        if nested {
            store.enter();
        }
        store.declare("z", 1);
        store.declare("a", 2);
        store.declare("z", 3);
        assert_eq!(store.resolve("z"), Some(&decl("z", 3)));
        assert_eq!(
            store.declared_in_current_scope(),
            [&decl("a", 2), &decl("z", 3)]
        );
        assert_eq!(
            store.all_declarations(),
            [decl("z", 1), decl("a", 2), decl("z", 3)]
        );
    }
}

#[test]
fn resolution_searches_inner_outer_then_global() {
    let mut store = ScopeStore::new();
    store.declare("x", 0);
    store.declare("global", 1);
    store.enter();
    store.declare("x", 2);
    store.declare("outer", 3);
    store.enter();
    store.declare("x", 4);
    assert_eq!(store.resolve("x"), Some(&decl("x", 4)));
    assert!(store.is_declared("outer"));
    assert!(store.is_declared("global"));
    store.leave();
    assert_eq!(store.resolve("x"), Some(&decl("x", 2)));
    store.leave();
    assert_eq!(store.resolve("x"), Some(&decl("x", 0)));
    assert!(!store.is_declared("outer"));
}

#[test]
fn leave_hides_names_but_keeps_successful_event_logs() {
    let mut store = ScopeStore::new();
    store.enter();
    store.declare("local", 2);
    store.add_reference("local", 8, 5);
    store.add_diagnostic("note", 8, 5, Severity::Info);
    store.leave();
    assert_eq!(store.depth(), 0);
    assert!(!store.is_declared("local"));
    assert!(store.declared_in_current_scope().is_empty());
    assert_eq!(store.all_declarations(), [decl("local", 2)]);
    assert_eq!(store.references()[0].name, "local");
    assert_eq!(store.diagnostics()[0].msg, "note");
}

#[test]
fn depth_zero_leave_is_a_complete_noop() {
    let mut store = ScopeStore::new();
    store.declare("global", 1);
    let before = store.clone();
    store.leave();
    store.leave();
    assert_same_state(&store, &before);
}

#[test]
fn empty_names_are_noops_and_store_does_not_trim_supplied_names() {
    let mut store = ScopeStore::new();
    let before = store.clone();
    store.declare("", 100);
    store.add_reference("", 100, 20);
    assert_same_state(&store, &before);
    assert!(!store.is_declared(""));
    store.declare(" ", 3);
    assert_eq!(store.resolve(" "), Some(&decl(" ", 3)));
}

#[test]
fn reference_logging_is_separate_from_resolution_and_warning() {
    let mut store = ScopeStore::new();
    store.add_reference("unknown", 4, 7);
    assert!(!store.is_declared("unknown"));
    assert!(store.diagnostics().is_empty());
    assert_eq!(store.references()[0].offset, 4);
    assert_eq!(store.references()[0].len, 7);
}

#[test]
fn failed_transaction_restores_popped_scopes_overwrites_global_and_all_logs() {
    let mut store = ScopeStore::new();
    store.declare("global", 1);
    store.enter();
    store.declare("outer", 2);
    store.enter();
    store.declare("inner", 3);
    store.add_reference("inner", 5, 5);
    store.add_diagnostic("baseline", 5, 5, Severity::Hint);
    let before = store.clone();
    let checkpoint = store.checkpoint();

    store.declare("inner", 10);
    store.leave();
    store.declare("outer", 20);
    store.leave();
    store.declare("global", 30);
    store.declare("discarded", 31);
    store.clear_diagnostics();
    store.add_reference("discarded", 40, 9);
    store.add_diagnostic("discarded", 40, 9, Severity::Error);
    store.enter();
    store.declare("other_branch", 50);
    store.restore(checkpoint);
    assert_same_state(&store, &before);
    assert!(!store.is_declared("discarded"));
    assert!(!store.is_declared("other_branch"));
    // 復元した scope の内部 lookup も、順に leave して確認する。
    store.leave();
    assert_eq!(store.resolve("outer"), Some(&decl("outer", 2)));
    store.leave();
    assert_eq!(store.resolve("global"), Some(&decl("global", 1)));
}

#[test]
fn successful_child_is_undone_when_parent_later_fails() {
    let mut store = ScopeStore::new();
    let parent = store.checkpoint();
    store.enter();
    let _child = store.checkpoint();
    store.enter();
    store.declare("child", 4);
    store.leave(); // 子成功。ログは親に残す。
    store.declare("parent", 1);
    store.restore(parent);
    assert_same_state(&store, &ScopeStore::new());
}

#[test]
fn nested_restore_keeps_parent_changes_until_parent_restore() {
    let mut store = ScopeStore::new();
    let parent = store.checkpoint();
    store.declare("parent", 0);
    let parent_state = store.clone();
    let child = store.checkpoint();
    store.declare("parent", 1);
    store.enter();
    store.declare("child", 2);
    store.restore(child);
    assert_same_state(&store, &parent_state);
    store.restore(parent);
    assert_same_state(&store, &ScopeStore::new());
}

#[test]
fn repeated_clear_and_append_restore_exact_diagnostic_order() {
    let mut store = ScopeStore::new();
    for (i, severity) in [
        Severity::Error,
        Severity::Warning,
        Severity::Info,
        Severity::Hint,
    ]
    .into_iter()
    .enumerate()
    {
        store.add_diagnostic(&format!("before {i}"), i, 1, severity);
    }
    let before = store.clone();
    let checkpoint = store.checkpoint();
    store.add_diagnostic("extra", 9, 1, Severity::Info);
    store.clear_diagnostics();
    store.add_diagnostic("middle", 10, 1, Severity::Error);
    store.clear_diagnostics();
    store.clear_diagnostics();
    store.add_diagnostic("after", 11, 1, Severity::Warning);
    store.restore(checkpoint);
    assert_same_state(&store, &before);
}

#[test]
fn first_mutation_inside_transaction_can_restore_empty_baseline() {
    let mut store = ScopeStore::new();
    let checkpoint = store.checkpoint();
    store.add_diagnostic("first", 0, 1, Severity::Info);
    store.add_reference("first", 0, 5);
    store.declare("first", 0);
    store.enter();
    store.restore(checkpoint);
    assert_same_state(&store, &ScopeStore::new());
}

#[test]
fn lookahead_restores_semantics_and_keeps_independent_matched_cursor() {
    let mut store = ScopeStore::new();
    let checkpoint = store.checkpoint();
    let mut cursor = State::default();
    store.enter();
    store.declare("temporary", 0);
    assert!(store.is_declared("temporary")); // 先読み内部の後続の子には見える。
    cursor.advance::<true>(4);
    store.restore(checkpoint);
    assert!(!store.is_declared("temporary"));
    assert_eq!(
        cursor,
        State {
            consumed: 0,
            matched: 4,
            ..State::default()
        }
    );
}

#[test]
fn syntax_failure_survives_semantic_rollback() {
    let mut store = ScopeStore::new();
    let mut syntax = Diagnostics::new();
    let checkpoint = store.checkpoint();
    store.add_diagnostic("speculative", 3, 1, Severity::Warning);
    let failure = syntax.fail(3, "number");
    store.restore(checkpoint);
    assert!(store.diagnostics().is_empty());
    assert_eq!(syntax.summary(failure).unwrap().1, ["number"]);
}

#[test]
fn cloned_store_is_an_owned_snapshot() {
    let mut store = ScopeStore::new();
    store.declare("owned", 1);
    store.add_reference("owned", 5, 5);
    let snapshot = store.clone();
    store.declare("owned", 20);
    store.enter();
    drop(store);
    assert_eq!(snapshot.resolve("owned"), Some(&decl("owned", 1)));
    assert_eq!(snapshot.references().len(), 1);
    assert_eq!(snapshot.depth(), 0);
}

#[test]
fn enter_leave_declare_and_clear_allocate_versions() {
    let mut store = ScopeStore::new();
    store.enter();
    assert_eq!(store.state_version(), 1);
    store.declare("x", 0);
    assert_eq!(store.state_version(), 2);
    store.declare("x", 0); // 同じ値でも新しい宣言履歴になる。
    assert_eq!(store.state_version(), 3);
    store.leave();
    assert_eq!(store.state_version(), 4);
    store.clear_diagnostics(); // 空でも明示 clear は version を進める。
    assert_eq!(store.state_version(), 5);
    store.add_diagnostic("warning", 0, 1, Severity::Warning);
    store.clear_diagnostics();
    assert_eq!(store.state_version(), 6);
}

#[test]
fn references_and_diagnostics_do_not_allocate_versions() {
    let mut store = ScopeStore::new();
    store.declare("x", 0);
    let version = store.state_version();
    store.add_reference("x", 2, 1);
    store.add_reference("unknown", 3, 7);
    store.add_diagnostic("unknown", 3, 7, Severity::Warning);
    assert_eq!(store.state_version(), version);
}

#[test]
fn rollback_restores_version_but_never_rewinds_epoch_allocator() {
    let mut store = ScopeStore::new();
    let root = store.checkpoint();
    let mut high_water = 0;
    for branch in 0..32 {
        store.enter();
        assert!(store.state_version() > high_water);
        store.declare("x", branch);
        high_water = store.state_version();
        store.restore(root);
        assert_eq!(store.state_version(), 0);
        assert!(store.all_declarations().is_empty());
    }
}

#[test]
fn scoped_declaration_is_registered_in_parent_after_leave() {
    let mut store = ScopeStore::new();
    for in_parent in [false, true] {
        if in_parent {
            store.enter();
        }
        let parent_depth = store.depth();
        store.enter();
        store.declare("local", 2);
        store.leave();
        store.declare("rule", 1);
        assert_eq!(store.depth(), parent_depth);
        assert!(!store.is_declared("local"));
        assert_eq!(store.declared_in_current_scope(), [&decl("rule", 1)]);
    }
}

// 生成側が行う capture 正規化とイベント順序の例。store 自体の処理とは区別する。
fn capture_effects(
    store: &mut ScopeStore,
    capture: Option<(&str, usize)>,
    declares: bool,
    references: bool,
) {
    let Some((text, offset)) = capture else {
        return;
    };
    let name = text.trim_matches(|ch| ch <= '\u{20}');
    if name.is_empty() {
        return;
    }
    let leading_cp = text.chars().take_while(|&ch| ch <= '\u{20}').count();
    let offset = offset + leading_cp;
    let len = name.chars().count();
    if declares {
        store.declare(name, offset);
    }
    if references {
        store.add_reference(name, offset, len);
        if !store.is_declared(name) {
            store.add_diagnostic(
                &format!("未定義のシンボル: '{name}'"),
                offset,
                len,
                Severity::Warning,
            );
        }
    }
}

#[test]
fn declarations_precede_references_and_undefined_use_is_only_a_warning() {
    let mut store = ScopeStore::new();
    capture_effects(&mut store, Some(("known", 0)), true, true);
    assert!(store.diagnostics().is_empty());
    capture_effects(&mut store, Some(("unknown", 6)), false, true);
    assert_eq!(store.references().len(), 2);
    assert_eq!(store.diagnostics()[0].severity, Severity::Warning);
    assert_eq!(store.diagnostics()[0].msg, "未定義のシンボル: 'unknown'");
}

#[test]
fn optional_absence_and_trimmed_empty_capture_emit_no_events() {
    let mut store = ScopeStore::new();
    capture_effects(&mut store, None, true, true);
    capture_effects(&mut store, Some(("\0\t \r\n", 8)), true, true);
    assert_same_state(&store, &ScopeStore::new());
}

#[test]
fn caller_trims_java_control_range_and_preserves_quotes_escapes_and_cp_spans() {
    let mut store = ScopeStore::new();
    capture_effects(&mut store, Some(("\0 \t\"😀\\n\"\r\n", 7)), true, true);
    assert_eq!(store.all_declarations(), [decl("\"😀\\n\"", 10)]);
    assert_eq!(store.references()[0].offset, 10);
    assert_eq!(store.references()[0].len, 5);
    capture_effects(&mut store, Some(("\u{a0}name\u{a0}", 20)), true, false);
    assert_eq!(store.all_declarations()[1], decl("\u{a0}name\u{a0}", 20));
}

#[test]
fn repeated_and_nested_capture_events_keep_completion_order() {
    let mut store = ScopeStore::new();
    // 内側→外側→次の反復。呼出し元は自分の capture だけを選んでこの列を渡す。
    for capture in [("inner", 3), ("outer", 1), ("next", 10)] {
        capture_effects(&mut store, Some(capture), true, true);
    }
    assert_eq!(
        store.all_declarations(),
        [decl("inner", 3), decl("outer", 1), decl("next", 10)]
    );
    assert_eq!(
        store
            .references()
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>(),
        ["inner", "outer", "next"]
    );
}
