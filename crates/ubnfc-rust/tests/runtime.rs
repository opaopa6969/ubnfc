#[path = "../templates/cursor.rs"]
mod cursor;
#[path = "../templates/diag.rs"]
mod diag;
#[path = "../templates/memo.rs"]
mod memo;
use cursor::State;

#[test]
fn cursor_transition_table() {
    let mut s = State {
        consumed: 2,
        matched: 5,
        invert: false,
        reset: true,
    };
    assert_eq!(s.begin().matched, 2);
    s.reset = false;
    assert_eq!(s.begin().matched, 5);
    s.advance::<true>(3);
    assert_eq!((s.consumed, s.matched), (2, 8));
    s.advance::<false>(0);
    assert_eq!((s.consumed, s.matched), (2, 2));
    s.advance::<false>(2);
    assert_eq!((s.consumed, s.matched), (4, 4));
    let child = State {
        consumed: 6,
        matched: 8,
        reset: true,
        ..s
    };
    let committed = s.commit(child);
    assert!(!committed.reset);
    assert_eq!(committed.position::<true>(), 8);
    assert_eq!(committed.position::<false>(), 6);
}
#[test]
fn memo_growth_collisions_and_every_key_component() {
    let mut table = memo::Memo::with_capacity(0);
    assert!(table.is_empty());
    let mut keys = Vec::new();
    for n in 0..20_000usize {
        let key = memo::Key {
            expression: n % 91,
            state: State {
                consumed: n / 91,
                matched: n / 91 + n % 3,
                invert: n & 1 != 0,
                reset: n & 2 != 0,
            },
            matched_mode: n & 4 != 0,
            version: (n % 13) as u32,
        };
        table.insert(key, n);
        keys.push(key);
    }
    assert_eq!(table.len(), keys.len());
    for (n, &key) in keys.iter().enumerate() {
        assert_eq!(table.get(key), Some(n));
    }
    let key = keys[0];
    table.insert(key, 77);
    assert_eq!(table.len(), keys.len());
    assert_eq!(table.get(key), Some(77));
    for bit in 0..3 {
        let mut variant = key;
        match bit {
            0 => variant.state.invert = !variant.state.invert,
            1 => variant.state.reset = !variant.state.reset,
            _ => variant.matched_mode = !variant.matched_mode,
        }
        assert_eq!(table.get(variant), None);
    }
    let mut variant = key;
    variant.version = u32::MAX;
    assert_eq!(table.get(variant), None);
}
#[test]
fn diagnostic_replay_rebases_without_copying_caller_stacks() {
    let mut d = diag::Diagnostics::new();
    let leaf = d.fail(3, "number");
    let local = d.rule(2, leaf);
    let first = d.rule(1, local);
    let second = d.rule(0, first);
    let joined = d.join(first, second);
    assert_eq!(d.summary(joined), Some((3, vec!["number"], vec![0, 1, 2])));
    let other = d.fail(3, "identifier");
    let joined = d.join(joined, other);
    assert_eq!(d.summary(joined).unwrap().1, vec!["number", "identifier"]);
    let farther = d.fail(4, "end");
    let joined = d.join(joined, farther);
    assert_eq!(d.summary(joined), Some((4, vec!["end"], vec![])));
}
