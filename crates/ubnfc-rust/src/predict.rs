//! 通常の `|` への保守的な候補除外（設計 v1 §3.4）。
//!
//! 候補 expression ごとに「成功し得る先頭 code point の集合」を trivia 越しに求める
//! （`Start`）。選択の実行時に、候補の呼出し位置の文字（`direct`）と、候補が最初の
//! token の前に必ず通る trivia policy で読み飛ばした位置の文字（`skipped`）のどちらも
//! 集合に無ければ、その候補は最初の significant token で失敗すると証明できる。
//! そのとき候補は呼ばず、同じ前提の下で候補が実行したはずの runtime 呼出し
//! （display の reach / failures、主診断 DAG の Failure / Rule / Join、rule 深さ検査）
//! だけを再生する `guard_e{i}` 関数を呼ぶ。葉（literal / token）は本物の関数を呼ぶ。
//!
//! 再生と本物の差は memo（lookup / store）と局所 frame（enter / leave）だけで、どちらも
//! 観測量（farthest / expected / rulePath / AST / 効果列）を変えない（memo 有無の同値は
//! 既存 oracle、frame は `open` の位置比較と `leave` の伝播が同じ規則）。
//! 除外できない候補（nullable、先読み、Extern token、scope 効果や recovery を持つ rule、
//! policy が混在する経路など）は `Unknown` として通常経路に残す。
use crate::emit::{suffix, Generator};
use std::collections::BTreeSet;
use std::fmt::Write;
use ubnfc_front::ir::*;

/// 昇順・互いに素な閉区間の集合。
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub(crate) struct Ranges(Vec<(u32, u32)>);
impl Ranges {
    fn single(c: u32) -> Self {
        Self(vec![(c, c)])
    }
    fn range(lo: u32, hi: u32) -> Self {
        if lo <= hi {
            Self(vec![(lo, hi)])
        } else {
            Self::default()
        }
    }
    fn union(&self, other: &Self) -> Self {
        let mut all: Vec<(u32, u32)> = self.0.iter().chain(other.0.iter()).copied().collect();
        all.sort_unstable();
        let mut out: Vec<(u32, u32)> = Vec::with_capacity(all.len());
        for (lo, hi) in all {
            if let Some(last) = out.last_mut() {
                if lo <= last.1.saturating_add(1) {
                    last.1 = last.1.max(hi);
                    continue;
                }
            }
            out.push((lo, hi));
        }
        Self(out)
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// `matches!` の pattern（`c as u32` に対する）。
    fn pattern(&self) -> String {
        self.0
            .iter()
            .map(|(lo, hi)| {
                if lo == hi {
                    lo.to_string()
                } else {
                    format!("{lo}..={hi}")
                }
            })
            .collect::<Vec<_>>()
            .join("|")
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub(crate) struct Known {
    pub nullable: bool,
    pub direct: Ranges,
    pub skipped: Ranges,
    /// `skipped` を読む前に通る trivia policy（正規化済み index）。
    pub policy: Option<usize>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Start {
    Unknown,
    Known(Known),
}

fn union(a: &Known, b: &Known, nullable: bool) -> Start {
    let policy = match (a.policy, b.policy) {
        (Some(p), Some(q)) if p != q => return Start::Unknown,
        (p, q) => p.or(q),
    };
    Start::Known(Known {
        nullable,
        direct: a.direct.union(&b.direct),
        skipped: a.skipped.union(&b.skipped),
        policy,
    })
}

fn wrap(policy: usize, k: &Known) -> Start {
    if k.policy.is_some_and(|q| q != policy) {
        return Start::Unknown;
    }
    let skipped = k.direct.union(&k.skipped);
    Start::Known(Known {
        nullable: k.nullable,
        direct: Ranges::default(),
        policy: (!skipped.is_empty()).then_some(policy),
        skipped,
    })
}

fn foldable_policy(policy: &TriviaPolicy) -> bool {
    !(policy.effective_delimiter_set.len() == 1
        && matches!(
            policy.effective_delimiter_set[0],
            Delimiter::LineComment { .. }
        ))
}

fn leading_byte(cp: u64) -> Option<u8> {
    let c = char::from_u32(u32::try_from(cp).ok()?)?;
    let mut buf = [0u8; 4];
    Some(c.encode_utf8(&mut buf).as_bytes()[0])
}

/// trivia policy の区切りが始まり得る先頭 byte（UTF-8 先頭 byte）。空の区切りがあれば None。
pub(crate) fn trivia_first_bytes(policy: &TriviaPolicy) -> Option<BTreeSet<u8>> {
    let mut bytes = BTreeSet::new();
    for d in &policy.effective_delimiter_set {
        match d {
            Delimiter::Characters { code_points } => {
                for &cp in code_points {
                    bytes.insert(leading_byte(cp)?);
                }
            }
            Delimiter::LineComment { open, .. } | Delimiter::BlockComment { open, .. } => {
                bytes.insert(*open.as_bytes().first()?);
            }
        }
    }
    Some(bytes)
}

/// 同じ区切り集合の policy を同じ index に正規化する。
pub(crate) fn canonical_policies(ir: &GrammarIr) -> Vec<usize> {
    let policies = &ir.semantics.trivia.policies;
    (0..policies.len())
        .map(|i| {
            (0..i)
                .find(|&j| {
                    policies[j].effective_delimiter_set == policies[i].effective_delimiter_set
                })
                .unwrap_or(i)
        })
        .collect()
}

pub(crate) struct Prediction {
    pub starts: Vec<Start>,
    pub templatable: Vec<bool>,
}

fn is_leaf(node: &ExprNode) -> bool {
    matches!(node, ExprNode::Literal { .. } | ExprNode::TokenRef { .. })
}

impl Prediction {
    pub fn new(g: &Generator<'_>) -> Self {
        let ir = g.ir;
        let policies = canonical_policies(ir);
        let n = ir.semantics.expressions.len();
        let mut starts = vec![Start::Known(Known::default()); n];
        let entry_policies = |i: usize| -> Vec<usize> {
            let id = &ir.semantics.expressions[i].id;
            ir.semantics
                .trivia
                .insertion_plan
                .iter()
                .filter(|p| {
                    p.boundary.owner_expr_id == *id
                        && matches!(p.boundary.at, BoundaryAt::Entry | BoundaryAt::HelperEntry)
                })
                .map(|p| {
                    policies[ir
                        .semantics
                        .trivia
                        .policies
                        .iter()
                        .position(|q| q.id == p.policy_id)
                        .unwrap()]
                })
                .collect()
        };
        let after_policies = |i: usize, index: u64| -> Vec<usize> {
            let id = &ir.semantics.expressions[i].id;
            ir.semantics
                .trivia
                .insertion_plan
                .iter()
                .filter(|p| {
                    p.boundary.owner_expr_id == *id
                        && p.boundary.at == BoundaryAt::AfterChild
                        && p.boundary.child_index == Some(index)
                })
                .map(|p| {
                    policies[ir
                        .semantics
                        .trivia
                        .policies
                        .iter()
                        .position(|q| q.id == p.policy_id)
                        .unwrap()]
                })
                .collect()
        };
        loop {
            let mut changed = false;
            for i in 0..n {
                let next = compute(g, i, &starts, &entry_policies, &after_policies);
                if next != starts[i] {
                    starts[i] = next;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        // 再生関数は経路を静的に展開する（共有しない）ので、展開の大きさを上限で抑える。
        // 例: `D28 ::= D27 | D27` の連鎖は指数的に広がるが、本物の経路は memo で線形に収まる。
        let mut cache: Vec<Option<Option<usize>>> = vec![None; n];
        let templatable = (0..n)
            .map(|i| {
                template_size(g, i, &starts, &mut cache, &mut BTreeSet::new())
                    .is_some_and(|size| size <= MAX_TEMPLATE_SIZE)
            })
            .collect();
        Self {
            starts,
            templatable,
        }
    }
}

fn compute(
    g: &Generator<'_>,
    i: usize,
    starts: &[Start],
    entry_policies: &dyn Fn(usize) -> Vec<usize>,
    after_policies: &dyn Fn(usize, u64) -> Vec<usize>,
) -> Start {
    let ir = g.ir;
    let expr = &ir.semantics.expressions[i];
    if expr.execution.child_mode != ExecutionChildMode::Inherit {
        return Start::Unknown;
    }
    let known = |k: Known| Start::Known(k);
    let wrap_all = |mut s: Start, ps: &[usize]| -> Start {
        for &p in ps.iter().rev() {
            s = match s {
                // 行コメントだけの policy は trivia 自体が主診断 node を作る（畳み込みで
                // 呼出し回数が変わると DAG の形が変わる）ので候補除外の対象にしない。
                Start::Known(k) if foldable_policy(&ir.semantics.trivia.policies[p]) => wrap(p, &k),
                _ => Start::Unknown,
            };
        }
        s
    };
    let inner = match &expr.node {
        ExprNode::Literal {
            value,
            case_sensitive: true,
            ..
        } => match value.chars().next() {
            None => known(Known {
                nullable: true,
                ..Default::default()
            }),
            Some(c) => known(Known {
                direct: Ranges::single(c as u32),
                ..Default::default()
            }),
        },
        ExprNode::Literal { .. } => Start::Unknown,
        ExprNode::TokenRef { symbol_id } => token_start(g, symbol_id),
        ExprNode::Seq { items } => {
            let mut acc = Known {
                nullable: true,
                ..Default::default()
            };
            let mut result = None;
            for (index, item) in items.iter().enumerate() {
                let mut ps = Vec::new();
                for j in 0..index {
                    ps.extend(after_policies(i, j as u64));
                }
                let w = wrap_all(starts[g.expr[item.as_str()]].clone(), &ps);
                let Start::Known(w) = w else {
                    result = Some(Start::Unknown);
                    break;
                };
                let nullable = acc.nullable && w.nullable;
                match union(&acc, &w, nullable) {
                    Start::Known(k) => acc = k,
                    Start::Unknown => {
                        result = Some(Start::Unknown);
                        break;
                    }
                }
                if !w.nullable {
                    break;
                }
            }
            result.unwrap_or(Start::Known(acc))
        }
        ExprNode::Choice { alternatives, .. } => {
            let mut acc = Known::default();
            let mut result = None;
            for alt in alternatives {
                let Start::Known(a) = &starts[g.expr[alt.as_str()]] else {
                    result = Some(Start::Unknown);
                    break;
                };
                match union(&acc, a, acc.nullable || a.nullable) {
                    Start::Known(k) => acc = k,
                    Start::Unknown => {
                        result = Some(Start::Unknown);
                        break;
                    }
                }
            }
            result.unwrap_or(Start::Known(acc))
        }
        ExprNode::Group { child } => starts[g.expr[child.as_str()]].clone(),
        ExprNode::Optional { child } => match &starts[g.expr[child.as_str()]] {
            Start::Known(k) => known(Known {
                nullable: true,
                ..k.clone()
            }),
            Start::Unknown => Start::Unknown,
        },
        ExprNode::Repeat {
            child,
            min,
            max,
            terminator: None,
        } if *max != Some(0) => match &starts[g.expr[child.as_str()]] {
            Start::Known(k) => known(Known {
                nullable: k.nullable || *min == 0,
                ..k.clone()
            }),
            Start::Unknown => Start::Unknown,
        },
        ExprNode::RuleRef { symbol_id } => {
            let rule = &ir.semantics.rules[g.rules[symbol_id.as_str()]];
            starts[g.expr[rule.body.as_str()]].clone()
        }
        _ => Start::Unknown,
    };
    wrap_all(inner, &entry_policies(i))
}

fn token_start(g: &Generator<'_>, symbol_id: &str) -> Start {
    let ir = g.ir;
    let token = &ir.semantics.tokens[g.tokens[symbol_id]];
    let dfa = ir
        .plan
        .lexical
        .iter()
        .find(|p| p.token_id == token.symbol_id && p.strategy == LexicalPlanStrategy::Dfa)
        .and_then(|p| p.dfa.as_ref());
    let known = |direct: Ranges| {
        Start::Known(Known {
            direct,
            ..Default::default()
        })
    };
    if let Some(dfa) = dfa {
        let Some(start) = dfa.states.iter().find(|s| s.id == dfa.start_state) else {
            return Start::Unknown;
        };
        let mut r = Ranges::default();
        for t in &start.transitions {
            let (Ok(lo), Ok(hi)) = (u32::try_from(t.range.start), u32::try_from(t.range.end))
            else {
                return Start::Unknown;
            };
            r = r.union(&Ranges::range(lo, hi));
        }
        return known(r);
    }
    let ascii = |chars: &str| {
        chars
            .chars()
            .fold(Ranges::default(), |r, c| r.union(&Ranges::single(c as u32)))
    };
    match &token.spec {
        TokenSpec::Builtin { which, code_points } => match which {
            BuiltinToken::Number => known(ascii("+-.0123456789")),
            BuiltinToken::Identifier | BuiltinToken::AlphabetUnderScore => known(
                Ranges::range('A' as u32, 'Z' as u32)
                    .union(&Ranges::range('a' as u32, 'z' as u32))
                    .union(&Ranges::single('_' as u32)),
            ),
            BuiltinToken::AlphabetNumericUnderScore => known(
                Ranges::range('A' as u32, 'Z' as u32)
                    .union(&Ranges::range('a' as u32, 'z' as u32))
                    .union(&Ranges::range('0' as u32, '9' as u32))
                    .union(&Ranges::single('_' as u32)),
            ),
            BuiltinToken::Alphabet => known(
                Ranges::range('A' as u32, 'Z' as u32).union(&Ranges::range('a' as u32, 'z' as u32)),
            ),
            BuiltinToken::Digit => known(Ranges::range('0' as u32, '9' as u32)),
            BuiltinToken::SingleQuoted => known(Ranges::single('\'' as u32)),
            BuiltinToken::Quoted => known(Ranges::single('"' as u32)),
            BuiltinToken::Space => known(Ranges::range(9, 13).union(&Ranges::single(32))),
            BuiltinToken::CodePointSet => {
                let mut r = Ranges::default();
                for &cp in code_points {
                    let Ok(cp) = u32::try_from(cp) else {
                        return Start::Unknown;
                    };
                    r = r.union(&Ranges::single(cp));
                }
                known(r)
            }
            BuiltinToken::EndOfSource => Start::Unknown,
        },
        TokenSpec::CharRange { min, max } => match (u32::try_from(*min), u32::try_from(*max)) {
            (Ok(lo), Ok(hi)) => known(Ranges::range(lo, hi)),
            _ => Start::Unknown,
        },
        _ => Start::Unknown,
    }
}

/// 失敗経路に副作用が無い rule だけ再生できる。成功時（successBeforeLeave 等）の scope
/// イベントは失敗経路では発火しないので妨げにならない。入口イベントと recovery は除外。
fn rule_templatable(g: &Generator<'_>, r: usize) -> bool {
    let rule = &g.ir.semantics.rules[r];
    let effects = g.ir.semantics.scope.effects.iter().any(|e| {
        e.rule_id == rule.symbol_id
            && e.events
                .iter()
                .any(|ev| ev.when == ScopeEffectEventsWhen::Entry)
    });
    let recovery =
        g.ir.semantics
            .recovery
            .iter()
            .any(|x| x.rule_id == rule.symbol_id);
    !effects && !recovery
}

/// 再生関数 1 つが展開する node（葉・rule・選択）数の上限。
const MAX_TEMPLATE_SIZE: usize = 192;

/// 「最初の significant token で失敗する」前提の失敗経路を静的に再生できるなら、その展開の大きさ。
fn template_size(
    g: &Generator<'_>,
    i: usize,
    starts: &[Start],
    cache: &mut Vec<Option<Option<usize>>>,
    visiting: &mut BTreeSet<usize>,
) -> Option<usize> {
    if let Some(v) = cache[i] {
        return v;
    }
    if !visiting.insert(i) {
        return None;
    }
    let expr = &g.ir.semantics.expressions[i];
    let nonnull = |j: usize| matches!(&starts[j], Start::Known(k) if !k.nullable);
    let child = |g: &Generator<'_>,
                 c: usize,
                 cache: &mut Vec<Option<Option<usize>>>,
                 visiting: &mut BTreeSet<usize>| {
        if nonnull(c) {
            template_size(g, c, starts, cache, visiting)
        } else {
            None
        }
    };
    let size = if !nonnull(i) || expr.execution.child_mode != ExecutionChildMode::Inherit {
        None
    } else {
        match &expr.node {
            ExprNode::Literal { .. } | ExprNode::TokenRef { .. } => Some(1),
            ExprNode::Seq { items } => items
                .first()
                .and_then(|c| child(g, g.expr[c.as_str()], cache, visiting))
                .map(|n| n + 1),
            ExprNode::Choice {
                alternatives,
                selection,
                ..
            } => {
                let plan = g.ir.plan.choices.iter().find(|p| p.expr_id == expr.id);
                let longest = plan.map_or(*selection == ChoiceSelection::Longest, |p| {
                    p.strategy == ChoicePlanStrategy::Longest
                });
                if longest {
                    None
                } else {
                    let mut total = 1usize;
                    let mut ok = true;
                    for a in alternatives {
                        match child(g, g.expr[a.as_str()], cache, visiting) {
                            Some(n) => total = total.saturating_add(n),
                            None => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    ok.then_some(total)
                }
            }
            ExprNode::Group { child: c } => {
                child(g, g.expr[c.as_str()], cache, visiting).map(|n| n + 1)
            }
            ExprNode::RuleRef { symbol_id } => {
                let r = g.rules[symbol_id.as_str()];
                let body = g.expr[g.ir.semantics.rules[r].body.as_str()];
                if rule_templatable(g, r) {
                    child(g, body, cache, visiting).map(|n| n + 1)
                } else {
                    None
                }
            }
            _ => None,
        }
    };
    visiting.remove(&i);
    cache[i] = Some(size);
    size
}

/// 選択 `choice` の候補列 `order` のうち除外対象になる候補と、その判定条件。
pub(crate) struct Guarded {
    pub index: usize,
    pub expr: usize,
    pub known: Known,
}
pub(crate) fn guarded_alternatives(
    g: &Generator<'_>,
    order: &[ExprId],
    longest: bool,
) -> (Vec<Guarded>, Option<usize>) {
    let mut out = Vec::new();
    let mut policy = None;
    if longest {
        return (out, policy);
    }
    for (index, id) in order.iter().enumerate() {
        let i = g.expr[id.as_str()];
        if is_leaf(&g.ir.semantics.expressions[i].node) || !g.prediction.templatable[i] {
            continue;
        }
        let Start::Known(k) = &g.prediction.starts[i] else {
            continue;
        };
        if k.nullable || (k.direct.is_empty() && k.skipped.is_empty()) {
            continue;
        }
        if let Some(p) = k.policy {
            match policy {
                None => policy = Some(p),
                Some(q) if q != p => continue,
                _ => {}
            }
        }
        out.push(Guarded {
            index,
            expr: i,
            known: k.clone(),
        });
    }
    (out, policy)
}

/// 候補が除外されたときの判定式（true なら除外して再生関数を呼ぶ）。
pub(crate) fn skip_condition(k: &Known) -> String {
    let mut parts = Vec::new();
    if !k.direct.is_empty() {
        parts.push(format!(
            "c0.is_some_and(|c| matches!(c as u32,{}))",
            k.direct.pattern()
        ));
    }
    if !k.skipped.is_empty() {
        parts.push(format!(
            "ct.is_some_and(|c| matches!(c as u32,{}))",
            k.skipped.pattern()
        ));
    }
    format!(
        "self.options.predict && !out.state.invert && !({})",
        parts.join(" || ")
    )
}

/// 失敗経路の静的畳み込み。生成コードは、候補が実行したはずの runtime 呼出しのうち
/// 観測に寄与するものだけを同じ順序・同じ位置で行う:
/// - display: 経路上の label は「最初の trivia より前（early）」「trivia 後の位置 T」
///   「trivia 後に T より手前の位置へ記録するもの（late）」に分かれる。early は順に記録、
///   T は 1 回の failures にまとめ、late は位置が T と等しいときだけ記録する
///   （`open` は far より手前の位置を捨て、同じ位置は連結するので同値）。trivia は
///   同じ policy で冪等なので、経路上の最初の 1 回だけ本物を呼ぶ。
/// - 主診断 DAG: literal は Failure(label)@位置、rule は Rule、choice は Join を同じ形で作る。
///   token 葉は本物の expression 関数を呼ぶ（内部の記録位置が token ごとに違うため）。
/// - 深さ・資源: 再生中に limit が発火し得る状態（深さ上限の近傍、予算枯渇、確定失敗）では
///   本物の候補関数へ戻す。
struct Fold<'g, 'a> {
    g: &'g Generator<'a>,
    mode: bool,
    code: String,
    vars: usize,
    seen_trivia: bool,
    t_var: Option<String>,
    early: Option<(String, Vec<String>)>,
    t_labels: Vec<String>,
    late: Vec<(String, Vec<String>)>,
    depth: usize,
    max_depth: usize,
}
impl Fold<'_, '_> {
    fn fresh(&mut self, prefix: &str) -> String {
        self.vars += 1;
        format!("{prefix}{}", self.vars)
    }
    fn flush_early(&mut self) {
        if let Some((var, labels)) = self.early.take() {
            writeln!(
                self.code,
                "self.display.failures({var}.consumed.max({var}.matched),&{labels:?});"
            )
            .unwrap();
        }
    }
    fn record(&mut self, var: &str, labels: Vec<String>) {
        if labels.is_empty() {
            return;
        }
        if !self.seen_trivia {
            match &mut self.early {
                Some((v, l)) if v == var => l.extend(labels),
                _ => {
                    self.flush_early();
                    self.early = Some((var.to_owned(), labels));
                }
            }
        } else if self.t_var.as_deref() == Some(var) {
            self.t_labels.extend(labels);
        } else if let Some((_, l)) = self.late.iter_mut().find(|(v, _)| v == var) {
            l.extend(labels);
        } else {
            self.late.push((var.to_owned(), labels));
        }
    }
    fn trivia(&mut self, id: &ExprId, at: BoundaryAt, var: &str) -> String {
        let g = self.g;
        let mut var = var.to_owned();
        for insertion in &g.ir.semantics.trivia.insertion_plan {
            if insertion.boundary.owner_expr_id == *id
                && insertion.boundary.at == at
                && insertion.boundary.child_index.is_none()
            {
                if self.seen_trivia {
                    // 同じ（正規化で等しい）policy の読み飛ばしは冪等: 位置も label も同じ。
                    var = self.t_var.clone().unwrap();
                    continue;
                }
                let p =
                    g.ir.semantics
                        .trivia
                        .policies
                        .iter()
                        .position(|p| p.id == insertion.policy_id)
                        .unwrap();
                self.flush_early();
                let t = self.fresh("t");
                writeln!(
                    self.code,
                    "let {t}=self.trivia_{p}::<{}>({var}).state;",
                    self.mode
                )
                .unwrap();
                self.seen_trivia = true;
                self.t_var = Some(t.clone());
                var = t;
            }
        }
        var
    }
    /// 式 `i` を状態変数 `var` で実行したときの主診断（変数名）を返す。
    fn walk(&mut self, i: usize, var: &str) -> String {
        let g = self.g;
        let expr = &g.ir.semantics.expressions[i];
        let mode = self.mode;
        let diag = match &expr.node {
            ExprNode::Literal { value, .. } => {
                let label = crate::expressions::label_of(g, expr);
                let d = self.fresh("d");
                writeln!(
                    self.code,
                    "let {d}=self.diag.fail({var}.position::<{mode}>(),{label:?});"
                )
                .unwrap();
                let _ = value;
                let mut labels = vec![g.display.token_label(expr.id.as_str())];
                labels.extend(g.display.expression(expr.id.as_str()));
                self.record(var, labels);
                return d;
            }
            ExprNode::TokenRef { .. } => {
                let d = self.fresh("d");
                writeln!(self.code, "let {d}={}.diag;", g.call(&expr.id, mode, var)).unwrap();
                return d;
            }
            _ => {
                let mut var = var.to_owned();
                if expr.execution.entry_flag == ExecutionEntryFlag::SetFalse
                    || matches!(
                        expr.execution.transaction,
                        ExecutionTransaction::Once | ExecutionTransaction::Occurs
                    )
                {
                    // 束縛せず式にする（使われない変数の警告を避ける。純粋なので重複評価してよい）。
                    let reset = expr.execution.entry_flag == ExecutionEntryFlag::SetFalse;
                    let begin = matches!(
                        expr.execution.transaction,
                        ExecutionTransaction::Once | ExecutionTransaction::Occurs
                    );
                    var = format!("{var}.entered({reset},{begin})");
                }
                var = self.trivia(&expr.id, BoundaryAt::Entry, &var);
                var = self.trivia(&expr.id, BoundaryAt::HelperEntry, &var);
                match &expr.node {
                    ExprNode::Seq { items } => {
                        let c = g.expr[items[0].as_str()];
                        self.walk(c, &var)
                    }
                    ExprNode::Choice { alternatives, .. } => {
                        let plan = g.ir.plan.choices.iter().find(|p| p.expr_id == expr.id);
                        let order = plan
                            .map(|p| &p.candidate_order)
                            .filter(|o| !o.is_empty())
                            .unwrap_or(alternatives);
                        let b = format!("{var}.begin()");
                        let d = self.fresh("d");
                        writeln!(self.code, "let mut {d}=Diag::NONE;").unwrap();
                        for alt in order {
                            let c = g.expr[alt.as_str()];
                            let child = self.walk(c, &b);
                            writeln!(self.code, "{d}=self.diag.join({d},{child});").unwrap();
                        }
                        d
                    }
                    ExprNode::Group { child } => {
                        let c = g.expr[child.as_str()];
                        self.walk(c, &var)
                    }
                    ExprNode::RuleRef { symbol_id } => {
                        let r = g.rules[symbol_id.as_str()];
                        let rule = &g.ir.semantics.rules[r];
                        self.depth += 1;
                        self.max_depth = self.max_depth.max(self.depth);
                        let body = g.expr[rule.body.as_str()];
                        let child = self.walk(body, &var);
                        self.depth -= 1;
                        self.record(&var, g.display.rule(rule.symbol_id.as_str()));
                        let d = self.fresh("d");
                        writeln!(self.code, "let {d}=self.diag.rule({r},{child});").unwrap();
                        d
                    }
                    _ => unreachable!("templatable expression"),
                }
            }
        };
        self.record(var, g.display.expression(expr.id.as_str()));
        diag
    }
}

/// 候補 `e{i}` を「最初の significant token で失敗する」前提で実行したときの観測を再生する。
pub(crate) fn guard_function(g: &Generator<'_>, i: usize, mode: bool) -> String {
    let suf = suffix(mode);
    let mut fold = Fold {
        g,
        mode,
        code: String::new(),
        vars: 0,
        seen_trivia: false,
        t_var: None,
        early: None,
        t_labels: Vec::new(),
        late: Vec::new(),
        depth: 0,
        max_depth: 0,
    };
    let diag = fold.walk(i, "state");
    fold.flush_early();
    let mut s = format!("fn guard_e{i}_{suf}(&mut self,state:State)->Step {{\n");
    writeln!(
        s,
        "if !self.can_replay({}) {{return self.e{i}_{suf}(state);}}",
        fold.max_depth
    )
    .unwrap();
    s.push_str("self.display.reach(state.consumed.max(state.matched));\n");
    s.push_str(&fold.code);
    if let Some(t) = &fold.t_var {
        if !fold.t_labels.is_empty() {
            writeln!(
                s,
                "self.display.failures({t}.consumed.max({t}.matched),&{:?});",
                fold.t_labels
            )
            .unwrap();
        }
        for (var, labels) in &fold.late {
            writeln!(s,"if {var}.consumed.max({var}.matched)=={t}.consumed.max({t}.matched) {{self.display.failures({t}.consumed.max({t}.matched),&{labels:?});}}").unwrap();
        }
    }
    writeln!(
        s,
        "Step {{ok:false,state,events:EventId(0),diag:{diag}}}\n}}"
    )
    .unwrap();
    s
}
