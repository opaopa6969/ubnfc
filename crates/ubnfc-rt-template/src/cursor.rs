// ubnfc runtime template: cursor

/// byte 単位の二重カーソル。生成側は matched >= consumed を保つ。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    pub consumed: usize,
    pub matched: usize,
}

impl Cursor {
    /// 両位置に n を加算する。先読みによる差分も保持する。
    pub fn advance(&mut self, n: usize) {
        let consumed = self.consumed.checked_add(n).expect("consumed overflow");
        let matched = self.matched.checked_add(n).expect("matched overflow");
        *self = Self { consumed, matched };
    }

    pub fn advance_matched_only(&mut self, n: usize) {
        self.matched = self.matched.checked_add(n).expect("matched overflow");
    }

    pub fn reset_matched_to_consumed(&mut self) {
        self.matched = self.consumed;
    }

    pub fn checkpoint(&self) -> Self {
        *self
    }

    pub fn restore(&mut self, checkpoint: Self) {
        *self = checkpoint;
    }
}
