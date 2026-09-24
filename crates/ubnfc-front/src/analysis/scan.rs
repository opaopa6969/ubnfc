//! D-067 — 観測を要求しない終端領域を 1 本の走査へ畳む計画（`plan.scans`）。
//!
//! scannerless PEG は 1 文字ごとに文法そのものを歩く。JSON の文字列本体
//! `String ::= '"' ( { Char } ) '"'` は 1 文字につき規則呼出し 1 回・選択 1 回・
//! 終端 1 回・mark/restore 1 組・cursor 更新 2 回を払っていた
//! （`docs/reports/2026-09-24-json-perf.md` §3）。
//!
//! ここで証明するのは「その領域は**終端だけ**でできていて、評価が**観測を 1 つも
//! 残さない**」こと。観測とは capture site、scope 効果、recovery、Extern token、
//! trivia 挿入、規則の mapping（AST 節点）である。これが成り立つ領域の結果は
//! 「受理したか」と「どこまで消費したか」だけなので、backend は parser の機構
//! （Step / 事象 arena / 診断 / mark / transaction）を通さずに同じ判定を再生できる。
//!
//! 失敗時の診断は D-023 が保証する: fast session が失敗したら診断 session が
//! 非最適化経路で入力全体を解析し直すので、走査経路は**成功した解析にしか使われない**。
//! そのため主診断 DAG・display・expected は構成上 1 bit も変わらない。

use super::{charset::CharSet, children, Context};
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};

/// 単一 code point を消費する終端の文字集合。`None` は「単一 code point のクラスではない」。
fn leaf_class(cx: &Context<'_>, i: usize) -> Option<CharSet> {
    match &cx.sem.expressions[i].node {
        ExprNode::Any {} => Some(CharSet::all()),
        ExprNode::Literal {
            value,
            case_sensitive,
            word_boundary,
        } => {
            if *word_boundary || !*case_sensitive || value.chars().count() != 1 {
                return None;
            }
            let c = value.chars().next()?;
            Some(CharSet::range(c as u64, c as u64))
        }
        ExprNode::TokenRef { symbol_id } => token_class(&cx.sem.tokens[cx.tokens[symbol_id]]),
        _ => None,
    }
}

/// 単一 code point を消費する token の文字集合。
fn token_class(token: &Token) -> Option<CharSet> {
    let ascii = |ranges: &[(u64, u64)]| {
        let mut set = CharSet::default();
        for &(lo, hi) in ranges {
            set.union(&CharSet::range(lo, hi));
        }
        set
    };
    match &token.spec {
        TokenSpec::CharRange { min, max } => Some(CharSet::range(*min, *max)),
        TokenSpec::Negation { excluded } => Some(CharSet::chars(excluded).complement()),
        TokenSpec::Any {} => Some(CharSet::all()),
        TokenSpec::Builtin { which, code_points } => match which {
            BuiltinToken::Digit => Some(ascii(&[(0x30, 0x39)])),
            BuiltinToken::Alphabet => Some(ascii(&[(0x41, 0x5a), (0x61, 0x7a)])),
            BuiltinToken::AlphabetUnderScore => {
                Some(ascii(&[(0x41, 0x5a), (0x5f, 0x5f), (0x61, 0x7a)]))
            }
            BuiltinToken::AlphabetNumericUnderScore => Some(ascii(&[
                (0x30, 0x39),
                (0x41, 0x5a),
                (0x5f, 0x5f),
                (0x61, 0x7a),
            ])),
            BuiltinToken::Space => Some(ascii(&[(9, 13), (32, 32)])),
            BuiltinToken::CodePointSet => Some(ascii(
                &code_points.iter().map(|&c| (c, c)).collect::<Vec<_>>(),
            )),
            _ => None,
        },
        _ => None,
    }
}

/// 終端だけで走査できる token。backend が同じ判定を生成できる形に限る
/// （`crates/ubnfc-rust/src/scan.rs::token_condition` と 1 対 1）。
fn scannable_token(token: &Token) -> bool {
    token_class(token).is_some()
}

/// 領域に置けない観測を 1 つでも持つ式か。
fn observable(cx: &Context<'_>, i: usize) -> bool {
    let expr = &cx.sem.expressions[i];
    // capture site（skip でないもの）は値 span を生むので領域に入れない。
    if expr.capture_site_ids.iter().any(|site| {
        cx.sem
            .captures
            .iter()
            .any(|c| &c.site_id == site && !c.skip)
    }) {
        return true;
    }
    // recovery の適用位置は失敗経路を観測に変える。
    if cx
        .sem
        .recovery
        .iter()
        .any(|r| r.application_sites.contains(&expr.id))
    {
        return true;
    }
    // trivia の挿入境界を持つ式は、走査では再生できない読み飛ばしを伴う。
    cx.sem
        .trivia
        .insertion_plan
        .iter()
        .any(|t| t.boundary.owner_expr_id == expr.id)
}

/// 規則そのものが観測を持つか（mapping 節点・scope 効果・recovery・演算子・左再帰）。
fn observable_rule(cx: &Context<'_>, r: usize) -> bool {
    let rule = &cx.sem.rules[r];
    rule.mapping_type_id.is_some()
        || rule.operator.is_some()
        || rule.eval.is_some()
        || cx.sem.scope.effects.iter().any(|e| e.rule_id == rule.symbol_id)
        || cx.sem.recovery.iter().any(|r| r.rule_id == rule.symbol_id)
        || cx
            .sem
            .normalizations
            .iter()
            .any(|n| n.rule_id == rule.symbol_id)
        // 再帰する規則は走査関数も再帰するので、stack 予算の契約（#35）を動かさない
        // ために対象から外す。
        || cx.rule_sccs.iter().any(|scc| {
            scc.len() > 1 && scc.contains(&r) || scc == &[r] && cx.rule_graph[r].contains(&r)
        })
}

/// 走査へ畳める式の集合。RuleRef は参照先の本体まで含めて判定する（最大不動点）。
fn scannable(cx: &Context<'_>) -> Vec<bool> {
    let n = cx.sem.expressions.len();
    // 楽観的に「畳める」から始めて、畳めない証拠が出た式を落とす（相互参照があっても停まる）。
    let mut ok: Vec<bool> = (0..n).map(|i| local_ok(cx, i)).collect();
    loop {
        let mut changed = false;
        for i in 0..n {
            if !ok[i] {
                continue;
            }
            let deps: Vec<usize> = match &cx.sem.expressions[i].node {
                ExprNode::RuleRef { symbol_id } => vec![cx.body(symbol_id)],
                node => children(node).iter().map(|id| cx.index(id)).collect(),
            };
            if deps.iter().any(|&j| !ok[j]) {
                ok[i] = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    ok
}

/// 子を見ない局所条件。
fn local_ok(cx: &Context<'_>, i: usize) -> bool {
    let expr = &cx.sem.expressions[i];
    if observable(cx, i) {
        return false;
    }
    // 子の mode を切り替える式（MatchOnly / CalleeDefault）は consumed mode の
    // 走査と等価にならない。
    if expr.execution.child_mode != ExecutionChildMode::Inherit {
        return false;
    }
    match &expr.node {
        ExprNode::Seq { .. } | ExprNode::Group { .. } | ExprNode::Optional { .. } => true,
        ExprNode::Choice { selection, .. } => {
            // longest は「全候補を試して最長」なので順序付き走査と等価にならない。
            *selection != ChoiceSelection::Longest
        }
        ExprNode::Repeat {
            child, terminator, ..
        } => {
            // terminator 付きの反復は周回ごとに終端子を先に試すので走査に畳まない。
            // 本体が 0 消費で成功し得る反復は「進まなかったら止める」判定が要るため除く。
            terminator.is_none() && cx.fact(child).minimum() > 0 && !cx.fact(child).nullable()
        }
        ExprNode::Literal {
            value,
            word_boundary,
            case_sensitive,
        } => *case_sensitive && !*word_boundary && !value.is_empty(),
        ExprNode::Any {} => true,
        ExprNode::TokenRef { symbol_id } => scannable_token(&cx.sem.tokens[cx.tokens[symbol_id]]),
        ExprNode::RuleRef { symbol_id } => {
            let r = cx.rules[symbol_id];
            !cx.sem.rules[r].skip && !observable_rule(cx, r)
        }
        _ => false,
    }
}

/// 反復本体の順序付き選択から「1 文字で確定して 1 code point だけ消費する」集合を作る。
/// 候補 k がクラス C_k のとき、使えるのは `C_k \ ∪_{j<k} mayStart(候補 j)`。
/// mayStart は先頭文字の**必要条件の上界**なので、その外の文字では候補 j は
/// 1 文字も消費できずに失敗する（0 消費成功もしない: nullable を除いてある）。
fn bulk_class(cx: &Context<'_>, child: usize) -> (CharSet, bool) {
    let mut inner = child;
    // Group は意味を持たないので剥がす。RuleRef は本体へ降りる。
    loop {
        match &cx.sem.expressions[inner].node {
            ExprNode::Group { child } => inner = cx.index(child),
            ExprNode::RuleRef { symbol_id } => inner = cx.body(symbol_id),
            _ => break,
        }
    }
    let alternatives: Vec<usize> = match &cx.sem.expressions[inner].node {
        ExprNode::Choice { alternatives, .. } => {
            alternatives.iter().map(|id| cx.index(id)).collect()
        }
        _ => vec![inner],
    };
    let mut blocked = CharSet::default();
    let mut class = CharSet::default();
    // 候補がすべて単一 code point のクラス（非 nullable）なら、表の外の文字では
    // 本体が必ず失敗するので、一括走査が止まった位置で反復も終わる。
    let mut complete = true;
    for a in alternatives {
        let mut resolved = a;
        loop {
            match &cx.sem.expressions[resolved].node {
                ExprNode::Group { child } => resolved = cx.index(child),
                ExprNode::RuleRef { symbol_id } => resolved = cx.body(symbol_id),
                _ => break,
            }
        }
        let facts = &cx.facts[a];
        match leaf_class(cx, resolved) {
            Some(set) if !facts.nullable() => {
                class.union(&set.intersect(&blocked.complement()));
            }
            _ => complete = false,
        }
        // 判定不能（eof・nullable・unknown）な候補は以降のすべてを塞ぐ。
        if facts.nullable() || facts.eof {
            blocked = CharSet::all();
        } else {
            blocked.union(&facts.chars);
        }
    }
    (class, complete)
}

/// 走査関数が互いを呼ぶ深さの上限。領域は再帰しない（`observable_rule`）ので有限だが、
/// `A ::= B; B ::= C; …` のような長い連鎖をそのまま関数の入れ子にすると native stack を
/// 深く使う。資源上限の契約（#35）を動かさないために、静的な深さで切る。
const MAX_SCAN_DEPTH: usize = 32;

/// 領域が「1 文字ごとの機構を外す価値がある」形か。反復を含み、かつ走査関数の
/// 入れ子が浅いものだけを対象にする（単発のリテラル 1 つを走査へ畳んでも回数は減らない）。
fn worth_scanning(cx: &Context<'_>, root: usize, region: &BTreeSet<usize>) -> bool {
    region
        .iter()
        .any(|&i| matches!(cx.sem.expressions[i].node, ExprNode::Repeat { .. }))
        && depth(cx, root, &mut BTreeMap::new()) <= MAX_SCAN_DEPTH
}

/// 領域の関数呼出しの入れ子の深さ（規則参照も 1 段と数える）。領域は非巡回。
fn depth(cx: &Context<'_>, i: usize, memo: &mut BTreeMap<usize, usize>) -> usize {
    if let Some(&d) = memo.get(&i) {
        return d;
    }
    // 循環しない前提だが、万一のために訪問中は上限を返して打ち切る。
    memo.insert(i, MAX_SCAN_DEPTH + 1);
    let next: Vec<usize> = match &cx.sem.expressions[i].node {
        ExprNode::RuleRef { symbol_id } => vec![cx.body(symbol_id)],
        node => children(node).iter().map(|id| cx.index(id)).collect(),
    };
    let d = 1 + next
        .into_iter()
        .map(|j| depth(cx, j, memo))
        .max()
        .unwrap_or(0);
    memo.insert(i, d);
    d
}

/// 走査が畳み込む式（RuleRef の先も含む）。
fn region(cx: &Context<'_>, root: usize) -> BTreeSet<usize> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(i) = pending.pop() {
        if !seen.insert(i) {
            continue;
        }
        match &cx.sem.expressions[i].node {
            ExprNode::RuleRef { symbol_id } => pending.push(cx.body(symbol_id)),
            node => pending.extend(children(node).iter().map(|id| cx.index(id))),
        }
    }
    seen
}

pub(super) fn plans(cx: &Context<'_>) -> Vec<ScanPlan> {
    // `@rightAssoc` + trivia の規則は、AST の span を「配下の最初の token 事象」から
    // 取り直す（`significant_end`）。走査は token 事象を 1 件に畳むので、この形の
    // 文法では計画そのものを出さない。
    if cx.sem.rules.iter().any(|r| {
        r.annotations.iter().any(|a| a.name == "rightAssoc")
            && r.annotations
                .iter()
                .any(|a| matches!(a.name.as_str(), "interleave" | "whitespace" | "comment"))
    }) {
        return Vec::new();
    }
    let ok = scannable(cx);
    // 親（局所木の親）が畳めるなら根にしない（最大の領域を選ぶ）。
    let mut parent = BTreeMap::<usize, usize>::new();
    for (i, expr) in cx.sem.expressions.iter().enumerate() {
        for id in children(&expr.node) {
            parent.insert(cx.index(id), i);
        }
    }
    let mut roots = BTreeMap::<usize, Vec<String>>::new();
    let mut covered = BTreeSet::<usize>::new();
    for (i, &scannable) in ok.iter().enumerate() {
        if !scannable || !cx.active.contains(&i) || parent.get(&i).is_some_and(|&p| ok[p]) {
            continue;
        }
        let region = region(cx, i);
        if !worth_scanning(cx, i, &region) {
            continue;
        }
        let mut rule_ids: Vec<String> = region
            .iter()
            .filter_map(|&j| match &cx.sem.expressions[j].node {
                ExprNode::RuleRef { symbol_id } => Some(symbol_id.clone()),
                _ => None,
            })
            .collect();
        rule_ids.sort();
        rule_ids.dedup();
        covered.extend(region);
        roots.insert(i, rule_ids);
    }
    let mut plans = Vec::new();
    for (i, &scannable) in ok.iter().enumerate() {
        let root = roots.contains_key(&i);
        if !root && !(covered.contains(&i) && scannable) {
            continue;
        }
        let class = match &cx.sem.expressions[i].node {
            ExprNode::Repeat { child, .. } => {
                let (set, complete) = bulk_class(cx, cx.index(child));
                (!set.is_empty()).then_some((set, complete))
            }
            _ => None,
        };
        if !root && class.is_none() {
            continue;
        }
        let kind = if class.is_some() {
            ScanPlanKind::CharClassRepetition
        } else {
            ScanPlanKind::TerminalSequence
        };
        let expr = &cx.sem.expressions[i];
        plans.push(ScanPlan {
            class: class.map(|(set, complete)| ScanClass {
                ascii_low: (set.ascii & u128::from(u64::MAX)) as u64,
                ascii_high: (set.ascii >> 64) as u64,
                complete,
                // ASCII は bitset が持つので、区間側は U+0080 以上に切り詰める。
                ranges: set
                    .ranges()
                    .into_iter()
                    .filter(|r| r.end >= 128)
                    .map(|r| CharRange {
                        start: r.start.max(128),
                        end: r.end,
                    })
                    .collect(),
            }),
            expr_id: expr.id.clone(),
            kind,
            proof: Proof {
                kind: ProofKind::TerminalOnlyScan,
                expr_ids: vec![expr.id.clone()],
                explanation: "領域は終端だけで構成され、capture site・scope 効果・recovery・Extern token・trivia 挿入・mapping 節点を 1 つも含まない。よって評価が残す観測は「受理したか」と「どこまで消費したか」だけで、1 本の走査が同じ受理と同じカーソルを返す。失敗した解析は D-023 により診断 session が非最適化経路で解析し直すので、主診断 DAG と expected は構成上変わらない。".into(),
            },
            root,
            rule_ids: roots.get(&i).cloned().unwrap_or_default(),
        });
    }
    plans
}
