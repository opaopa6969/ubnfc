use ubnfc_rt_template::lexcache::{LexCache, SkipCache};

/// 統合 DFA の代わりに「その位置の byte の下位 6 bit を id とする」規則で埋める。
fn fill(bytes: &[u8], pos: usize, out: &mut [u64]) {
    if let Some(&byte) = bytes.get(pos) {
        let id = usize::from(byte) % 64;
        out[0] |= 1 << id;
    }
}

#[test]
fn a_position_is_filled_once_and_answers_every_id() {
    let text = b"abc";
    let mut cache = LexCache::default();
    cache.reset(1, text.len() + 1);
    for _ in 0..3 {
        for (pos, byte) in text.iter().enumerate() {
            if cache.need_fill(pos).unwrap() {
                fill(text, pos, cache.slot(pos));
            }
            assert!(cache.test(pos, u32::from(*byte) % 64));
            assert!(!cache.test(pos, (u32::from(*byte) + 1) % 64));
        }
    }
    // 位置ごとに 1 回だけ走る。
    assert_eq!(cache.runs, text.len() as u64);
    // probes は need_fill の回数（test は数えない）。
    assert_eq!(cache.probes, (text.len() * 3) as u64);
}

#[test]
fn eof_position_is_addressable_and_matches_nothing() {
    let text = b"a";
    let mut cache = LexCache::default();
    cache.reset(1, text.len() + 1);
    assert_eq!(cache.need_fill(text.len()), Some(true));
    fill(text, text.len(), cache.slot(text.len()));
    assert!(!cache.test(text.len(), 0));
    // 表の外は None（呼出し側は走査へ戻る）。
    assert_eq!(cache.need_fill(text.len() + 1), None);
}

#[test]
fn reset_invalidates_without_clearing_and_keeps_the_allocation() {
    let mut cache = LexCache::default();
    cache.reset(1, 8);
    assert!(cache.need_fill(0).unwrap());
    cache.slot(0)[0] = 0b101;
    assert!(cache.test(0, 2));
    let bytes = cache.capacity_bytes();
    cache.reset(1, 4);
    assert_eq!(cache.capacity_bytes(), bytes, "縮む入力では確保を保つ");
    assert!(cache.need_fill(0).unwrap(), "前 parse の entry は無効");
    // need_fill が true を返した位置は slot で 0 に戻してから埋める契約。
    assert_eq!(cache.slot(0)[0], 0);
    assert!(!cache.test(0, 2));
}

#[test]
fn zero_sized_cache_never_answers() {
    let mut cache = LexCache::default();
    cache.reset(0, 0);
    assert_eq!(cache.need_fill(0), None);
    assert_eq!(cache.capacity_bytes(), 0);
}

#[test]
fn skip_cache_separates_policy_mode_and_position() {
    let mut cache = SkipCache::default();
    cache.reset(2, 4);
    assert_eq!(cache.get(0, false, 1), None);
    cache.insert(0, false, 1, (3, 3));
    cache.insert(0, true, 1, (0, 5));
    cache.insert(1, false, 1, (u32::MAX, 0));
    assert_eq!(cache.get(0, false, 1), Some((3, 3)));
    assert_eq!(cache.get(0, true, 1), Some((0, 5)));
    assert_eq!(cache.get(1, false, 1), Some((u32::MAX, 0)));
    assert_eq!(cache.get(1, true, 1), None);
    assert_eq!(cache.get(0, false, 2), None);
    // 表の外は None。
    assert_eq!(cache.get(0, false, 4), None);
    assert_eq!(cache.hits.get(), 3);
}

#[test]
fn skip_cache_reset_invalidates_and_keeps_the_stride() {
    let mut cache = SkipCache::default();
    cache.reset(1, 16);
    cache.insert(0, false, 9, (1, 1));
    assert_eq!(cache.get(0, false, 9), Some((1, 1)));
    let bytes = cache.capacity_bytes();
    cache.reset(1, 4);
    assert_eq!(cache.capacity_bytes(), bytes);
    assert_eq!(cache.get(0, false, 9), None);
    assert_eq!(cache.hits.get(), 0, "reset で計数も戻る");
}
