//! 順序付き選択の候補列の左因数分解（設計 v1 §3.4 の「候補除外は保守的証明付き」と同じ
//! 立場で、`ParseOptions::factor` で参照経路へ戻せる）。
//!
//! `X A | X B` の形（連続する候補 Seq が構造的に等しい先頭要素列を持つ）で、共有接頭辞を
//! 1 回だけ評価し、続きを宣言順に試す trie を生成する。PEG の意味論では X が同じ状態で
//! 同じ結果を返す（memo が保証する決定性）ので `X A | X B ≡ X (A | B)` が成り立つ。
//! 観測量の同値:
//! - 主診断 DAG: 参照経路は候補ごとに X の診断（memo hit でも `Rule` node は候補ごとに新しく、
//!   子は同じ）を `Join` する。`summary` は位置・label の初出順・最深 rule 経路（左優先）
//!   だけを見るので、同じ位置・同じ label 列・同じ深さの複製 1 個と n 個は区別できない。
//!   `join` は結合的（位置の最大、同位置の `Join` は左から DFS）。
//! - 公開 farthest（display frame）: 観測は「最遠位置とその位置の label 集合」で、同じ
//!   位置への同じ label の記録は冪等。呼ばれない wrapper（候補 Seq、共有要素の候補側の
//!   expression）が失敗時に記録する label は同じ位置に再生する。
//! - capture / Values / Token イベント: 共有要素は先頭候補の expression 関数で評価するので、
//!   採用された候補が別のときは Capture の site と Token の expr をその候補のものに付け替える
//!   （`Session::retag`）。共有要素の wrapper は memo されない（単一の親）ので、その event は
//!   この評価の結果からしか参照されない。候補 Seq の Values / capture / 出口 trivia は
//!   採用候補のものを生成する。
//! - scope 効果: 共有接頭辞の効果は 1 回だけ記録され、各候補は接頭辞後の mark へ restore
//!   して続きを試す。参照経路（候補ごとに mark へ restore して X を再評価 = memo hit で
//!   効果を再生）と最終的な効果列は同じ。
//! - 統計（rule 評価回数・memo hit）と診断予算の計数（記録回数）だけが変わる。
//!
//! 対象外（群にしない）: 最長一致、Seq でない候補、先頭要素が RuleRef / literal / token
//! 以外、execution・trivia policy・capture 規則・label が一致しない、要素 wrapper が memo
//! される、群内で候補除外の条件（predict.rs）が一致しない。rule 境界を越える接頭辞
//! （`A ::= B C | B` で B が別 rule の本体の先頭…）は Rule イベント・Rule node の形が変わる
//! ので対象外。
use crate::{
    emit::{suffix, Generator},
    expressions::{capture_events, label_of, memoized, semantic_shape_of},
    predict::{skip_condition, Guarded},
};
use std::fmt::Write;
use ubnfc_front::ir::*;

/// 候補 `i`（Seq）の要素列。Seq でなければ None。
fn items(g: &Generator<'_>, i: usize) -> Option<Vec<usize>> {
    match &g.ir.semantics.expressions[i].node {
        ExprNode::Seq { items } => Some(items.iter().map(|c| g.expr[c.as_str()]).collect()),
        _ => None,
    }
}

fn policies_at(g: &Generator<'_>, id: &ExprId, at: BoundaryAt, index: Option<u64>) -> Vec<String> {
    g.ir.semantics
        .trivia
        .insertion_plan
        .iter()
        .filter(|p| {
            p.boundary.owner_expr_id == *id
                && p.boundary.at == at
                && p.boundary.child_index == index
        })
        .map(|p| p.policy_id.clone())
        .collect()
}

/// 共有できる要素の性質（構造・実行 flag・trivia・capture 規則・label）。
#[derive(PartialEq, Eq, Debug)]
struct Shape {
    node: String,
    entry_flag: ExecutionEntryFlag,
    transaction: ExecutionTransaction,
    trivia: Vec<(BoundaryAt, Vec<String>)>,
    captures: Vec<(CaptureSpanRule, bool)>,
    label: String,
    token_label: String,
    display: Vec<String>,
}

fn element_shape(g: &Generator<'_>, i: usize) -> Option<Shape> {
    let expr = &g.ir.semantics.expressions[i];
    if expr.execution.child_mode != ExecutionChildMode::Inherit || memoized(g, i) {
        return None;
    }
    let node = match &expr.node {
        ExprNode::RuleRef { symbol_id } => format!("rule {symbol_id}"),
        ExprNode::TokenRef { symbol_id } => format!("token {symbol_id}"),
        ExprNode::Literal {
            value,
            case_sensitive,
            word_boundary,
        } => format!("literal {case_sensitive} {word_boundary} {value:?}"),
        _ => return None,
    };
    if expr.capture_site_ids.len() > 1 {
        return None;
    }
    let captures = expr
        .capture_site_ids
        .iter()
        .map(|s| {
            let c = &g.ir.semantics.captures[g.sites[s.as_str()]];
            (c.span_rule, c.skip)
        })
        .collect();
    Some(Shape {
        node,
        captures,
        entry_flag: expr.execution.entry_flag,
        transaction: expr.execution.transaction,
        trivia: [
            BoundaryAt::Entry,
            BoundaryAt::HelperEntry,
            BoundaryAt::Exit,
            BoundaryAt::HelperExit,
        ]
        .into_iter()
        .map(|at| (at, policies_at(g, &expr.id, at, None)))
        .collect(),
        label: label_of(g, expr).to_owned(),
        token_label: g.display.token_label(expr.id.as_str()),
        display: g.display.expression(expr.id.as_str()),
    })
}

/// 候補 Seq の群の共通性質（execution・入口 trivia）。
fn seq_shape(
    g: &Generator<'_>,
    i: usize,
) -> Option<(
    ExecutionEntryFlag,
    ExecutionTransaction,
    Vec<String>,
    Vec<String>,
)> {
    let expr = &g.ir.semantics.expressions[i];
    if expr.execution.child_mode != ExecutionChildMode::Inherit || memoized(g, i) {
        return None;
    }
    if !matches!(expr.node, ExprNode::Seq { .. }) {
        return None;
    }
    Some((
        expr.execution.entry_flag,
        expr.execution.transaction,
        policies_at(g, &expr.id, BoundaryAt::Entry, None),
        policies_at(g, &expr.id, BoundaryAt::HelperEntry, None),
    ))
}

/// 候補 `a` と `b` が index `depth` の要素を共有できるか（それ以前の要素は既に検査済み）。
fn share(g: &Generator<'_>, a: usize, b: usize, depth: usize) -> bool {
    let (Some(ia), Some(ib)) = (items(g, a), items(g, b)) else {
        return false;
    };
    let (Some(&ea), Some(&eb)) = (ia.get(depth), ib.get(depth)) else {
        return false;
    };
    let (Some(sa), Some(sb)) = (element_shape(g, ea), element_shape(g, eb)) else {
        return false;
    };
    sa == sb
        && policies_at(
            g,
            &g.ir.semantics.expressions[a].id,
            BoundaryAt::AfterChild,
            Some(depth as u64),
        ) == policies_at(
            g,
            &g.ir.semantics.expressions[b].id,
            BoundaryAt::AfterChild,
            Some(depth as u64),
        )
}

/// 候補列の分解結果。`groups` は先頭要素を共有する連続候補の区間（長さ 2 以上）。
pub(crate) struct Plan {
    /// 先頭要素を共有する連続候補の区間 [start, end)（候補 index は `order` 内）。
    pub groups: Vec<(usize, usize)>,
}

/// 順序付き選択 `order` の分解を計画する。群が無ければ None。
pub(crate) fn plan(g: &Generator<'_>, order: &[ExprId], guarded: &[Guarded]) -> Option<Plan> {
    let cands: Vec<usize> = order.iter().map(|id| g.expr[id.as_str()]).collect();
    let mut groups = Vec::new();
    let mut i = 0;
    while i < cands.len() {
        let mut j = i + 1;
        if seq_shape(g, cands[i]).is_some() {
            while j < cands.len()
                && seq_shape(g, cands[j]) == seq_shape(g, cands[i])
                && share(g, cands[i], cands[j], 0)
                && guarded.iter().find(|x| x.index == i).map(|x| &x.known)
                    == guarded.iter().find(|x| x.index == j).map(|x| &x.known)
            {
                j += 1;
            }
        }
        if j - i >= 2 {
            groups.push((i, j));
        }
        i = j;
    }
    (!groups.is_empty()).then_some(Plan { groups })
}

/// 呼ばれない wrapper が失敗時に記録する display label（Fold::walk と同じ内訳）。
fn wrapper_labels(g: &Generator<'_>, i: usize) -> Vec<String> {
    let expr = &g.ir.semantics.expressions[i];
    let mut labels = Vec::new();
    match &expr.node {
        ExprNode::Literal { .. } => labels.push(g.display.token_label(expr.id.as_str())),
        ExprNode::TokenRef { symbol_id } => {
            let token = &g.ir.semantics.tokens[g.tokens[symbol_id.as_str()]];
            if !matches!(token.spec, TokenSpec::Extern { .. }) {
                labels.push(g.display.token_label(expr.id.as_str()));
            }
        }
        _ => {}
    }
    labels.extend(g.display.expression(expr.id.as_str()));
    labels
}

/// 同じ位置への label 記録をまとめる（記録は冪等・順序不問。summary は内容で重複除去する）。
fn merged_failures(records: Vec<(String, Vec<String>)>) -> String {
    let mut merged: Vec<(String, Vec<String>)> = Vec::new();
    for (pos, labels) in records {
        if let Some((_, l)) = merged.iter_mut().find(|(p, _)| *p == pos) {
            for label in labels {
                if !l.contains(&label) {
                    l.push(label);
                }
            }
        } else {
            let mut l: Vec<String> = Vec::new();
            for label in labels {
                if !l.contains(&label) {
                    l.push(label);
                }
            }
            merged.push((pos, l));
        }
    }
    let mut s = String::new();
    for (pos, labels) in merged {
        if !labels.is_empty() {
            writeln!(
                s,
                "self.display_failures({pos}.consumed.max({pos}.matched),&{labels:?});"
            )
            .unwrap();
        }
    }
    s
}

struct Emit<'g, 'a> {
    g: &'g Generator<'a>,
    mode: bool,
    cands: Vec<usize>,
    code: String,
    vars: usize,
}
impl Emit<'_, '_> {
    fn fresh(&mut self, prefix: &str) -> String {
        self.vars += 1;
        format!("{prefix}{}", self.vars)
    }
    fn trivia(&mut self, id: &ExprId, at: BoundaryAt, index: Option<u64>) {
        let text = self.g.trivia_with(id, at, index, self.mode, "cur");
        self.code.push_str(&text);
    }
    /// 候補 `k`（`items` の `from` 以降）を実行し、成功なら tail を生成して抜ける。
    /// 失敗時は diagnostic に join し、`base` へ戻して次へ進む（呼出し側の流れ）。
    fn candidate_tail(&mut self, k: usize, from: usize, shared: &[(String, usize)]) {
        let g = self.g;
        let seq = &g.ir.semantics.expressions[self.cands[k]];
        let items = items(g, self.cands[k]).unwrap();
        let fail = self.fresh("f");
        writeln!(self.code, "'{fail}: {{").unwrap();
        for (index, &item) in items.iter().enumerate().skip(from) {
            writeln!(
                self.code,
                "let child={};cur=self.combine(cur,child);if !cur.ok {{break '{fail};}}",
                g.call(&g.ir.semantics.expressions[item].id, self.mode, "cur.state")
            )
            .unwrap();
            self.trivia(&seq.id, BoundaryAt::AfterChild, Some(index as u64));
        }
        self.success(k, shared);
        writeln!(self.code, "}}").unwrap();
        // 失敗: 候補 Seq の wrapper が記録する label と診断。
        let labels = g.display.expression(seq.id.as_str());
        self.code
            .push_str(&merged_failures(vec![("entry".to_owned(), labels)]));
        writeln!(self.code, "diagnostic=self.diag_join(diagnostic,cur.diag);").unwrap();
    }
    /// 候補 `k` の成功: 出口 trivia、transaction、Values、capture、共有要素の付け替え。
    fn success(&mut self, k: usize, shared: &[(String, usize)]) {
        let g = self.g;
        let seq = &g.ir.semantics.expressions[self.cands[k]];
        self.trivia(&seq.id, BoundaryAt::Exit, None);
        self.trivia(&seq.id, BoundaryAt::HelperExit, None);
        if matches!(
            seq.execution.transaction,
            ExecutionTransaction::Once | ExecutionTransaction::Occurs
        ) {
            self.code.push_str("cur.state=st.commit(cur.state);\n");
        }
        if let Some(count) = semantic_shape_of(g, &seq.id) {
            let wrap =
                count != Cardinality::List && !seq.id.as_str().ends_with("/lower/separated/2");
            writeln!(self.code,"cur.events=self.event(Event::Values{{span:[entry.consumed,cur.state.consumed],child:cur.events,wrap:{wrap},text:false}});").unwrap();
        } else if g.text_alternatives.contains(&self.cands[k]) {
            self.code.push_str(
                &crate::expressions::TEXT_ALTERNATIVE_VALUES
                    .replace("out.", "cur.")
                    .replace("[state.", "[entry."),
            );
        }
        self.code.push_str(&capture_events(g, seq, "entry", "cur"));
        let items = items(g, self.cands[k]).unwrap();
        for (var, from) in shared {
            // 共有要素は候補 `from` の関数で評価した。採用候補の site / expr に付け替える。
            if *from == k {
                continue;
            }
            let index = shared
                .iter()
                .position(|(v, _)| v == var)
                .expect("shared element index");
            let elem = items[index];
            let expr = &g.ir.semantics.expressions[elem];
            let site = expr
                .capture_site_ids
                .first()
                .map(|s| g.sites[s.as_str()])
                .filter(|&s| !g.ir.semantics.captures[s].skip);
            let token = matches!(
                expr.node,
                ExprNode::Literal { .. } | ExprNode::TokenRef { .. }
            )
            .then_some(elem);
            if site.is_some() || token.is_some() {
                writeln!(self.code, "self.retag({var},{site:?},{token:?});").unwrap();
            }
        }
        self.code
            .push_str("diagnostic=self.diag_join(diagnostic,cur.diag);break 'factored cur;\n");
    }
    /// 候補 `group`（`order` 内 index の区間）は `depth` 個の要素を共有済み。
    /// `shared` は共有要素の event 変数と評価に使った候補。
    fn level(&mut self, group: (usize, usize), depth: usize, shared: &mut Vec<(String, usize)>) {
        let g = self.g;
        let (start, end) = group;
        // この段の群（連続候補の区間）。接頭辞で完結する候補は単独の群で、以降は到達しない。
        let mut groups = Vec::new();
        let mut i = start;
        while i < end {
            let mut j = i + 1;
            let complete = items(g, self.cands[i]).unwrap().len() <= depth;
            if !complete {
                while j < end && share(g, self.cands[i], self.cands[j], depth) {
                    j += 1;
                }
            }
            groups.push((i, j, complete));
            if complete {
                break;
            }
            i = j;
        }
        let mark = self.fresh("m");
        let base = self.fresh("b");
        if groups.len() > 1 {
            writeln!(self.code, "let {mark}=self.mark();let {base}=cur;").unwrap();
        }
        for (n, &(i, j, complete)) in groups.iter().enumerate() {
            if n > 0 {
                writeln!(self.code, "self.restore({mark});cur={base};").unwrap();
            }
            if complete {
                // 接頭辞で完結する候補: ここで成功する（以降の候補は参照経路でも試されない）。
                self.success(i, shared);
                return;
            }
            if j - i == 1 {
                self.candidate_tail(i, depth, shared);
            } else {
                let items_i = items(g, self.cands[i]).unwrap();
                let elem = items_i[depth];
                let seq = &g.ir.semantics.expressions[self.cands[i]];
                let ev = self.fresh("ev");
                let pos = self.fresh("p");
                writeln!(self.code, "let {pos}=cur.state;").unwrap();
                // pos / ev は label 再生や付け替えが無ければ使われない（State / EventId は Copy）。
                writeln!(
                    self.code,
                    "let child={};let {ev}=child.events;let _=({pos},{ev});cur=self.combine(cur,child);",
                    g.call(&g.ir.semantics.expressions[elem].id, self.mode, "cur.state")
                )
                .unwrap();
                self.code.push_str("if !cur.ok {\n");
                // 呼ばれない wrapper の label: 他候補の要素（同じ位置）と群の各候補 Seq（入口位置）。
                let mut records = Vec::new();
                for k in i..j {
                    if k != i {
                        records.push((
                            pos.clone(),
                            wrapper_labels(g, items(g, self.cands[k]).unwrap()[depth]),
                        ));
                    }
                    records.push((
                        "entry".to_owned(),
                        g.display
                            .expression(g.ir.semantics.expressions[self.cands[k]].id.as_str()),
                    ));
                }
                self.code.push_str(&merged_failures(records));
                self.code
                    .push_str("diagnostic=self.diag_join(diagnostic,cur.diag);\n} else {\n");
                self.trivia(&seq.id, BoundaryAt::AfterChild, Some(depth as u64));
                shared.push((ev, i));
                self.level((i, j), depth + 1, shared);
                shared.pop();
                self.code.push_str("}\n");
            }
        }
    }
}

/// 選択の候補列の因数分解経路を生成する。`'choice` block の中で、参照経路の前に置く。
/// 全候補が失敗すると `diagnostic` を更新して抜け、呼出し側が参照経路と同じ失敗を返す。
pub(crate) fn emit(
    g: &Generator<'_>,
    order: &[ExprId],
    mode: bool,
    plan: &Plan,
    guarded: &[Guarded],
    reference_candidate: &dyn Fn(usize) -> String,
) -> String {
    let mut e = Emit {
        g,
        mode,
        cands: order.iter().map(|id| g.expr[id.as_str()]).collect(),
        code: String::new(),
        vars: 0,
    };
    let mut s = String::from("if self.options.factor && !out.state.invert {\nlet entry=out.state.begin();\nlet mut child='factored: {\n");
    let mut i = 0;
    while i < e.cands.len() {
        if let Some(&(start, end)) = plan.groups.iter().find(|(s, _)| *s == i) {
            let seq = &g.ir.semantics.expressions[e.cands[start]];
            e.code.clear();
            e.vars = 0;
            writeln!(s, "self.restore(mark);\n{{let mut st=entry;").unwrap();
            if seq.execution.entry_flag == ExecutionEntryFlag::SetFalse {
                s.push_str("st.reset=false;\n");
            }
            s.push_str("let mut cur=Step::yes(st);\n");
            if matches!(
                seq.execution.transaction,
                ExecutionTransaction::Once | ExecutionTransaction::Occurs
            ) {
                s.push_str("cur.state=st.begin();\n");
            }
            s.push_str(&g.trivia_with(&seq.id, BoundaryAt::Entry, None, mode, "cur"));
            s.push_str(&g.trivia_with(&seq.id, BoundaryAt::HelperEntry, None, mode, "cur"));
            // 群の先頭要素: 候補除外（predict.rs）は共有要素 1 回に対して行う。
            let elem = items(g, e.cands[start]).unwrap()[0];
            let elem_id = &g.ir.semantics.expressions[elem].id;
            let guard = guarded
                .iter()
                .find(|x| x.index == start)
                .filter(|_| g.guards.contains(&(elem, mode)));
            let call = match guard {
                Some(x) => format!(
                    "if {} {{self.guard_e{}_{}(cur.state)}} else {{{}}}",
                    skip_condition(&x.known),
                    elem,
                    suffix(mode),
                    g.call(elem_id, mode, "cur.state")
                ),
                None => g.call(elem_id, mode, "cur.state"),
            };
            writeln!(s, "let p0=cur.state;let child={call};let ev0=child.events;let _=(p0,ev0);cur=self.combine(cur,child);").unwrap();
            s.push_str("if !cur.ok {\n");
            let mut records = Vec::new();
            for k in start..end {
                if k != start {
                    records.push((
                        "p0".to_owned(),
                        wrapper_labels(g, items(g, e.cands[k]).unwrap()[0]),
                    ));
                }
                records.push((
                    "entry".to_owned(),
                    g.display
                        .expression(g.ir.semantics.expressions[e.cands[k]].id.as_str()),
                ));
            }
            s.push_str(&merged_failures(records));
            s.push_str("diagnostic=self.diag_join(diagnostic,cur.diag);\n} else {\n");
            s.push_str(&g.trivia_with(&seq.id, BoundaryAt::AfterChild, Some(0), mode, "cur"));
            let mut shared = vec![("ev0".to_owned(), start)];
            e.level((start, end), 1, &mut shared);
            s.push_str(&e.code);
            s.push_str("}\n}\n");
            i = end;
        } else {
            s.push_str(&reference_candidate(i));
            s.push_str("if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}\n");
            i += 1;
        }
    }
    s.push_str("let _=entry;self.restore(mark);out.ok=false;out.diag=diagnostic;out\n};\nif child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}\nself.restore(mark);break 'choice child;\n}\n");
    s
}
