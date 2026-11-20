use super::{charset::CharSet, graph, Context};
use crate::ir::*;
use MayStartZeroConsumeCondition as Zero;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Facts {
    pub chars: CharSet,
    pub eof: bool,
    pub zero: Zero,
    pub min: u64,
    pub matched_min: u64,
    pub consumes: bool,
    pub matches: bool,
    pub trivia: bool,
    pub certainty: MayStartCertainty,
}

impl Facts {
    pub fn bottom() -> Self {
        Self {
            chars: CharSet::default(),
            eof: false,
            zero: Zero::Never,
            min: u64::MAX,
            matched_min: u64::MAX,
            consumes: false,
            matches: false,
            trivia: false,
            certainty: MayStartCertainty::Exact,
        }
    }
    pub fn leaf(chars: CharSet, min: u64) -> Self {
        Self {
            chars,
            min,
            matched_min: min,
            consumes: min > 0,
            matches: min > 0,
            ..Self::bottom()
        }
    }
    pub fn empty() -> Self {
        Self {
            eof: true,
            zero: Zero::Always,
            min: 0,
            matched_min: 0,
            ..Self::bottom()
        }
    }
    pub fn predicate() -> Self {
        Self {
            chars: CharSet::all(),
            zero: Zero::Predicate,
            certainty: MayStartCertainty::Conservative,
            ..Self::empty()
        }
    }
    pub fn unknown() -> Self {
        Self {
            chars: CharSet::all(),
            zero: Zero::Unknown,
            consumes: true,
            matches: true,
            certainty: MayStartCertainty::Unknown,
            ..Self::empty()
        }
    }
    pub fn nullable(&self) -> bool {
        self.zero != Zero::Never
    }
    pub fn minimum(&self) -> u64 {
        if self.min == u64::MAX {
            0
        } else {
            self.min
        }
    }
    pub fn may_start(&self) -> MayStart {
        MayStart {
            certainty: self.certainty,
            ranges: self.chars.ranges(),
            eof: self.eof,
            includes_trivia: self.trivia,
            zero_consume_condition: self.zero,
        }
    }
    fn union(&mut self, other: &Self) {
        self.chars.union(&other.chars);
        self.eof |= other.eof;
        self.zero = zero_union(self.zero, other.zero);
        self.min = self.min.min(other.min);
        self.matched_min = self.matched_min.min(other.matched_min);
        self.consumes |= other.consumes;
        self.matches |= other.matches;
        self.trivia |= other.trivia;
        self.certainty = certainty(self.certainty, other.certainty);
    }
}

fn certainty(a: MayStartCertainty, b: MayStartCertainty) -> MayStartCertainty {
    if a == MayStartCertainty::Unknown || b == MayStartCertainty::Unknown {
        MayStartCertainty::Unknown
    } else if a == MayStartCertainty::Conservative || b == MayStartCertainty::Conservative {
        MayStartCertainty::Conservative
    } else {
        MayStartCertainty::Exact
    }
}
fn zero_union(a: Zero, b: Zero) -> Zero {
    if a == b || b == Zero::Never {
        a
    } else if a == Zero::Never {
        b
    } else if a == Zero::Always || b == Zero::Always {
        Zero::Always
    } else if a == Zero::Unknown || b == Zero::Unknown {
        Zero::Unknown
    } else {
        Zero::Predicate
    }
}
fn zero_seq(a: Zero, b: Zero) -> Zero {
    if a == Zero::Never || b == Zero::Never {
        Zero::Never
    } else if a == Zero::Unknown || b == Zero::Unknown {
        Zero::Unknown
    } else if a == Zero::AtEof || b == Zero::AtEof {
        Zero::AtEof
    } else if a == Zero::Predicate || b == Zero::Predicate {
        Zero::Predicate
    } else {
        Zero::Always
    }
}

pub(super) fn literal(value: &str, case_sensitive: bool) -> Facts {
    let Some(first) = value.chars().next() else {
        return Facts::empty();
    };
    let mut result = Facts::leaf(
        CharSet::chars(&first.to_string()),
        value.chars().count() as u64,
    );
    if !case_sensitive {
        // Java equalsIgnoreCase の Unicode 単純写像は Rust の full case folding と異なる。
        // 誤除外を避け、DFA/Unicode 表が利用可能になるまでは全 code point を残す。
        result.chars = CharSet::all();
        result.certainty = MayStartCertainty::Conservative;
    }
    result
}

pub(super) fn token(spec: &TokenSpec) -> Facts {
    match spec {
        TokenSpec::Declarative { expression } => Facts {
            certainty: MayStartCertainty::Unknown,
            ..if expression.nullable() {
                Facts::unknown()
            } else {
                Facts::leaf(CharSet::all(), 1)
            }
        },
        TokenSpec::Builtin { which, code_points } => {
            let chars = match which {
                BuiltinToken::Number => CharSet::chars("+-.0123456789"),
                BuiltinToken::Identifier | BuiltinToken::AlphabetUnderScore => {
                    CharSet::chars("ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz")
                }
                BuiltinToken::Alphabet => {
                    CharSet::chars("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")
                }
                BuiltinToken::AlphabetNumericUnderScore => CharSet::chars(
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz0123456789",
                ),
                BuiltinToken::SingleQuoted => CharSet::chars("'"),
                BuiltinToken::Quoted => CharSet::chars("\""),
                BuiltinToken::Digit => CharSet::range(48, 57),
                BuiltinToken::Space => CharSet::chars(" \t\n\r\u{b}\u{c}"),
                BuiltinToken::EndOfSource => return token(&TokenSpec::Eof {}),
                BuiltinToken::CodePointSet => {
                    let mut chars = CharSet::default();
                    for &cp in code_points {
                        chars.union(&CharSet::range(cp, cp));
                    }
                    chars
                }
            };
            Facts::leaf(
                chars,
                if matches!(which, BuiltinToken::Quoted | BuiltinToken::SingleQuoted) {
                    2
                } else {
                    1
                },
            )
        }
        TokenSpec::Negation { excluded } => Facts::leaf(CharSet::chars(excluded).complement(), 1),
        TokenSpec::CharRange { min, max } => Facts::leaf(CharSet::range(*min, *max), 1),
        TokenSpec::Any {} => Facts::leaf(CharSet::all(), 1),
        TokenSpec::CaseInsensitive { word, .. } => literal(word, false),
        TokenSpec::Regex { .. } => Facts {
            certainty: MayStartCertainty::Unknown,
            ..Facts::leaf(CharSet::all(), 1)
        },
        TokenSpec::Extern { .. } => Facts::unknown(),
        TokenSpec::Eof {} => Facts {
            zero: Zero::AtEof,
            ..Facts::empty()
        },
        TokenSpec::Empty {} => Facts {
            matches: true,
            ..Facts::empty()
        },
        TokenSpec::Until { .. } => Facts {
            chars: CharSet::all(),
            consumes: true,
            matches: true,
            zero: Zero::Predicate,
            certainty: MayStartCertainty::Conservative,
            ..Facts::empty()
        },
        TokenSpec::Lookahead { pattern } => {
            let mut f = Facts::predicate();
            f.matches = !pattern.is_empty();
            f.matched_min = pattern.chars().count() as u64;
            f
        }
        TokenSpec::NegativeLookahead { .. } => Facts::predicate(),
    }
}

fn delimiters(cx: &Context<'_>, policy: &str) -> CharSet {
    let mut chars = CharSet::default();
    for d in &cx
        .sem
        .trivia
        .policies
        .iter()
        .find(|p| p.id == policy)
        .unwrap()
        .effective_delimiter_set
    {
        match d {
            Delimiter::Lexical { .. } => {
                chars.union(&CharSet::all());
            }
            Delimiter::Characters { code_points } => {
                for &cp in code_points {
                    chars.union(&CharSet::range(cp, cp));
                }
            }
            Delimiter::LineComment { open, .. } | Delimiter::BlockComment { open, .. } => {
                if let Some(c) = open.chars().next() {
                    chars.union(&CharSet::chars(&c.to_string()));
                }
            }
        }
    }
    chars
}

fn evaluate(cx: &Context<'_>, i: usize) -> Facts {
    let e = &cx.sem.expressions[i];
    let mut f = match &e.node {
        ExprNode::Literal {
            value,
            case_sensitive,
            ..
        } => literal(value, *case_sensitive),
        ExprNode::TokenRef { symbol_id } => token(&cx.sem.tokens[cx.tokens[symbol_id]].spec),
        ExprNode::RuleRef { symbol_id } => cx.facts[cx.body(symbol_id)].clone(),
        ExprNode::Any {} => token(&TokenSpec::Any {}),
        ExprNode::Eof {} => token(&TokenSpec::Eof {}),
        ExprNode::Empty {} => token(&TokenSpec::Empty {}),
        ExprNode::Error { .. } => Facts::bottom(),
        ExprNode::Group { child } => cx.fact(child).clone(),
        ExprNode::Lookahead { child } => {
            let c = cx.fact(child);
            Facts {
                matches: c.matches,
                matched_min: c.matched_min,
                ..Facts::predicate()
            }
        }
        ExprNode::NegativeLookahead { .. } => Facts::predicate(),
        ExprNode::Choice { alternatives, .. } => {
            let mut result = Facts::bottom();
            for id in alternatives {
                result.union(cx.fact(id));
            }
            result.certainty = certainty(result.certainty, MayStartCertainty::Conservative);
            result
        }
        ExprNode::Seq { items } => {
            let mut result = Facts::empty();
            let mut prefix_nullable = true;
            for id in items {
                let c = cx.fact(id);
                if prefix_nullable {
                    result.chars.union(&c.chars);
                }
                prefix_nullable &= c.nullable();
                result.eof &= c.eof;
                result.zero = zero_seq(result.zero, c.zero);
                result.min = result.min.saturating_add(c.min);
                result.matched_min = result.matched_min.saturating_add(c.matched_min);
                result.consumes |= c.consumes;
                result.matches |= c.matches;
                result.trivia |= c.trivia;
                result.certainty = certainty(result.certainty, c.certainty);
            }
            result.certainty = certainty(result.certainty, MayStartCertainty::Conservative);
            result
        }
        ExprNode::Optional { child } => {
            let mut f = cx.fact(child).clone();
            f.union(&Facts::empty());
            f
        }
        ExprNode::Repeat {
            child,
            min,
            max,
            terminator,
        } => {
            let c = cx.fact(child);
            if *max == Some(0) {
                // 上限ゼロは子を試行せず、入力に依存しないゼロ幅成功。
                Facts::empty()
            } else {
                let mut f = c.clone();
                f.min = c.min.saturating_mul(*min);
                f.matched_min = c.matched_min.saturating_mul(*min);
                if *min == 0 {
                    f.union(&Facts::empty());
                }
                if let Some(t) = terminator {
                    // terminator は calleeDefault。consumed mode ではその終了カーソルを commit。
                    let t = cx.fact(t);
                    f.chars.union(&t.chars);
                    f.matches |= t.matches;
                    f.consumes |= t.consumes;
                    if *min == 0 {
                        f.matched_min = 0;
                    }
                }
                f
            }
        }
        ExprNode::Separated {
            element,
            separator,
            min,
            ..
        } => {
            let c = cx.fact(element);
            let s = cx.fact(separator);
            let mut f = c.clone();
            f.min = c
                .min
                .saturating_mul(*min)
                .saturating_add(s.min.saturating_mul(min.saturating_sub(1)));
            f.matched_min = c
                .matched_min
                .saturating_mul(*min)
                .saturating_add(s.matched_min.saturating_mul(min.saturating_sub(1)));
            f.consumes |= s.consumes;
            f.matches |= s.matches;
            f.trivia |= s.trivia;
            if *min == 0 {
                f.union(&Facts::empty());
            }
            if c.nullable() {
                f.chars.union(&s.chars);
            }
            if *min > 1 {
                f.zero = zero_seq(c.zero, s.zero);
                f.eof &= s.eof;
            }
            f
        }
    };
    for insertion in cx
        .sem
        .trivia
        .insertion_plan
        .iter()
        .filter(|p| p.boundary.owner_expr_id == e.id)
    {
        let chars = delimiters(cx, &insertion.policy_id);
        if chars == CharSet::default() {
            continue;
        }
        let at_start = match insertion.boundary.at {
            BoundaryAt::Entry | BoundaryAt::HelperEntry => true,
            BoundaryAt::Exit | BoundaryAt::HelperExit => f.nullable(),
            BoundaryAt::AfterChild => match &e.node {
                ExprNode::Seq { items } => items
                    .iter()
                    .take(insertion.boundary.child_index.unwrap() as usize + 1)
                    .all(|id| cx.fact(id).nullable()),
                _ => true,
            },
        };
        if at_start {
            f.chars.union(&chars);
        }
        f.trivia = true;
        f.consumes = true;
        f.matches = true;
        f.certainty = certainty(f.certainty, MayStartCertainty::Conservative);
    }
    // recovery wrapper は指定 ruleRef site のみ。前進/ゼロ幅とも不明として除外しない。
    if cx
        .sem
        .recovery
        .iter()
        .any(|r| r.application_sites.contains(&e.id))
    {
        let mut recovered = Facts::unknown();
        recovered.trivia = f.trivia;
        f = recovered;
    }
    f
}

pub(super) fn solve(cx: &mut Context<'_>) {
    for group in graph::scc(&cx.deps) {
        loop {
            let mut changed = false;
            for &i in &group {
                let next = evaluate(cx, i);
                if next != cx.facts[i] {
                    cx.facts[i] = next;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }
}
