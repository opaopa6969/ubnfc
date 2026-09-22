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
            diag: Diag(0),
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Saved {
    consumed: u32,
    matched: u32,
    events: u32,
    diag: u32,
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
            diag: u32::try_from(step.diag.0).expect("diagnostics"),
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
            diag: Diag(self.diag as usize),
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
            diag: Diagnostics::with_limit(options.limits.diagnostics),
            display: DisplayDiagnostics::new(options.limits.diagnostics),
            memo: Memo::with_capacity(text.len().saturating_mul(10).saturating_add(64).min(32768)),
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
        if self.diag.exhausted || self.display.arena.exhausted {
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
            if step.diag.0 == 0 {
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
const SCOPE_MODES:&[&str]=&["lexical"];
const HAS_SCOPE:bool=true;
const RULES:&[&str]=&["TinyExpressionP4::Formula", "TinyExpressionP4::CodeBlock", "TinyExpressionP4::ImportDeclaration", "TinyExpressionP4::ClassName", "TinyExpressionP4::VariableDeclaration", "TinyExpressionP4::NumberVariableDeclaration", "TinyExpressionP4::StringVariableDeclaration", "TinyExpressionP4::BooleanVariableDeclaration", "TinyExpressionP4::ObjectVariableDeclaration", "TinyExpressionP4::TypeHint", "TinyExpressionP4::NumberTypeHint", "TinyExpressionP4::StringTypeHint", "TinyExpressionP4::BooleanTypeHint", "TinyExpressionP4::ObjectTypeHint", "TinyExpressionP4::OnlyIfAbsent", "TinyExpressionP4::Description", "TinyExpressionP4::Annotation", "TinyExpressionP4::AnnotationParameters", "TinyExpressionP4::AnnotationParameter", "TinyExpressionP4::MethodDeclaration", "TinyExpressionP4::NumberMethodDeclaration", "TinyExpressionP4::StringMethodDeclaration", "TinyExpressionP4::BooleanMethodDeclaration", "TinyExpressionP4::ObjectMethodDeclaration", "TinyExpressionP4::MethodParameters", "TinyExpressionP4::MethodParameter", "TinyExpressionP4::NumberReturnType", "TinyExpressionP4::StringReturnType", "TinyExpressionP4::BooleanReturnType", "TinyExpressionP4::ObjectReturnType", "TinyExpressionP4::ReturnType", "TinyExpressionP4::ExternalBooleanInvocation", "TinyExpressionP4::ExternalNumberInvocation", "TinyExpressionP4::ExternalStringInvocation", "TinyExpressionP4::ExternalObjectInvocation", "TinyExpressionP4::MethodInvocationHeader", "TinyExpressionP4::MethodInvocation", "TinyExpressionP4::ArgumentTernary", "TinyExpressionP4::ArgumentExpression", "TinyExpressionP4::Arguments", "TinyExpressionP4::NumberExpression", "TinyExpressionP4::NumberTerm", "TinyExpressionP4::AddOp", "TinyExpressionP4::MulOp", "TinyExpressionP4::MathFunction", "TinyExpressionP4::SinFunction", "TinyExpressionP4::CosFunction", "TinyExpressionP4::TanFunction", "TinyExpressionP4::SqrtFunction", "TinyExpressionP4::MinFunction", "TinyExpressionP4::MaxFunction", "TinyExpressionP4::RandomFunction", "TinyExpressionP4::AbsFunction", "TinyExpressionP4::RoundFunction", "TinyExpressionP4::CeilFunction", "TinyExpressionP4::FloorFunction", "TinyExpressionP4::PowFunction", "TinyExpressionP4::LogFunction", "TinyExpressionP4::ExpFunction", "TinyExpressionP4::ToNumFunction", "TinyExpressionP4::NumberFactor", "TinyExpressionP4::ToUpperCaseFunction", "TinyExpressionP4::ToLowerCaseFunction", "TinyExpressionP4::TrimFunction", "TinyExpressionP4::LengthFunction", "TinyExpressionP4::LenFunction", "TinyExpressionP4::ToUpperCaseDotMethod", "TinyExpressionP4::ToLowerCaseDotMethod", "TinyExpressionP4::TrimDotMethod", "TinyExpressionP4::LengthDotMethod", "TinyExpressionP4::StartsWithFunction", "TinyExpressionP4::EndsWithFunction", "TinyExpressionP4::ContainsFunction", "TinyExpressionP4::InMethod", "TinyExpressionP4::StartsWithDotMethod", "TinyExpressionP4::EndsWithDotMethod", "TinyExpressionP4::ContainsDotMethod", "TinyExpressionP4::StringPredicateReceiver", "TinyExpressionP4::IsPresentFunction", "TinyExpressionP4::InTimeRangeFunction", "TinyExpressionP4::InDayTimeRangeFunction", "TinyExpressionP4::DayOfWeek", "TinyExpressionP4::SliceBaseReceiver", "TinyExpressionP4::SliceStartIndex", "TinyExpressionP4::SliceEndIndex", "TinyExpressionP4::SliceStepIndex", "TinyExpressionP4::SliceBaseExpression", "TinyExpressionP4::SliceNestedExpression", "TinyExpressionP4::SliceExpression", "TinyExpressionP4::StringExpression", "TinyExpressionP4::ParenthesizedStringExpression", "TinyExpressionP4::StringTerm", "TinyExpressionP4::StringCastVariable", "TinyExpressionP4::StringTypedVariable", "TinyExpressionP4::BooleanExpression", "TinyExpressionP4::BooleanAndExpression", "TinyExpressionP4::BooleanXorExpression", "TinyExpressionP4::NotExpression", "TinyExpressionP4::BooleanComparable", "TinyExpressionP4::BooleanEqualityExpression", "TinyExpressionP4::BooleanFactor", "TinyExpressionP4::StringComparisonExpression", "TinyExpressionP4::EqualityOp", "TinyExpressionP4::ComparisonExpression", "TinyExpressionP4::CompareOp", "TinyExpressionP4::ObjectExpression", "TinyExpressionP4::IfExpression", "TinyExpressionP4::BranchExpression", "TinyExpressionP4::TernaryExpression", "TinyExpressionP4::NumberMatchExpression", "TinyExpressionP4::NumberCase", "TinyExpressionP4::NumberDefaultCase", "TinyExpressionP4::NumberCaseValue", "TinyExpressionP4::StringMatchExpression", "TinyExpressionP4::StringCase", "TinyExpressionP4::StringDefaultCase", "TinyExpressionP4::StringCaseValue", "TinyExpressionP4::BooleanMatchExpression", "TinyExpressionP4::BooleanCase", "TinyExpressionP4::BooleanDefaultCase", "TinyExpressionP4::BooleanCaseValue", "TinyExpressionP4::VariableRef", "TinyExpressionP4::TypeKeyword", "TinyExpressionP4::Expression"];
const EXPRESSIONS:&[&str]=&["expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:886:1045:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:886:899:body/0/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:888:897:body/0/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:900:930:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:902:919:body/1/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:931:968:body/2/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:933:952:body/2/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:973:987:body/3/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:975:985:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:988:998:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1011:1041:body/5/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1013:1030:body/5/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1042:1045:body/6/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1290:1319:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1290:1300:body/0/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1301:1310:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1311:1319:body/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1767:1850:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1767:1775:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1776:1785:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1797:1823:body/2/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1799:1813:body/2/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1799:1802:body/2/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1803:1813:body/2/0/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1824:1828:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1829:1839:body/4/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1847:1850:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1921:1962:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1921:1931:body/0/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1938:1962:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1940:1954:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1940:1943:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1944:1954:body/1/0/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2113:2235:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2113:2138:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2145:2170:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2177:2203:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2210:2235:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2408:2584:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2408:2430:body/0/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2410:2428:body/0/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2410:2420:body/0/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2423:2428:body/0/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2435:2438:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2439:2449:body/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2463:2481:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2465:2479:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2486:2550:body/4/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2488:2541:body/4/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2488:2493:body/4/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2494:2524:body/4/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2496:2508:body/4/0/1/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2525:2541:body/4/0/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2555:2576:body/5/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2557:2568:body/5/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2581:2584:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2757:2933:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2757:2779:body/0/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2759:2777:body/0/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2759:2769:body/0/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2772:2777:body/0/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2784:2787:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2788:2798:body/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2812:2830:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2814:2828:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2835:2899:body/4/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2837:2890:body/4/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2837:2842:body/4/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2843:2873:body/4/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2845:2857:body/4/0/1/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2874:2890:body/4/0/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2904:2925:body/5/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2906:2917:body/5/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2930:2933:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3108:3286:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3108:3130:body/0/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3110:3128:body/0/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3110:3120:body/0/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3123:3128:body/0/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3135:3138:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3139:3149:body/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3163:3182:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3165:3180:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3187:3252:body/4/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3189:3243:body/4/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3189:3194:body/4/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3195:3225:body/4/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3197:3209:body/4/0/1/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3226:3243:body/4/0/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3257:3278:body/5/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3259:3270:body/5/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3283:3286:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3459:3635:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3459:3481:body/0/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3461:3479:body/0/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3461:3471:body/0/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3474:3479:body/0/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3486:3489:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3490:3500:body/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3514:3532:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3516:3530:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3537:3601:body/4/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3539:3592:body/4/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3539:3544:body/4/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3545:3575:body/4/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3547:3559:body/4/0/1/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3576:3592:body/4/0/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3606:3627:body/5/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3608:3619:body/5/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3632:3635:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3654:3758:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3654:3662:body/0/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3656:3660:body/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3663:3758:body/1/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3665:3756:body/1/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3665:3673:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3676:3684:body/1/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3687:3695:body/1/0/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3698:3706:body/1/0/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3713:3722:body/1/0/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3725:3734:body/1/0/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3737:3745:body/1/0/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3748:3756:body/1/0/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3783:3835:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3783:3791:body/0/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3785:3789:body/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3792:3835:body/1/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3794:3833:body/1/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3794:3802:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3805:3813:body/1/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3816:3823:body/1/0/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3826:3833:body/1/0/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3859:3891:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3859:3867:body/0/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3861:3865:body/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3868:3891:body/1/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3870:3889:body/1/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3870:3878:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3881:3889:body/1/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3916:3950:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3916:3924:body/0/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3918:3922:body/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3925:3950:body/1/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3927:3948:body/1/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3927:3936:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3939:3948:body/1/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3974:4006:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3974:3982:body/0/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3976:3980:body/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3983:4006:body/1/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3985:4004:body/1/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3985:3993:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3996:4004:body/1/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4058:4077:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4058:4062:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4063:4068:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4069:4077:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4099:4123:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4099:4112:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4113:4116:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4117:4123:body/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4144:4191:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4144:4147:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4148:4158:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4159:4162:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4163:4187:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4165:4185:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4188:4191:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4221:4268:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4221:4240:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4241:4268:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4243:4266:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4243:4246:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4247:4266:body/1/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4297:4322:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4297:4307:body/0/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4308:4311:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4312:4322:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4356:4470:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4356:4379:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4386:4409:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4416:4440:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4447:4470:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4651:4796:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4651:4667:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4668:4678:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4695:4698:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4705:4737:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4707:4723:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4742:4745:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4750:4753:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4760:4776:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4793:4796:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4977:5122:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4977:4993:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4994:5004:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5021:5024:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5031:5063:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5033:5049:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5068:5071:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5076:5079:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5086:5102:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5119:5122:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5305:5452:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5305:5322:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5323:5333:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5350:5353:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5360:5392:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5362:5378:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5397:5400:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5405:5408:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5415:5432:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5449:5452:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5633:5778:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5633:5649:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5650:5660:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5677:5680:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5687:5719:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5689:5705:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5724:5727:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5732:5735:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5742:5758:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5775:5778:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5855:5910:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5855:5870:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5879:5910:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5881:5900:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5881:5884:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5885:5900:body/1/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6024:6075:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6024:6027:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6028:6038:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6050:6075:body/2/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6052:6067:body/2/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6052:6056:body/2/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6057:6067:body/2/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6102:6120:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6102:6110:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6113:6120:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6146:6154:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6146:6154:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6181:6190:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6181:6190:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6216:6224:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6216:6224:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6244:6318:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6244:6260:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6263:6279:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6282:6299:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6302:6318:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6644:6810:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6644:6654:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6655:6679:body/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6657:6677:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6657:6668:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6669:6677:body/1/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6671:6675:body/1/0/1/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6680:6697:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6702:6709:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6704:6707:body/3/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6714:6778:body/4/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6716:6770:body/4/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6716:6751:body/4/0/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6716:6725:body/4/0/0/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6737:6740:body/4/0/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6741:6751:body/4/0/0/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6760:6770:body/4/0/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6783:6786:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6787:6806:body/6/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6789:6798:body/6/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6807:6810:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6922:7101:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6922:6932:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6937:7000:body/1/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6939:6998:body/1/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6939:6988:body/1/0/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6939:6963:body/1/0/0/0/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6941:6961:body/1/0/0/0/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6941:6952:body/1/0/0/0/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6953:6961:body/1/0/0/0/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6955:6959:body/1/0/0/0/0/1/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6964:6980:body/1/0/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6981:6988:body/1/0/0/2/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6983:6986:body/1/0/0/2/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6991:6998:body/1/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6993:6996:body/1/0/1/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7005:7069:body/2/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7007:7061:body/2/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7007:7042:body/2/0/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7007:7016:body/2/0/0/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7028:7031:body/2/0/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7032:7042:body/2/0/0/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7051:7061:body/2/0/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7074:7077:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7078:7097:body/4/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7080:7089:body/4/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7098:7101:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7213:7378:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7213:7223:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7224:7248:body/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7226:7246:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7226:7237:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7238:7246:body/1/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7240:7244:body/1/0/1/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7249:7265:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7270:7277:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7272:7275:body/3/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7282:7346:body/4/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7284:7338:body/4/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7284:7319:body/4/0/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7284:7293:body/4/0/0/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7305:7308:body/4/0/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7309:7319:body/4/0/0/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7328:7338:body/4/0/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7351:7354:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7355:7374:body/6/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7357:7366:body/6/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7375:7378:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7490:7655:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7490:7500:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7501:7525:body/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7503:7523:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7503:7514:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7515:7523:body/1/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7517:7521:body/1/0/1/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7526:7542:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7547:7554:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7549:7552:body/3/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7559:7623:body/4/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7561:7615:body/4/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7561:7596:body/4/0/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7561:7570:body/4/0/0/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7582:7585:body/4/0/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7586:7596:body/4/0/0/2/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7605:7615:body/4/0/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7628:7631:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7632:7651:body/6/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7634:7643:body/6/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7652:7655:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7860:7894:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7860:7881:body/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7860:7866:body/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7867:7881:body/0/1/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7869:7879:body/0/1/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7884:7894:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7997:8064:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7997:8019:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8020:8030:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8037:8040:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8041:8060:body/3/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8043:8052:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8061:8064:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8231:8311:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8231:8248:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8260:8263:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8264:8280:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8291:8294:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8295:8311:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8407:8520:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8407:8422:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8436:8456:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8470:8496:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8510:8520:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8590:8651:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8590:8608:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8617:8651:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8619:8641:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8619:8622:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8623:8641:body/1/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8846:8894:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8846:8856:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8863:8894:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8865:8885:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8865:8870:body/1/0/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8875:8885:body/1/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9001:9053:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9001:9013:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9020:9053:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9022:9044:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9022:9027:body/1/0/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9032:9044:body/1/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9069:9078:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9069:9072:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9075:9078:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9093:9102:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9093:9096:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9099:9102:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9210:9464:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9210:9221:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9228:9239:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9246:9257:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9264:9276:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9283:9294:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9301:9312:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9319:9333:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9340:9351:body/7/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9358:9371:body/8/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9378:9390:body/9/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9397:9410:body/10/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9417:9428:body/11/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9435:9446:body/12/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9453:9464:body/13/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9520:9557:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9520:9525:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9526:9529:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9530:9548:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9554:9557:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9613:9650:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9613:9618:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9619:9622:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9623:9641:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9647:9650:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9706:9743:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9706:9711:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9712:9715:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9716:9734:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9740:9743:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9801:9839:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9801:9807:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9808:9811:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9812:9830:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9836:9839:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9903:9975:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9903:9908:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9909:9912:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9913:9931:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9939:9971:body/3/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9941:9963:body/3/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9941:9944:body/3/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9945:9963:body/3/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9972:9975:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10039:10111:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10039:10044:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10045:10048:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10049:10067:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10075:10107:body/3/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10077:10099:body/3/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10077:10080:body/3/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10081:10099:body/3/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10108:10111:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10159:10175:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10159:10167:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10168:10171:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10172:10175:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10231:10268:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10231:10236:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10237:10240:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10241:10259:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10265:10268:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10328:10367:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10328:10335:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10336:10339:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10340:10358:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10364:10367:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10425:10463:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10425:10431:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10432:10435:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10436:10454:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10460:10463:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10523:10562:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10523:10530:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10531:10534:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10535:10553:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10559:10562:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10629:10700:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10629:10634:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10635:10638:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10639:10657:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10664:10667:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10668:10686:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10697:10700:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10756:10793:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10756:10761:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10762:10765:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10766:10784:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10790:10793:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10849:10886:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10849:10854:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10855:10858:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10859:10877:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10883:10886:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11040:11116:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11040:11047:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11048:11051:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11052:11068:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11076:11079:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11080:11098:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11113:11116:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11145:11425:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11145:11162:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11169:11190:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11197:11209:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11216:11228:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11235:11248:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11255:11270:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11277:11288:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11295:11309:body/7/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11316:11340:body/8/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11347:11353:body/9/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11360:11371:body/10/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11378:11394:body/11/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11401:11425:body/12/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11401:11404:body/12/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11405:11421:body/12/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11422:11425:body/12/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11578:11623:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11578:11591:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11592:11595:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11596:11612:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11620:11623:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11697:11742:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11697:11710:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11711:11714:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11715:11731:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11739:11742:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11802:11840:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11802:11808:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11809:11812:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11813:11829:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11837:11840:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11904:11944:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11904:11912:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11913:11916:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11917:11933:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11941:11944:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12005:12042:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12005:12010:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12011:12014:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12015:12031:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12039:12042:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12367:12408:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12367:12378:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12386:12400:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12401:12404:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12405:12408:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12486:12527:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12486:12497:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12505:12519:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12520:12523:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12524:12527:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12591:12625:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12591:12602:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12610:12617:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12618:12621:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12622:12625:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12693:12729:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12693:12704:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12712:12721:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12722:12725:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12726:12729:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12949:13063:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12949:12961:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12962:12965:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12966:12982:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12990:12993:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12994:13010:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13025:13059:body/5/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13027:13047:body/5/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13027:13030:body/5/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13031:13047:body/5/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13060:13063:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13141:13253:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13141:13151:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13152:13155:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13156:13172:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13180:13183:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13184:13200:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13215:13249:body/5/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13217:13237:body/5/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13217:13220:body/5/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13221:13237:body/5/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13250:13253:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13331:13443:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13331:13341:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13342:13345:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13346:13362:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13370:13373:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13374:13390:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13405:13439:body/5/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13407:13427:body/5/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13407:13410:body/5/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13411:13427:body/5/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13440:13443:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13509:13612:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13509:13525:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13533:13538:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13539:13542:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13543:13559:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13572:13608:body/4/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13574:13594:body/4/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13574:13577:body/4/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13578:13594:body/4/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13609:13612:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13777:13895:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13777:13800:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13808:13821:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13822:13825:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13826:13842:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13857:13891:body/4/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13859:13879:body/4/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13859:13862:body/4/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13863:13879:body/4/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13892:13895:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13977:14093:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13977:14000:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14008:14019:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14020:14023:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14024:14040:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14055:14089:body/4/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14057:14077:body/4/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14057:14060:body/4/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14061:14077:body/4/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14090:14093:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14175:14291:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14175:14198:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14206:14217:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14218:14221:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14222:14238:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14253:14287:body/4/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14255:14275:body/4/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14255:14258:body/4/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14259:14275:body/4/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14288:14291:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14325:14395:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14325:14344:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14347:14366:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14369:14381:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14384:14395:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14544:14582:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14544:14555:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14556:14559:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14560:14571:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14579:14582:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14746:14825:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14746:14759:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14760:14763:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14764:14780:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14792:14795:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14796:14812:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14822:14825:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14936:15064:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14936:14952:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14953:14956:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14957:14966:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14977:14980:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14981:14997:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15009:15012:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15013:15022:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15031:15034:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15035:15051:body/8/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15061:15064:body/9/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15084:15166:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15084:15092:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15095:15104:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15107:15118:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15121:15131:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15134:15142:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15145:15155:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15158:15166:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15334:15603:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15334:15358:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15365:15384:body/1/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15365:15368:body/1/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15369:15380:body/1/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15381:15384:body/1/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15391:15420:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15427:15446:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15449:15468:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15471:15483:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15490:15510:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15513:15533:body/7/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15536:15549:body/8/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15556:15562:body/9/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15569:15580:body/10/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15587:15603:body/11/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16119:16135:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16119:16135:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16160:16176:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16160:16176:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16201:16217:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16201:16217:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16914:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16412:body/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16326:body/0/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16334:16337:body/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16338:16353:body/0/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16361:16364:body/0/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16365:16378:body/0/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16384:16387:body/0/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16388:16402:body/0/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16409:16412:body/0/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16419:16497:body/1/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16419:16436:body/1/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16444:16447:body/1/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16448:16463:body/1/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16471:16474:body/1/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16475:16488:body/1/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16494:16497:body/1/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16504:16588:body/2/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16504:16521:body/2/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16529:16532:body/2/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16533:16548:body/2/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16556:16559:body/2/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16560:16563:body/2/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16564:16578:body/2/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16585:16588:body/2/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16595:16654:body/3/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16595:16612:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16620:16623:body/3/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16624:16639:body/3/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16647:16650:body/3/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16651:16654:body/3/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16661:16741:body/4/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16661:16678:body/4/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16686:16689:body/4/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16690:16693:body/4/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16694:16707:body/4/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16713:16716:body/4/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16717:16731:body/4/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16738:16741:body/4/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16748:16803:body/5/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16748:16765:body/5/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16773:16776:body/5/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16777:16780:body/5/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16781:16794:body/5/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16800:16803:body/5/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16810:16871:body/6/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16810:16827:body/6/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16835:16838:body/6/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16839:16842:body/6/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16843:16846:body/6/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16847:16861:body/6/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16868:16871:body/6/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16878:16914:body/7/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16878:16895:body/7/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16903:16906:body/7/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16907:16910:body/7/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16911:16914:body/7/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17629:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17113:body/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17027:body/0/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17035:17038:body/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17039:17054:body/0/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17062:17065:body/0/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17066:17079:body/0/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17085:17088:body/0/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17089:17103:body/0/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17110:17113:body/0/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17120:17200:body/1/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17120:17139:body/1/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17147:17150:body/1/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17151:17166:body/1/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17174:17177:body/1/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17178:17191:body/1/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17197:17200:body/1/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17207:17293:body/2/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17207:17226:body/2/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17234:17237:body/2/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17238:17253:body/2/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17261:17264:body/2/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17265:17268:body/2/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17269:17283:body/2/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17290:17293:body/2/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17300:17361:body/3/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17300:17319:body/3/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17327:17330:body/3/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17331:17346:body/3/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17354:17357:body/3/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17358:17361:body/3/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17368:17450:body/4/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17368:17387:body/4/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17395:17398:body/4/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17399:17402:body/4/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17403:17416:body/4/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17422:17425:body/4/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17426:17440:body/4/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17447:17450:body/4/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17457:17514:body/5/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17457:17476:body/5/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17484:17487:body/5/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17488:17491:body/5/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17492:17505:body/5/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17511:17514:body/5/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17521:17584:body/6/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17521:17540:body/6/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17548:17551:body/6/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17552:17555:body/6/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17556:17559:body/6/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17560:17574:body/6/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17581:17584:body/6/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17591:17629:body/7/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17591:17610:body/7/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17618:17621:body/7/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17622:17625:body/7/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17626:17629:body/7/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17655:17698:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17655:17676:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17679:17698:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17899:17945:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17899:17909:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17916:17945:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17918:17936:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17918:17921:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17926:17936:body/1/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17985:18009:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17985:17988:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17989:18005:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18006:18009:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18216:18579:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18216:18237:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18244:18256:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18263:18281:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18288:18307:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18314:18329:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18336:18365:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18372:18396:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18403:18422:body/7/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18425:18444:body/8/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18447:18459:body/9/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18466:18486:body/10/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18489:18509:body/11/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18512:18525:body/12/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18532:18538:body/13/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18545:18556:body/14/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18563:18579:body/15/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18661:18707:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18661:18664:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18665:18688:body/1/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18667:18686:body/1/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18667:18675:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18678:18686:body/1/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18689:18692:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18693:18696:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18697:18707:body/4/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18797:18846:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18797:18800:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18801:18811:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18818:18822:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18823:18846:body/3/group", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18825:18844:body/3/0/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18825:18833:body/3/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18836:18844:body/3/0/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19077:19143:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19077:19097:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19104:19143:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19106:19134:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19106:19109:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19114:19134:body/1/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19280:19346:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19280:19300:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19307:19346:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19309:19337:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19309:19312:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19317:19337:body/1/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19483:19535:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19483:19496:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19503:19535:body/1/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19505:19526:body/1/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19505:19508:body/1/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19513:19526:body/1/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19673:19711:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19673:19678:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19679:19682:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19683:19700:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19708:19711:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19745:20177:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19745:19758:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19765:19777:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19784:19806:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19813:19838:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19845:19853:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19860:19879:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19886:19903:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19910:19927:body/7/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19934:19952:body/8/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19959:19975:body/9/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19982:19998:body/10/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20005:20022:body/11/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20029:20048:body/12/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20055:20077:body/13/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20084:20090:body/14/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20097:20104:body/15/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20111:20122:body/16/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20129:20145:body/17/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20152:20177:body/18/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20152:20155:body/18/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20156:20173:body/18/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20174:20177:body/18/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20271:20327:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20271:20288:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20295:20305:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20310:20327:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20410:20540:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20410:20435:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20449:20469:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20483:20509:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20523:20540:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20804:20858:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20804:20820:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20827:20837:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20842:20858:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20885:20896:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20885:20889:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20892:20896:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20980:21033:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20980:20996:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21003:21012:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21017:21033:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21059:21096:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21059:21063:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21066:21070:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21073:21077:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21080:21084:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21087:21090:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21093:21096:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21250:21420:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21250:21266:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21280:21296:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21310:21327:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21341:21365:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21379:21390:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21404:21420:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21595:21725:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21595:21599:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21600:21603:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21604:21621:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21633:21636:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21641:21644:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21645:21661:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21672:21675:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21680:21686:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21691:21694:body/8/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21695:21711:body/9/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21722:21725:body/10/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21807:22057:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21807:21827:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21841:21867:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21881:21906:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21920:21936:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21950:21967:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21981:21997:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22011:22027:body/6/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22041:22057:body/7/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22232:22330:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22232:22235:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22236:22253:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22265:22268:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22269:22285:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22296:22299:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22300:22316:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22327:22330:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22520:22638:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22520:22527:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22528:22531:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22538:22548:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22560:22589:body/3/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22562:22576:body/3/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22562:22565:body/3/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22566:22576:body/3/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22596:22599:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22600:22617:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22635:22638:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22713:22762:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22713:22730:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22742:22746:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22747:22762:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22846:22876:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22846:22855:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22856:22860:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22861:22876:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22956:22972:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22956:22972:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23087:23205:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23087:23094:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23095:23098:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23105:23115:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23127:23156:body/3/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23129:23143:body/3/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23129:23132:body/3/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23133:23143:body/3/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23163:23166:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23167:23184:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23202:23205:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23280:23329:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23280:23297:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23309:23313:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23314:23329:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23413:23443:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23413:23422:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23423:23427:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23428:23443:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23523:23539:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23523:23539:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23656:23777:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23656:23663:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23664:23667:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23674:23685:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23697:23727:body/3/repeat", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23699:23714:body/3/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23699:23702:body/3/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23703:23714:body/3/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23734:23737:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23738:23756:body/5/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23774:23777:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23854:23904:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23854:23871:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23883:23887:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23888:23904:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23990:24021:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23990:23999:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24000:24004:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24005:24021:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24103:24120:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24103:24120:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24333:24384:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24333:24336:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24337:24347:body/1/tokenRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24354:24384:body/2/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24356:24376:body/2/0/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24356:24364:body/2/0/0/optional", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24358:24362:body/2/0/0/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24365:24376:body/2/0/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24406:24517:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24406:24414:body/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24417:24425:body/1/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24428:24435:body/2/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24438:24445:body/3/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24452:24460:body/4/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24463:24471:body/5/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24474:24483:body/6/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24486:24495:body/7/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24498:24506:body/8/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24509:24517:body/9/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24836:25012:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24836:24852:body/0/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24866:24883:body/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24897:24913:body/2/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24927:24943:body/3/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24957:24973:body/4/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24987:25012:body/5/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24987:24990:body/5/0/literal", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24991:25001:body/5/1/ruleRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:25009:25012:body/5/2/literal"];
const BODIES:&[&str]=&["expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:886:1045:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1290:1319:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1767:1850:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1921:1962:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2113:2235:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2408:2584:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2757:2933:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3108:3286:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3459:3635:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3654:3758:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3783:3835:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3859:3891:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3916:3950:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3974:4006:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4058:4077:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4099:4123:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4144:4191:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4221:4268:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4297:4322:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4356:4470:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4651:4796:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4977:5122:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5305:5452:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5633:5778:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5855:5910:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6024:6075:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6102:6120:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6146:6154:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6181:6190:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6216:6224:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6244:6318:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6644:6810:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6922:7101:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7213:7378:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7490:7655:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7860:7894:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7997:8064:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8231:8311:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8407:8520:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8590:8651:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8846:8894:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9001:9053:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9069:9078:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9093:9102:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9210:9464:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9520:9557:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9613:9650:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9706:9743:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9801:9839:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9903:9975:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10039:10111:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10159:10175:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10231:10268:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10328:10367:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10425:10463:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10523:10562:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10629:10700:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10756:10793:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10849:10886:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11040:11116:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11145:11425:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11578:11623:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11697:11742:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11802:11840:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11904:11944:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12005:12042:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12367:12408:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12486:12527:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12591:12625:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12693:12729:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12949:13063:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13141:13253:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13331:13443:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13509:13612:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13777:13895:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13977:14093:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14175:14291:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14325:14395:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14544:14582:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14746:14825:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14936:15064:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15084:15166:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15334:15603:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16119:16135:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16160:16176:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16201:16217:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16914:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17629:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17655:17698:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17899:17945:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17985:18009:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18216:18579:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18661:18707:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18797:18846:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19077:19143:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19280:19346:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19483:19535:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19673:19711:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19745:20177:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20271:20327:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20410:20540:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20804:20858:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20885:20896:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20980:21033:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21059:21096:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21250:21420:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21595:21725:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21807:22057:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22232:22330:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22520:22638:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22713:22762:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22846:22876:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22956:22972:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23087:23205:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23280:23329:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23413:23443:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23523:23539:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23656:23777:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23854:23904:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23990:24021:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24103:24120:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24333:24384:body/seq", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24406:24517:body/choice", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24836:25012:body/choice"];
const SITES:&[(&str,&str)]=&[("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:902:919:body/1/0/ruleRef/capture/0", "imports"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:933:952:body/2/0/ruleRef/capture/0", "declarations"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:988:998:body/4/ruleRef/capture/0", "expression"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1013:1030:body/5/0/ruleRef/capture/0", "methods"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1776:1785:body/1/ruleRef/capture/0", "className"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1803:1813:body/2/0/1/tokenRef/capture/0", "method"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1829:1839:body/4/tokenRef/capture/0", "alias"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1921:1931:body/0/tokenRef/capture/0", "head"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1944:1954:body/1/0/1/tokenRef/capture/0", "tail"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2439:2449:body/2/tokenRef/capture/0", "varName"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2496:2508:body/4/0/1/0/ruleRef/capture/0", "onlyIfAbsent"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2525:2541:body/4/0/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2557:2568:body/5/0/ruleRef/capture/0", "desc"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2788:2798:body/2/tokenRef/capture/0", "varName"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2845:2857:body/4/0/1/0/ruleRef/capture/0", "onlyIfAbsent"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2874:2890:body/4/0/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2906:2917:body/5/0/ruleRef/capture/0", "desc"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3139:3149:body/2/tokenRef/capture/0", "varName"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3197:3209:body/4/0/1/0/ruleRef/capture/0", "onlyIfAbsent"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3226:3243:body/4/0/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3259:3270:body/5/0/ruleRef/capture/0", "desc"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3490:3500:body/2/tokenRef/capture/0", "varName"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3547:3559:body/4/0/1/0/ruleRef/capture/0", "onlyIfAbsent"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3576:3592:body/4/0/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3608:3619:body/5/0/ruleRef/capture/0", "desc"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4668:4678:body/1/tokenRef/capture/0", "methodName"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4707:4723:body/3/0/ruleRef/capture/0", "parameters"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4760:4776:body/6/ruleRef/capture/0", "expression"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4994:5004:body/1/tokenRef/capture/0", "methodName"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5033:5049:body/3/0/ruleRef/capture/0", "parameters"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5086:5102:body/6/ruleRef/capture/0", "expression"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5323:5333:body/1/tokenRef/capture/0", "methodName"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5362:5378:body/3/0/ruleRef/capture/0", "parameters"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5415:5432:body/6/ruleRef/capture/0", "expression"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5650:5660:body/1/tokenRef/capture/0", "methodName"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5689:5705:body/3/0/ruleRef/capture/0", "parameters"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5742:5758:body/6/ruleRef/capture/0", "expression"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5855:5870:body/0/ruleRef/capture/0", "values"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5885:5900:body/1/0/1/ruleRef/capture/0", "values"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6028:6038:body/1/tokenRef/capture/0", "paramName"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6057:6067:body/2/0/1/ruleRef/capture/0", "type"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6716:6725:body/4/0/0/0/ruleRef/capture/0", "className"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6741:6751:body/4/0/0/2/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6760:6770:body/4/0/1/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6789:6798:body/6/0/ruleRef/capture/0", "args"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7007:7016:body/2/0/0/0/ruleRef/capture/0", "className"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7032:7042:body/2/0/0/2/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7051:7061:body/2/0/1/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7080:7089:body/4/0/ruleRef/capture/0", "args"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7284:7293:body/4/0/0/0/ruleRef/capture/0", "className"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7309:7319:body/4/0/0/2/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7328:7338:body/4/0/1/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7357:7366:body/6/0/ruleRef/capture/0", "args"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7561:7570:body/4/0/0/0/ruleRef/capture/0", "className"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7586:7596:body/4/0/0/2/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7605:7615:body/4/0/1/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7634:7643:body/6/0/ruleRef/capture/0", "args"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8020:8030:body/1/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8043:8052:body/3/0/ruleRef/capture/0", "args"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8231:8248:body/0/ruleRef/capture/0", "condition"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8264:8280:body/2/ruleRef/capture/0", "thenExpr"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8295:8311:body/4/ruleRef/capture/0", "elseExpr"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8407:8422:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8436:8456:body/1/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8470:8496:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8510:8520:body/3/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8590:8608:body/0/ruleRef/capture/0", "values"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8623:8641:body/1/0/1/ruleRef/capture/0", "values"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8846:8856:body/0/ruleRef/capture/0", "left"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8865:8870:body/1/0/0/ruleRef/capture/0", "op"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8875:8885:body/1/0/1/ruleRef/capture/0", "right"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9001:9013:body/0/ruleRef/capture/0", "left"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9022:9027:body/1/0/0/ruleRef/capture/0", "op"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9032:9044:body/1/0/1/ruleRef/capture/0", "right"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9530:9548:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9623:9641:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9716:9734:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9812:9830:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9913:9931:body/2/ruleRef/capture/0", "first"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9945:9963:body/3/0/1/ruleRef/capture/0", "rest"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10049:10067:body/2/ruleRef/capture/0", "first"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10081:10099:body/3/0/1/ruleRef/capture/0", "rest"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10241:10259:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10340:10358:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10436:10454:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10535:10553:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10639:10657:body/2/ruleRef/capture/0", "base"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10668:10686:body/4/ruleRef/capture/0", "exponent"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10766:10784:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10859:10877:body/2/ruleRef/capture/0", "arg"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11052:11068:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11080:11098:body/4/ruleRef/capture/0", "defaultValue"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11596:11612:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11715:11731:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11813:11829:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11917:11933:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12015:12031:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12367:12378:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12486:12497:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12591:12602:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12693:12704:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12966:12982:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12994:13010:body/4/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13031:13047:body/5/0/1/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13156:13172:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13184:13200:body/4/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13221:13237:body/5/0/1/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13346:13362:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13374:13390:body/4/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13411:13427:body/5/0/1/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13509:13525:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13543:13559:body/3/ruleRef/capture/0", "candidates"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13578:13594:body/4/0/1/ruleRef/capture/0", "candidates"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13777:13800:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13826:13842:body/3/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13863:13879:body/4/0/1/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13977:14000:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14024:14040:body/3/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14061:14077:body/4/0/1/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14175:14198:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14222:14238:body/3/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14259:14275:body/4/0/1/ruleRef/capture/0", "patterns"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14560:14571:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14764:14780:body/2/ruleRef/capture/0", "startHour"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14796:14812:body/4/ruleRef/capture/0", "endHour"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14957:14966:body/2/ruleRef/capture/0", "startDay"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14981:14997:body/4/ruleRef/capture/0", "startHour"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15013:15022:body/6/ruleRef/capture/0", "endDay"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15035:15051:body/8/ruleRef/capture/0", "endHour"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16326:body/0/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16338:16353:body/0/2/ruleRef/capture/0", "start"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16365:16378:body/0/4/ruleRef/capture/0", "end"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16388:16402:body/0/6/ruleRef/capture/0", "step"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16419:16436:body/1/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16448:16463:body/1/2/ruleRef/capture/0", "start"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16475:16488:body/1/4/ruleRef/capture/0", "end"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16504:16521:body/2/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16533:16548:body/2/2/ruleRef/capture/0", "start"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16564:16578:body/2/5/ruleRef/capture/0", "step"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16595:16612:body/3/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16624:16639:body/3/2/ruleRef/capture/0", "start"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16661:16678:body/4/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16694:16707:body/4/3/ruleRef/capture/0", "end"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16717:16731:body/4/5/ruleRef/capture/0", "step"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16748:16765:body/5/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16781:16794:body/5/3/ruleRef/capture/0", "end"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16810:16827:body/6/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16847:16861:body/6/4/ruleRef/capture/0", "step"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16878:16895:body/7/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17027:body/0/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17039:17054:body/0/2/ruleRef/capture/0", "start"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17066:17079:body/0/4/ruleRef/capture/0", "end"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17089:17103:body/0/6/ruleRef/capture/0", "step"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17120:17139:body/1/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17151:17166:body/1/2/ruleRef/capture/0", "start"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17178:17191:body/1/4/ruleRef/capture/0", "end"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17207:17226:body/2/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17238:17253:body/2/2/ruleRef/capture/0", "start"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17269:17283:body/2/5/ruleRef/capture/0", "step"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17300:17319:body/3/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17331:17346:body/3/2/ruleRef/capture/0", "start"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17368:17387:body/4/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17403:17416:body/4/3/ruleRef/capture/0", "end"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17426:17440:body/4/5/ruleRef/capture/0", "step"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17457:17476:body/5/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17492:17505:body/5/3/ruleRef/capture/0", "end"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17521:17540:body/6/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17560:17574:body/6/4/ruleRef/capture/0", "step"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17591:17610:body/7/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17899:17909:body/0/ruleRef/capture/0", "left"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17918:17921:body/1/0/0/literal/capture/0", "op"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17926:17936:body/1/0/1/ruleRef/capture/0", "right"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18697:18707:body/4/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18801:18811:body/1/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19077:19097:body/0/ruleRef/capture/0", "left"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19106:19109:body/1/0/0/literal/capture/0", "op"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19114:19134:body/1/0/1/ruleRef/capture/0", "right"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19280:19300:body/0/ruleRef/capture/0", "left"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19309:19312:body/1/0/0/literal/capture/0", "op"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19317:19337:body/1/0/1/ruleRef/capture/0", "right"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19483:19496:body/0/ruleRef/capture/0", "left"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19505:19508:body/1/0/0/literal/capture/0", "op"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19513:19526:body/1/0/1/ruleRef/capture/0", "right"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19683:19700:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20271:20288:body/0/ruleRef/capture/0", "left"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20295:20305:body/1/ruleRef/capture/0", "op"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20310:20327:body/2/ruleRef/capture/0", "right"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20410:20435:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20449:20469:body/1/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20483:20509:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20523:20540:body/3/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20804:20820:body/0/ruleRef/capture/0", "left"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20827:20837:body/1/ruleRef/capture/0", "op"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20842:20858:body/2/ruleRef/capture/0", "right"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20980:20996:body/0/ruleRef/capture/0", "left"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21003:21012:body/1/ruleRef/capture/0", "op"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21017:21033:body/2/ruleRef/capture/0", "right"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21250:21266:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21280:21296:body/1/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21310:21327:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21341:21365:body/3/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21379:21390:body/4/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21404:21420:body/5/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21604:21621:body/2/ruleRef/capture/0", "condition"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21645:21661:body/5/ruleRef/capture/0", "thenExpr"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21695:21711:body/9/ruleRef/capture/0", "elseExpr"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21807:21827:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21841:21867:body/1/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21881:21906:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21920:21936:body/3/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21950:21967:body/4/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21981:21997:body/5/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22011:22027:body/6/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22041:22057:body/7/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22236:22253:body/1/ruleRef/capture/0", "condition"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22269:22285:body/3/ruleRef/capture/0", "thenExpr"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22300:22316:body/5/ruleRef/capture/0", "elseExpr"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22538:22548:body/2/ruleRef/capture/0", "firstCase"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22566:22576:body/3/0/1/ruleRef/capture/0", "moreCases"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22600:22617:body/5/ruleRef/capture/0", "defaultCase"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22713:22730:body/0/ruleRef/capture/0", "condition"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22747:22762:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22861:22876:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22956:22972:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23105:23115:body/2/ruleRef/capture/0", "firstCase"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23133:23143:body/3/0/1/ruleRef/capture/0", "moreCases"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23167:23184:body/5/ruleRef/capture/0", "defaultCase"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23280:23297:body/0/ruleRef/capture/0", "condition"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23314:23329:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23428:23443:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23523:23539:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23674:23685:body/2/ruleRef/capture/0", "firstCase"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23703:23714:body/3/0/1/ruleRef/capture/0", "moreCases"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23738:23756:body/5/ruleRef/capture/0", "defaultCase"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23854:23871:body/0/ruleRef/capture/0", "condition"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23888:23904:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24005:24021:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24103:24120:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24337:24347:body/1/tokenRef/capture/0", "name"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24365:24376:body/2/0/1/ruleRef/capture/0", "type"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24836:24852:body/0/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24866:24883:body/1/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24897:24913:body/2/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24927:24943:body/3/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24957:24973:body/4/ruleRef/capture/0", "value"), ("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24991:25001:body/5/1/ruleRef/capture/0", "value")];
impl Session<'_> {
#[inline(never)] fn build_values(&mut self,root:EventId)->Result<Vec<Ast>,String> {let frame=0u8;let address=std::ptr::addr_of!(frame) as usize;let anchor=*self.mapping_anchor.get_or_insert(address);if self.mapping_depth>=self.options.limits.mapping_depth || anchor.abs_diff(address)>256*1024 {return Err("maximum mapping depth exceeded".into());}self.mapping_depth+=1;let result=self.build_values_inner(root);self.mapping_depth-=1;result}
fn build_values_inner(&mut self,root:EventId)->Result<Vec<Ast>,String> {let mut values=Vec::new();let mut stack=vec![root];while let Some(id)=stack.pop() {match self.arena[id.0] {Event::Join(a,b)=>{stack.push(b);stack.push(a);},Event::Capture{child,..}=>stack.push(child),Event::Values{child,span,wrap}=>{let mut found=self.build_values(child)?;if wrap && (matches!(found.as_slice(),[Ast::Text(_)]) || (found.is_empty() && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child))) {found=vec![Ast::Text(self.semantic_text(child,span))];}values.extend(found);},Event::Rule{rule,span,child}=>values.extend(self.build_rule(rule,span,child)?),_=>{}}}Ok(values)}
fn field_nodes(&mut self,root:EventId,sites:&[usize],fallback:Option<usize>)->Result<Vec<Ast>,String> {let mut values=Vec::new();for (span,child) in self.owned_captures(root,sites) {let mut found=self.build_values(child)?;if found.is_empty() && span[0]!=span[1] && !self.has_recovery(child) && !self.has_value_group(child) {found.push(Ast::Text(self.semantic_text(child,span)));}for value in found {values.push(if let (Some(ty),Ast::Text(text))=(fallback,&value) {self.leaf(ty,span,text.clone())?} else {value});}}Ok(values)}
fn leaf(&mut self,ty:usize,span:Span,text:String)->Result<Ast,String> {let text=self.remember_text(text,span);let span=self.span(span);match ty {
24=>{let node_id=self.node(RULES[40],"BinaryExpr",span);Ok(Ast::g_TinyExpressionP4AST_2e_BinaryExpr(g_TinyExpressionP4AST_2e_BinaryExpr{span,node_id,g_left:None,g_op:vec![text],g_right:vec![]}))},
_=>{let _=(span,text);Err(format!("unknown leaf type {ty}"))}}}
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
59=>self.map_rule_59(span,child),
60=>self.map_rule_60(span,child),
61=>self.map_rule_61(span,child),
62=>self.map_rule_62(span,child),
63=>self.map_rule_63(span,child),
64=>self.map_rule_64(span,child),
65=>self.map_rule_65(span,child),
66=>self.map_rule_66(span,child),
67=>self.map_rule_67(span,child),
68=>self.map_rule_68(span,child),
69=>self.map_rule_69(span,child),
70=>self.map_rule_70(span,child),
71=>self.map_rule_71(span,child),
72=>self.map_rule_72(span,child),
73=>self.map_rule_73(span,child),
74=>self.map_rule_74(span,child),
75=>self.map_rule_75(span,child),
76=>self.map_rule_76(span,child),
77=>self.map_rule_77(span,child),
78=>self.map_rule_78(span,child),
79=>self.map_rule_79(span,child),
80=>self.map_rule_80(span,child),
81=>self.map_rule_81(span,child),
82=>self.map_rule_82(span,child),
83=>self.map_rule_83(span,child),
84=>self.map_rule_84(span,child),
85=>self.map_rule_85(span,child),
86=>self.map_rule_86(span,child),
87=>self.map_rule_87(span,child),
88=>self.map_rule_88(span,child),
89=>self.map_rule_89(span,child),
90=>self.map_rule_90(span,child),
91=>self.map_rule_91(span,child),
92=>self.map_rule_92(span,child),
93=>self.map_rule_93(span,child),
94=>self.map_rule_94(span,child),
95=>self.map_rule_95(span,child),
96=>self.map_rule_96(span,child),
97=>self.map_rule_97(span,child),
98=>self.map_rule_98(span,child),
99=>self.map_rule_99(span,child),
100=>self.map_rule_100(span,child),
101=>self.map_rule_101(span,child),
102=>self.map_rule_102(span,child),
103=>self.map_rule_103(span,child),
104=>self.map_rule_104(span,child),
105=>self.map_rule_105(span,child),
106=>self.map_rule_106(span,child),
107=>self.map_rule_107(span,child),
108=>self.map_rule_108(span,child),
109=>self.map_rule_109(span,child),
110=>self.map_rule_110(span,child),
111=>self.map_rule_111(span,child),
112=>self.map_rule_112(span,child),
113=>self.map_rule_113(span,child),
114=>self.map_rule_114(span,child),
115=>self.map_rule_115(span,child),
116=>self.map_rule_116(span,child),
117=>self.map_rule_117(span,child),
118=>self.map_rule_118(span,child),
119=>self.map_rule_119(span,child),
120=>self.map_rule_120(span,child),
121=>self.map_rule_121(span,child),
122=>self.map_rule_122(span,child),
123=>self.map_rule_123(span,child),
_=>Err(format!("unknown rule {rule}"))}}
fn map_rule_0(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_nodes(child,&[0],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ImportDeclarationExpr(_))) {return Err("imports: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let f1=self.field_nodes(child,&[1],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringVariableDeclarationExpr(_))) {return Err("declarations: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let mut f2=self.field_nodes(child,&[2],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpressionExpr(_))) {return Err("expression: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[2]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("expression requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let f3=self.field_nodes(child,&[3],None)?;
if !f3.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMethodDeclarationExpr(_))) {return Err("methods: mapped value type mismatch".into());}
let f3:Vec<Ast>=f3;
let span=self.span(span);let node_id=self.node(RULES[0],"FormulaExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_FormulaExpr(g_TinyExpressionP4AST_2e_FormulaExpr{span,node_id,
g_imports:f0,
g_declarations:f1,
g_expression:f2,
g_methods:f3,
})])}}
fn map_rule_1(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[1],"CodeBlockExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_CodeBlockExpr(g_TinyExpressionP4AST_2e_CodeBlockExpr{span,node_id,
})])}}
fn map_rule_2(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[4],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(_))) {return Err("className: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[4]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("className requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_text(child,&[5]);
if f1.len()>1 {return Err("method requires at most one value".into());}let f1=f1.pop();
let f1:Option<String>=f1;
let mut f2=self.field_text(child,&[6]);
if f2.is_empty() && self.field_missing(child,&[6]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("alias requires one value".into());}let f2=f2.remove(0);
let f2:String=f2;
let span=self.span(span);let node_id=self.node(RULES[2],"ImportDeclarationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ImportDeclarationExpr(g_TinyExpressionP4AST_2e_ImportDeclarationExpr{span,node_id,
g_className:f0,
g_method:f1,
g_alias:f2,
})])}}
fn map_rule_3(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[7]);
if f0.is_empty() && self.field_missing(child,&[7]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("head requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let f1=self.field_text(child,&[8]);
let f1:Vec<String>=f1;
let span=self.span(span);let node_id=self.node(RULES[3],"QualifiedNameExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(g_TinyExpressionP4AST_2e_QualifiedNameExpr{span,node_id,
g_head:f0,
g_tail:f1,
})])}}
fn map_rule_4(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_5(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[9]);
if f0.is_empty() && self.field_missing(child,&[9]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("varName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[10],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(_))) {return Err("onlyIfAbsent: mapped value type mismatch".into());}
if f1.len()>1 {return Err("onlyIfAbsent requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[11],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f2.len()>1 {return Err("value requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let mut f3=self.field_text(child,&[12]);
if f3.len()>1 {return Err("desc requires at most one value".into());}let f3=f3.pop();
let f3:Option<String>=f3;
let span=self.span(span);let node_id=self.node(RULES[5],"NumberVariableDeclarationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_NumberVariableDeclarationExpr(g_TinyExpressionP4AST_2e_NumberVariableDeclarationExpr{span,node_id,
g_varName:f0,
g_onlyIfAbsent:f1,
g_value:f2,
g_desc:f3,
})])}}
fn map_rule_6(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[13]);
if f0.is_empty() && self.field_missing(child,&[13]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("varName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[14],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(_))) {return Err("onlyIfAbsent: mapped value type mismatch".into());}
if f1.len()>1 {return Err("onlyIfAbsent requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[15],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f2.len()>1 {return Err("value requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let mut f3=self.field_text(child,&[16]);
if f3.len()>1 {return Err("desc requires at most one value".into());}let f3=f3.pop();
let f3:Option<String>=f3;
let span=self.span(span);let node_id=self.node(RULES[6],"StringVariableDeclarationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringVariableDeclarationExpr(g_TinyExpressionP4AST_2e_StringVariableDeclarationExpr{span,node_id,
g_varName:f0,
g_onlyIfAbsent:f1,
g_value:f2,
g_desc:f3,
})])}}
fn map_rule_7(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[17]);
if f0.is_empty() && self.field_missing(child,&[17]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("varName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[18],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(_))) {return Err("onlyIfAbsent: mapped value type mismatch".into());}
if f1.len()>1 {return Err("onlyIfAbsent requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[19],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f2.len()>1 {return Err("value requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let mut f3=self.field_text(child,&[20]);
if f3.len()>1 {return Err("desc requires at most one value".into());}let f3=f3.pop();
let f3:Option<String>=f3;
let span=self.span(span);let node_id=self.node(RULES[7],"BooleanVariableDeclarationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanVariableDeclarationExpr(g_TinyExpressionP4AST_2e_BooleanVariableDeclarationExpr{span,node_id,
g_varName:f0,
g_onlyIfAbsent:f1,
g_value:f2,
g_desc:f3,
})])}}
fn map_rule_8(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[21]);
if f0.is_empty() && self.field_missing(child,&[21]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("varName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[22],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(_))) {return Err("onlyIfAbsent: mapped value type mismatch".into());}
if f1.len()>1 {return Err("onlyIfAbsent requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[23],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f2.len()>1 {return Err("value requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let mut f3=self.field_text(child,&[24]);
if f3.len()>1 {return Err("desc requires at most one value".into());}let f3=f3.pop();
let f3:Option<String>=f3;
let span=self.span(span);let node_id=self.node(RULES[8],"ObjectVariableDeclarationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ObjectVariableDeclarationExpr(g_TinyExpressionP4AST_2e_ObjectVariableDeclarationExpr{span,node_id,
g_varName:f0,
g_onlyIfAbsent:f1,
g_value:f2,
g_desc:f3,
})])}}
fn map_rule_9(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_10(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_11(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_12(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_13(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_14(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[14],"OnlyIfAbsentExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr{span,node_id,
})])}}
fn map_rule_15(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_16(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_17(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_18(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_19(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_20(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[25]);
if f0.is_empty() && self.field_missing(child,&[25]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("methodName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[26],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParametersExpr(_))) {return Err("parameters: mapped value type mismatch".into());}
if f1.len()>1 {return Err("parameters requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[27],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("expression: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[27]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("expression requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[20],"NumberMethodDeclarationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_NumberMethodDeclarationExpr(g_TinyExpressionP4AST_2e_NumberMethodDeclarationExpr{span,node_id,
g_methodName:f0,
g_parameters:f1,
g_expression:f2,
})])}}
fn map_rule_21(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[28]);
if f0.is_empty() && self.field_missing(child,&[28]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("methodName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[29],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParametersExpr(_))) {return Err("parameters: mapped value type mismatch".into());}
if f1.len()>1 {return Err("parameters requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[30],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("expression: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[30]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("expression requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[21],"StringMethodDeclarationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringMethodDeclarationExpr(g_TinyExpressionP4AST_2e_StringMethodDeclarationExpr{span,node_id,
g_methodName:f0,
g_parameters:f1,
g_expression:f2,
})])}}
fn map_rule_22(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[31]);
if f0.is_empty() && self.field_missing(child,&[31]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("methodName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[32],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParametersExpr(_))) {return Err("parameters: mapped value type mismatch".into());}
if f1.len()>1 {return Err("parameters requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[33],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("expression: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[33]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("expression requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[22],"BooleanMethodDeclarationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanMethodDeclarationExpr(g_TinyExpressionP4AST_2e_BooleanMethodDeclarationExpr{span,node_id,
g_methodName:f0,
g_parameters:f1,
g_expression:f2,
})])}}
fn map_rule_23(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[34]);
if f0.is_empty() && self.field_missing(child,&[34]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("methodName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[35],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParametersExpr(_))) {return Err("parameters: mapped value type mismatch".into());}
if f1.len()>1 {return Err("parameters requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[36],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectExpr(_))) {return Err("expression: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[36]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("expression requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[23],"ObjectMethodDeclarationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ObjectMethodDeclarationExpr(g_TinyExpressionP4AST_2e_ObjectMethodDeclarationExpr{span,node_id,
g_methodName:f0,
g_parameters:f1,
g_expression:f2,
})])}}
fn map_rule_24(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_nodes(child,&[37, 38],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParameterExpr(_))) {return Err("values: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[24],"MethodParametersExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_MethodParametersExpr(g_TinyExpressionP4AST_2e_MethodParametersExpr{span,node_id,
g_values:f0,
})])}}
fn map_rule_25(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[39]);
if f0.is_empty() && self.field_missing(child,&[39]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("paramName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[40]);
if f1.len()>1 {return Err("type requires at most one value".into());}let f1=f1.pop();
let f1:Option<String>=f1;
let span=self.span(span);let node_id=self.node(RULES[25],"MethodParameterExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_MethodParameterExpr(g_TinyExpressionP4AST_2e_MethodParameterExpr{span,node_id,
g_paramName:f0,
g_type:f1,
})])}}
fn map_rule_26(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_27(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_28(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_29(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_30(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_31(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[41],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(_))) {return Err("className: mapped value type mismatch".into());}
if f0.len()>1 {return Err("className requires at most one node".into());}let f0=f0.pop().map(Box::new);
let f0:Option<Box<Ast>>=f0;
let mut f1=self.field_text(child,&[42, 43]);
if f1.is_empty() && self.field_missing(child,&[42, 43]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("name requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let mut f2=self.field_nodes(child,&[44],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(_))) {return Err("args: mapped value type mismatch".into());}
if f2.len()>1 {return Err("args requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let span=self.span(span);let node_id=self.node(RULES[31],"ExternalBooleanInvocationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr(g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr{span,node_id,
g_className:f0,
g_name:f1,
g_args:f2,
})])}}
fn map_rule_32(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[45],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(_))) {return Err("className: mapped value type mismatch".into());}
if f0.len()>1 {return Err("className requires at most one node".into());}let f0=f0.pop().map(Box::new);
let f0:Option<Box<Ast>>=f0;
let mut f1=self.field_text(child,&[46, 47]);
if f1.is_empty() && self.field_missing(child,&[46, 47]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("name requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let mut f2=self.field_nodes(child,&[48],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(_))) {return Err("args: mapped value type mismatch".into());}
if f2.len()>1 {return Err("args requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let span=self.span(span);let node_id=self.node(RULES[32],"ExternalNumberInvocationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ExternalNumberInvocationExpr(g_TinyExpressionP4AST_2e_ExternalNumberInvocationExpr{span,node_id,
g_className:f0,
g_name:f1,
g_args:f2,
})])}}
fn map_rule_33(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[49],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(_))) {return Err("className: mapped value type mismatch".into());}
if f0.len()>1 {return Err("className requires at most one node".into());}let f0=f0.pop().map(Box::new);
let f0:Option<Box<Ast>>=f0;
let mut f1=self.field_text(child,&[50, 51]);
if f1.is_empty() && self.field_missing(child,&[50, 51]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("name requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let mut f2=self.field_nodes(child,&[52],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(_))) {return Err("args: mapped value type mismatch".into());}
if f2.len()>1 {return Err("args requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let span=self.span(span);let node_id=self.node(RULES[33],"ExternalStringInvocationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr{span,node_id,
g_className:f0,
g_name:f1,
g_args:f2,
})])}}
fn map_rule_34(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[53],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(_))) {return Err("className: mapped value type mismatch".into());}
if f0.len()>1 {return Err("className requires at most one node".into());}let f0=f0.pop().map(Box::new);
let f0:Option<Box<Ast>>=f0;
let mut f1=self.field_text(child,&[54, 55]);
if f1.is_empty() && self.field_missing(child,&[54, 55]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("name requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let mut f2=self.field_nodes(child,&[56],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(_))) {return Err("args: mapped value type mismatch".into());}
if f2.len()>1 {return Err("args requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let span=self.span(span);let node_id=self.node(RULES[34],"ExternalObjectInvocationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ExternalObjectInvocationExpr(g_TinyExpressionP4AST_2e_ExternalObjectInvocationExpr{span,node_id,
g_className:f0,
g_name:f1,
g_args:f2,
})])}}
fn map_rule_35(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_36(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[57]);
if f0.is_empty() && self.field_missing(child,&[57]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[58],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(_))) {return Err("args: mapped value type mismatch".into());}
if f1.len()>1 {return Err("args requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let span=self.span(span);let node_id=self.node(RULES[36],"MethodInvocationExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(g_TinyExpressionP4AST_2e_MethodInvocationExpr{span,node_id,
g_name:f0,
g_args:f1,
})])}}
fn map_rule_37(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[59],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("condition: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[59]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("condition requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[60],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_))) {return Err("thenExpr: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[60]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("thenExpr requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let mut f2=self.field_nodes(child,&[61],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_))) {return Err("elseExpr: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[61]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("elseExpr requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[37],"TernaryExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_TernaryExpr(g_TinyExpressionP4AST_2e_TernaryExpr{span,node_id,
g_condition:f0,
g_thenExpr:f1,
g_elseExpr:f2,
})])}}
fn map_rule_38(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[62, 63, 64, 65],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TernaryExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[62, 63, 64, 65]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[38],"ArgumentExpressionExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(g_TinyExpressionP4AST_2e_ArgumentExpressionExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_39(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let f0=self.field_nodes(child,&[66, 67],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("values: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[39],"ArgumentsExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(g_TinyExpressionP4AST_2e_ArgumentsExpr{span,node_id,
g_values:f0,
})])}}
fn map_rule_40(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[68],Some(24))?;
if !f0.iter().all(|value|matches!(value,Ast::Null) || matches!(value,Ast::g_TinyExpressionP4AST_2e_AbsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanAndExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanEqualityExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanFactorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanXorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CeilExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CodeBlockExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CosExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalNumberInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalObjectInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_FloorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_FormulaExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IfExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ImportDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InDayTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IsPresentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LengthDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LengthExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LogExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MaxExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParameterExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParametersExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MinExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_PowExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_RandomExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_RoundExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SinExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SliceExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SqrtExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCastVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringTypedVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TanExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TernaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToNumExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("left: mapped value type mismatch".into());}
if f0.len()>1 {return Err("left requires at most one node".into());}let f0=f0.pop().map(Box::new);
let f0:Option<Box<Ast>>=f0;
let f1=self.field_text(child,&[69]);
let f1:Vec<String>=f1;
let f2=self.field_nodes(child,&[70],Some(24))?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_AbsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanAndExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanEqualityExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanFactorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanXorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CeilExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CodeBlockExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CosExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalNumberInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalObjectInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_FloorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_FormulaExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IfExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ImportDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InDayTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IsPresentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LengthDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LengthExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LogExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MaxExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParameterExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParametersExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MinExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_PowExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_RandomExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_RoundExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SinExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SliceExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SqrtExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCastVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringTypedVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TanExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TernaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToNumExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("right: mapped value type mismatch".into());}
let f2:Vec<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[40],"BinaryExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BinaryExpr(g_TinyExpressionP4AST_2e_BinaryExpr{span,node_id,
g_left:f0,
g_op:f1,
g_right:f2,
})])}}
fn map_rule_41(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[71],Some(24))?;
if !f0.iter().all(|value|matches!(value,Ast::Null) || matches!(value,Ast::g_TinyExpressionP4AST_2e_AbsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanAndExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanEqualityExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanFactorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanXorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CeilExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CodeBlockExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CosExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalNumberInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalObjectInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_FloorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_FormulaExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IfExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ImportDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InDayTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IsPresentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LengthDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LengthExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LogExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MaxExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParameterExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParametersExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MinExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_PowExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_RandomExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_RoundExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SinExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SliceExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SqrtExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCastVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringTypedVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TanExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TernaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToNumExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("left: mapped value type mismatch".into());}
if f0.len()>1 {return Err("left requires at most one node".into());}let f0=f0.pop().map(Box::new);
let f0:Option<Box<Ast>>=f0;
let f1=self.field_text(child,&[72]);
let f1:Vec<String>=f1;
let f2=self.field_nodes(child,&[73],Some(24))?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_AbsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanAndExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanEqualityExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanFactorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanXorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CeilExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CodeBlockExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_CosExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalNumberInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalObjectInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_FloorExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_FormulaExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IfExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ImportDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InDayTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IsPresentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LengthDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LengthExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_LogExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MaxExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParameterExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodParametersExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MinExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_PowExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_QualifiedNameExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_RandomExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_RoundExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SinExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SliceExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SqrtExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseValueExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCastVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringDefaultCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMethodDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringTypedVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringVariableDeclarationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TanExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TernaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToNumExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("right: mapped value type mismatch".into());}
let f2:Vec<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[41],"BinaryExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BinaryExpr(g_TinyExpressionP4AST_2e_BinaryExpr{span,node_id,
g_left:f0,
g_op:f1,
g_right:f2,
})])}}
fn map_rule_42(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_43(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_44(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_45(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[74],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[74]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[45],"SinExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_SinExpr(g_TinyExpressionP4AST_2e_SinExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_46(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[75],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[75]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[46],"CosExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_CosExpr(g_TinyExpressionP4AST_2e_CosExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_47(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[76],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[76]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[47],"TanExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_TanExpr(g_TinyExpressionP4AST_2e_TanExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_48(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[77],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[77]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[48],"SqrtExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_SqrtExpr(g_TinyExpressionP4AST_2e_SqrtExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_49(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[78],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("first: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[78]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("first requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[79],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("rest: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[49],"MinExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_MinExpr(g_TinyExpressionP4AST_2e_MinExpr{span,node_id,
g_first:f0,
g_rest:f1,
})])}}
fn map_rule_50(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[80],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("first: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[80]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("first requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[81],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("rest: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[50],"MaxExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_MaxExpr(g_TinyExpressionP4AST_2e_MaxExpr{span,node_id,
g_first:f0,
g_rest:f1,
})])}}
fn map_rule_51(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let span=self.span(span);let node_id=self.node(RULES[51],"RandomExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_RandomExpr(g_TinyExpressionP4AST_2e_RandomExpr{span,node_id,
})])}}
fn map_rule_52(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[82],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[82]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[52],"AbsExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_AbsExpr(g_TinyExpressionP4AST_2e_AbsExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_53(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[83],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[83]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[53],"RoundExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_RoundExpr(g_TinyExpressionP4AST_2e_RoundExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_54(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[84],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[84]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[54],"CeilExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_CeilExpr(g_TinyExpressionP4AST_2e_CeilExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_55(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[85],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[85]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[55],"FloorExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_FloorExpr(g_TinyExpressionP4AST_2e_FloorExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_56(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[86],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("base: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[86]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("base requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[87],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("exponent: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[87]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("exponent requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[56],"PowExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_PowExpr(g_TinyExpressionP4AST_2e_PowExpr{span,node_id,
g_base:f0,
g_exponent:f1,
})])}}
fn map_rule_57(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[88],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[88]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[57],"LogExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_LogExpr(g_TinyExpressionP4AST_2e_LogExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_58(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[89],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("arg: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[89]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("arg requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[58],"ExpExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ExpExpr(g_TinyExpressionP4AST_2e_ExpExpr{span,node_id,
g_arg:f0,
})])}}
fn map_rule_59(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[90],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[90]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[91],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(_))) {return Err("defaultValue: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[91]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("defaultValue requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[59],"ToNumExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ToNumExpr(g_TinyExpressionP4AST_2e_ToNumExpr{span,node_id,
g_value:f0,
g_defaultValue:f1,
})])}}
fn map_rule_60(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_61(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[92],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[92]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[61],"ToUpperCaseExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(g_TinyExpressionP4AST_2e_ToUpperCaseExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_62(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[93],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[93]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[62],"ToLowerCaseExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(g_TinyExpressionP4AST_2e_ToLowerCaseExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_63(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[94],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[94]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[63],"TrimExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_TrimExpr(g_TinyExpressionP4AST_2e_TrimExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_64(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[95],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[95]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[64],"LengthExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_LengthExpr(g_TinyExpressionP4AST_2e_LengthExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_65(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[96],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[96]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[65],"LengthExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_LengthExpr(g_TinyExpressionP4AST_2e_LengthExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_66(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[97],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[97]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[66],"ToUpperCaseDotExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_67(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[98],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[98]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[67],"ToLowerCaseDotExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_68(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[99],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[99]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[68],"TrimDotExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_TrimDotExpr(g_TinyExpressionP4AST_2e_TrimDotExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_69(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[100],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[100]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[69],"LengthDotExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_LengthDotExpr(g_TinyExpressionP4AST_2e_LengthDotExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_70(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[101],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[101]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[102, 103],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("patterns: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[70],"StartsWithExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StartsWithExpr(g_TinyExpressionP4AST_2e_StartsWithExpr{span,node_id,
g_value:f0,
g_patterns:f1,
})])}}
fn map_rule_71(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[104],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[104]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[105, 106],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("patterns: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[71],"EndsWithExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_EndsWithExpr(g_TinyExpressionP4AST_2e_EndsWithExpr{span,node_id,
g_value:f0,
g_patterns:f1,
})])}}
fn map_rule_72(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[107],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[107]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[108, 109],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("patterns: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[72],"ContainsExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ContainsExpr(g_TinyExpressionP4AST_2e_ContainsExpr{span,node_id,
g_value:f0,
g_patterns:f1,
})])}}
fn map_rule_73(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[110],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[110]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[111, 112],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("candidates: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[73],"InExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_InExpr(g_TinyExpressionP4AST_2e_InExpr{span,node_id,
g_value:f0,
g_candidates:f1,
})])}}
fn map_rule_74(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[113],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[113]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[114, 115],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("patterns: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[74],"StartsWithDotExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StartsWithDotExpr(g_TinyExpressionP4AST_2e_StartsWithDotExpr{span,node_id,
g_value:f0,
g_patterns:f1,
})])}}
fn map_rule_75(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[116],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[116]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[117, 118],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("patterns: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[75],"EndsWithDotExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_EndsWithDotExpr(g_TinyExpressionP4AST_2e_EndsWithDotExpr{span,node_id,
g_value:f0,
g_patterns:f1,
})])}}
fn map_rule_76(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[119],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[119]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[120, 121],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("patterns: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[76],"ContainsDotExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ContainsDotExpr(g_TinyExpressionP4AST_2e_ContainsDotExpr{span,node_id,
g_value:f0,
g_patterns:f1,
})])}}
fn map_rule_77(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_78(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[122],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[122]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[78],"IsPresentExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_IsPresentExpr(g_TinyExpressionP4AST_2e_IsPresentExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_79(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[123],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("startHour: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[123]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("startHour requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[124],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("endHour: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[124]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("endHour requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[79],"InTimeRangeExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_InTimeRangeExpr(g_TinyExpressionP4AST_2e_InTimeRangeExpr{span,node_id,
g_startHour:f0,
g_endHour:f1,
})])}}
fn map_rule_80(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[125]);
if f0.is_empty() && self.field_missing(child,&[125]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("startDay requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[126],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("startHour: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[126]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("startHour requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let mut f2=self.field_text(child,&[127]);
if f2.is_empty() && self.field_missing(child,&[127]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("endDay requires one value".into());}let f2=f2.remove(0);
let f2:String=f2;
let mut f3=self.field_nodes(child,&[128],None)?;
if !f3.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("endHour: mapped value type mismatch".into());}
if f3.is_empty() && self.field_missing(child,&[128]) {return Ok(vec![]);}
if f3.len()!=1 {return Err("endHour requires one node".into());}let f3=Box::new(f3.remove(0));
let f3:Box<Ast>=f3;
let span=self.span(span);let node_id=self.node(RULES[80],"InDayTimeRangeExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_InDayTimeRangeExpr(g_TinyExpressionP4AST_2e_InDayTimeRangeExpr{span,node_id,
g_startDay:f0,
g_startHour:f1,
g_endDay:f2,
g_endHour:f3,
})])}}
fn map_rule_81(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_82(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_83(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_84(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_85(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_86(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[129, 133, 136, 139, 141, 144, 146, 148],None)?;
if !f0.iter().all(|value|matches!(value,Ast::Text(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SliceExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[129, 133, 136, 139, 141, 144, 146, 148]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[130, 134, 137, 140],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("start: mapped value type mismatch".into());}
if f1.len()>1 {return Err("start requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[131, 135, 142, 145],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("end: mapped value type mismatch".into());}
if f2.len()>1 {return Err("end requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let mut f3=self.field_nodes(child,&[132, 138, 143, 147],None)?;
if !f3.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("step: mapped value type mismatch".into());}
if f3.len()>1 {return Err("step requires at most one node".into());}let f3=f3.pop().map(Box::new);
let f3:Option<Box<Ast>>=f3;
let span=self.span(span);let node_id=self.node(RULES[86],"SliceExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_SliceExpr(g_TinyExpressionP4AST_2e_SliceExpr{span,node_id,
g_value:f0,
g_start:f1,
g_end:f2,
g_step:f3,
})])}}
fn map_rule_87(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[149, 153, 156, 159, 161, 164, 166, 168],None)?;
if !f0.iter().all(|value|matches!(value,Ast::Text(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SliceExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[149, 153, 156, 159, 161, 164, 166, 168]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[150, 154, 157, 160],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("start: mapped value type mismatch".into());}
if f1.len()>1 {return Err("start requires at most one node".into());}let f1=f1.pop().map(Box::new);
let f1:Option<Box<Ast>>=f1;
let mut f2=self.field_nodes(child,&[151, 155, 162, 165],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("end: mapped value type mismatch".into());}
if f2.len()>1 {return Err("end requires at most one node".into());}let f2=f2.pop().map(Box::new);
let f2:Option<Box<Ast>>=f2;
let mut f3=self.field_nodes(child,&[152, 158, 163, 167],None)?;
if !f3.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("step: mapped value type mismatch".into());}
if f3.len()>1 {return Err("step requires at most one node".into());}let f3=f3.pop().map(Box::new);
let f3:Option<Box<Ast>>=f3;
let span=self.span(span);let node_id=self.node(RULES[87],"SliceExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_SliceExpr(g_TinyExpressionP4AST_2e_SliceExpr{span,node_id,
g_value:f0,
g_start:f1,
g_end:f2,
g_step:f3,
})])}}
fn map_rule_88(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_89(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[169],None)?;
if !f0.iter().all(|value|matches!(value,Ast::Text(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IfExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SliceExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCastVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringTypedVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("left: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[169]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("left requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_text(child,&[170]);
let f1:Vec<String>=f1;
let f2=self.field_nodes(child,&[171],None)?;
if !f2.iter().all(|value|matches!(value,Ast::Text(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IfExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_SliceExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCastVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringTypedVariableRefExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_TrimExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("right: mapped value type mismatch".into());}
let f2:Vec<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[89],"StringConcatExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(g_TinyExpressionP4AST_2e_StringConcatExpr{span,node_id,
g_left:f0,
g_op:f1,
g_right:f2,
})])}}
fn map_rule_90(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_91(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_92(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[172]);
if f0.is_empty() && self.field_missing(child,&[172]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[92],"StringCastVariableRefExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringCastVariableRefExpr(g_TinyExpressionP4AST_2e_StringCastVariableRefExpr{span,node_id,
g_name:f0,
})])}}
fn map_rule_93(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[173]);
if f0.is_empty() && self.field_missing(child,&[173]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[93],"StringTypedVariableRefExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringTypedVariableRefExpr(g_TinyExpressionP4AST_2e_StringTypedVariableRefExpr{span,node_id,
g_name:f0,
})])}}
fn map_rule_94(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[174],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanAndExpr(_))) {return Err("left: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[174]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("left requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_text(child,&[175]);
let f1:Vec<String>=f1;
let f2=self.field_nodes(child,&[176],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanAndExpr(_))) {return Err("right: mapped value type mismatch".into());}
let f2:Vec<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[94],"BooleanOrExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(g_TinyExpressionP4AST_2e_BooleanOrExpr{span,node_id,
g_left:f0,
g_op:f1,
g_right:f2,
})])}}
fn map_rule_95(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[177],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanXorExpr(_))) {return Err("left: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[177]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("left requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_text(child,&[178]);
let f1:Vec<String>=f1;
let f2=self.field_nodes(child,&[179],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanXorExpr(_))) {return Err("right: mapped value type mismatch".into());}
let f2:Vec<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[95],"BooleanAndExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanAndExpr(g_TinyExpressionP4AST_2e_BooleanAndExpr{span,node_id,
g_left:f0,
g_op:f1,
g_right:f2,
})])}}
fn map_rule_96(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[180],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanFactorExpr(_))) {return Err("left: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[180]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("left requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_text(child,&[181]);
let f1:Vec<String>=f1;
let f2=self.field_nodes(child,&[182],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanFactorExpr(_))) {return Err("right: mapped value type mismatch".into());}
let f2:Vec<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[96],"BooleanXorExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanXorExpr(g_TinyExpressionP4AST_2e_BooleanXorExpr{span,node_id,
g_left:f0,
g_op:f1,
g_right:f2,
})])}}
fn map_rule_97(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[183],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[183]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[97],"NotExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_NotExpr(g_TinyExpressionP4AST_2e_NotExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_98(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_99(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[184],None)?;
if !f0.iter().all(|value|matches!(value,Ast::Text(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IfExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InDayTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IsPresentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("left: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[184]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("left requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_text(child,&[185]);
if f1.is_empty() && self.field_missing(child,&[185]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("op requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let mut f2=self.field_nodes(child,&[186],None)?;
if !f2.iter().all(|value|matches!(value,Ast::Text(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IfExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InDayTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IsPresentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("right: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[186]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("right requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[99],"BooleanEqualityExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanEqualityExpr(g_TinyExpressionP4AST_2e_BooleanEqualityExpr{span,node_id,
g_left:f0,
g_op:f1,
g_right:f2,
})])}}
fn map_rule_100(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[187, 188, 189, 190],None)?;
if !f0.iter().all(|value|matches!(value,Ast::Text(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanEqualityExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanMatchExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ContainsExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_EndsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IfExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InDayTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_InTimeRangeExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_IsPresentExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_NotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithDotExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StartsWithExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[187, 188, 189, 190]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[100],"BooleanFactorExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanFactorExpr(g_TinyExpressionP4AST_2e_BooleanFactorExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_101(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[191],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("left: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[191]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("left requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_text(child,&[192]);
if f1.is_empty() && self.field_missing(child,&[192]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("op requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let mut f2=self.field_nodes(child,&[193],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("right: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[193]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("right requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[101],"StringComparisonExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringComparisonExpr(g_TinyExpressionP4AST_2e_StringComparisonExpr{span,node_id,
g_left:f0,
g_op:f1,
g_right:f2,
})])}}
fn map_rule_102(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_103(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[194],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("left: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[194]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("left requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_text(child,&[195]);
if f1.is_empty() && self.field_missing(child,&[195]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("op requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let mut f2=self.field_nodes(child,&[196],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("right: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[196]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("right requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[103],"ComparisonExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ComparisonExpr(g_TinyExpressionP4AST_2e_ComparisonExpr{span,node_id,
g_left:f0,
g_op:f1,
g_right:f2,
})])}}
fn map_rule_104(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_105(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[197, 198, 199, 200, 201, 202],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExternalObjectInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[197, 198, 199, 200, 201, 202]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[105],"ObjectExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ObjectExpr(g_TinyExpressionP4AST_2e_ObjectExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_106(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[203],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("condition: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[203]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("condition requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[204],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_))) {return Err("thenExpr: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[204]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("thenExpr requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let mut f2=self.field_nodes(child,&[205],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_))) {return Err("elseExpr: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[205]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("elseExpr requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[106],"IfExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_IfExpr(g_TinyExpressionP4AST_2e_IfExpr{span,node_id,
g_condition:f0,
g_thenExpr:f1,
g_elseExpr:f2,
})])}}
fn map_rule_107(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[206, 207, 208, 209, 210, 211, 212, 213],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanEqualityExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringComparisonExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[206, 207, 208, 209, 210, 211, 212, 213]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[107],"BranchExpressionExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(g_TinyExpressionP4AST_2e_BranchExpressionExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_108(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[214],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("condition: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[214]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("condition requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[215],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_))) {return Err("thenExpr: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[215]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("thenExpr requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let mut f2=self.field_nodes(child,&[216],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BranchExpressionExpr(_))) {return Err("elseExpr: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[216]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("elseExpr requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[108],"TernaryExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_TernaryExpr(g_TinyExpressionP4AST_2e_TernaryExpr{span,node_id,
g_condition:f0,
g_thenExpr:f1,
g_elseExpr:f2,
})])}}
fn map_rule_109(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[217],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseExpr(_))) {return Err("firstCase: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[217]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("firstCase requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[218],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseExpr(_))) {return Err("moreCases: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let mut f2=self.field_nodes(child,&[219],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberDefaultCaseExpr(_))) {return Err("defaultCase: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[219]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("defaultCase requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[109],"NumberMatchExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_NumberMatchExpr(g_TinyExpressionP4AST_2e_NumberMatchExpr{span,node_id,
g_firstCase:f0,
g_moreCases:f1,
g_defaultCase:f2,
})])}}
fn map_rule_110(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[220],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("condition: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[220]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("condition requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[221],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseValueExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[221]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("value requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[110],"NumberCaseExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_NumberCaseExpr(g_TinyExpressionP4AST_2e_NumberCaseExpr{span,node_id,
g_condition:f0,
g_value:f1,
})])}}
fn map_rule_111(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[222],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_NumberCaseValueExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[222]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[111],"NumberDefaultCaseExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_NumberDefaultCaseExpr(g_TinyExpressionP4AST_2e_NumberDefaultCaseExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_112(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[223],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[223]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[112],"NumberCaseValueExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_NumberCaseValueExpr(g_TinyExpressionP4AST_2e_NumberCaseValueExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_113(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[224],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseExpr(_))) {return Err("firstCase: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[224]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("firstCase requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[225],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseExpr(_))) {return Err("moreCases: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let mut f2=self.field_nodes(child,&[226],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringDefaultCaseExpr(_))) {return Err("defaultCase: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[226]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("defaultCase requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[113],"StringMatchExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringMatchExpr(g_TinyExpressionP4AST_2e_StringMatchExpr{span,node_id,
g_firstCase:f0,
g_moreCases:f1,
g_defaultCase:f2,
})])}}
fn map_rule_114(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[227],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("condition: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[227]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("condition requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[228],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseValueExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[228]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("value requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[114],"StringCaseExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringCaseExpr(g_TinyExpressionP4AST_2e_StringCaseExpr{span,node_id,
g_condition:f0,
g_value:f1,
})])}}
fn map_rule_115(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[229],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringCaseValueExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[229]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[115],"StringDefaultCaseExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringDefaultCaseExpr(g_TinyExpressionP4AST_2e_StringDefaultCaseExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_116(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[230],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[230]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[116],"StringCaseValueExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_StringCaseValueExpr(g_TinyExpressionP4AST_2e_StringCaseValueExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_117(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[231],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseExpr(_))) {return Err("firstCase: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[231]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("firstCase requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let f1=self.field_nodes(child,&[232],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseExpr(_))) {return Err("moreCases: mapped value type mismatch".into());}
let f1:Vec<Ast>=f1;
let mut f2=self.field_nodes(child,&[233],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanDefaultCaseExpr(_))) {return Err("defaultCase: mapped value type mismatch".into());}
if f2.is_empty() && self.field_missing(child,&[233]) {return Ok(vec![]);}
if f2.len()!=1 {return Err("defaultCase requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[117],"BooleanMatchExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanMatchExpr(g_TinyExpressionP4AST_2e_BooleanMatchExpr{span,node_id,
g_firstCase:f0,
g_moreCases:f1,
g_defaultCase:f2,
})])}}
fn map_rule_118(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[234],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("condition: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[234]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("condition requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let mut f1=self.field_nodes(child,&[235],None)?;
if !f1.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseValueExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f1.is_empty() && self.field_missing(child,&[235]) {return Ok(vec![]);}
if f1.len()!=1 {return Err("value requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[118],"BooleanCaseExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanCaseExpr(g_TinyExpressionP4AST_2e_BooleanCaseExpr{span,node_id,
g_condition:f0,
g_value:f1,
})])}}
fn map_rule_119(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[236],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanCaseValueExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[236]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[119],"BooleanDefaultCaseExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanDefaultCaseExpr(g_TinyExpressionP4AST_2e_BooleanDefaultCaseExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_120(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[237],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[237]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[120],"BooleanCaseValueExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_BooleanCaseValueExpr(g_TinyExpressionP4AST_2e_BooleanCaseValueExpr{span,node_id,
g_value:f0,
})])}}
fn map_rule_121(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_text(child,&[238]);
if f0.is_empty() && self.field_missing(child,&[238]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[239]);
if f1.len()>1 {return Err("type requires at most one value".into());}let f1=f1.pop();
let f1:Option<String>=f1;
let span=self.span(span);let node_id=self.node(RULES[121],"VariableRefExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_VariableRefExpr(g_TinyExpressionP4AST_2e_VariableRefExpr{span,node_id,
g_name:f0,
g_type:f1,
})])}}
fn map_rule_122(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{let mut values=self.build_values(child)?;if values.is_empty() && !self.has_recovery(child) && !self.has_value_group(child) {values.push(Ast::Text(self.semantic_text(child,span)));}Ok(values)}}
fn map_rule_123(&mut self,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);{
let mut f0=self.field_nodes(child,&[240, 241, 242, 243, 244, 245],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_TinyExpressionP4AST_2e_BinaryExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_BooleanOrExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ExpressionExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_MethodInvocationExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_ObjectExpr(_)) || matches!(value,Ast::g_TinyExpressionP4AST_2e_StringConcatExpr(_))) {return Err("value: mapped value type mismatch".into());}
if f0.is_empty() && self.field_missing(child,&[240, 241, 242, 243, 244, 245]) {return Ok(vec![]);}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[123],"ExpressionExpr",span);Ok(vec![Ast::g_TinyExpressionP4AST_2e_ExpressionExpr(g_TinyExpressionP4AST_2e_ExpressionExpr{span,node_id,
g_value:f0,
})])}}
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
    if self.diag.exhausted || self.display.arena.exhausted {
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
        ast.collect("", &self.text_spans, &mut value_spans);
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
trait CollectValueSpans {
    fn collect(
        &self,
        path: &str,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    );
}
impl CollectValueSpans for String {
    fn collect(
        &self,
        path: &str,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    ) {
        if let Some(span) = texts.get(&(self.as_ptr() as usize)) {
            out.push(ValueSpan {
                path: path.into(),
                span: *span,
                text: None,
            });
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Vec<T> {
    fn collect(
        &self,
        path: &str,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    ) {
        for (i, value) in self.iter().enumerate() {
            value.collect(&format!("{path}/{i}"), texts, out);
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Option<T> {
    fn collect(
        &self,
        path: &str,
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
        path: &str,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    ) {
        self.as_ref().collect(path, texts, out);
    }
}
impl CollectValueSpans for Ast { fn collect(&self,path:&str,texts:&std::collections::HashMap<usize,Span>,out:&mut Vec<ValueSpan>) { match self { Self::Text(text)=>{if let Some(span)=texts.get(&(text.as_ptr() as usize)) {out.push(ValueSpan{path:path.into(),span:*span,text:Some(text.clone())});}},Self::Null=>{},
Self::g_TinyExpressionP4AST_2e_FormulaExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_imports.collect(&format!("{path}/fields/imports"),texts,out);
node.g_declarations.collect(&format!("{path}/fields/declarations"),texts,out);
node.g_expression.collect(&format!("{path}/fields/expression"),texts,out);
node.g_methods.collect(&format!("{path}/fields/methods"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_CodeBlockExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
},
Self::g_TinyExpressionP4AST_2e_ImportDeclarationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_className.collect(&format!("{path}/fields/className"),texts,out);
node.g_method.collect(&format!("{path}/fields/method"),texts,out);
node.g_alias.collect(&format!("{path}/fields/alias"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_QualifiedNameExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_head.collect(&format!("{path}/fields/head"),texts,out);
node.g_tail.collect(&format!("{path}/fields/tail"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_NumberVariableDeclarationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_varName.collect(&format!("{path}/fields/varName"),texts,out);
node.g_onlyIfAbsent.collect(&format!("{path}/fields/onlyIfAbsent"),texts,out);
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_desc.collect(&format!("{path}/fields/desc"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringVariableDeclarationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_varName.collect(&format!("{path}/fields/varName"),texts,out);
node.g_onlyIfAbsent.collect(&format!("{path}/fields/onlyIfAbsent"),texts,out);
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_desc.collect(&format!("{path}/fields/desc"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanVariableDeclarationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_varName.collect(&format!("{path}/fields/varName"),texts,out);
node.g_onlyIfAbsent.collect(&format!("{path}/fields/onlyIfAbsent"),texts,out);
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_desc.collect(&format!("{path}/fields/desc"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ObjectVariableDeclarationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_varName.collect(&format!("{path}/fields/varName"),texts,out);
node.g_onlyIfAbsent.collect(&format!("{path}/fields/onlyIfAbsent"),texts,out);
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_desc.collect(&format!("{path}/fields/desc"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_OnlyIfAbsentExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
},
Self::g_TinyExpressionP4AST_2e_NumberMethodDeclarationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_methodName.collect(&format!("{path}/fields/methodName"),texts,out);
node.g_parameters.collect(&format!("{path}/fields/parameters"),texts,out);
node.g_expression.collect(&format!("{path}/fields/expression"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringMethodDeclarationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_methodName.collect(&format!("{path}/fields/methodName"),texts,out);
node.g_parameters.collect(&format!("{path}/fields/parameters"),texts,out);
node.g_expression.collect(&format!("{path}/fields/expression"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanMethodDeclarationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_methodName.collect(&format!("{path}/fields/methodName"),texts,out);
node.g_parameters.collect(&format!("{path}/fields/parameters"),texts,out);
node.g_expression.collect(&format!("{path}/fields/expression"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ObjectMethodDeclarationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_methodName.collect(&format!("{path}/fields/methodName"),texts,out);
node.g_parameters.collect(&format!("{path}/fields/parameters"),texts,out);
node.g_expression.collect(&format!("{path}/fields/expression"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_MethodParametersExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_values.collect(&format!("{path}/fields/values"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_MethodParameterExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_paramName.collect(&format!("{path}/fields/paramName"),texts,out);
node.g_type.collect(&format!("{path}/fields/type"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ExternalBooleanInvocationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_className.collect(&format!("{path}/fields/className"),texts,out);
node.g_name.collect(&format!("{path}/fields/name"),texts,out);
node.g_args.collect(&format!("{path}/fields/args"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ExternalNumberInvocationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_className.collect(&format!("{path}/fields/className"),texts,out);
node.g_name.collect(&format!("{path}/fields/name"),texts,out);
node.g_args.collect(&format!("{path}/fields/args"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ExternalStringInvocationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_className.collect(&format!("{path}/fields/className"),texts,out);
node.g_name.collect(&format!("{path}/fields/name"),texts,out);
node.g_args.collect(&format!("{path}/fields/args"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ExternalObjectInvocationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_className.collect(&format!("{path}/fields/className"),texts,out);
node.g_name.collect(&format!("{path}/fields/name"),texts,out);
node.g_args.collect(&format!("{path}/fields/args"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_MethodInvocationExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_name.collect(&format!("{path}/fields/name"),texts,out);
node.g_args.collect(&format!("{path}/fields/args"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_TernaryExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_condition.collect(&format!("{path}/fields/condition"),texts,out);
node.g_thenExpr.collect(&format!("{path}/fields/thenExpr"),texts,out);
node.g_elseExpr.collect(&format!("{path}/fields/elseExpr"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ArgumentExpressionExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ArgumentsExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_values.collect(&format!("{path}/fields/values"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BinaryExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_left.collect(&format!("{path}/fields/left"),texts,out);
node.g_op.collect(&format!("{path}/fields/op"),texts,out);
node.g_right.collect(&format!("{path}/fields/right"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_SinExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_CosExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_TanExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_SqrtExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_MinExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_first.collect(&format!("{path}/fields/first"),texts,out);
node.g_rest.collect(&format!("{path}/fields/rest"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_MaxExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_first.collect(&format!("{path}/fields/first"),texts,out);
node.g_rest.collect(&format!("{path}/fields/rest"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_RandomExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
},
Self::g_TinyExpressionP4AST_2e_AbsExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_RoundExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_CeilExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_FloorExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_PowExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_base.collect(&format!("{path}/fields/base"),texts,out);
node.g_exponent.collect(&format!("{path}/fields/exponent"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_LogExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ExpExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_arg.collect(&format!("{path}/fields/arg"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ToNumExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_defaultValue.collect(&format!("{path}/fields/defaultValue"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ToUpperCaseExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ToLowerCaseExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_TrimExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_LengthExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ToUpperCaseDotExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ToLowerCaseDotExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_TrimDotExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_LengthDotExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StartsWithExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_patterns.collect(&format!("{path}/fields/patterns"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_EndsWithExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_patterns.collect(&format!("{path}/fields/patterns"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ContainsExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_patterns.collect(&format!("{path}/fields/patterns"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_InExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_candidates.collect(&format!("{path}/fields/candidates"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StartsWithDotExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_patterns.collect(&format!("{path}/fields/patterns"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_EndsWithDotExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_patterns.collect(&format!("{path}/fields/patterns"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ContainsDotExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_patterns.collect(&format!("{path}/fields/patterns"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_IsPresentExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_InTimeRangeExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_startHour.collect(&format!("{path}/fields/startHour"),texts,out);
node.g_endHour.collect(&format!("{path}/fields/endHour"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_InDayTimeRangeExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_startDay.collect(&format!("{path}/fields/startDay"),texts,out);
node.g_startHour.collect(&format!("{path}/fields/startHour"),texts,out);
node.g_endDay.collect(&format!("{path}/fields/endDay"),texts,out);
node.g_endHour.collect(&format!("{path}/fields/endHour"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_SliceExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
node.g_start.collect(&format!("{path}/fields/start"),texts,out);
node.g_end.collect(&format!("{path}/fields/end"),texts,out);
node.g_step.collect(&format!("{path}/fields/step"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringConcatExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_left.collect(&format!("{path}/fields/left"),texts,out);
node.g_op.collect(&format!("{path}/fields/op"),texts,out);
node.g_right.collect(&format!("{path}/fields/right"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringCastVariableRefExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_name.collect(&format!("{path}/fields/name"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringTypedVariableRefExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_name.collect(&format!("{path}/fields/name"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanOrExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_left.collect(&format!("{path}/fields/left"),texts,out);
node.g_op.collect(&format!("{path}/fields/op"),texts,out);
node.g_right.collect(&format!("{path}/fields/right"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanAndExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_left.collect(&format!("{path}/fields/left"),texts,out);
node.g_op.collect(&format!("{path}/fields/op"),texts,out);
node.g_right.collect(&format!("{path}/fields/right"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanXorExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_left.collect(&format!("{path}/fields/left"),texts,out);
node.g_op.collect(&format!("{path}/fields/op"),texts,out);
node.g_right.collect(&format!("{path}/fields/right"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_NotExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanEqualityExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_left.collect(&format!("{path}/fields/left"),texts,out);
node.g_op.collect(&format!("{path}/fields/op"),texts,out);
node.g_right.collect(&format!("{path}/fields/right"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanFactorExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringComparisonExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_left.collect(&format!("{path}/fields/left"),texts,out);
node.g_op.collect(&format!("{path}/fields/op"),texts,out);
node.g_right.collect(&format!("{path}/fields/right"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ComparisonExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_left.collect(&format!("{path}/fields/left"),texts,out);
node.g_op.collect(&format!("{path}/fields/op"),texts,out);
node.g_right.collect(&format!("{path}/fields/right"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ObjectExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_IfExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_condition.collect(&format!("{path}/fields/condition"),texts,out);
node.g_thenExpr.collect(&format!("{path}/fields/thenExpr"),texts,out);
node.g_elseExpr.collect(&format!("{path}/fields/elseExpr"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BranchExpressionExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_NumberMatchExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_firstCase.collect(&format!("{path}/fields/firstCase"),texts,out);
node.g_moreCases.collect(&format!("{path}/fields/moreCases"),texts,out);
node.g_defaultCase.collect(&format!("{path}/fields/defaultCase"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_NumberCaseExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_condition.collect(&format!("{path}/fields/condition"),texts,out);
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_NumberDefaultCaseExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_NumberCaseValueExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringMatchExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_firstCase.collect(&format!("{path}/fields/firstCase"),texts,out);
node.g_moreCases.collect(&format!("{path}/fields/moreCases"),texts,out);
node.g_defaultCase.collect(&format!("{path}/fields/defaultCase"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringCaseExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_condition.collect(&format!("{path}/fields/condition"),texts,out);
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringDefaultCaseExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_StringCaseValueExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanMatchExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_firstCase.collect(&format!("{path}/fields/firstCase"),texts,out);
node.g_moreCases.collect(&format!("{path}/fields/moreCases"),texts,out);
node.g_defaultCase.collect(&format!("{path}/fields/defaultCase"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanCaseExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_condition.collect(&format!("{path}/fields/condition"),texts,out);
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanDefaultCaseExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_BooleanCaseValueExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_VariableRefExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_name.collect(&format!("{path}/fields/name"),texts,out);
node.g_type.collect(&format!("{path}/fields/type"),texts,out);
},
Self::g_TinyExpressionP4AST_2e_ExpressionExpr(node)=>{out.push(ValueSpan{path:path.into(),span:node.span,text:None});
node.g_value.collect(&format!("{path}/fields/value"),texts,out);
},
}}}
