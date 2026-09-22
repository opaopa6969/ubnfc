fn occurrences(
    &self,
    root: EventId,
) -> (Vec<Capture>, Vec<LexicalOccurrence>, Vec<LexicalOccurrence>) {
    let mut captures = Vec::new();
    let mut capture_owners = Vec::new();
    let mut rule_orders = Vec::new();
    let mut rule_order = 0;
    let mut lexical = Vec::new();
    let mut tokens = Vec::new();
    let mut stack = vec![(root, None, false, None)];
    let mut lexical_order = 0;
    while let Some((id, parent, done, current)) = stack.pop() {
        match self.arena[id.0] {
            Event::Join(a, b) => {
                stack.push((b, parent, false, None));
                stack.push((a, parent, false, None));
            }
            Event::Values { child, .. } => stack.push((child, parent, false, None)),
            Event::Capture {
                site, span, child, ..
            } => {
                if done {
                    if self.options.build_ast || self.options.lexical {
                        capture_owners.push(parent);
                        let order = captures.len();
                        captures.push(Capture {
                            rule_completion_order: 0,
                            occurrence_id: order,
                            site_id: SITES[site].0,
                            name: SITES[site].1,
                            span: self.span(span),
                            completion_order: order,
                        });
                    }
                } else {
                    stack.push((id, parent, true, None));
                    stack.push((child, parent, false, None));
                }
            }
            Event::Token {
                expr, rule, span, ..
            } if self.options.lexical && expr != usize::MAX => {
                tokens.push(LexicalOccurrence {
                    occurrence_id: tokens.len(),
                    parent_occurrence_id: parent,
                    rule_id: RULES[rule],
                    expr_id: EXPRESSIONS[expr],
                    span: self.span(span),
                    completion_order: lexical_order,
                });
                lexical_order += 1;
            }
            Event::Rule { rule, span, child } => {
                if done {
                    if let Some(index) = current {
                        rule_orders[index] = rule_order;
                        rule_order += 1;
                        let node: &mut LexicalOccurrence = &mut lexical[index];
                        node.completion_order = lexical_order;
                        lexical_order += 1;
                    }
                } else {
                    let index = if self.options.lexical || self.options.build_ast {
                        rule_orders.push(0);
                        let index = lexical.len();
                        lexical.push(LexicalOccurrence {
                            occurrence_id: index,
                            parent_occurrence_id: parent,
                            rule_id: RULES[rule],
                            expr_id: BODIES[rule],
                            span: self.span(span),
                            completion_order: 0,
                        });
                        Some(index)
                    } else {
                        parent
                    };
                    stack.push((id, parent, true, index));
                    stack.push((child, index, false, None));
                }
            }
            _ => {}
        }
    }
    for (capture, owner) in captures.iter_mut().zip(capture_owners) {
        capture.rule_completion_order = owner.map_or(0, |i| rule_orders[i]);
    }
    for token in &mut tokens {
        token.occurrence_id += lexical.len();
    }
    if !self.options.lexical {
        lexical.clear();
    }
    (captures, lexical, tokens)
}
fn recovery_occurrences(&self, root: EventId) -> (Vec<Recovery>, Vec<Diagnostic>) {
    let mut recoveries: Vec<Recovery> = Vec::new();
    let mut diagnostics = Vec::new();
    let mut stack = vec![(root, None)];
    let mut capture_order = 0;
    while let Some((id, done)) = stack.pop() {
        match self.arena[id.0] {
            Event::Join(a, b) => {
                stack.push((b, None));
                stack.push((a, None));
            }
            Event::Rule { child, .. } | Event::Values { child, .. } => stack.push((child, None)),
            Event::Capture { child, .. } => {
                if let Some(start) = done {
                    if self.options.build_ast || self.options.lexical {
                        for r in &mut recoveries[start..] {
                            r.capture_occurrence_ids.push(capture_order);
                        }
                        capture_order += 1;
                    }
                } else {
                    stack.push((id, Some(recoveries.len())));
                    stack.push((child, None));
                }
            }
            Event::Recovery {
                rule,
                mode,
                span,
                sync_span,
                diag,
            } => {
                let (pos, expected, path) =
                    self.diag.summary(diag).unwrap_or((span[0], vec![], vec![]));
                let rule_path: Vec<_> = path.into_iter().map(|r| RULES[r]).collect();
                let span = self.span(span);
                diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Recovery,
                    offset_cp: span[0],
                    length_cp: span[1] - span[0],
                    farthest_cp: self.cp(pos),
                    expected: expected.clone(),
                    farthest_expected: expected,
                    deepest_rule: rule_path.last().copied(),
                    rule_path,
                    message: Some("recovered input".into()),
                    recovery_id: Some(recoveries.len()),
                });
                recoveries.push(Recovery {
                    rule_id: RULES[rule],
                    mode,
                    span,
                    sync_span: sync_span.map(|s| self.span(s)),
                    capture_occurrence_ids: vec![],
                });
            }
            _ => {}
        }
    }
    (recoveries, diagnostics)
}
pub(crate) fn finish(mut self, mut step: Step) -> ParseResult {
    if let Some(offset) = self.depth_failure {
        step = self.fail(State::default(), offset, "maximum parse depth exceeded");
    }
    if self.diag.exhausted || self.display.exhausted {
        self.resource_failure
            .get_or_insert((0, "diagnostic budget exceeded"));
    }
    if self.resource_failure.is_some() {
        step = Step {
            ok: false,
            ..Step::yes(State::default())
        };
    }
    let trailing = step.ok && self.options.require_eof && step.state.consumed != self.text.len();
    let ok = step.ok && !trailing;
    let consumed_cp = if step.ok {
        self.cp(step.state.consumed)
    } else {
        0
    };
    let matched_cp = if step.ok {
        self.cp(step.state.matched)
    } else {
        0
    };
    let mut diagnostics = Vec::new();
    if let Some((offset, label)) = self.resource_failure {
        diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Resource,
            offset_cp: self.cp(offset),
            farthest_cp: self.cp(offset),
            expected: vec![label],
            farthest_expected: vec![label],
            deepest_rule: None,
            rule_path: vec![],
            message: Some(label.into()),
            recovery_id: None,
            length_cp: 0,
        });
    }
    if let Some((pos, expected, path)) = self.diag.summary(step.diag) {
        let rule_path: Vec<_> = path.into_iter().map(|r| RULES[r]).collect();
        diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Syntax,
            offset_cp: self.cp(pos),
            farthest_cp: self.cp(pos),
            farthest_expected: expected.clone(),
            expected,
            deepest_rule: rule_path.last().copied(),
            rule_path,
            message: None,
            recovery_id: None,
            length_cp: 0,
        });
    }
    // D-014: farthest は失敗 frame / 成功枝内の失敗も含む独立観測。
    let (display_pos, display_expected) = self.display.summary();
    for diagnostic in &mut diagnostics {
        if diagnostic.kind == DiagnosticKind::Syntax {
            diagnostic.farthest_cp = self.cp(display_pos);
            diagnostic.farthest_expected = display_expected.clone();
        }
    }
    let hints = diagnostics
        .iter()
        .filter(|d| d.kind == DiagnosticKind::Syntax)
        .cloned()
        .collect();
    if trailing {
        let (farthest_cp, farthest_expected, deepest_rule, rule_path) =
            diagnostics
                .pop()
                .map_or((self.cp(display_pos), display_expected, None, vec![]), |d| {
                    (
                        d.farthest_cp,
                        d.farthest_expected,
                        d.deepest_rule,
                        d.rule_path,
                    )
                });
        diagnostics.push(Diagnostic {
            kind: DiagnosticKind::TrailingInput,
            offset_cp: consumed_cp,
            farthest_cp,
            farthest_expected,
            expected: vec!["end of input"],
            deepest_rule,
            rule_path,
            message: None,
            recovery_id: None,
            length_cp: 0,
        });
    }
    if ok {
        diagnostics.clear();
    }
    let (recoveries, recovery_diagnostics) = if step.ok {
        self.recovery_occurrences(step.events)
    } else {
        (vec![], vec![])
    };
    diagnostics.splice(0..0, recovery_diagnostics);
    let (captures, lexical, tokens) = if step.ok {
        self.occurrences(step.events)
    } else {
        (vec![], vec![], vec![])
    };
    let ast = if ok && self.options.build_ast {
        match self.build_values(step.events).and_then(|mut nodes| {
            if nodes.len() == 1 {
                Ok(Some(nodes.remove(0)))
            } else if nodes.is_empty() && self.has_recovery(step.events) {
                Ok(None)
            } else {
                Err("entry must produce one AST value".into())
            }
        }) {
            Ok(ast) => ast,
            Err(message) => {
                self.node_spans.clear();
                diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Mapping,
                    offset_cp: consumed_cp,
                    farthest_cp: consumed_cp,
                    expected: vec![],
                    farthest_expected: vec![],
                    deepest_rule: None,
                    rule_path: vec![],
                    message: Some(message),
                    recovery_id: None,
                    length_cp: 0,
                });
                None
            }
        }
    } else {
        None
    };
    self.statistics.memo_entries = self.memo.len();
    self.statistics.memo_max_probe = self
        .memo
        .max_probe_len()
        .max(self.trivia_cache.max_probe_len());
    self.statistics.recipes = self.arena.len() - 1;
    self.statistics.ast_nodes = self.node_spans.len();
    let mut value_spans = Vec::new();
    if let Some(ast) = &ast {
        let mut path = String::with_capacity(128);
        ast.collect(&mut path, &self.text_spans, &mut value_spans);
    }
    ParseResult {
        value_spans,
        ok,
        consumed_cp,
        matched_cp,
        ast,
        captures,
        lexical,
        tokens,
        node_spans: self.node_spans,
        diagnostics,
        hints,
        recoveries,
        declarations: if step.ok {
            self.scope.all_declarations().to_vec()
        } else {
            vec![]
        },
        references: if step.ok {
            self.scope.references().to_vec()
        } else {
            vec![]
        },
        scope_diagnostics: if step.ok {
            self.scope.diagnostics().to_vec()
        } else {
            vec![]
        },
        statistics: self.statistics,
        scope: ScopeResult {
            depth: self.scope.depth(),
            mode: SCOPE_MODES.to_vec(),
            events: if step.ok {
                self.effects
                    .iter()
                    .enumerate()
                    .map(|(order, effect)| {
                        let (action, name, offset_cp, length_cp) = match &*effect.value {
                            ScanEffect::Enter => ("enter", None, effect.offset_cp, 0),
                            ScanEffect::Leave => ("leave", None, effect.offset_cp, 0),
                            ScanEffect::Clear => ("clear", None, effect.offset_cp, 0),
                            ScanEffect::Declare { name, offset_cp } => {
                                ("declare", Some(name.clone()), *offset_cp, effect.length_cp)
                            }
                            ScanEffect::Use {
                                name,
                                offset_cp,
                                len_cp,
                            } => ("use", Some(name.clone()), *offset_cp, *len_cp),
                            ScanEffect::Diagnostic {
                                message,
                                offset_cp,
                                len_cp,
                                ..
                            } => ("error", Some(message.clone()), *offset_cp, *len_cp),
                        };
                        ScopeEvent {
                            order,
                            action,
                            mode: effect.mode,
                            name,
                            offset_cp,
                            length_cp,
                        }
                    })
                    .collect()
            } else {
                vec![]
            },
        },
    }
}
