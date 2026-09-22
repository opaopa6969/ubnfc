// Vec open addressing。slot は payload index のみ。rehash で結果を複製しない。
use super::cursor::State;
use std::{
    collections::hash_map::RandomState,
    hash::BuildHasher,
    sync::atomic::{AtomicUsize, Ordering},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub expression: usize,
    pub state: State,
    pub matched_mode: bool,
    pub version: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PackedKey {
    expression: u32,
    consumed: u32,
    matched: u32,
    version: u32,
    flags: u32,
}
impl PackedKey {
    fn pack(key: Key) -> Option<Self> {
        Some(Self {
            expression: u32::try_from(key.expression).ok()?,
            consumed: u32::try_from(key.state.consumed).ok()?,
            matched: u32::try_from(key.state.matched).ok()?,
            version: key.version,
            flags: key.matched_mode as u32
                | ((key.state.invert as u32) << 1)
                | ((key.state.reset as u32) << 2),
        })
    }
}
pub struct Memo<T> {
    slots: Vec<u32>,
    entries: Vec<(PackedKey, T)>,
    hash: RandomState,
    max_probe: AtomicUsize,
}
impl<T: Copy> Default for Memo<T> {
    fn default() -> Self {
        Self {
            slots: vec![],
            entries: vec![],
            hash: RandomState::new(),
            max_probe: AtomicUsize::new(0),
        }
    }
}
impl<T: Copy> Memo<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.min(1_000_000);
        Self {
            slots: vec![0; ((capacity + 1) * 2).next_power_of_two().max(64)],
            entries: Vec::with_capacity(capacity),
            hash: RandomState::new(),
            max_probe: AtomicUsize::new(0),
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn max_probe_len(&self) -> usize {
        self.max_probe.load(Ordering::Relaxed)
    }
    pub fn get(&self, key: Key) -> Option<T> {
        if self.slots.is_empty() {
            return None;
        }
        let key = PackedKey::pack(key)?;
        let mut index = self.hash.hash_one(key) as usize & (self.slots.len() - 1);
        let mut probes = 0;
        loop {
            probes += 1;
            self.max_probe.fetch_max(probes, Ordering::Relaxed);
            let slot = self.slots[index];
            if slot == 0 {
                return None;
            }
            let (found, value) = self.entries[slot as usize - 1];
            if found == key {
                return Some(value);
            }
            index = (index + 1) & (self.slots.len() - 1);
        }
    }
    pub fn insert(&mut self, key: Key, value: T) {
        if self.entries.len() >= u32::MAX as usize - 1 {
            return;
        }
        if (self.entries.len() + 1) * 2 >= self.slots.len() {
            self.slots = vec![0; (self.slots.len() * 2).max(64)];
            for (i, (key, _)) in self.entries.iter().enumerate() {
                let mut index = self.hash.hash_one(key) as usize & (self.slots.len() - 1);
                while self.slots[index] != 0 {
                    index = (index + 1) & (self.slots.len() - 1);
                }
                self.slots[index] = (i + 1) as u32;
            }
        }
        let Some(key) = PackedKey::pack(key) else {
            return;
        };
        let mut index = self.hash.hash_one(key) as usize & (self.slots.len() - 1);
        let mut probes = 1;
        while self.slots[index] != 0 {
            probes += 1;
            self.max_probe.fetch_max(probes, Ordering::Relaxed);
            let entry = &mut self.entries[self.slots[index] as usize - 1];
            if entry.0 == key {
                entry.1 = value;
                return;
            }
            index = (index + 1) & (self.slots.len() - 1);
        }
        self.max_probe.fetch_max(probes, Ordering::Relaxed);
        self.entries.push((key, value));
        self.slots[index] = self.entries.len() as u32;
    }
}
