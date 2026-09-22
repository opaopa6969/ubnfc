fn occurrences(&self, root: EventId) -> (Vec<Capture>, Vec<LexicalOccurrence>) {
    let mut captures = Vec::new();
    let mut lexical = Vec::new();
    let mut stack = vec![(root, None, false, None)];
    let mut lexical_order = 0;
    while let Some((id, parent, done, current)) = stack.pop() {
        match self.arena[id.0] {
            Event::Join(a, b) => {
                stack.push((b, parent, false, None));
                stack.push((a, parent, false, None));
            }
            Event::Capture { site, span, child } => {
                if done {
                    if self.options.build_ast {
                        let order = captures.len();
                        captures.push(Capture {
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
            Event::Rule { rule, span, child } => {
                if done {
                    if let Some(index) = current {
                        let node: &mut LexicalOccurrence = &mut lexical[index];
                        node.completion_order = lexical_order;
                        lexical_order += 1;
                    }
                } else {
                    let index = if self.options.lexical {
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
    (captures, lexical)
}
pub(crate) fn finish(mut self, mut step: Step) -> ParseResult {
    if let Some(offset) = self.depth_failure {
        step = self.fail(State::default(), offset, "maximum parse depth exceeded");
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
        });
    }
    if trailing {
        let (farthest_cp, farthest_expected, deepest_rule, rule_path) =
            diagnostics
                .pop()
                .map_or((consumed_cp, vec![], None, vec![]), |d| {
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
        });
    }
    let (captures, lexical) = if ok {
        self.occurrences(step.events)
    } else {
        (vec![], vec![])
    };
    let ast = if ok && self.options.build_ast {
        match self.build_values(step.events).and_then(|mut nodes| {
            if nodes.len() == 1 {
                Ok(nodes.remove(0))
            } else {
                Err("entry must produce one AST value".into())
            }
        }) {
            Ok(ast) => Some(ast),
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
                });
                None
            }
        }
    } else {
        None
    };
    self.statistics.memo_entries = self.memo.len();
    self.statistics.recipes = self.arena.len() - 1;
    self.statistics.ast_nodes = self.node_spans.len();
    ParseResult {
        ok,
        consumed_cp,
        matched_cp,
        ast,
        captures,
        lexical,
        node_spans: self.node_spans,
        diagnostics,
        declarations: if ok {
            self.scope.all_declarations().to_vec()
        } else {
            vec![]
        },
        references: if ok {
            self.scope.references().to_vec()
        } else {
            vec![]
        },
        scope_diagnostics: if ok {
            self.scope.diagnostics().to_vec()
        } else {
            vec![]
        },
        statistics: self.statistics,
        scope: ScopeResult {
            mode: SCOPE_MODES.to_vec(),
            events: if ok {
                self.effects
                    .iter()
                    .enumerate()
                    .map(|(order, effect)| {
                        let (action, name, offset_cp, length_cp) = match &effect.value {
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
