use crate::ir::CharRange;

/// ASCII は 128 bit、残りは正規化済み閉区間。IR では両者を ranges に統合する。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CharSet {
    pub ascii: u128,
    pub unicode: Vec<(u64, u64)>,
}

impl CharSet {
    pub fn all() -> Self {
        Self {
            ascii: u128::MAX,
            unicode: vec![(128, 0x10ffff)],
        }
    }

    pub fn range(start: u64, end: u64) -> Self {
        let mut set = Self::default();
        if start > end {
            return set;
        }
        for cp in start..=end.min(127) {
            set.ascii |= 1 << cp;
        }
        if end >= 128 {
            set.unicode.push((start.max(128), end));
        }
        set
    }

    pub fn chars(chars: &str) -> Self {
        let mut result = Self::default();
        for c in chars.chars() {
            result.union(&Self::range(c as u64, c as u64));
        }
        result
    }

    pub fn union(&mut self, other: &Self) {
        self.ascii |= other.ascii;
        self.unicode.extend_from_slice(&other.unicode);
        self.unicode.sort_unstable();
        let mut ranges: Vec<(u64, u64)> = Vec::new();
        for &(lo, hi) in &self.unicode {
            if let Some(last) = ranges.last_mut() {
                if lo <= last.1.saturating_add(1) {
                    last.1 = last.1.max(hi);
                    continue;
                }
            }
            ranges.push((lo, hi));
        }
        self.unicode = ranges;
    }

    pub fn complement(&self) -> Self {
        let mut result = Self {
            ascii: !self.ascii,
            unicode: Vec::new(),
        };
        let mut next = 128;
        for &(lo, hi) in &self.unicode {
            if next < lo {
                result.unicode.push((next, lo - 1));
            }
            next = hi + 1;
        }
        if next <= 0x10ffff {
            result.unicode.push((next, 0x10ffff));
        }
        result
    }

    pub fn disjoint(&self, other: &Self) -> bool {
        self.ascii & other.ascii == 0
            && !self
                .unicode
                .iter()
                .any(|a| other.unicode.iter().any(|b| a.0 <= b.1 && b.0 <= a.1))
    }

    pub fn ranges(&self) -> Vec<CharRange> {
        let mut ranges: Vec<CharRange> = Vec::new();
        for cp in 0..128 {
            if self.ascii & (1 << cp) != 0 {
                if let Some(last) = ranges.last_mut() {
                    if last.end + 1 == cp {
                        last.end = cp;
                        continue;
                    }
                }
                ranges.push(CharRange { start: cp, end: cp });
            }
        }
        for &(start, end) in &self.unicode {
            if let Some(last) = ranges.last_mut() {
                if last.end + 1 == start {
                    last.end = end;
                    continue;
                }
            }
            ranges.push(CharRange { start, end });
        }
        ranges
    }
}
