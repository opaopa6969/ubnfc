// 局所診断を共有する arena。memo replay の rule path は親に接続して復元する。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Diag(pub usize);
#[derive(Clone, Copy)]
enum Node {
    Empty,
    Failure(usize, &'static str),
    Join(Diag, Diag),
    Rule(usize, Diag),
}
pub struct Diagnostics {
    nodes: Vec<(usize, Node)>,
    limit: usize,
    pub exhausted: bool,
}
impl Diagnostics {
    pub fn new() -> Self {
        Self::with_limit(2_000_000)
    }
    pub fn with_limit(limit: usize) -> Self {
        Self {
            nodes: vec![(0, Node::Empty)],
            limit: limit.min(u32::MAX as usize),
            exhausted: false,
        }
    }
    fn push(&mut self, pos: usize, node: Node) -> Diag {
        if self.nodes.len() >= self.limit {
            self.exhausted = true;
            return Diag(0);
        }
        let id = Diag(self.nodes.len());
        self.nodes.push((pos, node));
        id
    }
    pub fn fail(&mut self, pos: usize, label: &'static str) -> Diag {
        self.push(pos, Node::Failure(pos, label))
    }
    pub fn join(&mut self, a: Diag, b: Diag) -> Diag {
        if a.0 == 0 {
            return b;
        }
        if b.0 == 0 || a == b {
            return a;
        }
        let pa = self.nodes[a.0].0;
        let pb = self.nodes[b.0].0;
        if pa > pb {
            a
        } else if pb > pa {
            b
        } else {
            self.push(pa, Node::Join(a, b))
        }
    }
    pub fn rule(&mut self, rule: usize, child: Diag) -> Diag {
        if child.0 == 0 {
            child
        } else {
            self.push(self.nodes[child.0].0, Node::Rule(rule, child))
        }
    }
    pub fn summary(&self, root: Diag) -> Option<(usize, Vec<&'static str>, Vec<usize>)> {
        if root.0 == 0 {
            return None;
        }
        let mut expected = Vec::new();
        let mut labels = std::collections::HashSet::new();
        // Local suffix depth is independent of the caller. Reconnect the chosen
        // suffix from the root, so a shared node reached by a longer path wins.
        let mut depths = std::collections::HashMap::<usize, usize>::new();
        let mut stack = vec![(root, false)];
        while let Some((id, leave)) = stack.pop() {
            if leave {
                depths.insert(
                    id.0,
                    match self.nodes[id.0].1 {
                        Node::Join(a, b) => depths[&a.0].max(depths[&b.0]),
                        Node::Rule(_, child) => depths[&child.0] + 1,
                        _ => 0usize,
                    },
                );
                continue;
            }
            if depths.contains_key(&id.0) {
                continue;
            }
            match self.nodes[id.0].1 {
                Node::Empty => {
                    depths.insert(id.0, 0);
                }
                Node::Failure(pos, label) => {
                    debug_assert_eq!(pos, self.nodes[root.0].0);
                    if labels.insert(label) {
                        expected.push(label);
                    }
                    depths.insert(id.0, 0);
                }
                Node::Join(a, b) => {
                    stack.push((id, true));
                    stack.push((b, false));
                    stack.push((a, false));
                }
                Node::Rule(_, child) => {
                    stack.push((id, true));
                    stack.push((child, false));
                }
            }
        }
        let mut deepest = Vec::new();
        let mut id = root;
        loop {
            match self.nodes[id.0].1 {
                Node::Join(a, b) => id = if depths[&a.0] >= depths[&b.0] { a } else { b },
                Node::Rule(rule, child) => {
                    deepest.push(rule);
                    id = child;
                }
                _ => break,
            }
        }
        Some((self.nodes[root.0].0, expected, deepest))
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

// D-014: 公開 farthest 候補の局所 frame。Rust の fail_at 主位置とは独立。
// 観測できるのは「最遠位置とその位置の label 集合」だけなので、frame ごとに
// (far, base, reached) を持ち、label は 1 本の log に追記する。frame の生存 label は
// log[base..] で、全て far 位置のもの。leave で親へ O(1)（同位置は連結済み、
// より遠い場合は移動、近い場合は切り捨て）に伝播する。失敗 1 件は O(1)。
// floor は Not の外で記録した最遠位置。Not の外の frame は破棄されないので、
// floor は最終的な最遠位置の下界であり、それより手前の label は（memo に保存しても
// 再生時に捨てられるため）記録しない。
// 成功 rule の失敗も残し、Not 成功時だけ frame を破棄する。memo は局所
// snapshot（far, reached, saved label 範囲）を再生するため、保存時の親 frame を混ぜない。
#[derive(Clone, Copy, Debug, Default)]
pub struct DisplaySnapshot {
    pub far: u32,
    pub reached: u32,
    pub labels: [u32; 2],
}
#[derive(Clone, Copy, Debug, Default)]
struct Frame {
    far: usize,
    base: usize,
    reached: usize,
}
pub struct DisplayDiagnostics {
    log: Vec<&'static str>,
    saved: Vec<&'static str>,
    frames: Vec<Frame>,
    floor: usize,
    guards: usize,
    work: usize,
    limit: usize,
    pub exhausted: bool,
}
#[inline]
fn same(a: &'static str, b: &'static str) -> bool {
    // 静的 label は同じ文字列定数を指すことが多い。内容比較は summary だけで行う。
    a.as_ptr() == b.as_ptr() && a.len() == b.len()
}
impl DisplayDiagnostics {
    pub fn new(limit: usize) -> Self {
        Self {
            log: Vec::with_capacity(64),
            saved: vec![],
            frames: vec![Frame::default()],
            floor: 0,
            guards: 0,
            work: 0,
            limit,
            exhausted: false,
        }
    }
    #[inline]
    pub fn enter(&mut self, reached: usize) {
        self.frames.push(Frame {
            far: 0,
            base: self.log.len(),
            reached,
        });
    }
    // Not の子 frame。成功時（子失敗）は discard、失敗時（子成功）は leave_guard。
    #[inline]
    pub fn enter_guard(&mut self, reached: usize) {
        self.guards += 1;
        self.enter(reached);
    }
    // 子 frame の観測を親へ伝える。
    #[inline]
    pub fn leave(&mut self) {
        let inner = self.frames.pop().expect("balanced diagnostic frame");
        let parent = self.frames.last_mut().expect("global diagnostic frame");
        parent.reached = parent.reached.max(inner.reached);
        let len = self.log.len();
        if len == inner.base {
            return;
        }
        if inner.base == parent.base {
            parent.far = inner.far;
        } else if inner.far > parent.far {
            self.log.copy_within(inner.base..len, parent.base);
            self.log.truncate(parent.base + (len - inner.base));
            parent.far = inner.far;
        } else if inner.far < parent.far {
            self.log.truncate(inner.base);
        }
    }
    #[inline]
    pub fn leave_guard(&mut self) {
        self.guards -= 1;
        self.leave();
        if self.guards == 0 {
            // Not の外へ出た観測は破棄されない。
            let frame = self.frames.last().expect("global diagnostic frame");
            if self.log.len() > frame.base {
                self.floor = self.floor.max(frame.far);
            }
        }
    }
    // Not 成功: 子 frame の観測（reached を含む）を捨てる。
    #[inline]
    pub fn discard(&mut self) {
        self.guards -= 1;
        let inner = self.frames.pop().expect("balanced diagnostic frame");
        self.log.truncate(inner.base);
    }
    // 現在 frame の局所 snapshot。memo と trivia cache が保存する。
    pub fn snapshot(&mut self) -> DisplaySnapshot {
        let frame = *self.frames.last().expect("global diagnostic frame");
        let start = self.saved.len();
        if self.log.len() > frame.base && frame.far >= self.floor {
            for i in frame.base..self.log.len() {
                let label = self.log[i];
                if !self.saved[start..].iter().any(|&s| same(s, label)) {
                    self.saved.push(label);
                }
            }
        }
        DisplaySnapshot {
            far: u32::try_from(frame.far).expect("input byte budget"),
            reached: u32::try_from(frame.reached).expect("input byte budget"),
            labels: [
                u32::try_from(start).expect("saved labels"),
                u32::try_from(self.saved.len()).expect("saved labels"),
            ],
        }
    }
    #[inline]
    pub fn reach(&mut self, reached: usize) {
        let frame = self.frames.last_mut().expect("global diagnostic frame");
        frame.reached = frame.reached.max(reached);
    }
    // 予算は「候補が残った記録の回数」（fail / failures / merge 呼出し単位）。
    #[inline]
    fn budget(&mut self) -> bool {
        self.work += 1;
        if self.work > self.limit {
            self.exhausted = true;
            return false;
        }
        true
    }
    // 現在 frame に pos の label を置ける状態にし、置けるなら base を返す。
    #[inline]
    fn open(&mut self, pos: usize) -> Option<usize> {
        let frame = self.frames.last_mut().expect("global diagnostic frame");
        frame.reached = frame.reached.max(pos);
        if pos < self.floor {
            return None;
        }
        if self.log.len() == frame.base || pos > frame.far {
            self.log.truncate(frame.base);
            frame.far = pos;
        } else if pos < frame.far {
            return None;
        }
        if self.guards == 0 {
            self.floor = pos;
        }
        Some(frame.base)
    }
    #[inline]
    pub fn fail(&mut self, pos: usize, label: &'static str) {
        let Some(base) = self.open(pos) else {
            return;
        };
        if !self.budget() {
            return;
        }
        self.log.push(label);
        if self.log.len() - base >= 64 {
            Self::dedupe(&mut self.log, base);
        }
    }
    pub fn failures(&mut self, pos: usize, labels: &[&'static str]) {
        let Some(base) = self.open(pos) else {
            return;
        };
        if labels.is_empty() || !self.budget() {
            return;
        }
        self.log.extend_from_slice(labels);
        if self.log.len() - base >= 64 {
            Self::dedupe(&mut self.log, base);
        }
    }
    // memo hit などで保存した局所 snapshot を現在 frame へ再生する。
    pub fn merge(&mut self, snapshot: DisplaySnapshot) {
        self.reach(snapshot.reached as usize);
        let range = snapshot.labels[0] as usize..snapshot.labels[1] as usize;
        if range.is_empty() {
            return;
        }
        let Some(base) = self.open(snapshot.far as usize) else {
            return;
        };
        if !self.budget() {
            return;
        }
        self.log.extend_from_slice(&self.saved[range]);
        if self.log.len() - base >= 64 {
            Self::dedupe(&mut self.log, base);
        }
    }
    // 同位置の label 集合は和集合。log が伸びたときだけ重複を落とす（意味は不変）。
    fn dedupe(log: &mut Vec<&'static str>, base: usize) {
        let mut keep = base;
        for i in base..log.len() {
            let label = log[i];
            if !log[base..keep].iter().any(|&s| same(s, label)) {
                log[keep] = label;
                keep += 1;
            }
        }
        log.truncate(keep);
    }
    // 全 frame を内側から畳み込む（parse 終了後は global frame だけ）。
    pub fn summary(&self) -> (usize, Vec<&'static str>) {
        let mut reached = 0;
        let mut far = 0;
        let mut segments: Vec<(usize, usize)> = vec![];
        let mut end = self.log.len();
        for frame in self.frames.iter().rev() {
            reached = reached.max(frame.reached);
            if frame.base < end {
                if segments.is_empty() || frame.far > far {
                    far = frame.far;
                    segments.clear();
                    segments.push((frame.base, end));
                } else if frame.far == far {
                    segments.push((frame.base, end));
                }
            }
            end = frame.base;
        }
        if segments.is_empty() {
            return (reached, vec![]);
        }
        let mut labels = Vec::new();
        for (start, end) in segments {
            for &label in &self.log[start..end] {
                if !labels.contains(&label) {
                    labels.push(label);
                }
            }
        }
        labels.sort();
        (far, labels)
    }
}
