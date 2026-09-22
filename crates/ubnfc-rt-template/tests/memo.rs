use ubnfc_rt_template::memo::{MemoEntry, MemoTable};
use ubnfc_rt_template::scope::ScopeStore;

fn entry(payload: u32) -> MemoEntry {
    MemoEntry {
        ok: payload % 2 == 0,
        consumed_end: 8,
        matched_end: 12,
        payload,
    }
}

#[test]
fn all_five_key_components_distinguish_entries() {
    let keys = [
        (1, 2, 3, 4, 5),
        (0, 2, 3, 4, 5),
        (1, 0, 3, 4, 5),
        (1, 2, 0, 4, 5),
        (1, 2, 3, 0, 5),
        (1, 2, 3, 4, 0),
    ];
    let mut table = MemoTable::new();
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(table.insert(*key, entry(i as u32)), None);
    }
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(table.get(key), Some(&entry(i as u32)));
    }
    assert_eq!(table.len(), 6);
    assert_eq!(table.get(&(9, 2, 3, 4, 5)), None);
}

#[test]
fn replacement_and_clear_include_failure_entries() {
    let mut table = MemoTable::default();
    let key = (u32::MAX, u32::MAX, u32::MAX, u8::MAX, u32::MAX);
    assert!(table.is_empty());
    table.insert(key, entry(0));
    assert_eq!(table.insert(key, entry(1)), Some(entry(0)));
    assert_eq!(table.get(&key), Some(&entry(1)));
    assert_eq!(table.len(), 1);
    table.clear();
    assert!(table.is_empty());
    assert_eq!(table.get(&key), None);
    table.insert(key, entry(2));
    assert_eq!(table.get(&key), Some(&entry(2)));
}

#[test]
fn table_retains_entries_across_growth() {
    let mut table = MemoTable::new();
    for i in 0..4096 {
        table.insert((i, i * 2, i * 3, (i % 4) as u8, i % 7), entry(i));
    }
    for i in 0..4096 {
        assert_eq!(
            table.get(&(i, i * 2, i * 3, (i % 4) as u8, i % 7)),
            Some(&entry(i))
        );
    }
}

#[test]
fn rollback_restores_memo_visibility_without_reusing_branch_versions() {
    let mut table = MemoTable::new();
    let mut scopes = ScopeStore::new();
    let baseline = scopes.checkpoint();
    let key = (1, 0, 0, 0, scopes.state_version());
    table.insert(key, entry(0));
    scopes.declare("a", 0);
    let first_branch = scopes.state_version();
    table.insert((1, 0, 0, 0, first_branch), entry(1));
    scopes.restore(baseline);
    assert_eq!(
        table.get(&(1, 0, 0, 0, scopes.state_version())),
        Some(&entry(0))
    );
    scopes.declare("b", 0);
    assert!(scopes.state_version() > first_branch);
    assert_eq!(table.get(&(1, 0, 0, 0, scopes.state_version())), None);
}
