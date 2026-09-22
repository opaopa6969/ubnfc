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
pub(crate) struct Session<'a> {
    pub text: &'a str,
    pub input: Input<'a>,
    pub options: ParseOptions,
    pub scanner: &'a mut dyn TokenScanner,
    pub arena: Vec<Event>,
    pub diag: Diagnostics,
    pub memo: Memo<Saved>,
    pub scope: ScopeStore,
    pub trivia_cache: Memo<State>,
    effects: Vec<ScanEffect>,
    saved_effects: Vec<ScanEffect>,
    pub statistics: Statistics,
    pub node_spans: Vec<NodeSpan>,
}
impl<'a> Session<'a> {
    pub fn new(text: &'a str, options: ParseOptions, scanner: &'a mut dyn TokenScanner) -> Self {
        Self {
            text,
            input: Input::new(text),
            options,
            scanner,
            arena: {let mut arena=Vec::with_capacity(if options.build_ast || options.lexical || HAS_SCOPE {text.len().saturating_mul(8).saturating_add(8).min(32768)} else {1});arena.push(Event::Empty);arena},
            diag: Diagnostics::new(),
            memo: Memo::with_capacity(text.len().saturating_mul(10).saturating_add(64).min(32768)),
            scope: ScopeStore::new(),
            trivia_cache: Memo::default(),
            effects: vec![],
            saved_effects: vec![],
            statistics: Statistics::default(),
            node_spans: vec![],
        }
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
        self.saved_effects
            .extend_from_slice(&self.effects[mark.effects..]);
        [start, self.saved_effects.len()]
    }
    pub fn replay_effects(&mut self, span: Span) {
        for i in span[0]..span[1] {
            self.effect(self.saved_effects[i].clone());
        }
    }
    pub fn effect(&mut self, effect: ScanEffect) {
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
        self.effects.push(effect);
    }
    pub fn lookup(&mut self, key: Key) -> Option<Step> {
        let saved = self.memo.get(key)?;
        self.statistics.memo_hits += 1;
        self.replay_effects(saved.effects.map(|p| p as usize));
        Some(saved.step())
    }
    pub fn store(&mut self, key: Key, step: Step, mark: Mark) {
        let effects = if step.ok {
            self.save_effects(mark)
        } else {
            [0, 0]
        };
        self.memo.insert(key, Saved::new(step, effects));
    }
    pub fn event(&mut self, event: Event) -> EventId {
        if !self.options.build_ast && !self.options.lexical && !HAS_SCOPE {
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
                let actual = match actual {
                    '\u{130}' | '\u{131}' => 'i',
                    '\u{17f}' => 's',
                    '\u{212a}' => 'k',
                    c => c.to_ascii_lowercase(),
                };
                if actual != expected.to_ascii_lowercase() {
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
                Event::Rule { span: child, .. } | Event::Token { span: child } => {
                    span = Some(span.map_or(child, |s: Span| [s[0], child[1]]));
                }
                Event::Empty => {}
            }
        }
        span.unwrap_or(fallback)
    }
    pub fn field_text(&self, root: EventId, sites: &[usize]) -> Vec<String> {
        self.owned_captures(root, sites)
            .into_iter()
            .map(|(span, _)| self.text(span))
            .collect()
    }
    pub fn scope_capture(&self, root: EventId, site: usize) -> Option<Span> {
        self.owned_captures(root, &[site])
            .last()
            .map(|(span, _)| *span)
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
const HAS_SCOPE:bool=true;
const RULES:&[&str]=&["UBNF::UBNFFile", "UBNF::ImportDecl", "UBNF::GrammarDecl", "UBNF::GlobalSetting", "UBNF::SettingValue", "UBNF::StringSettingValue", "UBNF::BlockSettingValue", "UBNF::KeyValuePair", "UBNF::TokenDecl", "UBNF::TokenValue", "UBNF::UntilExpression", "UBNF::NegationExpression", "UBNF::LookaheadExpression", "UBNF::NegativeLookaheadExpression", "UBNF::CharRangeExpression", "UBNF::CIExpression", "UBNF::RegexExpression", "UBNF::AnyExpression", "UBNF::EofExpression", "UBNF::EmptyExpression", "UBNF::RuleDecl", "UBNF::Annotation", "UBNF::RootAnnotation", "UBNF::MappingAnnotation", "UBNF::EvalAnnotation", "UBNF::WhitespaceAnnotation", "UBNF::InterleaveAnnotation", "UBNF::BackrefAnnotation", "UBNF::ScopeTreeAnnotation", "UBNF::DeclaresAnnotation", "UBNF::CatalogAnnotation", "UBNF::LeftAssocAnnotation", "UBNF::RightAssocAnnotation", "UBNF::LongestChoiceAnnotation", "UBNF::PredictiveChoiceAnnotation", "UBNF::PrecedenceAnnotation", "UBNF::DocAnnotation", "UBNF::RecoveryAnnotation", "UBNF::SkipAnnotation", "UBNF::CommonFieldAnnotation", "UBNF::EnumAnnotation", "UBNF::SimpleAnnotation", "UBNF::ChoiceBody", "UBNF::SequenceBody", "UBNF::AnnotatedElement", "UBNF::PostfixQuantifier", "UBNF::SeparatedBy", "UBNF::BoundedQuantifier", "UBNF::Digits", "UBNF::AtomicElement", "UBNF::GroupElement", "UBNF::OptionalElement", "UBNF::RepeatElement", "UBNF::ErrorElement", "UBNF::TerminalElement", "UBNF::QuantifiedRef", "UBNF::TypeofElement", "UBNF::DottedIdentifier", "UBNF::RuleBody"];
const BODIES:&[&str]=&["expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1326:1367:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1507:1685:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1785:1821:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1860:1903:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2084:2117:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2215:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2357:2396:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2455:2717:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2821:2855:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:2958:2990:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3100:3135:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3270:3314:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3424:3472:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3558:3583:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3681:3712:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3775:3780:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3843:3848:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:3917:3924:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4199:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4255:4764:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4819:4826:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:4924:5066:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5139:5206:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5289:5332:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5417:5472:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5548:5594:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5674:5722:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5815:5912:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:5991:6039:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6104:6116:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6182:6195:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6265:6281:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6357:6376:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6458:6515:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6583:6610:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6700:6772:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6827:6834:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:6997:7073:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7181:7188:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7262:7276:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7396:7457:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7543:7563:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7727:7912:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:7961:7976:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8025:8042:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8100:8133:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8152:8167:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8314:8434:body/choice", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8524:8546:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8636:8658:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8749:8771:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8871:8902:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:8997:9003:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9214:9254:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9381:9422:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9526:9569:body/seq", "expr:unlaxer-dsl/grammar/ubnf.ubnf:9627:9637:body/seq"];
const SITES:&[(&str,&str)]=&[("expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/0/repeat/capture/0", "grammars"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1336:1346:body/1/tokenRef/capture/0", "alias"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1361:1367:body/3/tokenRef/capture/0", "path"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1517:1527:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1550:1564:body/3/repeat/capture/0", "imports"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1584:1601:body/4/repeat/capture/0", "settings"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1619:1632:body/5/repeat/capture/0", "tokens"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1652:1664:body/6/repeat/capture/0", "rules"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1789:1799:body/1/tokenRef/capture/0", "key"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1809:1821:body/3/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/0/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2088:2104:body/1/repeat/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2199:body/0/tokenRef/capture/0", "key"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2209:2215:body/2/tokenRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2365:2375:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2396:body/3/ruleRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2833:2839:body/2/tokenRef/capture/0", "terminator"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:2973:2979:body/2/tokenRef/capture/0", "chars"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3116:3122:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3295:3301:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3441:3447:body/2/tokenRef/capture/0", "min"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3457:3463:body/4/tokenRef/capture/0", "max"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3567:3573:body/2/tokenRef/capture/0", "word"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:3693:3699:body/2/tokenRef/capture/0", "pattern"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4129:body/0/repeat/capture/0", "annotations"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4148:4158:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4176:4184:body/3/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4946:4956:body/2/tokenRef/capture/0", "className"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:4998:5008:body/3/0/4/tokenRef/capture/0", "paramNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5027:5037:body/3/0/5/0/1/tokenRef/capture/0", "paramNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5151:5163:body/2/ruleRef/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5179:5191:body/3/0/1/ruleRef/capture/0", "entries"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5309:5319:body/1/0/1/tokenRef/capture/0", "style"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5449:5459:body/4/tokenRef/capture/0", "profile"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5574:5584:body/4/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5702:5712:body/4/tokenRef/capture/0", "mode"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5844:5854:body/4/tokenRef/capture/0", "symbol"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:5887:5893:body/5/0/3/tokenRef/capture/0", "description"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6020:6026:body/4/tokenRef/capture/0", "context"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6488:6504:body/4/tokenRef/capture/0", "level"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6594:6600:body/2/tokenRef/capture/0", "text"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6729:6735:body/2/0/0/2/tokenRef/capture/0", "syncTokens"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:6750:6760:body/2/0/1/tokenRef/capture/0", "mode"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7016:7026:body/2/tokenRef/capture/0", "fieldNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7045:7055:body/3/0/1/tokenRef/capture/0", "fieldNames"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7266:7276:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7396:7408:body/0/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7429:7441:body/1/0/1/ruleRef/capture/0", "alternatives"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7543:7563:body/0/repeat/capture/0", "elements"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7729:7742:body/0/0/ruleRef/capture/0", "typeofConstraint"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7770:7783:body/1/ruleRef/capture/0", "element"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:7887:7897:body/5/0/1/tokenRef/capture/0", "captureName"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8528:8536:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8640:8648:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8753:8761:body/1/ruleRef/capture/0", "body"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8883:8889:body/2/tokenRef/capture/0", "message"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:8997:9003:body/0/tokenRef/capture/0", "value"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9216:9226:body/0/0/0/tokenRef/capture/0", "namespace"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9244:9254:body/1/tokenRef/capture/0", "name"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9395:9405:body/2/tokenRef/capture/0", "captureName"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9526:9536:body/0/tokenRef/capture/0", "parts"), ("expr:unlaxer-dsl/grammar/ubnf.ubnf:9550:9560:body/1/0/1/tokenRef/capture/0", "parts")];
impl Session<'_> {
fn build_values(&mut self,root:EventId)->Result<Vec<Ast>,String> {let mut values=Vec::new();let mut stack=vec![root];while let Some(id)=stack.pop() {match self.arena[id.0] {Event::Join(a,b)=>{stack.push(b);stack.push(a);},Event::Capture{child,..}=>stack.push(child),Event::Rule{rule,span,child}=>values.extend(self.build_rule(rule,span,child)?),_=>{}}}Ok(values)}
fn field_nodes(&mut self,root:EventId,sites:&[usize],fallback:Option<usize>)->Result<Vec<Ast>,String> {let mut values=Vec::new();for (span,child) in self.owned_captures(root,sites) {let mut found=self.build_values(child)?;if found.is_empty() && span[0]!=span[1] {found.push(Ast::Text(self.text(span)));}for value in found {values.push(if let (Some(ty),Ast::Text(text))=(fallback,&value) {self.leaf(ty,span,text.clone())?} else {value});}}Ok(values)}
fn leaf(&mut self,ty:usize,span:Span,text:String)->Result<Ast,String> {let span=self.span(span);let _=(span,text);Err(format!("unknown leaf type {ty}"))}
fn build_rule(&mut self,rule:usize,span:Span,child:EventId)->Result<Vec<Ast>,String> {let _=(span,child);match rule {
0=>{
let f0=self.field_nodes(child,&[0],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_GrammarDecl(_))) {return Err("grammars: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[0],"UBNFFile",span);Ok(vec![Ast::g_UBNFAST_2e_UBNFFile(g_UBNFAST_2e_UBNFFile{span,node_id,
g_grammars:f0,
})])},
1=>{
let mut f0=self.field_text(child,&[1]);
if f0.len()!=1 {return Err("alias requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[2]);
if f1.len()!=1 {return Err("path requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let span=self.span(span);let node_id=self.node(RULES[1],"ImportDecl",span);Ok(vec![Ast::g_UBNFAST_2e_ImportDecl(g_UBNFAST_2e_ImportDecl{span,node_id,
g_alias:f0,
g_path:f1,
})])},
2=>{
let mut f0=self.field_text(child,&[3]);
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
})])},
3=>{
let mut f0=self.field_text(child,&[8]);
if f0.len()!=1 {return Err("key requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[9],None)?;
if !f1.iter().all(|value|(matches!(value,Ast::g_UBNFAST_2e_StringSettingValue(_)) || matches!(value,Ast::g_UBNFAST_2e_BlockSettingValue(_)))) {return Err("value: mapped value type mismatch".into());}
if f1.len()!=1 {return Err("value requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[3],"GlobalSetting",span);Ok(vec![Ast::g_UBNFAST_2e_GlobalSetting(g_UBNFAST_2e_GlobalSetting{span,node_id,
g_key:f0,
g_value:f1,
})])},
4=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
5=>{
let mut f0=self.field_nodes(child,&[10],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_DottedIdentifier(_))) {return Err("value: mapped value type mismatch".into());}
if f0.len()!=1 {return Err("value requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[5],"StringSettingValue",span);Ok(vec![Ast::g_UBNFAST_2e_StringSettingValue(g_UBNFAST_2e_StringSettingValue{span,node_id,
g_value:f0,
})])},
6=>{
let f0=self.field_nodes(child,&[11],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_KeyValuePair(_))) {return Err("entries: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[6],"BlockSettingValue",span);Ok(vec![Ast::g_UBNFAST_2e_BlockSettingValue(g_UBNFAST_2e_BlockSettingValue{span,node_id,
g_entries:f0,
})])},
7=>{
let mut f0=self.field_text(child,&[12]);
if f0.len()!=1 {return Err("key requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[13]);
if f1.len()!=1 {return Err("value requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let span=self.span(span);let node_id=self.node(RULES[7],"KeyValuePair",span);Ok(vec![Ast::g_UBNFAST_2e_KeyValuePair(g_UBNFAST_2e_KeyValuePair{span,node_id,
g_key:f0,
g_value:f1,
})])},
8=>{
let mut f0=self.field_text(child,&[14]);
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_nodes(child,&[15],None)?;
if !f1.iter().all(|value|(matches!(value,Ast::g_UBNFAST_2e_UntilExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_NegationExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_LookaheadExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_NegativeLookaheadExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_CharRangeExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_CIExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_RegexExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_AnyExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_EofExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_EmptyExpression(_)) || matches!(value,Ast::g_UBNFAST_2e_DottedIdentifier(_)))) {return Err("value: mapped value type mismatch".into());}
if f1.len()!=1 {return Err("value requires one node".into());}let f1=Box::new(f1.remove(0));
let f1:Box<Ast>=f1;
let span=self.span(span);let node_id=self.node(RULES[8],"TokenDecl",span);Ok(vec![Ast::g_UBNFAST_2e_TokenDecl(g_UBNFAST_2e_TokenDecl{span,node_id,
g_name:f0,
g_value:f1,
})])},
9=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
10=>{
let mut f0=self.field_text(child,&[16]);
if f0.len()!=1 {return Err("terminator requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[10],"UntilExpression",span);Ok(vec![Ast::g_UBNFAST_2e_UntilExpression(g_UBNFAST_2e_UntilExpression{span,node_id,
g_terminator:f0,
})])},
11=>{
let mut f0=self.field_text(child,&[17]);
if f0.len()!=1 {return Err("chars requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[11],"NegationExpression",span);Ok(vec![Ast::g_UBNFAST_2e_NegationExpression(g_UBNFAST_2e_NegationExpression{span,node_id,
g_chars:f0,
})])},
12=>{
let mut f0=self.field_text(child,&[18]);
if f0.len()!=1 {return Err("pattern requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[12],"LookaheadExpression",span);Ok(vec![Ast::g_UBNFAST_2e_LookaheadExpression(g_UBNFAST_2e_LookaheadExpression{span,node_id,
g_pattern:f0,
})])},
13=>{
let mut f0=self.field_text(child,&[19]);
if f0.len()!=1 {return Err("pattern requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[13],"NegativeLookaheadExpression",span);Ok(vec![Ast::g_UBNFAST_2e_NegativeLookaheadExpression(g_UBNFAST_2e_NegativeLookaheadExpression{span,node_id,
g_pattern:f0,
})])},
14=>{
let mut f0=self.field_text(child,&[20]);
if f0.len()!=1 {return Err("min requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[21]);
if f1.len()!=1 {return Err("max requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let span=self.span(span);let node_id=self.node(RULES[14],"CharRangeExpression",span);Ok(vec![Ast::g_UBNFAST_2e_CharRangeExpression(g_UBNFAST_2e_CharRangeExpression{span,node_id,
g_min:f0,
g_max:f1,
})])},
15=>{
let mut f0=self.field_text(child,&[22]);
if f0.len()!=1 {return Err("word requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[15],"CIExpression",span);Ok(vec![Ast::g_UBNFAST_2e_CIExpression(g_UBNFAST_2e_CIExpression{span,node_id,
g_word:f0,
})])},
16=>{
let mut f0=self.field_text(child,&[23]);
if f0.len()!=1 {return Err("pattern requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[16],"RegexExpression",span);Ok(vec![Ast::g_UBNFAST_2e_RegexExpression(g_UBNFAST_2e_RegexExpression{span,node_id,
g_pattern:f0,
})])},
17=>{
let span=self.span(span);let node_id=self.node(RULES[17],"AnyExpression",span);Ok(vec![Ast::g_UBNFAST_2e_AnyExpression(g_UBNFAST_2e_AnyExpression{span,node_id,
})])},
18=>{
let span=self.span(span);let node_id=self.node(RULES[18],"EofExpression",span);Ok(vec![Ast::g_UBNFAST_2e_EofExpression(g_UBNFAST_2e_EofExpression{span,node_id,
})])},
19=>{
let span=self.span(span);let node_id=self.node(RULES[19],"EmptyExpression",span);Ok(vec![Ast::g_UBNFAST_2e_EmptyExpression(g_UBNFAST_2e_EmptyExpression{span,node_id,
})])},
20=>{
let f0=self.field_nodes(child,&[24],None)?;
if !f0.iter().all(|value|(matches!(value,Ast::g_UBNFAST_2e_RootAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_MappingAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_EvalAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_WhitespaceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_InterleaveAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_BackrefAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_ScopeTreeAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_DeclaresAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_CatalogAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_LeftAssocAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_RightAssocAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_LongestChoiceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_PrecedenceAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_DocAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_RecoveryAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_SkipAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_CommonFieldAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_EnumAnnotation(_)) || matches!(value,Ast::g_UBNFAST_2e_SimpleAnnotation(_)))) {return Err("annotations: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let mut f1=self.field_text(child,&[25]);
if f1.len()!=1 {return Err("name requires one value".into());}let f1=f1.remove(0);
let f1:String=f1;
let mut f2=self.field_nodes(child,&[26],None)?;
if !f2.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if f2.len()!=1 {return Err("body requires one node".into());}let f2=Box::new(f2.remove(0));
let f2:Box<Ast>=f2;
let span=self.span(span);let node_id=self.node(RULES[20],"RuleDecl",span);Ok(vec![Ast::g_UBNFAST_2e_RuleDecl(g_UBNFAST_2e_RuleDecl{span,node_id,
g_annotations:f0,
g_name:f1,
g_body:f2,
})])},
21=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
22=>{
let span=self.span(span);let node_id=self.node(RULES[22],"RootAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_RootAnnotation(g_UBNFAST_2e_RootAnnotation{span,node_id,
})])},
23=>{
let mut f0=self.field_text(child,&[27]);
if f0.len()!=1 {return Err("className requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let f1=self.field_text(child,&[28, 29]);
let f1:Vec<String>=f1;
let span=self.span(span);let node_id=self.node(RULES[23],"MappingAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_MappingAnnotation(g_UBNFAST_2e_MappingAnnotation{span,node_id,
g_className:f0,
g_paramNames:f1,
})])},
24=>{
let f0=self.field_nodes(child,&[30, 31],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_KeyValuePair(_))) {return Err("entries: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[24],"EvalAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_EvalAnnotation(g_UBNFAST_2e_EvalAnnotation{span,node_id,
g_entries:f0,
})])},
25=>{
let mut f0=self.field_text(child,&[32]);
if f0.len()>1 {return Err("style requires at most one value".into());}let f0=f0.pop();
let f0:Option<String>=f0;
let span=self.span(span);let node_id=self.node(RULES[25],"WhitespaceAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_WhitespaceAnnotation(g_UBNFAST_2e_WhitespaceAnnotation{span,node_id,
g_style:f0,
})])},
26=>{
let mut f0=self.field_text(child,&[33]);
if f0.len()!=1 {return Err("profile requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[26],"InterleaveAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_InterleaveAnnotation(g_UBNFAST_2e_InterleaveAnnotation{span,node_id,
g_profile:f0,
})])},
27=>{
let mut f0=self.field_text(child,&[34]);
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[27],"BackrefAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_BackrefAnnotation(g_UBNFAST_2e_BackrefAnnotation{span,node_id,
g_name:f0,
})])},
28=>{
let mut f0=self.field_text(child,&[35]);
if f0.len()!=1 {return Err("mode requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[28],"ScopeTreeAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_ScopeTreeAnnotation(g_UBNFAST_2e_ScopeTreeAnnotation{span,node_id,
g_mode:f0,
})])},
29=>{
let mut f0=self.field_text(child,&[36]);
if f0.len()!=1 {return Err("symbol requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let mut f1=self.field_text(child,&[37]);
if f1.len()>1 {return Err("description requires at most one value".into());}let f1=f1.pop();
let f1:Option<String>=f1;
let span=self.span(span);let node_id=self.node(RULES[29],"DeclaresAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_DeclaresAnnotation(g_UBNFAST_2e_DeclaresAnnotation{span,node_id,
g_symbol:f0,
g_description:f1,
})])},
30=>{
let mut f0=self.field_text(child,&[38]);
if f0.len()!=1 {return Err("context requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[30],"CatalogAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_CatalogAnnotation(g_UBNFAST_2e_CatalogAnnotation{span,node_id,
g_context:f0,
})])},
31=>{
let span=self.span(span);let node_id=self.node(RULES[31],"LeftAssocAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_LeftAssocAnnotation(g_UBNFAST_2e_LeftAssocAnnotation{span,node_id,
})])},
32=>{
let span=self.span(span);let node_id=self.node(RULES[32],"RightAssocAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_RightAssocAnnotation(g_UBNFAST_2e_RightAssocAnnotation{span,node_id,
})])},
33=>{
let span=self.span(span);let node_id=self.node(RULES[33],"LongestChoiceAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_LongestChoiceAnnotation(g_UBNFAST_2e_LongestChoiceAnnotation{span,node_id,
})])},
34=>{
let span=self.span(span);let node_id=self.node(RULES[34],"PredictiveChoiceAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(g_UBNFAST_2e_PredictiveChoiceAnnotation{span,node_id,
})])},
35=>{
let mut f0=self.field_text(child,&[39]);
if f0.len()!=1 {return Err("level requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[35],"PrecedenceAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_PrecedenceAnnotation(g_UBNFAST_2e_PrecedenceAnnotation{span,node_id,
g_level:f0,
})])},
36=>{
let mut f0=self.field_text(child,&[40]);
if f0.len()!=1 {return Err("text requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[36],"DocAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_DocAnnotation(g_UBNFAST_2e_DocAnnotation{span,node_id,
g_text:f0,
})])},
37=>{
let mut f0=self.field_text(child,&[42]);
if f0.len()>1 {return Err("mode requires at most one value".into());}let f0=f0.pop();
let f0:Option<String>=f0;
let mut f1=self.field_text(child,&[41]);
if f1.len()>1 {return Err("syncTokens requires at most one value".into());}let f1=f1.pop();
let f1:Option<String>=f1;
let span=self.span(span);let node_id=self.node(RULES[37],"RecoveryAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_RecoveryAnnotation(g_UBNFAST_2e_RecoveryAnnotation{span,node_id,
g_mode:f0,
g_syncTokens:f1,
})])},
38=>{
let span=self.span(span);let node_id=self.node(RULES[38],"SkipAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_SkipAnnotation(g_UBNFAST_2e_SkipAnnotation{span,node_id,
})])},
39=>{
let f0=self.field_text(child,&[43, 44]);
let f0:Vec<String>=f0;
let span=self.span(span);let node_id=self.node(RULES[39],"CommonFieldAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_CommonFieldAnnotation(g_UBNFAST_2e_CommonFieldAnnotation{span,node_id,
g_fieldNames:f0,
})])},
40=>{
let span=self.span(span);let node_id=self.node(RULES[40],"EnumAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_EnumAnnotation(g_UBNFAST_2e_EnumAnnotation{span,node_id,
})])},
41=>{
let mut f0=self.field_text(child,&[45]);
if f0.len()!=1 {return Err("name requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[41],"SimpleAnnotation",span);Ok(vec![Ast::g_UBNFAST_2e_SimpleAnnotation(g_UBNFAST_2e_SimpleAnnotation{span,node_id,
g_name:f0,
})])},
42=>{
let f0=self.field_nodes(child,&[46, 47],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_SequenceBody(_))) {return Err("alternatives: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[42],"ChoiceBody",span);Ok(vec![Ast::g_UBNFAST_2e_ChoiceBody(g_UBNFAST_2e_ChoiceBody{span,node_id,
g_alternatives:f0,
})])},
43=>{
let f0=self.field_nodes(child,&[48],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_AnnotatedElement(_))) {return Err("elements: mapped value type mismatch".into());}
let f0:Vec<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[43],"SequenceBody",span);Ok(vec![Ast::g_UBNFAST_2e_SequenceBody(g_UBNFAST_2e_SequenceBody{span,node_id,
g_elements:f0,
})])},
44=>{
let mut f0=self.field_nodes(child,&[50],None)?;
if !f0.iter().all(|value|(matches!(value,Ast::g_UBNFAST_2e_GroupElement(_)) || matches!(value,Ast::g_UBNFAST_2e_OptionalElement(_)) || matches!(value,Ast::g_UBNFAST_2e_RepeatElement(_)) || matches!(value,Ast::g_UBNFAST_2e_ErrorElement(_)) || matches!(value,Ast::g_UBNFAST_2e_TerminalElement(_)) || matches!(value,Ast::Text(_)))) {return Err("element: mapped value type mismatch".into());}
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
})])},
45=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
46=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
47=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
48=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
49=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
50=>{
let mut f0=self.field_nodes(child,&[52],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if f0.len()!=1 {return Err("body requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[50],"GroupElement",span);Ok(vec![Ast::g_UBNFAST_2e_GroupElement(g_UBNFAST_2e_GroupElement{span,node_id,
g_body:f0,
})])},
51=>{
let mut f0=self.field_nodes(child,&[53],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if f0.len()!=1 {return Err("body requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[51],"OptionalElement",span);Ok(vec![Ast::g_UBNFAST_2e_OptionalElement(g_UBNFAST_2e_OptionalElement{span,node_id,
g_body:f0,
})])},
52=>{
let mut f0=self.field_nodes(child,&[54],None)?;
if !f0.iter().all(|value|matches!(value,Ast::g_UBNFAST_2e_ChoiceBody(_))) {return Err("body: mapped value type mismatch".into());}
if f0.len()!=1 {return Err("body requires one node".into());}let f0=Box::new(f0.remove(0));
let f0:Box<Ast>=f0;
let span=self.span(span);let node_id=self.node(RULES[52],"RepeatElement",span);Ok(vec![Ast::g_UBNFAST_2e_RepeatElement(g_UBNFAST_2e_RepeatElement{span,node_id,
g_body:f0,
})])},
53=>{
let mut f0=self.field_text(child,&[55]);
if f0.len()!=1 {return Err("message requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[53],"ErrorElement",span);Ok(vec![Ast::g_UBNFAST_2e_ErrorElement(g_UBNFAST_2e_ErrorElement{span,node_id,
g_message:f0,
})])},
54=>{
let mut f0=self.field_text(child,&[56]);
if f0.len()!=1 {return Err("value requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[54],"TerminalElement",span);Ok(vec![Ast::g_UBNFAST_2e_TerminalElement(g_UBNFAST_2e_TerminalElement{span,node_id,
g_value:f0,
})])},
55=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
56=>{
let mut f0=self.field_text(child,&[59]);
if f0.len()!=1 {return Err("captureName requires one value".into());}let f0=f0.remove(0);
let f0:String=f0;
let span=self.span(span);let node_id=self.node(RULES[56],"TypeofElement",span);Ok(vec![Ast::g_UBNFAST_2e_TypeofElement(g_UBNFAST_2e_TypeofElement{span,node_id,
g_captureName:f0,
})])},
57=>{
let f0=self.field_text(child,&[60, 61]);
let f0:Vec<String>=f0;
let span=self.span(span);let node_id=self.node(RULES[57],"DottedIdentifier",span);Ok(vec![Ast::g_UBNFAST_2e_DottedIdentifier(g_UBNFAST_2e_DottedIdentifier{span,node_id,
g_parts:f0,
})])},
58=>{let mut values=self.build_values(child)?;if values.is_empty() {values.push(Ast::Text(self.text(span)));}Ok(values)},
_=>Err(format!("unknown rule {rule}"))}}
fn occurrences(&self, root: EventId) -> (Vec<Capture>, Vec<LexicalOccurrence>) {
    let mut captures = Vec::new();
    let mut lexical = Vec::new();
    let mut stack = vec![(root, None, false, None)];
    let mut lexical_order = 0;
    while let Some((id, parent, done, current)) = stack.pop() {
        match self.arena[id.0] {
            Event::Join(a, b) => {
                stack.push((b, parent, false, None));
                stack.push((a, parent, false, None));
            }
            Event::Capture { site, span, child } => {
                if done {
                    if self.options.build_ast {
                        let order = captures.len();
                        captures.push(Capture {
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
            Event::Rule { rule, span, child } => {
                if done {
                    if let Some(index) = current {
                        let node: &mut LexicalOccurrence = &mut lexical[index];
                        node.completion_order = lexical_order;
                        lexical_order += 1;
                    }
                } else {
                    let index = if self.options.lexical {
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
    (captures, lexical)
}
pub(crate) fn finish(mut self, step: Step) -> ParseResult {
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
    if let Some((pos, expected, path)) = self.diag.summary(step.diag) {
        let rule_path: Vec<_> = path.into_iter().map(|r| RULES[r]).collect();
        diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Syntax,
            offset_cp: self.cp(pos),
            farthest_cp: self.cp(pos),
            expected,
            deepest_rule: rule_path.last().copied(),
            rule_path,
            message: None,
        });
    }
    if trailing {
        let farthest_cp = diagnostics.last().map_or(consumed_cp, |d| d.farthest_cp);
        diagnostics.push(Diagnostic {
            kind: DiagnosticKind::TrailingInput,
            offset_cp: consumed_cp,
            farthest_cp,
            expected: vec!["end of input"],
            deepest_rule: None,
            rule_path: vec![],
            message: None,
        });
    }
    let (captures, lexical) = if ok {
        self.occurrences(step.events)
    } else {
        (vec![], vec![])
    };
    let ast = if ok && self.options.build_ast {
        match self.build_values(step.events).and_then(|mut nodes| {
            if nodes.len() == 1 {
                Ok(nodes.remove(0))
            } else {
                Err("entry must produce one AST value".into())
            }
        }) {
            Ok(ast) => Some(ast),
            Err(message) => {
                self.node_spans.clear();
                diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Mapping,
                    offset_cp: consumed_cp,
                    farthest_cp: consumed_cp,
                    expected: vec![],
                    deepest_rule: None,
                    rule_path: vec![],
                    message: Some(message),
                });
                None
            }
        }
    } else {
        None
    };
    self.statistics.memo_entries = self.memo.len();
    self.statistics.recipes = self.arena.len() - 1;
    self.statistics.ast_nodes = self.node_spans.len();
    ParseResult {
        ok,
        consumed_cp,
        matched_cp,
        ast,
        captures,
        lexical,
        node_spans: self.node_spans,
        diagnostics,
        declarations: if ok {
            self.scope.all_declarations().to_vec()
        } else {
            vec![]
        },
        references: if ok {
            self.scope.references().to_vec()
        } else {
            vec![]
        },
        scope_diagnostics: if ok {
            self.scope.diagnostics().to_vec()
        } else {
            vec![]
        },
        statistics: self.statistics,
    }
}
}
