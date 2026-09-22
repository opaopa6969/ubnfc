// Vec open addressing。slot は payload index のみ。rehash で結果を複製しない。
use super::cursor::State;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub expression: usize,
    pub state: State,
    pub matched_mode: bool,
    pub version: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackedKey {
    expression: u32,
    consumed: u32,
    matched: u32,
    version: u32,
    flags: u32,
}
impl From<Key> for PackedKey {
    fn from(key: Key) -> Self {
        Self {
            expression: u32::try_from(key.expression).expect("expression index"),
            consumed: u32::try_from(key.state.consumed).expect("input exceeds u32 bytes"),
            matched: u32::try_from(key.state.matched).expect("input exceeds u32 bytes"),
            version: key.version,
            flags: key.matched_mode as u32
                | ((key.state.invert as u32) << 1)
                | ((key.state.reset as u32) << 2),
        }
    }
}
impl PackedKey {
    fn hash(self) -> usize {
        let mut h = (self.consumed as usize).wrapping_mul(0x9e3779b1);
        for x in [self.matched, self.expression, self.version, self.flags] {
            h = (h ^ x as usize).wrapping_mul(0x85ebca6b);
            h ^= h >> 16;
        }
        h
    }
}
pub struct Memo<T> {
    slots: Vec<u32>,
    entries: Vec<(PackedKey, T)>,
}
impl<T: Copy> Default for Memo<T> {
    fn default() -> Self {
        Self {
            slots: vec![],
            entries: vec![],
        }
    }
}
impl<T: Copy> Memo<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: vec![0; ((capacity + 1) * 4 / 3).next_power_of_two().max(64)],
            entries: Vec::with_capacity(capacity),
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn get(&self, key: Key) -> Option<T> {
        if self.slots.is_empty() {
            return None;
        }
        let key = PackedKey::from(key);
        let mut index = key.hash() & (self.slots.len() - 1);
        loop {
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
        if (self.entries.len() + 1) * 4 >= self.slots.len() * 3 {
            self.slots = vec![0; (self.slots.len() * 2).max(64)];
            for (i, (key, _)) in self.entries.iter().enumerate() {
                let mut index = key.hash() & (self.slots.len() - 1);
                while self.slots[index] != 0 {
                    index = (index + 1) & (self.slots.len() - 1);
                }
                self.slots[index] = (i + 1) as u32;
            }
        }
        let key = PackedKey::from(key);
        let mut index = key.hash() & (self.slots.len() - 1);
        while self.slots[index] != 0 {
            let entry = &mut self.entries[self.slots[index] as usize - 1];
            if entry.0 == key {
                entry.1 = value;
                return;
            }
            index = (index + 1) & (self.slots.len() - 1);
        }
        self.entries.push((key, value));
        self.slots[index] = u32::try_from(self.entries.len()).expect("memo entries exceed u32");
    }
}
