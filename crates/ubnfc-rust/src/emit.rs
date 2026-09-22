use crate::{EmitError, EmitOptions, GeneratedCrate};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};
use ubnfc_front::ir::*;

pub(crate) struct Generator<'a> {
    pub ir: &'a GrammarIr,
    pub display: crate::diagnostics::Plan,
    pub expr: BTreeMap<&'a str, usize>,
    pub rules: BTreeMap<&'a str, usize>,
    pub tokens: BTreeMap<&'a str, usize>,
    pub sites: BTreeMap<&'a str, usize>,
    pub modes: BTreeSet<(usize, bool)>,
    /// rule 本体の expression index。memo・診断 frame は本体と共有 expression にだけ置く（expressions.rs 参照）。
    pub bodies: BTreeSet<usize>,
    /// 複数の親（expression の子、または rule 本体）から参照される expression。同一 rule の評価中に
    /// 同じ状態で再訪され得るので、plan が許せば memo を置く。
    pub shared: BTreeSet<usize>,
    /// 選択の候補除外（§3.4）に使う先頭文字集合と再生可能性（predict.rs）。
    pub prediction: crate::predict::Prediction,
    /// 生成する失敗再生関数 `guard_e{i}_{mode}` の集合。
    pub guards: BTreeSet<(usize, bool)>,
}
impl<'a> Generator<'a> {
    pub fn new(ir: &'a GrammarIr) -> Self {
        let mut g = Self {
            ir,
            display: crate::diagnostics::Plan::new(ir),
            expr: ir
                .semantics
                .expressions
                .iter()
                .enumerate()
                .map(|(i, e)| (e.id.as_str(), i))
                .collect(),
            rules: ir
                .semantics
                .rules
                .iter()
                .enumerate()
                .map(|(i, r)| (r.symbol_id.as_str(), i))
                .collect(),
            tokens: ir
                .semantics
                .tokens
                .iter()
                .enumerate()
                .map(|(i, t)| (t.symbol_id.as_str(), i))
                .collect(),
            sites: ir
                .semantics
                .captures
                .iter()
                .enumerate()
                .map(|(i, c)| (c.site_id.as_str(), i))
                .collect(),
            modes: BTreeSet::new(),
            bodies: BTreeSet::new(),
            shared: BTreeSet::new(),
            prediction: crate::predict::Prediction {
                starts: Vec::new(),
                templatable: Vec::new(),
                sizes: Vec::new(),
            },
            guards: BTreeSet::new(),
        };
        g.bodies = ir
            .semantics
            .rules
            .iter()
            .map(|r| g.expr[r.body.as_str()])
            .collect();
        let mut incoming = vec![0usize; ir.semantics.expressions.len()];
        for rule in &ir.semantics.rules {
            incoming[g.expr[rule.body.as_str()]] += 1;
        }
        for expr in &ir.semantics.expressions {
            for child in expr.node.children() {
                incoming[g.expr[child.as_str()]] += 1;
            }
        }
        g.shared = incoming
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 1)
            .map(|(i, _)| i)
            .collect();
        for rule in &ir.semantics.rules {
            g.visit(g.expr[rule.body.as_str()], false);
        }
        g.prediction = crate::predict::Prediction::new(&g);
        let mut pending = Vec::new();
        for &(i, mode) in &g.modes {
            let expr = &ir.semantics.expressions[i];
            if let ExprNode::Choice {
                alternatives,
                selection,
                ..
            } = &expr.node
            {
                let child_mode = match expr.execution.child_mode {
                    ExecutionChildMode::Inherit => mode,
                    ExecutionChildMode::MatchOnly => true,
                    ExecutionChildMode::CalleeDefault => false,
                };
                let plan = ir.plan.choices.iter().find(|p| p.expr_id == expr.id);
                let longest = plan.map_or(*selection == ChoiceSelection::Longest, |p| {
                    p.strategy == ChoicePlanStrategy::Longest
                });
                let order = plan
                    .map(|p| &p.candidate_order)
                    .filter(|o| !o.is_empty())
                    .unwrap_or(alternatives);
                let (guarded, _) = crate::predict::guarded_alternatives(&g, order, longest);
                pending.extend(guarded.iter().map(|x| (x.expr, child_mode)));
                // 因数分解した群は共有先頭要素に対して候補除外を行う（factor.rs）。
                if !longest {
                    if let Some(plan) = crate::factor::plan(&g, order, &guarded) {
                        for &(start, _) in &plan.groups {
                            if guarded.iter().any(|x| x.index == start) {
                                if let ExprNode::Seq { items } =
                                    &ir.semantics.expressions[g.expr[order[start].as_str()]].node
                                {
                                    let elem = g.expr[items[0].as_str()];
                                    if g.prediction.templatable[elem] {
                                        pending.push((elem, child_mode));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        g.guards = pending.into_iter().collect();
        g
    }
    fn visit(&mut self, i: usize, mode: bool) {
        let mut pending = vec![(i, mode)];
        while let Some((i, mode)) = pending.pop() {
            if !self.modes.insert((i, mode)) {
                continue;
            }
            let expr = &self.ir.semantics.expressions[i];
            let mode = match expr.execution.child_mode {
                ExecutionChildMode::Inherit => mode,
                ExecutionChildMode::MatchOnly => true,
                ExecutionChildMode::CalleeDefault => false,
            };
            let mode = mode
                || matches!(
                    expr.node,
                    ExprNode::Lookahead { .. } | ExprNode::NegativeLookahead { .. }
                );
            for child in expr.node.children() {
                let m = if matches!(&expr.node,ExprNode::Repeat{terminator:Some(t),..} if t==child)
                {
                    false
                } else {
                    mode
                };
                pending.push((self.expr[child.as_str()], m));
            }
            if let ExprNode::RuleRef { symbol_id } = &expr.node {
                let r = &self.ir.semantics.rules[self.rules[symbol_id.as_str()]];
                pending.push((self.expr[r.body.as_str()], mode));
            }
        }
    }

    pub fn call(&self, id: &ExprId, mode: bool, state: &str) -> String {
        format!("self.e{}_{}({state})", self.expr[id.as_str()], suffix(mode))
    }
    pub fn trivia(&self, id: &ExprId, at: BoundaryAt, index: Option<u64>, mode: bool) -> String {
        self.trivia_with(id, at, index, mode, "out")
    }
    /// 境界 `at` の trivia を Step 変数 `var` に対して読み飛ばす文。
    pub fn trivia_with(
        &self,
        id: &ExprId,
        at: BoundaryAt,
        index: Option<u64>,
        mode: bool,
        var: &str,
    ) -> String {
        let mut s = String::new();
        for insertion in &self.ir.semantics.trivia.insertion_plan {
            if insertion.boundary.owner_expr_id == *id
                && insertion.boundary.at == at
                && insertion.boundary.child_index == index
            {
                let p = self
                    .ir
                    .semantics
                    .trivia
                    .policies
                    .iter()
                    .position(|p| p.id == insertion.policy_id)
                    .unwrap();
                writeln!(
                    s,
                    "let trivia=self.trivia_{p}::<{mode}>({var}.state);{var}=self.combine({var},trivia);"
                )
                .unwrap();
            }
        }
        s
    }
}
pub(crate) fn suffix(mode: bool) -> &'static str {
    if mode {
        "m"
    } else {
        "c"
    }
}
pub(crate) fn generate(ir: &GrammarIr, options: &EmitOptions) -> Result<GeneratedCrate, EmitError> {
    let g = Generator::new(ir);
    let mut files = BTreeMap::new();
    let mut root=String::from("// Generated by ubnfc-rust; deterministic Grammar IR v1.\n#![forbid(unsafe_code)]\npub mod api;\npub mod ast;\npub mod metadata;\npub mod evaluator;\npub mod rt;\nmod parser;\n#[allow(dead_code)]\nmod runtime;\n#[allow(unused_imports)]\npub use api::*;\n#[allow(unused_imports)]\npub use ast::Ast;\n#[allow(unused_imports)]\npub use parser::{parse,parse_with_options,parse_with_scanner,parse_entry_with_options};\n");
    if let Some(documentation) = &options.documentation {
        for line in documentation.lines() {
            writeln!(root, "// {line}").unwrap();
        }
    }
    files.insert("mod.rs".into(), root);
    files.insert("api.rs".into(), include_str!("../templates/api.rs").into());
    files.insert("ast.rs".into(), crate::ast::generate(ir));
    files.insert("metadata.rs".into(), crate::metadata::generate(ir));
    files.insert("evaluator.rs".into(), crate::evaluator::generate(ir)?);
    let mut runtime = include_str!("../templates/runtime.rs").to_owned();
    runtime.push_str(&crate::mapping::generate(&g)?);
    files.insert("runtime.rs".into(), runtime);
    files.insert(
        "rt/mod.rs".into(),
        "pub mod cursor;\npub mod input;\npub mod memo;\npub mod diag;\npub mod scope;\n".into(),
    );
    for (name, text) in [
        ("cursor", include_str!("../templates/cursor.rs")),
        ("input", include_str!("../templates/input.rs")),
        ("memo", include_str!("../templates/memo.rs")),
        ("diag", include_str!("../templates/diag.rs")),
        ("scope", include_str!("../templates/scope.rs")),
    ] {
        files.insert(format!("rt/{name}.rs"), text.into());
    }
    let mut manifest = crate::driver::MANIFEST.to_owned();
    if options.emit_driver {
        manifest.push_str(crate::driver::BIN);
        files.insert("src/bin/driver.rs".into(), crate::driver::generate(ir));
        files.insert(
            "src/bin/driver_json.rs".into(),
            include_str!("../templates/driver_json.rs").into(),
        );
    }
    files.insert("Cargo.toml".into(), manifest);
    let mut p=String::from("//! IR の expression/rule ごとの静的関数。c=consumed, m=matchOnly。\n#![allow(dead_code, unused_mut)]\nuse super::api::*;\nuse super::runtime::*;\n#[allow(unused_imports)]\nuse super::rt::{diag::Diag,memo::Key};\npub fn parse(text:&str)->ParseResult {parse_with_options(text,ParseOptions::default())}\npub fn parse_with_options(text:&str,options:ParseOptions)->ParseResult {parse_with_scanner(text,options,&mut RejectExtern)}\npub fn parse_with_scanner(text:&str,options:ParseOptions,scanner:&mut dyn TokenScanner)->ParseResult {\nlet mut parser=Session::new(text,options,scanner);\n");
    writeln!(
        p,
        "let step=parser.r{}_c(State::default());parser.finish(step)\n}}",
        g.rules[ir.entry.rule_id.as_str()]
    )
    .unwrap();
    p.push_str("pub fn parse_entry_with_options(grammar:&str,entry:Option<&str>,text:&str,options:ParseOptions)->Result<ParseResult,&'static str> {\nlet mut scanner=RejectExtern;let mut parser=Session::new(text,options,&mut scanner);let step=match (grammar,entry) {\n");
    for unit in &ir.grammar_units {
        let roots = ir
            .semantics
            .rules
            .iter()
            .filter(|rule| {
                rule.annotations.iter().any(|a| a.name == "root")
                    && ir
                        .semantics
                        .symbols
                        .iter()
                        .any(|s| s.id == rule.symbol_id && s.grammar_id == unit.id)
            })
            .collect::<Vec<_>>();
        let root = match roots.as_slice() {
            [rule] => Some(rule.symbol_id.as_str()),
            [] if unit.id == ir.entry.grammar_id => Some(ir.entry.rule_id.as_str()),
            _ => None,
        };
        if let Some(root) = root {
            writeln!(
                p,
                "({:?},None)=>parser.r{}_c(State::default()),",
                unit.name, g.rules[root]
            )
            .unwrap();
        }
    }
    for (i, rule) in ir.semantics.rules.iter().enumerate() {
        let symbol = ir
            .semantics
            .symbols
            .iter()
            .find(|s| s.id == rule.symbol_id)
            .expect("rule symbol");
        let unit = ir
            .grammar_units
            .iter()
            .find(|g| g.id == symbol.grammar_id)
            .expect("rule grammar");
        writeln!(
            p,
            "({:?},Some({:?}))=>parser.r{i}_c(State::default()),",
            unit.name, symbol.name
        )
        .unwrap();
    }
    p.push_str("_=>return Err(\"unsupported grammar or entry\"),};Ok(parser.finish(step))}\nimpl Session<'_> {\n");
    for (i, rule) in ir.semantics.rules.iter().enumerate() {
        for mode in [false, true] {
            if !g.modes.contains(&(g.expr[rule.body.as_str()], mode)) {
                continue;
            }
            writeln!(p,"fn r{i}_{}(&mut self,state:State)->Step {{ if let Some(limit)=self.enter_rule(state) {{return limit;}}self.statistics.rule_evaluations+=1;let mark=self.mark();",suffix(mode)).unwrap();
            p.push_str(&crate::expressions::scope_events(
                &g,
                i,
                ScopeEffectEventsWhen::Entry,
                "EventId(0)",
                "state",
            ));
            writeln!(
                p,
                "let mut out={};\nif out.ok {{",
                g.call(&rule.body, mode, "state")
            )
            .unwrap();
            for when in [
                ScopeEffectEventsWhen::SuccessBeforeLeave,
                ScopeEffectEventsWhen::SuccessAfterLeave,
                ScopeEffectEventsWhen::AfterDeclarations,
            ] {
                p.push_str(&crate::expressions::scope_events(
                    &g,
                    i,
                    when,
                    "out.events",
                    "out.state",
                ));
            }
            if rule.annotations.iter().any(|a| a.name == "rightAssoc")
                && rule
                    .annotations
                    .iter()
                    .any(|a| matches!(a.name.as_str(), "interleave" | "whitespace" | "comment"))
            {
                p.push_str("let end=if self.depth>1 {self.significant_end(out.events).unwrap_or(out.state.consumed)} else {out.state.consumed};\n");
            } else {
                p.push_str("let end=out.state.consumed;\n");
            }
            writeln!(p,"out.events=self.event(Event::Rule {{rule:{i},span:[state.consumed,end],child:out.events}});\n}} else {{ self.restore(mark); }}").unwrap();
            writeln!(
                p,
                "if !out.ok {{self.display.failures(state.consumed.max(state.matched),&{:?});}}",
                g.display.rule(rule.symbol_id.as_str())
            )
            .unwrap();
            if let Some(recovery) = ir
                .semantics
                .recovery
                .iter()
                .find(|r| r.rule_id == rule.symbol_id)
            {
                let mode_name = match recovery.mode {
                    RecoveryMode::Sync => "SYNC",
                    RecoveryMode::Auto => "AUTO",
                    RecoveryMode::Skip => "SKIP",
                };
                // recovery は葉を通らずに位置を進めるので、rule の出口で到達位置を記録する。
                writeln!(
                    p,
                    "if !out.ok {{out=self.recover::<{mode}>(state,out,{i},{mode_name:?},&{:?});}}self.display.reach(out.state.consumed.max(out.state.matched));",
                    recovery.resolved_sync_patterns
                )
                .unwrap();
            }
            writeln!(
                p,
                "self.depth-=1;out.diag=self.diag.rule({i},out.diag);out\n}}"
            )
            .unwrap();
        }
    }
    for &(i, mode) in &g.modes {
        p.push_str(&crate::expressions::generate(&g, i, mode)?);
    }
    for &(i, mode) in &g.guards {
        p.push_str(&crate::predict::guard_function(&g, i, mode));
    }
    p.push_str(&crate::tokens::generate(&g)?);
    p.push_str("}\n");
    files.insert("parser.rs".into(), p);
    for source in files.values_mut() {
        *source = source
            .replace("} if", "}\nif")
            .replace("}if", "}\nif")
            .replace("if out.ok {\n}\n", "");
    }
    Ok(GeneratedCrate { files })
}
