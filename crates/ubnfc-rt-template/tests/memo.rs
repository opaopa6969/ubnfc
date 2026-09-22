use ubnfc_rt_template::{
    cursor::State,
    memo::{Key, Memo},
    scope::ScopeStore,
};
fn key(version: u32) -> Key {
    Key {
        expression: 1,
        state: State {
            consumed: 2,
            matched: 3,
            invert: false,
            reset: true,
        },
        matched_mode: false,
        version,
    }
}
#[test]
fn every_key_component_distinguishes_entries() {
    let mut keys = vec![key(4); 8];
    keys[1].expression = 0;
    keys[2].state.consumed = 0;
    keys[3].state.matched = 0;
    keys[4].state.invert = true;
    keys[5].state.reset = false;
    keys[6].matched_mode = true;
    keys[7].version = 0;
    let mut table = Memo::default();
    for (i, &key) in keys.iter().enumerate() {
        table.insert(key, i);
    }
    for (i, &key) in keys.iter().enumerate() {
        assert_eq!(table.get(key), Some(i));
    }
    assert_eq!(table.len(), 8);
}
#[test]
fn replacement_supports_success_and_failure_payloads() {
    let mut table = Memo::default();
    assert!(table.is_empty());
    table.insert(key(u32::MAX), (true, 8, 12, 0));
    table.insert(key(u32::MAX), (false, 2, 3, 1));
    assert_eq!(table.get(key(u32::MAX)), Some((false, 2, 3, 1)));
    assert_eq!(table.len(), 1);
    assert_eq!(table.get(key(0)), None);
    // memo の寿命は parse session。clear の代わりに空 table を作る。
    table = Memo::default();
    assert!(table.is_empty());
    assert_eq!(table.get(key(u32::MAX)), None);
}
#[test]
fn open_addressing_retains_entries_across_growth() {
    let mut table = Memo::with_capacity(0);
    for i in 0..20_000 {
        let mut k = key(i % 7);
        k.expression = i as usize;
        table.insert(k, i);
    }
    for i in 0..20_000 {
        let mut k = key(i % 7);
        k.expression = i as usize;
        assert_eq!(table.get(k), Some(i));
    }
    assert_eq!(table.len(), 20_000);
}
#[test]
fn rollback_restores_memo_visibility_without_reusing_branch_versions() {
    let mut table = Memo::default();
    let mut scopes = ScopeStore::new();
    let baseline = scopes.checkpoint();
    table.insert(key(scopes.state_version()), 0);
    scopes.declare("a", 0);
    let branch = scopes.state_version();
    table.insert(key(branch), 1);
    scopes.restore(baseline);
    assert_eq!(table.get(key(scopes.state_version())), Some(0));
    scopes.declare("b", 0);
    assert!(scopes.state_version() > branch);
    assert_eq!(table.get(key(scopes.state_version())), None);
}
