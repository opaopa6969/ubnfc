//! Grammar IR semantics の保守的な静的解析。入力を変更せず、式配列の順序を保つ。
//!
//! `mayStart` は必要条件の上界であり、成功判定器ではない。ASCII は内部で
//! 128 bitset、非 ASCII は区間集合。出力は IR v1 の ranges に正規化する。
//! Predict の表は candidateOrder と各候補の mayStart から backend が構成する。
//! fullChoice は候補なし・候補全失敗の両方で完全選択を再実行する契約。
mod charset;
mod checks;
mod choices;
mod cursor;
mod facts;
mod follow;
mod graph;
mod operators;
mod planning;
mod reachability;
mod reentry;
mod regex;
mod scan;
pub use reachability::ChoiceIssue;
pub(crate) use regex::attach as attach_regex_plans;

use crate::ir::*;
use facts::Facts;
use std::collections::{BTreeMap, BTreeSet};

/// analysis 単独で受け取った不正 IR も panic せず明示拒否する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisError {
    InvalidSemantics {
        location: String,
        message: String,
    },
    DirectLeftRecursion {
        rule_id: String,
    },
    IndirectLeftRecursion {
        rule_ids: Vec<String>,
    },
    CaptureCardinality {
        site_id: String,
        declared: Cardinality,
        inferred: Cardinality,
    },
    InvalidOperator {
        rule_id: String,
        message: String,
    },
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AnalysisError {}

struct Context<'a> {
    pub sem: &'a Semantics,
    pub exprs: BTreeMap<ExprId, usize>,
    pub rules: BTreeMap<String, usize>,
    pub tokens: BTreeMap<String, usize>,
    pub deps: Vec<Vec<usize>>,
    pub facts: Vec<Facts>,
    pub active: BTreeSet<usize>,
    pub rule_graph: Vec<Vec<usize>>,
    pub rule_sccs: Vec<Vec<usize>>,
}

pub(super) fn children(node: &ExprNode) -> Vec<&ExprId> {
    match node {
        ExprNode::Seq { items } => items.iter().collect(),
        ExprNode::Choice { alternatives, .. } => alternatives.iter().collect(),
        ExprNode::Optional { child }
        | ExprNode::Group { child }
        | ExprNode::Lookahead { child }
        | ExprNode::NegativeLookahead { child } => vec![child],
        ExprNode::Repeat {
            child, terminator, ..
        } => terminator.iter().chain([child]).collect(),
        ExprNode::Separated {
            element, separator, ..
        } => vec![element, separator],
        _ => Vec::new(),
    }
}

impl Context<'_> {
    fn index(&self, id: &ExprId) -> usize {
        self.exprs[id]
    }
    fn fact(&self, id: &ExprId) -> &Facts {
        &self.facts[self.index(id)]
    }
    fn body(&self, rule: &str) -> usize {
        self.index(&self.sem.rules[self.rules[rule]].body)
    }
    fn local_tree(&self, root: usize) -> BTreeSet<usize> {
        let mut seen = BTreeSet::new();
        let mut pending = vec![root];
        while let Some(i) = pending.pop() {
            if seen.insert(i) {
                pending.extend(
                    children(&self.sem.expressions[i].node)
                        .iter()
                        .map(|id| self.index(id)),
                );
            }
        }
        seen
    }
    fn reachable(&self, root: usize) -> BTreeSet<usize> {
        let mut seen = BTreeSet::new();
        let mut pending = vec![root];
        while let Some(i) = pending.pop() {
            if seen.insert(i) {
                pending.extend(&self.deps[i]);
            }
        }
        seen
    }
}

/// lowering 済みの意味論を解析する。直接左再帰は lowering の正規化記録を参照し、
/// 書換えが残っていれば拒否する。正規化前の保存式は出力するが実行グラフから除く。
pub fn analyze(semantics: &Semantics) -> Result<(Analysis, Plan), Vec<AnalysisError>> {
    analyze_with_choice_issues(semantics).map(|(analysis, plan, _)| (analysis, plan))
}

/// `analyze` に加えて、順序付き選択の到達不能候補（D-037）の警告を返す。
/// 警告は IR に載らない。受理・AST・診断は変えない。
pub fn analyze_with_choice_issues(
    semantics: &Semantics,
) -> Result<(Analysis, Plan, Vec<ChoiceIssue>), Vec<AnalysisError>> {
    checks::validate(semantics)?;
    let mut cx = Context {
        sem: semantics,
        exprs: semantics
            .expressions
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.clone(), i))
            .collect(),
        rules: semantics
            .rules
            .iter()
            .enumerate()
            .map(|(i, r)| (r.symbol_id.clone(), i))
            .collect(),
        tokens: semantics
            .tokens
            .iter()
            .enumerate()
            .map(|(i, t)| (t.symbol_id.clone(), i))
            .collect(),
        deps: Vec::new(),
        facts: vec![Facts::bottom(); semantics.expressions.len()],
        active: BTreeSet::new(),
        rule_graph: vec![Vec::new(); semantics.rules.len()],
        rule_sccs: Vec::new(),
    };
    cx.deps = semantics
        .expressions
        .iter()
        .map(|e| {
            let mut deps: Vec<_> = children(&e.node).iter().map(|id| cx.index(id)).collect();
            if let ExprNode::RuleRef { symbol_id } = &e.node {
                deps.push(cx.body(symbol_id));
            }
            deps
        })
        .collect();
    for (r, rule) in semantics.rules.iter().enumerate() {
        let tree = cx.local_tree(cx.index(&rule.body));
        for &e in &tree {
            if let ExprNode::RuleRef { symbol_id } = &semantics.expressions[e].node {
                cx.rule_graph[r].push(cx.rules[symbol_id]);
            }
        }
        cx.active.extend(tree);
        cx.rule_graph[r].sort_unstable();
        cx.rule_graph[r].dedup();
    }
    cx.rule_sccs = graph::scc(&cx.rule_graph);
    cx.rule_sccs.sort_by_key(|s| s[0]);
    facts::solve(&mut cx);
    let mut errors = checks::left_recursion(&cx);
    errors.extend(checks::cardinality(&cx));
    let operators = operators::plans(&cx, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }
    let (choices, prefix_tries) = choices::plans(&cx);
    // 左再帰・cardinality・operator の検査を通った後でだけ到達性を論じる。
    let choice_issues = reachability::check(&cx);
    let (memo, safety, effects, omitted) = planning::memo(&cx, &choices);
    let follow = follow::compute(&cx);
    let sccs: Vec<_> = cx
        .rule_sccs
        .iter()
        .enumerate()
        .map(|(i, group)| Scc {
            id: format!("scc:{i}"),
            recursive: group.len() > 1 || cx.rule_graph[group[0]].contains(&group[0]),
            rule_ids: group
                .iter()
                .map(|&r| semantics.rules[r].symbol_id.clone())
                .collect(),
            memo_policy: if group.iter().any(|&r| omitted.contains(&r)) {
                SccMemoPolicy::ProvedOmissions
            } else {
                SccMemoPolicy::AllEligible
            },
        })
        .collect();
    let analysis = Analysis {
        expressions: semantics
            .expressions
            .iter()
            .enumerate()
            .map(|(i, e)| ExprAnalysis {
                expr_id: e.id.clone(),
                nullable: cx.facts[i].nullable(),
                min_consume: cx.facts[i].minimum(),
                may_start: cx.facts[i].may_start(),
                cursor_transition: cursor::transition(&cx, i),
                prefix_trie_id: prefix_tries
                    .iter()
                    .find(|t| t.id == format!("trie:{}", e.id))
                    .map(|t| t.id.clone()),
                memo_safety: safety[i].clone(),
                effects: effects[i].clone(),
            })
            .collect(),
        rules: semantics
            .rules
            .iter()
            .map(|r| RuleAnalysis {
                rule_id: r.symbol_id.clone(),
                scc_id: sccs
                    .iter()
                    .find(|s| s.rule_ids.contains(&r.symbol_id))
                    .unwrap()
                    .id
                    .clone(),
                left_recursion: if semantics
                    .normalizations
                    .iter()
                    .any(|n| n.rule_id == r.symbol_id)
                {
                    RuleAnalysisLeftRecursion::DirectRewritten
                } else {
                    RuleAnalysisLeftRecursion::None
                },
                progress_proven: cx.fact(&r.body).minimum() > 0,
                follow_sync_patterns: follow.get(&r.symbol_id).cloned(),
            })
            .collect(),
        sccs,
        prefix_tries,
    };
    let plan = Plan {
        profile: if choices
            .iter()
            .any(|p| p.strategy == ChoicePlanStrategy::Predict)
            || !omitted.is_empty()
            || operators
                .iter()
                .any(|p| p.strategy == OperatorPlanStrategy::PrecedenceClimbing)
        {
            PlanProfile::Optimized
        } else {
            PlanProfile::Conservative
        },
        choices,
        operators,
        memo,
        scans: scan::plans(&cx),
        lexical: semantics.tokens.iter().map(regex::plan).collect(),
        trivia_cache: TriviaCache {
            enabled: false,
            key: PlanTriviaCacheKey,
        },
    };
    Ok((analysis, plan, choice_issues))
}

pub use planning::{scope_schedule, ScopeStep};

/// analyze の結果を GrammarIr に取り込む際、既存 requiredFeatures に追加する機能。
/// 意味論側の機能宣言は削除しない。順序は feature 名の辞書順。
pub fn plan_required_features(plan: &Plan) -> Vec<RequiredFeature> {
    let mut features = Vec::new();
    if plan
        .choices
        .iter()
        .any(|p| p.strategy == ChoicePlanStrategy::Longest)
    {
        features.push(RequiredFeature::LongestChoice);
    }
    if !plan.memo.entries.is_empty() {
        features.push(RequiredFeature::MemoReplay);
    }
    if !plan.operators.is_empty() {
        features.push(RequiredFeature::Operators);
    }
    if plan
        .choices
        .iter()
        .any(|p| p.strategy == ChoicePlanStrategy::Ordered)
    {
        features.push(RequiredFeature::OrderedChoice);
    }
    if plan
        .choices
        .iter()
        .any(|p| p.strategy == ChoicePlanStrategy::Predict)
    {
        features.push(RequiredFeature::PredictiveChoice);
    }
    features
}
