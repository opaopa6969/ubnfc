//! front の認識表を共有の明示 stack VM へ落とす。scan 生成には干渉しない。
use crate::emit::Generator;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use ubnfc_front::ir::*;

fn targets(op: &Ll1Operation) -> Vec<u64> {
    match op {
        Ll1Operation::Sequence { targets } => targets.clone(),
        Ll1Operation::Choice { transitions } => transitions.iter().map(|t| t.target).collect(),
        Ll1Operation::Optional { target }
        | Ll1Operation::Repeat { target, .. }
        | Ll1Operation::Call { target } => vec![*target],
        _ => vec![],
    }
}

/// memo の容量制限を変えないよう、memo 対象を含む閉包は実行しない。
pub(crate) fn roots(g: &Generator<'_>) -> BTreeMap<usize, usize> {
    // 現実装は Session が事象を全く記録しない文法に限定する。
    // 効果を持つ別規則の capture が子孫の事象を読む場合も保守的に除外する。
    if !g.ir.semantics.scope.effects.is_empty() || !g.ir.semantics.recovery.is_empty() {
        return BTreeMap::new();
    }
    let table = &g.ir.plan.ll1;
    table
        .iter()
        .enumerate()
        .filter(|(_, s)| s.root)
        .filter_map(|(root, state)| {
            let mut seen = BTreeSet::new();
            let mut pending = vec![root];
            while let Some(i) = pending.pop() {
                if !seen.insert(i) {
                    continue;
                }
                let state = table.get(i)?;
                let &expr = g.expr.get(state.expr_id.as_str())?;
                if crate::expressions::memoized(g, expr) {
                    return None;
                }
                pending.extend(targets(&state.operation).into_iter().map(|n| n as usize));
            }
            // 単発の葉を VM に渡しても機構の回数は減らない。
            if seen.len() < 3
                || !seen.iter().any(|&i| {
                    matches!(
                        table[i].operation,
                        Ll1Operation::Choice { .. } | Ll1Operation::Repeat { .. }
                    )
                })
            {
                return None;
            }
            Some((g.expr[state.expr_id.as_str()], root))
        })
        .collect()
}

fn ranges(rs: &[CharRange]) -> String {
    format!(
        "&{:?}",
        rs.iter()
            .map(|r| (r.start as u32, r.end as u32))
            .collect::<Vec<_>>()
    )
}

pub(crate) fn generate(g: &Generator<'_>) -> String {
    let mut s = String::from(include_str!("../templates/ll1.rs"));
    s.push_str("\nstatic LL1_TABLE:&[Ll1Op]=&[\n");
    for state in &g.ir.plan.ll1 {
        let op = match &state.operation {
            Ll1Operation::Sequence { targets } => format!("Ll1Op::Sequence(&{targets:?})"),
            Ll1Operation::Choice { transitions } => format!(
                "Ll1Op::Choice(&[{}])",
                transitions
                    .iter()
                    .map(|t| format!("({}, {})", ranges(&t.ranges), t.target))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Ll1Operation::Optional { target } => format!("Ll1Op::Optional({target})"),
            Ll1Operation::Repeat { target, min, max } => {
                format!("Ll1Op::Repeat({target},{min},{max:?})")
            }
            Ll1Operation::Call { target } => format!("Ll1Op::Call({target})"),
            Ll1Operation::Literal {
                value,
                word_boundary,
            } => format!("Ll1Op::Literal({value:?},{word_boundary})"),
            Ll1Operation::Class { ranges: rs } => format!("Ll1Op::Class({})", ranges(rs)),
            Ll1Operation::Eof {} => "Ll1Op::Eof".into(),
            Ll1Operation::Empty {} => "Ll1Op::Sequence(&[])".into(),
        };
        writeln!(s, "{op},").unwrap();
    }
    s.push_str("];\n");
    s
}

pub(crate) fn fast_path(g: &Generator<'_>, i: usize, mode: bool) -> Option<String> {
    if mode {
        return None;
    }
    let root = g.ll1_roots.get(&i)?;
    let reset = if sets_false(g, i, &mut BTreeSet::new()) {
        "next.reset=false;"
    } else {
        ""
    };
    Some(format!("if self.options.ll1 && !DIAG && self.can_replay(0) && !self.options.wants_ast() && !self.options.lexical && !self.options.occurrences && !state.invert && state.matched==state.consumed {{\nif let Some(end)=self.ll1_run({root},state.consumed,|i,p|self.ll1_scan(i,p)) {{let mut next=state;{reset}if end>state.consumed {{next.advance::<false>(end-state.consumed);}}return Step::yes(next);}}\n}}\n"))
}

/// 既存の純粋な一括走査を表の辺として呼ぶ。scan の実装・SWAR 化は変更しない。
pub(crate) fn scanners(g: &Generator<'_>) -> String {
    let mut s = String::from("fn ll1_scan(&self,i:usize,p:usize)->Option<Option<usize>> {let _=p;if !self.options.scan {return None;}match i {\n");
    for (state, row) in g.ir.plan.ll1.iter().enumerate() {
        let i = g.expr[row.expr_id.as_str()];
        if g.scans.roots.contains(&i) {
            writeln!(s, "{state}=>Some(self.s{i}(p).map(|n|p+n)),").unwrap();
        }
    }
    if !s.contains("=>Some(self.s") {
        return "fn ll1_scan(&self,i:usize,p:usize)->Option<Option<usize>> {let _=(i,p);None}\n"
            .into();
    }
    s.push_str("_=>None,}}\n");
    s
}

/// consumed と matched が等しい閉領域では途中の flag は位置に作用しないが、
/// Delegate（Group / RuleRef）の出口 flag は後続の lookahead に渡るので再現する。
fn sets_false(g: &Generator<'_>, i: usize, seen: &mut BTreeSet<usize>) -> bool {
    if !seen.insert(i) {
        return false;
    }
    let e = &g.ir.semantics.expressions[i];
    if e.execution.entry_flag == ExecutionEntryFlag::SetFalse {
        return true;
    }
    match &e.node {
        ExprNode::Group { child } => sets_false(g, g.expr[child.as_str()], seen),
        ExprNode::RuleRef { symbol_id } => sets_false(
            g,
            g.expr[g.ir.semantics.rules[g.rules[symbol_id.as_str()]]
                .body
                .as_str()],
            seen,
        ),
        _ => false,
    }
}
