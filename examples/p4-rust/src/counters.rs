//! `counters` feature の計測フック。build.rs が複製 parser/runtime に呼出しを挿入する。
//! thread local なので並列テストでも混ざらない。計時は `Instant`（rule 単位、包含と自己時間）。
use std::cell::RefCell;
use std::time::Instant;

pub const LIMIT: usize = 0;
pub const ENTER_RULE: usize = 1;
pub const MARK: usize = 2;
pub const RESTORE: usize = 3;
pub const SAVE_EFFECTS: usize = 4;
pub const REPLAY_EFFECTS: usize = 5;
pub const EFFECT_AT: usize = 6;
pub const LOOKUP: usize = 7;
pub const MEMO_HIT: usize = 8;
pub const STORE: usize = 9;
pub const MEMO_INSERT: usize = 10;
pub const EVENT: usize = 11;
pub const JOIN: usize = 12;
pub const COMBINE: usize = 13;
pub const FAIL: usize = 14;
pub const PRIMITIVE: usize = 15;
pub const LITERAL: usize = 16;
pub const ANY: usize = 17;
pub const EOF: usize = 18;
pub const NUMBER: usize = 19;
pub const IDENTIFIER: usize = 20;
pub const QUOTED: usize = 21;
pub const UNTIL: usize = 22;
pub const SCAN: usize = 23;
pub const MAPPED_TEXT: usize = 24;
pub const NODE: usize = 25;
pub const DISPLAY_ENTER: usize = 26;
pub const DISPLAY_FAILURES: usize = 27;
pub const DISPLAY_REACH: usize = 28;
pub const DISPLAY_LEAVE: usize = 29;
pub const DIAG_PUSH: usize = 30;
pub const DIAG_JOIN: usize = 31;
pub const DIAG_RULE: usize = 32;
/// display の reach / merge が走査した frame 数の合計。
pub const FRAME_ITER: usize = 33;
/// display frame の最大深さ（max で更新）。
pub const MAX_FRAMES: usize = 34;
pub const NAMES: [&str; 35] = [
    "limit",
    "enter_rule",
    "mark",
    "restore",
    "save_effects",
    "replay_effects",
    "effect_at",
    "lookup",
    "memo_hit",
    "store",
    "memo_insert",
    "event",
    "join",
    "combine",
    "fail",
    "primitive",
    "literal",
    "any",
    "eof",
    "number",
    "identifier",
    "quoted",
    "until",
    "scan",
    "mapped_text",
    "node",
    "display_enter",
    "display_failures",
    "display_reach",
    "display_leave",
    "diag_push",
    "diag_join",
    "diag_rule",
    "display_frame_iterations",
    "display_max_frames",
];

#[derive(Clone, Copy, Debug, Default)]
pub struct RuleCounter {
    pub calls_consumed: u64,
    pub calls_match_only: u64,
    pub inclusive_ns: u64,
    pub self_ns: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ExpressionCounter {
    pub calls_consumed: u64,
    pub calls_match_only: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Counters {
    pub events: Vec<u64>,
    pub rules: Vec<RuleCounter>,
    pub expressions: Vec<ExpressionCounter>,
    /// 候補除外の失敗再生関数 `guard_e*`（§3.4）の呼出しと時間。rule と同じ包含/自己の扱い。
    pub guards: RuleCounter,
    /// 実行中 rule の子時間の累計（自己時間の算出用）。
    stack: Vec<u64>,
}

thread_local! {
    static COUNTERS: RefCell<Counters> = RefCell::new(Counters::fresh());
}

impl Counters {
    fn fresh() -> Self {
        Self {
            events: vec![0; NAMES.len()],
            rules: Vec::new(),
            expressions: Vec::new(),
            guards: RuleCounter::default(),
            stack: Vec::with_capacity(1024),
        }
    }
}

/// 計数をゼロに戻す。
pub fn reset() {
    COUNTERS.with(|c| *c.borrow_mut() = Counters::fresh());
}

/// 現在の計数の複製。
pub fn snapshot() -> Counters {
    COUNTERS.with(|c| c.borrow().clone())
}

#[inline]
pub fn bump(key: usize) {
    COUNTERS.with(|c| c.borrow_mut().events[key] += 1);
}

#[inline]
pub fn add(key: usize, amount: u64) {
    COUNTERS.with(|c| c.borrow_mut().events[key] += amount);
}

#[inline]
pub fn max(key: usize, value: u64) {
    COUNTERS.with(|c| {
        let mut c = c.borrow_mut();
        c.events[key] = c.events[key].max(value);
    });
}

#[inline]
pub fn expression(id: usize, match_only: bool) {
    COUNTERS.with(|c| {
        let mut c = c.borrow_mut();
        if c.expressions.len() <= id {
            c.expressions.resize(id + 1, ExpressionCounter::default());
        }
        let counter = &mut c.expressions[id];
        if match_only {
            counter.calls_match_only += 1;
        } else {
            counter.calls_consumed += 1;
        }
    });
}

/// rule 関数の入口で作り、drop で包含時間と自己時間を記録する。
pub struct Guard {
    /// None は失敗再生関数（guards に集計）。
    rule: Option<usize>,
    start: Instant,
}

impl Guard {
    pub fn guard() -> Self {
        COUNTERS.with(|c| {
            let mut c = c.borrow_mut();
            c.guards.calls_consumed += 1;
            c.stack.push(0);
        });
        Self {
            rule: None,
            start: Instant::now(),
        }
    }
    pub fn rule(rule: usize, match_only: bool) -> Self {
        COUNTERS.with(|c| {
            let mut c = c.borrow_mut();
            if c.rules.len() <= rule {
                c.rules.resize(rule + 1, RuleCounter::default());
            }
            if match_only {
                c.rules[rule].calls_match_only += 1;
            } else {
                c.rules[rule].calls_consumed += 1;
            }
            c.stack.push(0);
        });
        Self {
            rule: Some(rule),
            start: Instant::now(),
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let elapsed = u64::try_from(self.start.elapsed().as_nanos()).unwrap_or(u64::MAX);
        COUNTERS.with(|c| {
            let mut c = c.borrow_mut();
            let children = c.stack.pop().unwrap_or(0);
            let counter = match self.rule {
                Some(rule) => &mut c.rules[rule],
                None => &mut c.guards,
            };
            counter.inclusive_ns += elapsed;
            counter.self_ns += elapsed.saturating_sub(children);
            if let Some(parent) = c.stack.last_mut() {
                *parent += elapsed;
            }
        });
    }
}
