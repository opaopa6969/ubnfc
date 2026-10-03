use super::api::*;
use super::ast::tree::AstTree;
use super::rt::{
    diag::{Diag, Diagnostics, DisplayDiagnostics, DisplaySnapshot},
    input::Input,
    lexcache::{LexCache, SkipCache},
    memo::{Key, Memo},
    scope::{Checkpoint, ScopeStore},
};

// issue #35 / D-070: ネイティブスタック番兵と、それに当たったときのエスカレーション先
// thread のサイズをここへ集める。Rust cannot recover from native stack overflow, so a
// proactive check must fire before the real stack is exhausted — but the check cannot
// know the calling thread's actual stack size. 既定呼出し（呼出し元 thread で直接走る）は
// 保守的な小さい値のままにし（呼出し元が明示的に小さい stack で呼んでも安全）、
// `max_depth` にまだ達していないのに番兵へ当たったときだけ、`max_depth` から見積もった
// 十分な stack を持つ専用 thread へ 1 度だけエスカレーションする（`parse_with_options` /
// `parse_entry_with_options` 側。外部 `TokenScanner` を受け取る `parse_with_scanner` は
// 任意の状態を持つ scanner を thread 境界へ渡せないので対象外 — 既知の制約として
// docs/reports/2026-09-24-runtime-limits.md に記録）。
//
// 実測（同報告）: JSON 入れ子（typed AST）は深さ単位（`enter_rule` 呼出し）あたり約 568 byte、
// P4 の括弧入れ子は約 620–750 byte。3 倍前後の安全率を見て 1 深さ単位 2048 byte とする。
/// 既定呼出し（呼出し元 thread）の番兵。旧来の固定値のまま（呼出し元が 1 MiB 未満の
/// 明示的に小さい stack で呼んでも安全）。
pub(crate) const NATIVE_STACK_SENTINEL_BYTES: usize = 256 * 1024;
/// エスカレーション先 thread が使ってよい native stack の目安（byte）。
pub(crate) fn escalated_stack_budget(max_depth: usize) -> usize {
    const BYTES_PER_DEPTH: usize = 2048;
    max_depth.saturating_mul(BYTES_PER_DEPTH)
}
/// エスカレーション先 thread に実際に確保する stack size。番兵の閾値より確実に大きくし
/// （番兵が実枯渇より先に必ず発火するように）、下限 8 MiB・上限 512 MiB に丸める。
pub(crate) fn escalated_thread_stack_bytes(max_depth: usize) -> usize {
    escalated_stack_budget(max_depth)
        .saturating_add(2 * 1024 * 1024)
        .clamp(8 * 1024 * 1024, 512 * 1024 * 1024)
}

// issue #35 / D-071: 既定の `arena_entries`（2,000,000）は 1 MB の入力を typed AST 付きで
// 解析すると足りない（実測 4.29 entries/byte、docs/reports/2026-09-24-runtime-limits.md）。
// 明示的に小さい値を設定したテスト・呼出し（`resource_limits_fail_without_partial_success` 等）
// を壊さないよう、既定値と厳密に一致するときだけ入力長に比例して引き上げる。
/// 実測の最大（Operators-512 の 9.00 entries/byte）に約 1.8 倍の安全率を見た値。
const ARENA_ENTRIES_PER_BYTE: usize = 16;
/// 既定スケーリングの上限（約 2,000,000 byte 入力相当）。それを超える入力は
/// `arena_entries` を明示するか、`ResourceLimits::default()` 以外の値を選ぶ。
const ARENA_ENTRIES_CEILING: usize = 32_000_000;
pub(crate) fn scaled_default_arena_entries(input_bytes: usize) -> usize {
    DEFAULT_ARENA_ENTRIES
        .max(input_bytes.saturating_mul(ARENA_ENTRIES_PER_BYTE))
        .min(ARENA_ENTRIES_CEILING)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EventId(pub usize);
#[derive(Clone, Copy, Debug)]
pub(crate) enum Event {
    Empty,
    /// 回復した領域。残りの観測（規則・mode・同期範囲・診断・公開候補）は
    /// `Session::recovery_events[detail]` にある（event arena の要素を小さく保つため）。
    Recovery {
        span: Span,
        detail: u32,
    },
    Join(EventId, EventId),
    Values {
        span: Span,
        child: EventId,
        wrap: bool,
        /// D-078: 意味値を持つ選択の text だけの候補。一致範囲の字句（空でも）を 1 つの値にする。
        text: bool,
    },
    Capture {
        token_extent: bool,
        site: usize,
        span: Span,
        child: EventId,
    },
    Rule {
        rule: usize,
        span: Span,
        child: EventId,
        /// この rule の直下 capture が `caps_flat` のどこにあるか（開始, 個数）。
        /// 出現収集が書き、AST 構築が読む。event を読むときに同じ cache line に載るので、
        /// event id で索く別表（x64 で 640 KB の疎な書き込み）を持たない。
        caps: (u32, u32),
    },
    Token {
        text_span: Option<Span>,
        content_span: Option<Span>,
        expr: usize,
        rule: usize,
        span: Span,
    },
}
/// 回復 event の本体（`Event::Recovery::detail` が索く）。
#[derive(Clone, Copy, Debug)]
pub(crate) struct RecoveryEvent {
    rule: usize,
    mode: &'static str,
    sync_span: Option<Span>,
    diag: Diag,
    /// `Session::recovery_hints` の添字。回復した規則の frame に属する公開候補
    /// （D-027 の表示語彙）を回復時に取っておく（診断は後段の走査で組み立てる）。
    hints: u32,
}
/// event arena の 1 要素（D-077、32 byte。`Event` のままだと 88 byte で、JSON 1 MB の
/// typed AST では 156 万 event × 88 byte の arena とその伸長が確保の 7 割を占めていた）。
/// 位置・event id・site / expr の番号は u32 に収まる（入力は `u32::MAX` byte 以下、arena は
/// `u32::MAX` 要素以下で打ち切る）。Recovery の本体は別表（`RecoveryEvent`）。
/// 読むときは `Session::ev` で `Event` に戻す（値の複製。書き換えは `set_ev`）。
/// `unpack` の結果は scalar だけなので、`match self.ev(id)` は memory へ実体化されない。
#[derive(Clone, Copy, Debug)]
pub(crate) struct PackedEvent {
    tag: u8,
    flags: u8,
    /// Token の規則（`u16::MAX` は無し）。
    rule: u16,
    w: [u32; 7],
}
const EV_EMPTY: u8 = 0;
const EV_RECOVERY: u8 = 1;
const EV_JOIN: u8 = 2;
const EV_VALUES: u8 = 3;
const EV_CAPTURE: u8 = 4;
const EV_RULE: u8 = 5;
const EV_TOKEN: u8 = 6;
const NO_RULE: u16 = u16::MAX;
#[inline(always)]
fn w32(value: usize) -> u32 {
    debug_assert!(value <= u32::MAX as usize);
    value as u32
}
#[inline(always)]
fn wide(value: u32) -> usize {
    if value == u32::MAX {
        usize::MAX
    } else {
        value as usize
    }
}
impl PackedEvent {
    const EMPTY: Self = Self {
        tag: EV_EMPTY,
        flags: 0,
        rule: 0,
        w: [0; 7],
    };
    #[inline(always)]
    fn pack(event: Event) -> Self {
        let mut p = Self::EMPTY;
        match event {
            Event::Empty => {}
            Event::Recovery { span, detail } => {
                p.tag = EV_RECOVERY;
                p.w[0] = w32(span[0]);
                p.w[1] = w32(span[1]);
                p.w[2] = detail;
            }
            Event::Join(a, b) => {
                p.tag = EV_JOIN;
                p.w[0] = w32(a.0);
                p.w[1] = w32(b.0);
            }
            Event::Values {
                span,
                child,
                wrap,
                text,
            } => {
                p.tag = EV_VALUES;
                p.flags = wrap as u8 | (text as u8) << 1;
                p.w[..3].copy_from_slice(&[w32(span[0]), w32(span[1]), w32(child.0)]);
            }
            Event::Capture {
                token_extent,
                site,
                span,
                child,
            } => {
                p.tag = EV_CAPTURE;
                p.flags = token_extent as u8;
                p.w[..4].copy_from_slice(&[w32(site), w32(span[0]), w32(span[1]), w32(child.0)]);
            }
            Event::Rule {
                rule,
                span,
                child,
                caps,
            } => {
                p.tag = EV_RULE;
                p.w[..6].copy_from_slice(&[
                    w32(rule),
                    w32(span[0]),
                    w32(span[1]),
                    w32(child.0),
                    caps.0,
                    caps.1,
                ]);
            }
            Event::Token {
                text_span,
                content_span,
                expr,
                rule,
                span,
            } => {
                p.tag = EV_TOKEN;
                p.rule = if rule == usize::MAX {
                    NO_RULE
                } else {
                    debug_assert!(rule < NO_RULE as usize);
                    rule as u16
                };
                p.w[0] = w32(span[0]);
                p.w[1] = w32(span[1]);
                if let Some(t) = text_span {
                    p.flags |= 1;
                    p.w[2] = w32(t[0]);
                    p.w[3] = w32(t[1]);
                }
                if let Some(c) = content_span {
                    p.flags |= 2;
                    p.w[4] = w32(c[0]);
                    p.w[5] = w32(c[1]);
                }
                p.w[6] = if expr == usize::MAX {
                    u32::MAX
                } else {
                    w32(expr)
                };
            }
        }
        p
    }
    #[inline(always)]
    fn unpack(self) -> Event {
        let w = self.w;
        let span = |i: usize| [w[i] as usize, w[i + 1] as usize];
        match self.tag {
            EV_JOIN => Event::Join(EventId(w[0] as usize), EventId(w[1] as usize)),
            EV_VALUES => Event::Values {
                span: span(0),
                child: EventId(w[2] as usize),
                wrap: self.flags & 1 != 0,
                text: self.flags & 2 != 0,
            },
            EV_CAPTURE => Event::Capture {
                token_extent: self.flags != 0,
                site: w[0] as usize,
                span: span(1),
                child: EventId(w[3] as usize),
            },
            EV_RULE => Event::Rule {
                rule: w[0] as usize,
                span: span(1),
                child: EventId(w[3] as usize),
                caps: (w[4], w[5]),
            },
            EV_TOKEN => Event::Token {
                text_span: (self.flags & 1 != 0).then(|| span(2)),
                content_span: (self.flags & 2 != 0).then(|| span(4)),
                expr: wide(w[6]),
                rule: if self.rule == NO_RULE {
                    usize::MAX
                } else {
                    self.rule as usize
                },
                span: span(0),
            },
            EV_RECOVERY => Event::Recovery {
                span: span(0),
                detail: w[2],
            },
            _ => Event::Empty,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Step {
    pub ok: bool,
    pub state: State,
    pub events: EventId,
    pub diag: Diag,
}
impl Step {
    pub fn yes(state: State) -> Self {
        Self {
            ok: true,
            state,
            events: EventId(0),
            diag: Diag::NONE,
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Saved {
    consumed: u32,
    matched: u32,
    events: u32,
    diag: u32,
    diag_pos: u32,
    flags: u32,
    effects: [u32; 2],
    display: DisplaySnapshot,
}
impl Saved {
    fn new(step: Step, effects: Span, display: DisplaySnapshot) -> Self {
        Self {
            display,
            consumed: u32::try_from(step.state.consumed).expect("cursor"),
            matched: u32::try_from(step.state.matched).expect("cursor"),
            events: u32::try_from(step.events.0).expect("arena"),
            diag: step.diag.id,
            diag_pos: step.diag.pos,
            flags: step.ok as u32
                | ((step.state.invert as u32) << 1)
                | ((step.state.reset as u32) << 2),
            effects: effects.map(|p| u32::try_from(p).expect("effects")),
        }
    }
    fn step(self) -> Step {
        Step {
            ok: self.flags & 1 != 0,
            state: State {
                consumed: self.consumed as usize,
                matched: self.matched as usize,
                invert: self.flags & 2 != 0,
                reset: self.flags & 4 != 0,
            },
            events: EventId(self.events as usize),
            diag: Diag {
                id: self.diag,
                pos: self.diag_pos,
            },
        }
    }
}
/// capture の出現（site, span, 子 event）。
pub(crate) type Cap = (usize, Span, EventId);
#[derive(Clone, Copy)]
pub(crate) struct Mark {
    scope: Checkpoint,
    effects: usize,
}
// 効果は Rc で共有する。memo の save / replay は参照を複製するだけで文字列を複製しない
// （scope 側の Decl / Reference は結果 API なので所有する）。
#[derive(Clone)]
struct Effect {
    value: std::rc::Rc<ScanEffect>,
    mode: &'static str,
    offset_cp: usize,
    length_cp: usize,
    /// D-029: enter / leave が属する規則名。declare / use は効果自身が名前を持つ。
    name: Option<&'static str>,
}
/// parse をまたいで再利用する大きな buffer（`ParseOptions::reuse_buffers`）。
/// 内容は parse ごとに捨て、確保だけを thread ごとに 1 組保持する。大きな入力を扱った後は
/// `POOL_RETAIN_BYTES` を超える組を捨てて、常駐量を抑える。結果には影響しない。
pub(crate) struct Buffers {
    arena: Vec<PackedEvent>,
    diag: Diagnostics,
    display: DisplayDiagnostics,
    memo: Memo<Saved>,
    trivia_cache: Memo<(Step, DisplaySnapshot)>,
    lex: LexCache,
    trivia_skip: SkipCache,
    effects: Vec<Effect>,
    saved_effects: Vec<Effect>,
    stacks: Vec<Vec<(EventId, bool)>>,
    captures_pool: Vec<Vec<Cap>>,
    values_pool: Vec<Vec<u32>>,
    scope_pool: Vec<Vec<(usize, Span)>>,
    /// AST の構築先（D-077）。所有 `Ast` だけを返す parse では木を捨てずに再利用する。
    tree: AstTree,
    caps_flat: Vec<Cap>,
    caps_pending: Vec<Cap>,
}
const POOL_RETAIN_BYTES: usize = 64 << 20;
/// 位置ごとの字句 cache の上限（入力長に比例するため）。超える入力では cache を使わない。
const LEX_CACHE_MAX_BYTES: usize = 256 << 20;
thread_local! {
    static POOL: std::cell::RefCell<Option<Buffers>> = const { std::cell::RefCell::new(None) };
}
impl Buffers {
    fn bytes(&self) -> usize {
        self.arena.capacity() * std::mem::size_of::<PackedEvent>()
            + self.diag.capacity_bytes()
            + self.display.capacity_bytes()
            + self.memo.capacity_bytes()
            + self.trivia_cache.capacity_bytes()
            + self.lex.capacity_bytes()
            + self.trivia_skip.capacity_bytes()
            + (self.effects.capacity() + self.saved_effects.capacity())
                * std::mem::size_of::<Effect>()
            + (self.caps_flat.capacity() + self.caps_pending.capacity())
                * std::mem::size_of::<Cap>()
    }
}
/// 生成 parser の session。`DIAG` は診断を記録するかの単相化 flag で、`ParseOptions::diagnostics`
/// と 1 対 1 に対応する（D-036）。`false` は fast mode で、主診断 DAG と公開 farthest の配管が
/// 機械語から消える。
pub(crate) struct Session<'a, const DIAG: bool> {
    pub text: &'a str,
    pub input: Input<'a>,
    pub options: ParseOptions,
    pub scanner: &'a mut dyn TokenScanner,
    pub arena: Vec<PackedEvent>,
    /// `Event::Recovery` の本体（`Event::Recovery::detail` が索く）。
    recovery_events: Vec<RecoveryEvent>,
    pub diag: Diagnostics,
    pub display: DisplayDiagnostics,
    pub memo: Memo<Saved>,
    pub scope: ScopeStore,
    pub trivia_cache: Memo<(Step, DisplaySnapshot)>,
    /// 位置ごとの字句 cache（統合 DFA の結果）。key は位置だけ（rt/lexcache.rs の健全性議論）。
    pub lex: LexCache,
    /// trivia policy ごとの「次の非 trivia 位置」（診断を記録しない読み飛ばし）。
    pub trivia_skip: SkipCache,
    /// cache を引くか（`ParseOptions::lexical_cache` と、文法に cache 対象があるか）。
    pub lex_enabled: bool,
    effects: Vec<Effect>,
    saved_effects: Vec<Effect>,
    pub statistics: Statistics,
    /// AST の構築先（D-077）。節点・Text はここへ積み、所有 `Ast` はここから写す。
    pub tree: AstTree,
    /// AST 構築の走査用 stack の再利用 pool（再帰するので複数）。
    stacks: Vec<Vec<(EventId, bool)>>,
    /// AST 構築の作業 Vec の再利用 pool（capture 出現、field 値、field 文字列）。
    captures_pool: Vec<Vec<Cap>>,
    values_pool: Vec<Vec<u32>>,
    scope_pool: Vec<Vec<(usize, Span)>>,
    /// 出現収集で組み立てる「rule ごとの直下 capture」の並び。どの区間がどの rule のものかは
    /// `Event::Rule::caps`（開始, 個数）が持つ。AST 構築はその区間を写すだけで木を再走査しない。
    caps_flat: Vec<Cap>,
    caps_pending: Vec<Cap>,
    pub depth: usize,
    stack_anchor: Option<usize>,
    depth_failure: Option<usize>,
    /// D-070: `depth_failure` が native stack 番兵（`native_stack_budget`）によるものか。
    /// true なら `options.max_depth` へはまだ達していない失敗で、大きい stack を持つ
    /// thread へのエスカレーションで通る可能性がある（`enter_rule` は論理上限に先に
    /// 当たっていれば `limit` を呼ぶ前に `depth_failure` を確定させるので、両方が
    /// 同時に true になることはない）。
    depth_failure_is_stack: bool,
    /// この session が使ってよい native stack の目安（byte）。既定呼出しは
    /// `NATIVE_STACK_SENTINEL_BYTES`、エスカレーション後の thread は
    /// `escalated_stack_budget(options.max_depth)`。
    native_stack_budget: usize,
    /// D-071: 実効の arena 上限（byte 単位の `arena_entries`）。既定値のときだけ
    /// `scaled_default_arena_entries` で入力長に応じて引き上げたもの。
    arena_limit: usize,
    pub resource_failure: Option<(usize, &'static str)>,
    effect_work: usize,
    effect_bytes: usize,
    mapping_depth: usize,
    mapping_anchor: Option<usize>,
    /// #86: いま値を組み立てている意味値の領域（`Values{text:false}`）の入れ子の深さ。
    /// 写像される規則の本体に入るたびに 0 から数え直す。
    value_region: u32,
    /// 回復ごとの公開候補（最遠位置 byte, label）。`Event::Recovery::hints` が索く。
    recovery_hints: Vec<Option<(usize, Vec<&'static str>)>>,
}
impl<'a, const DIAG: bool> Session<'a, DIAG> {
    // 診断の配管は `const DIAG` で単相化する（D-036）。fast mode（`ParseOptions::diagnostics ==
    // false`）は `Session::<false>` で走り、以下の wrapper が定数 false の分岐になるので、主診断
    // DAG（`Diagnostics`）と公開 farthest（`DisplayDiagnostics`）への呼出しが機械語から消える。
    // 実行時の `enabled` 検査だけでは、呼出し・引数の準備・`Step.diag` の合流が残っていた。
    // 診断モードは `Session::<true>` で従来どおり記録する。観測は両者で変わらない。
    #[inline(always)]
    pub fn diag_fail(&mut self, pos: usize, label: &'static str) -> Diag {
        if DIAG {
            self.diag.fail(pos, label)
        } else {
            Diag::NONE
        }
    }
    #[inline(always)]
    pub fn diag_join(&mut self, a: Diag, b: Diag) -> Diag {
        if DIAG {
            self.diag.join(a, b)
        } else {
            Diag::NONE
        }
    }
    #[inline(always)]
    pub fn diag_rule(&mut self, rule: usize, child: Diag) -> Diag {
        if DIAG {
            self.diag.rule(rule, child)
        } else {
            Diag::NONE
        }
    }
    #[inline(always)]
    pub fn display_reach(&mut self, reached: usize) {
        if DIAG {
            self.display.reach(reached);
        }
    }
    #[inline(always)]
    pub fn display_failures(&mut self, pos: usize, labels: &'static [&'static str]) {
        if DIAG {
            self.display.failures(pos, labels);
        }
    }
    #[inline(always)]
    pub fn display_fail(&mut self, pos: usize, label: &'static str) {
        if DIAG {
            self.display.fail(pos, label);
        }
    }
    #[inline(always)]
    pub fn display_local_far(&self) -> Option<usize> {
        if DIAG {
            self.display.local_far()
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn display_enter(&mut self, reached: usize) {
        if DIAG {
            self.display.enter(reached);
        }
    }
    #[inline(always)]
    pub fn display_leave(&mut self) {
        if DIAG {
            self.display.leave();
        }
    }
    #[inline(always)]
    pub fn display_enter_guard(&mut self, reached: usize) {
        if DIAG {
            self.display.enter_guard(reached);
        }
    }
    #[inline(always)]
    pub fn display_leave_guard(&mut self) {
        if DIAG {
            self.display.leave_guard();
        }
    }
    #[inline(always)]
    pub fn display_discard(&mut self) {
        if DIAG {
            self.display.discard();
        }
    }
    #[inline(always)]
    pub fn display_snapshot(&mut self) -> DisplaySnapshot {
        if DIAG {
            self.display.snapshot()
        } else {
            DisplaySnapshot::default()
        }
    }
    #[inline(always)]
    pub fn display_merge(&mut self, snapshot: DisplaySnapshot) {
        if DIAG {
            self.display.merge(snapshot);
        }
    }

    pub fn new(
        text: &'a str,
        options: ParseOptions,
        scanner: &'a mut dyn TokenScanner,
        native_stack_budget: usize,
    ) -> Self {
        let rejected = text.len() > options.limits.input_bytes.min(u32::MAX as usize);
        let text = if rejected { "" } else { text };
        // D-071: 既定値のときだけ入力長に応じて引き上げる（明示的に設定した値はそのまま使う）。
        let arena_limit = if options.limits.arena_entries == DEFAULT_ARENA_ENTRIES {
            scaled_default_arena_entries(text.len())
        } else {
            options.limits.arena_entries
        };
        let arena_capacity = if options.wants_ast() || options.lexical || HAS_SCOPE || HAS_RECOVERY {
            // P4 実測 約 4.4 event / byte（小さい入力はこの見積りで倍々伸長を避ける）。
            // D-077: 大きい入力は 2 event / byte を下限に見積もる（JSON は D-072 後 1.56、
            // P4 3.8、Operators 9.0）。以前は 32,768 で打ち切って倍々で伸ばしていたので、
            // pool の無い最初の parse は JSON 1 MB で 210 万 event 分を 7 回確保し直していた。
            text.len()
                .saturating_mul(5)
                .min(32768)
                .max(text.len().saturating_mul(2))
                .min(arena_limit)
                .saturating_add(8)
        } else {
            1
        };
        // fast mode（`ParseOptions::diagnostics == false`）は主診断 DAG も公開 farthest も
        // 記録しない。arena は確保せず、予算（limit 0）も消費しない。
        // 単相化（D-036）と `ParseOptions::diagnostics` は入口で 1 対 1 に対応する。
        let diag_on = DIAG;
        let diag_limit = if diag_on { options.limits.diagnostics } else { 0 };
        let diag_capacity = if diag_on {
            text.len().saturating_mul(24).saturating_add(64).min(1 << 20)
        } else {
            0
        };
        // memo は rule 本体単位（P4 実測 約 2.3 entry / byte。候補除外で失敗 entry が減った）。
        let memo_capacity = text.len().saturating_mul(3).saturating_add(64).min(1 << 20);
        // 字句 cache は位置で直接引く（位置は 0..=text.len()、EOF 位置を含む）。
        // 文法に cache 対象の literal / trivia policy が無ければ 0 を渡して確保しない。
        // 表は入力長に比例するので、割に合わない大きさになるときは cache を使わない。
        let lex_bytes = (text.len() + 1)
            .saturating_mul(LEX_WORDS * std::mem::size_of::<u64>() + std::mem::size_of::<u32>());
        let lex_enabled = options.lexical_cache
            && !rejected
            && lex_bytes <= LEX_CACHE_MAX_BYTES
            && (LEX_WORDS != 0 || TRIVIA_POLICIES != 0);
        let lex_words = if lex_enabled { LEX_WORDS } else { 0 };
        let skip_policies = if lex_enabled { TRIVIA_POLICIES } else { 0 };
        let lex_positions = if lex_enabled { text.len() + 1 } else { 0 };
        let pooled = if options.reuse_buffers {
            POOL.with(|p| p.borrow_mut().take())
        } else {
            None
        };
        let mut buffers = match pooled {
            Some(mut b) => {
                b.arena.clear();
                b.diag.reset(diag_limit);
                b.display.reset(options.limits.diagnostics);
                b.memo.reset(memo_capacity);
                // trivia cache は前回の entry 数を見積りにする（0 だと毎 parse 縮小と再拡張を繰り返す）。
                let trivia_capacity = b.trivia_cache.len();
                b.trivia_cache.reset(trivia_capacity);
                b.lex.reset(lex_words, lex_positions);
                b.trivia_skip.reset(skip_policies, lex_positions);
                b.effects.clear();
                b.saved_effects.clear();
                b.tree.clear();
                b.values_pool.iter_mut().for_each(Vec::clear);
                b
            }
            None => Buffers {
                arena: Vec::with_capacity(arena_capacity),
                diag: Diagnostics::with_limit_and_capacity(diag_limit, diag_capacity),
                display: DisplayDiagnostics::new(options.limits.diagnostics),
                memo: Memo::with_capacity(memo_capacity),
                trivia_cache: Memo::default(),
                lex: {
                    let mut lex = LexCache::default();
                    lex.reset(lex_words, lex_positions);
                    lex
                },
                trivia_skip: {
                    let mut cache = SkipCache::default();
                    cache.reset(skip_policies, lex_positions);
                    cache
                },
                effects: vec![],
                saved_effects: vec![],
                stacks: vec![],
                captures_pool: vec![],
                values_pool: vec![],
                scope_pool: vec![],
                tree: AstTree::default(),
                caps_flat: vec![],
                caps_pending: vec![],
            },
        };
        buffers.diag.enable(diag_on);
        buffers.display.enable(diag_on);
        let mut arena = buffers.arena;
        arena.push(PackedEvent::EMPTY);
        Self {
            text,
            input: Input::new(text),
            options,
            scanner,
            arena,
            diag: buffers.diag,
            display: buffers.display,
            memo: buffers.memo,
            scope: ScopeStore::new(),
            trivia_cache: buffers.trivia_cache,
            lex: buffers.lex,
            trivia_skip: buffers.trivia_skip,
            lex_enabled,
            effects: buffers.effects,
            saved_effects: buffers.saved_effects,
            statistics: Statistics::default(),
            tree: buffers.tree,
            stacks: buffers.stacks,
            captures_pool: buffers.captures_pool,
            values_pool: buffers.values_pool,
            scope_pool: buffers.scope_pool,
            caps_flat: buffers.caps_flat,
            caps_pending: buffers.caps_pending,
            depth: 0,
            stack_anchor: None,
            depth_failure: None,
            depth_failure_is_stack: false,
            native_stack_budget,
            arena_limit,
            resource_failure: rejected.then_some((0, "input byte budget exceeded")),
            effect_work: 0,
            effect_bytes: 0,
            mapping_depth: 0,
            mapping_anchor: None,
            value_region: 0,
            recovery_hints: Vec::new(),
            recovery_events: Vec::new(),
        }
    }
    /// 大きな buffer を thread の pool へ返す（`finish` の最後）。
    fn recycle(&mut self) {
        if !self.options.reuse_buffers {
            return;
        }
        let mut buffers = Buffers {
            arena: std::mem::take(&mut self.arena),
            diag: std::mem::replace(&mut self.diag, Diagnostics::with_limit_and_capacity(0, 0)),
            display: std::mem::replace(&mut self.display, DisplayDiagnostics::new(0)),
            memo: std::mem::take(&mut self.memo),
            trivia_cache: std::mem::take(&mut self.trivia_cache),
            lex: std::mem::take(&mut self.lex),
            trivia_skip: std::mem::take(&mut self.trivia_skip),
            effects: std::mem::take(&mut self.effects),
            saved_effects: std::mem::take(&mut self.saved_effects),
            stacks: std::mem::take(&mut self.stacks),
            captures_pool: std::mem::take(&mut self.captures_pool),
            values_pool: std::mem::take(&mut self.values_pool),
            scope_pool: std::mem::take(&mut self.scope_pool),
            tree: std::mem::take(&mut self.tree),
            caps_flat: std::mem::take(&mut self.caps_flat),
            caps_pending: std::mem::take(&mut self.caps_pending),
        };
        if buffers.bytes() > POOL_RETAIN_BYTES {
            // 伸長の倍々で余った容量だけで上限を超えることがある（JSON 1 MB: event 156 万に
            // 容量 210 万）。使った分へ縮めて収まるなら保持する。同じ規模の次の parse は
            // 伸長しないので、縮めるのは規模が変わったときだけ。
            buffers.arena.shrink_to_fit();
            buffers.caps_flat.shrink_to_fit();
            buffers.caps_pending.shrink_to_fit();
        }
        if buffers.bytes() <= POOL_RETAIN_BYTES {
            POOL.with(|p| *p.borrow_mut() = Some(buffers));
        }
    }
    // Rust cannot recover from native stack overflow. Stop before consuming the
    // per-parse 256 KiB native stack budget, in addition to the logical rule limit.
    // Check expressions too: one rule may contain a deeply nested expression tree.
    #[inline(never)]
    pub fn limit(&mut self, state: State) -> Option<Step> {
        if self.diag.exhausted || self.display.exhausted {
            self.resource_failure
                .get_or_insert((state.consumed, "diagnostic budget exceeded"));
        }
        if let Some((offset, label)) = self.resource_failure {
            return Some(self.fail(state, offset, label));
        }
        let frame = 0u8;
        let address = std::ptr::addr_of!(frame) as usize;
        let anchor = *self.stack_anchor.get_or_insert(address);
        if self.depth_failure.is_none() && anchor.abs_diff(address) > self.native_stack_budget {
            self.depth_failure = Some(state.consumed.max(state.matched));
            self.depth_failure_is_stack = true;
        }
        self.depth_failure
            .map(|offset| self.fail(state, offset, "maximum parse depth exceeded"))
    }
    /// D-070: 直近の `depth_failure` が native stack 番兵によるものだったか
    /// （`options.max_depth` にはまだ達していない）。エスカレーション判定に使う。
    #[inline]
    pub fn stack_sentinel_tripped(&self) -> bool {
        self.depth_failure_is_stack
    }
    // 候補除外の失敗再生（§3.4、predict.rs）は rule 入口の検査を行わないので、
    // 再生中に limit / 深さ上限が発火し得る状態では本物の候補関数へ戻す。
    #[inline]
    pub fn can_replay(&self, nesting: usize) -> bool {
        self.depth + nesting <= self.options.max_depth
            && !self.diag.exhausted
            && !self.display.exhausted
            && self.resource_failure.is_none()
            && self.depth_failure.is_none()
    }
    pub fn enter_rule(&mut self, state: State) -> Option<Step> {
        if self.depth >= self.options.max_depth {
            self.depth_failure
                .get_or_insert(state.consumed.max(state.matched));
        }
        if let Some(limit) = self.limit(state) {
            return Some(limit);
        }
        self.depth += 1;
        None
    }
    pub fn cp(&self, byte: usize) -> usize {
        self.input.byte_to_cp(byte).expect("parser UTF-8 boundary")
    }
    pub fn span(&self, span: Span) -> Span {
        [self.cp(span[0]), self.cp(span[1])]
    }
    fn trimmed(&self, span: Span) -> &str {
        let [start, end] = self.trimmed_range(span);
        &self.text[start..end]
    }
    /// `trimmed` の byte 範囲（AST の Text 節点は値を複製せずこの範囲で持つ）。
    fn trimmed_range(&self, span: Span) -> [usize; 2] {
        let trim = |c: char| {
            c.is_whitespace() && !matches!(c, '\u{85}' | '\u{a0}' | '\u{2007}' | '\u{202f}')
        };
        let text = &self.text[span[0]..span[1]];
        let head = text.trim_start_matches(trim);
        let start = span[0] + (text.len() - head.len());
        [start, start + head.trim_end_matches(trim).len()]
    }
    pub fn text(&self, span: Span) -> String {
        self.trimmed(span).to_owned()
    }
    pub fn mark(&self) -> Mark {
        Mark {
            scope: self.scope.checkpoint(),
            effects: self.effects.len(),
        }
    }
    pub fn restore(&mut self, mark: Mark) {
        self.scope.restore(mark.scope);
        self.effects.truncate(mark.effects);
    }
    pub fn save_effects(&mut self, mark: Mark) -> Span {
        let start = self.saved_effects.len();
        let count = self.effects.len() - mark.effects;
        let bytes = self.effects[mark.effects..]
            .iter()
            .map(Self::effect_size)
            .sum::<usize>();
        if start.saturating_add(count) > self.options.limits.effect_entries.min(u32::MAX as usize)
            || self.effect_bytes.saturating_add(bytes) > self.options.limits.effect_bytes
        {
            self.resource_failure
                .get_or_insert((0, "saved effect budget exceeded"));
            return [0, 0];
        }
        self.effect_bytes += bytes;
        self.saved_effects
            .extend_from_slice(&self.effects[mark.effects..]);
        [start, self.saved_effects.len()]
    }
    pub fn replay_effects(&mut self, span: Span) {
        for i in span[0]..span[1] {
            let effect = self.saved_effects[i].clone();
            self.effect_shared(
                effect.value,
                effect.mode,
                effect.offset_cp,
                effect.length_cp,
                effect.name,
            );
        }
    }
    fn effect_size(effect: &Effect) -> usize {
        match &*effect.value {
            ScanEffect::Declare { name, .. } | ScanEffect::Use { name, .. } => name.len(),
            ScanEffect::Diagnostic { message, .. } => message.len(),
            _ => 0,
        }
    }
    pub fn effect(&mut self, effect: ScanEffect) {
        self.effect_at(effect, "lexical", 0, 0);
    }
    pub fn effect_at(
        &mut self,
        effect: ScanEffect,
        mode: &'static str,
        offset_cp: usize,
        length_cp: usize,
    ) {
        self.effect_shared(std::rc::Rc::new(effect), mode, offset_cp, length_cp, None);
    }
    /// D-029: scope を開閉する効果は、その scope を持つ規則名を伴う。
    pub fn effect_scope_at(
        &mut self,
        effect: ScanEffect,
        mode: &'static str,
        offset_cp: usize,
        length_cp: usize,
        name: &'static str,
    ) {
        self.effect_shared(
            std::rc::Rc::new(effect),
            mode,
            offset_cp,
            length_cp,
            Some(name),
        );
    }
    fn effect_shared(
        &mut self,
        effect: std::rc::Rc<ScanEffect>,
        mode: &'static str,
        offset_cp: usize,
        length_cp: usize,
        name: Option<&'static str>,
    ) {
        // D-029: trim 後に空になった記号名は記号ではない。宣言も参照も記録しない
        // （Java の `Session.applyEffects` と同じ規則）。
        if matches!(
            &*effect,
            ScanEffect::Declare { name, .. } | ScanEffect::Use { name, .. }
if name.is_empty()
        ) {
            return;
        }
        if self.effect_work
            >= self
                .options
                .limits
                .effect_entries
                .min(u32::MAX as usize - 1)
        {
            self.resource_failure
                .get_or_insert((0, "effect budget exceeded"));
            return;
        }
        let size = match &*effect {
            ScanEffect::Declare { name, .. } | ScanEffect::Use { name, .. } => name.len(),
            ScanEffect::Diagnostic { message, .. } => message.len(),
            _ => 0,
        };
        if self.effect_bytes.saturating_add(size) > self.options.limits.effect_bytes {
            self.resource_failure
                .get_or_insert((0, "effect byte budget exceeded"));
            return;
        }
        self.effect_work += 1;
        self.effect_bytes += size;
        match &*effect {
            ScanEffect::Diagnostic {
                message,
                offset_cp,
                len_cp,
                severity,
            } => self
                .scope
                .add_diagnostic(message, *offset_cp, *len_cp, *severity),
            ScanEffect::Enter => self.scope.enter(),
            ScanEffect::Leave => self.scope.leave(),
            ScanEffect::Clear => self.scope.clear_diagnostics(),
            ScanEffect::Declare { name, offset_cp } => self.scope.declare(name, *offset_cp),
            ScanEffect::Use {
                name,
                offset_cp,
                len_cp,
            } => {
                self.scope.add_reference(name, *offset_cp, *len_cp);
                if !self.scope.is_declared(name) {
                    self.scope.add_diagnostic(
                        &format!("未定義のシンボル: '{name}'"),
                        *offset_cp,
                        *len_cp,
                        Severity::Warning,
                    );
                }
            }
        }
        self.effects.push(Effect {
            value: effect,
            mode,
            offset_cp,
            length_cp,
            name,
        });
    }
    pub fn lookup(&mut self, key: Key) -> Option<Step> {
        if !self.options.memo {
            return None;
        }
        let saved = self.memo.get(key)?;
        self.statistics.memo_hits += 1;
        self.display_merge(saved.display);
        self.replay_effects(saved.effects.map(|p| p as usize));
        Some(saved.step())
    }
    pub fn store(&mut self, key: Key, step: Step, mark: Mark) {
        if !self.options.memo || self.depth_failure.is_some() || self.resource_failure.is_some() {
            return;
        }
        if self.memo.len() >= self.options.limits.memo_entries.min(u32::MAX as usize - 1) {
            self.resource_failure
                .get_or_insert((step.state.consumed, "memo budget exceeded"));
            return;
        }
        let effects = if step.ok {
            self.save_effects(mark)
        } else {
            [0, 0]
        };
        let snapshot = self.display_snapshot();
        self.memo.insert(key, Saved::new(step, effects, snapshot));
    }
    #[inline(always)]
    pub fn event(&mut self, event: Event) -> EventId {
        // 認識だけの parse（scope / recovery の無い文法）は記録しない。判定だけを呼出し側へ inline する。
        if !self.options.build_ast
            && !self.options.ast_tree
            && !self.options.lexical
            && !HAS_SCOPE
            && !HAS_RECOVERY
        {
            return EventId(0);
        }
        self.record_event(PackedEvent::pack(event))
    }
    #[inline]
    fn record_event(&mut self, packed: PackedEvent) -> EventId {
        if self.arena.len() >= self.arena_limit.min(u32::MAX as usize) {
            self.resource_failure
                .get_or_insert((0, "arena budget exceeded"));
            return EventId(0);
        }
        let id = EventId(self.arena.len());
        self.arena.push(packed);
        id
    }
    /// event を読む（`PackedEvent` から戻した値）。
    #[inline(always)]
    pub fn ev(&self, id: EventId) -> Event {
        self.arena[id.0].unpack()
    }
    /// literal / token 参照の結果の Token event を、expr と規則を付け替えて複製する
    /// （`Event` に戻さず packed のまま写す。Token でなければ `id` をそのまま返す）。
    #[inline(always)]
    pub fn relabel_token(&mut self, id: EventId, expr: usize, rule: usize) -> EventId {
        // 認識だけの parse では event を記録しない（`id` は 0 = Empty）ので、判定だけを inline にする。
        if self.arena[id.0].tag != EV_TOKEN {
            return id;
        }
        self.relabel_token_copy(id, expr, rule)
    }
    #[inline(never)]
    fn relabel_token_copy(&mut self, id: EventId, expr: usize, rule: usize) -> EventId {
        let mut packed = self.arena[id.0];
        debug_assert!(rule < NO_RULE as usize);
        packed.rule = rule as u16;
        packed.w[6] = w32(expr);
        self.record_event(packed)
    }
    /// Rule event の直下 capture 表の区間（出現収集が書く）。
    #[inline]
    pub fn set_rule_caps(&mut self, id: EventId, caps: (u32, u32)) {
        let packed = &mut self.arena[id.0];
        if packed.tag == EV_RULE {
            packed.w[4] = caps.0;
            packed.w[5] = caps.1;
        }
    }
    /// event を書き換える（書き換える箇所は Token / Capture だけ）。
    #[inline(always)]
    pub fn set_ev(&mut self, id: EventId, event: Event) {
        self.arena[id.0] = PackedEvent::pack(event);
    }
    pub fn join(&mut self, a: EventId, b: EventId) -> EventId {
        if a.0 == 0 {
            return b;
        }
        if b.0 == 0 {
            return a;
        }
        self.event(Event::Join(a, b))
    }
    pub fn combine(&mut self, a: Step, b: Step) -> Step {
        Step {
            ok: b.ok,
            state: b.state,
            events: if b.ok {
                self.join(a.events, b.events)
            } else {
                EventId(0)
            },
            diag: self.diag_join(a.diag, b.diag),
        }
    }
    pub fn fail(&mut self, state: State, offset: usize, label: &'static str) -> Step {
        Step {
            ok: false,
            state,
            events: EventId(0),
            diag: self.diag_fail(offset, label),
        }
    }
    pub fn primitive<const MATCH: bool>(
        &mut self,
        mut state: State,
        length: Option<usize>,
        label: &'static str,
    ) -> Step {
        let start = state.position::<MATCH>();
        let length = if state.invert {
            if length.is_some() {
                None
            } else {
                self.input.cp_at(start).map(|(_, n)| n)
            }
        } else {
            length
        };
        match length {
            Some(length) => {
                state.advance::<MATCH>(length);
                let mut step = Step::yes(state);
                step.events = self.event(Event::Token {
                    text_span: None,
                    content_span: None,
                    expr: usize::MAX,
                    rule: usize::MAX,
                    span: [start, start + length],
                });
                step
            }
            None => {
                state.advance::<MATCH>(0);
                self.fail(state, start, label)
            }
        }
    }
    /// 統合 DFA の結果を位置ごとに引く。範囲外（起こらないはず）は None を返し、
    /// 呼出し側は走査へ戻る。`lex_fill` は (入力, 位置) だけの関数なので、どの parser 状態
    /// から引いても同じ答えになる。
    #[inline]
    pub fn lex_matches(&mut self, pos: usize, id: u32) -> Option<bool> {
        if self.lex.need_fill(pos)? {
            self.lex_run(pos);
        }
        Some(self.lex.test(pos, id))
    }
    /// この位置の統合 DFA を 1 回走らせて bitset を埋める（cache miss のときだけ）。
    #[inline]
    pub fn lex_run(&mut self, pos: usize) {
        let bytes = self.text.as_bytes();
        let out = self.lex.slot(pos);
        lex_fill(bytes, pos, out);
    }
    /// `boundary` は D-040 の語境界。末尾が識別子構成文字のリテラルにだけ front が
    /// 立てる。境界違反は「一致しなかった」と同じ失敗記録（同じ位置・同じ label）になる。
    pub fn literal<const MATCH: bool>(
        &mut self,
        state: State,
        word: &'static str,
        sensitive: bool,
        lit: u32,
        boundary: bool,
        label: &'static str,
    ) -> Step {
        let start = state.position::<MATCH>();
        let length = 'length: {
            if self.lex_enabled && lit != u32::MAX {
                if let Some(hit) = self.lex_matches(start, lit) {
                    break 'length hit.then_some(word.len());
                }
            }
            if sensitive {
                self.input.starts_with(start, word).then_some(word.len())
            } else {
                let mut p = start;
                let mut ok = true;
                for expected in word.chars() {
                    let Some((actual, n)) = self.input.cp_at(p) else {
                        ok = false;
                        break;
                    };
                    if !super::rt::input::equal_ignore_case(actual, expected) {
                        ok = false;
                        break;
                    }
                    p += n;
                }
                ok.then_some(p - start)
            }
        };
        let length = match length {
            Some(n) if boundary && self.input.identifier_byte_at(start + n) => None,
            other => other,
        };
        self.primitive::<MATCH>(state, length, label)
    }
    pub fn any<const MATCH: bool>(&mut self, state: State, label: &'static str) -> Step {
        self.primitive::<MATCH>(
            state,
            self.input.cp_at(state.position::<MATCH>()).map(|(_, n)| n),
            label,
        )
    }
    pub fn empty(&mut self, state: State) -> Step {
        let mut child = state.begin();
        if let Some((_, length)) = self.input.cp_at(child.matched) {
            child.matched += length;
        }
        Step::yes(state.commit(child))
    }
    pub fn eof<const MATCH: bool>(&mut self, state: State, label: &'static str) -> Step {
        if state.begin().position::<MATCH>() == self.text.len() {
            self.display_fail(state.consumed.max(state.begin().matched),"WildCardCharacterParser");
            Step::yes(state)
        } else {
            self.fail(state, state.position::<MATCH>(), label)
        }
    }
    pub fn number<const MATCH: bool>(&mut self, state: State, label: &'static str) -> Step {
        // NumberParser.java の失敗試行を表示用に記録する。Rust の主 fail_at は
        // 仮数なしの場合のみ進め、不完全指数のロールバック位置は維持する。
        let start = state.position::<MATCH>();
        let bytes = self.text.as_bytes();
        let mut p = start;
        if matches!(bytes.get(p), Some(b'+' | b'-')) { p += 1; }
        else { self.display_fail(p, "SignParser"); }
        let digits = p;
        while bytes.get(p).is_some_and(u8::is_ascii_digit) { p += 1; }
        self.display_fail(p, "DigitParser");
        let mut count = p - digits;
        if count == 0 { self.display_fail(p, "OneOrMore"); }
        if bytes.get(p) == Some(&b'.') {
            p += 1;
            let fraction = p;
            while bytes.get(p).is_some_and(u8::is_ascii_digit) { p += 1; }
            self.display_fail(p, "DigitParser");
            if p == fraction { self.display_fail(p, "OneOrMore"); }
            count += p - fraction;
            if count == 0 { self.display_fail(start,"Chain"); self.display_fail(start,"'.'"); }
        } else {
            self.display_fail(p, "'.'");
            if count == 0 { self.display_fail(digits,"Chain"); }
        }
        if count == 0 {
            self.display_fail(start,"NumberParser");
            let mut out = self.primitive::<MATCH>(state, None, label);
            if !out.ok { out.diag = self.diag_fail(p,label); }
            return out;
        }
        if matches!(bytes.get(p), Some(b'e' | b'E')) {
            let exponent = p;
            p += 1;
            if matches!(bytes.get(p), Some(b'+' | b'-')) { p += 1; }
            else { self.display_fail(p,"SignParser"); }
            let digits = p;
            while bytes.get(p).is_some_and(u8::is_ascii_digit) { p += 1; }
            self.display_fail(p,"DigitParser");
            if digits == p { self.display_fail(p,"OneOrMore"); p = exponent; }
        } else { self.display_fail(p,"EParser"); self.display_fail(p,"ExponentParser"); }
        self.primitive::<MATCH>(state, Some(p - start), label)
    }
    pub fn identifier<const MATCH: bool>(&mut self, state: State, label: &'static str) -> Step {
        let start = state.position::<MATCH>();
        let bytes = self.text.as_bytes();
        let mut p = start;
        if bytes
            .get(p)
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
        {
            p += 1;
            while bytes
                .get(p)
                .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
            {
                p += 1;
            }
        }
        if p == start { self.display_fail(start,"AlphabetUnderScoreParser"); self.display_fail(start,"IdentifierParser"); }
        else { self.display_fail(p,"AlphabetNumericUnderScoreParser"); }
        self.primitive::<MATCH>(state, (p > start).then_some(p - start), label)
    }
    pub fn quoted<const MATCH: bool>(
        &mut self,
        state: State,
        quote: char,
        label: &'static str,
    ) -> Step {
        let start = state.position::<MATCH>();
        let mut p = start;
        if self.input.cp_at(p).is_none_or(|(c, _)| c != quote) {
            return self.fail(state, start, label);
        }
        p += 1;
        while let Some((c, n)) = self.input.cp_at(p) {
            p += n;
            if c == quote {
                let step = self.primitive::<MATCH>(state, Some(p - start), label);
                let mut event = self.ev(step.events);
                if let Event::Token { text_span, content_span, .. } = &mut event {
                    *text_span = Some([start, p]);
                    *content_span = Some([start + 1, p - 1]);
                    self.set_ev(step.events, event);
                }
                return step;
            }
            if c == '\\' {
                if let Some((_, n)) = self.input.cp_at(p) {
                    p += n;
                } else {
                    break;
                }
            }
        }
        self.display_fail(p, if quote == '\'' {"SingleQuoted"} else {"Quoted"});
        self.fail(state, start, label)
    }
    pub fn until<const MATCH: bool>(&mut self, state: State, terminator: &'static str, display: &'static str) -> Step {
        let mut out = state.begin();
        loop {
            if !terminator.is_empty() && self.input.starts_with(out.matched, terminator) {
                if !MATCH {
                    out.matched += terminator.len();
                }
                return Step::yes(state.commit(out));
            }
            if !terminator.is_empty() { self.display_fail(out.consumed.max(out.matched),display); }
            let Some((_, n)) = self.input.cp_at(out.position::<MATCH>()) else {
                self.display_fail(out.consumed.max(out.matched),"ANY");
                return Step::yes(state.commit(out));
            };
            out.advance::<MATCH>(n);
        }
    }
    pub fn scan<const MATCH: bool>(
        &mut self,
        state: State,
        logical_id: &'static str,
        label: &'static str,
    ) -> Step {
        let mark = self.mark();
        let result = self
            .scanner
            .scan(logical_id, self.text, state, MATCH, &self.scope);
        let valid = result.consumed_end >= state.consumed
            && (!MATCH || result.consumed_end == state.consumed)
            && result.matched_end >= result.consumed_end
            && result.value_span[0] <= result.value_span[1]
            && [
                result.consumed_end,
                result.matched_end,
                result.value_span[0],
                result.value_span[1],
            ]
            .iter()
            .all(|&p| self.input.byte_to_cp(p).is_some())
            && result
                .diagnostics
                .iter()
                .all(|d| self.input.byte_to_cp(d.offset).is_some());
        if !valid {
            return self.fail(state, state.position::<MATCH>(), "valid Extern cursor/span");
        }
        let mut step = Step::yes(State {
            consumed: result.consumed_end,
            matched: result.matched_end,
            ..state
        });
        step.ok = result.ok;
        for d in result.diagnostics {
            self.display_fail(d.offset, d.expected);
            let diag = self.diag_fail(d.offset, d.expected);
            step.diag = self.diag_join(step.diag, diag);
        }
        if step.ok {
            for effect in result.effects {
                self.effect(effect);
            }
            step.events = self.event(Event::Token {
                expr: usize::MAX,
                rule: usize::MAX,
                span: result.value_span,
                text_span: Some(result.value_span),
                content_span: None,
            });
        } else {
            self.restore(mark);
            if step.diag.id == 0 {
                step.diag = self.diag_fail(state.position::<MATCH>(), label);
            }
        }
        step
    }
    /// 左因数分解（factor.rs）: 共有要素を先頭候補の expression 関数で評価した後、採用された
    /// 候補の capture site / token expr に付け替える。共有要素の wrapper は memo されないので
    /// この event は今回の結果からしか参照されない。
    pub fn retag(&mut self, id: EventId, site: Option<usize>, expr: Option<usize>) {
        if id.0 == 0 {
            return;
        }
        let mut target = id;
        let mut event = self.ev(id);
        if let Event::Capture {
            site: current,
            child,
            ..
        } = &mut event
        {
            if let Some(site) = site {
                *current = site;
                target = *child;
                self.set_ev(id, event);
            } else {
                target = *child;
            }
        }
        let mut event = self.ev(target);
        if let (Some(expr), Event::Token { expr: current, .. }) = (expr, &mut event) {
            *current = expr;
            self.set_ev(target, event);
        }
    }
    pub fn take_stack(&mut self) -> Vec<(EventId, bool)> {
        let mut stack = self.stacks.pop().unwrap_or_default();
        stack.clear();
        stack
    }
    pub fn give_stack(&mut self, stack: Vec<(EventId, bool)>) {
        self.stacks.push(stack);
    }
    /// rule の直下 capture 出現（完了順）を出現収集が並べた区間から写す。
    /// 以前は rule ごとに局所 event を再走査していた（`collect_captures`）。同じ木を 2 度
    /// 下るのをやめ、出現収集の 1 回の走査で rule ごとに切り出しておく。
    pub fn rule_captures(&mut self, caps: (u32, u32), out: &mut Vec<Cap>) {
        let (offset, len) = (caps.0 as usize, caps.1 as usize);
        out.extend_from_slice(&self.caps_flat[offset..offset + len]);
    }
    pub fn take_captures(&mut self) -> Vec<Cap> {
        let mut v = self.captures_pool.pop().unwrap_or_default();
        v.clear();
        v
    }
    pub fn give_captures(&mut self, v: Vec<Cap>) {
        self.captures_pool.push(v);
    }
    pub fn take_values(&mut self) -> Vec<u32> {
        let mut v = self.values_pool.pop().unwrap_or_default();
        v.clear();
        v
    }
    pub fn give_values(&mut self, v: Vec<u32>) {
        self.values_pool.push(v);
    }
    /// text 変換の field 値（recovery で直接置換された出現は除く）。
    pub fn text_values(&mut self, caps: &[Cap], sites: &[usize], out: &mut Vec<u32>) {
        for &(site, span, child) in caps {
            if sites.contains(&site) && !self.directly_recovered(child) {
                let text = self.semantic_text(child, span);
                out.push(text);
            }
        }
    }
    pub fn text_nodes(&mut self, caps: &[Cap], sites: &[usize], out: &mut Vec<u32>) {
        self.text_values(caps, sites, out);
    }
    /// node 変換の field 値。出現ごとに子の値を構築し、値が無く空でない出現は字句 Text にする。
    /// `text_sites` は中身が text だけの site で、出現 1 つが一致範囲の字句 1 つになる（#89、D-032）。
    /// `fallback` は leaf 型（Text を leaf node に昇格する recipe）。
    pub fn node_values(
        &mut self,
        caps: &[Cap],
        sites: &[usize],
        text_sites: &[usize],
        fallback: Option<usize>,
        out: &mut Vec<u32>,
    ) -> Result<(), String> {
        for &(site, span, child) in caps {
            if !sites.contains(&site) {
                continue;
            }
            let start = out.len();
            if text_sites.contains(&site) && span[0] != span[1] && !self.has_recovery(child) {
                let text = self.semantic_text(child, span);
                out.push(text);
            } else {
                self.build_values_into(child, out)?;
            }
            if out.len() == start
                && span[0] != span[1]
                && !self.has_recovery(child)
                && !self.has_value_group(child)
            {
                let text = self.semantic_text(child, span);
                out.push(text);
            }
            if let Some(ty) = fallback {
                let mut i = start;
                while i < out.len() {
                    if self.tree.kind(out[i]) == super::ast::tree::KIND_TEXT {
                        out[i] = self.leaf(ty, span, out[i])?;
                    }
                    i += 1;
                }
            }
        }
        Ok(())
    }
    /// field の出現のどれかが recovery を含む（値が欠けた node は作らず省略する）。
    pub fn missing_field(&mut self, caps: &[Cap], sites: &[usize]) -> bool {
        if !HAS_RECOVERY {
            return false;
        }
        for &(site, _, child) in caps {
            if sites.contains(&site) && self.has_recovery(child) {
                return true;
            }
        }
        false
    }
    pub fn selected_extent(&mut self, root: EventId, fallback: Span) -> Span {
        let mut stack = self.take_stack();
        stack.push((root, false));
        let mut span = None;
        while let Some((id, _)) = stack.pop() {
            match self.ev(id) {
                Event::Join(a, b) => {
                    stack.push((b, false));
                    stack.push((a, false));
                }
                Event::Capture { child, .. } | Event::Values { child, .. } => {
                    stack.push((child, false))
                }
                Event::Rule { span: child, .. } | Event::Token { span: child, .. } => {
                    span = Some(span.map_or(child, |s: Span| [s[0], child[1]]));
                }
                Event::Empty => {}
                Event::Recovery { span: child, .. } => {
                    span = Some(span.map_or(child, |s: Span| [s[0], child[1]]));
                }
            }
        }
        self.give_stack(stack);
        span.unwrap_or(fallback)
    }
    pub fn significant_end(&self, root: EventId) -> Option<usize> {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            match self.ev(id) {
                Event::Join(a, b) => {
                    stack.push(a);
                    stack.push(b);
                }
                Event::Rule { child, .. }
                | Event::Capture { child, .. }
                | Event::Values { child, .. } => stack.push(child),
                Event::Token { span, .. } => return Some(span[1]),
                _ => {}
            }
        }
        None
    }
    pub fn has_recovery(&mut self, root: EventId) -> bool {
        // recovery を持たない文法では Recovery event が無い（走査を省く）。
        if !HAS_RECOVERY {
            return false;
        }
        let mut stack = self.take_stack();
        stack.push((root, false));
        let mut found = false;
        while let Some((id, _)) = stack.pop() {
            match self.ev(id) {
                Event::Recovery { .. } => {
                    found = true;
                    break;
                }
                Event::Join(a, b) => {
                    stack.push((b, false));
                    stack.push((a, false));
                }
                Event::Capture { child, .. }
                | Event::Rule { child, .. }
                | Event::Values { child, .. } => stack.push((child, false)),
                _ => {}
            }
        }
        self.give_stack(stack);
        found
    }
    pub fn directly_recovered(&self, mut root: EventId) -> bool {
        if !HAS_RECOVERY {
            return false;
        }
        loop {
            match self.ev(root) {
                Event::Recovery { .. } => return true,
                Event::Capture { child, .. } | Event::Values { child, .. } => root = child,
                _ => return false,
            }
        }
    }
    /// design v1 §3.10。mode / 同期集合 / `consume_sync` / `no_sync` は IR が正本で、
    /// ここでは推測しない。SYNC だけが同期文字列を消費し、AUTO の同期文字列（FOLLOW）は
    /// 外側が待っている字句なので残す。SKIP は同期文字列の手前まで進む別規則。
    #[allow(clippy::too_many_arguments)]
    pub fn recover<const MATCH: bool>(
        &mut self,
        state: State,
        failed: Step,
        rule: usize,
        mode: &'static str,
        patterns: &[&str],
        consume_sync: bool,
        no_sync: &'static str,
    ) -> Step {
        if self.depth_failure.is_some() || self.resource_failure.is_some() || self.diag.exhausted {
            return failed;
        }
        let start = state.position::<MATCH>();
        let one = |s: &Self, at: usize| s.input.cp_at(at).map_or(0, |(_, n)| n);
        let eof = self.input.bytes().len();
        // §3.10: AUTO の FOLLOW が空なら既定の `;`。既定文字列は FOLLOW ではないので消費する。
        let fallback = mode == "AUTO" && patterns.is_empty();
        let patterns = if fallback { &[";"][..] } else { patterns };
        let consume_sync = consume_sync || fallback;
        let mut sync_span = None;
        let mut at = start;
        while !patterns.is_empty() {
            let Some((_, n)) = self.input.cp_at(at) else {
                break;
            };
            if let Some(sync) = patterns
                .iter()
                .find(|s| !s.is_empty() && self.input.starts_with(at, s))
            {
                sync_span = Some([at, at + sync.len()]);
                break;
            }
            at += n;
        }
        let end = if mode == "SKIP" {
            match sync_span {
                _ if self.input.cp_at(start).is_none() => return failed,
                None if patterns.is_empty() => start + one(self, start),
                None => eof,
                Some([found, _]) if found == start => start + one(self, start),
                Some([found, _]) => found,
            }
        } else {
            match sync_span {
                Some([found, stop]) => {
                    if consume_sync {
                        stop
                    } else {
                        found
                    }
                }
                None => match no_sync {
                    "consumeToEof" => eof,
                    "succeed" => start,
                    _ => return failed,
                },
            }
        };
        if end <= start {
            return failed;
        }
        let mut child = state.begin();
        child.advance::<MATCH>(end - start);
        let diag = self.diag_rule(rule, failed.diag);
        // D-027: 公開候補は表示語彙で、回復した規則の frame に属するものだけ
        // （生成側が回復つき規則の本体を `display_enter` / `display_leave` で囲う）。
        let hints = u32::try_from(self.recovery_hints.len()).unwrap_or(u32::MAX);
        self.recovery_hints
            .push(if DIAG { self.display.local_summary() } else { None });
        let detail = u32::try_from(self.recovery_events.len()).unwrap_or(u32::MAX);
        self.recovery_events.push(RecoveryEvent {
            rule,
            mode,
            sync_span,
            diag,
            hints,
        });
        let events = self.event(Event::Recovery {
            span: [start, end],
            detail,
        });
        Step {
            ok: true,
            state: state.commit(child),
            events,
            diag: failed.diag,
        }
    }
    pub fn has_value_group(&mut self, root: EventId) -> bool {
        let mut stack = self.take_stack();
        stack.push((root, false));
        let mut found = false;
        while let Some((id, _)) = stack.pop() {
            match self.ev(id) {
                Event::Values { .. } => {
                    found = true;
                    break;
                }
                Event::Join(a, b) => {
                    stack.push((b, false));
                    stack.push((a, false));
                }
                Event::Rule { child, .. } | Event::Capture { child, .. } => {
                    stack.push((child, false))
                }
                _ => {}
            }
        }
        self.give_stack(stack);
        found
    }
    pub fn has_token_text(&self, mut root: EventId) -> bool {
        loop {
            match self.ev(root) {
                Event::Capture { child, .. }
                | Event::Rule { child, .. }
                | Event::Values { child, .. } => root = child,
                Event::Token { text_span, .. } => return text_span.is_some(),
                _ => return false,
            }
        }
    }
    pub fn text_extent(&self, mut root: EventId, fallback: Span) -> Span {
        loop {
            match self.ev(root) {
                Event::Capture { child, .. }
                | Event::Rule { child, .. }
                | Event::Values { child, .. } => root = child,
                Event::Token {
                    text_span: Some(span),
                    ..
                } => return span,
                _ => return fallback,
            }
        }
    }
    fn text_content_extent(&self, mut root: EventId, fallback: Span) -> Span {
        loop {
            match self.ev(root) {
                Event::Capture { child, .. } | Event::Rule { child, .. } | Event::Values { child, .. } => root = child,
                Event::Token { content_span, text_span, .. } => return content_span.or(text_span).unwrap_or(fallback),
                _ => return fallback,
            }
        }
    }
    /// capture の値となる Text 節点を作る（D-077: 値は複製せず入力の byte 範囲で持つ）。
    pub fn semantic_text(&mut self, root: EventId, span: Span) -> u32 {
        // Token まで下って text_span / content_span を 1 回の走査で読む
        // （text_extent / text_content_extent / has_token_text と同じ規則）。
        let mut node = root;
        let token = loop {
            match self.ev(node) {
                Event::Capture { child, .. } | Event::Rule { child, .. } | Event::Values { child, .. } => node = child,
                Event::Token { text_span, content_span, .. } => break Some((text_span, content_span)),
                _ => break None,
            }
        };
        let (extent, content, token_text) = match token {
            Some((Some(text), content)) => (text, content.unwrap_or(text), true),
            Some((None, content)) => (span, content.unwrap_or(span), false),
            None => (span, span, false),
        };
        let value = if token_text { content } else { self.trimmed_range(content) };
        let extent = self.span(extent);
        self.tree.text(extent, value)
    }
    pub fn take_scope(&mut self) -> Vec<(usize, Span)> {
        let mut v = self.scope_pool.pop().unwrap_or_default();
        v.clear();
        v
    }
    pub fn give_scope(&mut self, v: Vec<(usize, Span)>) {
        self.scope_pool.push(v);
    }
    /// scope イベントの capture 出現（trim 済み span）を完了順に集める。
    pub fn scope_captures(&mut self, root: EventId, sites: &[usize], out: &mut Vec<(usize, Span)>) {
        let mut stack = self.take_stack();
        stack.push((root, false));
        while let Some((id, done)) = stack.pop() {
            match self.ev(id) {
                Event::Join(a, b) => {
                    stack.push((b, false));
                    stack.push((a, false));
                }
                Event::Values { child, .. } => stack.push((child, false)),
                Event::Capture {
                    site, span, child, ..
                } => {
                    if done {
                        if sites.contains(&site) {
                            let text = &self.text[span[0]..span[1]];
                            let leading =
                                text.len() - text.trim_start_matches(|c: char| c <= '\u{20}').len();
                            let len = text.trim_matches(|c: char| c <= '\u{20}').len();
                            out.push((site, [span[0] + leading, span[0] + leading + len]));
                        }
                    } else {
                        stack.push((id, true));
                        stack.push((child, false));
                    }
                }
                _ => {}
            }
        }
        self.give_stack(stack);
    }
}

pub mod lexical_program {
use std::collections::HashMap;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    LITERAL,
    ANY,
    EOF,
    BOF,
    BOL,
    EOL,
    RANGE,
    EXCEPT,
    SEQUENCE,
    CHOICE,
    REPEAT,
    LOOK,
    NOT,
    CAPTURE,
    BACKREF,
    SCOPE,
}

#[derive(Clone, Debug)]
pub struct LexicalExpression {
    pub op: Op,
    pub text: &'static str,
    pub min: i32,
    pub max: i32,
    pub children: &'static [LexicalExpression],
}

impl LexicalExpression {
    pub fn match_at(&self, source: &str, start: usize) -> Option<usize> {
        self.run(source, start, &mut HashMap::new())
    }
    fn run<'a>(
        &self,
        s: &'a str,
        p: usize,
        bindings: &mut HashMap<&'static str, &'a str>,
    ) -> Option<usize> {
        let before = bindings.clone();
        let end = self.eval(s, p, bindings);
        if end.is_none() {
            *bindings = before;
        }
        end
    }
    fn eval<'a>(
        &self,
        s: &'a str,
        p: usize,
        bindings: &mut HashMap<&'static str, &'a str>,
    ) -> Option<usize> {
        match self.op {
            Op::LITERAL => s[p..].starts_with(self.text).then_some(p + self.text.len()),
            Op::ANY => s[p..].chars().next().map(|c| p + c.len_utf8()),
            Op::EOF => (p == s.len()).then_some(p),
            Op::BOF => (p == 0).then_some(p),
            Op::BOL => {
                (p == 0 || matches!(s.as_bytes().get(p - 1), Some(b'\r' | b'\n'))).then_some(p)
            }
            Op::EOL => {
                (p == s.len() || matches!(s.as_bytes().get(p), Some(b'\r' | b'\n'))).then_some(p)
            }
            Op::RANGE => s[p..]
                .chars()
                .next()
                .filter(|c| (*c as i32) >= self.min && (*c as i32) <= self.max)
                .map(|c| p + c.len_utf8()),
            Op::EXCEPT => s[p..]
                .chars()
                .next()
                .filter(|c| !self.text.contains(*c))
                .map(|c| p + c.len_utf8()),
            Op::SEQUENCE => self
                .children
                .iter()
                .try_fold(p, |p, child| child.run(s, p, bindings)),
            Op::CHOICE => self
                .children
                .iter()
                .find_map(|child| child.run(s, p, bindings)),
            Op::REPEAT => {
                let mut end = p;
                let mut count = 0;
                while self.max < 0 || count < self.max {
                    let Some(next) = self.children[0].run(s, end, bindings) else {
                        break;
                    };
                    count += 1;
                    assert!(next != end || self.max >= 0, "nullable lexical repeat");
                    end = next;
                }
                (count >= self.min).then_some(end)
            }
            Op::LOOK | Op::NOT => {
                let matched = self.children[0].run(s, p, &mut bindings.clone()).is_some();
                (matched == (self.op == Op::LOOK)).then_some(p)
            }
            Op::CAPTURE => {
                let end = self.children[0].run(s, p, bindings)?;
                bindings.insert(self.text, &s[p..end]);
                Some(end)
            }
            Op::BACKREF => bindings
                .get(self.text)
                .filter(|text| s[p..].starts_with(**text))
                .map(|text| p + text.len()),
            Op::SCOPE => self.children[0].run(s, p, &mut HashMap::new()),
        }
    }
}


}
/// 統合 DFA の bitset 語数（literal 66 個）。0 なら cache を作らない。
const LEX_WORDS:usize=2;
/// 診断を記録しない読み飛ばし cache を持つ trivia policy 数（区切り集合で正規化済み）。
const TRIVIA_POLICIES:usize=1;
/// 全 literal 終端を 1 本にまとめた DFA（trie）。位置 `pos` から 1 回走らせ、
/// 一致した literal の id を `out` の bitset に立てる。長さは id ごとに静的。
fn lex_fill(bytes:&[u8],pos:usize,out:&mut [u64]) {
let mut p=pos;let mut state=0usize;
loop {
let Some(&c)=bytes.get(p) else {return;};
state=match state {
0=>match c {
37=>{out[0]|=1;1},
40=>{out[0]|=2;2},
41=>{out[0]|=4;3},
42=>{out[0]|=8;4},
43=>{out[0]|=16;5},
44=>{out[0]|=32;6},
46=>{out[0]|=64;7},
58=>{out[0]|=128;8},
59=>{out[0]|=512;11},
61=>{out[0]|=1024;12},
63=>{out[0]|=2048;13},
64=>{out[0]|=4096;14},
65=>170,
67=>179,
69=>196,
76=>207,
78=>216,
82=>236,
83=>241,
85=>248,
91=>{out[0]|=281474976710656;253},
93=>{out[0]|=562949953421312;254},
99=>255,
100=>262,
102=>273,
103=>277,
108=>284,
109=>289,
110=>293,
112=>297,
115=>309,
116=>317,
118=>322,
123=>{out[0]|=9223372036854775808;329},
124=>{out[1]|=1;330},
125=>{out[1]|=2;331},
_=>return,
},
8=>match c {
58=>9,
_=>return,
},
9=>match c {
61=>{out[0]|=256;10},
_=>return,
},
14=>match c {
98=>15,
99=>22,
100=>39,
101=>49,
105=>56,
108=>71,
109=>92,
112=>99,
114=>122,
115=>142,
116=>154,
119=>160,
_=>return,
},
15=>match c {
97=>16,
_=>return,
},
16=>match c {
99=>17,
_=>return,
},
17=>match c {
107=>18,
_=>return,
},
18=>match c {
114=>19,
_=>return,
},
19=>match c {
101=>20,
_=>return,
},
20=>match c {
102=>{out[0]|=8192;21},
_=>return,
},
22=>match c {
97=>23,
111=>29,
_=>return,
},
23=>match c {
116=>24,
_=>return,
},
24=>match c {
97=>25,
_=>return,
},
25=>match c {
108=>26,
_=>return,
},
26=>match c {
111=>27,
_=>return,
},
27=>match c {
103=>{out[0]|=16384;28},
_=>return,
},
29=>match c {
109=>30,
_=>return,
},
30=>match c {
109=>31,
_=>return,
},
31=>match c {
111=>32,
_=>return,
},
32=>match c {
110=>33,
_=>return,
},
33=>match c {
70=>34,
_=>return,
},
34=>match c {
105=>35,
_=>return,
},
35=>match c {
101=>36,
_=>return,
},
36=>match c {
108=>37,
_=>return,
},
37=>match c {
100=>{out[0]|=32768;38},
_=>return,
},
39=>match c {
101=>40,
111=>47,
_=>return,
},
40=>match c {
99=>41,
_=>return,
},
41=>match c {
108=>42,
_=>return,
},
42=>match c {
97=>43,
_=>return,
},
43=>match c {
114=>44,
_=>return,
},
44=>match c {
101=>45,
_=>return,
},
45=>match c {
115=>{out[0]|=65536;46},
_=>return,
},
47=>match c {
99=>{out[0]|=131072;48},
_=>return,
},
49=>match c {
110=>50,
118=>53,
_=>return,
},
50=>match c {
117=>51,
_=>return,
},
51=>match c {
109=>{out[0]|=262144;52},
_=>return,
},
53=>match c {
97=>54,
_=>return,
},
54=>match c {
108=>{out[0]|=524288;55},
_=>return,
},
56=>match c {
109=>57,
110=>62,
_=>return,
},
57=>match c {
112=>58,
_=>return,
},
58=>match c {
111=>59,
_=>return,
},
59=>match c {
114=>60,
_=>return,
},
60=>match c {
116=>{out[0]|=1048576;61},
_=>return,
},
62=>match c {
116=>63,
_=>return,
},
63=>match c {
101=>64,
_=>return,
},
64=>match c {
114=>65,
_=>return,
},
65=>match c {
108=>66,
_=>return,
},
66=>match c {
101=>67,
_=>return,
},
67=>match c {
97=>68,
_=>return,
},
68=>match c {
118=>69,
_=>return,
},
69=>match c {
101=>{out[0]|=2097152;70},
_=>return,
},
71=>match c {
101=>72,
111=>80,
_=>return,
},
72=>match c {
102=>73,
_=>return,
},
73=>match c {
116=>74,
_=>return,
},
74=>match c {
65=>75,
_=>return,
},
75=>match c {
115=>76,
_=>return,
},
76=>match c {
115=>77,
_=>return,
},
77=>match c {
111=>78,
_=>return,
},
78=>match c {
99=>{out[0]|=4194304;79},
_=>return,
},
80=>match c {
110=>81,
_=>return,
},
81=>match c {
103=>82,
_=>return,
},
82=>match c {
101=>83,
_=>return,
},
83=>match c {
115=>84,
_=>return,
},
84=>match c {
116=>85,
_=>return,
},
85=>match c {
67=>86,
_=>return,
},
86=>match c {
104=>87,
_=>return,
},
87=>match c {
111=>88,
_=>return,
},
88=>match c {
105=>89,
_=>return,
},
89=>match c {
99=>90,
_=>return,
},
90=>match c {
101=>{out[0]|=8388608;91},
_=>return,
},
92=>match c {
97=>93,
_=>return,
},
93=>match c {
112=>94,
_=>return,
},
94=>match c {
112=>95,
_=>return,
},
95=>match c {
105=>96,
_=>return,
},
96=>match c {
110=>97,
_=>return,
},
97=>match c {
103=>{out[0]|=16777216;98},
_=>return,
},
99=>match c {
114=>100,
_=>return,
},
100=>match c {
101=>101,
_=>return,
},
101=>match c {
99=>102,
100=>109,
_=>return,
},
102=>match c {
101=>103,
_=>return,
},
103=>match c {
100=>104,
_=>return,
},
104=>match c {
101=>105,
_=>return,
},
105=>match c {
110=>106,
_=>return,
},
106=>match c {
99=>107,
_=>return,
},
107=>match c {
101=>{out[0]|=33554432;108},
_=>return,
},
109=>match c {
105=>110,
_=>return,
},
110=>match c {
99=>111,
_=>return,
},
111=>match c {
116=>112,
_=>return,
},
112=>match c {
105=>113,
_=>return,
},
113=>match c {
118=>114,
_=>return,
},
114=>match c {
101=>115,
_=>return,
},
115=>match c {
67=>116,
_=>return,
},
116=>match c {
104=>117,
_=>return,
},
117=>match c {
111=>118,
_=>return,
},
118=>match c {
105=>119,
_=>return,
},
119=>match c {
99=>120,
_=>return,
},
120=>match c {
101=>{out[0]|=67108864;121},
_=>return,
},
122=>match c {
101=>123,
105=>130,
111=>139,
_=>return,
},
123=>match c {
99=>124,
_=>return,
},
124=>match c {
111=>125,
_=>return,
},
125=>match c {
118=>126,
_=>return,
},
126=>match c {
101=>127,
_=>return,
},
127=>match c {
114=>128,
_=>return,
},
128=>match c {
121=>{out[0]|=134217728;129},
_=>return,
},
130=>match c {
103=>131,
_=>return,
},
131=>match c {
104=>132,
_=>return,
},
132=>match c {
116=>133,
_=>return,
},
133=>match c {
65=>134,
_=>return,
},
134=>match c {
115=>135,
_=>return,
},
135=>match c {
115=>136,
_=>return,
},
136=>match c {
111=>137,
_=>return,
},
137=>match c {
99=>{out[0]|=268435456;138},
_=>return,
},
139=>match c {
111=>140,
_=>return,
},
140=>match c {
116=>{out[0]|=536870912;141},
_=>return,
},
142=>match c {
99=>143,
107=>151,
_=>return,
},
143=>match c {
111=>144,
_=>return,
},
144=>match c {
112=>145,
_=>return,
},
145=>match c {
101=>146,
_=>return,
},
146=>match c {
84=>147,
_=>return,
},
147=>match c {
114=>148,
_=>return,
},
148=>match c {
101=>149,
_=>return,
},
149=>match c {
101=>{out[0]|=1073741824;150},
_=>return,
},
151=>match c {
105=>152,
_=>return,
},
152=>match c {
112=>{out[0]|=2147483648;153},
_=>return,
},
154=>match c {
121=>155,
_=>return,
},
155=>match c {
112=>156,
_=>return,
},
156=>match c {
101=>157,
_=>return,
},
157=>match c {
111=>158,
_=>return,
},
158=>match c {
102=>{out[0]|=4294967296;159},
_=>return,
},
160=>match c {
104=>161,
_=>return,
},
161=>match c {
105=>162,
_=>return,
},
162=>match c {
116=>163,
_=>return,
},
163=>match c {
101=>164,
_=>return,
},
164=>match c {
115=>165,
_=>return,
},
165=>match c {
112=>166,
_=>return,
},
166=>match c {
97=>167,
_=>return,
},
167=>match c {
99=>168,
_=>return,
},
168=>match c {
101=>{out[0]|=8589934592;169},
_=>return,
},
170=>match c {
68=>171,
78=>177,
_=>return,
},
171=>match c {
65=>172,
_=>return,
},
172=>match c {
80=>173,
_=>return,
},
173=>match c {
84=>174,
_=>return,
},
174=>match c {
69=>175,
_=>return,
},
175=>match c {
82=>{out[0]|=17179869184;176},
_=>return,
},
177=>match c {
89=>{out[0]|=34359738368;178},
_=>return,
},
179=>match c {
65=>180,
72=>186,
73=>{out[0]|=274877906944;195},
_=>return,
},
180=>match c {
80=>181,
_=>return,
},
181=>match c {
84=>182,
_=>return,
},
182=>match c {
85=>183,
_=>return,
},
183=>match c {
82=>184,
_=>return,
},
184=>match c {
69=>{out[0]|=68719476736;185},
_=>return,
},
186=>match c {
65=>187,
_=>return,
},
187=>match c {
82=>188,
_=>return,
},
188=>match c {
95=>189,
_=>return,
},
189=>match c {
82=>190,
_=>return,
},
190=>match c {
65=>191,
_=>return,
},
191=>match c {
78=>192,
_=>return,
},
192=>match c {
71=>193,
_=>return,
},
193=>match c {
69=>{out[0]|=137438953472;194},
_=>return,
},
196=>match c {
77=>197,
79=>201,
82=>203,
_=>return,
},
197=>match c {
80=>198,
_=>return,
},
198=>match c {
84=>199,
_=>return,
},
199=>match c {
89=>{out[0]|=549755813888;200},
_=>return,
},
201=>match c {
70=>{out[0]|=1099511627776;202},
_=>return,
},
203=>match c {
82=>204,
_=>return,
},
204=>match c {
79=>205,
_=>return,
},
205=>match c {
82=>{out[0]|=2199023255552;206},
_=>return,
},
207=>match c {
79=>208,
_=>return,
},
208=>match c {
79=>209,
_=>return,
},
209=>match c {
75=>210,
_=>return,
},
210=>match c {
65=>211,
_=>return,
},
211=>match c {
72=>212,
_=>return,
},
212=>match c {
69=>213,
_=>return,
},
213=>match c {
65=>214,
_=>return,
},
214=>match c {
68=>{out[0]|=4398046511104;215},
_=>return,
},
216=>match c {
69=>217,
_=>return,
},
217=>match c {
71=>218,
_=>return,
},
218=>match c {
65=>219,
_=>return,
},
219=>match c {
84=>220,
_=>return,
},
220=>match c {
73=>221,
_=>return,
},
221=>match c {
79=>222,
86=>224,
_=>return,
},
222=>match c {
78=>{out[0]|=8796093022208;223},
_=>return,
},
224=>match c {
69=>225,
_=>return,
},
225=>match c {
95=>226,
_=>return,
},
226=>match c {
76=>227,
_=>return,
},
227=>match c {
79=>228,
_=>return,
},
228=>match c {
79=>229,
_=>return,
},
229=>match c {
75=>230,
_=>return,
},
230=>match c {
65=>231,
_=>return,
},
231=>match c {
72=>232,
_=>return,
},
232=>match c {
69=>233,
_=>return,
},
233=>match c {
65=>234,
_=>return,
},
234=>match c {
68=>{out[0]|=17592186044416;235},
_=>return,
},
236=>match c {
69=>237,
_=>return,
},
237=>match c {
71=>238,
_=>return,
},
238=>match c {
69=>239,
_=>return,
},
239=>match c {
88=>{out[0]|=35184372088832;240},
_=>return,
},
241=>match c {
65=>242,
_=>return,
},
242=>match c {
77=>243,
_=>return,
},
243=>match c {
69=>244,
_=>return,
},
244=>match c {
95=>245,
_=>return,
},
245=>match c {
65=>246,
_=>return,
},
246=>match c {
83=>{out[0]|=70368744177664;247},
_=>return,
},
248=>match c {
78=>249,
_=>return,
},
249=>match c {
84=>250,
_=>return,
},
250=>match c {
73=>251,
_=>return,
},
251=>match c {
76=>{out[0]|=140737488355328;252},
_=>return,
},
255=>match c {
111=>256,
_=>return,
},
256=>match c {
110=>257,
_=>return,
},
257=>match c {
116=>258,
_=>return,
},
258=>match c {
101=>259,
_=>return,
},
259=>match c {
120=>260,
_=>return,
},
260=>match c {
116=>{out[0]|=1125899906842624;261},
_=>return,
},
262=>match c {
101=>263,
_=>return,
},
263=>match c {
115=>264,
_=>return,
},
264=>match c {
99=>265,
_=>return,
},
265=>match c {
114=>266,
_=>return,
},
266=>match c {
105=>267,
_=>return,
},
267=>match c {
112=>268,
_=>return,
},
268=>match c {
116=>269,
_=>return,
},
269=>match c {
105=>270,
_=>return,
},
270=>match c {
111=>271,
_=>return,
},
271=>match c {
110=>{out[0]|=2251799813685248;272},
_=>return,
},
273=>match c {
114=>274,
_=>return,
},
274=>match c {
111=>275,
_=>return,
},
275=>match c {
109=>{out[0]|=4503599627370496;276},
_=>return,
},
277=>match c {
114=>278,
_=>return,
},
278=>match c {
97=>279,
_=>return,
},
279=>match c {
109=>280,
_=>return,
},
280=>match c {
109=>281,
_=>return,
},
281=>match c {
97=>282,
_=>return,
},
282=>match c {
114=>{out[0]|=9007199254740992;283},
_=>return,
},
284=>match c {
101=>285,
_=>return,
},
285=>match c {
118=>286,
_=>return,
},
286=>match c {
101=>287,
_=>return,
},
287=>match c {
108=>{out[0]|=18014398509481984;288},
_=>return,
},
289=>match c {
111=>290,
_=>return,
},
290=>match c {
100=>291,
_=>return,
},
291=>match c {
101=>{out[0]|=36028797018963968;292},
_=>return,
},
293=>match c {
97=>294,
_=>return,
},
294=>match c {
109=>295,
_=>return,
},
295=>match c {
101=>{out[0]|=72057594037927936;296},
_=>return,
},
297=>match c {
97=>298,
114=>303,
_=>return,
},
298=>match c {
114=>299,
_=>return,
},
299=>match c {
97=>300,
_=>return,
},
300=>match c {
109=>301,
_=>return,
},
301=>match c {
115=>{out[0]|=144115188075855872;302},
_=>return,
},
303=>match c {
111=>304,
_=>return,
},
304=>match c {
102=>305,
_=>return,
},
305=>match c {
105=>306,
_=>return,
},
306=>match c {
108=>307,
_=>return,
},
307=>match c {
101=>{out[0]|=288230376151711744;308},
_=>return,
},
309=>match c {
121=>310,
_=>return,
},
310=>match c {
109=>311,
110=>315,
_=>return,
},
311=>match c {
98=>312,
_=>return,
},
312=>match c {
111=>313,
_=>return,
},
313=>match c {
108=>{out[0]|=576460752303423488;314},
_=>return,
},
315=>match c {
99=>{out[0]|=1152921504606846976;316},
_=>return,
},
317=>match c {
111=>318,
_=>return,
},
318=>match c {
107=>319,
_=>return,
},
319=>match c {
101=>320,
_=>return,
},
320=>match c {
110=>{out[0]|=2305843009213693952;321},
_=>return,
},
322=>match c {
101=>323,
_=>return,
},
323=>match c {
114=>324,
_=>return,
},
324=>match c {
115=>325,
_=>return,
},
325=>match c {
105=>326,
_=>return,
},
326=>match c {
111=>327,
_=>return,
},
327=>match c {
110=>{out[0]|=4611686018427387904;328},
_=>return,
},
_=>return,
};
p+=1;
}
}

use super::ast::*;
const HAS_RECOVERY:bool=false;
const SCOPE_MODES:&[&str]=&["sameRuleText"];
const HAS_SCOPE:bool=true;
const RULES:&[&str]=&["UBNF::UBNFFile", "UBNF::ImportDecl", "UBNF::GrammarDecl", "UBNF::GlobalSetting", "UBNF::SettingValue", "UBNF::StringSettingValue", "UBNF::BlockSettingValue", "UBNF::KeyValuePair", "UBNF::TokenDecl", "UBNF::DeclarativeTokenValue", "UBNF::LexicalBody", "UBNF::LexicalSequence", "UBNF::LexicalElement", "UBNF::LexicalSuffix", "UBNF::LexicalAtom", "UBNF::TokenValue", "UBNF::UntilExpression", "UBNF::NegationExpression", "UBNF::LookaheadExpression", "UBNF::NegativeLookaheadExpression", "UBNF::CharRangeExpression", "UBNF::CIExpression", "UBNF::RegexExpression", "UBNF::AdapterExpression", "UBNF::AnyExpression", "UBNF::EofExpression", "UBNF::EmptyExpression", "UBNF::RuleDecl", "UBNF::Annotation", "UBNF::RootAnnotation", "UBNF::MappingAnnotation", "UBNF::EvalAnnotation", "UBNF::WhitespaceAnnotation", "UBNF::InterleaveAnnotation", "UBNF::BackrefAnnotation", "UBNF::ScopeTreeAnnotation", "UBNF::DeclaresAnnotation", "UBNF::CatalogAnnotation", "UBNF::LeftAssocAnnotation", "UBNF::RightAssocAnnotation", "UBNF::LongestChoiceAnnotation", "UBNF::PredictiveChoiceAnnotation", "UBNF::PrecedenceAnnotation", "UBNF::DocAnnotation", "UBNF::RecoveryAnnotation", "UBNF::SkipAnnotation", "UBNF::CommonFieldAnnotation", "UBNF::EnumAnnotation", "UBNF::SimpleAnnotation", "UBNF::ChoiceBody", "UBNF::SequenceBody", "UBNF::AnnotatedElement", "UBNF::PostfixQuantifier", "UBNF::SeparatedBy", "UBNF::BoundedQuantifier", "UBNF::Digits", "UBNF::AtomicElement", "UBNF::GroupElement", "UBNF::OptionalElement", "UBNF::RepeatElement", "UBNF::ErrorElement", "UBNF::TerminalElement", "UBNF::QuantifiedRef", "UBNF::TypeofElement", "UBNF::DottedIdentifier", "UBNF::RuleBody"];
const EXPRESSIONS:&[&str]=&["expr:unlaxer-dsl/grammar/ubnf.ubnf:1350:1365:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1350:1365:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1352:1363:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1523:1564:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1523:1532:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1533:1543:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1551:1557:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1558:1564:body/3/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1704:1882:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1704:1713:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1714:1724:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1736:1739:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1747:1761:body/3/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1749:1759:body/3/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1781:1798:body/4/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1783:1796:body/4/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1816:1829:body/5/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1818:1827:body/5/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1849:1861:body/6/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1851:1859:body/6/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1879:1882:body/7/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:2018:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1985:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1986:1996:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2002:2005:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2006:2018:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2057:2100:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2057:2075:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2083:2100:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2179:2195:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2179:2195:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2281:2314:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2281:2284:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2285:2301:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2287:2299:body/1/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2311:2314:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2412:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2396:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2402:2405:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2406:2412:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2554:2645:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2554:2561:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2562:2572:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2583:2645:body/2/group", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2585:2636:body/2/0/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2585:2599:body/2/0/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2585:2588:body/2/0/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2589:2599:body/2/0/0/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2609:2636:body/2/0/1/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2609:2614:body/2/0/1/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2615:2636:body/2/0/1/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2728:2751:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2728:2739:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2748:2751:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2888:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2836:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2851:2888:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2853:2872:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2853:2856:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2857:2872:body/1/0/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2960:2975:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2960:2975:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2960:2974:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3059:3102:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3059:3070:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3077:3102:body/1/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3079:3092:body/1/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3166:3245:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3166:3245:body/0/group", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3168:3243:body/0/0/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3168:3171:body/0/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3174:3177:body/0/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3180:3183:body/0/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3190:3243:body/0/0/3/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3190:3193:body/0/0/3/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3194:3210:body/0/0/3/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3211:3239:body/0/0/3/2/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3213:3237:body/0/0/3/2/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3213:3216:body/0/0/3/2/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3217:3237:body/0/0/3/2/0/1/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3219:3235:body/0/0/3/2/0/1/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3240:3243:body/0/0/3/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3316:3655:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3316:3655:body/0/group", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3318:3653:body/0/0/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3318:3324:body/0/0/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3331:3350:body/0/0/1/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3331:3334:body/0/0/1/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3335:3346:body/0/0/1/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3347:3350:body/0/0/1/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3357:3376:body/0/0/2/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3357:3360:body/0/0/2/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3361:3372:body/0/0/2/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3373:3376:body/0/0/2/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3383:3402:body/0/0/3/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3383:3386:body/0/0/3/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3387:3398:body/0/0/3/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3399:3402:body/0/0/3/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3409:3467:body/0/0/4/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3409:3447:body/0/0/4/0/group", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3411:3445:body/0/0/4/0/0/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3411:3422:body/0/0/4/0/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3425:3445:body/0/0/4/0/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3448:3451:body/0/0/4/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3452:3463:body/0/0/4/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3464:3467:body/0/0/4/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3474:3518:body/0/0/5/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3474:3483:body/0/0/5/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3484:3487:body/0/0/5/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3488:3498:body/0/0/5/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3499:3502:body/0/0/5/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3503:3514:body/0/0/5/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3515:3518:body/0/0/5/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3525:3553:body/0/0/6/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3525:3534:body/0/0/6/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3535:3538:body/0/0/6/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3539:3549:body/0/0/6/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3550:3553:body/0/0/6/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3560:3598:body/0/0/7/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3560:3572:body/0/0/7/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3573:3576:body/0/0/7/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3577:3583:body/0/0/7/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3584:3587:body/0/0/7/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3588:3594:body/0/0/7/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3595:3598:body/0/0/7/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3605:3630:body/0/0/8/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3605:3615:body/0/0/8/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3616:3619:body/0/0/8/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3620:3626:body/0/0/8/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3627:3630:body/0/0/8/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3637:3653:body/0/0/9/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3713:4000:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3713:3728:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3736:3754:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3762:3781:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3789:3816:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3824:3843:body/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3851:3863:body/5/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3871:3886:body/6/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3894:3911:body/7/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3919:3932:body/8/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3940:3953:body/9/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3961:3976:body/10/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3984:4000:body/11/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4104:4138:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4104:4111:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4112:4115:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4116:4122:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4135:4138:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4241:4273:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4241:4251:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4252:4255:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4256:4262:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4270:4273:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4383:4418:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4383:4394:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4395:4398:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4399:4405:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4415:4418:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4553:4597:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4553:4573:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4574:4577:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4578:4584:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4594:4597:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4707:4755:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4707:4719:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4720:4723:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4724:4730:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4736:4739:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4740:4746:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4752:4755:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4841:4866:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4841:4845:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4846:4849:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4850:4856:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4863:4866:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4964:4995:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4964:4971:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4972:4975:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4976:4982:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4992:4995:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5124:5196:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5124:5133:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5134:5137:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5138:5144:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5149:5152:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5153:5162:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5163:5166:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5167:5183:body/6/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5193:5196:body/7/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5259:5264:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5259:5264:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5327:5332:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5327:5332:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5401:5408:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5401:5408:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5599:5683:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5599:5613:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5601:5611:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5632:5642:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5654:5659:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5660:5668:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5680:5683:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5739:6251:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5739:5753:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5761:5778:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5786:5800:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5808:5828:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5836:5856:body/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5864:5881:body/5/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5889:5908:body/6/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5916:5934:body/7/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5942:5959:body/8/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5967:5986:body/9/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5994:6014:body/10/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6022:6045:body/11/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6053:6079:body/12/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6087:6107:body/13/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6115:6128:body/14/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6136:6154:body/15/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6162:6176:body/16/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6184:6205:body/17/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6213:6227:body/18/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6235:6251:body/19/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6306:6313:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6306:6313:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6411:6553:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6411:6421:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6422:6425:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6433:6443:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6462:6544:body/3/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6464:6542:body/3/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6464:6467:body/3/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6468:6476:body/3/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6477:6480:body/3/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6481:6484:body/3/0/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6485:6495:body/3/0/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6508:6538:body/3/0/5/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6510:6524:body/3/0/5/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6510:6513:body/3/0/5/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6514:6524:body/3/0/5/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6539:6542:body/3/0/6/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6550:6553:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6626:6693:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6626:6633:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6634:6637:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6638:6650:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6660:6689:body/3/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6662:6678:body/3/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6662:6665:body/3/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6666:6678:body/3/0/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6690:6693:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6776:6819:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6776:6789:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6790:6819:body/1/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6792:6817:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6792:6795:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6796:6806:body/1/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6814:6817:body/1/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6904:6959:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6904:6917:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6918:6921:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6922:6931:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6932:6935:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6936:6946:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6956:6959:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7035:7081:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7035:7045:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7046:7049:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7050:7056:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7057:7060:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7061:7071:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7078:7081:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7161:7209:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7161:7173:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7174:7177:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7178:7184:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7185:7188:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7189:7199:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7206:7209:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7302:7399:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7302:7313:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7314:7317:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7318:7326:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7327:7330:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7331:7341:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7350:7395:body/5/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7352:7380:body/5/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7352:7355:body/5/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7356:7369:body/5/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7370:7373:body/5/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7374:7380:body/5/0/3/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7396:7399:body/6/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7478:7526:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7478:7488:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7489:7492:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7493:7502:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7503:7506:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7507:7513:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7523:7526:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7591:7603:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7591:7603:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7670:7683:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7670:7683:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7756:7772:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7756:7772:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7851:7870:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7851:7870:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7953:8010:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7953:7966:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7967:7970:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7971:7978:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7979:7982:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7983:7999:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8007:8010:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8078:8105:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8078:8084:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8085:8088:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8089:8095:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8102:8105:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8195:8267:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8195:8206:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8207:8210:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8211:8263:body/2/group", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8213:8255:body/2/0/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8213:8230:body/2/0/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8213:8219:body/2/0/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8220:8223:body/2/0/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8224:8230:body/2/0/0/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8245:8255:body/2/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8264:8267:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8322:8329:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8322:8329:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8492:8568:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8492:8506:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8507:8510:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8511:8521:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8534:8564:body/3/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8536:8550:body/3/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8536:8539:body/3/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8540:8550:body/3/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8565:8568:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8676:8683:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8676:8683:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8757:8771:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8757:8760:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8761:8771:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8891:8952:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8891:8903:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8918:8952:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8920:8936:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8920:8923:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8924:8936:body/1/0/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9038:9058:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9038:9058:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9040:9056:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9222:9407:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9222:9257:body/0/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9224:9237:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9265:9278:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9295:9316:body/2/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9297:9314:body/2/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9324:9345:body/3/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9326:9343:body/3/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9353:9368:body/4/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9355:9366:body/4/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9376:9407:body/5/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9378:9392:body/5/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9378:9381:body/5/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9382:9392:body/5/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9456:9471:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9456:9459:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9462:9465:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9468:9471:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9520:9537:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9520:9523:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9524:9537:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9595:9628:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9595:9598:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9599:9605:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9606:9624:body/2/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9608:9622:body/2/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9608:9611:body/2/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9612:9622:body/2/0/1/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9614:9620:body/2/0/1/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9625:9628:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9647:9662:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9647:9652:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9653:9662:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9655:9660:body/1/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9809:9929:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9809:9821:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9829:9844:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9852:9865:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9873:9885:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9893:9908:body/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9916:9929:body/5/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10019:10041:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10019:10022:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10023:10031:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10038:10041:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10131:10153:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10131:10134:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10135:10143:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10150:10153:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10244:10266:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10244:10247:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10248:10256:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10263:10266:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10366:10397:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10366:10373:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10374:10377:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10378:10384:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10394:10397:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10492:10498:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10492:10498:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10835:10875:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10835:10853:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10837:10851:body/0/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10837:10847:body/0/0/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10848:10851:body/0/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10865:10875:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11002:11043:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11002:11011:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11012:11015:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11016:11026:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11040:11043:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11147:11190:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11147:11157:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11165:11190:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11167:11181:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11167:11170:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11171:11181:body/1/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11248:11258:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11248:11258:body/0/ruleRef"];
const BODIES:&[&str]=&["expr:unlaxer-dsl/grammar/ubnf.ubnf:1350:1365:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1523:1564:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1704:1882:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:2018:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2057:2100:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2179:2195:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2281:2314:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2412:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2554:2645:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2728:2751:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2888:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2960:2975:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3059:3102:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3166:3245:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3316:3655:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3713:4000:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4104:4138:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4241:4273:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4383:4418:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4553:4597:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4707:4755:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4841:4866:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4964:4995:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5124:5196:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5259:5264:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5327:5332:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5401:5408:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5599:5683:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5739:6251:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6306:6313:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6411:6553:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6626:6693:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6776:6819:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6904:6959:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7035:7081:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7161:7209:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7302:7399:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7478:7526:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7591:7603:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7670:7683:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7756:7772:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7851:7870:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7953:8010:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8078:8105:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8195:8267:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8322:8329:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8492:8568:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8676:8683:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8757:8771:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8891:8952:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9038:9058:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9222:9407:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9456:9471:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9520:9537:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9595:9628:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9647:9662:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9809:9929:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10019:10041:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10131:10153:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10244:10266:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10366:10397:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10492:10498:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:10835:10875:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11002:11043:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11147:11190:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:11248:11258:body/seq"];
const RULE_NODE:&[bool]=&[true, true, true, true, false, true, true, true, true, true, true, true, true, true, true, false, true, true, true, true, true, true, true, true, true, true, true, true, false, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, false, false, false, false, false, true, true, true, true, true, false, true, true, false];
const RULE_SLOTS:&[u32]=&[2, 2, 9, 2, 0, 1, 2, 2, 2, 1, 2, 2, 2, 1, 1, 0, 1, 1, 1, 1, 2, 1, 1, 2, 0, 0, 0, 4, 0, 0, 3, 2, 1, 1, 1, 1, 2, 1, 0, 0, 0, 0, 1, 1, 2, 0, 2, 0, 1, 2, 2, 3, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 2, 0];
const SITES:&[(&str,&str)]=&[("expr:unlaxer-dsl/grammar/ubnf.ubnf:1350:1365:body/0/repeat/capture/0", "grammars"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1533:1543:body/1/tokenRef/capture/0", "alias"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1558:1564:body/3/tokenRef/capture/0", "path"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1714:1724:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1747:1761:body/3/repeat/capture/0", "imports"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1781:1798:body/4/repeat/capture/0", "settings"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1816:1829:body/5/repeat/capture/0", "tokens"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1849:1861:body/6/repeat/capture/0", "rules"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1986:1996:body/1/tokenRef/capture/0", "key"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2006:2018:body/3/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2179:2195:body/0/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2285:2301:body/1/repeat/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2396:body/0/tokenRef/capture/0", "key"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2406:2412:body/2/tokenRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2562:2572:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2589:2599:body/2/0/0/1/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2615:2636:body/2/0/1/1/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2728:2739:body/0/ruleRef/capture/0", "source"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2836:body/0/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2857:2872:body/1/0/1/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2960:2975:body/0/repeat/capture/0", "elements"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3059:3070:body/0/ruleRef/capture/0", "atom"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3079:3092:body/1/0/ruleRef/capture/0", "suffix"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3166:3245:body/0/group/capture/0", "text"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3316:3655:body/0/group/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4116:4122:body/2/tokenRef/capture/0", "terminator"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4256:4262:body/2/tokenRef/capture/0", "chars"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4399:4405:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4578:4584:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4724:4730:body/2/tokenRef/capture/0", "min"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4740:4746:body/4/tokenRef/capture/0", "max"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4850:4856:body/2/tokenRef/capture/0", "word"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4976:4982:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5138:5144:body/2/tokenRef/capture/0", "id"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5167:5183:body/6/tokenRef/capture/0", "version"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5599:5613:body/0/repeat/capture/0", "annotations"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5632:5642:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5660:5668:body/3/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6433:6443:body/2/tokenRef/capture/0", "className"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6485:6495:body/3/0/4/tokenRef/capture/0", "paramNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6514:6524:body/3/0/5/0/1/tokenRef/capture/0", "paramNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6638:6650:body/2/ruleRef/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6666:6678:body/3/0/1/ruleRef/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6796:6806:body/1/0/1/tokenRef/capture/0", "style"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6936:6946:body/4/tokenRef/capture/0", "profile"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7061:7071:body/4/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7189:7199:body/4/tokenRef/capture/0", "mode"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7331:7341:body/4/tokenRef/capture/0", "symbol"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7374:7380:body/5/0/3/tokenRef/capture/0", "description"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7507:7513:body/4/tokenRef/capture/0", "context"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7983:7999:body/4/tokenRef/capture/0", "level"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8089:8095:body/2/tokenRef/capture/0", "text"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8224:8230:body/2/0/0/2/tokenRef/capture/0", "syncTokens"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8245:8255:body/2/0/1/tokenRef/capture/0", "mode"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8511:8521:body/2/tokenRef/capture/0", "fieldNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8540:8550:body/3/0/1/tokenRef/capture/0", "fieldNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8761:8771:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8891:8903:body/0/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8924:8936:body/1/0/1/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9038:9058:body/0/repeat/capture/0", "elements"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9224:9237:body/0/0/ruleRef/capture/0", "typeofConstraint"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9265:9278:body/1/ruleRef/capture/0", "element"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9382:9392:body/5/0/1/tokenRef/capture/0", "captureName"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:10023:10031:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:10135:10143:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:10248:10256:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:10378:10384:body/2/tokenRef/capture/0", "message"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:10492:10498:body/0/tokenRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:10835:10853:body/0/repeat/capture/0", "namespace"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:10865:10875:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:11016:11026:body/2/tokenRef/capture/0", "captureName"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:11147:11157:body/0/tokenRef/capture/0", "parts"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:11171:11181:body/1/0/1/tokenRef/capture/0", "parts")];
impl<const DIAG: bool> Session<'_, DIAG> {
fn build_values(&mut self,root:EventId)->Result<Vec<u32>,String> {let mut out=Vec::new();self.build_values_into(root,&mut out)?;Ok(out)}
#[inline(never)] fn build_values_into(&mut self,root:EventId,out:&mut Vec<u32>)->Result<(),String> {let frame=0u8;let address=std::ptr::addr_of!(frame) as usize;let anchor=*self.mapping_anchor.get_or_insert(address);if self.mapping_depth>=self.options.limits.mapping_depth || anchor.abs_diff(address)>256*1024 {return Err("maximum mapping depth exceeded".into());}self.mapping_depth+=1;let result=self.build_values_inner(root,out);self.mapping_depth-=1;result}
fn build_values_inner(&mut self,mut root:EventId,out:&mut Vec<u32>)->Result<(),String> {
// 大半の呼出しは Join を含まない 1 本の枝（P4 実測: complex-x64 で 10,883 回すべてが 1 event）。
// その場合は pool から stack を借りずに降りる。
loop {match self.ev(root) {
Event::Capture{child,..}=>root=child,
Event::Values{child,span,wrap,text}=>{let start=out.len();if !text {self.value_region+=1;}let built=self.build_values_into(child,out);if !text {self.value_region-=1;}built?;if text || wrap && ((out.len()==start+1 && self.tree.kind(out[start])==tree::KIND_TEXT) || (out.len()==start && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {let text=self.semantic_text(child,span);out.truncate(start);out.push(text);}return Ok(());},
Event::Rule{rule,span,child,caps}=>return self.build_rule(rule,caps,span,child,out),
Event::Join(..)=>break,
_=>return Ok(()),}}
let mut stack=self.take_stack();stack.push((root,false));while let Some((id,_))=stack.pop() {match self.ev(id) {Event::Join(a,b)=>{stack.push((b,false));stack.push((a,false));},Event::Capture{child,..}=>stack.push((child,false)),Event::Values{child,span,wrap,text}=>{let start=out.len();if !text {self.value_region+=1;}let built=self.build_values_into(child,out);if !text {self.value_region-=1;}built?;if text || wrap && ((out.len()==start+1 && self.tree.kind(out[start])==tree::KIND_TEXT) || (out.len()==start && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {let text=self.semantic_text(child,span);out.truncate(start);out.push(text);}},Event::Rule{rule,span,child,caps}=>self.build_rule(rule,caps,span,child,out)?,_=>{}}}self.give_stack(stack);Ok(())}
fn leaf(&mut self,ty:usize,span:Span,text:u32)->Result<u32,String> {let span=self.span(span);self.tree.set_extent(text,span);let _=(span,text);Err(format!("unknown leaf type {ty}"))}
fn build_rule(&mut self,rule:usize,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);match rule {
0=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_0(caps,span,child,out);self.value_region=outer;built},
1=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_1(caps,span,child,out);self.value_region=outer;built},
2=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_2(caps,span,child,out);self.value_region=outer;built},
3=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_3(caps,span,child,out);self.value_region=outer;built},
4=>self.map_rule_4(caps,span,child,out),
5=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_5(caps,span,child,out);self.value_region=outer;built},
6=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_6(caps,span,child,out);self.value_region=outer;built},
7=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_7(caps,span,child,out);self.value_region=outer;built},
8=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_8(caps,span,child,out);self.value_region=outer;built},
9=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_9(caps,span,child,out);self.value_region=outer;built},
10=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_10(caps,span,child,out);self.value_region=outer;built},
11=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_11(caps,span,child,out);self.value_region=outer;built},
12=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_12(caps,span,child,out);self.value_region=outer;built},
13=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_13(caps,span,child,out);self.value_region=outer;built},
14=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_14(caps,span,child,out);self.value_region=outer;built},
15=>self.map_rule_15(caps,span,child,out),
16=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_16(caps,span,child,out);self.value_region=outer;built},
17=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_17(caps,span,child,out);self.value_region=outer;built},
18=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_18(caps,span,child,out);self.value_region=outer;built},
19=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_19(caps,span,child,out);self.value_region=outer;built},
20=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_20(caps,span,child,out);self.value_region=outer;built},
21=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_21(caps,span,child,out);self.value_region=outer;built},
22=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_22(caps,span,child,out);self.value_region=outer;built},
23=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_23(caps,span,child,out);self.value_region=outer;built},
24=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_24(caps,span,child,out);self.value_region=outer;built},
25=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_25(caps,span,child,out);self.value_region=outer;built},
26=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_26(caps,span,child,out);self.value_region=outer;built},
27=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_27(caps,span,child,out);self.value_region=outer;built},
28=>self.map_rule_28(caps,span,child,out),
29=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_29(caps,span,child,out);self.value_region=outer;built},
30=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_30(caps,span,child,out);self.value_region=outer;built},
31=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_31(caps,span,child,out);self.value_region=outer;built},
32=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_32(caps,span,child,out);self.value_region=outer;built},
33=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_33(caps,span,child,out);self.value_region=outer;built},
34=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_34(caps,span,child,out);self.value_region=outer;built},
35=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_35(caps,span,child,out);self.value_region=outer;built},
36=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_36(caps,span,child,out);self.value_region=outer;built},
37=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_37(caps,span,child,out);self.value_region=outer;built},
38=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_38(caps,span,child,out);self.value_region=outer;built},
39=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_39(caps,span,child,out);self.value_region=outer;built},
40=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_40(caps,span,child,out);self.value_region=outer;built},
41=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_41(caps,span,child,out);self.value_region=outer;built},
42=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_42(caps,span,child,out);self.value_region=outer;built},
43=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_43(caps,span,child,out);self.value_region=outer;built},
44=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_44(caps,span,child,out);self.value_region=outer;built},
45=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_45(caps,span,child,out);self.value_region=outer;built},
46=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_46(caps,span,child,out);self.value_region=outer;built},
47=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_47(caps,span,child,out);self.value_region=outer;built},
48=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_48(caps,span,child,out);self.value_region=outer;built},
49=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_49(caps,span,child,out);self.value_region=outer;built},
50=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_50(caps,span,child,out);self.value_region=outer;built},
51=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_51(caps,span,child,out);self.value_region=outer;built},
52=>self.map_rule_52(caps,span,child,out),
53=>self.map_rule_53(caps,span,child,out),
54=>self.map_rule_54(caps,span,child,out),
55=>self.map_rule_55(caps,span,child,out),
56=>self.map_rule_56(caps,span,child,out),
57=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_57(caps,span,child,out);self.value_region=outer;built},
58=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_58(caps,span,child,out);self.value_region=outer;built},
59=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_59(caps,span,child,out);self.value_region=outer;built},
60=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_60(caps,span,child,out);self.value_region=outer;built},
61=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_61(caps,span,child,out);self.value_region=outer;built},
62=>self.map_rule_62(caps,span,child,out),
63=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_63(caps,span,child,out);self.value_region=outer;built},
64=>{let outer=std::mem::take(&mut self.value_region);let built=self.map_rule_64(caps,span,child,out);self.value_region=outer;built},
65=>self.map_rule_65(caps,span,child,out),
_=>Err(format!("unknown rule {rule}"))}}
fn map_rule_0(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.node_values(&caps,&[0],&[],None,&mut f0)?;
if !f0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_GrammarDecl}) {return Err("grammars: mapped value type mismatch".into());}
let f0:Vec<u32>=f0;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_UBNFFile,0,span,&[l0[0],l0[1]]);out.push(node);Ok(())}}
fn map_rule_1(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[1],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[1]) {return Ok(());}
if t0.len()!=1 {return Err("alias requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
let mut t1=self.take_values();self.text_values(&caps,&[2],&mut t1);if t1.is_empty() && self.missing_field(&caps,&[2]) {return Ok(());}
if t1.len()!=1 {return Err("path requires one value".into());}let f1=t1.pop().unwrap();self.give_values(t1);
let f1:u32=f1;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_ImportDecl,1,span,&[f0,f1]);out.push(node);Ok(())}}
fn map_rule_2(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[3],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[3]) {return Ok(());}
if t0.len()!=1 {return Err("name requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
let mut f1=self.take_values();self.node_values(&caps,&[4],&[],None,&mut f1)?;
if !f1.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_ImportDecl}) {return Err("imports: mapped value type mismatch".into());}
let f1:Vec<u32>=f1;
let mut f2=self.take_values();self.node_values(&caps,&[5],&[],None,&mut f2)?;
if !f2.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_GlobalSetting}) {return Err("settings: mapped value type mismatch".into());}
let f2:Vec<u32>=f2;
let mut f3=self.take_values();self.node_values(&caps,&[6],&[],None,&mut f3)?;
if !f3.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_TokenDecl}) {return Err("tokens: mapped value type mismatch".into());}
let f3:Vec<u32>=f3;
let mut f4=self.take_values();self.node_values(&caps,&[7],&[],None,&mut f4)?;
if !f4.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_RuleDecl}) {return Err("rules: mapped value type mismatch".into());}
let f4:Vec<u32>=f4;
self.give_captures(caps);let span=self.span(span);let l1=self.tree.list(&f1);self.give_values(f1);let l2=self.tree.list(&f2);self.give_values(f2);let l3=self.tree.list(&f3);self.give_values(f3);let l4=self.tree.list(&f4);self.give_values(f4);let node=self.tree.record(tree::K_g_UBNFAST_2e_GrammarDecl,2,span,&[f0,l1[0],l1[1],l2[0],l2[1],l3[0],l3[1],l4[0],l4[1]]);out.push(node);Ok(())}}
fn map_rule_3(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[8],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[8]) {return Ok(());}
if t0.len()!=1 {return Err("key requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
let mut v1=self.take_values();self.node_values(&caps,&[9],&[],None,&mut v1)?;
if !v1.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_BlockSettingValue || k==tree::K_g_UBNFAST_2e_StringSettingValue}) {return Err("value: mapped value type mismatch".into());}
if v1.is_empty() && self.missing_field(&caps,&[9]) {return Ok(());}
if v1.len()!=1 {return Err("value requires one node".into());}let f1=v1.pop().unwrap();self.give_values(v1);
let f1:u32=f1;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_GlobalSetting,3,span,&[f0,f1]);out.push(node);Ok(())}}
fn map_rule_4(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
fn map_rule_5(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[10],&[],None,&mut v0)?;
if !v0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_DottedIdentifier}) {return Err("value: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[10]) {return Ok(());}
if v0.len()!=1 {return Err("value requires one node".into());}let f0=v0.pop().unwrap();self.give_values(v0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_StringSettingValue,5,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_6(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.node_values(&caps,&[11],&[],None,&mut f0)?;
if !f0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_KeyValuePair}) {return Err("entries: mapped value type mismatch".into());}
let f0:Vec<u32>=f0;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_BlockSettingValue,6,span,&[l0[0],l0[1]]);out.push(node);Ok(())}}
fn map_rule_7(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[12],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[12]) {return Ok(());}
if t0.len()!=1 {return Err("key requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
let mut t1=self.take_values();self.text_values(&caps,&[13],&mut t1);if t1.is_empty() && self.missing_field(&caps,&[13]) {return Ok(());}
if t1.len()!=1 {return Err("value requires one value".into());}let f1=t1.pop().unwrap();self.give_values(t1);
let f1:u32=f1;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_KeyValuePair,7,span,&[f0,f1]);out.push(node);Ok(())}}
fn map_rule_8(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[14],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[14]) {return Ok(());}
if t0.len()!=1 {return Err("name requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
let mut v1=self.take_values();self.node_values(&caps,&[15, 16],&[],None,&mut v1)?;
if !v1.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_AdapterExpression || k==tree::K_g_UBNFAST_2e_AnyExpression || k==tree::K_g_UBNFAST_2e_CIExpression || k==tree::K_g_UBNFAST_2e_CharRangeExpression || k==tree::K_g_UBNFAST_2e_DeclarativeTokenValue || k==tree::K_g_UBNFAST_2e_DottedIdentifier || k==tree::K_g_UBNFAST_2e_EmptyExpression || k==tree::K_g_UBNFAST_2e_EofExpression || k==tree::K_g_UBNFAST_2e_LookaheadExpression || k==tree::K_g_UBNFAST_2e_NegationExpression || k==tree::K_g_UBNFAST_2e_NegativeLookaheadExpression || k==tree::K_g_UBNFAST_2e_RegexExpression || k==tree::K_g_UBNFAST_2e_UntilExpression}) {return Err("value: mapped value type mismatch".into());}
if v1.is_empty() && self.missing_field(&caps,&[15, 16]) {return Ok(());}
if v1.len()!=1 {return Err("value requires one node".into());}let f1=v1.pop().unwrap();self.give_values(v1);
let f1:u32=f1;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_TokenDecl,8,span,&[f0,f1]);out.push(node);Ok(())}}
fn map_rule_9(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[17],&[],None,&mut v0)?;
if !v0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_LexicalChoice}) {return Err("source: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[17]) {return Ok(());}
if v0.len()!=1 {return Err("source requires one node".into());}let f0=v0.pop().unwrap();self.give_values(v0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_DeclarativeTokenValue,9,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_10(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.node_values(&caps,&[18, 19],&[],None,&mut f0)?;
if !f0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_LexicalSequence}) {return Err("alternatives: mapped value type mismatch".into());}
let f0:Vec<u32>=f0;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_LexicalChoice,10,span,&[l0[0],l0[1]]);out.push(node);Ok(())}}
fn map_rule_11(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.node_values(&caps,&[20],&[],None,&mut f0)?;
if !f0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_LexicalElement}) {return Err("elements: mapped value type mismatch".into());}
let f0:Vec<u32>=f0;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_LexicalSequence,11,span,&[l0[0],l0[1]]);out.push(node);Ok(())}}
fn map_rule_12(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[21],&[],None,&mut v0)?;
if !v0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_LexicalAtom}) {return Err("atom: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[21]) {return Ok(());}
if v0.len()!=1 {return Err("atom requires one node".into());}let f0=v0.pop().unwrap();self.give_values(v0);
let f0:u32=f0;
let mut v1=self.take_values();self.node_values(&caps,&[22],&[],None,&mut v1)?;
if !v1.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_LexicalSuffix}) {return Err("suffix: mapped value type mismatch".into());}
if v1.len()>1 {return Err("suffix requires at most one node".into());}let f1=v1.pop();self.give_values(v1);
let f1:Option<u32>=f1;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_LexicalElement,12,span,&[f0,f1.unwrap_or(tree::NONE)]);out.push(node);Ok(())}}
fn map_rule_13(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[23],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[23]) {return Ok(());}
if t0.len()!=1 {return Err("text requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_LexicalSuffix,13,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_14(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[24],&[],None,&mut v0)?;
if !v0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::KIND_TEXT || k==tree::K_g_UBNFAST_2e_DottedIdentifier || k==tree::K_g_UBNFAST_2e_LexicalChoice}) {return Err("value: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[24]) {return Ok(());}
if v0.len()!=1 {return Err("value requires one node".into());}let f0=v0.pop().unwrap();self.give_values(v0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_LexicalAtom,14,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_15(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
fn map_rule_16(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[25],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[25]) {return Ok(());}
if t0.len()!=1 {return Err("terminator requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_UntilExpression,16,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_17(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[26],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[26]) {return Ok(());}
if t0.len()!=1 {return Err("chars requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_NegationExpression,17,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_18(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[27],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[27]) {return Ok(());}
if t0.len()!=1 {return Err("pattern requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_LookaheadExpression,18,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_19(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[28],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[28]) {return Ok(());}
if t0.len()!=1 {return Err("pattern requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_NegativeLookaheadExpression,19,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_20(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[29],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[29]) {return Ok(());}
if t0.len()!=1 {return Err("min requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
let mut t1=self.take_values();self.text_values(&caps,&[30],&mut t1);if t1.is_empty() && self.missing_field(&caps,&[30]) {return Ok(());}
if t1.len()!=1 {return Err("max requires one value".into());}let f1=t1.pop().unwrap();self.give_values(t1);
let f1:u32=f1;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_CharRangeExpression,20,span,&[f0,f1]);out.push(node);Ok(())}}
fn map_rule_21(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[31],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[31]) {return Ok(());}
if t0.len()!=1 {return Err("word requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_CIExpression,21,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_22(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[32],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[32]) {return Ok(());}
if t0.len()!=1 {return Err("pattern requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_RegexExpression,22,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_23(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[33],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[33]) {return Ok(());}
if t0.len()!=1 {return Err("id requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
let mut t1=self.take_values();self.text_values(&caps,&[34],&mut t1);if t1.is_empty() && self.missing_field(&caps,&[34]) {return Ok(());}
if t1.len()!=1 {return Err("version requires one value".into());}let f1=t1.pop().unwrap();self.give_values(t1);
let f1:u32=f1;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_AdapterExpression,23,span,&[f0,f1]);out.push(node);Ok(())}}
fn map_rule_24(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_AnyExpression,24,span,&[]);out.push(node);Ok(())}}
fn map_rule_25(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_EofExpression,25,span,&[]);out.push(node);Ok(())}}
fn map_rule_26(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_EmptyExpression,26,span,&[]);out.push(node);Ok(())}}
fn map_rule_27(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.node_values(&caps,&[35],&[],None,&mut f0)?;
if !f0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_BackrefAnnotation || k==tree::K_g_UBNFAST_2e_CatalogAnnotation || k==tree::K_g_UBNFAST_2e_CommonFieldAnnotation || k==tree::K_g_UBNFAST_2e_DeclaresAnnotation || k==tree::K_g_UBNFAST_2e_DocAnnotation || k==tree::K_g_UBNFAST_2e_EnumAnnotation || k==tree::K_g_UBNFAST_2e_EvalAnnotation || k==tree::K_g_UBNFAST_2e_InterleaveAnnotation || k==tree::K_g_UBNFAST_2e_LeftAssocAnnotation || k==tree::K_g_UBNFAST_2e_LongestChoiceAnnotation || k==tree::K_g_UBNFAST_2e_MappingAnnotation || k==tree::K_g_UBNFAST_2e_PrecedenceAnnotation || k==tree::K_g_UBNFAST_2e_PredictiveChoiceAnnotation || k==tree::K_g_UBNFAST_2e_RecoveryAnnotation || k==tree::K_g_UBNFAST_2e_RightAssocAnnotation || k==tree::K_g_UBNFAST_2e_RootAnnotation || k==tree::K_g_UBNFAST_2e_ScopeTreeAnnotation || k==tree::K_g_UBNFAST_2e_SimpleAnnotation || k==tree::K_g_UBNFAST_2e_SkipAnnotation || k==tree::K_g_UBNFAST_2e_WhitespaceAnnotation}) {return Err("annotations: mapped value type mismatch".into());}
let f0:Vec<u32>=f0;
let mut t1=self.take_values();self.text_values(&caps,&[36],&mut t1);if t1.is_empty() && self.missing_field(&caps,&[36]) {return Ok(());}
if t1.len()!=1 {return Err("name requires one value".into());}let f1=t1.pop().unwrap();self.give_values(t1);
let f1:u32=f1;
let mut v2=self.take_values();self.node_values(&caps,&[37],&[],None,&mut v2)?;
if !v2.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_ChoiceBody}) {return Err("body: mapped value type mismatch".into());}
if v2.is_empty() && self.missing_field(&caps,&[37]) {return Ok(());}
if v2.len()!=1 {return Err("body requires one node".into());}let f2=v2.pop().unwrap();self.give_values(v2);
let f2:u32=f2;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_RuleDecl,27,span,&[l0[0],l0[1],f1,f2]);out.push(node);Ok(())}}
fn map_rule_28(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
fn map_rule_29(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_RootAnnotation,29,span,&[]);out.push(node);Ok(())}}
fn map_rule_30(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[38],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[38]) {return Ok(());}
if t0.len()!=1 {return Err("className requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
let mut f1=self.take_values();self.text_values(&caps,&[39, 40],&mut f1);
let f1:Vec<u32>=f1;
self.give_captures(caps);let span=self.span(span);let l1=self.tree.list(&f1);self.give_values(f1);let node=self.tree.record(tree::K_g_UBNFAST_2e_MappingAnnotation,30,span,&[f0,l1[0],l1[1]]);out.push(node);Ok(())}}
fn map_rule_31(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.node_values(&caps,&[41, 42],&[],None,&mut f0)?;
if !f0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_KeyValuePair}) {return Err("entries: mapped value type mismatch".into());}
let f0:Vec<u32>=f0;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_EvalAnnotation,31,span,&[l0[0],l0[1]]);out.push(node);Ok(())}}
fn map_rule_32(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[43],&mut t0);if t0.len()>1 {return Err("style requires at most one value".into());}let f0=t0.pop();self.give_values(t0);
let f0:Option<u32>=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_WhitespaceAnnotation,32,span,&[f0.unwrap_or(tree::NONE)]);out.push(node);Ok(())}}
fn map_rule_33(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[44],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[44]) {return Ok(());}
if t0.len()!=1 {return Err("profile requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_InterleaveAnnotation,33,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_34(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[45],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[45]) {return Ok(());}
if t0.len()!=1 {return Err("name requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_BackrefAnnotation,34,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_35(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[46],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[46]) {return Ok(());}
if t0.len()!=1 {return Err("mode requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_ScopeTreeAnnotation,35,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_36(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[47],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[47]) {return Ok(());}
if t0.len()!=1 {return Err("symbol requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
let mut t1=self.take_values();self.text_values(&caps,&[48],&mut t1);if t1.len()>1 {return Err("description requires at most one value".into());}let f1=t1.pop();self.give_values(t1);
let f1:Option<u32>=f1;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_DeclaresAnnotation,36,span,&[f0,f1.unwrap_or(tree::NONE)]);out.push(node);Ok(())}}
fn map_rule_37(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[49],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[49]) {return Ok(());}
if t0.len()!=1 {return Err("context requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_CatalogAnnotation,37,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_38(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_LeftAssocAnnotation,38,span,&[]);out.push(node);Ok(())}}
fn map_rule_39(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_RightAssocAnnotation,39,span,&[]);out.push(node);Ok(())}}
fn map_rule_40(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_LongestChoiceAnnotation,40,span,&[]);out.push(node);Ok(())}}
fn map_rule_41(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_PredictiveChoiceAnnotation,41,span,&[]);out.push(node);Ok(())}}
fn map_rule_42(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[50],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[50]) {return Ok(());}
if t0.len()!=1 {return Err("level requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_PrecedenceAnnotation,42,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_43(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[51],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[51]) {return Ok(());}
if t0.len()!=1 {return Err("text requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_DocAnnotation,43,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_44(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[53],&mut t0);if t0.len()>1 {return Err("mode requires at most one value".into());}let f0=t0.pop();self.give_values(t0);
let f0:Option<u32>=f0;
let mut t1=self.take_values();self.text_values(&caps,&[52],&mut t1);if t1.len()>1 {return Err("syncTokens requires at most one value".into());}let f1=t1.pop();self.give_values(t1);
let f1:Option<u32>=f1;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_RecoveryAnnotation,44,span,&[f0.unwrap_or(tree::NONE),f1.unwrap_or(tree::NONE)]);out.push(node);Ok(())}}
fn map_rule_45(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_SkipAnnotation,45,span,&[]);out.push(node);Ok(())}}
fn map_rule_46(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.text_values(&caps,&[54, 55],&mut f0);
let f0:Vec<u32>=f0;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_CommonFieldAnnotation,46,span,&[l0[0],l0[1]]);out.push(node);Ok(())}}
fn map_rule_47(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_EnumAnnotation,47,span,&[]);out.push(node);Ok(())}}
fn map_rule_48(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[56],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[56]) {return Ok(());}
if t0.len()!=1 {return Err("name requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_SimpleAnnotation,48,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_49(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.node_values(&caps,&[57, 58],&[],None,&mut f0)?;
if !f0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_SequenceBody}) {return Err("alternatives: mapped value type mismatch".into());}
let f0:Vec<u32>=f0;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_ChoiceBody,49,span,&[l0[0],l0[1]]);out.push(node);Ok(())}}
fn map_rule_50(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.node_values(&caps,&[59],&[],None,&mut f0)?;
if !f0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_AnnotatedElement}) {return Err("elements: mapped value type mismatch".into());}
let f0:Vec<u32>=f0;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_SequenceBody,50,span,&[l0[0],l0[1]]);out.push(node);Ok(())}}
fn map_rule_51(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[61],&[],None,&mut v0)?;
if !v0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::KIND_TEXT || k==tree::K_g_UBNFAST_2e_ErrorElement || k==tree::K_g_UBNFAST_2e_GroupElement || k==tree::K_g_UBNFAST_2e_OptionalElement || k==tree::K_g_UBNFAST_2e_RepeatElement || k==tree::K_g_UBNFAST_2e_TerminalElement}) {return Err("element: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[61]) {return Ok(());}
if v0.len()!=1 {return Err("element requires one node".into());}let f0=v0.pop().unwrap();self.give_values(v0);
let f0:u32=f0;
let mut t1=self.take_values();self.text_values(&caps,&[62],&mut t1);if t1.len()>1 {return Err("captureName requires at most one value".into());}let f1=t1.pop();self.give_values(t1);
let f1:Option<u32>=f1;
let mut v2=self.take_values();self.node_values(&caps,&[60],&[],None,&mut v2)?;
if !v2.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_TypeofElement}) {return Err("typeofConstraint: mapped value type mismatch".into());}
if v2.len()>1 {return Err("typeofConstraint requires at most one node".into());}let f2=v2.pop();self.give_values(v2);
let f2:Option<u32>=f2;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_AnnotatedElement,51,span,&[f0,f1.unwrap_or(tree::NONE),f2.unwrap_or(tree::NONE)]);out.push(node);Ok(())}}
fn map_rule_52(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{if self.value_region>0 {return Ok(());}let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
fn map_rule_53(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
fn map_rule_54(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{if self.value_region>0 {return Ok(());}let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
fn map_rule_55(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{if self.value_region>0 {return Ok(());}let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
fn map_rule_56(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
fn map_rule_57(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[63],&[],None,&mut v0)?;
if !v0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_ChoiceBody}) {return Err("body: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[63]) {return Ok(());}
if v0.len()!=1 {return Err("body requires one node".into());}let f0=v0.pop().unwrap();self.give_values(v0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_GroupElement,57,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_58(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[64],&[],None,&mut v0)?;
if !v0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_ChoiceBody}) {return Err("body: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[64]) {return Ok(());}
if v0.len()!=1 {return Err("body requires one node".into());}let f0=v0.pop().unwrap();self.give_values(v0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_OptionalElement,58,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_59(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[65],&[],None,&mut v0)?;
if !v0.iter().all(|value|{let k=self.tree.kind(*value);k==tree::K_g_UBNFAST_2e_ChoiceBody}) {return Err("body: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[65]) {return Ok(());}
if v0.len()!=1 {return Err("body requires one node".into());}let f0=v0.pop().unwrap();self.give_values(v0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_RepeatElement,59,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_60(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[66],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[66]) {return Ok(());}
if t0.len()!=1 {return Err("message requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_ErrorElement,60,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_61(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[67],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[67]) {return Ok(());}
if t0.len()!=1 {return Err("value requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_TerminalElement,61,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_62(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{if self.value_region>0 {return Ok(());}let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
fn map_rule_63(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_values();self.text_values(&caps,&[70],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[70]) {return Ok(());}
if t0.len()!=1 {return Err("captureName requires one value".into());}let f0=t0.pop().unwrap();self.give_values(t0);
let f0:u32=f0;
self.give_captures(caps);let span=self.span(span);let node=self.tree.record(tree::K_g_UBNFAST_2e_TypeofElement,63,span,&[f0]);out.push(node);Ok(())}}
fn map_rule_64(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=self.take_values();self.text_values(&caps,&[71, 72],&mut f0);
let f0:Vec<u32>=f0;
self.give_captures(caps);let span=self.span(span);let l0=self.tree.list(&f0);self.give_values(f0);let node=self.tree.record(tree::K_g_UBNFAST_2e_DottedIdentifier,64,span,&[l0[0],l0[1]]);out.push(node);Ok(())}}
fn map_rule_65(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<u32>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(text);}Ok(())}}
/// 出現（capture / rule / token）の収集と、AST 構築が rule ごとに読む直下 capture 表を
/// 1 回の走査で作る。以前は出現収集の後に AST 構築が rule ごとに `collect_captures` で
/// 同じ木をもう一度下っていた。
///
/// 直下 capture 表の作り方: capture は完了順に `pending` へ積む。走査は深さ優先なので
/// rule R に入った時点の `pending` の長さ `start` を覚えておけば、R が完了した時点の
/// `pending[start..]` はちょうど「R の直下 capture（間に別の rule を挟まないもの）を完了順に
/// 並べたもの」になる（内側の rule は自分の完了時に自分の分を抜き取って `pending` を
/// `start` まで戻すため、R の分だけが連続して残る）。これを `caps_flat` へ移して
/// `(開始, 個数)` を rule の event id に記録する。公開する出現（`captures` / `lexical` /
/// `tokens`）の並びと番号付けは従来どおり。
fn occurrences(
    &mut self,
    root: EventId,
) -> (Vec<Capture>, Vec<LexicalOccurrence>, Vec<LexicalOccurrence>) {
    // 出現数は event 数で上から見積もる（capture / rule は event の一部）。伸長の複製を避ける。
    // P4 実測は capture も rule も event の 1/8 前後（complex 155+177 / 1,308、
    // complex-x64 9,731+10,824 / 80,246）。1/4 は 2 倍以上の過大確保で、x64 では
    // `Capture` 72 byte × 20,061 を確保して半分しか使っていなかった。
    let estimate = self.arena.len() / 6 + 8;
    let build_ast = self.options.wants_ast();
    let want_lexical = self.options.lexical;
    // 公開する出現表は opt-in（D-035）。見積りで確保するのは要求されたときだけ
    // （AST だけのときに JSON 1 MB で 260,000 × 72 byte を確保して捨てていた）。
    let want_occurrences = self.options.occurrences || want_lexical;
    let mut captures = Vec::with_capacity(if want_occurrences { estimate } else { 0 });
    let mut lexical = Vec::new();
    let mut tokens = Vec::new();
    // AST の直下 capture 表（`pending` / `caps_flat`）は `build_ast` のままで、出現表の flag には
    // 依らない。走査の骨組み（rule ごとの開始位置）はどちらかが要るときに積む。
    let want_intervals = build_ast || want_occurrences;
    // 未完了の rule に属する capture の番号（完了順）。rule の完了時にその rule の
    // 完了順を直接書き込むので、所属表と後段の付け直しの走査は要らない。
    let mut pending_ids: Vec<u32> = Vec::with_capacity(if want_occurrences { estimate } else { 0 });
    // rule ごとの開始位置。`.0` は直下 capture 表（AST 用）、`.1` は公開出現の番号（出現表用）。
    let mut rule_starts: Vec<(u32, u32)> = Vec::new();
    let mut rule_occurrences: u32 = 0;
    let mut rule_order: u32 = 0;
    // 直下 capture 表（AST を作るときだけ）。event id で索く表は伸長だけする（本文の注記参照）。
    let mut caps_flat = std::mem::take(&mut self.caps_flat);
    let mut pending = std::mem::take(&mut self.caps_pending);
    if want_intervals {
        rule_starts.reserve(64);
    }
    if build_ast {
        caps_flat.clear();
        caps_flat.reserve(estimate);
        pending.clear();
    }
    // stack の要素は (event, 親 rule 出現, 出口か, 自身の rule 出現)。無しは u32::MAX。
    // event id も出現番号も入力 byte 数（u32 上限）で抑えられるので u32 に詰める（1 要素 16 byte）。
    const NONE: u32 = u32::MAX;
    let mut stack: Vec<(u32, u32, bool, u32)> = Vec::with_capacity(64);
    stack.push((root.0 as u32, NONE, false, NONE));
    let mut lexical_order = 0;
    // AST 木の見積り（D-077）: 完了した規則が作る record の slot 数の和。
    let mut rule_slots = 0usize;
    while let Some((id, parent, done, current)) = stack.pop() {
        let parent_index = (parent != NONE).then_some(parent as usize);
        let id = EventId(id as usize);
        match self.ev(id) {
            Event::Join(a, b) => {
                stack.push((b.0 as u32, parent, false, NONE));
                stack.push((a.0 as u32, parent, false, NONE));
            }
            Event::Values { child, .. } => stack.push((child.0 as u32, parent, false, NONE)),
            Event::Capture {
                site,
                span,
                child,
                token_extent,
            } => {
                if done {
                    if build_ast {
                        let extent = if token_extent {
                            self.selected_extent(child, span)
                        } else {
                            span
                        };
                        pending.push((site, extent, child));
                    }
                    if want_occurrences {
                        let order = captures.len();
                        pending_ids.push(order as u32);
                        captures.push(Capture {
                            rule_completion_order: 0,
                            occurrence_id: order,
                            site_id: SITES[site].0,
                            name: SITES[site].1,
                            span: self.span(span),
                            completion_order: order,
                        });
                    }
                } else {
                    stack.push((id.0 as u32, parent, true, NONE));
                    stack.push((child.0 as u32, parent, false, NONE));
                }
            }
            Event::Token {
                expr, rule, span, ..
            }
if want_lexical && expr != usize::MAX => {
                tokens.push(LexicalOccurrence {
                    occurrence_id: tokens.len(),
                    parent_occurrence_id: parent_index,
                    rule_id: RULES[rule],
                    expr_id: EXPRESSIONS[expr],
                    span: self.span(span),
                    completion_order: lexical_order,
                });
                lexical_order += 1;
            }
            Event::Rule {
                rule, span, child, ..
            } => {
                if done {
                    if want_intervals {
                        let (cap_start, id_start) = rule_starts.pop().unwrap_or((0, 0));
                        if build_ast {
                            rule_slots += RULE_SLOTS[rule] as usize;
                            let start = cap_start as usize;
                            let offset = caps_flat.len();
                            caps_flat.extend_from_slice(&pending[start..]);
                            pending.truncate(start);
                            self.set_rule_caps(
                                id,
                                (offset as u32, (caps_flat.len() - offset) as u32),
                            );
                        }
                        if want_occurrences {
                            let start = id_start as usize;
                            // この rule の直下 capture に rule 完了順を書く（後段の走査の代わり）。
                            for &order in &pending_ids[start..] {
                                captures[order as usize].rule_completion_order =
                                    rule_order as usize;
                            }
                            pending_ids.truncate(start);
                        }
                    }
                    if current != NONE {
                        let index = current as usize;
                        rule_order += 1;
                        // rule 出現は lexical のときだけ保持する（AST だけなら完了順だけ要る）。
                        if want_lexical {
                            let node: &mut LexicalOccurrence = &mut lexical[index];
                            node.completion_order = lexical_order;
                        }
                        lexical_order += 1;
                    }
                } else {
                    if want_intervals {
                        rule_starts.push((pending.len() as u32, pending_ids.len() as u32));
                    }
                    let index = if want_occurrences {
                        let index = rule_occurrences;
                        rule_occurrences += 1;
                        if want_lexical {
                            lexical.push(LexicalOccurrence {
                                occurrence_id: index as usize,
                                parent_occurrence_id: parent_index,
                                rule_id: RULES[rule],
                                expr_id: BODIES[rule],
                                span: self.span(span),
                                completion_order: 0,
                            });
                        }
                        index
                    } else {
                        parent
                    };
                    stack.push((id.0 as u32, parent, true, index));
                    stack.push((child.0 as u32, index, false, NONE));
                }
            }
            _ => {}
        }
    }
    for token in &mut tokens {
        token.occurrence_id += lexical.len();
    }
    if build_ast {
        // 木は所有 `Ast` だけの parse では pool に戻るので、既に足りていれば確保しない。
        // 根以外の値はどれかの capture の値なので、節点数は capture 数でほぼ決まる
        // （JSON 1 MB: capture 178,342 / 節点 178,343。P4 x64: capture 9,731 / 節点 10,434。
        // P4 は leaf 型への昇格が節点を足す）。slot は record の field 分（`RULE_SLOTS`）と
        // list の要素（capture 数が上限）。
        let captured = caps_flat.len();
        self.tree
            .reserve(captured + captured / 8 + 1, rule_slots + captured);
    }
    self.caps_flat = caps_flat;
    self.caps_pending = pending;
    (captures, lexical, tokens)
}
fn recovery_occurrences(&self, root: EventId) -> (Vec<Recovery>, Vec<Diagnostic>) {
    // D-025: `captureOccurrences` は「回復した領域を**自分の値としてそのまま持つ**
    // capture 出現」。間に AST ノードを作る（あるいは捨てる）規則が入ると、その外側の
    // capture の値はノードであって回復領域ではないので数えない。node 境界の深さを
    // 数え、capture と同じ深さにある回復だけを結び付ける。
    enum Visit {
        Enter(EventId),
        CaptureDone { start: usize, depth: u32 },
        NodeLeave,
    }
    let mut recoveries: Vec<Recovery> = Vec::new();
    // 回復ごとの node 境界の深さ（`recoveries` と同じ並び）。
    let mut recovery_depth: Vec<u32> = Vec::new();
    let mut diagnostics = Vec::new();
    let mut stack = vec![Visit::Enter(root)];
    let mut capture_order = 0;
    let mut depth = 0u32;
    while let Some(visit) = stack.pop() {
        let id = match visit {
            Visit::NodeLeave => {
                depth -= 1;
                continue;
            }
            Visit::CaptureDone { start, depth: at } => {
                // `capture_occurrence_ids` は出現表を返すかに依らない観測なので、
                // 番号付けは AST / lexical のときそのまま数える（D-035 の対象外）。
                if self.options.wants_ast() || self.options.lexical {
                    for (r, recovery) in recoveries[start..].iter_mut().enumerate() {
                        if recovery_depth[start + r] == at {
                            recovery.capture_occurrence_ids.push(capture_order);
                        }
                    }
                    capture_order += 1;
                }
                continue;
            }
            Visit::Enter(id) => id,
        };
        match self.ev(id) {
            Event::Join(a, b) => {
                stack.push(Visit::Enter(b));
                stack.push(Visit::Enter(a));
            }
            Event::Rule { rule, child, .. } => {
                if RULE_NODE[rule] {
                    depth += 1;
                    stack.push(Visit::NodeLeave);
                }
                stack.push(Visit::Enter(child));
            }
            Event::Values { child, .. } => stack.push(Visit::Enter(child)),
            Event::Capture { child, .. } => {
                stack.push(Visit::CaptureDone {
                    start: recoveries.len(),
                    depth,
                });
                stack.push(Visit::Enter(child));
            }
            Event::Recovery { span, detail } => {
                let RecoveryEvent {
                    rule,
                    mode,
                    sync_span,
                    diag,
                    hints,
                } = self.recovery_events[detail as usize];
                // D-027: 公開する候補は表示語彙（`farthestExpected` と同じ蓄積）で、
                // 範囲は回復した規則の frame。主診断 DAG の label は Rust 内部の語彙
                // なので候補には使わず、規則経路の復元にだけ使う（D-020）。
                let path = self.diag.summary(diag).map_or(vec![], |(_, _, path)| path);
                let (far, expected) = self
                    .recovery_hints
                    .get(hints as usize)
                    .and_then(Clone::clone)
                    .unwrap_or((span[0], vec![]));
                let rule_path: Vec<_> = path.into_iter().map(|r| RULES[r]).collect();
                let span = self.span(span);
                diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Recovery,
                    offset_cp: span[0],
                    length_cp: span[1] - span[0],
                    farthest_cp: self.cp(far),
                    expected: expected.clone(),
                    farthest_expected: expected,
                    deepest_rule: rule_path.last().copied(),
                    rule_path,
                    message: Some("recovered input".into()),
                    recovery_id: Some(recoveries.len()),
                });
                recovery_depth.push(depth);
                recoveries.push(Recovery {
                    rule_id: RULES[rule],
                    mode,
                    span,
                    sync_span: sync_span.map(|s| self.span(s)),
                    capture_occurrence_ids: vec![],
                });
            }
            _ => {}
        }
    }
    (recoveries, diagnostics)
}
pub(crate) fn finish(self, step: Step) -> ParseResult {
    self.finish_checked(step).0
}
/// 結果と「診断付きで解析し直す必要があるか」を返す（`ParseOptions::diagnostics` 参照）。
/// fast mode で診断が要る結果になった入口は、同じ入力を診断モードで解析し直す。
pub(crate) fn finish_checked(mut self, mut step: Step) -> (ParseResult, bool) {
    if let Some(offset) = self.depth_failure {
        step = self.fail(State::default(), offset, "maximum parse depth exceeded");
    }
    if self.diag.exhausted || self.display.exhausted {
        self.resource_failure
            .get_or_insert((0, "diagnostic budget exceeded"));
    }
    if self.resource_failure.is_some() {
        step = Step {
            ok: false,
            ..Step::yes(State::default())
        };
    }
    // require_eof を問わず「入力が残った」ことを再解析の条件にする（保守的）。
    let incomplete = step.state.consumed != self.text.len();
    let trailing = step.ok && self.options.require_eof && incomplete;
    let ok = step.ok && !trailing;
    let consumed_cp = if step.ok {
        self.cp(step.state.consumed)
    } else {
        0
    };
    let matched_cp = if step.ok {
        self.cp(step.state.matched)
    } else {
        0
    };
    let mut diagnostics = Vec::new();
    if let Some((offset, label)) = self.resource_failure {
        diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Resource,
            offset_cp: self.cp(offset),
            farthest_cp: self.cp(offset),
            expected: vec![label],
            farthest_expected: vec![label],
            deepest_rule: None,
            rule_path: vec![],
            message: Some(label.into()),
            recovery_id: None,
            length_cp: 0,
        });
    }
    if let Some((pos, expected, path)) = self.diag.summary(step.diag) {
        let rule_path: Vec<_> = path.into_iter().map(|r| RULES[r]).collect();
        diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Syntax,
            offset_cp: self.cp(pos),
            farthest_cp: self.cp(pos),
            farthest_expected: expected.clone(),
            expected,
            deepest_rule: rule_path.last().copied(),
            rule_path,
            message: None,
            recovery_id: None,
            length_cp: 0,
        });
    }
    // D-014: farthest は失敗 frame / 成功枝内の失敗も含む独立観測。
    let (display_pos, display_expected) = self.display.summary();
    for diagnostic in &mut diagnostics {
        if diagnostic.kind == DiagnosticKind::Syntax {
            diagnostic.farthest_cp = self.cp(display_pos);
            diagnostic.farthest_expected = display_expected.clone();
        }
    }
    let hints = diagnostics
        .iter()
        .filter(|d| d.kind == DiagnosticKind::Syntax)
        .cloned()
        .collect();
    if trailing {
        let (farthest_cp, farthest_expected, deepest_rule, rule_path) =
            diagnostics
                .pop()
                .map_or((self.cp(display_pos), display_expected, None, vec![]), |d| {
                    (
                        d.farthest_cp,
                        d.farthest_expected,
                        d.deepest_rule,
                        d.rule_path,
                    )
                });
        diagnostics.push(Diagnostic {
            kind: DiagnosticKind::TrailingInput,
            offset_cp: consumed_cp,
            farthest_cp,
            farthest_expected,
            expected: vec!["end of input"],
            deepest_rule,
            rule_path,
            message: None,
            recovery_id: None,
            length_cp: 0,
        });
    }
    if ok {
        diagnostics.clear();
    }
    // recovery を持たない文法では Recovery event が無く、走査しても空になる（走査を省く）。
    let (recoveries, recovery_diagnostics) = if step.ok && HAS_RECOVERY {
        self.recovery_occurrences(step.events)
    } else {
        (vec![], vec![])
    };
    diagnostics.splice(0..0, recovery_diagnostics);
    // 走査は AST の直下 capture 表のために `build_ast` でも要る。公開する出現表を作るかは
    // `occurrences` / `lexical` が決める（D-035）。認識だけで要求も無ければ走査ごと省く。
    let (captures, lexical, tokens) =
        if step.ok
            && (self.options.wants_ast() || self.options.occurrences || self.options.lexical)
        {
            self.occurrences(step.events)
        } else {
            (vec![], vec![], vec![])
        };
    // D-077: AST は木（`self.tree`）へ作り、所有 `Ast` はそこから写す。
    let mut root = None;
    let mut mapped = false;
    if ok && self.options.wants_ast() {
        match self.build_values(step.events).and_then(|nodes| {
            if nodes.len() == 1 {
                Ok(Some(nodes[0]))
            } else if nodes.is_empty() {
                // D-028: entry が AST 値を作らないのは mapping 失敗ではない（ast=null / mappingError=null）。
                Ok(None)
            } else {
                Err("entry produced more than one AST value".into())
            }
        }) {
            Ok(value) => {
                root = value;
                mapped = true;
            }
            Err(message) => {
                diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Mapping,
                    offset_cp: consumed_cp,
                    farthest_cp: consumed_cp,
                    expected: vec![],
                    farthest_expected: vec![],
                    deepest_rule: None,
                    rule_path: vec![],
                    message: Some(message),
                    recovery_id: None,
                    length_cp: 0,
                });
            }
        }
    }
    let ast = if self.options.build_ast {
        root.map(|root| self.tree.project(self.text, root))
    } else {
        None
    };
    let mut node_spans = Vec::new();
    if mapped && self.options.build_ast {
        node_spans.reserve_exact(self.tree.typed_node_count());
        self.tree.push_node_spans(&mut node_spans);
    }
    self.statistics.memo_entries = self.memo.len();
    self.statistics.memo_max_probe = self
        .memo
        .max_probe_len()
        .max(self.trivia_cache.max_probe_len());
    self.statistics.recipes = self.arena.len() - 1;
    self.statistics.ast_nodes = if mapped {
        self.tree.typed_node_count()
    } else {
        0
    };
    self.statistics.lexical_runs = self.lex.runs;
    self.statistics.lexical_probes = self.lex.probes;
    self.statistics.trivia_skip_hits = self.trivia_skip.hits.get();
    self.statistics.trivia_skip_misses = self.trivia_skip.misses.get();
    // value span は要求されたときだけ集める（D-035）。AST 自体は先に完成している。
    let mut value_spans = Vec::new();
    if let (true, Some(root)) = (self.options.value_spans, root) {
        value_spans.reserve(self.tree.node_count());
        let mut path = String::with_capacity(128);
        self.tree
            .collect_value_spans(self.text, root, &mut path, &mut value_spans);
    }
    // 木を返すときは結果へ移し（入力を 1 回複製する）、返さないときは pool へ戻す。
    let tree = match root {
        Some(root) if self.options.ast_tree => {
            let mut tree = std::mem::take(&mut self.tree);
            tree.set_root(root);
            tree.set_source(self.text);
            Some(tree)
        }
        _ => None,
    };
    let result = ParseResult {
        value_spans,
        ok,
        consumed_cp,
        matched_cp,
        ast,
        tree,
        captures,
        lexical,
        tokens,
        node_spans,
        diagnostics,
        hints,
        recoveries,
        declarations: if step.ok {
            self.scope.all_declarations().to_vec()
        } else {
            vec![]
        },
        references: if step.ok {
            self.scope.references().to_vec()
        } else {
            vec![]
        },
        scope_diagnostics: if step.ok {
            self.scope.diagnostics().to_vec()
        } else {
            vec![]
        },
        statistics: self.statistics,
        scope: ScopeResult {
            depth: self.scope.depth(),
            mode: SCOPE_MODES.to_vec(),
            events: if step.ok {
                self.effects
                    .iter()
                    .enumerate()
                    .map(|(order, effect)| {
                        // D-029: enter / leave / clear の name は scope を持つ規則名。
                        let owner = || effect.name.map(str::to_owned);
                        let (action, name, offset_cp, length_cp) = match &*effect.value {
                            ScanEffect::Enter => ("enter", owner(), effect.offset_cp, 0),
                            ScanEffect::Leave => ("leave", owner(), effect.offset_cp, 0),
                            ScanEffect::Clear => ("clear", owner(), effect.offset_cp, 0),
                            ScanEffect::Declare { name, offset_cp } => {
                                ("declare", Some(name.clone()), *offset_cp, effect.length_cp)
                            }
                            ScanEffect::Use {
                                name,
                                offset_cp,
                                len_cp,
                            } => ("use", Some(name.clone()), *offset_cp, *len_cp),
                            ScanEffect::Diagnostic {
                                message,
                                offset_cp,
                                len_cp,
                                ..
                            } => ("error", Some(message.clone()), *offset_cp, *len_cp),
                        };
                        ScopeEvent {
                            order,
                            action,
                            mode: effect.mode,
                            name,
                            offset_cp,
                            length_cp,
                        }
                    })
                    .collect()
            } else {
                vec![]
            },
        },
    };
    // 診断を要する結果（失敗・入力の残り・recovery / mapping / resource 診断）は fast mode では
    // 観測を持たないので、入口が診断モードで解析し直す。scope の意味診断（`scope_diagnostics`）は
    // 診断機構ではなく scope store の効果なので、fast mode でも同じものが得られる（再解析しない）。
    let escalate = !DIAG
        && (!result.ok
            || incomplete
            || !result.diagnostics.is_empty()
            || !result.recoveries.is_empty());
    self.recycle();
    (result, escalate)
}
}
