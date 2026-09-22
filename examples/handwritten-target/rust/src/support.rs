//! 生成時に共有できる静的処理。rule ディスパッチや汎用 parser VM は持たない。
use crate::api::*;
use crate::rt::{
    cursor::Cursor,
    diag::{Diagnostics, Snapshot},
    input::Input,
    memo::{MemoEntry, MemoKey, MemoTable},
    scope::ScopeStore,
};

pub(crate) type Step = Option<(usize, usize)>;
pub(crate) type RecipeId = usize;
pub(crate) type ByteSpan = [usize; 2];
pub(crate) const NUMBER: u32 = 100;
pub(crate) const NUMBER_GROUP: u32 = 101;
pub(crate) const EXPECTED: &[&str] = &[
    "NUMBER", "'+'", "'-'", "'*'", "'/'", "'('", "')'", "'^'", "'😀'", "EOF", "digit",
];

#[derive(Debug)]
pub(crate) enum RecipeKind {
    Number {
        value: ByteSpan,
        token: ByteSpan,
    },
    Text {
        value: ByteSpan,
        token: ByteSpan,
    },
    Binary {
        left: RecipeId,
        tail: Vec<(ByteSpan, RecipeId)>,
    },
    Power {
        left: RecipeId,
        tail: Option<(ByteSpan, RecipeId)>,
    },
    Tail {
        operator: ByteSpan,
        right: RecipeId,
    },
}

#[derive(Debug)]
struct Recipe {
    rule: u32,
    span: ByteSpan,
    kind: RecipeKind,
}

#[derive(Clone, Copy)]
enum Event {
    Child(usize),
    Capture {
        site: u32,
        name: &'static str,
        span: ByteSpan,
    },
}

struct Payload {
    recipe: Option<RecipeId>,
    events: Vec<Event>,
    diagnostics: Snapshot,
}

/// 認識時の recipe / capture は index だけで結び、memo 生存中は arena を truncate しない。
pub(crate) struct Session<'a> {
    pub input: Input<'a>,
    pub options: ParseOptions,
    pub mode: u8,
    pub scope: ScopeStore,
    pub diag: Diagnostics,
    pub last: Option<RecipeId>,
    memo: MemoTable,
    payloads: Vec<Payload>,
    frames: Vec<Vec<Event>>,
    recipes: Vec<Recipe>,
    stats: Statistics,
    rules: &'static [(u32, &'static str)],
}

impl<'a> Session<'a> {
    pub fn new(
        text: &'a str,
        options: ParseOptions,
        variant: u8,
        rules: &'static [(u32, &'static str)],
    ) -> Self {
        // rt::MemoKey の byte offset は u32。切り捨てたキーを使わない。
        assert!(
            u32::try_from(text.len()).is_ok(),
            "入力は u32 byte offset の範囲内"
        );
        Self {
            input: Input::new(text),
            options,
            mode: (variant << 1) | u8::from(options.build_ast),
            scope: ScopeStore::new(),
            diag: Diagnostics::new(),
            last: None,
            memo: MemoTable::new(),
            payloads: Vec::new(),
            frames: Vec::new(),
            recipes: Vec::new(),
            stats: Statistics::default(),
            rules,
        }
    }

    pub fn key(&self, rule: u32, consumed: usize, matched: usize) -> MemoKey {
        (
            rule,
            u32::try_from(consumed).expect("consumed offset"),
            u32::try_from(matched).expect("matched offset"),
            self.mode,
            self.scope.state_version(),
        )
    }

    /// 外側 None は miss、Some(None) は memo 化済み失敗。
    pub fn replay(&mut self, key: MemoKey) -> Option<Step> {
        let entry = *self.memo.get(&key)?;
        self.stats.memo_hits += 1;
        let payload = &self.payloads[entry.payload as usize];
        self.diag.merge(&payload.diagnostics);
        self.last = payload.recipe;
        if entry.ok {
            self.child(entry.payload as usize);
            Some(Some((
                entry.consumed_end as usize,
                entry.matched_end as usize,
            )))
        } else {
            Some(None)
        }
    }

    pub fn enter(&mut self, rule: u32) {
        self.stats.rule_evaluations += 1;
        self.diag.enter_rule(rule);
        self.frames.push(Vec::new());
        self.last = None;
    }

    pub fn leave(&mut self, key: MemoKey, end: Step) -> Step {
        let mut events = self.frames.pop().expect("rule frame");
        let diagnostics = self.diag.leave_rule().expect("diagnostic frame");
        if end.is_none() {
            self.last = None;
            events.clear();
        }
        let payload = self.payloads.len();
        self.payloads.push(Payload {
            recipe: self.last,
            events,
            diagnostics,
        });
        let (c, m) = end.unwrap_or((key.1 as usize, key.2 as usize));
        self.memo.insert(
            key,
            MemoEntry {
                ok: end.is_some(),
                consumed_end: c as u32,
                matched_end: m as u32,
                payload: u32::try_from(payload).expect("payload index"),
            },
        );
        if end.is_some() {
            self.child(payload);
        }
        end
    }

    fn child(&mut self, payload: usize) {
        if self.options.build_ast {
            if let Some(frame) = self.frames.last_mut() {
                frame.push(Event::Child(payload));
            }
        }
    }

    pub fn checkpoint(&self) -> usize {
        self.frames.last().expect("rule frame").len()
    }
    pub fn rollback(&mut self, checkpoint: usize) {
        self.frames
            .last_mut()
            .expect("rule frame")
            .truncate(checkpoint);
        self.last = None;
    }

    pub fn capture(&mut self, site: u32, name: &'static str, start: usize, end: usize) {
        if self.options.build_ast {
            self.frames
                .last_mut()
                .expect("rule frame")
                .push(Event::Capture {
                    site,
                    name,
                    span: [start, end],
                });
        }
    }

    pub fn recipe(&mut self, rule: u32, span: ByteSpan, kind: RecipeKind) {
        self.last = Some(self.recipes.len());
        self.recipes.push(Recipe { rule, span, kind });
    }

    pub fn tail(&self) -> (ByteSpan, RecipeId) {
        match self.recipes[self.last.expect("tail recipe")].kind {
            RecipeKind::Tail { operator, right } => (operator, right),
            _ => unreachable!("反復 helper の構築レシピ"),
        }
    }

    pub fn literal(
        &mut self,
        consumed: usize,
        matched: usize,
        literal: &str,
        expected: u32,
    ) -> Step {
        if self.input.starts_with(consumed, literal) {
            Some(advance(consumed, matched, literal.len()))
        } else {
            self.fail(consumed, expected);
            None
        }
    }

    pub fn fail(&mut self, byte: usize, expected: u32) {
        self.diag.fail_at(byte as u32, expected);
    }

    /// 呼出し位置は生成された chain 境界だけ。delimiter の失敗は診断へ寄与しない。
    pub fn trivia(&self, consumed: usize, matched: usize) -> (usize, usize) {
        let bytes = self.input.bytes();
        let mut pos = consumed;
        loop {
            let start = pos;
            while matches!(
                bytes.get(pos),
                Some(b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r')
            ) {
                pos += 1;
            }
            if self.input.starts_with(pos, "//") {
                pos += 2;
                while let Some(&b) = bytes.get(pos) {
                    if matches!(b, b'\r' | b'\n') {
                        break;
                    }
                    pos += 1;
                }
            } else if self.input.starts_with(pos, "/*") {
                let mut close = pos + 2;
                while close < bytes.len() && !self.input.starts_with(close, "*/") {
                    close += 1;
                }
                if close < bytes.len() {
                    pos = close + 2;
                }
                // 閉じない comment は delimiter 一回分の失敗。開始位置へ戻す。
            }
            if pos == start {
                break;
            }
        }
        advance(consumed, matched, pos - consumed)
    }

    /// [+-]?(D+('.'D*)?|'.'D+)([eE][+-]?D+)?。指数失敗は prefix へ戻る。
    pub fn parse_number(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.key(NUMBER, consumed, matched);
        if let Some(hit) = self.replay(key) {
            return hit;
        }
        self.enter(NUMBER);
        let result = self.number_body(consumed, matched);
        self.leave(key, result)
    }

    fn number_body(&mut self, consumed: usize, matched: usize) -> Step {
        let bytes = self.input.bytes();
        let mut pos = consumed;
        if matches!(bytes.get(pos), Some(b'+' | b'-')) {
            pos += 1;
        }
        let digits = pos;
        while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
            pos += 1;
        }
        let integer = pos > digits;
        if bytes.get(pos) == Some(&b'.') {
            pos += 1;
            let fraction = pos;
            while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                pos += 1;
            }
            if !integer && pos == fraction {
                self.fail(pos, 0);
                return None;
            }
        } else if !integer {
            self.fail(pos, 0);
            return None;
        }
        if matches!(bytes.get(pos), Some(b'e' | b'E')) {
            let exponent = pos;
            pos += 1;
            if matches!(bytes.get(pos), Some(b'+' | b'-')) {
                pos += 1;
            }
            let digits = pos;
            while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                pos += 1;
            }
            if pos == digits {
                self.fail(pos, 10);
                pos = exponent;
            }
        }
        Some(advance(consumed, matched, pos - consumed))
    }

    /// (NUMBER) 自体が一要素 chain。group の終端 trivia まで @value に含める。
    pub fn parse_number_group(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.key(NUMBER_GROUP, consumed, matched);
        if let Some(hit) = self.replay(key) {
            return hit;
        }
        self.enter(NUMBER_GROUP);
        let (c, m) = self.trivia(consumed, matched);
        let result = self.parse_number(c, m).map(|(c, m)| self.trivia(c, m));
        self.leave(key, result)
    }

    pub fn cp(&self, byte: usize) -> usize {
        self.input.byte_to_cp(byte).expect("文字境界")
    }
    fn span(&self, span: ByteSpan) -> Span {
        [self.cp(span[0]), self.cp(span[1])]
    }
    fn text(&self, span: ByteSpan) -> &str {
        std::str::from_utf8(&self.input.bytes()[span[0]..span[1]]).expect("UTF-8 文字境界")
    }
    fn rule_name(&self, id: u32) -> &'static str {
        match id {
            NUMBER => "NUMBER",
            NUMBER_GROUP => "(NUMBER)",
            _ => self
                .rules
                .iter()
                .find(|(rule, _)| *rule == id)
                .map_or("root", |(_, name)| *name),
        }
    }

    pub fn finish(mut self, end: Step, mapper: NumberMapper) -> ParseResult {
        let (c, m) = end.unwrap_or((0, 0));
        let ok = end.is_some() && (!self.options.require_eof || c == self.input.bytes().len());
        if end.is_some() && !ok {
            self.fail(c, 9);
        }
        let mut result = ParseResult {
            ok,
            consumed_cp: self.cp(c),
            matched_cp: self.cp(m),
            ast: None,
            node_spans: Vec::new(),
            captures: Vec::new(),
            diagnostics: Vec::new(),
            statistics: self.stats,
        };
        let snapshot = self.diag.snapshot();
        if !snapshot.expected.is_empty() {
            result.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::Syntax,
                offset_cp: self.cp(snapshot.farthest as usize),
                expected: snapshot
                    .expected
                    .iter()
                    .map(|&e| EXPECTED[e as usize])
                    .collect(),
                deepest_rule: self.rule_name(snapshot.deepest_rule),
                rule_path: snapshot
                    .rule_path
                    .iter()
                    .map(|&id| self.rule_name(id))
                    .collect(),
                message: None,
            });
        }
        if ok && self.options.build_ast {
            // 最後の payload は root。失敗枝は DAG から到達不能なので出現を再生しない。
            self.captures(self.payloads.len() - 1, &mut result.captures);
            result.ast = self.build(self.last.expect("root recipe"), mapper, &mut result);
            if result.ast.is_none() {
                result.node_spans.clear();
            }
        }
        result.statistics.memo_entries = self.memo.len();
        result.statistics.recipes = self.recipes.len();
        result
    }

    fn captures(&self, root: usize, out: &mut Vec<Capture>) {
        let mut stack = vec![self.payloads[root].events.iter()];
        while let Some(events) = stack.last_mut() {
            match events.next() {
                Some(Event::Child(id)) => stack.push(self.payloads[*id].events.iter()),
                Some(Event::Capture { site, name, span }) => out.push(Capture {
                    site_id: *site,
                    name,
                    span: self.span(*span),
                    completion_order: out.len(),
                }),
                None => {
                    stack.pop();
                }
            }
        }
    }

    fn build(&self, id: RecipeId, mapper: NumberMapper, result: &mut ParseResult) -> Option<Ast> {
        let recipe = &self.recipes[id];
        let span = self.span(recipe.span);
        let kind = match &recipe.kind {
            RecipeKind::Tail { .. } => unreachable!("tail は親のリストへ展開済み"),
            RecipeKind::Number { value, token } | RecipeKind::Text { value, token } => {
                // NUMBER の token 範囲は採用後に初めて文字列/数値化する。
                result.statistics.mapped_numbers += 1;
                let Some(number) = mapper(self.text(*token)) else {
                    result.diagnostics.push(Diagnostic {
                        kind: DiagnosticKind::Mapping,
                        offset_cp: self.cp(token[0]),
                        expected: Vec::new(),
                        deepest_rule: self.rule_name(recipe.rule),
                        rule_path: vec![self.rule_name(recipe.rule)],
                        message: Some(format!("数値 mapping 失敗: {:?}", self.text(*token))),
                    });
                    return None;
                };
                let value = self
                    .text(*value)
                    .trim_matches(|ch| ch <= '\u{20}')
                    .to_owned();
                if matches!(recipe.kind, RecipeKind::Number { .. }) {
                    AstKind::Number { value, number }
                } else {
                    AstKind::Text { value, number }
                }
            }
            RecipeKind::Binary { left, tail } => {
                let left = Box::new(self.build(*left, mapper, result)?);
                let mut op = Vec::with_capacity(tail.len());
                let mut right = Vec::with_capacity(tail.len());
                for (operator, rhs) in tail {
                    op.push(self.text(*operator).to_owned());
                    right.push(self.build(*rhs, mapper, result)?);
                }
                AstKind::Binary { left, op, right }
            }
            RecipeKind::Power { left, tail } => {
                let left = Box::new(self.build(*left, mapper, result)?);
                let mut op = Vec::with_capacity(usize::from(tail.is_some()));
                let mut right = Vec::with_capacity(op.capacity());
                if let Some((operator, rhs)) = tail {
                    op.push(self.text(*operator).to_owned());
                    right.push(self.build(*rhs, mapper, result)?);
                }
                AstKind::Power { left, op, right }
            }
        };
        let mut ast = Ast {
            node_id: None,
            span,
            kind,
        };
        if !matches!(ast.kind, AstKind::Text { .. }) {
            let node_id = result.node_spans.len();
            result.node_spans.push(NodeSpan {
                node_id,
                rule_id: recipe.rule,
                node_type: ast.type_name(),
                span,
            });
            result.statistics.ast_nodes += 1;
            ast.node_id = Some(node_id);
        }
        Some(ast)
    }

    /// group が消費した trivia を除く NUMBER 終端。scanner の memo を読み、再実行しない。
    pub fn number_end(&self, consumed: usize, matched: usize) -> usize {
        self.memo
            .get(&self.key(NUMBER, consumed, matched))
            .expect("採用済み NUMBER")
            .consumed_end as usize
    }
}

pub(crate) fn advance(consumed: usize, matched: usize, bytes: usize) -> (usize, usize) {
    let mut cursor = Cursor { consumed, matched };
    cursor.advance(bytes);
    (cursor.consumed, cursor.matched)
}

/// Occurs: 成功周回を先に数え、ゼロ進捗なら停止。その後 min/max を検査する。
pub(crate) fn repeat_more(count: &mut usize, before: usize, after: usize, max: usize) -> bool {
    *count += 1;
    after != before && *count < max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memo_key_separates_both_cursors_mode_and_scope_version() {
        let mut s = Session::new("1abc", ParseOptions::default(), 0, &[]);
        assert_eq!(s.parse_number(0, 0), Some((1, 1)));
        assert_eq!(s.parse_number(0, 0), Some((1, 1)));
        assert_eq!(s.parse_number(0, 1), Some((1, 2)));
        assert_eq!(s.parse_number(1, 1), None);
        s.mode ^= 1;
        assert_eq!(s.parse_number(0, 0), Some((1, 1)));
        s.scope.enter();
        assert_eq!(s.parse_number(0, 0), Some((1, 1)));
        assert_eq!(s.memo.len(), 5);
        assert_eq!(s.stats.memo_hits, 1);
    }

    #[test]
    fn memo_replays_local_failure_under_the_current_call_stack() {
        let mut s = Session::new("1e+", ParseOptions::default(), 0, &[]);
        let key = s.key(1, 0, 0);
        s.enter(1);
        assert_eq!(s.parse_number(0, 0), Some((1, 1)));
        s.leave(key, Some((1, 1)));
        s.enter(2);
        assert_eq!(s.parse_number(0, 0), Some((1, 1)));
        let local = s.diag.snapshot();
        assert_eq!(local.farthest, 3);
        assert_eq!(local.rule_path, [2, NUMBER]);
        assert_eq!(local.expected, [10]);
        assert_eq!(s.stats.memo_hits, 1);
    }

    #[test]
    fn failure_memo_replays_without_rescanning() {
        let mut s = Session::new("+", ParseOptions::default(), 0, &[]);
        assert_eq!(s.parse_number(0, 0), None);
        s.diag = Diagnostics::new();
        assert_eq!(s.parse_number(0, 0), None);
        assert_eq!(s.diag.farthest, 1);
        assert_eq!(s.diag.expected, [0]);
        assert_eq!((s.stats.rule_evaluations, s.stats.memo_hits), (1, 1));
    }

    #[test]
    fn literal_is_a_prefix_without_a_word_boundary() {
        let mut s = Session::new("letter", ParseOptions::default(), 0, &[]);
        assert_eq!(s.literal(0, 0, "let", 0), Some((3, 3)));
        assert_eq!(s.literal(0, 1, "let", 0), Some((3, 4)));
        assert_eq!(s.literal(6, 6, "", 0), Some((6, 6)));
    }

    #[test]
    fn zero_width_success_counts_once_before_min_is_checked() {
        let mut count = 0;
        assert!(!repeat_more(&mut count, 3, 3, 2));
        assert_eq!(count, 1);
        assert!((1..=2).contains(&count)); // EMPTY{1,2}
        assert!(!(2..=2).contains(&count)); // EMPTY{2,2}
        let mut count = 0;
        assert!(repeat_more(&mut count, 0, 1, 2));
        assert!(!repeat_more(&mut count, 1, 2, 2));
        assert_eq!(count, 2);
    }
}
