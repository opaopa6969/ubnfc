// ubnfc runtime template: diag

pub type ExpectedId = u32;
/// まだ rule 内で失敗していない場合の deepest_rule。生成側はこの ID を予約する。
pub const NO_RULE: u32 = u32::MAX;

/// memo に保存する局所診断。rule_path は対象 rule 自身を含む相対 stack。
/// expected が空なら失敗なし（offset 0 での失敗とは区別する）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub farthest: u32,
    pub expected: Vec<ExpectedId>,
    pub deepest_rule: u32,
    pub rule_path: Vec<u32>,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            farthest: 0,
            expected: Vec::new(),
            deepest_rule: NO_RULE,
            rule_path: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub rule_id: u32,
    local: Snapshot,
}

/// 構文診断は成功した子 rule の失敗も保持し、scope transaction とは別に管理する。
/// pos の単位と consumed/matched の選択は呼出し側が決める（設計 D-011 は未確定）。
#[derive(Clone, Debug)]
pub struct Diagnostics {
    pub farthest: u32,
    pub expected: Vec<ExpectedId>,
    pub deepest_rule: u32,
    pub frames: Vec<Frame>,
    deepest_stack: Vec<u32>,
    suppression: usize,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self {
            farthest: 0,
            expected: Vec::new(),
            deepest_rule: NO_RULE,
            frames: Vec::new(),
            deepest_stack: Vec::new(),
            suppression: 0,
        }
    }
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enter_rule(&mut self, rule_id: u32) {
        self.frames.push(Frame {
            rule_id,
            local: Snapshot::default(),
        });
    }

    /// 成功/失敗のどちらでも局所記録を返す。global と親への寄与は既に反映済み。
    pub fn leave_rule(&mut self) -> Option<Snapshot> {
        self.frames.pop().map(|frame| frame.local)
    }

    pub fn fail_at(&mut self, pos: u32, expected_id: ExpectedId) {
        if self.suppression == 0 {
            let path: Vec<_> = self.frames.iter().map(|frame| frame.rule_id).collect();
            self.record(pos, &[expected_id], &path);
        }
    }

    pub fn suppress_begin(&mut self) {
        self.suppression = self
            .suppression
            .checked_add(1)
            .expect("suppression overflow");
    }

    pub fn suppress_end(&mut self) {
        self.suppression = self
            .suppression
            .checked_sub(1)
            .expect("suppress_begin と対にすること");
    }

    /// active rule があればその局所記録だけを保存する。呼出元や先行 sibling の
    /// global failure を混入させない。active rule がなければ global snapshot。
    pub fn snapshot(&self) -> Snapshot {
        self.frames.last().map_or_else(
            || Snapshot {
                farthest: self.farthest,
                expected: self.expected.clone(),
                deepest_rule: self.deepest_rule,
                rule_path: self.deepest_stack.clone(),
            },
            |frame| frame.local.clone(),
        )
    }

    /// memo hit では対象 rule の enter_rule を呼ぶ前に実行する。
    /// 保存した suffix を現在の呼出元 stack に接続し、global と全 active frame に再生。
    pub fn merge(&mut self, snapshot: &Snapshot) {
        if self.suppression != 0 || snapshot.expected.is_empty() {
            return;
        }
        let path: Vec<_> = self
            .frames
            .iter()
            .map(|frame| frame.rule_id)
            .chain(snapshot.rule_path.iter().copied())
            .collect();
        self.record(snapshot.farthest, &snapshot.expected, &path);
    }

    fn record(&mut self, pos: u32, hints: &[ExpectedId], path: &[u32]) {
        merge_failure(
            &mut self.farthest,
            &mut self.expected,
            &mut self.deepest_stack,
            pos,
            hints,
            path,
        );
        self.deepest_rule = self.deepest_stack.last().copied().unwrap_or(NO_RULE);
        for (depth, frame) in self.frames.iter_mut().enumerate() {
            let local = &mut frame.local;
            merge_failure(
                &mut local.farthest,
                &mut local.expected,
                &mut local.rule_path,
                pos,
                hints,
                &path[depth..],
            );
            local.deepest_rule = local.rule_path.last().copied().unwrap_or(NO_RULE);
        }
    }
}

// 同位置の expected は初出順で重複排除。最遠位置の stack は最深を採り、同深度は先勝ち。
fn merge_failure(
    farthest: &mut u32,
    expected: &mut Vec<ExpectedId>,
    path: &mut Vec<u32>,
    pos: u32,
    hints: &[ExpectedId],
    candidate: &[u32],
) {
    if !expected.is_empty() && pos < *farthest {
        return;
    }
    if expected.is_empty() || pos > *farthest {
        *farthest = pos;
        expected.clear();
        path.clear();
    }
    for &hint in hints {
        if !expected.contains(&hint) {
            expected.push(hint);
        }
    }
    if candidate.len() > path.len() {
        path.clear();
        path.extend_from_slice(candidate);
    }
}
