use crate::{
    emit::{suffix, Generator},
    EmitError,
};
use std::fmt::Write;
use ubnfc_front::ir::*;

pub(crate) fn generate(g: &Generator<'_>, i: usize, mode: bool) -> Result<String, EmitError> {
    let expr = &g.ir.semantics.expressions[i];
    let suf = suffix(mode);
    let child_mode = match expr.execution.child_mode {
        ExecutionChildMode::Inherit => mode,
        ExecutionChildMode::MatchOnly => true,
        ExecutionChildMode::CalleeDefault => false,
    };
    let analysis =
        g.ir.analysis
            .expressions
            .iter()
            .find(|a| a.expr_id == expr.id)
            .unwrap();
    let memo = g.ir.plan.memo.entries.iter().find(|p| p.expr_id == expr.id);
    let (success, failure) = memo.map_or(
        (analysis.memo_safety.success, analysis.memo_safety.failure),
        |p| (p.cache_success, p.cache_failure),
    );
    let mut s = format!("fn e{i}_{suf}(&mut self,state:State)->Step {{\nif let Some(limit)=self.limit(state) {{return limit;}}\nlet mark=self.mark();\n");
    if success || failure {
        writeln!(s,"let key=Key {{expression:{i},state,matched_mode:{mode},version:self.scope.state_version()}};if let Some(hit)=self.lookup(key) {{return hit;}}").unwrap();
    }
    writeln!(s, "let mut out=self.b{i}_{suf}(state);\nif out.ok {{").unwrap();
    for site in &expr.capture_site_ids {
        let index = g.sites[site.as_str()];
        let capture = &g.ir.semantics.captures[index];
        if capture.skip {
            continue;
        }
        let span = match capture.span_rule {
            CaptureSpanRule::MatchedExtent => "[state.matched,out.state.matched]",
            CaptureSpanRule::SelectedChildExtent => {
                "self.selected_extent(out.events,[state.consumed,out.state.consumed])"
            }
            _ => "[state.consumed,out.state.consumed]",
        };
        writeln!(
            s,
            "out.events=self.event(Event::Capture {{site:{index},span:{span},child:out.events}});"
        )
        .unwrap();
    }
    s.push_str("} else {self.restore(mark);}\n");
    if success || failure {
        writeln!(
            s,
            "if (out.ok && {success}) || (!out.ok && {failure}) {{self.store(key,out,mark);}}"
        )
        .unwrap();
    }
    writeln!(
        s,
        "out\n}}\nfn b{i}_{suf}(&mut self,mut state:State)->Step {{"
    )
    .unwrap();
    if expr.execution.entry_flag == ExecutionEntryFlag::SetFalse {
        s.push_str("state.reset=false;\n");
    }
    let label =
        g.ir.semantics
            .diagnostics
            .expected_display
            .iter()
            .find(|d| d.expr_id == expr.id)
            .map(|d| d.rust.as_str())
            .or(expr.diagnostic_label.as_deref())
            .unwrap_or_else(|| match &expr.node {
                ExprNode::Literal { value, .. } => value,
                ExprNode::Error { message } => message,
                ExprNode::TokenRef { symbol_id } => {
                    g.ir.semantics
                        .symbols
                        .iter()
                        .find(|symbol| symbol.id == *symbol_id)
                        .map_or(symbol_id.as_str(), |symbol| symbol.name.as_str())
                }
                _ => "expression",
            });
    let transaction = matches!(
        expr.execution.transaction,
        ExecutionTransaction::Once | ExecutionTransaction::Occurs
    );
    s.push_str("let mut out=Step::yes(state);\n");
    if transaction {
        s.push_str("out.state=state.begin();\n");
    }
    s.push_str(&g.trivia(&expr.id, BoundaryAt::Entry, None, mode));
    s.push_str(&g.trivia(&expr.id, BoundaryAt::HelperEntry, None, mode));
    match &expr.node {
        ExprNode::Seq { items } => {
            for (index, child) in items.iter().enumerate() {
                writeln!(s,"let child={};out=self.combine(out,child);if !out.ok {{out.state=state;return out;}}",g.call(child,child_mode,"out.state")).unwrap();
                s.push_str(&g.trivia(&expr.id, BoundaryAt::AfterChild, Some(index as u64), mode));
            }
        }
        ExprNode::Choice {
            alternatives,
            selection,
            ..
        } => {
            let plan = g.ir.plan.choices.iter().find(|p| p.expr_id == expr.id);
            let longest = plan.map_or(*selection == ChoiceSelection::Longest, |p| {
                p.strategy == ChoicePlanStrategy::Longest
            });
            let predict = plan.map_or(*selection == ChoiceSelection::Predictive, |p| {
                p.strategy == ChoicePlanStrategy::Predict
            });
            let order = plan
                .map(|p| &p.candidate_order)
                .filter(|o| !o.is_empty())
                .unwrap_or(alternatives);
            s.push_str("out='choice: {let mark=self.mark();let mut diagnostic=Diag(0);let mut best:Option<(Step,[usize;2])>=None;\n");
            if predict && !longest {
                s.push_str(&predict_literals(g, order, child_mode));
            }
            for alt in order {
                writeln!(s,"self.restore(mark);let mut child={};diagnostic=self.diag.join(diagnostic,child.diag);",g.call(alt,child_mode,"out.state.begin()")).unwrap();
                if longest {
                    s.push_str("if child.ok && best.as_ref().is_none_or(|(b,_)|child.state.consumed>b.state.consumed) {child.state=out.state.commit(child.state);let effects=self.save_effects(mark);best=Some((child,effects));}\n");
                } else {
                    s.push_str("if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}\n");
                }
            }
            s.push_str("self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};\n};\n");
        }
        ExprNode::Optional { child } | ExprNode::Repeat { child, .. } => {
            let (min, max, terminator) = match &expr.node {
                ExprNode::Repeat {
                    min,
                    max,
                    terminator,
                    ..
                } => (*min, *max, terminator.as_ref()),
                _ => (0, Some(1), None),
            };
            s.push_str("let mut count=0usize;loop {let mark=self.mark();let before=out.state;\n");
            if let Some(t) = terminator {
                writeln!(
                    s,
                    "let end={};out.diag=self.diag.join(out.diag,end.diag);if end.ok {{",
                    g.call(t, false, "out.state.begin()")
                )
                .unwrap();
                if !mode {
                    s.push_str("out.state=out.state.commit(end.state);out.events=self.join(out.events,end.events);\n");
                } else {
                    s.push_str("self.restore(mark);\n");
                }
                s.push_str("break;}self.restore(mark);\n");
            }
            writeln!(s,"let child={};out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {{self.restore(mark);break;}}\nout.events=self.join(out.events,child.events);count+=1;\nif before.position::<{mode}>()==out.state.position::<{mode}>() {{break;}}",g.call(child,child_mode,"out.state")).unwrap();
            if let Some(max) = max {
                writeln!(s, "if count>={} {{break;}}", max.max(1)).unwrap();
            }
            s.push_str("}\n");
            s.push_str("let _=count;\n");
            let bound = match (min, max) {
                (0, None) => "false".to_owned(),
                (0, Some(max)) => format!("count>{max}"),
                (min, None) => format!("count<{min}"),
                (min, Some(max)) => format!("!({min}..={max}).contains(&count)"),
            };
            writeln!(s,"if {bound} {{out.ok=false;out.state=state;let d=self.diag.fail(state.position::<{mode}>(),{label:?});out.diag=self.diag.join(out.diag,d);}}").unwrap();
        }
        ExprNode::Separated {
            element,
            separator,
            min,
            ..
        } => {
            s.push_str("let mark=self.mark();\n");
            writeln!(s,"let first={};out.diag=self.diag.join(out.diag,first.diag);let mut count=0usize;if first.ok {{out=self.combine(out,first);count+=1;loop {{let pair=self.mark();let before=out;",g.call(element,child_mode,"out.state.begin()")).unwrap();
            writeln!(s,"let sep={};out.diag=self.diag.join(out.diag,sep.diag);if !sep.ok {{self.restore(pair);break;}}let item={};out.diag=self.diag.join(out.diag,item.diag);if !item.ok {{self.restore(pair);break;}}let joined=self.combine(sep,item);out=self.combine(out,joined);count+=1;if before.state.position::<{mode}>()==out.state.position::<{mode}>() {{break;}}}}}} else {{self.restore(mark);}}",g.call(separator,child_mode,"out.state.begin()"),g.call(element,child_mode,"sep.state")).unwrap();
            if *min > 0 {
                writeln!(s, "if count<{min} {{out.ok=false;out.state=state;}}").unwrap();
            } else {
                s.push_str("let _=count;\n");
            }
        }
        ExprNode::Group { child } => {
            writeln!(s, "out={};", g.call(child, child_mode, "out.state")).unwrap();
        }
        ExprNode::RuleRef { symbol_id } => {
            writeln!(
                s,
                "out=self.r{}_{}(out.state);",
                g.rules[symbol_id.as_str()],
                suffix(child_mode)
            )
            .unwrap();
        }
        ExprNode::TokenRef { symbol_id } => {
            writeln!(
                s,
                "out=self.token_{}::<{child_mode}>(out.state,{label:?});",
                g.tokens[symbol_id.as_str()]
            )
            .unwrap();
        }
        ExprNode::Literal {
            value,
            case_sensitive,
            ..
        } => {
            writeln!(
                s,
                "out=self.literal::<{mode}>(out.state,{value:?},{case_sensitive},{label:?});"
            )
            .unwrap();
        }
        ExprNode::Any {} => {
            writeln!(s, "out=self.any::<{mode}>(out.state,{label:?});").unwrap();
        }
        ExprNode::Empty {} => s.push_str("out=self.empty(out.state);\n"),
        ExprNode::Eof {} => {
            writeln!(s, "out=self.eof::<{mode}>(out.state,{label:?});").unwrap();
        }
        ExprNode::Error { message } => {
            writeln!(
                s,
                "out=self.fail(out.state,out.state.position::<{mode}>(),{message:?});"
            )
            .unwrap();
        }
        ExprNode::Lookahead { child } | ExprNode::NegativeLookahead { child } => {
            s.push_str("let mark=self.mark();\n");
            writeln!(
                s,
                "let child={};self.restore(mark);",
                g.call(child, true, "out.state")
            )
            .unwrap();
            if matches!(expr.node, ExprNode::NegativeLookahead { .. }) {
                writeln!(s,"out=if child.ok {{let mut failed=self.fail(state,state.position::<{mode}>(),{label:?});failed.diag=self.diag.join(child.diag,failed.diag);failed}} else {{Step::yes(state.commit(out.state))}};").unwrap();
            } else {
                s.push_str("out.ok=child.ok;out.diag=child.diag;out.events=EventId(0);out.state=if child.ok {state.commit(child.state)} else {state};\n");
            }
        }
    }
    s.push_str("if out.ok {\n");
    s.push_str(&g.trivia(&expr.id, BoundaryAt::Exit, None, mode));
    s.push_str(&g.trivia(&expr.id, BoundaryAt::HelperExit, None, mode));
    s.push_str("}\n");
    if transaction {
        s.push_str("out.state=if out.ok {state.commit(out.state)} else {state};\n");
    }
    s.push_str("out\n}\n");
    Ok(s)
}

pub(crate) fn scope_events(
    g: &Generator<'_>,
    rule: usize,
    when: ScopeEffectEventsWhen,
    events: &str,
    state: &str,
) -> String {
    let mut s = String::new();
    for effect in
        g.ir.semantics
            .scope
            .effects
            .iter()
            .filter(|e| e.rule_id == g.ir.semantics.rules[rule].symbol_id)
    {
        let mode = match effect.mode {
            ScopeEffectMode::Lexical => "lexical",
            ScopeEffectMode::Dynamic => "dynamic",
            ScopeEffectMode::SameRuleText => "sameRuleText",
        };
        for event in effect.events.iter().filter(|e| e.when == when) {
            if effect.mode == ScopeEffectMode::SameRuleText
                && matches!(
                    event.action,
                    ScopeEffectEventsAction::Declare | ScopeEffectEventsAction::Use
                )
            {
                if event.action == ScopeEffectEventsAction::Use {
                    if let Some(site) = &event.capture_site_id {
                        let declarations = effect
                            .events
                            .iter()
                            .filter(|d| {
                                d.action == ScopeEffectEventsAction::Declare && d.name == event.name
                            })
                            .filter_map(|d| d.capture_site_id.as_ref())
                            .map(|d| g.sites[d.as_str()])
                            .collect::<Vec<_>>();
                        writeln!(s,"if let Some(span)=self.scope_capture({events},{}) {{let name=self.text(span);if !{:?}.iter().any(|&site|self.scope_capture({events},site).is_some_and(|s|self.text(s)==name)) {{let offset_cp=self.cp(span[0]);let len_cp=self.cp(span[1])-offset_cp;self.effect(ScanEffect::Diagnostic{{message:format!(\"backreference mismatch: {{name}}\"),offset_cp,len_cp,severity:Severity::Error}});}}}}",g.sites[site.as_str()],declarations).unwrap();
                    }
                }
                continue;
            }
            match event.action {
                ScopeEffectEventsAction::Enter => writeln!(
                    s,
                    "self.effect_at(ScanEffect::Enter,{mode:?},self.cp({state}.consumed),0);"
                )
                .unwrap(),
                ScopeEffectEventsAction::Leave => writeln!(
                    s,
                    "self.effect_at(ScanEffect::Leave,{mode:?},self.cp({state}.consumed),0);"
                )
                .unwrap(),
                ScopeEffectEventsAction::Clear => writeln!(
                    s,
                    "self.effect_at(ScanEffect::Clear,{mode:?},self.cp({state}.consumed),0);"
                )
                .unwrap(),
                ScopeEffectEventsAction::Declare | ScopeEffectEventsAction::Use => {
                    if let Some(site) = &event.capture_site_id {
                        writeln!(s,"if let Some(span)=self.scope_capture({events},{}) {{let name=self.text(span);let offset_cp=self.cp(span[0]);",g.sites[site.as_str()]).unwrap();
                    } else {
                        writeln!(s,"{{let name={:?}.to_owned();let span=[{state}.consumed,{state}.consumed];let offset_cp=self.cp(span[0]);",event.name.as_deref().unwrap_or("")).unwrap();
                    }
                    if event.action == ScopeEffectEventsAction::Declare {
                        writeln!(s,"let len_cp=self.cp(span[1])-offset_cp;self.effect_at(ScanEffect::Declare {{name,offset_cp}},{mode:?},offset_cp,len_cp);}}").unwrap();
                    } else {
                        writeln!(s,"let len_cp=self.cp(span[1])-offset_cp;self.effect_at(ScanEffect::Use {{name,offset_cp,len_cp}},{mode:?},offset_cp,len_cp);}}").unwrap();
                    }
                }
            }
        }
    }
    s
}

// 失敗位置と効果が静的に確定する literal Choice を分岐する。
// 複合候補は診断を失わない保守経路。共通接頭辞は full fallback。
fn predict_literals(g: &Generator<'_>, order: &[ExprId], mode: bool) -> String {
    let mut literals = Vec::new();
    let mut firsts = std::collections::BTreeSet::new();
    for id in order {
        let expr = &g.ir.semantics.expressions[g.expr[id.as_str()]];
        let ExprNode::Literal {
            value,
            case_sensitive: true,
            ..
        } = &expr.node
        else {
            return String::new();
        };
        let Some(first) = value.chars().next() else {
            return String::new();
        };
        if !firsts.insert(first) {
            return String::new();
        }
        if g.ir
            .semantics
            .trivia
            .insertion_plan
            .iter()
            .any(|p| p.boundary.owner_expr_id == *id)
        {
            return String::new();
        }
        let label =
            g.ir.semantics
                .diagnostics
                .expected_display
                .iter()
                .find(|d| d.expr_id == expr.id)
                .map(|d| d.rust.as_str())
                .or(expr.diagnostic_label.as_deref())
                .unwrap_or(value);
        literals.push((id, first, label));
    }
    let mut s=format!("if !out.state.invert {{let predicted=match self.input.cp_at(out.state.begin().position::<{mode}>()).map(|(c,_)|c) {{\n");
    for (i, (_, c, _)) in literals.iter().enumerate() {
        writeln!(s, "Some({c:?})=>{i}usize,").unwrap();
    }
    s.push_str("_=>usize::MAX,};let candidate=match predicted {\n");
    for (i, (id, _, _)) in literals.iter().enumerate() {
        writeln!(s, "{i}=>Some({}),", g.call(id, mode, "out.state.begin()")).unwrap();
    }
    s.push_str("_=>None,};if let Some(mut child)=candidate {if child.ok {\n");
    for (i, (_, _, label)) in literals.iter().enumerate() {
        writeln!(s,"if predicted>{i} {{let d=self.diag.fail(out.state.begin().position::<{mode}>(),{label:?});diagnostic=self.diag.join(diagnostic,d);}}").unwrap();
    }
    s.push_str("child.state=out.state.commit(child.state);child.diag=self.diag.join(diagnostic,child.diag);break 'choice child;}}self.restore(mark); }\n");
    s
}
