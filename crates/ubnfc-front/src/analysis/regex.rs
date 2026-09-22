//! Java Pattern の保守的な正則部分集合。Thompson NFA → 優先順位付き部分集合 DFA。
//! lookingAt と同じ leftmost-first（左枝優先・貪欲量指定子）の接頭辞を採用する。
use super::charset::CharSet;
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};

const STATE_LIMIT: usize = 10_000;
const WORK_LIMIT: usize = 2_000_000;
const DEPTH_LIMIT: usize = 128;

#[derive(Debug)]
enum Re {
    Set(CharSet),
    Start,
    Seq(Vec<Re>),
    Alt(Vec<Re>),
    Repeat(Box<Re>, usize, Option<usize>),
}

impl Re {
    // ^ は開始位置で空文字を受理し得るので、ここでは保守的に nullable とする。
    fn nullable(&self) -> bool {
        match self {
            Self::Set(_) => false,
            Self::Start => true,
            Self::Seq(items) => items.iter().all(Self::nullable),
            Self::Alt(items) => items.iter().any(Self::nullable),
            Self::Repeat(item, min, _) => *min == 0 || item.nullable(),
        }
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}
impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    fn take(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn alt(&mut self, depth: usize) -> Option<Re> {
        if depth > DEPTH_LIMIT {
            return None;
        }
        let mut alternatives = vec![self.seq(depth)?];
        while self.eat('|') {
            alternatives.push(self.seq(depth)?);
        }
        Some(Re::Alt(alternatives))
    }
    fn seq(&mut self, depth: usize) -> Option<Re> {
        let mut items = vec![];
        while !matches!(self.peek(), None | Some(')' | '|')) {
            let atom = match self.take()? {
                '(' => {
                    // (?:...) 以外の (?...) は flags/先読み/後読み等なので委譲。
                    if self.eat('?') && !self.eat(':') {
                        return None;
                    }
                    let inner = self.alt(depth + 1)?;
                    if !self.eat(')') {
                        return None;
                    }
                    inner
                }
                '[' => Re::Set(self.class()?),
                '.' => Re::Set(CharSet::chars("\n\r\u{85}\u{2028}\u{2029}").complement()),
                '^' => Re::Start,
                '\\' => Re::Set(self.escape()?),
                '$' | '*' | '+' | '?' | '{' | '}' | ']' => return None,
                c => Re::Set(CharSet::chars(&c.to_string())),
            };
            let bounds = match self.peek() {
                Some('*') => {
                    self.take();
                    Some((0, None))
                }
                Some('+') => {
                    self.take();
                    Some((1, None))
                }
                Some('?') => {
                    self.take();
                    Some((0, Some(1)))
                }
                Some('{') => {
                    self.take();
                    let min = self.number()?;
                    let max = if self.eat(',') {
                        if self.peek() == Some('}') {
                            None
                        } else {
                            Some(self.number()?)
                        }
                    } else {
                        Some(min)
                    };
                    if !self.eat('}') || max.is_some_and(|m| m < min) {
                        return None;
                    }
                    Some((min, max))
                }
                _ => None,
            };
            items.push(if let Some((min, max)) = bounds {
                // lazy/possessive は正則でも Java の採用結果を変えるので委譲。
                if matches!(self.peek(), Some('?' | '+')) || matches!(atom, Re::Start) {
                    return None;
                }
                // Java は反復本体のゼロ幅成功で進捗停止し、通常の Thompson NFA と
                // 探索順が異なり得る（(|a)* / a は Java ではゼロ幅）。有界反復にも
                // 差分があるので、本体が nullable で複数回試す場合は保守的に委譲。
                if max.is_none_or(|max| max > 1) && atom.nullable() {
                    return None;
                }
                Re::Repeat(Box::new(atom), min, max)
            } else {
                atom
            });
        }
        Some(Re::Seq(items))
    }
    fn number(&mut self) -> Option<usize> {
        let start = self.pos;
        let mut n = 0usize;
        while let Some(c @ '0'..='9') = self.peek() {
            self.take();
            n = n.checked_mul(10)?.checked_add(c as usize - '0' as usize)?;
            if n > STATE_LIMIT {
                return None;
            }
        }
        (self.pos > start).then_some(n)
    }
    fn escape(&mut self) -> Option<CharSet> {
        let c = self.take()?;
        let set = match c {
            'd' | 'D' => CharSet::range(48, 57),
            'w' | 'W' => {
                let mut set = CharSet::chars("_");
                for (lo, hi) in [('a', 'z'), ('A', 'Z'), ('0', '9')] {
                    set.union(&CharSet::range(lo as u64, hi as u64));
                }
                set
            }
            's' | 'S' => CharSet::chars(" \t\n\u{b}\u{c}\r"),
            't' => CharSet::chars("\t"),
            'n' => CharSet::chars("\n"),
            'r' => CharSet::chars("\r"),
            'f' => CharSet::chars("\u{c}"),
            'a' => CharSet::chars("\u{7}"),
            'e' => CharSet::chars("\u{1b}"),
            // 数字/backref、境界、Unicode property、未対応 escape は推測しない。
            c if c.is_ascii_alphanumeric() => return None,
            c => CharSet::chars(&c.to_string()),
        };
        Some(if matches!(c, 'D' | 'W' | 'S') {
            set.complement()
        } else {
            set
        })
    }
    fn class_atom(&mut self) -> Option<CharSet> {
        match self.take()? {
            '\\' => self.escape(),
            '[' | ']' => None, // Java の nested class/交差は委譲。
            '&' if self.peek() == Some('&') => None,
            c => Some(CharSet::chars(&c.to_string())),
        }
    }
    fn class(&mut self) -> Option<CharSet> {
        let negate = self.eat('^');
        let mut set = CharSet::default();
        let mut count = 0;
        // Java は先頭 ] を文字として扱う。
        if self.eat(']') {
            set.union(&CharSet::chars("]"));
            count += 1;
        }
        while self.peek() != Some(']') {
            let mut item = self.class_atom()?;
            if self.peek() == Some('-') && self.chars.get(self.pos + 1) != Some(&']') {
                self.take();
                let end = self.class_atom()?;
                let a = item.ranges();
                let b = end.ranges();
                if a.len() != 1
                    || b.len() != 1
                    || a[0].start != a[0].end
                    || b[0].start != b[0].end
                    || a[0].start > b[0].start
                {
                    return None;
                }
                item = CharSet::range(a[0].start, b[0].start);
            }
            set.union(&item);
            count += 1;
            if count > 1024 {
                return None;
            }
        }
        self.take();
        if count == 0 {
            return None;
        }
        Some(if negate { set.complement() } else { set })
    }
}

#[derive(Default)]
struct State {
    epsilon: Vec<usize>,
    at_start: Vec<usize>,
    edges: Vec<(CharRange, usize)>,
}
struct Nfa {
    states: Vec<State>,
    limit: usize,
    work: usize,
}
impl Nfa {
    fn state(&mut self) -> Option<usize> {
        if self.states.len() >= self.limit {
            return None;
        }
        self.states.push(State::default());
        Some(self.states.len() - 1)
    }
    fn build(&mut self, re: &Re) -> Option<(usize, usize)> {
        let start = self.state()?;
        let end = self.state()?;
        match re {
            Re::Set(set) => {
                let ranges = set.ranges();
                self.work = self.work.checked_sub(ranges.len())?;
                self.states[start]
                    .edges
                    .extend(ranges.into_iter().map(|r| (r, end)));
            }
            Re::Start => self.states[start].at_start.push(end),
            Re::Seq(items) => {
                let mut prev = start;
                for item in items {
                    let (a, b) = self.build(item)?;
                    self.states[prev].epsilon.push(a);
                    prev = b;
                }
                self.states[prev].epsilon.push(end);
            }
            Re::Alt(items) => {
                for item in items {
                    let (a, b) = self.build(item)?;
                    self.states[start].epsilon.push(a);
                    self.states[b].epsilon.push(end);
                }
            }
            Re::Repeat(item, min, max) => {
                let mut prev = start;
                for _ in 0..*min {
                    let (a, b) = self.build(item)?;
                    self.states[prev].epsilon.push(a);
                    prev = b;
                }
                if let Some(max) = max {
                    for _ in *min..*max {
                        let (a, b) = self.build(item)?;
                        // 貪欲量指定子では「続ける」を「終了する」より先に試す。
                        self.states[prev].epsilon.push(a);
                        self.states[prev].epsilon.push(end);
                        prev = b;
                    }
                } else {
                    let (a, b) = self.build(item)?;
                    self.states[prev].epsilon.push(a);
                    self.states[b].epsilon.push(prev);
                }
                self.states[prev].epsilon.push(end);
            }
        }
        Some((start, end))
    }
    fn closure(
        &self,
        seeds: impl IntoIterator<Item = usize>,
        start: bool,
        end: usize,
        work: &mut usize,
    ) -> Option<Vec<usize>> {
        let mut seen = BTreeSet::new();
        let mut ordered = vec![];
        let mut pending: Vec<_> = seeds.into_iter().collect();
        pending.reverse();
        while let Some(i) = pending.pop() {
            *work = work.checked_sub(1)?;
            if seen.insert(i) {
                ordered.push(i);
                if i == end {
                    break;
                }
                // stack は逆順に積み、左枝・貪欲の継続枝から深さ優先で列挙する。
                // at_start と epsilon は同じ NFA 状態には共存しない。
                pending.extend(self.states[i].epsilon.iter().rev());
                if start {
                    pending.extend(self.states[i].at_start.iter().rev());
                }
            }
        }
        Some(ordered)
    }
    // 各リストは、同じ入力位置でのバックトラック探索の優先順を表す。
    // 同一 NFA 状態への再到達は以降の照合が同じなので、最優先の経路だけ残せる
    // （後方参照等の経路依存の構文・nullable 本体の反復は compile しない）。
    // 受理より低優先の経路は今の候補に勝てず、closure で切り捨てる。
    // 高優先の未完了経路は残して進め、
    // 後で受理したときだけ候補を置き換える。この不変条件により、入力終端または
    // 遷移なしで停止したときの最後の候補が、バックトラック探索の最初の成功になる。
    // 同じ集合でも順序が異なれば優先順位が違うため、DFA 状態のキーは Vec のまま。
    fn determinize(&self, start: usize, end: usize, limit: usize) -> Option<Dfa> {
        let mut work = WORK_LIMIT;
        let initial = self.closure([start], true, end, &mut work)?;
        let mut ids = BTreeMap::from([(initial.clone(), 0)]);
        let mut subsets = vec![initial];
        let mut states = vec![];
        while states.len() < subsets.len() {
            let subset = &subsets[states.len()];
            let accepting = subset.contains(&end);
            let mut edges = vec![];
            let mut boundaries = BTreeSet::new();
            for &i in subset {
                work = work.checked_sub(1)?;
                for (range, target) in &self.states[i].edges {
                    boundaries.insert(range.start);
                    boundaries.insert(range.end + 1);
                    edges.push((range, *target));
                }
            }
            let boundaries: Vec<_> = boundaries.into_iter().collect();
            let mut transitions: Vec<DfaTransition> = vec![];
            for pair in boundaries.windows(2) {
                work = work.checked_sub(edges.len())?;
                let seeds = edges
                    .iter()
                    .filter_map(|(r, to)| (r.start <= pair[0] && pair[0] <= r.end).then_some(*to));
                let next = self.closure(seeds, false, end, &mut work)?;
                if next.is_empty() {
                    continue;
                }
                let target = if let Some(id) = ids.get(&next) {
                    *id
                } else {
                    if subsets.len() >= limit {
                        return None;
                    }
                    let id = subsets.len() as u64;
                    ids.insert(next.clone(), id);
                    subsets.push(next);
                    id
                };
                if let Some(last) = transitions.last_mut() {
                    if last.target == target && last.range.end + 1 == pair[0] {
                        last.range.end = pair[1] - 1;
                        continue;
                    }
                }
                transitions.push(DfaTransition {
                    range: CharRange {
                        start: pair[0],
                        end: pair[1] - 1,
                    },
                    target,
                });
            }
            states.push(DfaState {
                id: states.len() as u64,
                accepting,
                transitions,
            });
        }
        Some(Dfa {
            start_state: 0,
            states,
        })
    }
}

fn compile(pattern: &str, limit: usize) -> Option<Dfa> {
    if pattern.len() > 65_536 || limit == 0 {
        return None;
    }
    let mut parser = Parser {
        chars: pattern.chars().collect(),
        pos: 0,
    };
    let re = parser.alt(0)?;
    if parser.peek().is_some() {
        return None;
    }
    let mut nfa = Nfa {
        states: vec![],
        limit: STATE_LIMIT,
        work: WORK_LIMIT,
    };
    let (start, end) = nfa.build(&re)?;
    nfa.determinize(start, end, limit)
}

pub(super) fn plan(token: &Token) -> LexicalPlan {
    let (strategy, dfa) = match &token.spec {
        TokenSpec::Regex { pattern, .. } => match compile(pattern, STATE_LIMIT) {
            Some(dfa) => (LexicalPlanStrategy::Dfa, Some(dfa)),
            None => (LexicalPlanStrategy::JavaPattern, None),
        },
        TokenSpec::Extern { .. } => (LexicalPlanStrategy::Extern, None),
        _ => (LexicalPlanStrategy::Scalar, None),
    };
    LexicalPlan {
        token_id: token.symbol_id.clone(),
        strategy,
        dfa,
    }
}

/// 同一計画から token の追加表現を作る。lexical DFA と token 側の表は同じ意味を持つ。
pub(crate) fn attach(semantics: &mut Semantics, plan: &Plan) {
    for (token, lexical) in semantics.tokens.iter_mut().zip(&plan.lexical) {
        if let TokenSpec::Regex {
            dfa,
            fallback,
            regular_subset,
            java_fallback,
            ..
        } = &mut token.spec
        {
            *regular_subset = lexical.dfa.is_some();
            *java_fallback = if *regular_subset {
                TokenSpecRegexJavaFallback::None
            } else {
                TokenSpecRegexJavaFallback::JavaUtilRegexPattern
            };
            *fallback = (!*regular_subset).then_some(RegexFallback);
            *dfa = Some(lexical.dfa.as_ref().map(RegexDfa::from));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_and_resource_limits_fall_back() {
        assert!(compile("ab", 2).is_none());
        assert!(compile("ab", 3).is_some());
        // 小さい NFA から DFA 状態数が指数増加するパターン。
        assert!(compile("(a|b)*a(a|b){5}", 20).is_none());
        assert!(compile("(a|b)*a(a|b){5}", 100).is_some());
        assert!(compile("(a|b)*a(a|b){14}", STATE_LIMIT).is_none());
        assert!(compile("a{10001}", STATE_LIMIT).is_none());
        assert!(compile("a{9999}", STATE_LIMIT).is_none());
        assert!(compile(
            &format!("{}a{}", "(".repeat(129), ")".repeat(129)),
            STATE_LIMIT
        )
        .is_none());
    }
}
