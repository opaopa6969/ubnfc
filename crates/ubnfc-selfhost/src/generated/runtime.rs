use super::api::*;
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
    Recovery {
        rule: usize,
        mode: &'static str,
        span: Span,
        sync_span: Option<Span>,
        diag: Diag,
    },
    Join(EventId, EventId),
    Values {
        span: Span,
        child: EventId,
        wrap: bool,
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
/// mapped text の識別子（String の先頭 pointer）から span を引く。乗算 hash で十分に散る。
#[derive(Default)]
pub(crate) struct PtrHasher(u64);
impl std::hash::Hasher for PtrHasher {
    fn finish(&self) -> u64 {
        let h = self.0.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 29)
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 << 8) | u64::from(b);
        }
    }
    fn write_usize(&mut self, n: usize) {
        self.0 = n as u64 >> 3;
    }
}
pub(crate) type TextSpans =
    std::collections::HashMap<usize, Span, std::hash::BuildHasherDefault<PtrHasher>>;
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
    arena: Vec<Event>,
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
    values_pool: Vec<Vec<Ast>>,
    texts_pool: Vec<Vec<String>>,
    scope_pool: Vec<Vec<(usize, Span)>>,
    text_spans: TextSpans,
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
        self.arena.capacity() * std::mem::size_of::<Event>()
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
    pub arena: Vec<Event>,
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
    pub node_spans: Vec<NodeSpan>,
    // Allocation identity is used only while mapping. finish resolves it into
    // owned AST paths, so ParseResult remains valid after moves and clones.
    pub text_spans: TextSpans,
    /// AST 構築の走査用 stack の再利用 pool（再帰するので複数）。
    stacks: Vec<Vec<(EventId, bool)>>,
    /// AST 構築の作業 Vec の再利用 pool（capture 出現、field 値、field 文字列）。
    captures_pool: Vec<Vec<Cap>>,
    values_pool: Vec<Vec<Ast>>,
    texts_pool: Vec<Vec<String>>,
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
        let arena_capacity = if options.build_ast || options.lexical || HAS_SCOPE || HAS_RECOVERY {
            // P4 実測 約 4.4 event / byte。
            text.len().saturating_mul(5).saturating_add(8).min(32768)
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
                b.text_spans.clear();
                b.values_pool.iter_mut().for_each(Vec::clear);
                b.texts_pool.iter_mut().for_each(Vec::clear);
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
                texts_pool: vec![],
                scope_pool: vec![],
                text_spans: Default::default(),
                caps_flat: vec![],
                caps_pending: vec![],
            },
        };
        buffers.diag.enable(diag_on);
        buffers.display.enable(diag_on);
        let mut arena = buffers.arena;
        arena.push(Event::Empty);
        // AST node は入力 byte の約 1/2.5（P4 実測 332 byte / 131 node、20,923 byte / 8,195 node）。
        // 結果へ移すので pool 化しない。
        let node_capacity = if options.build_ast {
            text.len() / 2
        } else {
            0
        };
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
            node_spans: Vec::with_capacity(node_capacity),
            text_spans: buffers.text_spans,
            stacks: buffers.stacks,
            captures_pool: buffers.captures_pool,
            values_pool: buffers.values_pool,
            texts_pool: buffers.texts_pool,
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
        }
    }
    /// 大きな buffer を thread の pool へ返す（`finish` の最後）。
    fn recycle(&mut self) {
        if !self.options.reuse_buffers {
            return;
        }
        let buffers = Buffers {
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
            texts_pool: std::mem::take(&mut self.texts_pool),
            scope_pool: std::mem::take(&mut self.scope_pool),
            text_spans: std::mem::take(&mut self.text_spans),
            caps_flat: std::mem::take(&mut self.caps_flat),
            caps_pending: std::mem::take(&mut self.caps_pending),
        };
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
    /// `remember_text` へ渡す複製は識別用の 1 byte を最初から確保する
    /// （`to_owned` は丁度 len で確保するので、直後の `reserve(1)` が必ず再確保になる）。
    fn owned_text(text: &str) -> String {
        let mut owned = String::with_capacity(text.len() + 1);
        owned.push_str(text);
        owned
    }
    fn text_for_identity(&self, span: Span) -> String {
        Self::owned_text(self.trimmed(span))
    }
    pub fn span(&self, span: Span) -> Span {
        [self.cp(span[0]), self.cp(span[1])]
    }
    fn trimmed(&self, span: Span) -> &str {
        self.text[span[0]..span[1]].trim_matches(|c: char| {
            c.is_whitespace() && !matches!(c, '\u{85}' | '\u{a0}' | '\u{2007}' | '\u{202f}')
        })
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
    pub fn event(&mut self, event: Event) -> EventId {
        if !self.options.build_ast && !self.options.lexical && !HAS_SCOPE && !HAS_RECOVERY {
            return EventId(0);
        }
        if self.arena.len() >= self.arena_limit.min(u32::MAX as usize) {
            self.resource_failure
                .get_or_insert((0, "arena budget exceeded"));
            return EventId(0);
        }
        let id = EventId(self.arena.len());
        self.arena.push(event);
        id
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
                if let Event::Token { text_span, content_span, .. } = &mut self.arena[step.events.0] {
                    *text_span = Some([start, p]);
                    *content_span = Some([start + 1, p - 1]);
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
        if let Event::Capture {
            site: current,
            child,
            ..
        } = &mut self.arena[id.0]
        {
            if let Some(site) = site {
                *current = site;
            }
            target = *child;
        }
        if let (Some(expr), Event::Token { expr: current, .. }) =
            (expr, &mut self.arena[target.0])
        {
            *current = expr;
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
    pub fn take_values(&mut self) -> Vec<Ast> {
        let mut v = self.values_pool.pop().unwrap_or_default();
        v.clear();
        v
    }
    pub fn give_values(&mut self, v: Vec<Ast>) {
        self.values_pool.push(v);
    }
    pub fn take_texts(&mut self) -> Vec<String> {
        let mut v = self.texts_pool.pop().unwrap_or_default();
        v.clear();
        v
    }
    pub fn give_texts(&mut self, v: Vec<String>) {
        self.texts_pool.push(v);
    }
    /// text 変換の field 値（recovery で直接置換された出現は除く）。
    pub fn text_values(&mut self, caps: &[Cap], sites: &[usize], out: &mut Vec<String>) {
        for &(site, span, child) in caps {
            if sites.contains(&site) && !self.directly_recovered(child) {
                let text = self.semantic_text(child, span);
                out.push(text);
            }
        }
    }
    pub fn text_nodes(&mut self, caps: &[Cap], sites: &[usize], out: &mut Vec<Ast>) {
        for &(site, span, child) in caps {
            if sites.contains(&site) && !self.directly_recovered(child) {
                let text = self.semantic_text(child, span);
                out.push(Ast::Text(text));
            }
        }
    }
    /// node 変換の field 値。出現ごとに子の値を構築し、値が無く空でない出現は字句 Text にする。
    /// `fallback` は leaf 型（Text を leaf node に昇格する recipe）。
    pub fn node_values(
        &mut self,
        caps: &[Cap],
        sites: &[usize],
        fallback: Option<usize>,
        out: &mut Vec<Ast>,
    ) -> Result<(), String> {
        for &(site, span, child) in caps {
            if !sites.contains(&site) {
                continue;
            }
            let start = out.len();
            self.build_values_into(child, out)?;
            if out.len() == start
                && span[0] != span[1]
                && !self.has_recovery(child)
                && !self.has_value_group(child)
            {
                let text = self.semantic_text(child, span);
                out.push(Ast::Text(text));
            }
            if let Some(ty) = fallback {
                let mut i = start;
                while i < out.len() {
                    if let Ast::Text(text) = &mut out[i] {
                        let text = std::mem::take(text);
                        out[i] = self.leaf(ty, span, text)?;
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
            match self.arena[id.0] {
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
            match self.arena[id.0] {
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
            match self.arena[id.0] {
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
            match self.arena[root.0] {
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
        let events = self.event(Event::Recovery {
            rule,
            mode,
            span: [start, end],
            sync_span,
            diag,
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
            match self.arena[id.0] {
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
    pub fn mapped_text(&mut self, span: Span) -> String {
        self.remember_text(self.text_for_identity(span), span)
    }
    fn remember_text(&mut self, mut text: String, span: Span) -> String {
        // Empty strings also need distinct identities until paths are resolved.
        text.reserve(1);
        self.text_spans
            .insert(text.as_ptr() as usize, self.span(span));
        text
    }
    pub fn has_token_text(&self, mut root: EventId) -> bool {
        loop {
            match self.arena[root.0] {
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
            match self.arena[root.0] {
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
            match self.arena[root.0] {
                Event::Capture { child, .. } | Event::Rule { child, .. } | Event::Values { child, .. } => root = child,
                Event::Token { content_span, text_span, .. } => return content_span.or(text_span).unwrap_or(fallback),
                _ => return fallback,
            }
        }
    }
    pub fn semantic_text(&mut self, root: EventId, span: Span) -> String {
        // Token まで下って text_span / content_span を 1 回の走査で読む
        // （text_extent / text_content_extent / has_token_text と同じ規則）。
        let mut node = root;
        let token = loop {
            match self.arena[node.0] {
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
        let value = if token_text { Self::owned_text(&self.text[content[0]..content[1]]) } else { self.text_for_identity(content) };
        self.remember_text(value, extent)
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
            match self.arena[id.0] {
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
    pub fn node(&mut self, rule: &'static str, name: &'static str, span: Span) -> usize {
        let id = self.node_spans.len();
        self.node_spans.push(NodeSpan {
            node_id: id,
            rule_id: rule,
            node_type: name,
            span,
        });
        id
    }
}
/// 統合 DFA の bitset 語数（literal 62 個）。0 なら cache を作らない。
const LEX_WORDS:usize=1;
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
67=>173,
69=>184,
76=>195,
78=>204,
82=>224,
85=>229,
91=>{out[0]|=35184372088832;234},
93=>{out[0]|=70368744177664;235},
99=>236,
100=>243,
102=>254,
103=>258,
108=>265,
109=>270,
110=>274,
112=>278,
115=>290,
116=>298,
123=>{out[0]|=576460752303423488;303},
124=>{out[0]|=1152921504606846976;304},
125=>{out[0]|=2305843009213693952;305},
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
78=>171,
_=>return,
},
171=>match c {
89=>{out[0]|=17179869184;172},
_=>return,
},
173=>match c {
72=>174,
73=>{out[0]|=68719476736;183},
_=>return,
},
174=>match c {
65=>175,
_=>return,
},
175=>match c {
82=>176,
_=>return,
},
176=>match c {
95=>177,
_=>return,
},
177=>match c {
82=>178,
_=>return,
},
178=>match c {
65=>179,
_=>return,
},
179=>match c {
78=>180,
_=>return,
},
180=>match c {
71=>181,
_=>return,
},
181=>match c {
69=>{out[0]|=34359738368;182},
_=>return,
},
184=>match c {
77=>185,
79=>189,
82=>191,
_=>return,
},
185=>match c {
80=>186,
_=>return,
},
186=>match c {
84=>187,
_=>return,
},
187=>match c {
89=>{out[0]|=137438953472;188},
_=>return,
},
189=>match c {
70=>{out[0]|=274877906944;190},
_=>return,
},
191=>match c {
82=>192,
_=>return,
},
192=>match c {
79=>193,
_=>return,
},
193=>match c {
82=>{out[0]|=549755813888;194},
_=>return,
},
195=>match c {
79=>196,
_=>return,
},
196=>match c {
79=>197,
_=>return,
},
197=>match c {
75=>198,
_=>return,
},
198=>match c {
65=>199,
_=>return,
},
199=>match c {
72=>200,
_=>return,
},
200=>match c {
69=>201,
_=>return,
},
201=>match c {
65=>202,
_=>return,
},
202=>match c {
68=>{out[0]|=1099511627776;203},
_=>return,
},
204=>match c {
69=>205,
_=>return,
},
205=>match c {
71=>206,
_=>return,
},
206=>match c {
65=>207,
_=>return,
},
207=>match c {
84=>208,
_=>return,
},
208=>match c {
73=>209,
_=>return,
},
209=>match c {
79=>210,
86=>212,
_=>return,
},
210=>match c {
78=>{out[0]|=2199023255552;211},
_=>return,
},
212=>match c {
69=>213,
_=>return,
},
213=>match c {
95=>214,
_=>return,
},
214=>match c {
76=>215,
_=>return,
},
215=>match c {
79=>216,
_=>return,
},
216=>match c {
79=>217,
_=>return,
},
217=>match c {
75=>218,
_=>return,
},
218=>match c {
65=>219,
_=>return,
},
219=>match c {
72=>220,
_=>return,
},
220=>match c {
69=>221,
_=>return,
},
221=>match c {
65=>222,
_=>return,
},
222=>match c {
68=>{out[0]|=4398046511104;223},
_=>return,
},
224=>match c {
69=>225,
_=>return,
},
225=>match c {
71=>226,
_=>return,
},
226=>match c {
69=>227,
_=>return,
},
227=>match c {
88=>{out[0]|=8796093022208;228},
_=>return,
},
229=>match c {
78=>230,
_=>return,
},
230=>match c {
84=>231,
_=>return,
},
231=>match c {
73=>232,
_=>return,
},
232=>match c {
76=>{out[0]|=17592186044416;233},
_=>return,
},
236=>match c {
111=>237,
_=>return,
},
237=>match c {
110=>238,
_=>return,
},
238=>match c {
116=>239,
_=>return,
},
239=>match c {
101=>240,
_=>return,
},
240=>match c {
120=>241,
_=>return,
},
241=>match c {
116=>{out[0]|=140737488355328;242},
_=>return,
},
243=>match c {
101=>244,
_=>return,
},
244=>match c {
115=>245,
_=>return,
},
245=>match c {
99=>246,
_=>return,
},
246=>match c {
114=>247,
_=>return,
},
247=>match c {
105=>248,
_=>return,
},
248=>match c {
112=>249,
_=>return,
},
249=>match c {
116=>250,
_=>return,
},
250=>match c {
105=>251,
_=>return,
},
251=>match c {
111=>252,
_=>return,
},
252=>match c {
110=>{out[0]|=281474976710656;253},
_=>return,
},
254=>match c {
114=>255,
_=>return,
},
255=>match c {
111=>256,
_=>return,
},
256=>match c {
109=>{out[0]|=562949953421312;257},
_=>return,
},
258=>match c {
114=>259,
_=>return,
},
259=>match c {
97=>260,
_=>return,
},
260=>match c {
109=>261,
_=>return,
},
261=>match c {
109=>262,
_=>return,
},
262=>match c {
97=>263,
_=>return,
},
263=>match c {
114=>{out[0]|=1125899906842624;264},
_=>return,
},
265=>match c {
101=>266,
_=>return,
},
266=>match c {
118=>267,
_=>return,
},
267=>match c {
101=>268,
_=>return,
},
268=>match c {
108=>{out[0]|=2251799813685248;269},
_=>return,
},
270=>match c {
111=>271,
_=>return,
},
271=>match c {
100=>272,
_=>return,
},
272=>match c {
101=>{out[0]|=4503599627370496;273},
_=>return,
},
274=>match c {
97=>275,
_=>return,
},
275=>match c {
109=>276,
_=>return,
},
276=>match c {
101=>{out[0]|=9007199254740992;277},
_=>return,
},
278=>match c {
97=>279,
114=>284,
_=>return,
},
279=>match c {
114=>280,
_=>return,
},
280=>match c {
97=>281,
_=>return,
},
281=>match c {
109=>282,
_=>return,
},
282=>match c {
115=>{out[0]|=18014398509481984;283},
_=>return,
},
284=>match c {
111=>285,
_=>return,
},
285=>match c {
102=>286,
_=>return,
},
286=>match c {
105=>287,
_=>return,
},
287=>match c {
108=>288,
_=>return,
},
288=>match c {
101=>{out[0]|=36028797018963968;289},
_=>return,
},
290=>match c {
121=>291,
_=>return,
},
291=>match c {
109=>292,
110=>296,
_=>return,
},
292=>match c {
98=>293,
_=>return,
},
293=>match c {
111=>294,
_=>return,
},
294=>match c {
108=>{out[0]|=72057594037927936;295},
_=>return,
},
296=>match c {
99=>{out[0]|=144115188075855872;297},
_=>return,
},
298=>match c {
111=>299,
_=>return,
},
299=>match c {
107=>300,
_=>return,
},
300=>match c {
101=>301,
_=>return,
},
301=>match c {
110=>{out[0]|=288230376151711744;302},
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
const RULES:&[&str]=&["UBNF::UBNFFile", "UBNF::ImportDecl", "UBNF::GrammarDecl", "UBNF::GlobalSetting", "UBNF::SettingValue", "UBNF::StringSettingValue", "UBNF::BlockSettingValue", "UBNF::KeyValuePair", "UBNF::TokenDecl", "UBNF::TokenValue", "UBNF::UntilExpression", "UBNF::NegationExpression", "UBNF::LookaheadExpression", "UBNF::NegativeLookaheadExpression", "UBNF::CharRangeExpression", "UBNF::CIExpression", "UBNF::RegexExpression", "UBNF::AnyExpression", "UBNF::EofExpression", "UBNF::EmptyExpression", "UBNF::RuleDecl", "UBNF::Annotation", "UBNF::RootAnnotation", "UBNF::MappingAnnotation", "UBNF::EvalAnnotation", "UBNF::WhitespaceAnnotation", "UBNF::InterleaveAnnotation", "UBNF::BackrefAnnotation", "UBNF::ScopeTreeAnnotation", "UBNF::DeclaresAnnotation", "UBNF::CatalogAnnotation", "UBNF::LeftAssocAnnotation", "UBNF::RightAssocAnnotation", "UBNF::LongestChoiceAnnotation", "UBNF::PredictiveChoiceAnnotation", "UBNF::PrecedenceAnnotation", "UBNF::DocAnnotation", "UBNF::RecoveryAnnotation", "UBNF::SkipAnnotation", "UBNF::CommonFieldAnnotation", "UBNF::EnumAnnotation", "UBNF::SimpleAnnotation", "UBNF::ChoiceBody", "UBNF::SequenceBody", "UBNF::AnnotatedElement", "UBNF::PostfixQuantifier", "UBNF::SeparatedBy", "UBNF::BoundedQuantifier", "UBNF::Digits", "UBNF::AtomicElement", "UBNF::GroupElement", "UBNF::OptionalElement", "UBNF::RepeatElement", "UBNF::ErrorElement", "UBNF::TerminalElement", "UBNF::QuantifiedRef", "UBNF::TypeofElement", "UBNF::DottedIdentifier", "UBNF::RuleBody"];
const EXPRESSIONS:&[&str]=&["expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1155:1166:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1326:1367:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1326:1335:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1336:1346:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1354:1360:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1361:1367:body/3/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1507:1685:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1507:1516:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1517:1527:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1539:1542:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1550:1564:body/3/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1552:1562:body/3/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1584:1601:body/4/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1586:1599:body/4/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1619:1632:body/5/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1621:1630:body/5/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1652:1664:body/6/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1654:1662:body/6/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1682:1685:body/7/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1785:1821:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1785:1788:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1789:1799:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1805:1808:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1809:1821:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1860:1903:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1860:1878:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1886:1903:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2084:2117:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2084:2087:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2088:2104:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2090:2102:body/1/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2114:2117:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2215:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2199:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2205:2208:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2209:2215:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2357:2396:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2357:2364:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2365:2375:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2382:2385:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2396:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2455:2717:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2455:2470:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2478:2496:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2504:2523:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2531:2558:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2566:2585:body/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2593:2605:body/5/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2613:2628:body/6/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2636:2649:body/7/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2657:2670:body/8/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2678:2693:body/9/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2701:2717:body/10/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2855:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2828:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2829:2832:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2833:2839:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2852:2855:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2958:2990:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2958:2968:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2969:2972:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2973:2979:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2987:2990:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3100:3135:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3100:3111:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3112:3115:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3116:3122:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3132:3135:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3270:3314:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3270:3290:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3291:3294:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3295:3301:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3311:3314:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3424:3472:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3424:3436:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3437:3440:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3441:3447:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3453:3456:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3457:3463:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3469:3472:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3558:3583:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3558:3562:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3563:3566:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3567:3573:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3580:3583:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3681:3712:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3681:3688:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3689:3692:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3693:3699:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3709:3712:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3775:3780:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3775:3780:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3843:3848:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3843:3848:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3917:3924:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3917:3924:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4199:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4129:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4117:4127:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4148:4158:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4170:4175:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4176:4184:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4196:4199:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4255:4767:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4255:4269:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4277:4294:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4302:4316:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4324:4344:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4352:4372:body/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4380:4397:body/5/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4405:4424:body/6/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4432:4450:body/7/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4458:4475:body/8/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4483:4502:body/9/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4510:4530:body/10/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4538:4561:body/11/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4569:4595:body/12/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4603:4623:body/13/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4631:4644:body/14/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4652:4670:body/15/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4678:4692:body/16/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4700:4721:body/17/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4729:4743:body/18/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4751:4767:body/19/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4822:4829:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4822:4829:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4927:5069:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4927:4937:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4938:4941:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4949:4959:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4978:5060:body/3/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4980:5058:body/3/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4980:4983:body/3/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4984:4992:body/3/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4993:4996:body/3/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4997:5000:body/3/0/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5001:5011:body/3/0/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5024:5054:body/3/0/5/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5026:5040:body/3/0/5/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5026:5029:body/3/0/5/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5030:5040:body/3/0/5/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5055:5058:body/3/0/6/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5066:5069:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5142:5209:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5142:5149:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5150:5153:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5154:5166:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5176:5205:body/3/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5178:5194:body/3/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5178:5181:body/3/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5182:5194:body/3/0/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5206:5209:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5292:5335:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5292:5305:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5306:5335:body/1/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5308:5333:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5308:5311:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5312:5322:body/1/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5330:5333:body/1/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5420:5475:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5420:5433:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5434:5437:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5438:5447:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5448:5451:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5452:5462:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5472:5475:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5551:5597:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5551:5561:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5562:5565:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5566:5572:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5573:5576:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5577:5587:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5594:5597:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5677:5725:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5677:5689:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5690:5693:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5694:5700:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5701:5704:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5705:5715:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5722:5725:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5818:5915:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5818:5829:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5830:5833:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5834:5842:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5843:5846:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5847:5857:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5866:5911:body/5/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5868:5896:body/5/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5868:5871:body/5/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5872:5885:body/5/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5886:5889:body/5/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5890:5896:body/5/0/3/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5912:5915:body/6/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5994:6042:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5994:6004:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6005:6008:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6009:6018:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6019:6022:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6023:6029:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6039:6042:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6107:6119:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6107:6119:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6186:6199:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6186:6199:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6272:6288:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6272:6288:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6367:6386:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6367:6386:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6469:6526:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6469:6482:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6483:6486:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6487:6494:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6495:6498:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6499:6515:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6523:6526:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6594:6621:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6594:6600:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6601:6604:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6605:6611:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6618:6621:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6711:6783:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6711:6722:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6723:6726:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6727:6779:body/2/group", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6729:6771:body/2/0/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6729:6746:body/2/0/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6729:6735:body/2/0/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6736:6739:body/2/0/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6740:6746:body/2/0/0/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6761:6771:body/2/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6780:6783:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6838:6845:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6838:6845:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7008:7084:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7008:7022:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7023:7026:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7027:7037:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7050:7080:body/3/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7052:7066:body/3/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7052:7055:body/3/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7056:7066:body/3/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7081:7084:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7192:7199:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7192:7199:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7273:7287:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7273:7276:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7277:7287:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7407:7468:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7407:7419:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7434:7468:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7436:7452:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7436:7439:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7440:7452:body/1/0/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7554:7574:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7554:7574:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7556:7572:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7738:7923:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7738:7773:body/0/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7740:7753:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7781:7794:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7811:7832:body/2/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7813:7830:body/2/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7840:7861:body/3/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7842:7859:body/3/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7869:7884:body/4/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7871:7882:body/4/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7892:7923:body/5/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7894:7908:body/5/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7894:7897:body/5/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7898:7908:body/5/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7972:7987:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7972:7975:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7978:7981:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7984:7987:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8036:8053:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8036:8039:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8040:8053:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8111:8144:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8111:8114:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8115:8121:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8122:8140:body/2/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8124:8138:body/2/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8124:8127:body/2/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8128:8138:body/2/0/1/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8130:8136:body/2/0/1/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8141:8144:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8163:8178:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8163:8168:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8169:8178:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8171:8176:body/1/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8325:8445:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8325:8337:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8345:8360:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8368:8381:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8389:8401:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8409:8424:body/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8432:8445:body/5/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8535:8557:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8535:8538:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8539:8547:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8554:8557:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8647:8669:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8647:8650:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8651:8659:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8666:8669:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8760:8782:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8760:8763:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8764:8772:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8779:8782:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8882:8913:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8882:8889:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8890:8893:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8894:8900:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8910:8913:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9008:9014:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9008:9014:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9351:9391:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9351:9369:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9353:9367:body/0/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9353:9363:body/0/0/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9364:9367:body/0/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9381:9391:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9518:9559:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9518:9527:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9528:9531:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9532:9542:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9556:9559:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9663:9706:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9663:9673:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9681:9706:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9683:9697:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9683:9686:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9687:9697:body/1/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9764:9774:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9764:9774:body/0/ruleRef"];
const BODIES:&[&str]=&["expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1326:1367:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1507:1685:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1785:1821:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1860:1903:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2084:2117:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2215:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2357:2396:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2455:2717:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2855:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2958:2990:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3100:3135:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3270:3314:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3424:3472:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3558:3583:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3681:3712:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3775:3780:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3843:3848:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3917:3924:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4199:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4255:4767:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4822:4829:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4927:5069:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5142:5209:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5292:5335:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5420:5475:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5551:5597:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5677:5725:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5818:5915:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5994:6042:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6107:6119:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6186:6199:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6272:6288:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6367:6386:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6469:6526:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6594:6621:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6711:6783:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6838:6845:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7008:7084:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7192:7199:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7273:7287:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7407:7468:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7554:7574:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7738:7923:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7972:7987:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8036:8053:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8111:8144:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8163:8178:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8325:8445:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8535:8557:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8647:8669:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8760:8782:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8882:8913:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9008:9014:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9351:9391:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9518:9559:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9663:9706:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9764:9774:body/seq"];
const SITES:&[(&str,&str)]=&[("expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/0/repeat/capture/0", "grammars"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1336:1346:body/1/tokenRef/capture/0", "alias"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1361:1367:body/3/tokenRef/capture/0", "path"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1517:1527:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1550:1564:body/3/repeat/capture/0", "imports"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1584:1601:body/4/repeat/capture/0", "settings"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1619:1632:body/5/repeat/capture/0", "tokens"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1652:1664:body/6/repeat/capture/0", "rules"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1789:1799:body/1/tokenRef/capture/0", "key"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1809:1821:body/3/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/0/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2088:2104:body/1/repeat/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2199:body/0/tokenRef/capture/0", "key"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2209:2215:body/2/tokenRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2365:2375:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2396:body/3/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2833:2839:body/2/tokenRef/capture/0", "terminator"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2973:2979:body/2/tokenRef/capture/0", "chars"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3116:3122:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3295:3301:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3441:3447:body/2/tokenRef/capture/0", "min"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3457:3463:body/4/tokenRef/capture/0", "max"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3567:3573:body/2/tokenRef/capture/0", "word"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3693:3699:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4129:body/0/repeat/capture/0", "annotations"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4148:4158:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4176:4184:body/3/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4949:4959:body/2/tokenRef/capture/0", "className"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5001:5011:body/3/0/4/tokenRef/capture/0", "paramNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5030:5040:body/3/0/5/0/1/tokenRef/capture/0", "paramNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5154:5166:body/2/ruleRef/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5182:5194:body/3/0/1/ruleRef/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5312:5322:body/1/0/1/tokenRef/capture/0", "style"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5452:5462:body/4/tokenRef/capture/0", "profile"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5577:5587:body/4/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5705:5715:body/4/tokenRef/capture/0", "mode"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5847:5857:body/4/tokenRef/capture/0", "symbol"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5890:5896:body/5/0/3/tokenRef/capture/0", "description"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6023:6029:body/4/tokenRef/capture/0", "context"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6499:6515:body/4/tokenRef/capture/0", "level"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6605:6611:body/2/tokenRef/capture/0", "text"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6740:6746:body/2/0/0/2/tokenRef/capture/0", "syncTokens"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6761:6771:body/2/0/1/tokenRef/capture/0", "mode"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7027:7037:body/2/tokenRef/capture/0", "fieldNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7056:7066:body/3/0/1/tokenRef/capture/0", "fieldNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7277:7287:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7407:7419:body/0/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7440:7452:body/1/0/1/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7554:7574:body/0/repeat/capture/0", "elements"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7740:7753:body/0/0/ruleRef/capture/0", "typeofConstraint"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7781:7794:body/1/ruleRef/capture/0", "element"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7898:7908:body/5/0/1/tokenRef/capture/0", "captureName"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8539:8547:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8651:8659:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8764:8772:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8894:8900:body/2/tokenRef/capture/0", "message"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9008:9014:body/0/tokenRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9351:9369:body/0/repeat/capture/0", "namespace"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9381:9391:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9532:9542:body/2/tokenRef/capture/0", "captureName"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9663:9673:body/0/tokenRef/capture/0", "parts"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9687:9697:body/1/0/1/tokenRef/capture/0", "parts")];
impl<const DIAG: bool> Session<'_, DIAG> {
fn build_values(&mut self,root:EventId)->Result<Vec<Ast>,String> {let mut out=Vec::new();self.build_values_into(root,&mut out)?;Ok(out)}
#[inline(never)] fn build_values_into(&mut self,root:EventId,out:&mut Vec<Ast>)->Result<(),String> {let frame=0u8;let address=std::ptr::addr_of!(frame) as usize;let anchor=*self.mapping_anchor.get_or_insert(address);if self.mapping_depth>=self.options.limits.mapping_depth || anchor.abs_diff(address)>256*1024 {return Err("maximum mapping depth exceeded".into());}self.mapping_depth+=1;let result=self.build_values_inner(root,out);self.mapping_depth-=1;result}
fn build_values_inner(&mut self,mut root:EventId,out:&mut Vec<Ast>)->Result<(),String> {
// 大半の呼出しは Join を含まない 1 本の枝（P4 実測: complex-x64 で 10,883 回すべてが 1 event）。
// その場合は pool から stack を借りずに降りる。
loop {match self.arena[root.0] {
Event::Capture{child,..}=>root=child,
Event::Values{child,span,wrap}=>{let start=out.len();self.build_values_into(child,out)?;if wrap && (matches!(&out[start..],[Ast::Text(_)]) || (out.len()==start && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {let text=self.semantic_text(child,span);out.truncate(start);out.push(Ast::Text(text));}return Ok(());},
Event::Rule{rule,span,child,caps}=>return self.build_rule(rule,caps,span,child,out),
Event::Join(..)=>break,
_=>return Ok(()),}}
let mut stack=self.take_stack();stack.push((root,false));while let Some((id,_))=stack.pop() {match self.arena[id.0] {Event::Join(a,b)=>{stack.push((b,false));stack.push((a,false));},Event::Capture{child,..}=>stack.push((child,false)),Event::Values{child,span,wrap}=>{let start=out.len();self.build_values_into(child,out)?;if wrap && (matches!(&out[start..],[Ast::Text(_)]) || (out.len()==start && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {let text=self.semantic_text(child,span);out.truncate(start);out.push(Ast::Text(text));}},Event::Rule{rule,span,child,caps}=>self.build_rule(rule,caps,span,child,out)?,_=>{}}}self.give_stack(stack);Ok(())}
fn leaf(&mut self,ty:usize,span:Span,text:String)->Result<Ast,String> {let text=self.remember_text(text,span);let span=self.span(span);let _=(span,text);Err(format!("unknown leaf type {ty}"))}
fn build_rule(&mut self,rule:usize,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);match rule {
0=>self.map_rule_0(caps,span,child,out),
1=>self.map_rule_1(caps,span,child,out),
2=>self.map_rule_2(caps,span,child,out),
3=>self.map_rule_3(caps,span,child,out),
4=>self.map_rule_4(caps,span,child,out),
5=>self.map_rule_5(caps,span,child,out),
6=>self.map_rule_6(caps,span,child,out),
7=>self.map_rule_7(caps,span,child,out),
8=>self.map_rule_8(caps,span,child,out),
9=>self.map_rule_9(caps,span,child,out),
10=>self.map_rule_10(caps,span,child,out),
11=>self.map_rule_11(caps,span,child,out),
12=>self.map_rule_12(caps,span,child,out),
13=>self.map_rule_13(caps,span,child,out),
14=>self.map_rule_14(caps,span,child,out),
15=>self.map_rule_15(caps,span,child,out),
16=>self.map_rule_16(caps,span,child,out),
17=>self.map_rule_17(caps,span,child,out),
18=>self.map_rule_18(caps,span,child,out),
19=>self.map_rule_19(caps,span,child,out),
20=>self.map_rule_20(caps,span,child,out),
21=>self.map_rule_21(caps,span,child,out),
22=>self.map_rule_22(caps,span,child,out),
23=>self.map_rule_23(caps,span,child,out),
24=>self.map_rule_24(caps,span,child,out),
25=>self.map_rule_25(caps,span,child,out),
26=>self.map_rule_26(caps,span,child,out),
27=>self.map_rule_27(caps,span,child,out),
28=>self.map_rule_28(caps,span,child,out),
29=>self.map_rule_29(caps,span,child,out),
30=>self.map_rule_30(caps,span,child,out),
31=>self.map_rule_31(caps,span,child,out),
32=>self.map_rule_32(caps,span,child,out),
33=>self.map_rule_33(caps,span,child,out),
34=>self.map_rule_34(caps,span,child,out),
35=>self.map_rule_35(caps,span,child,out),
36=>self.map_rule_36(caps,span,child,out),
37=>self.map_rule_37(caps,span,child,out),
38=>self.map_rule_38(caps,span,child,out),
39=>self.map_rule_39(caps,span,child,out),
40=>self.map_rule_40(caps,span,child,out),
41=>self.map_rule_41(caps,span,child,out),
42=>self.map_rule_42(caps,span,child,out),
43=>self.map_rule_43(caps,span,child,out),
44=>self.map_rule_44(caps,span,child,out),
45=>self.map_rule_45(caps,span,child,out),
46=>self.map_rule_46(caps,span,child,out),
47=>self.map_rule_47(caps,span,child,out),
48=>self.map_rule_48(caps,span,child,out),
49=>self.map_rule_49(caps,span,child,out),
50=>self.map_rule_50(caps,span,child,out),
51=>self.map_rule_51(caps,span,child,out),
52=>self.map_rule_52(caps,span,child,out),
53=>self.map_rule_53(caps,span,child,out),
54=>self.map_rule_54(caps,span,child,out),
55=>self.map_rule_55(caps,span,child,out),
56=>self.map_rule_56(caps,span,child,out),
57=>self.map_rule_57(caps,span,child,out),
58=>self.map_rule_58(caps,span,child,out),
_=>Err(format!("unknown rule {rule}"))}}
fn map_rule_0(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=Vec::new();self.node_values(&caps,&[0],None,&mut f0)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_GrammarDecl(_))) {return Err("grammars: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[0],"UBNFFile",span);out.push(Ast::g_UBNFAST_2e_UBNFFile(g_UBNFAST_2e_UBNFFile{span,node_id,
g_grammars:f0,
}));Ok(())}}
fn map_rule_1(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[1],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[1]) {return Ok(());}
if t0.len()!=1 {return Err("alias requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
let mut t1=self.take_texts();self.text_values(&caps,&[2],&mut t1);if t1.is_empty() && self.missing_field(&caps,&[2]) {return Ok(());}
if t1.len()!=1 {return Err("path requires one value".into());}let f1=t1.pop().unwrap();self.give_texts(t1);
let f1:String=f1;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[1],"ImportDecl",span);out.push(Ast::g_UBNFAST_2e_ImportDecl(g_UBNFAST_2e_ImportDecl{span,node_id,
g_alias:f0,
g_path:f1,
}));Ok(())}}
fn map_rule_2(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[3],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[3]) {return Ok(());}
if t0.len()!=1 {return Err("name requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
let mut f1=Vec::new();self.node_values(&caps,&[4],None,&mut f1)?;
if !f1.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ImportDecl(_))) {return Err("imports: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let mut f2=Vec::new();self.node_values(&caps,&[5],None,&mut f2)?;
if !f2.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_GlobalSetting(_))) {return Err("settings: mapped value type mismatch".into());}
let f2:Vec<Ast>=f2;
let mut f3=Vec::new();self.node_values(&caps,&[6],None,&mut f3)?;
if !f3.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_TokenDecl(_))) {return Err("tokens: mapped value type mismatch".into());}
let f3:Vec<Ast>=f3;
let mut f4=Vec::new();self.node_values(&caps,&[7],None,&mut f4)?;
if !f4.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_RuleDecl(_))) {return Err("rules: mapped value type mismatch".into());}
let f4:Vec<Ast>=f4;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[2],"GrammarDecl",span);out.push(Ast::g_UBNFAST_2e_GrammarDecl(g_UBNFAST_2e_GrammarDecl{span,node_id,
g_name:f0,
g_imports:f1,
g_settings:f2,
g_tokens:f3,
g_rules:f4,
}));Ok(())}}
fn map_rule_3(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[8],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[8]) {return Ok(());}
if t0.len()!=1 {return Err("key requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
let mut v1=self.take_values();self.node_values(&caps,&[9],None,&mut v1)?;
if !v1.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_BlockSettingValue(_)) || matches!(value,Ast::g_UBNFAST_2e_StringSettingValue(_))) {return Err("value: mapped value type mismatch".into());}
if v1.is_empty() && self.missing_field(&caps,&[9]) {return Ok(());}
if v1.len()!=1 {return Err("value requires one node".into());}let f1=Box::new(v1.pop().unwrap());self.give_values(v1);
let f1:Box<Ast>=f1;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[3],"GlobalSetting",span);out.push(Ast::g_UBNFAST_2e_GlobalSetting(g_UBNFAST_2e_GlobalSetting{span,node_id,
g_key:f0,
g_value:f1,
}));Ok(())}}
fn map_rule_4(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
fn map_rule_5(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[10],None,&mut v0)?;
if !v0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_DottedIdentifier(_))) {return Err("value: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[10]) {return Ok(());}
if v0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(v0.pop().unwrap());self.give_values(v0);
let f0:Box<Ast>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[5],"StringSettingValue",span);out.push(Ast::g_UBNFAST_2e_StringSettingValue(g_UBNFAST_2e_StringSettingValue{span,node_id,
g_value:f0,
}));Ok(())}}
fn map_rule_6(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=Vec::new();self.node_values(&caps,&[11],None,&mut f0)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_KeyValuePair(_))) {return Err("entries: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[6],"BlockSettingValue",span);out.push(Ast::g_UBNFAST_2e_BlockSettingValue(g_UBNFAST_2e_BlockSettingValue{span,node_id,
g_entries:f0,
}));Ok(())}}
fn map_rule_7(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[12],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[12]) {return Ok(());}
if t0.len()!=1 {return Err("key requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
let mut t1=self.take_texts();self.text_values(&caps,&[13],&mut t1);if t1.is_empty() && self.missing_field(&caps,&[13]) {return Ok(());}
if t1.len()!=1 {return Err("value requires one value".into());}let f1=t1.pop().unwrap();self.give_texts(t1);
let f1:String=f1;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[7],"KeyValuePair",span);out.push(Ast::g_UBNFAST_2e_KeyValuePair(g_UBNFAST_2e_KeyValuePair{span,node_id,
g_key:f0,
g_value:f1,
}));Ok(())}}
fn map_rule_8(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[14],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[14]) {return Ok(());}
if t0.len()!=1 {return Err("name requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
let mut v1=self.take_values();self.node_values(&caps,&[15],None,&mut v1)?;
if !v1.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_AnyExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_CIExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_CharRangeExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_DottedIdentifier(_)) || matches!(value,Ast::g_UBNFAST_2e_EmptyExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_EofExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_LookaheadExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_NegationExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_NegativeLookaheadExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_RegexExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_UntilExpression(_))) {return Err("value: mapped value type mismatch".into());}
if v1.is_empty() && self.missing_field(&caps,&[15]) {return Ok(());}
if v1.len()!=1 {return Err("value requires one node".into());}let f1=Box::new(v1.pop().unwrap());self.give_values(v1);
let f1:Box<Ast>=f1;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[8],"TokenDecl",span);out.push(Ast::g_UBNFAST_2e_TokenDecl(g_UBNFAST_2e_TokenDecl{span,node_id,
g_name:f0,
g_value:f1,
}));Ok(())}}
fn map_rule_9(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
fn map_rule_10(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[16],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[16]) {return Ok(());}
if t0.len()!=1 {return Err("terminator requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[10],"UntilExpression",span);out.push(Ast::g_UBNFAST_2e_UntilExpression(g_UBNFAST_2e_UntilExpression{span,node_id,
g_terminator:f0,
}));Ok(())}}
fn map_rule_11(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[17],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[17]) {return Ok(());}
if t0.len()!=1 {return Err("chars requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[11],"NegationExpression",span);out.push(Ast::g_UBNFAST_2e_NegationExpression(g_UBNFAST_2e_NegationExpression{span,node_id,
g_chars:f0,
}));Ok(())}}
fn map_rule_12(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[18],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[18]) {return Ok(());}
if t0.len()!=1 {return Err("pattern requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[12],"LookaheadExpression",span);out.push(Ast::g_UBNFAST_2e_LookaheadExpression(g_UBNFAST_2e_LookaheadExpression{span,node_id,
g_pattern:f0,
}));Ok(())}}
fn map_rule_13(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[19],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[19]) {return Ok(());}
if t0.len()!=1 {return Err("pattern requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[13],"NegativeLookaheadExpression",span);out.push(Ast::g_UBNFAST_2e_NegativeLookaheadExpression(g_UBNFAST_2e_NegativeLookaheadExpression{span,node_id,
g_pattern:f0,
}));Ok(())}}
fn map_rule_14(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[20],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[20]) {return Ok(());}
if t0.len()!=1 {return Err("min requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
let mut t1=self.take_texts();self.text_values(&caps,&[21],&mut t1);if t1.is_empty() && self.missing_field(&caps,&[21]) {return Ok(());}
if t1.len()!=1 {return Err("max requires one value".into());}let f1=t1.pop().unwrap();self.give_texts(t1);
let f1:String=f1;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[14],"CharRangeExpression",span);out.push(Ast::g_UBNFAST_2e_CharRangeExpression(g_UBNFAST_2e_CharRangeExpression{span,node_id,
g_min:f0,
g_max:f1,
}));Ok(())}}
fn map_rule_15(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[22],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[22]) {return Ok(());}
if t0.len()!=1 {return Err("word requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[15],"CIExpression",span);out.push(Ast::g_UBNFAST_2e_CIExpression(g_UBNFAST_2e_CIExpression{span,node_id,
g_word:f0,
}));Ok(())}}
fn map_rule_16(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[23],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[23]) {return Ok(());}
if t0.len()!=1 {return Err("pattern requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[16],"RegexExpression",span);out.push(Ast::g_UBNFAST_2e_RegexExpression(g_UBNFAST_2e_RegexExpression{span,node_id,
g_pattern:f0,
}));Ok(())}}
fn map_rule_17(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[17],"AnyExpression",span);out.push(Ast::g_UBNFAST_2e_AnyExpression(g_UBNFAST_2e_AnyExpression{span,node_id,
}));Ok(())}}
fn map_rule_18(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[18],"EofExpression",span);out.push(Ast::g_UBNFAST_2e_EofExpression(g_UBNFAST_2e_EofExpression{span,node_id,
}));Ok(())}}
fn map_rule_19(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[19],"EmptyExpression",span);out.push(Ast::g_UBNFAST_2e_EmptyExpression(g_UBNFAST_2e_EmptyExpression{span,node_id,
}));Ok(())}}
fn map_rule_20(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=Vec::new();self.node_values(&caps,&[24],None,&mut f0)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_BackrefAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_CatalogAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_CommonFieldAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_DeclaresAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_DocAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_EnumAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_EvalAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_InterleaveAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_LeftAssocAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_LongestChoiceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_MappingAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_PrecedenceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_RecoveryAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_RightAssocAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_RootAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_ScopeTreeAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_SimpleAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_SkipAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_WhitespaceAnnotation(_))) {return Err("annotations: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let mut t1=self.take_texts();self.text_values(&caps,&[25],&mut t1);if t1.is_empty() && self.missing_field(&caps,&[25]) {return Ok(());}
if t1.len()!=1 {return Err("name requires one value".into());}let f1=t1.pop().unwrap();self.give_texts(t1);
let f1:String=f1;
let mut v2=self.take_values();self.node_values(&caps,&[26],None,&mut v2)?;
if !v2.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if v2.is_empty() && self.missing_field(&caps,&[26]) {return Ok(());}
if v2.len()!=1 {return Err("body requires one node".into());}let f2=Box::new(v2.pop().unwrap());self.give_values(v2);
let f2:Box<Ast>=f2;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[20],"RuleDecl",span);out.push(Ast::g_UBNFAST_2e_RuleDecl(g_UBNFAST_2e_RuleDecl{span,node_id,
g_annotations:f0,
g_name:f1,
g_body:f2,
}));Ok(())}}
fn map_rule_21(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
fn map_rule_22(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[22],"RootAnnotation",span);out.push(Ast::g_UBNFAST_2e_RootAnnotation(g_UBNFAST_2e_RootAnnotation{span,node_id,
}));Ok(())}}
fn map_rule_23(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[27],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[27]) {return Ok(());}
if t0.len()!=1 {return Err("className requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
let mut f1=Vec::new();self.text_values(&caps,&[28, 29],&mut f1);
let f1:Vec<String>=f1;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[23],"MappingAnnotation",span);out.push(Ast::g_UBNFAST_2e_MappingAnnotation(g_UBNFAST_2e_MappingAnnotation{span,node_id,
g_className:f0,
g_paramNames:f1,
}));Ok(())}}
fn map_rule_24(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=Vec::new();self.node_values(&caps,&[30, 31],None,&mut f0)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_KeyValuePair(_))) {return Err("entries: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[24],"EvalAnnotation",span);out.push(Ast::g_UBNFAST_2e_EvalAnnotation(g_UBNFAST_2e_EvalAnnotation{span,node_id,
g_entries:f0,
}));Ok(())}}
fn map_rule_25(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[32],&mut t0);if t0.len()>1 {return Err("style requires at most one value".into());}let f0=t0.pop();self.give_texts(t0);
let f0:Option<String>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[25],"WhitespaceAnnotation",span);out.push(Ast::g_UBNFAST_2e_WhitespaceAnnotation(g_UBNFAST_2e_WhitespaceAnnotation{span,node_id,
g_style:f0,
}));Ok(())}}
fn map_rule_26(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[33],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[33]) {return Ok(());}
if t0.len()!=1 {return Err("profile requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[26],"InterleaveAnnotation",span);out.push(Ast::g_UBNFAST_2e_InterleaveAnnotation(g_UBNFAST_2e_InterleaveAnnotation{span,node_id,
g_profile:f0,
}));Ok(())}}
fn map_rule_27(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[34],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[34]) {return Ok(());}
if t0.len()!=1 {return Err("name requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[27],"BackrefAnnotation",span);out.push(Ast::g_UBNFAST_2e_BackrefAnnotation(g_UBNFAST_2e_BackrefAnnotation{span,node_id,
g_name:f0,
}));Ok(())}}
fn map_rule_28(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[35],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[35]) {return Ok(());}
if t0.len()!=1 {return Err("mode requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[28],"ScopeTreeAnnotation",span);out.push(Ast::g_UBNFAST_2e_ScopeTreeAnnotation(g_UBNFAST_2e_ScopeTreeAnnotation{span,node_id,
g_mode:f0,
}));Ok(())}}
fn map_rule_29(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[36],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[36]) {return Ok(());}
if t0.len()!=1 {return Err("symbol requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
let mut t1=self.take_texts();self.text_values(&caps,&[37],&mut t1);if t1.len()>1 {return Err("description requires at most one value".into());}let f1=t1.pop();self.give_texts(t1);
let f1:Option<String>=f1;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[29],"DeclaresAnnotation",span);out.push(Ast::g_UBNFAST_2e_DeclaresAnnotation(g_UBNFAST_2e_DeclaresAnnotation{span,node_id,
g_symbol:f0,
g_description:f1,
}));Ok(())}}
fn map_rule_30(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[38],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[38]) {return Ok(());}
if t0.len()!=1 {return Err("context requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[30],"CatalogAnnotation",span);out.push(Ast::g_UBNFAST_2e_CatalogAnnotation(g_UBNFAST_2e_CatalogAnnotation{span,node_id,
g_context:f0,
}));Ok(())}}
fn map_rule_31(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[31],"LeftAssocAnnotation",span);out.push(Ast::g_UBNFAST_2e_LeftAssocAnnotation(g_UBNFAST_2e_LeftAssocAnnotation{span,node_id,
}));Ok(())}}
fn map_rule_32(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[32],"RightAssocAnnotation",span);out.push(Ast::g_UBNFAST_2e_RightAssocAnnotation(g_UBNFAST_2e_RightAssocAnnotation{span,node_id,
}));Ok(())}}
fn map_rule_33(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[33],"LongestChoiceAnnotation",span);out.push(Ast::g_UBNFAST_2e_LongestChoiceAnnotation(g_UBNFAST_2e_LongestChoiceAnnotation{span,node_id,
}));Ok(())}}
fn map_rule_34(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[34],"PredictiveChoiceAnnotation",span);out.push(Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(g_UBNFAST_2e_PredictiveChoiceAnnotation{span,node_id,
}));Ok(())}}
fn map_rule_35(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[39],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[39]) {return Ok(());}
if t0.len()!=1 {return Err("level requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[35],"PrecedenceAnnotation",span);out.push(Ast::g_UBNFAST_2e_PrecedenceAnnotation(g_UBNFAST_2e_PrecedenceAnnotation{span,node_id,
g_level:f0,
}));Ok(())}}
fn map_rule_36(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[40],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[40]) {return Ok(());}
if t0.len()!=1 {return Err("text requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[36],"DocAnnotation",span);out.push(Ast::g_UBNFAST_2e_DocAnnotation(g_UBNFAST_2e_DocAnnotation{span,node_id,
g_text:f0,
}));Ok(())}}
fn map_rule_37(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[42],&mut t0);if t0.len()>1 {return Err("mode requires at most one value".into());}let f0=t0.pop();self.give_texts(t0);
let f0:Option<String>=f0;
let mut t1=self.take_texts();self.text_values(&caps,&[41],&mut t1);if t1.len()>1 {return Err("syncTokens requires at most one value".into());}let f1=t1.pop();self.give_texts(t1);
let f1:Option<String>=f1;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[37],"RecoveryAnnotation",span);out.push(Ast::g_UBNFAST_2e_RecoveryAnnotation(g_UBNFAST_2e_RecoveryAnnotation{span,node_id,
g_mode:f0,
g_syncTokens:f1,
}));Ok(())}}
fn map_rule_38(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[38],"SkipAnnotation",span);out.push(Ast::g_UBNFAST_2e_SkipAnnotation(g_UBNFAST_2e_SkipAnnotation{span,node_id,
}));Ok(())}}
fn map_rule_39(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=Vec::new();self.text_values(&caps,&[43, 44],&mut f0);
let f0:Vec<String>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[39],"CommonFieldAnnotation",span);out.push(Ast::g_UBNFAST_2e_CommonFieldAnnotation(g_UBNFAST_2e_CommonFieldAnnotation{span,node_id,
g_fieldNames:f0,
}));Ok(())}}
fn map_rule_40(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[40],"EnumAnnotation",span);out.push(Ast::g_UBNFAST_2e_EnumAnnotation(g_UBNFAST_2e_EnumAnnotation{span,node_id,
}));Ok(())}}
fn map_rule_41(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[45],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[45]) {return Ok(());}
if t0.len()!=1 {return Err("name requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[41],"SimpleAnnotation",span);out.push(Ast::g_UBNFAST_2e_SimpleAnnotation(g_UBNFAST_2e_SimpleAnnotation{span,node_id,
g_name:f0,
}));Ok(())}}
fn map_rule_42(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=Vec::new();self.node_values(&caps,&[46, 47],None,&mut f0)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_SequenceBody(_))) {return Err("alternatives: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[42],"ChoiceBody",span);out.push(Ast::g_UBNFAST_2e_ChoiceBody(g_UBNFAST_2e_ChoiceBody{span,node_id,
g_alternatives:f0,
}));Ok(())}}
fn map_rule_43(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=Vec::new();self.node_values(&caps,&[48],None,&mut f0)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_AnnotatedElement(_))) {return Err("elements: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[43],"SequenceBody",span);out.push(Ast::g_UBNFAST_2e_SequenceBody(g_UBNFAST_2e_SequenceBody{span,node_id,
g_elements:f0,
}));Ok(())}}
fn map_rule_44(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[50],None,&mut v0)?;
if !v0.iter().all(|value|matches!(value,Ast::Text(_)) || matches!(value,Ast::g_UBNFAST_2e_ErrorElement(_)) || matches!(value,Ast::g_UBNFAST_2e_GroupElement(_)) || matches!(value,Ast::g_UBNFAST_2e_OptionalElement(_)) || matches!(value,Ast::g_UBNFAST_2e_RepeatElement(_)) || matches!(value,Ast::g_UBNFAST_2e_TerminalElement(_))) {return Err("element: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[50]) {return Ok(());}
if v0.len()!=1 {return Err("element requires one node".into());}let f0=Box::new(v0.pop().unwrap());self.give_values(v0);
let f0:Box<Ast>=f0;
let mut t1=self.take_texts();self.text_values(&caps,&[51],&mut t1);if t1.len()>1 {return Err("captureName requires at most one value".into());}let f1=t1.pop();self.give_texts(t1);
let f1:Option<String>=f1;
let mut v2=self.take_values();self.node_values(&caps,&[49],None,&mut v2)?;
if !v2.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_TypeofElement(_))) {return Err("typeofConstraint: mapped value type mismatch".into());}
if v2.len()>1 {return Err("typeofConstraint requires at most one node".into());}let f2=v2.pop().map(Box::new);self.give_values(v2);
let f2:Option<Box<Ast>>=f2;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[44],"AnnotatedElement",span);out.push(Ast::g_UBNFAST_2e_AnnotatedElement(g_UBNFAST_2e_AnnotatedElement{span,node_id,
g_element:f0,
g_captureName:f1,
g_typeofConstraint:f2,
}));Ok(())}}
fn map_rule_45(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
fn map_rule_46(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
fn map_rule_47(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
fn map_rule_48(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
fn map_rule_49(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
fn map_rule_50(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[52],None,&mut v0)?;
if !v0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[52]) {return Ok(());}
if v0.len()!=1 {return Err("body requires one node".into());}let f0=Box::new(v0.pop().unwrap());self.give_values(v0);
let f0:Box<Ast>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[50],"GroupElement",span);out.push(Ast::g_UBNFAST_2e_GroupElement(g_UBNFAST_2e_GroupElement{span,node_id,
g_body:f0,
}));Ok(())}}
fn map_rule_51(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[53],None,&mut v0)?;
if !v0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[53]) {return Ok(());}
if v0.len()!=1 {return Err("body requires one node".into());}let f0=Box::new(v0.pop().unwrap());self.give_values(v0);
let f0:Box<Ast>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[51],"OptionalElement",span);out.push(Ast::g_UBNFAST_2e_OptionalElement(g_UBNFAST_2e_OptionalElement{span,node_id,
g_body:f0,
}));Ok(())}}
fn map_rule_52(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut v0=self.take_values();self.node_values(&caps,&[54],None,&mut v0)?;
if !v0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if v0.is_empty() && self.missing_field(&caps,&[54]) {return Ok(());}
if v0.len()!=1 {return Err("body requires one node".into());}let f0=Box::new(v0.pop().unwrap());self.give_values(v0);
let f0:Box<Ast>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[52],"RepeatElement",span);out.push(Ast::g_UBNFAST_2e_RepeatElement(g_UBNFAST_2e_RepeatElement{span,node_id,
g_body:f0,
}));Ok(())}}
fn map_rule_53(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[55],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[55]) {return Ok(());}
if t0.len()!=1 {return Err("message requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[53],"ErrorElement",span);out.push(Ast::g_UBNFAST_2e_ErrorElement(g_UBNFAST_2e_ErrorElement{span,node_id,
g_message:f0,
}));Ok(())}}
fn map_rule_54(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[56],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[56]) {return Ok(());}
if t0.len()!=1 {return Err("value requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[54],"TerminalElement",span);out.push(Ast::g_UBNFAST_2e_TerminalElement(g_UBNFAST_2e_TerminalElement{span,node_id,
g_value:f0,
}));Ok(())}}
fn map_rule_55(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
fn map_rule_56(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut t0=self.take_texts();self.text_values(&caps,&[59],&mut t0);if t0.is_empty() && self.missing_field(&caps,&[59]) {return Ok(());}
if t0.len()!=1 {return Err("captureName requires one value".into());}let f0=t0.pop().unwrap();self.give_texts(t0);
let f0:String=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[56],"TypeofElement",span);out.push(Ast::g_UBNFAST_2e_TypeofElement(g_UBNFAST_2e_TypeofElement{span,node_id,
g_captureName:f0,
}));Ok(())}}
fn map_rule_57(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let mut slots=self.take_captures();self.rule_captures(caps,&mut slots);let caps=slots;
let mut f0=Vec::new();self.text_values(&caps,&[60, 61],&mut f0);
let f0:Vec<String>=f0;
self.give_captures(caps);let span=self.span(span);let node_id=self.node(RULES[57],"DottedIdentifier",span);out.push(Ast::g_UBNFAST_2e_DottedIdentifier(g_UBNFAST_2e_DottedIdentifier{span,node_id,
g_parts:f0,
}));Ok(())}}
fn map_rule_58(&mut self,caps:(u32,u32),span:Span,child:EventId,out:&mut Vec<Ast>)->Result<(),String> {let _=(caps,span,child,&out);{let start=out.len();self.build_values_into(child,out)?;if out.len()==start && !self.has_recovery(child) && !self.has_value_group(child) {let text=self.semantic_text(child,span);out.push(Ast::Text(text));}Ok(())}}
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
    let mut captures = Vec::with_capacity(estimate);
    let mut lexical = Vec::new();
    let mut tokens = Vec::new();
    let build_ast = self.options.build_ast;
    let want_lexical = self.options.lexical;
    // 公開する出現表は opt-in（D-035）。AST の直下 capture 表（`pending` / `caps_flat`）は
    // `build_ast` のままで、こちらの flag には依らない。走査の骨組み（rule ごとの開始位置）は
    // どちらかが要るときに積む。
    let want_occurrences = self.options.occurrences || want_lexical;
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
    while let Some((id, parent, done, current)) = stack.pop() {
        let parent_index = (parent != NONE).then_some(parent as usize);
        let id = EventId(id as usize);
        match self.arena[id.0] {
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
                            let start = cap_start as usize;
                            let offset = caps_flat.len();
                            caps_flat.extend_from_slice(&pending[start..]);
                            pending.truncate(start);
                            if let Event::Rule { caps, .. } = &mut self.arena[id.0] {
                                *caps = (offset as u32, (caps_flat.len() - offset) as u32);
                            }
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
    self.caps_flat = caps_flat;
    self.caps_pending = pending;
    (captures, lexical, tokens)
}
fn recovery_occurrences(&self, root: EventId) -> (Vec<Recovery>, Vec<Diagnostic>) {
    let mut recoveries: Vec<Recovery> = Vec::new();
    let mut diagnostics = Vec::new();
    let mut stack = vec![(root, None)];
    let mut capture_order = 0;
    while let Some((id, done)) = stack.pop() {
        match self.arena[id.0] {
            Event::Join(a, b) => {
                stack.push((b, None));
                stack.push((a, None));
            }
            Event::Rule { child, .. } | Event::Values { child, .. } => stack.push((child, None)),
            Event::Capture { child, .. } => {
                if let Some(start) = done {
                    // `capture_occurrence_ids` は出現表を返すかに依らない観測なので、
                    // 番号付けは AST / lexical のときそのまま数える（D-035 の対象外）。
                    if self.options.build_ast || self.options.lexical {
                        for r in &mut recoveries[start..] {
                            r.capture_occurrence_ids.push(capture_order);
                        }
                        capture_order += 1;
                    }
                } else {
                    stack.push((id, Some(recoveries.len())));
                    stack.push((child, None));
                }
            }
            Event::Recovery {
                rule,
                mode,
                span,
                sync_span,
                diag,
            } => {
                let (pos, expected, path) =
                    self.diag.summary(diag).unwrap_or((span[0], vec![], vec![]));
                let rule_path: Vec<_> = path.into_iter().map(|r| RULES[r]).collect();
                let span = self.span(span);
                diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Recovery,
                    offset_cp: span[0],
                    length_cp: span[1] - span[0],
                    farthest_cp: self.cp(pos),
                    expected: expected.clone(),
                    farthest_expected: expected,
                    deepest_rule: rule_path.last().copied(),
                    rule_path,
                    message: Some("recovered input".into()),
                    recovery_id: Some(recoveries.len()),
                });
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
            && (self.options.build_ast || self.options.occurrences || self.options.lexical)
        {
            self.occurrences(step.events)
        } else {
            (vec![], vec![], vec![])
        };
    let ast = if ok && self.options.build_ast {
        match self.build_values(step.events).and_then(|mut nodes| {
            if nodes.len() == 1 {
                Ok(Some(nodes.remove(0)))
            } else if nodes.is_empty() {
                // D-028: entry が AST 値を作らないのは mapping 失敗ではない（ast=null / mappingError=null）。
                Ok(None)
            } else {
                Err("entry produced more than one AST value".into())
            }
        }) {
            Ok(ast) => ast,
            Err(message) => {
                self.node_spans.clear();
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
                None
            }
        }
    } else {
        None
    };
    self.statistics.memo_entries = self.memo.len();
    self.statistics.memo_max_probe = self
        .memo
        .max_probe_len()
        .max(self.trivia_cache.max_probe_len());
    self.statistics.recipes = self.arena.len() - 1;
    self.statistics.ast_nodes = self.node_spans.len();
    self.statistics.lexical_runs = self.lex.runs;
    self.statistics.lexical_probes = self.lex.probes;
    self.statistics.trivia_skip_hits = self.trivia_skip.hits.get();
    self.statistics.trivia_skip_misses = self.trivia_skip.misses.get();
    // value span は要求されたときだけ集める（D-035）。AST 自体は先に完成している。
    let want_value_spans = self.options.value_spans && ast.is_some();
    let mut value_spans = Vec::with_capacity(if want_value_spans {
        self.node_spans.len() + self.text_spans.len()
    } else {
        0
    });
    if want_value_spans {
        if let Some(ast) = &ast {
            let mut path = String::with_capacity(128);
            ast.collect(&mut path, &self.text_spans, &mut value_spans);
        }
    }
    let result = ParseResult {
        value_spans,
        ok,
        consumed_cp,
        matched_cp,
        ast,
        captures,
        lexical,
        tokens,
        node_spans: std::mem::take(&mut self.node_spans),
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
// value span の path は共有バッファに push / truncate で組み立てる（field ごとの format! を避ける）。
// 添字も `to_string` ではなく共有バッファへ直接書く（要素ごとの確保を避ける）。
use std::fmt::Write as _;
trait CollectValueSpans {
    fn collect(
        &self,
        path: &mut String,
        texts: &TextSpans,
        out: &mut Vec<ValueSpan>,
    );
}
impl CollectValueSpans for String {
    fn collect(
        &self,
        path: &mut String,
        texts: &TextSpans,
        out: &mut Vec<ValueSpan>,
    ) {
        if let Some(span) = texts.get(&(self.as_ptr() as usize)) {
            out.push(ValueSpan {
                path: path.clone(),
                span: *span,
                text: None,
            });
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Vec<T> {
    fn collect(
        &self,
        path: &mut String,
        texts: &TextSpans,
        out: &mut Vec<ValueSpan>,
    ) {
        for (i, value) in self.iter().enumerate() {
            let len = path.len();
            let _ = write!(path, "/{i}");
            value.collect(path, texts, out);
            path.truncate(len);
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Option<T> {
    fn collect(
        &self,
        path: &mut String,
        texts: &TextSpans,
        out: &mut Vec<ValueSpan>,
    ) {
        if let Some(value) = self {
            value.collect(path, texts, out);
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Box<T> {
    fn collect(
        &self,
        path: &mut String,
        texts: &TextSpans,
        out: &mut Vec<ValueSpan>,
    ) {
        self.as_ref().collect(path, texts, out);
    }
}
impl CollectValueSpans for Ast { fn collect(&self,path:&mut String,texts:&TextSpans,out:&mut Vec<ValueSpan>) { match self { Self::Text(text)=>{if let Some(span)=texts.get(&(text.as_ptr() as usize)) {out.push(ValueSpan{path:path.clone(),span:*span,text:Some(text.clone())});}},Self::Null=>{},
Self::g_UBNFAST_2e_UBNFFile(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/grammars");node.g_grammars.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_ImportDecl(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/alias");node.g_alias.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/path");node.g_path.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_GrammarDecl(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/name");node.g_name.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/imports");node.g_imports.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/settings");node.g_settings.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/tokens");node.g_tokens.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/rules");node.g_rules.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_GlobalSetting(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/key");node.g_key.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/value");node.g_value.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_StringSettingValue(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/value");node.g_value.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_BlockSettingValue(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/entries");node.g_entries.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_KeyValuePair(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/key");node.g_key.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/value");node.g_value.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_TokenDecl(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/name");node.g_name.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/value");node.g_value.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_UntilExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/terminator");node.g_terminator.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_NegationExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/chars");node.g_chars.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_LookaheadExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/pattern");node.g_pattern.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_NegativeLookaheadExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/pattern");node.g_pattern.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_CharRangeExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/min");node.g_min.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/max");node.g_max.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_CIExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/word");node.g_word.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_RegexExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/pattern");node.g_pattern.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_AnyExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_EofExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_EmptyExpression(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_RuleDecl(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/annotations");node.g_annotations.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/name");node.g_name.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/body");node.g_body.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_RootAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_MappingAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/className");node.g_className.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/paramNames");node.g_paramNames.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_EvalAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/entries");node.g_entries.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_WhitespaceAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/style");node.g_style.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_InterleaveAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/profile");node.g_profile.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_BackrefAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/name");node.g_name.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_ScopeTreeAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/mode");node.g_mode.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_DeclaresAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/symbol");node.g_symbol.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/description");node.g_description.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_CatalogAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/context");node.g_context.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_LeftAssocAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_RightAssocAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_LongestChoiceAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_PredictiveChoiceAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_PrecedenceAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/level");node.g_level.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_DocAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/text");node.g_text.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_RecoveryAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/mode");node.g_mode.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/syncTokens");node.g_syncTokens.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_SkipAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_CommonFieldAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/fieldNames");node.g_fieldNames.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_EnumAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
},
Self::g_UBNFAST_2e_SimpleAnnotation(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/name");node.g_name.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_ChoiceBody(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/alternatives");node.g_alternatives.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_SequenceBody(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/elements");node.g_elements.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_AnnotatedElement(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/element");node.g_element.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/captureName");node.g_captureName.collect(path,texts,out);path.truncate(len);}
{let len=path.len();path.push_str("/fields/typeofConstraint");node.g_typeofConstraint.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_GroupElement(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/body");node.g_body.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_OptionalElement(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/body");node.g_body.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_RepeatElement(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/body");node.g_body.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_ErrorElement(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/message");node.g_message.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_TerminalElement(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/value");node.g_value.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_TypeofElement(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/captureName");node.g_captureName.collect(path,texts,out);path.truncate(len);}
},
Self::g_UBNFAST_2e_DottedIdentifier(node)=>{out.push(ValueSpan{path:path.clone(),span:node.span,text:None});
{let len=path.len();path.push_str("/fields/parts");node.g_parts.collect(path,texts,out);path.truncate(len);}
},
}}}
