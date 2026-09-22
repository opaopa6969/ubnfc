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
// 成功 rule の失敗も残し、Not 成功時だけ checkpoint に戻す。memo は局所
// snapshot を再生するため、保存時の親 frame を混ぜない。
#[derive(Clone, Copy, Debug, Default)]
pub struct DisplaySnapshot {
    pub diag: Diag,
    pub reached: usize,
}
pub struct DisplayDiagnostics {
    pub arena: Diagnostics,
    global: DisplaySnapshot,
    frames: Vec<DisplaySnapshot>,
}
impl DisplayDiagnostics {
    pub fn new(limit: usize) -> Self {
        Self {
            arena: Diagnostics::with_limit(limit),
            global: DisplaySnapshot::default(),
            frames: vec![],
        }
    }
    pub fn enter(&mut self, reached: usize) {
        self.frames.push(DisplaySnapshot::default());
        self.reach(reached);
    }
    pub fn leave(&mut self) -> DisplaySnapshot {
        self.frames.pop().expect("balanced diagnostic frame")
    }
    pub fn snapshot(&self) -> DisplaySnapshot {
        self.frames.last().copied().unwrap_or(self.global)
    }
    pub fn checkpoint(&self) -> (DisplaySnapshot, Vec<DisplaySnapshot>) {
        (self.global, self.frames.clone())
    }
    pub fn restore(&mut self, checkpoint: (DisplaySnapshot, Vec<DisplaySnapshot>)) {
        debug_assert_eq!(self.frames.len(), checkpoint.1.len());
        self.global = checkpoint.0;
        self.frames = checkpoint.1;
    }
    pub fn reach(&mut self, reached: usize) {
        self.global.reached = self.global.reached.max(reached);
        for frame in &mut self.frames {
            frame.reached = frame.reached.max(reached);
        }
    }
    pub fn fail(&mut self, pos: usize, label: &'static str) {
        let diag = self.arena.fail(pos, label);
        self.merge(DisplaySnapshot { diag, reached: pos });
    }
    pub fn failures(&mut self, pos: usize, labels: &[&'static str]) {
        for &label in labels {
            self.fail(pos, label);
        }
    }
    pub fn merge(&mut self, snapshot: DisplaySnapshot) {
        self.reach(snapshot.reached);
        self.global.diag = self.arena.join(self.global.diag, snapshot.diag);
        for frame in &mut self.frames {
            frame.diag = self.arena.join(frame.diag, snapshot.diag);
        }
    }
    pub fn summary(&self) -> (usize, Vec<&'static str>) {
        self.arena.summary(self.global.diag).map_or(
            (self.global.reached, vec![]),
            |(pos, mut labels, _)| {
                labels.sort();
                (pos, labels)
            },
        )
    }
}
