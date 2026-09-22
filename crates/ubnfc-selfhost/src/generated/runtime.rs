use super::api::*;
use super::rt::{
    diag::{Diag, Diagnostics, DisplayDiagnostics, DisplaySnapshot},
    input::Input,
    memo::{Key, Memo},
    scope::{Checkpoint, ScopeStore},
};

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
#[derive(Clone, Copy)]
pub(crate) struct Mark {
    scope: Checkpoint,
    effects: usize,
}
#[derive(Clone)]
struct Effect {
    value: ScanEffect,
    mode: &'static str,
    offset_cp: usize,
    length_cp: usize,
}
pub(crate) struct Session<'a> {
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
    effects: Vec<Effect>,
    saved_effects: Vec<Effect>,
    pub statistics: Statistics,
    pub node_spans: Vec<NodeSpan>,
    // Allocation identity is used only while mapping. finish resolves it into
    // owned AST paths, so ParseResult remains valid after moves and clones.
    pub text_spans: std::collections::HashMap<usize, Span>,
    pub depth: usize,
    stack_anchor: Option<usize>,
    depth_failure: Option<usize>,
    pub resource_failure: Option<(usize, &'static str)>,
    effect_work: usize,
    effect_bytes: usize,
    mapping_depth: usize,
    mapping_anchor: Option<usize>,
}
impl<'a> Session<'a> {
    pub fn new(text: &'a str, options: ParseOptions, scanner: &'a mut dyn TokenScanner) -> Self {
        let rejected = text.len() > options.limits.input_bytes.min(u32::MAX as usize);
        let text = if rejected { "" } else { text };
        Self {
            text,
            input: Input::new(text),
            options,
            scanner,
            arena: {
                let mut arena = Vec::with_capacity(
                    if options.build_ast || options.lexical || HAS_SCOPE || HAS_RECOVERY {
                        text.len().saturating_mul(8).saturating_add(8).min(32768)
                    } else {
                        1
                    },
                );
                arena.push(Event::Empty);
                arena
            },
            diag: Diagnostics::with_limit_and_capacity(
                options.limits.diagnostics,
                text.len().saturating_mul(24).saturating_add(64).min(1 << 20),
            ),
            display: DisplayDiagnostics::new(options.limits.diagnostics),
            // memo は rule 本体単位（P4 実測 約 5 entry / byte）。
            memo: Memo::with_capacity(text.len().saturating_mul(6).saturating_add(64).min(1 << 20)),
            scope: ScopeStore::new(),
            trivia_cache: Memo::default(),
            effects: vec![],
            saved_effects: vec![],
            statistics: Statistics::default(),
            node_spans: vec![],
            text_spans: Default::default(),
            depth: 0,
            stack_anchor: None,
            depth_failure: None,
            resource_failure: rejected.then_some((0, "input byte budget exceeded")),
            effect_work: 0,
            effect_bytes: 0,
            mapping_depth: 0,
            mapping_anchor: None,
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
        if self.depth_failure.is_none() && anchor.abs_diff(address) > 256 * 1024 {
            self.depth_failure = Some(state.consumed.max(state.matched));
        }
        self.depth_failure
            .map(|offset| self.fail(state, offset, "maximum parse depth exceeded"))
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
    pub fn text(&self, span: Span) -> String {
        self.text[span[0]..span[1]]
            .trim_matches(|c: char| {
                c.is_whitespace() && !matches!(c, '\u{85}' | '\u{a0}' | '\u{2007}' | '\u{202f}')
            })
            .to_owned()
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
            self.effect_at(
                effect.value,
                effect.mode,
                effect.offset_cp,
                effect.length_cp,
            );
        }
    }
    fn effect_size(effect: &Effect) -> usize {
        match &effect.value {
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
        let size = match &effect {
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
        match &effect {
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
        });
    }
    pub fn lookup(&mut self, key: Key) -> Option<Step> {
        if !self.options.memo {
            return None;
        }
        let saved = self.memo.get(key)?;
        self.statistics.memo_hits += 1;
        self.display.merge(saved.display);
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
        self.memo.insert(key, Saved::new(step, effects, self.display.snapshot()));
    }
    pub fn event(&mut self, event: Event) -> EventId {
        if !self.options.build_ast && !self.options.lexical && !HAS_SCOPE && !HAS_RECOVERY {
            return EventId(0);
        }
        if self.arena.len() >= self.options.limits.arena_entries.min(u32::MAX as usize) {
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
            diag: self.diag.join(a.diag, b.diag),
        }
    }
    pub fn fail(&mut self, state: State, offset: usize, label: &'static str) -> Step {
        Step {
            ok: false,
            state,
            events: EventId(0),
            diag: self.diag.fail(offset, label),
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
    pub fn literal<const MATCH: bool>(
        &mut self,
        state: State,
        word: &'static str,
        sensitive: bool,
        label: &'static str,
    ) -> Step {
        let start = state.position::<MATCH>();
        let length = if sensitive {
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
            self.display.fail(state.consumed.max(state.begin().matched),"WildCardCharacterParser");
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
        else { self.display.fail(p, "SignParser"); }
        let digits = p;
        while bytes.get(p).is_some_and(u8::is_ascii_digit) { p += 1; }
        self.display.fail(p, "DigitParser");
        let mut count = p - digits;
        if count == 0 { self.display.fail(p, "OneOrMore"); }
        if bytes.get(p) == Some(&b'.') {
            p += 1;
            let fraction = p;
            while bytes.get(p).is_some_and(u8::is_ascii_digit) { p += 1; }
            self.display.fail(p, "DigitParser");
            if p == fraction { self.display.fail(p, "OneOrMore"); }
            count += p - fraction;
            if count == 0 { self.display.fail(start,"Chain"); self.display.fail(start,"'.'"); }
        } else {
            self.display.fail(p, "'.'");
            if count == 0 { self.display.fail(digits,"Chain"); }
        }
        if count == 0 {
            self.display.fail(start,"NumberParser");
            let mut out = self.primitive::<MATCH>(state, None, label);
            if !out.ok { out.diag = self.diag.fail(p,label); }
            return out;
        }
        if matches!(bytes.get(p), Some(b'e' | b'E')) {
            let exponent = p;
            p += 1;
            if matches!(bytes.get(p), Some(b'+' | b'-')) { p += 1; }
            else { self.display.fail(p,"SignParser"); }
            let digits = p;
            while bytes.get(p).is_some_and(u8::is_ascii_digit) { p += 1; }
            self.display.fail(p,"DigitParser");
            if digits == p { self.display.fail(p,"OneOrMore"); p = exponent; }
        } else { self.display.fail(p,"EParser"); self.display.fail(p,"ExponentParser"); }
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
        if p == start { self.display.fail(start,"AlphabetUnderScoreParser"); self.display.fail(start,"IdentifierParser"); }
        else { self.display.fail(p,"AlphabetNumericUnderScoreParser"); }
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
        self.display.fail(p, if quote == '\'' {"SingleQuoted"} else {"Quoted"});
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
            if !terminator.is_empty() { self.display.fail(out.consumed.max(out.matched),display); }
            let Some((_, n)) = self.input.cp_at(out.position::<MATCH>()) else {
                self.display.fail(out.consumed.max(out.matched),"ANY");
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
            self.display.fail(d.offset, d.expected);
            let diag = self.diag.fail(d.offset, d.expected);
            step.diag = self.diag.join(step.diag, diag);
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
                step.diag = self.diag.fail(state.position::<MATCH>(), label);
            }
        }
        step
    }
    pub fn owned_captures(&self, root: EventId, sites: &[usize]) -> Vec<(Span, EventId)> {
        let mut out = Vec::new();
        let mut stack = vec![(root, false)];
        while let Some((id, done)) = stack.pop() {
            match self.arena[id.0] {
                Event::Join(a, b) => {
                    stack.push((b, false));
                    stack.push((a, false));
                }
                Event::Values { child, .. } => stack.push((child, false)),
                Event::Capture {
                    site,
                    span,
                    child,
                    token_extent,
                } => {
                    if done {
                        if sites.contains(&site) {
                            out.push((
                                if token_extent {
                                    self.selected_extent(child, span)
                                } else {
                                    span
                                },
                                child,
                            ));
                        }
                    } else {
                        stack.push((id, true));
                        stack.push((child, false));
                    }
                }
                _ => {}
            }
        }
        out
    }
    pub fn selected_extent(&self, root: EventId, fallback: Span) -> Span {
        let mut stack = vec![root];
        let mut span = None;
        while let Some(id) = stack.pop() {
            match self.arena[id.0] {
                Event::Join(a, b) => {
                    stack.push(b);
                    stack.push(a);
                }
                Event::Capture { child, .. } | Event::Values { child, .. } => stack.push(child),
                Event::Rule { span: child, .. } | Event::Token { span: child, .. } => {
                    span = Some(span.map_or(child, |s: Span| [s[0], child[1]]));
                }
                Event::Empty => {}
                Event::Recovery { span: child, .. } => {
                    span = Some(span.map_or(child, |s: Span| [s[0], child[1]]));
                }
            }
        }
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
    pub fn has_recovery(&self, root: EventId) -> bool {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            match self.arena[id.0] {
                Event::Recovery { .. } => return true,
                Event::Join(a, b) => {
                    stack.push(b);
                    stack.push(a);
                }
                Event::Capture { child, .. }
                | Event::Rule { child, .. }
                | Event::Values { child, .. } => stack.push(child),
                _ => {}
            }
        }
        false
    }
    pub fn directly_recovered(&self, mut root: EventId) -> bool {
        loop {
            match self.arena[root.0] {
                Event::Recovery { .. } => return true,
                Event::Capture { child, .. } | Event::Values { child, .. } => root = child,
                _ => return false,
            }
        }
    }
    pub fn field_missing(&self, root: EventId, sites: &[usize]) -> bool {
        self.owned_captures(root, sites)
            .iter()
            .any(|(_, child)| self.has_recovery(*child))
    }
    pub fn recover<const MATCH: bool>(
        &mut self,
        state: State,
        failed: Step,
        rule: usize,
        mode: &'static str,
        patterns: &[&str],
    ) -> Step {
        if self.depth_failure.is_some() || self.resource_failure.is_some() || self.diag.exhausted {
            return failed;
        }
        let start = state.position::<MATCH>();
        let mut end = start;
        let mut sync_span = None;
        if mode == "SKIP" && patterns.is_empty() {
            end += self.input.cp_at(start).map_or(0, |(_, n)| n);
        } else {
            let fallback = mode == "AUTO" && patterns.is_empty();
            let patterns = if fallback { &[";"][..] } else { patterns };
            while let Some((_, n)) = self.input.cp_at(end) {
                if let Some(sync) = patterns
                    .iter()
                    .find(|s| !s.is_empty() && self.input.starts_with(end, s))
                {
                    sync_span = Some([end, end + sync.len()]);
                    if mode == "SYNC" || fallback {
                        end += sync.len();
                    }
                    break;
                }
                end += n;
            }
            if sync_span.is_none() && mode != "SKIP" {
                return failed;
            }
            if mode == "SKIP" && end == start {
                end += self.input.cp_at(start).map_or(0, |(_, n)| n);
            }
        }
        if end <= start {
            return failed;
        }
        let mut child = state.begin();
        child.advance::<MATCH>(end - start);
        let diag = self.diag.rule(rule, failed.diag);
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
    pub fn has_value_group(&self, root: EventId) -> bool {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            match self.arena[id.0] {
                Event::Values { .. } => return true,
                Event::Join(a, b) => {
                    stack.push(b);
                    stack.push(a);
                }
                Event::Rule { child, .. } | Event::Capture { child, .. } => stack.push(child),
                _ => {}
            }
        }
        false
    }
    pub fn mapped_text(&mut self, span: Span) -> String {
        self.remember_text(self.text(span), span)
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
        let extent = self.text_extent(root, span);
        let content = self.text_content_extent(root, extent);
        let value = if self.has_token_text(root) { self.text[content[0]..content[1]].to_owned() } else { self.text(content) };
        self.remember_text(value, extent)
    }
    pub fn field_text(&mut self, root: EventId, sites: &[usize]) -> Vec<String> {
        self.owned_captures(root, sites)
            .into_iter()
            .filter_map(|(span, child)| {
                if self.directly_recovered(child) {
                    None
                } else {
                    Some(self.semantic_text(child, span))
                }
            })
            .collect()
    }
    pub fn scope_captures(&self, root: EventId, sites: &[usize]) -> Vec<(usize, Span)> {
        let mut out = Vec::new();
        let mut stack = vec![(root, false)];
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
        out
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

use super::ast::*;
const HAS_RECOVERY:bool=false;
const SCOPE_MODES:&[&str]=&["sameRuleText"];
const HAS_SCOPE:bool=true;
const RULES:&[&str]=&["UBNF::UBNFFile", "UBNF::ImportDecl", "UBNF::GrammarDecl", "UBNF::GlobalSetting", "UBNF::SettingValue", "UBNF::StringSettingValue", "UBNF::BlockSettingValue", "UBNF::KeyValuePair", "UBNF::TokenDecl", "UBNF::TokenValue", "UBNF::UntilExpression", "UBNF::NegationExpression", "UBNF::LookaheadExpression", "UBNF::NegativeLookaheadExpression", "UBNF::CharRangeExpression", "UBNF::CIExpression", "UBNF::RegexExpression", "UBNF::AnyExpression", "UBNF::EofExpression", "UBNF::EmptyExpression", "UBNF::RuleDecl", "UBNF::Annotation", "UBNF::RootAnnotation", "UBNF::MappingAnnotation", "UBNF::EvalAnnotation", "UBNF::WhitespaceAnnotation", "UBNF::InterleaveAnnotation", "UBNF::BackrefAnnotation", "UBNF::ScopeTreeAnnotation", "UBNF::DeclaresAnnotation", "UBNF::CatalogAnnotation", "UBNF::LeftAssocAnnotation", "UBNF::RightAssocAnnotation", "UBNF::LongestChoiceAnnotation", "UBNF::PredictiveChoiceAnnotation", "UBNF::PrecedenceAnnotation", "UBNF::DocAnnotation", "UBNF::RecoveryAnnotation", "UBNF::SkipAnnotation", "UBNF::CommonFieldAnnotation", "UBNF::EnumAnnotation", "UBNF::SimpleAnnotation", "UBNF::ChoiceBody", "UBNF::SequenceBody", "UBNF::AnnotatedElement", "UBNF::PostfixQuantifier", "UBNF::SeparatedBy", "UBNF::BoundedQuantifier", "UBNF::Digits", "UBNF::AtomicElement", "UBNF::GroupElement", "UBNF::OptionalElement", "UBNF::RepeatElement", "UBNF::ErrorElement", "UBNF::TerminalElement", "UBNF::QuantifiedRef", "UBNF::TypeofElement", "UBNF::DottedIdentifier", "UBNF::RuleBody"];
const EXPRESSIONS:&[&str]=&["expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1155:1166:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1326:1367:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1326:1335:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1336:1346:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1354:1360:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1361:1367:body/3/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1507:1685:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1507:1516:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1517:1527:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1539:1542:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1550:1564:body/3/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1552:1562:body/3/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1584:1601:body/4/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1586:1599:body/4/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1619:1632:body/5/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1621:1630:body/5/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1652:1664:body/6/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1654:1662:body/6/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1682:1685:body/7/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1785:1821:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1785:1788:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1789:1799:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1805:1808:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1809:1821:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1860:1903:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1860:1878:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1886:1903:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2084:2117:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2084:2087:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2088:2104:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2090:2102:body/1/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2114:2117:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2215:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2199:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2205:2208:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2209:2215:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2357:2396:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2357:2364:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2365:2375:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2382:2385:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2396:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2455:2717:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2455:2470:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2478:2496:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2504:2523:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2531:2558:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2566:2585:body/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2593:2605:body/5/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2613:2628:body/6/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2636:2649:body/7/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2657:2670:body/8/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2678:2693:body/9/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2701:2717:body/10/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2855:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2828:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2829:2832:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2833:2839:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2852:2855:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2958:2990:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2958:2968:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2969:2972:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2973:2979:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2987:2990:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3100:3135:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3100:3111:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3112:3115:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3116:3122:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3132:3135:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3270:3314:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3270:3290:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3291:3294:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3295:3301:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3311:3314:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3424:3472:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3424:3436:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3437:3440:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3441:3447:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3453:3456:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3457:3463:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3469:3472:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3558:3583:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3558:3562:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3563:3566:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3567:3573:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3580:3583:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3681:3712:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3681:3688:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3689:3692:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3693:3699:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3709:3712:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3775:3780:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3775:3780:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3843:3848:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3843:3848:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3917:3924:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3917:3924:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4199:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4129:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4117:4127:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4148:4158:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4170:4175:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4176:4184:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4196:4199:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4255:4764:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4255:4269:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4277:4294:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4302:4316:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4324:4344:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4352:4372:body/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4380:4397:body/5/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4405:4424:body/6/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4432:4450:body/7/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4458:4475:body/8/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4483:4502:body/9/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4510:4530:body/10/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4537:4560:body/11/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4567:4593:body/12/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4600:4620:body/13/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4628:4641:body/14/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4649:4667:body/15/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4675:4689:body/16/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4697:4718:body/17/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4726:4740:body/18/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4748:4764:body/19/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4819:4826:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4819:4826:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4924:5066:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4924:4934:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4935:4938:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4946:4956:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4975:5057:body/3/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4977:5055:body/3/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4977:4980:body/3/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4981:4989:body/3/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4990:4993:body/3/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4994:4997:body/3/0/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4998:5008:body/3/0/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5021:5051:body/3/0/5/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5023:5037:body/3/0/5/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5023:5026:body/3/0/5/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5027:5037:body/3/0/5/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5052:5055:body/3/0/6/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5063:5066:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5139:5206:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5139:5146:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5147:5150:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5151:5163:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5173:5202:body/3/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5175:5191:body/3/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5175:5178:body/3/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5179:5191:body/3/0/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5203:5206:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5289:5332:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5289:5302:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5303:5332:body/1/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5305:5330:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5305:5308:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5309:5319:body/1/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5327:5330:body/1/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5417:5472:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5417:5430:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5431:5434:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5435:5444:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5445:5448:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5449:5459:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5469:5472:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5548:5594:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5548:5558:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5559:5562:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5563:5569:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5570:5573:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5574:5584:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5591:5594:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5674:5722:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5674:5686:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5687:5690:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5691:5697:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5698:5701:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5702:5712:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5719:5722:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5815:5912:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5815:5826:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5827:5830:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5831:5839:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5840:5843:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5844:5854:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5863:5908:body/5/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5865:5893:body/5/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5865:5868:body/5/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5869:5882:body/5/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5883:5886:body/5/0/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5887:5893:body/5/0/3/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5909:5912:body/6/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5991:6039:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5991:6001:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6002:6005:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6006:6015:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6016:6019:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6020:6026:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6036:6039:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6104:6116:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6104:6116:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6182:6195:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6182:6195:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6265:6281:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6265:6281:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6357:6376:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6357:6376:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6458:6515:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6458:6471:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6472:6475:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6476:6483:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6484:6487:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6488:6504:body/4/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6512:6515:body/5/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6583:6610:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6583:6589:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6590:6593:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6594:6600:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6607:6610:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6700:6772:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6700:6711:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6712:6715:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6716:6768:body/2/group", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6718:6760:body/2/0/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6718:6735:body/2/0/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6718:6724:body/2/0/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6725:6728:body/2/0/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6729:6735:body/2/0/0/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6750:6760:body/2/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6769:6772:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6827:6834:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6827:6834:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6997:7073:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6997:7011:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7012:7015:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7016:7026:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7039:7069:body/3/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7041:7055:body/3/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7041:7044:body/3/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7045:7055:body/3/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7070:7073:body/4/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7181:7188:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7181:7188:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7262:7276:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7262:7265:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7266:7276:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7396:7457:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7396:7408:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7423:7457:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7425:7441:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7425:7428:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7429:7441:body/1/0/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7543:7563:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7543:7563:body/0/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7545:7561:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7727:7912:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7727:7762:body/0/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7729:7742:body/0/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7770:7783:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7800:7821:body/2/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7802:7819:body/2/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7829:7850:body/3/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7831:7848:body/3/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7858:7873:body/4/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7860:7871:body/4/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7881:7912:body/5/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7883:7897:body/5/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7883:7886:body/5/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7887:7897:body/5/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7961:7976:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7961:7964:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7967:7970:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7973:7976:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8025:8042:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8025:8028:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8029:8042:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8100:8133:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8100:8103:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8104:8110:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8111:8129:body/2/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8113:8127:body/2/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8113:8116:body/2/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8117:8127:body/2/0/1/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8119:8125:body/2/0/1/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8130:8133:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8152:8167:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8152:8157:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8158:8167:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8160:8165:body/1/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8314:8434:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8314:8326:body/0/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8334:8349:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8357:8370:body/2/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8378:8390:body/3/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8398:8413:body/4/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8421:8434:body/5/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8524:8546:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8524:8527:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8528:8536:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8543:8546:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8636:8658:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8636:8639:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8640:8648:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8655:8658:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8749:8771:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8749:8752:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8753:8761:body/1/ruleRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8768:8771:body/2/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8871:8902:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8871:8878:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8879:8882:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8883:8889:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8899:8902:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8997:9003:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8997:9003:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9214:9254:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9214:9243:body/0/optional", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9216:9241:body/0/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9216:9226:body/0/0/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9238:9241:body/0/0/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9244:9254:body/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9381:9422:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9381:9390:body/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9391:9394:body/1/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9395:9405:body/2/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9419:9422:body/3/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9526:9569:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9526:9536:body/0/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9544:9569:body/1/repeat", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9546:9560:body/1/0/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9546:9549:body/1/0/0/literal", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9550:9560:body/1/0/1/tokenRef", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9627:9637:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9627:9637:body/0/ruleRef"];
const BODIES:&[&str]=&["expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1326:1367:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1507:1685:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1785:1821:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1860:1903:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2084:2117:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2215:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2357:2396:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2455:2717:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2855:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2958:2990:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3100:3135:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3270:3314:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3424:3472:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3558:3583:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3681:3712:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3775:3780:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3843:3848:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3917:3924:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4199:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4255:4764:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4819:4826:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4924:5066:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5139:5206:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5289:5332:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5417:5472:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5548:5594:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5674:5722:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5815:5912:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5991:6039:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6104:6116:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6182:6195:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6265:6281:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6357:6376:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6458:6515:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6583:6610:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6700:6772:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6827:6834:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6997:7073:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7181:7188:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7262:7276:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7396:7457:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7543:7563:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7727:7912:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7961:7976:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8025:8042:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8100:8133:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8152:8167:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8314:8434:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8524:8546:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8636:8658:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8749:8771:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8871:8902:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8997:9003:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9214:9254:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9381:9422:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9526:9569:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9627:9637:body/seq"];
const SITES:&[(&str,&str)]=&[("expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/0/repeat/capture/0", "grammars"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1336:1346:body/1/tokenRef/capture/0", "alias"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1361:1367:body/3/tokenRef/capture/0", "path"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1517:1527:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1550:1564:body/3/repeat/capture/0", "imports"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1584:1601:body/4/repeat/capture/0", "settings"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1619:1632:body/5/repeat/capture/0", "tokens"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1652:1664:body/6/repeat/capture/0", "rules"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1789:1799:body/1/tokenRef/capture/0", "key"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1809:1821:body/3/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/0/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2088:2104:body/1/repeat/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2199:body/0/tokenRef/capture/0", "key"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2209:2215:body/2/tokenRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2365:2375:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2396:body/3/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2833:2839:body/2/tokenRef/capture/0", "terminator"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2973:2979:body/2/tokenRef/capture/0", "chars"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3116:3122:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3295:3301:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3441:3447:body/2/tokenRef/capture/0", "min"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3457:3463:body/4/tokenRef/capture/0", "max"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3567:3573:body/2/tokenRef/capture/0", "word"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3693:3699:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4129:body/0/repeat/capture/0", "annotations"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4148:4158:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4176:4184:body/3/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4946:4956:body/2/tokenRef/capture/0", "className"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4998:5008:body/3/0/4/tokenRef/capture/0", "paramNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5027:5037:body/3/0/5/0/1/tokenRef/capture/0", "paramNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5151:5163:body/2/ruleRef/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5179:5191:body/3/0/1/ruleRef/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5309:5319:body/1/0/1/tokenRef/capture/0", "style"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5449:5459:body/4/tokenRef/capture/0", "profile"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5574:5584:body/4/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5702:5712:body/4/tokenRef/capture/0", "mode"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5844:5854:body/4/tokenRef/capture/0", "symbol"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5887:5893:body/5/0/3/tokenRef/capture/0", "description"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6020:6026:body/4/tokenRef/capture/0", "context"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6488:6504:body/4/tokenRef/capture/0", "level"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6594:6600:body/2/tokenRef/capture/0", "text"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6729:6735:body/2/0/0/2/tokenRef/capture/0", "syncTokens"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6750:6760:body/2/0/1/tokenRef/capture/0", "mode"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7016:7026:body/2/tokenRef/capture/0", "fieldNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7045:7055:body/3/0/1/tokenRef/capture/0", "fieldNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7266:7276:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7396:7408:body/0/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7429:7441:body/1/0/1/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7543:7563:body/0/repeat/capture/0", "elements"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7729:7742:body/0/0/ruleRef/capture/0", "typeofConstraint"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7770:7783:body/1/ruleRef/capture/0", "element"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7887:7897:body/5/0/1/tokenRef/capture/0", "captureName"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8528:8536:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8640:8648:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8753:8761:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8883:8889:body/2/tokenRef/capture/0", "message"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8997:9003:body/0/tokenRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9216:9226:body/0/0/0/tokenRef/capture/0", "namespace"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9244:9254:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9395:9405:body/2/tokenRef/capture/0", "captureName"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9526:9536:body/0/tokenRef/capture/0", "parts"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9550:9560:body/1/0/1/tokenRef/capture/0", "parts")];
impl Session<'_> {
#[inline(never)] fn build_values(&mut self,root:EventId)->Result<Vec<Ast>,String> {let frame=0u8;let address=std::ptr::addr_of!(frame) as usize;let anchor=*self.mapping_anchor.get_or_insert(address);if self.mapping_depth>=self.options.limits.mapping_depth || anchor.abs_diff(address)>256*1024 {return Err("maximum mapping depth exceeded".into());}self.mapping_depth+=1;let result=self.build_values_inner(root);self.mapping_depth-=1;result}
fn build_values_inner(&mut self,root:EventId)->Result<Vec<Ast>,String> {let mut values=Vec::new();let mut stack=vec![root];while let Some(id)=stack.pop() {match self.arena[id.0] {Event::Join(a,b)=>{stack.push(b);stack.push(a);},Event::Capture{child,..}=>stack.push(child),Event::Values{child,span,wrap}=>{let mut found=self.build_values(child)?;if wrap && (matches!(found.as_slice(),[Ast::Text(_)]) || (found.is_empty() && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {found=vec![Ast::Text(self.semantic_text(child,span))];}values.extend(found);},Event::Rule{rule,span,child}=>values.extend(self.build_rule(rule,span,child)?),_=>{}}}Ok(values)}
fn field_nodes(&mut self,root:EventId,sites:&[usize],fallback:Option<usize>)->Result<Vec<Ast>,String> {let mut values=Vec::new();for (span,child) in self.owned_captures(root,sites) {let mut found=self.build_values(child)?;if found.is_empty() && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child) {found.push(Ast::Text(self.semantic_text(child,span)));}for value in found {values.push(if let (Some(ty),Ast::Text(text))=(fallback,&value) {self.leaf(ty,span,text.clone())?} else {value});}}Ok(values)}
fn leaf(&mut self,ty:usize,span:Span,text:String)->Result<Ast,String> {let text=self.remember_text(text,span);let span=self.span(span);let _=(span,text);Err(format!("unknown leaf type {ty}"))}
fn build_rule(&mut self,rule:usize,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);match rule {
0=>self.map_rule_0(span,child),
1=>self.map_rule_1(span,child),
2=>self.map_rule_2(span,child),
3=>self.map_rule_3(span,child),
4=>self.map_rule_4(span,child),
5=>self.map_rule_5(span,child),
6=>self.map_rule_6(span,child),
7=>self.map_rule_7(span,child),
8=>self.map_rule_8(span,child),
9=>self.map_rule_9(span,child),
10=>self.map_rule_10(span,child),
11=>self.map_rule_11(span,child),
12=>self.map_rule_12(span,child),
13=>self.map_rule_13(span,child),
14=>self.map_rule_14(span,child),
15=>self.map_rule_15(span,child),
16=>self.map_rule_16(span,child),
17=>self.map_rule_17(span,child),
18=>self.map_rule_18(span,child),
19=>self.map_rule_19(span,child),
20=>self.map_rule_20(span,child),
21=>self.map_rule_21(span,child),
22=>self.map_rule_22(span,child),
23=>self.map_rule_23(span,child),
24=>self.map_rule_24(span,child),
25=>self.map_rule_25(span,child),
26=>self.map_rule_26(span,child),
27=>self.map_rule_27(span,child),
28=>self.map_rule_28(span,child),
29=>self.map_rule_29(span,child),
30=>self.map_rule_30(span,child),
31=>self.map_rule_31(span,child),
32=>self.map_rule_32(span,child),
33=>self.map_rule_33(span,child),
34=>self.map_rule_34(span,child),
35=>self.map_rule_35(span,child),
36=>self.map_rule_36(span,child),
37=>self.map_rule_37(span,child),
38=>self.map_rule_38(span,child),
39=>self.map_rule_39(span,child),
40=>self.map_rule_40(span,child),
41=>self.map_rule_41(span,child),
42=>self.map_rule_42(span,child),
43=>self.map_rule_43(span,child),
44=>self.map_rule_44(span,child),
45=>self.map_rule_45(span,child),
46=>self.map_rule_46(span,child),
47=>self.map_rule_47(span,child),
48=>self.map_rule_48(span,child),
49=>self.map_rule_49(span,child),
50=>self.map_rule_50(span,child),
51=>self.map_rule_51(span,child),
52=>self.map_rule_52(span,child),
53=>self.map_rule_53(span,child),
54=>self.map_rule_54(span,child),
55=>self.map_rule_55(span,child),
56=>self.map_rule_56(span,child),
57=>self.map_rule_57(span,child),
58=>self.map_rule_58(span,child),
_=>Err(format!("unknown rule {rule}"))}}
fn map_rule_0(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_nodes(child,&[0],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_GrammarDecl(_))) {return Err("grammars: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[0],"UBNFFile",span);Ok(vec![Ast::g_UBNFAST_2e_UBNFFile(g_UBNFAST_2e_UBNFFile{span,node_id,
g_grammars:f0,
})])}}
fn map_rule_1(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[1]);
if f0.is_empty() && self.field_missing(child,&[1]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("alias requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[2]);
if f1.is_empty() && self.field_missing(child,&[2]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("path requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let span=self.span(span);let node_id=self.node(RULES[1],"ImportDecl",span);Ok(vec![Ast::g_UBNFAST_2e_ImportDecl(g_UBNFAST_2e_ImportDecl{span,node_id,
g_alias:f0,
g_path:f1,
})])}}
fn map_rule_2(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[3]);
if f0.is_empty() && self.field_missing(child,&[3]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let f1=self.field_nodes(child,&[4],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ImportDecl(_))) {return Err("imports: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let f2=self.field_nodes(child,&[5],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_GlobalSetting(_))) {return Err("settings: mapped value type mismatch".into());}
let f2:Vec<Ast>=f2;
let f3=self.field_nodes(child,&[6],None)?;
if !f3.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_TokenDecl(_))) {return Err("tokens: mapped value type mismatch".into());}
let f3:Vec<Ast>=f3;
let f4=self.field_nodes(child,&[7],None)?;
if !f4.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_RuleDecl(_))) {return Err("rules: mapped value type mismatch".into());}
let f4:Vec<Ast>=f4;
let span=self.span(span);let node_id=self.node(RULES[2],"GrammarDecl",span);Ok(vec![Ast::g_UBNFAST_2e_GrammarDecl(g_UBNFAST_2e_GrammarDecl{span,node_id,
g_name:f0,
g_imports:f1,
g_settings:f2,
g_tokens:f3,
g_rules:f4,
})])}}
fn map_rule_3(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[8]);
if f0.is_empty() && self.field_missing(child,&[8]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("key requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[9],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_BlockSettingValue(_)) || matches!(value,Ast::g_UBNFAST_2e_StringSettingValue(_))) {return Err("value: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[9]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("value requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[3],"GlobalSetting",span);Ok(vec![Ast::g_UBNFAST_2e_GlobalSetting(g_UBNFAST_2e_GlobalSetting{span,node_id,
g_key:f0,
g_value:f1,
})])}}
fn map_rule_4(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_5(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[10],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_DottedIdentifier(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[10]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[5],"StringSettingValue",span);Ok(vec![Ast::g_UBNFAST_2e_StringSettingValue(g_UBNFAST_2e_StringSettingValue{span,node_id,
g_value:f0,
})])}}
fn map_rule_6(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_nodes(child,&[11],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_KeyValuePair(_))) {return Err("entries: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[6],"BlockSettingValue",span);Ok(vec![Ast::g_UBNFAST_2e_BlockSettingValue(g_UBNFAST_2e_BlockSettingValue{span,node_id,
g_entries:f0,
})])}}
fn map_rule_7(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[12]);
if f0.is_empty() && self.field_missing(child,&[12]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("key requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[13]);
if f1.is_empty() && self.field_missing(child,&[13]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("value requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let span=self.span(span);let node_id=self.node(RULES[7],"KeyValuePair",span);Ok(vec![Ast::g_UBNFAST_2e_KeyValuePair(g_UBNFAST_2e_KeyValuePair{span,node_id,
g_key:f0,
g_value:f1,
})])}}
fn map_rule_8(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[14]);
if f0.is_empty() && self.field_missing(child,&[14]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[15],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_AnyExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_CIExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_CharRangeExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_DottedIdentifier(_)) || matches!(value,Ast::g_UBNFAST_2e_EmptyExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_EofExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_LookaheadExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_NegationExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_NegativeLookaheadExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_RegexExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_UntilExpression(_))) {return Err("value: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[15]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("value requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[8],"TokenDecl",span);Ok(vec![Ast::g_UBNFAST_2e_TokenDecl(g_UBNFAST_2e_TokenDecl{span,node_id,
g_name:f0,
g_value:f1,
})])}}
fn map_rule_9(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_10(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[16]);
if f0.is_empty() && self.field_missing(child,&[16]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("terminator requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[10],"UntilExpression",span);Ok(vec![Ast::g_UBNFAST_2e_UntilExpression(g_UBNFAST_2e_UntilExpression{span,node_id,
g_terminator:f0,
})])}}
fn map_rule_11(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[17]);
if f0.is_empty() && self.field_missing(child,&[17]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("chars requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[11],"NegationExpression",span);Ok(vec![Ast::g_UBNFAST_2e_NegationExpression(g_UBNFAST_2e_NegationExpression{span,node_id,
g_chars:f0,
})])}}
fn map_rule_12(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[18]);
if f0.is_empty() && self.field_missing(child,&[18]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("pattern requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[12],"LookaheadExpression",span);Ok(vec![Ast::g_UBNFAST_2e_LookaheadExpression(g_UBNFAST_2e_LookaheadExpression{span,node_id,
g_pattern:f0,
})])}}
fn map_rule_13(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[19]);
if f0.is_empty() && self.field_missing(child,&[19]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("pattern requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[13],"NegativeLookaheadExpression",span);Ok(vec![Ast::g_UBNFAST_2e_NegativeLookaheadExpression(g_UBNFAST_2e_NegativeLookaheadExpression{span,node_id,
g_pattern:f0,
})])}}
fn map_rule_14(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[20]);
if f0.is_empty() && self.field_missing(child,&[20]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("min requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[21]);
if f1.is_empty() && self.field_missing(child,&[21]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("max requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let span=self.span(span);let node_id=self.node(RULES[14],"CharRangeExpression",span);Ok(vec![Ast::g_UBNFAST_2e_CharRangeExpression(g_UBNFAST_2e_CharRangeExpression{span,node_id,
g_min:f0,
g_max:f1,
})])}}
fn map_rule_15(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[22]);
if f0.is_empty() && self.field_missing(child,&[22]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("word requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[15],"CIExpression",span);Ok(vec![Ast::g_UBNFAST_2e_CIExpression(g_UBNFAST_2e_CIExpression{span,node_id,
g_word:f0,
})])}}
fn map_rule_16(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[23]);
if f0.is_empty() && self.field_missing(child,&[23]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("pattern requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[16],"RegexExpression",span);Ok(vec![Ast::g_UBNFAST_2e_RegexExpression(g_UBNFAST_2e_RegexExpression{span,node_id,
g_pattern:f0,
})])}}
fn map_rule_17(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[17],"AnyExpression",span);Ok(vec![Ast::g_UBNFAST_2e_AnyExpression(g_UBNFAST_2e_AnyExpression{span,node_id,
})])}}
fn map_rule_18(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[18],"EofExpression",span);Ok(vec![Ast::g_UBNFAST_2e_EofExpression(g_UBNFAST_2e_EofExpression{span,node_id,
})])}}
fn map_rule_19(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[19],"EmptyExpression",span);Ok(vec![Ast::g_UBNFAST_2e_EmptyExpression(g_UBNFAST_2e_EmptyExpression{span,node_id,
})])}}
fn map_rule_20(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_nodes(child,&[24],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_BackrefAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_CatalogAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_CommonFieldAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_DeclaresAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_DocAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_EnumAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_EvalAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_InterleaveAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_LeftAssocAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_LongestChoiceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_MappingAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_PrecedenceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_RecoveryAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_RightAssocAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_RootAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_ScopeTreeAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_SimpleAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_SkipAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_WhitespaceAnnotation(_))) {return Err("annotations: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let mut f1=self.field_text(child,&[25]);
if f1.is_empty() && self.field_missing(child,&[25]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("name requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let mut f2=self.field_nodes(child,&[26],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[26]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("body requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[20],"RuleDecl",span);Ok(vec![Ast::g_UBNFAST_2e_RuleDecl(g_UBNFAST_2e_RuleDecl{span,node_id,
g_annotations:f0,
g_name:f1,
g_body:f2,
})])}}
fn map_rule_21(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_22(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[22],"RootAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_RootAnnotation(g_UBNFAST_2e_RootAnnotation{span,node_id,
})])}}
fn map_rule_23(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[27]);
if f0.is_empty() && self.field_missing(child,&[27]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("className requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let f1=self.field_text(child,&[28, 29]);
let f1:Vec<String>=f1;
let span=self.span(span);let node_id=self.node(RULES[23],"MappingAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_MappingAnnotation(g_UBNFAST_2e_MappingAnnotation{span,node_id,
g_className:f0,
g_paramNames:f1,
})])}}
fn map_rule_24(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_nodes(child,&[30, 31],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_KeyValuePair(_))) {return Err("entries: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[24],"EvalAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_EvalAnnotation(g_UBNFAST_2e_EvalAnnotation{span,node_id,
g_entries:f0,
})])}}
fn map_rule_25(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[32]);
if f0.len()>1 {return Err("style requires at most one value".into());}let f0=f0.pop();
let f0:Option<String>=f0;
let span=self.span(span);let node_id=self.node(RULES[25],"WhitespaceAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_WhitespaceAnnotation(g_UBNFAST_2e_WhitespaceAnnotation{span,node_id,
g_style:f0,
})])}}
fn map_rule_26(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[33]);
if f0.is_empty() && self.field_missing(child,&[33]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("profile requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[26],"InterleaveAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_InterleaveAnnotation(g_UBNFAST_2e_InterleaveAnnotation{span,node_id,
g_profile:f0,
})])}}
fn map_rule_27(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[34]);
if f0.is_empty() && self.field_missing(child,&[34]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[27],"BackrefAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_BackrefAnnotation(g_UBNFAST_2e_BackrefAnnotation{span,node_id,
g_name:f0,
})])}}
fn map_rule_28(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[35]);
if f0.is_empty() && self.field_missing(child,&[35]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("mode requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[28],"ScopeTreeAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_ScopeTreeAnnotation(g_UBNFAST_2e_ScopeTreeAnnotation{span,node_id,
g_mode:f0,
})])}}
fn map_rule_29(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[36]);
if f0.is_empty() && self.field_missing(child,&[36]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("symbol requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[37]);
if f1.len()>1 {return Err("description requires at most one value".into());}let f1=f1.pop();
let f1:Option<String>=f1;
let span=self.span(span);let node_id=self.node(RULES[29],"DeclaresAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_DeclaresAnnotation(g_UBNFAST_2e_DeclaresAnnotation{span,node_id,
g_symbol:f0,
g_description:f1,
})])}}
fn map_rule_30(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[38]);
if f0.is_empty() && self.field_missing(child,&[38]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("context requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[30],"CatalogAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_CatalogAnnotation(g_UBNFAST_2e_CatalogAnnotation{span,node_id,
g_context:f0,
})])}}
fn map_rule_31(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[31],"LeftAssocAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_LeftAssocAnnotation(g_UBNFAST_2e_LeftAssocAnnotation{span,node_id,
})])}}
fn map_rule_32(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[32],"RightAssocAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_RightAssocAnnotation(g_UBNFAST_2e_RightAssocAnnotation{span,node_id,
})])}}
fn map_rule_33(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[33],"LongestChoiceAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_LongestChoiceAnnotation(g_UBNFAST_2e_LongestChoiceAnnotation{span,node_id,
})])}}
fn map_rule_34(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[34],"PredictiveChoiceAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(g_UBNFAST_2e_PredictiveChoiceAnnotation{span,node_id,
})])}}
fn map_rule_35(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[39]);
if f0.is_empty() && self.field_missing(child,&[39]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("level requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[35],"PrecedenceAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_PrecedenceAnnotation(g_UBNFAST_2e_PrecedenceAnnotation{span,node_id,
g_level:f0,
})])}}
fn map_rule_36(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[40]);
if f0.is_empty() && self.field_missing(child,&[40]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("text requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[36],"DocAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_DocAnnotation(g_UBNFAST_2e_DocAnnotation{span,node_id,
g_text:f0,
})])}}
fn map_rule_37(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[42]);
if f0.len()>1 {return Err("mode requires at most one value".into());}let f0=f0.pop();
let f0:Option<String>=f0;
let mut f1=self.field_text(child,&[41]);
if f1.len()>1 {return Err("syncTokens requires at most one value".into());}let f1=f1.pop();
let f1:Option<String>=f1;
let span=self.span(span);let node_id=self.node(RULES[37],"RecoveryAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_RecoveryAnnotation(g_UBNFAST_2e_RecoveryAnnotation{span,node_id,
g_mode:f0,
g_syncTokens:f1,
})])}}
fn map_rule_38(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[38],"SkipAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_SkipAnnotation(g_UBNFAST_2e_SkipAnnotation{span,node_id,
})])}}
fn map_rule_39(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_text(child,&[43, 44]);
let f0:Vec<String>=f0;
let span=self.span(span);let node_id=self.node(RULES[39],"CommonFieldAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_CommonFieldAnnotation(g_UBNFAST_2e_CommonFieldAnnotation{span,node_id,
g_fieldNames:f0,
})])}}
fn map_rule_40(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[40],"EnumAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_EnumAnnotation(g_UBNFAST_2e_EnumAnnotation{span,node_id,
})])}}
fn map_rule_41(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[45]);
if f0.is_empty() && self.field_missing(child,&[45]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[41],"SimpleAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_SimpleAnnotation(g_UBNFAST_2e_SimpleAnnotation{span,node_id,
g_name:f0,
})])}}
fn map_rule_42(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_nodes(child,&[46, 47],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_SequenceBody(_))) {return Err("alternatives: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[42],"ChoiceBody",span);Ok(vec![Ast::g_UBNFAST_2e_ChoiceBody(g_UBNFAST_2e_ChoiceBody{span,node_id,
g_alternatives:f0,
})])}}
fn map_rule_43(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_nodes(child,&[48],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_AnnotatedElement(_))) {return Err("elements: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[43],"SequenceBody",span);Ok(vec![Ast::g_UBNFAST_2e_SequenceBody(g_UBNFAST_2e_SequenceBody{span,node_id,
g_elements:f0,
})])}}
fn map_rule_44(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[50],None)?;
if !f0.iter().all(|value|matches!(value,Ast::Text(_)) || matches!(value,Ast::g_UBNFAST_2e_ErrorElement(_)) || matches!(value,Ast::g_UBNFAST_2e_GroupElement(_)) || matches!(value,Ast::g_UBNFAST_2e_OptionalElement(_)) || matches!(value,Ast::g_UBNFAST_2e_RepeatElement(_)) || matches!(value,Ast::g_UBNFAST_2e_TerminalElement(_))) {return Err("element: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[50]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("element requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_text(child,&[51]);
if f1.len()>1 {return Err("captureName requires at most one value".into());}let f1=f1.pop();
let f1:Option<String>=f1;
let mut f2=self.field_nodes(child,&[49],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_TypeofElement(_))) {return Err("typeofConstraint: mapped value type mismatch".into());}
if f2.len()>1 {return Err("typeofConstraint requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let span=self.span(span);let node_id=self.node(RULES[44],"AnnotatedElement",span);Ok(vec![Ast::g_UBNFAST_2e_AnnotatedElement(g_UBNFAST_2e_AnnotatedElement{span,node_id,
g_element:f0,
g_captureName:f1,
g_typeofConstraint:f2,
})])}}
fn map_rule_45(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_46(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_47(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_48(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_49(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_50(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[52],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[52]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("body requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[50],"GroupElement",span);Ok(vec![Ast::g_UBNFAST_2e_GroupElement(g_UBNFAST_2e_GroupElement{span,node_id,
g_body:f0,
})])}}
fn map_rule_51(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[53],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[53]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("body requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[51],"OptionalElement",span);Ok(vec![Ast::g_UBNFAST_2e_OptionalElement(g_UBNFAST_2e_OptionalElement{span,node_id,
g_body:f0,
})])}}
fn map_rule_52(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[54],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[54]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("body requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[52],"RepeatElement",span);Ok(vec![Ast::g_UBNFAST_2e_RepeatElement(g_UBNFAST_2e_RepeatElement{span,node_id,
g_body:f0,
})])}}
fn map_rule_53(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[55]);
if f0.is_empty() && self.field_missing(child,&[55]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("message requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[53],"ErrorElement",span);Ok(vec![Ast::g_UBNFAST_2e_ErrorElement(g_UBNFAST_2e_ErrorElement{span,node_id,
g_message:f0,
})])}}
fn map_rule_54(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[56]);
if f0.is_empty() && self.field_missing(child,&[56]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[54],"TerminalElement",span);Ok(vec![Ast::g_UBNFAST_2e_TerminalElement(g_UBNFAST_2e_TerminalElement{span,node_id,
g_value:f0,
})])}}
fn map_rule_55(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_56(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[59]);
if f0.is_empty() && self.field_missing(child,&[59]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("captureName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[56],"TypeofElement",span);Ok(vec![Ast::g_UBNFAST_2e_TypeofElement(g_UBNFAST_2e_TypeofElement{span,node_id,
g_captureName:f0,
})])}}
fn map_rule_57(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_text(child,&[60, 61]);
let f0:Vec<String>=f0;
let span=self.span(span);let node_id=self.node(RULES[57],"DottedIdentifier",span);Ok(vec![Ast::g_UBNFAST_2e_DottedIdentifier(g_UBNFAST_2e_DottedIdentifier{span,node_id,
g_parts:f0,
})])}}
fn map_rule_58(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn occurrences(
    &self,
    root: EventId,
) -> (Vec<Capture>, Vec<LexicalOccurrence>, Vec<LexicalOccurrence>) {
    let mut captures = Vec::new();
    let mut capture_owners = Vec::new();
    let mut rule_orders = Vec::new();
    let mut rule_order = 0;
    let mut lexical = Vec::new();
    let mut tokens = Vec::new();
    let mut stack = vec![(root, None, false, None)];
    let mut lexical_order = 0;
    while let Some((id, parent, done, current)) = stack.pop() {
        match self.arena[id.0] {
            Event::Join(a, b) => {
                stack.push((b, parent, false, None));
                stack.push((a, parent, false, None));
            }
            Event::Values { child, .. } => stack.push((child, parent, false, None)),
            Event::Capture {
                site, span, child, ..
            } => {
                if done {
                    if self.options.build_ast || self.options.lexical {
                        capture_owners.push(parent);
                        let order = captures.len();
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
                    stack.push((id, parent, true, None));
                    stack.push((child, parent, false, None));
                }
            }
            Event::Token {
                expr, rule, span, ..
            }
if self.options.lexical && expr != usize::MAX => {
                tokens.push(LexicalOccurrence {
                    occurrence_id: tokens.len(),
                    parent_occurrence_id: parent,
                    rule_id: RULES[rule],
                    expr_id: EXPRESSIONS[expr],
                    span: self.span(span),
                    completion_order: lexical_order,
                });
                lexical_order += 1;
            }
            Event::Rule { rule, span, child } => {
                if done {
                    if let Some(index) = current {
                        rule_orders[index] = rule_order;
                        rule_order += 1;
                        let node: &mut LexicalOccurrence = &mut lexical[index];
                        node.completion_order = lexical_order;
                        lexical_order += 1;
                    }
                } else {
                    let index = if self.options.lexical || self.options.build_ast {
                        rule_orders.push(0);
                        let index = lexical.len();
                        lexical.push(LexicalOccurrence {
                            occurrence_id: index,
                            parent_occurrence_id: parent,
                            rule_id: RULES[rule],
                            expr_id: BODIES[rule],
                            span: self.span(span),
                            completion_order: 0,
                        });
                        Some(index)
                    } else {
                        parent
                    };
                    stack.push((id, parent, true, index));
                    stack.push((child, index, false, None));
                }
            }
            _ => {}
        }
    }
    for (capture, owner) in captures.iter_mut().zip(capture_owners) {
        capture.rule_completion_order = owner.map_or(0, |i| rule_orders[i]);
    }
    for token in &mut tokens {
        token.occurrence_id += lexical.len();
    }
    if !self.options.lexical {
        lexical.clear();
    }
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
pub(crate) fn finish(mut self, mut step: Step) -> ParseResult {
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
    let trailing = step.ok && self.options.require_eof && step.state.consumed != self.text.len();
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
    let (recoveries, recovery_diagnostics) = if step.ok {
        self.recovery_occurrences(step.events)
    } else {
        (vec![], vec![])
    };
    diagnostics.splice(0..0, recovery_diagnostics);
    let (captures, lexical, tokens) = if step.ok {
        self.occurrences(step.events)
    } else {
        (vec![], vec![], vec![])
    };
    let ast = if ok && self.options.build_ast {
        match self.build_values(step.events).and_then(|mut nodes| {
            if nodes.len() == 1 {
                Ok(Some(nodes.remove(0)))
            } else if nodes.is_empty() && self.has_recovery(step.events) {
                Ok(None)
            } else {
                Err("entry must produce one AST value".into())
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
    let mut value_spans = Vec::new();
    if let Some(ast) = &ast {
        let mut path = String::with_capacity(128);
        ast.collect(&mut path, &self.text_spans, &mut value_spans);
    }
    ParseResult {
        value_spans,
        ok,
        consumed_cp,
        matched_cp,
        ast,
        captures,
        lexical,
        tokens,
        node_spans: self.node_spans,
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
                        let (action, name, offset_cp, length_cp) = match &effect.value {
                            ScanEffect::Enter => ("enter", None, effect.offset_cp, 0),
                            ScanEffect::Leave => ("leave", None, effect.offset_cp, 0),
                            ScanEffect::Clear => ("clear", None, effect.offset_cp, 0),
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
    }
}
}
// value span の path は共有バッファに push / truncate で組み立てる（field ごとの format! を避ける）。
trait CollectValueSpans {
    fn collect(
        &self,
        path: &mut String,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    );
}
impl CollectValueSpans for String {
    fn collect(
        &self,
        path: &mut String,
        texts: &std::collections::HashMap<usize, Span>,
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
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    ) {
        for (i, value) in self.iter().enumerate() {
            let len = path.len();
            path.push('/');
            path.push_str(&i.to_string());
            value.collect(path, texts, out);
            path.truncate(len);
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Option<T> {
    fn collect(
        &self,
        path: &mut String,
        texts: &std::collections::HashMap<usize, Span>,
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
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    ) {
        self.as_ref().collect(path, texts, out);
    }
}
impl CollectValueSpans for Ast { fn collect(&self,path:&mut String,texts:&std::collections::HashMap<usize,Span>,out:&mut Vec<ValueSpan>) { match self { Self::Text(text)=>{if let Some(span)=texts.get(&(text.as_ptr() as usize)) {out.push(ValueSpan{path:path.clone(),span:*span,text:Some(text.clone())});}},Self::Null=>{},
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
