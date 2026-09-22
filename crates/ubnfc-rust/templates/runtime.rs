use super::api::*;
use super::rt::{
    diag::{Diag, Diagnostics},
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
    Capture {
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
}
impl Saved {
    fn new(step: Step, effects: Span) -> Self {
        Self {
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
    pub memo: Memo<Saved>,
    pub scope: ScopeStore,
    pub trivia_cache: Memo<Step>,
    effects: Vec<Effect>,
    saved_effects: Vec<Effect>,
    pub statistics: Statistics,
    pub node_spans: Vec<NodeSpan>,
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
            memo: Memo::with_capacity(text.len().saturating_mul(10).saturating_add(64).min(32768)),
            scope: ScopeStore::new(),
            trivia_cache: Memo::default(),
            effects: vec![],
            saved_effects: vec![],
            statistics: Statistics::default(),
            node_spans: vec![],
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
        if self.diag.exhausted {
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
            .trim_matches(|c: char| c <= '\u{20}')
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
                        &format!("undefined: {name}"),
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
        self.memo.insert(key, Saved::new(step, effects));
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
            Step::yes(state)
        } else {
            self.fail(state, state.position::<MATCH>(), label)
        }
    }
    pub fn number<const MATCH: bool>(&mut self, state: State, label: &'static str) -> Step {
        let start = state.position::<MATCH>();
        let bytes = self.text.as_bytes();
        let mut p = start;
        if matches!(bytes.get(p), Some(b'+' | b'-')) {
            p += 1;
        }
        let digits = p;
        while bytes.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1;
        }
        let mut count = p - digits;
        if bytes.get(p) == Some(&b'.') {
            p += 1;
            let fraction = p;
            while bytes.get(p).is_some_and(u8::is_ascii_digit) {
                p += 1;
            }
            count += p - fraction;
        }
        if count == 0 {
            return self.primitive::<MATCH>(state, None, label);
        }
        if matches!(bytes.get(p), Some(b'e' | b'E')) {
            let exponent = p;
            p += 1;
            if matches!(bytes.get(p), Some(b'+' | b'-')) {
                p += 1;
            }
            let digits = p;
            while bytes.get(p).is_some_and(u8::is_ascii_digit) {
                p += 1;
            }
            if digits == p {
                p = exponent;
            }
        }
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
            return self.primitive::<MATCH>(state, None, label);
        }
        p += 1;
        while let Some((c, n)) = self.input.cp_at(p) {
            p += n;
            if c == quote {
                return self.primitive::<MATCH>(state, Some(p - start), label);
            }
            if c == '\\' {
                if let Some((_, n)) = self.input.cp_at(p) {
                    p += n;
                } else {
                    break;
                }
            }
        }
        self.primitive::<MATCH>(state, None, label)
    }
    pub fn until<const MATCH: bool>(&mut self, state: State, terminator: &'static str) -> Step {
        let mut out = state.begin();
        loop {
            if !terminator.is_empty() && self.input.starts_with(out.matched, terminator) {
                if !MATCH {
                    out.matched += terminator.len();
                }
                return Step::yes(state.commit(out));
            }
            let Some((_, n)) = self.input.cp_at(out.position::<MATCH>()) else {
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
                Event::Capture { site, span, child } => {
                    if done {
                        if sites.contains(&site) {
                            out.push((span, child));
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
                Event::Capture { child, .. } => stack.push(child),
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
                Event::Rule { child, .. } | Event::Capture { child, .. } => stack.push(child),
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
                Event::Capture { child, .. } | Event::Rule { child, .. } => stack.push(child),
                _ => {}
            }
        }
        false
    }
    pub fn directly_recovered(&self, mut root: EventId) -> bool {
        loop {
            match self.arena[root.0] {
                Event::Recovery { .. } => return true,
                Event::Capture { child, .. } => root = child,
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
    pub fn field_text(&self, root: EventId, sites: &[usize]) -> Vec<String> {
        self.owned_captures(root, sites)
            .into_iter()
            .filter(|(_, child)| !self.directly_recovered(*child))
            .map(|(span, _)| self.text(span))
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
                Event::Capture { site, span, child } => {
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
