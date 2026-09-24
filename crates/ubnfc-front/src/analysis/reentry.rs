//! 「その規則は、確定した経路上で**同じ位置**にもう一度入り得るか」の保守的判定。
//!
//! memo が当たる（hit する）唯一の条件は、同じ `(rule, position)` が 2 回試されること。
//! 位置が戻るのは巻戻し（ordered choice の候補失敗、0 消費可能な要素の後続、反復の
//! 周回境界と脱出、先読みの巻戻し）だけなので、**巻戻しで開始位置が一致し得る
//! 兄弟の組**を列挙し、その両側から同じ規則が同じ相対位置で試され得るかを見る。
//!
//! 相対位置は決定不能なので `{Zero, Pos}` の 2 値に丸める。`Zero` は式の開始位置、
//! `Pos` は開始より後、`Tail` は式の終了位置。丸めた側が一致しない組は衝突にしない。
//! 消費済み入力を越えて探索してから戻った先（over-run）を別の兄弟が踏み直す形は
//! モデル化していない。memo は最適化なので受理・AST・診断には影響せず、誤って
//! 落とした場合の代償は速度だけなので、動的な hit 計数で裏を取る前提の近似とする。
use super::Context;
use crate::ir::*;
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Kind {
    /// 式の開始位置で試される規則。
    Zero,
    /// 式の開始位置より後で試される規則。
    Pos,
    /// 式の終了位置で試される規則（末尾の 0 消費探索・反復の最後の失敗周回）。
    Tail,
}

/// 走査の打ち切り。超えたら「全 rule が再入し得る」に倒して memo を残す。
const BUDGET: u64 = 40_000_000;

/// 巻戻しで開始位置が一致し得る兄弟の組。
enum Pair {
    /// 2 つが同じ位置から評価され得る（choice の候補、0 消費要素の後続）。
    SameStart(usize, usize),
    /// 後者が前者の終了位置から評価される（反復の周回境界・脱出、Seq の隣接）。
    Boundary(usize, usize),
}

fn zero_consumable(cx: &Context<'_>, id: &ExprId) -> bool {
    cx.fact(id).minimum() == 0
}

fn successors(cx: &Context<'_>, i: usize, kind: Kind) -> Vec<(usize, Kind)> {
    let node = &cx.sem.expressions[i].node;
    let idx = |id: &ExprId| cx.index(id);
    let zero0 = |id: &ExprId| zero_consumable(cx, id);
    let mut out = Vec::new();
    match kind {
        Kind::Zero => match node {
            ExprNode::RuleRef { symbol_id } => out.push((cx.body(symbol_id), Kind::Zero)),
            ExprNode::Seq { items } => {
                for id in items {
                    out.push((idx(id), Kind::Zero));
                    if !zero0(id) {
                        break;
                    }
                }
            }
            ExprNode::Choice { alternatives, .. } => {
                out.extend(alternatives.iter().map(|id| (idx(id), Kind::Zero)));
            }
            ExprNode::Group { child }
            | ExprNode::Optional { child }
            | ExprNode::Lookahead { child }
            | ExprNode::NegativeLookahead { child } => out.push((idx(child), Kind::Zero)),
            ExprNode::Repeat {
                child, terminator, ..
            } => {
                out.push((idx(child), Kind::Zero));
                out.extend(terminator.iter().map(|id| (idx(id), Kind::Zero)));
            }
            ExprNode::Separated {
                element, separator, ..
            } => {
                out.push((idx(element), Kind::Zero));
                if zero0(element) {
                    out.push((idx(separator), Kind::Zero));
                }
            }
            _ => {}
        },
        Kind::Pos => match node {
            ExprNode::RuleRef { symbol_id } => out.push((cx.body(symbol_id), Kind::Pos)),
            ExprNode::Seq { items } => {
                for (n, id) in items.iter().enumerate() {
                    out.push((idx(id), Kind::Pos));
                    // 先行要素が消費し得るので、2 番目以降は開始位置より後にもなり得る。
                    if n > 0 {
                        out.push((idx(id), Kind::Zero));
                    }
                }
            }
            ExprNode::Choice { alternatives, .. } => {
                out.extend(alternatives.iter().map(|id| (idx(id), Kind::Pos)));
            }
            ExprNode::Group { child }
            | ExprNode::Optional { child }
            | ExprNode::Lookahead { child }
            | ExprNode::NegativeLookahead { child } => out.push((idx(child), Kind::Pos)),
            ExprNode::Repeat {
                child, terminator, ..
            } => {
                // 2 周目以降は開始位置より後。
                out.extend([(idx(child), Kind::Pos), (idx(child), Kind::Zero)]);
                if let Some(id) = terminator {
                    out.extend([(idx(id), Kind::Pos), (idx(id), Kind::Zero)]);
                }
            }
            ExprNode::Separated {
                element, separator, ..
            } => {
                out.extend([
                    (idx(element), Kind::Pos),
                    (idx(element), Kind::Zero),
                    (idx(separator), Kind::Pos),
                    (idx(separator), Kind::Zero),
                ]);
            }
            _ => {}
        },
        Kind::Tail => match node {
            ExprNode::RuleRef { symbol_id } => out.push((cx.body(symbol_id), Kind::Tail)),
            ExprNode::Seq { items } => {
                for id in items.iter().rev() {
                    out.push((idx(id), Kind::Tail));
                    if !zero0(id) {
                        break;
                    }
                    out.push((idx(id), Kind::Zero));
                }
            }
            ExprNode::Choice { alternatives, .. } => {
                out.extend(alternatives.iter().map(|id| (idx(id), Kind::Tail)));
            }
            ExprNode::Group { child } => out.push((idx(child), Kind::Tail)),
            ExprNode::Optional { child } => {
                out.extend([(idx(child), Kind::Tail), (idx(child), Kind::Zero)]);
            }
            // 先読みは 0 消費なので、終了位置＝開始位置。
            ExprNode::Lookahead { child } | ExprNode::NegativeLookahead { child } => {
                out.push((idx(child), Kind::Zero));
            }
            ExprNode::Repeat {
                child, terminator, ..
            } => {
                out.extend([(idx(child), Kind::Tail), (idx(child), Kind::Zero)]);
                if let Some(id) = terminator {
                    out.extend([(idx(id), Kind::Tail), (idx(id), Kind::Zero)]);
                }
            }
            ExprNode::Separated {
                element, separator, ..
            } => {
                out.extend([
                    (idx(element), Kind::Tail),
                    (idx(element), Kind::Zero),
                    (idx(separator), Kind::Tail),
                    (idx(separator), Kind::Zero),
                ]);
            }
            _ => {}
        },
    }
    out
}

/// `(式, 種別)` から到達する規則。深い連鎖でも stack を使わない明示走査。
/// 予算を使い切ったら `None`（＝判定不能）。
fn rules_at(cx: &Context<'_>, start: (usize, Kind), budget: &mut u64) -> Option<BTreeSet<usize>> {
    let mut seen = BTreeSet::new();
    let mut out = BTreeSet::new();
    let mut stack = vec![start];
    while let Some((i, k)) = stack.pop() {
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        if !seen.insert((i, k)) {
            continue;
        }
        if k == Kind::Zero {
            if let ExprNode::RuleRef { symbol_id } = &cx.sem.expressions[i].node {
                out.insert(cx.rules[symbol_id]);
            }
        }
        stack.extend(successors(cx, i, k));
    }
    Some(out)
}

/// 同じ開始位置から評価される 2 つの兄弟が、**開始位置より後の同じ位置**で同じ規則を
/// 踏み直し得るか。入力は 1 本なので、両方が開始位置より先へ進むには**同じ 1 文字目を
/// 消費する**しかない。`mayStart`（`Facts::chars`、必要条件の**上界**）が交わらなければ、
/// 片方が 1 文字でも消費できる入力では他方は開始位置から 1 文字も動けない（動くには
/// その文字を消費する終端が要る）。したがって `Pos∩Pos` の衝突は起こらない。
/// 上界なので「交わらない」は実際に交わらないことを含意する。判定不能
/// （Extern / recovery / regex）は `chars` が全 code point になるので、交わる側に倒れる。
fn may_share_prefix(cx: &Context<'_>, a: usize, b: usize) -> bool {
    let (fa, fb) = (&cx.facts[a], &cx.facts[b]);
    fa.certainty == MayStartCertainty::Unknown
        || fb.certainty == MayStartCertainty::Unknown
        || !fa.chars.disjoint(&fb.chars)
}

fn backtracking_choice(cx: &Context<'_>, i: usize, choices: &[ChoicePlan]) -> bool {
    let id = &cx.sem.expressions[i].id;
    // Predict が fallback 無しで確定するなら候補は 1 つしか評価されない。
    !choices.iter().any(|c| {
        &c.expr_id == id
            && c.strategy == ChoicePlanStrategy::Predict
            && c.fallback == ChoicePlanFallback::None
    })
}

/// 同じ位置で再入され得る規則の index 集合。判定不能・予算超過は「全 rule」に倒す。
pub(super) fn reentrant_rules(cx: &Context<'_>, choices: &[ChoicePlan]) -> BTreeSet<usize> {
    let all = || (0..cx.sem.rules.len()).collect::<BTreeSet<_>>();
    let mut pairs = Vec::new();
    let mut forced = BTreeSet::new();
    let mut budget = BUDGET;
    for i in 0..cx.sem.expressions.len() {
        if !cx.active.contains(&i) {
            continue;
        }
        let idx = |id: &ExprId| cx.index(id);
        match &cx.sem.expressions[i].node {
            ExprNode::Choice { alternatives, .. } if backtracking_choice(cx, i, choices) => {
                for a in 0..alternatives.len() {
                    for b in (a + 1)..alternatives.len() {
                        pairs.push(Pair::SameStart(
                            idx(&alternatives[a]),
                            idx(&alternatives[b]),
                        ));
                    }
                }
            }
            ExprNode::Seq { items } => {
                for a in 0..items.len() {
                    let mut same = zero_consumable(cx, &items[a]);
                    for b in (a + 1)..items.len() {
                        pairs.push(Pair::Boundary(idx(&items[a]), idx(&items[b])));
                        if same {
                            pairs.push(Pair::SameStart(idx(&items[a]), idx(&items[b])));
                        }
                        if !zero_consumable(cx, &items[b]) {
                            break;
                        }
                        same = same && zero_consumable(cx, &items[b]);
                    }
                }
            }
            ExprNode::Repeat {
                child, terminator, ..
            } => {
                pairs.push(Pair::Boundary(idx(child), idx(child)));
                if let Some(t) = terminator {
                    pairs.push(Pair::SameStart(idx(child), idx(t)));
                    pairs.push(Pair::Boundary(idx(child), idx(t)));
                    pairs.push(Pair::Boundary(idx(t), idx(child)));
                }
            }
            ExprNode::Separated {
                element, separator, ..
            } => {
                pairs.push(Pair::Boundary(idx(element), idx(separator)));
                pairs.push(Pair::Boundary(idx(separator), idx(element)));
                pairs.push(Pair::Boundary(idx(element), idx(element)));
            }
            // 先読みの中身は必ず巻戻るので、その部分グラフの規則は後続が同じ位置で踏み直す。
            ExprNode::Lookahead { child } | ExprNode::NegativeLookahead { child } => {
                for e in cx.reachable(idx(child)) {
                    if let ExprNode::RuleRef { symbol_id } = &cx.sem.expressions[e].node {
                        forced.insert(cx.rules[symbol_id]);
                    }
                }
            }
            _ => {}
        }
    }
    let mut cache: HashMap<(usize, Kind), BTreeSet<usize>> = HashMap::new();
    let mut out = forced;
    for pair in &pairs {
        let kinds: Vec<(usize, Kind, usize, Kind)> = match *pair {
            Pair::SameStart(a, b) => {
                let mut kinds = vec![(a, Kind::Zero, b, Kind::Zero)];
                if may_share_prefix(cx, a, b) {
                    kinds.push((a, Kind::Pos, b, Kind::Pos));
                }
                kinds
            }
            Pair::Boundary(a, b) => vec![(a, Kind::Tail, b, Kind::Zero)],
        };
        for (ai, ak, bi, bk) in kinds {
            for key in [(ai, ak), (bi, bk)] {
                if let std::collections::hash_map::Entry::Vacant(slot) = cache.entry(key) {
                    let Some(set) = rules_at(cx, key, &mut budget) else {
                        return all();
                    };
                    slot.insert(set);
                }
            }
            let left = &cache[&(ai, ak)];
            let right = &cache[&(bi, bk)];
            out.extend(left.intersection(right).copied());
        }
    }
    out
}
