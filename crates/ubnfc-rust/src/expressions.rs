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
    // memo は plan が許す expression のうち、rule 本体と共有 expression（複数の親を持つ）にだけ置く
    // （packrat の rule 単位 memo）。単一の親しか持たない expression は、その親の 1 回の評価で
    // 高々 1 回、しかも互いに異なる位置でしか実行されず、同じ状態での再訪は最寄りの memo
    // （rule 本体か共有 expression）が吸収する。本体が memo されない rule（Extern 閉包）では
    // plan どおり内側の expression を memo する。
    // stack 検査（limit）は rule 入口で行う。局所診断 frame と mark は memo の保存に必要な
    // 箇所にだけ置き、失敗時の rollback は choice / repeat / optional / separated /
    // lookahead / rule の各構造が自前で行う。
    let (success, failure) = cacheable(g, i);
    let rule_body = g.expr[g.ir.semantics.rules[g.rules[expr.rule_id.as_str()]].body.as_str()];
    let (success, failure) = if g.bodies.contains(&i)
        || g.shared.contains(&i)
        || cacheable(g, rule_body) == (false, false)
    {
        (success, failure)
    } else {
        (false, false)
    };
    let memoized = success || failure;
    let mut s = format!("fn e{i}_{suf}(&mut self,state:State)->Step {{\n");
    if memoized {
        s.push_str("let mark=self.mark();\n");
        // scope を読み書きしない（effects が空の）expression は stateVersion に依存しない。
        let version = if analysis.effects.is_empty() {
            "0".to_owned()
        } else {
            "self.scope.state_version()".to_owned()
        };
        writeln!(s,"let key=Key {{expression:{i},state,matched_mode:{mode},version:{version}}};if let Some(hit)=self.lookup(key) {{return hit;}}").unwrap();
        writeln!(s, "self.display.enter(state.consumed.max(state.matched));let mut out=self.b{i}_{suf}(state);self.display.reach(out.state.consumed.max(out.state.matched));\nif out.ok {{").unwrap();
    } else {
        writeln!(s, "self.display.reach(state.consumed.max(state.matched));let mut out=self.b{i}_{suf}(state);self.display.reach(out.state.consumed.max(out.state.matched));\nif out.ok {{").unwrap();
    }
    if matches!(
        expr.node,
        ExprNode::Literal { .. } | ExprNode::TokenRef { .. } | ExprNode::Any {}
    ) {
        let rule = g.rules[expr.rule_id.as_str()];
        writeln!(s, "if let Event::Token{{span,text_span,content_span,..}}=self.arena[out.events.0] {{out.events=self.event(Event::Token{{expr:{i},rule:{rule},span,text_span,content_span}});}}").unwrap();
    }
    if matches!(
        expr.node,
        ExprNode::Seq { .. }
            | ExprNode::Group { .. }
            | ExprNode::Optional { .. }
            | ExprNode::Repeat { .. }
    ) {
        if let Some(count) = semantic_shape(g, &expr.id, &mut std::collections::BTreeSet::new()) {
            // The lowered separator/element pair is plumbing, not a source
            // group whose delimiters belong to a scalar text value.
            let wrap =
                count != Cardinality::List && !expr.id.as_str().ends_with("/lower/separated/2");
            writeln!(s,"out.events=self.event(Event::Values{{span:[state.consumed,out.state.consumed],child:out.events,wrap:{wrap}}});").unwrap();
        }
    }
    if !matches!(
        expr.node,
        ExprNode::Optional { .. } | ExprNode::Repeat { .. }
    ) {
        s.push_str(&capture_events(g, expr, "state", "out"));
    }
    writeln!(s,"}} else {{{}self.display.failures(state.consumed.max(state.matched),&{:?});}}",if memoized {"self.restore(mark);"} else {""},g.display.expression(expr.id.as_str())).unwrap();
    if memoized {
        writeln!(
            s,
            "if (out.ok && {success}) || (!out.ok && {failure}) {{self.store(key,out,mark);}}self.display.leave();"
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
            s.push_str("out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;\n");
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
            writeln!(s,"let mut child={};out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {{self.restore(mark);break;}}",g.call(child,child_mode,"out.state")).unwrap();
            s.push_str(&capture_events(g, expr, "before", "child"));
            writeln!(s,"out.events=self.join(out.events,child.events);count+=1;\nif before.position::<{mode}>()==out.state.position::<{mode}>() {{break;}}").unwrap();
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
                "out=self.token_{}::<{child_mode}>(out.state,{label:?},{:?});",
                g.tokens[symbol_id.as_str()],
                g.display.token_label(expr.id.as_str())
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
            let not = matches!(expr.node, ExprNode::NegativeLookahead { .. });
            writeln!(s,"let mark=self.mark();self.display.{}(state.consumed.max(state.matched));",if not {"enter_guard"} else {"enter"}).unwrap();
            writeln!(
                s,
                "let child={};self.restore(mark);",
                g.call(child, true, "out.state")
            )
            .unwrap();
            if matches!(expr.node, ExprNode::NegativeLookahead { .. }) {
                writeln!(s,"out=if child.ok {{self.display.leave_guard();let mut failed=self.fail(state,state.position::<{mode}>(),{label:?});failed.diag=self.diag.join(child.diag,failed.diag);failed}} else {{self.display.discard();Step::yes(state.commit(out.state))}};").unwrap();
            } else {
                s.push_str("self.display.leave();out.ok=child.ok;out.diag=child.diag;out.events=EventId(0);out.state=if child.ok {state.commit(child.state)} else {state};\n");
            }
        }
    }
    if matches!(
        expr.node,
        ExprNode::Literal { .. } | ExprNode::Any {} | ExprNode::Eof {} | ExprNode::Error { .. }
    ) {
        let display = g.display.token_label(expr.id.as_str());
        writeln!(
            s,
            "if !out.ok {{self.display.fail(state.consumed.max(state.matched),{display:?});}}"
        )
        .unwrap();
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

/// plan.memo（無ければ analysis.memoSafety）が許す成功・失敗の memo。
fn cacheable(g: &Generator<'_>, i: usize) -> (bool, bool) {
    let expr = &g.ir.semantics.expressions[i];
    let analysis =
        g.ir.analysis
            .expressions
            .iter()
            .find(|a| a.expr_id == expr.id)
            .unwrap();
    g.ir.plan
        .memo
        .entries
        .iter()
        .find(|p| p.expr_id == expr.id)
        .map_or(
            (analysis.memo_safety.success, analysis.memo_safety.failure),
            |p| (p.cache_success, p.cache_failure),
        )
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
        let selected: Vec<_> = effect
            .events
            .iter()
            .filter(|e| {
                e.when == when
                    && !(effect.mode == ScopeEffectMode::SameRuleText
                        && e.action == ScopeEffectEventsAction::Declare)
            })
            .collect();
        let sites: Vec<_> = selected
            .iter()
            .filter_map(|e| e.capture_site_id.as_ref())
            .map(|id| g.sites[id.as_str()])
            .collect();
        // One traversal for all sites preserves completion order across repeats,
        // alternatives and nested captures. Action order at the same site is IR order.
        if !sites.is_empty() {
            writeln!(s, "for (site,span) in self.scope_captures({events},&{sites:?}) {{ let name=self.text(span);let offset_cp=self.cp(span[0]);let len_cp=self.cp(span[1])-offset_cp;").unwrap();
            for event in selected.iter().filter(|e| e.capture_site_id.is_some()) {
                let site = g.sites[event.capture_site_id.as_ref().unwrap().as_str()];
                if effect.mode != ScopeEffectMode::SameRuleText {
                    writeln!(s, "if site=={site} {{").unwrap();
                }
                if effect.mode == ScopeEffectMode::SameRuleText {
                    if event.action == ScopeEffectEventsAction::Use {
                        let targets: Vec<_> =
                            g.ir.semantics
                                .captures
                                .iter()
                                .filter(|c| {
                                    c.rule_id == effect.rule_id
                                        && Some(c.name.as_str()) == event.name.as_deref()
                                })
                                .map(|c| g.sites[c.site_id.as_str()])
                                .chain(
                                    effect
                                        .events
                                        .iter()
                                        .filter(|e| {
                                            e.action == ScopeEffectEventsAction::Declare
                                                && e.name == event.name
                                        })
                                        .filter_map(|e| e.capture_site_id.as_ref())
                                        .map(|id| g.sites[id.as_str()]),
                                )
                                .collect();
                        writeln!(s, "if site=={site} {{if let Some((_,target))=self.scope_captures({events},&{targets:?}).first() {{let expected=self.text(*target);if expected!=name {{self.effect(ScanEffect::Diagnostic{{message:format!(\"back-reference mismatch: expected '{{expected}}' but got '{{name}}'\"),offset_cp,len_cp,severity:Severity::Error}});}}}}}}").unwrap();
                    }
                } else {
                    match event.action {
                        ScopeEffectEventsAction::Declare => writeln!(s,"self.effect_at(ScanEffect::Declare{{name:name.clone(),offset_cp}},{mode:?},offset_cp,len_cp);").unwrap(),
                        ScopeEffectEventsAction::Use => writeln!(s,"self.effect_at(ScanEffect::Use{{name:name.clone(),offset_cp,len_cp}},{mode:?},offset_cp,len_cp);").unwrap(),
                        _ => {}
                    }
                }
                if effect.mode != ScopeEffectMode::SameRuleText {
                    s.push_str("}\n");
                }
            }
            s.push_str("let _=(&name,offset_cp,len_cp); }\n");
        }
        for event in selected.iter().filter(|e| e.capture_site_id.is_none()) {
            let action = match event.action {
                ScopeEffectEventsAction::Enter => "ScanEffect::Enter".to_owned(),
                ScopeEffectEventsAction::Leave => "ScanEffect::Leave".to_owned(),
                ScopeEffectEventsAction::Clear => "ScanEffect::Clear".to_owned(),
                ScopeEffectEventsAction::Declare => format!("ScanEffect::Declare{{name:{:?}.into(),offset_cp:self.cp({state}.consumed)}}",event.name.as_deref().unwrap_or("")),
                ScopeEffectEventsAction::Use => format!("ScanEffect::Use{{name:{:?}.into(),offset_cp:self.cp({state}.consumed),len_cp:0}}",event.name.as_deref().unwrap_or("")),
            };
            writeln!(
                s,
                "self.effect_at({action},{mode:?},self.cp({state}.consumed),0);"
            )
            .unwrap();
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

fn capture_events(g: &Generator<'_>, expr: &Expr, state: &str, out: &str) -> String {
    let mut s = String::new();
    for site in &expr.capture_site_ids {
        let index = g.sites[site.as_str()];
        let capture = &g.ir.semantics.captures[index];
        if capture.skip {
            continue;
        }
        let token_extent = match &expr.node {
            ExprNode::Optional { child } | ExprNode::Repeat { child, .. } => direct_token(g, child),
            _ => false,
        };
        let span = match capture.span_rule {
            CaptureSpanRule::MatchedExtent => "[state.matched,out.state.matched]",
            CaptureSpanRule::SelectedChildExtent => {
                "self.selected_extent(out.events,[state.consumed,out.state.consumed])"
            }
            _ => "[state.consumed,out.state.consumed]",
        };
        writeln!(
            s,
            "out.events=self.event(Event::Capture {{site:{index},span:{span},child:out.events,token_extent:{token_extent}}});"
        )
        .unwrap();
    }
    s.replace("out.", &format!("{out}."))
        .replace("[state.", &format!("[{state}."))
}

// Text-only syntax supplies one lexeme. Only references that can produce mapped
// values participate in semantic cardinality; delimiters do not become values.
fn semantic_shape(
    g: &Generator<'_>,
    id: &ExprId,
    visiting: &mut std::collections::BTreeSet<ExprId>,
) -> Option<Cardinality> {
    semantic_shape_cached(g, id, visiting, &mut std::collections::BTreeMap::new())
}
fn semantic_shape_cached(
    g: &Generator<'_>,
    id: &ExprId,
    visiting: &mut std::collections::BTreeSet<ExprId>,
    cache: &mut std::collections::BTreeMap<ExprId, Option<Cardinality>>,
) -> Option<Cardinality> {
    if let Some(shape) = cache.get(id) {
        return *shape;
    }
    if !visiting.insert(id.to_owned()) {
        return None;
    }
    let shape = match &g.ir.semantics.expressions[g.expr[id.as_str()]].node {
        ExprNode::RuleRef { symbol_id } => {
            let rule = &g.ir.semantics.rules[g.rules[symbol_id.as_str()]];
            if rule.skip {
                None
            } else if rule.mapping_type_id.is_some() {
                Some(Cardinality::Scalar)
            } else {
                semantic_shape_cached(g, &rule.body, visiting, cache)
            }
        }
        ExprNode::Seq { items } => {
            let values: Vec<_> = items
                .iter()
                .filter_map(|id| semantic_shape_cached(g, id, visiting, cache))
                .collect();
            match values.as_slice() {
                [] => None,
                [one] => Some(*one),
                _ => Some(Cardinality::List),
            }
        }
        ExprNode::Choice { alternatives, .. } => alternatives
            .iter()
            .filter_map(|id| semantic_shape_cached(g, id, visiting, cache))
            .reduce(|a, b| {
                if a == Cardinality::List || b == Cardinality::List {
                    Cardinality::List
                } else if a == Cardinality::Optional || b == Cardinality::Optional {
                    Cardinality::Optional
                } else {
                    Cardinality::Scalar
                }
            }),
        ExprNode::Group { child } => semantic_shape_cached(g, child, visiting, cache),
        ExprNode::Optional { child } => semantic_shape_cached(g, child, visiting, cache).map(|n| {
            if n == Cardinality::List {
                n
            } else {
                Cardinality::Optional
            }
        }),
        ExprNode::Repeat { child, .. } | ExprNode::Separated { element: child, .. } => {
            semantic_shape_cached(g, child, visiting, cache).map(|_| Cardinality::List)
        }
        _ => None,
    };
    visiting.remove(id);
    cache.insert(id.clone(), shape);
    shape
}

fn direct_token(g: &Generator<'_>, id: &ExprId) -> bool {
    direct_token_inner(g, id, false)
}
fn direct_token_inner(g: &Generator<'_>, id: &ExprId, grouped: bool) -> bool {
    match &g.ir.semantics.expressions[g.expr[id.as_str()]].node {
        ExprNode::Seq { items } if items.len() == 1 => direct_token_inner(g, &items[0], grouped),
        ExprNode::Group { child } if !grouped => direct_token_inner(g, child, true),
        ExprNode::Choice { alternatives, .. } => alternatives
            .iter()
            .all(|id| direct_token_inner(g, id, grouped)),
        ExprNode::Literal { .. } | ExprNode::TokenRef { .. } | ExprNode::Any {} => true,
        _ => false,
    }
}
