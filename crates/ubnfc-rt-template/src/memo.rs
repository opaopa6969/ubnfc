// ubnfc runtime template: memo

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// (expr_id, consumed byte offset, matched byte offset, mode, state_version)。
pub type MemoKey = (u32, u32, u32, u8, u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoEntry {
    pub ok: bool,
    pub consumed_end: u32,
    pub matched_end: u32,
    /// 呼出し側が所有する構築レシピ・局所診断の index。
    /// memo の生存中は参照先を rollback で再利用しない。
    pub payload: u32,
}

// 固定整数キー用の rotate/xor/multiply ハッシュ。暗号学的ハッシュではない。
#[derive(Default)]
struct MemoHasher(u64);
impl Hasher for MemoHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0.rotate_left(5) ^ u64::from(byte)).wrapping_mul(0x517c_c1b7_2722_0a95);
        }
    }
}

/// parse session ごとの表。scope 効果・診断の再生は呼出し側が行う。
#[derive(Default)]
pub struct MemoTable {
    entries: HashMap<MemoKey, MemoEntry, BuildHasherDefault<MemoHasher>>,
}

impl MemoTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &MemoKey) -> Option<&MemoEntry> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: MemoKey, entry: MemoEntry) -> Option<MemoEntry> {
        self.entries.insert(key, entry)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
