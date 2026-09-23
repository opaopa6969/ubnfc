//! 順序付き選択（PEG）の到達不能候補の静的検出。設計 v1 §3.4 / D-037。
//!
//! PEG の `|` は宣言順で最初に成功した候補を採り、後続（継続）の失敗でも別候補へ
//! 戻らない。よって「先行候補 A が成功する位置では後続候補 B は決して走らない」。
//! ここでは *証明できた* 場合だけ警告を出す。受理・AST・診断は一切変えない。
//!
//! 未知（Extern / regex / recovery / 複数 trivia policy）は常に不明として扱い、
//! 警告を出さない。`ubnfc-rust` の predict と同じ保守性の水準を守る。

use super::Context;
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};
use MayStartZeroConsumeCondition as Zero;

/// 証明付きの choice 警告。pipeline が `ValidationIssue` へ変換する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceIssue {
    pub code: String,
    pub grammar: String,
    pub rule: Option<String>,
    pub message: String,
    pub hint: String,
}

/// 左端で呼ばれうる rule への到達情報。
///
/// - `must`: この式が成功したなら、その rule も同じ開始位置で必ず成功していた
/// - `more`: その rule の一致の後に必ず 1 文字以上消費する経路がある
/// - `trivia`: rule 呼出しへ降りる途中で跨いだ trivia 挿入境界の policy 集合
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Lead {
    must: bool,
    more: bool,
    trivia: BTreeSet<String>,
}

type LeadMap = BTreeMap<String, Lead>;

fn merge(into: &mut LeadMap, from: LeadMap, must: bool, more: bool, outer: &BTreeSet<String>) {
    for (rule, mut lead) in from {
        lead.must &= must;
        lead.more |= more;
        lead.trivia.extend(outer.iter().cloned());
        into.entry(rule)
            .and_modify(|e| {
                e.must |= lead.must;
                e.more |= lead.more;
                e.trivia.extend(lead.trivia.iter().cloned());
            })
            .or_insert(lead);
    }
}

struct Reach<'a, 'b> {
    cx: &'a Context<'b>,
    /// 式ごとの entry / helperEntry 挿入 policy。
    entry: Vec<BTreeSet<String>>,
    /// 式ごとの挿入計画（構造同一性の比較に使う）。
    plan: Vec<Vec<(BoundaryAt, Option<u64>, String)>>,
    recovery: Vec<bool>,
    lead: BTreeMap<usize, LeadMap>,
    prefix: BTreeMap<usize, (String, BTreeSet<String>)>,
    stack: Vec<usize>,
    prefix_stack: Vec<usize>,
}

/// 必須リテラル接頭辞の打切り長。これ以上は判定に要らない。
const PREFIX_CAP: usize = 64;

impl Reach<'_, '_> {
    fn node(&self, i: usize) -> &ExprNode {
        &self.cx.sem.expressions[i].node
    }
    fn at(&self, id: &ExprId) -> usize {
        self.cx.exprs[id]
    }
    fn body(&self, symbol_id: &str) -> usize {
        self.at(&self.cx.sem.rules[self.cx.rules[symbol_id]].body)
    }

    /// 左端呼出し関係。左再帰は `analyze` がこの前に拒否しているので循環しないが、
    /// 想定外の循環でも空を返して打ち切る。打ち切った値も memo する: 得られる集合は
    /// 常に下界（＝警告が減る方向）なので健全で、走査順は入力だけで決まるため決定的。
    fn lead(&mut self, i: usize) -> LeadMap {
        if let Some(cached) = self.lead.get(&i) {
            return cached.clone();
        }
        if self.stack.contains(&i) || self.recovery[i] {
            return LeadMap::new();
        }
        self.stack.push(i);
        let outer = self.entry[i].clone();
        let mut out = LeadMap::new();
        match self.node(i).clone() {
            ExprNode::RuleRef { symbol_id } => {
                out.insert(
                    symbol_id.clone(),
                    Lead {
                        must: true,
                        more: false,
                        trivia: outer.clone(),
                    },
                );
                let body = self.body(&symbol_id);
                let map = self.lead(body);
                merge(&mut out, map, true, false, &outer);
            }
            ExprNode::Seq { items } => {
                for (index, item) in items.iter().enumerate() {
                    let rest: u64 = items[index + 1..]
                        .iter()
                        .map(|id| self.cx.fact(id).minimum())
                        .sum();
                    let at = self.at(item);
                    let map = self.lead(at);
                    // Seq は全要素の成功が必要。先行要素が一切消費しない間は開始位置が動かない。
                    merge(&mut out, map, true, rest > 0, &outer);
                    if self.cx.fact(item).consumes {
                        break;
                    }
                }
            }
            ExprNode::Group { child } | ExprNode::Lookahead { child } => {
                let at = self.at(&child);
                let map = self.lead(at);
                merge(&mut out, map, true, false, &outer);
            }
            ExprNode::Optional { child } => {
                let at = self.at(&child);
                let map = self.lead(at);
                merge(&mut out, map, false, false, &outer);
            }
            ExprNode::Choice { alternatives, .. } => {
                let mut common: Option<BTreeSet<String>> = None;
                for id in &alternatives {
                    let at = self.at(id);
                    let map = self.lead(at);
                    let musts: BTreeSet<String> = map
                        .iter()
                        .filter(|(_, lead)| lead.must)
                        .map(|(rule, _)| rule.clone())
                        .collect();
                    common = Some(match common {
                        None => musts,
                        Some(prev) => prev.intersection(&musts).cloned().collect(),
                    });
                    merge(&mut out, map, false, false, &outer);
                }
                // 全候補が必ず呼ぶ rule だけが、選択全体としても必須になる。
                for rule in common.unwrap_or_default() {
                    if let Some(lead) = out.get_mut(&rule) {
                        lead.must = true;
                    }
                }
            }
            ExprNode::Repeat {
                child,
                min,
                terminator,
                ..
            } => {
                let at = self.at(&child);
                let map = self.lead(at);
                // terminator（UNTIL）付きは本体より先に終端判定が走るので必須にしない。
                merge(
                    &mut out,
                    map,
                    min >= 1 && terminator.is_none(),
                    false,
                    &outer,
                );
            }
            ExprNode::Separated { element, min, .. } => {
                let at = self.at(&element);
                let map = self.lead(at);
                merge(&mut out, map, min >= 1, false, &outer);
            }
            _ => {}
        }
        self.stack.pop();
        self.lead.insert(i, out.clone());
        out
    }

    /// 式が必ず先頭に持つ、大文字小文字を区別するリテラル文字列と、
    /// その途中で跨いだ trivia 挿入境界の policy。`PREFIX_CAP` で打ち切る。
    fn literal_prefix(&mut self, i: usize) -> (String, BTreeSet<String>) {
        if let Some(cached) = self.prefix.get(&i) {
            return cached.clone();
        }
        let mut trivia = self.entry[i].clone();
        if self.prefix_stack.contains(&i) || self.recovery[i] {
            return (String::new(), trivia);
        }
        self.prefix_stack.push(i);
        let mut value = String::new();
        match self.node(i).clone() {
            ExprNode::Literal {
                value: literal,
                case_sensitive: true,
                ..
            } => value.push_str(&literal),
            ExprNode::Group { child } => {
                let (v, t) = self.literal_prefix(self.at(&child));
                value.push_str(&v);
                trivia.extend(t);
            }
            ExprNode::RuleRef { symbol_id } => {
                let body = self.body(&symbol_id);
                let (v, t) = self.literal_prefix(body);
                value.push_str(&v);
                trivia.extend(t);
            }
            ExprNode::Seq { items } => {
                for item in &items {
                    let at = self.at(item);
                    let (v, t) = self.literal_prefix(at);
                    value.push_str(&v);
                    trivia.extend(t);
                    // 要素全体がリテラルのときだけ次の要素へ連結できる。
                    let whole = matches!(
                        self.node(at),
                        ExprNode::Literal {
                            case_sensitive: true,
                            ..
                        }
                    );
                    if !whole || value.chars().count() >= PREFIX_CAP {
                        break;
                    }
                }
            }
            ExprNode::Choice { alternatives, .. } => {
                let mut common: Option<String> = None;
                for id in &alternatives {
                    let at = self.at(id);
                    let (v, t) = self.literal_prefix(at);
                    trivia.extend(t);
                    common = Some(match common {
                        None => v,
                        Some(prev) => prev
                            .chars()
                            .zip(v.chars())
                            .take_while(|(a, b)| a == b)
                            .map(|(a, _)| a)
                            .collect(),
                    });
                }
                value = common.unwrap_or_default();
            }
            ExprNode::Repeat { child, min, .. } if min >= 1 => {
                let (v, t) = self.literal_prefix(self.at(&child));
                value.push_str(&v);
                trivia.extend(t);
            }
            ExprNode::Separated { element, min, .. } if min >= 1 => {
                let (v, t) = self.literal_prefix(self.at(&element));
                value.push_str(&v);
                trivia.extend(t);
            }
            _ => {}
        }
        self.prefix_stack.pop();
        let result = (value.chars().take(PREFIX_CAP).collect::<String>(), trivia);
        self.prefix.insert(i, result.clone());
        result
    }

    /// 式木の構造同一性。rule 呼出しは symbol 名で比較して本体へは降りない
    /// （式木は RuleRef を除けば非循環なので停止する）。capture / 診断ラベルは
    /// 成功可否を変えないので無視し、trivia 挿入計画は一致を要求する。
    fn same(&self, a: usize, b: usize) -> bool {
        if self.recovery[a] || self.recovery[b] || self.plan[a] != self.plan[b] {
            return false;
        }
        let pair = |x: &ExprId, y: &ExprId| self.same(self.at(x), self.at(y));
        let list = |x: &[ExprId], y: &[ExprId]| {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| pair(a, b))
        };
        match (self.node(a), self.node(b)) {
            (ExprNode::Seq { items: x }, ExprNode::Seq { items: y }) => list(x, y),
            (
                ExprNode::Choice {
                    alternatives: x,
                    selection: sx,
                    tie_break: tx,
                },
                ExprNode::Choice {
                    alternatives: y,
                    selection: sy,
                    tie_break: ty,
                },
            ) => sx == sy && tx == ty && list(x, y),
            (ExprNode::Optional { child: x }, ExprNode::Optional { child: y })
            | (ExprNode::Group { child: x }, ExprNode::Group { child: y })
            | (ExprNode::Lookahead { child: x }, ExprNode::Lookahead { child: y })
            | (
                ExprNode::NegativeLookahead { child: x },
                ExprNode::NegativeLookahead { child: y },
            ) => pair(x, y),
            (
                ExprNode::Repeat {
                    child: x,
                    max: mx,
                    min: nx,
                    terminator: tx,
                },
                ExprNode::Repeat {
                    child: y,
                    max: my,
                    min: ny,
                    terminator: ty,
                },
            ) => {
                mx == my
                    && nx == ny
                    && pair(x, y)
                    && match (tx, ty) {
                        (None, None) => true,
                        (Some(x), Some(y)) => pair(x, y),
                        _ => false,
                    }
            }
            (
                ExprNode::Separated {
                    element: ex,
                    min: nx,
                    pair_rollback: rx,
                    separator: sx,
                },
                ExprNode::Separated {
                    element: ey,
                    min: ny,
                    pair_rollback: ry,
                    separator: sy,
                },
            ) => nx == ny && rx == ry && pair(ex, ey) && pair(sx, sy),
            (
                ExprNode::Literal {
                    case_sensitive: cx,
                    value: vx,
                    word_boundary: wx,
                },
                ExprNode::Literal {
                    case_sensitive: cy,
                    value: vy,
                    word_boundary: wy,
                },
            ) => cx == cy && vx == vy && wx == wy,
            (ExprNode::RuleRef { symbol_id: x }, ExprNode::RuleRef { symbol_id: y })
            | (ExprNode::TokenRef { symbol_id: x }, ExprNode::TokenRef { symbol_id: y }) => x == y,
            (ExprNode::Any {}, ExprNode::Any {})
            | (ExprNode::Eof {}, ExprNode::Eof {})
            | (ExprNode::Empty {}, ExprNode::Empty {}) => true,
            // Error は生成不能な式。同一視しない。
            _ => false,
        }
    }
}

fn split(symbol: &str) -> (String, String) {
    match symbol.rsplit_once("::") {
        Some((grammar, rule)) => (grammar.to_owned(), rule.to_owned()),
        None => (String::new(), symbol.to_owned()),
    }
}

/// 順序付き選択の到達不能候補を検出する。`analyze` の facts 確定後に呼ぶ。
pub(super) fn check(cx: &Context<'_>) -> Vec<ChoiceIssue> {
    let count = cx.sem.expressions.len();
    let mut entry = vec![BTreeSet::new(); count];
    let mut plan: Vec<Vec<(BoundaryAt, Option<u64>, String)>> = vec![Vec::new(); count];
    for insertion in &cx.sem.trivia.insertion_plan {
        let Some(&i) = cx.exprs.get(&insertion.boundary.owner_expr_id) else {
            continue;
        };
        if matches!(
            insertion.boundary.at,
            BoundaryAt::Entry | BoundaryAt::HelperEntry
        ) {
            entry[i].insert(insertion.policy_id.clone());
        }
        plan[i].push((
            insertion.boundary.at,
            insertion.boundary.child_index,
            insertion.policy_id.clone(),
        ));
    }
    for entries in &mut plan {
        entries.sort();
    }
    let mut recovery = vec![false; count];
    for site in cx.sem.recovery.iter().flat_map(|r| &r.application_sites) {
        if let Some(&i) = cx.exprs.get(site) {
            recovery[i] = true;
        }
    }
    let mut reach = Reach {
        cx,
        entry,
        plan,
        recovery,
        lead: BTreeMap::new(),
        prefix: BTreeMap::new(),
        stack: Vec::new(),
        prefix_stack: Vec::new(),
    };
    let mut issues = Vec::new();
    for i in 0..count {
        // longest は全候補を試すので順序による死は起きない。predictive は候補除外の
        // 証明側で扱うため、ここでは素の ordered だけを見る。
        let ExprNode::Choice {
            alternatives,
            selection: ChoiceSelection::Ordered,
            ..
        } = reach.node(i).clone()
        else {
            continue;
        };
        let (grammar, rule) = split(&cx.sem.expressions[i].rule_id);
        let indexes: Vec<usize> = alternatives.iter().map(|id| reach.at(id)).collect();
        let mut found: Vec<(String, String, String)> = Vec::new();

        // 1. 無条件にゼロ幅で成功しうる候補が末尾でない ⇒ 以降は必ず死ぬ。
        for (k, &a) in indexes.iter().enumerate() {
            if k + 1 == indexes.len() || reach.recovery[a] {
                continue;
            }
            if reach.cx.facts[a].zero == Zero::Always {
                found.push((
                    "W-CHOICE-NULLABLE-NOT-LAST".into(),
                    format!(
                        "alternative {} of rule {rule} always succeeds (it matches the empty string unconditionally), so the {} alternative(s) after it are unreachable",
                        k + 1,
                        indexes.len() - k - 1
                    ),
                    "Move the nullable alternative last, or make it consume input.".into(),
                ));
                break;
            }
        }

        // 2/3. 先行候補 A が成功する位置でしか後続候補 B は成功しない。
        for (k, &a) in indexes.iter().enumerate() {
            if reach.recovery[a] {
                continue;
            }
            let literal = match reach.node(a) {
                ExprNode::Literal {
                    value,
                    case_sensitive: true,
                    word_boundary,
                } if !value.is_empty() && reach.entry[a].is_empty() => {
                    Some((value.clone(), *word_boundary))
                }
                _ => None,
            };
            let rule_ref = match reach.node(a) {
                ExprNode::RuleRef { symbol_id } if reach.entry[a].is_empty() => {
                    Some(symbol_id.clone())
                }
                _ => None,
            };
            for (l, &b) in indexes.iter().enumerate().skip(k + 1) {
                if reach.recovery[b] {
                    continue;
                }
                if reach.same(a, b) {
                    found.push((
                        "W-CHOICE-UNREACHABLE".into(),
                        format!(
                            "alternative {} of rule {rule} is structurally identical to alternative {}, so it is never reached",
                            l + 1,
                            k + 1
                        ),
                        "Remove the duplicated alternative, or make the two differ.".into(),
                    ));
                    continue;
                }
                if let Some((value, boundary)) = &literal {
                    let (prefix, trivia) = reach.literal_prefix(b);
                    // D-040: 語境界つきの `L` は、直後が識別子構成文字なら一致しない。
                    // 「B が一致する位置では A も必ず一致する」と言えるのは、B の必須接頭辞が
                    // `L` の直後に**識別子構成文字でない文字**を持つと分かるときだけ。
                    // `P == L`（続きが不明）や続きが識別子構成文字のときは B が生き残りうる。
                    let covers = |rest: &str| {
                        !*boundary
                            || rest
                                .chars()
                                .next()
                                .is_some_and(|c| !crate::boundary::is_identifier_char(c))
                    };
                    if trivia.is_empty()
                        && prefix.starts_with(value.as_str())
                        && covers(&prefix[value.len()..])
                    {
                        found.push((
                            "W-CHOICE-PREFIX".into(),
                            format!(
                                "alternative {} of rule {rule} ('{value}') is a prefix of every string alternative {} can match (it must start with '{prefix}'), so alternative {} is never reached",
                                k + 1,
                                l + 1,
                                l + 1
                            ),
                            "PEG ordered choice commits to the first success; put the longer literal first.".into(),
                        ));
                        continue;
                    }
                }
                let Some(symbol_id) = &rule_ref else {
                    continue;
                };
                let leads = reach.lead(b);
                let Some(lead) = leads.get(symbol_id) else {
                    continue;
                };
                // trivia を跨ぐ降下は、同じ policy を rule 本体の入口でも読み飛ばすときだけ
                // 位置が一致する（trivia の読み飛ばしは最長一致で冪等）。
                let body_entry = &reach.entry[reach.body(symbol_id)];
                if !lead.trivia.is_empty()
                    && !(lead.trivia.len() == 1 && &lead.trivia == body_entry)
                {
                    continue;
                }
                let name = split(symbol_id).1;
                if lead.must {
                    found.push((
                        "W-CHOICE-UNREACHABLE".into(),
                        format!(
                            "alternative {} of rule {rule} always starts by matching {name} at the same position, and alternative {} is {name} itself, so alternative {} is never reached",
                            l + 1,
                            k + 1,
                            l + 1
                        ),
                        "PEG ordered choice commits to the first success; remove the alternative or move it before.".into(),
                    ));
                } else if lead.more {
                    found.push((
                        "W-CHOICE-SHADOWED-PATH".into(),
                        format!(
                            "alternative {} of rule {rule} has a derivation that starts with {name} and then consumes more, but alternative {} is {name} itself, so that derivation is never reached",
                            l + 1,
                            k + 1
                        ),
                        "PEG ordered choice does not retry a later alternative when the continuation fails; move the longer alternative before.".into(),
                    ));
                }
            }
        }
        issues.extend(found.into_iter().map(|(code, message, hint)| ChoiceIssue {
            code,
            grammar: grammar.clone(),
            rule: Some(rule.clone()),
            message,
            hint,
        }));
    }
    issues.sort_by(|a, b| {
        (&a.grammar, &a.rule, &a.code, &a.message).cmp(&(&b.grammar, &b.rule, &b.code, &b.message))
    });
    issues.dedup();
    issues
}
