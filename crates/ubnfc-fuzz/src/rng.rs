//! 決定的 PRNG。campaign は seed だけで再現できる（外部乱数・時刻を読まない）。

/// SplitMix64。同じ seed からは同じ列、seed が違えば独立な列になる。
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x2545_F491_4F6C_DD1D)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// `0..n`。n=0 は 0 を返す（呼出側の空集合判定を省かない）。
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }

    /// 閉区間 `[lo, hi]`。lo > hi なら lo。
    pub fn between(&mut self, lo: usize, hi: usize) -> usize {
        lo + self.below(hi.saturating_sub(lo) + 1)
    }

    pub fn chance(&mut self, percent: u32) -> bool {
        self.below(100) < percent as usize
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }

    /// 重み付き選択。重みの総和が 0 なら最後の要素。
    pub fn weighted(&mut self, weights: &[u32]) -> usize {
        let total: u32 = weights.iter().sum();
        if total == 0 {
            return weights.len().saturating_sub(1);
        }
        let mut point = self.below(total as usize) as u32;
        for (index, weight) in weights.iter().enumerate() {
            if point < *weight {
                return index;
            }
            point -= weight;
        }
        weights.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn the_same_seed_replays_the_same_stream_and_different_seeds_diverge() {
        let take = |seed| {
            let mut rng = Rng::new(seed);
            (0..16).map(|_| rng.next_u64()).collect::<Vec<_>>()
        };
        assert_eq!(take(7), take(7));
        assert_ne!(take(7), take(8));
        let mut rng = Rng::new(1);
        for _ in 0..1000 {
            assert!(rng.between(3, 5) >= 3 && rng.between(3, 5) <= 5);
            assert_eq!(rng.below(1), 0);
            assert!(rng.weighted(&[0, 0, 1]) == 2);
        }
    }
}
