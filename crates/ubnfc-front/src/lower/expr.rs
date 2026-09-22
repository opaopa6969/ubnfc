use super::*;
use r::{ResolvedAtom as A, ResolvedBody, ResolvedElement, ResolvedRule};

pub(super) fn lower(
    cx: &mut Context<'_>,
    rule: &ResolvedRule,
    policy: &str,
) -> (ExprId, Vec<String>) {
    let start = cx.out.semantics.captures.len();
    let typeof_start = cx.out.semantics.typeof_constraints.len();
    let duplicated = cx
        .grammar
        .grammars
        .iter()
        .flat_map(|g| &g.rules)
        .filter(|r| r.unit == rule.unit && r.span == rule.span)
        .count()
        > 1;
    let role = if duplicated {
        format!(
            "body/binding/{}/{}",
            cx.grammar.grammars[cx.grammar.symbol(rule.id).grammar.0].name,
            rule.name
        )
    } else {
        "body".into()
    };
    let mut builder = Builder {
        occurrences: captures::occurrences(cx, rule),
        cx,
        rule,
        policy,
        depth: 0,
        completion: 0,
    };
    let body = builder.body(&rule.body, &role, true, true);
    // typeof は同名 site 群を集約した mapping field の型を比較する。
    // スキーマの参照キーにはその field の代表 site を保存する（前方参照も可）。
    for constraint in &mut builder.cx.out.semantics.typeof_constraints[typeof_start..] {
        if let Some(reference) = builder.cx.out.semantics.captures[start..]
            .iter()
            .find(|c| c.name == constraint.referenced_capture)
        {
            constraint.referenced_capture = reference.site_id.clone();
        } else {
            builder.cx.errors.push(error(
                "E-LOWER-TYPEOF-CAPTURE",
                format!(
                    "{}: @typeof({}) の capture がありません",
                    rule.name, constraint.referenced_capture
                ),
            ));
        }
    }
    let sites = builder.cx.out.semantics.captures[start..]
        .iter()
        .map(|c| c.site_id.clone())
        .collect();
    (body, sites)
}
struct Builder<'a, 'b> {
    occurrences: BTreeMap<String, captures::Shape>,
    cx: &'a mut Context<'b>,
    rule: &'a ResolvedRule,
    policy: &'a str,
    depth: usize,
    completion: usize,
}
impl Builder<'_, '_> {
    fn reserve(&mut self, span: a::Span, role: &str) -> usize {
        let source = self.cx.span(self.rule.unit, span);
        let id = ExprId(format!(
            "expr:{}:{}:{}:{role}",
            source.source_id, source.start, source.end
        ));
        let i = self.cx.out.semantics.expressions.len();
        self.cx.out.semantics.expressions.push(Expr {
            id,
            source,
            rule_id: self.cx.sid(self.rule.id),
            capture_site_ids: vec![],
            diagnostic_label: None,
            execution: execution(&ExprNode::Empty {}),
            node: ExprNode::Empty {},
        });
        i
    }
    fn id(&self, i: usize) -> ExprId {
        self.cx.out.semantics.expressions[i].id.clone()
    }
    fn finish(&mut self, i: usize, node: ExprNode) -> ExprId {
        let id = self.id(i);
        let label = match &node {
            ExprNode::Literal { value, .. } => Some(format!("'{value}'")),
            ExprNode::Error { message } => Some(message.clone()),
            _ => None,
        };
        if let Some(text) = &label {
            self.cx
                .out
                .semantics
                .diagnostics
                .expected_display
                .push(ExpectedDisplay {
                    expr_id: id.clone(),
                    java: text.clone(),
                    rust: match &node {
                        ExprNode::Literal { value, .. } => value.clone(),
                        _ => text.clone(),
                    },
                });
        }
        let e = &mut self.cx.out.semantics.expressions[i];
        e.execution = execution(&node);
        e.node = node;
        e.diagnostic_label = label;
        id
    }
    fn body(&mut self, body: &ResolvedBody, role: &str, force_chain: bool, top: bool) -> ExprId {
        if let [seq] = body.alternatives.as_slice() {
            return self.seq(seq, role, force_chain, body.span);
        }
        let i = self.reserve(body_extent(body), &format!("{role}/choice"));
        let alternatives = body
            .alternatives
            .iter()
            .enumerate()
            .map(|(n, s)| {
                let empty_span =
                    original_sequence_span(self.cx, self.rule, body.span, n).unwrap_or(body.span);
                self.seq(s, &format!("{role}/{n}"), false, empty_span)
            })
            .collect();
        let selection = if top
            && self
                .rule
                .annotations
                .iter()
                .any(|a| matches!(a, a::Annotation::LongestChoice { .. }))
        {
            ChoiceSelection::Longest
        } else if top
            && self
                .rule
                .annotations
                .iter()
                .any(|a| matches!(a, a::Annotation::PredictiveChoice { .. }))
        {
            ChoiceSelection::Predictive
        } else {
            ChoiceSelection::Ordered
        };
        self.finish(
            i,
            ExprNode::Choice {
                alternatives,
                selection,
                tie_break: ExprNodeChoiceTieBreak,
            },
        )
    }
    fn seq(
        &mut self,
        seq: &[ResolvedElement],
        role: &str,
        force: bool,
        empty_span: a::Span,
    ) -> ExprId {
        if seq.len() == 1 && !force {
            return self.element(&seq[0], role);
        }
        let span = if seq.is_empty() {
            empty_span
        } else {
            seq_extent(seq)
        };
        let i = self.reserve(span, &format!("{role}/seq"));
        trivia::chain(self.cx, self.policy, &self.id(i), seq.len());
        let items = seq
            .iter()
            .enumerate()
            .map(|(n, e)| self.element(e, &format!("{role}/{n}")))
            .collect();
        self.finish(i, ExprNode::Seq { items })
    }
    fn element(&mut self, e: &ResolvedElement, role: &str) -> ExprId {
        // site 配列は構文前順。完了順は子の処理後に確定する。
        let ci = e.capture.as_ref().map(|name| {
            let index = self.cx.out.semantics.captures.len();
            let shape = &self.occurrences[name];
            self.cx.out.semantics.captures.push(Capture {
                cardinality: shape.count,
                conversion: captures::conversion(&captures::captured(self.cx, &e.atom).value),
                expr_id: ExprId(String::new()),
                name: name.clone(),
                occurrence_order: CaptureOccurrenceOrder,
                rule_id: self.cx.sid(self.rule.id),
                site_id: String::new(),
                skip: skipped(self.rule),
                span_rule: CaptureSpanRule::ConsumedExtent,
                value_type: captures::captured(self.cx, &e.atom).value,
            });
            index
        });
        let id = self.atom(&e.atom, role);
        if let Some(ci) = ci {
            let site_id = format!("{id}/capture/0");
            let c = &mut self.cx.out.semantics.captures[ci];
            c.expr_id = id.clone();
            c.site_id = site_id.clone();
            self.cx
                .out
                .semantics
                .expressions
                .iter_mut()
                .find(|e| e.id == id)
                .unwrap()
                .capture_site_ids
                .push(site_id.clone());
            let values = value_ids(&self.cx.out.semantics.expressions, &id);
            self.cx.out.capture_bindings.push(CaptureBinding {
                site_id: site_id.clone(),
                value_expr_ids: values,
                inside_quantifier: self.depth > 0,
                quantifier_depth: self.depth,
                completion_order: self.completion,
            });
            self.completion += 1;
            if let Some(reference) = &e.typeof_constraint {
                self.cx.out.semantics.typeof_constraints.push(Typeof {
                    site_id: format!("{id}/typeof/0"),
                    own_capture: site_id,
                    referenced_capture: reference.clone(),
                    comparison_kind: TypeofComparisonKind,
                    mismatch: TypeofMismatch,
                    phase: TypeofPhase,
                });
            }
        }
        id
    }
    fn atom(&mut self, atom: &A, role: &str) -> ExprId {
        let bracket_optional = matches!(atom, A::Optional { .. })
            && self.cx.text(self.rule.unit)[atom.span().start..].starts_with('[');
        let kind = match atom {
            A::Literal { .. } => "literal",
            A::Ref { symbol, .. } => {
                if self.cx.grammar.symbol(*symbol).kind == r::SymbolKind::Rule {
                    "ruleRef"
                } else {
                    "tokenRef"
                }
            }
            A::Error { .. } => "error",
            A::Group { .. } => "group",
            A::Separated { .. } => "separated",
            A::Optional { .. } if bracket_optional => "optional",
            A::Optional { .. } | A::Repeat { .. } | A::OneOrMore { .. } | A::Bounded { .. } => {
                "repeat"
            }
        };
        let role_node = format!("{role}/{kind}");
        let lowered_role = if matches!(atom, A::Separated { .. }) {
            format!("{role_node}/lower/separated/0")
        } else {
            role_node.clone()
        };
        let i = self.reserve(atom.span(), &lowered_role);
        let node = match atom {
            A::Literal { value, .. } => ExprNode::Literal {
                value: value.clone(),
                case_sensitive: true,
                word_boundary: ExprNodeLiteralWordBoundary,
            },
            A::Ref { symbol, .. } => {
                if self.cx.grammar.symbol(*symbol).kind == r::SymbolKind::Rule {
                    ExprNode::RuleRef {
                        symbol_id: self.cx.sid(*symbol),
                    }
                } else {
                    ExprNode::TokenRef {
                        symbol_id: self.cx.sid(*symbol),
                    }
                }
            }
            A::Error { message, .. } => ExprNode::Error {
                message: message.clone(),
            },
            A::Group { body, .. } => ExprNode::Group {
                child: self.body(body, &format!("{role}/0"), true, false),
            },
            A::Optional { body, .. } | A::Repeat { body, .. } => {
                self.depth += 1;
                let single = single_atom(body);
                let helper = if matches!(atom, A::Optional { .. }) {
                    !matches!(single, Some(A::Ref { .. } | A::Literal { .. }))
                } else {
                    !matches!(single, Some(A::Ref { .. }))
                };
                let child = self.body(body, &format!("{role}/0"), helper, false);
                self.depth -= 1;
                if bracket_optional {
                    ExprNode::Optional { child }
                } else {
                    ExprNode::Repeat {
                        child,
                        min: 0,
                        max: matches!(atom, A::Optional { .. }).then_some(1),
                        terminator: None,
                    }
                }
            }
            A::OneOrMore { body, .. } | A::Bounded { body, .. } => {
                self.depth += 1;
                let child = if matches!(**body, A::Ref { .. }) {
                    self.atom(body, &format!("{role}/0"))
                } else {
                    let j = self.reserve(body.span(), &format!("{role_node}/lower/helper/0"));
                    let id = self.id(j);
                    trivia::insert(self.cx, self.policy, &id, BoundaryAt::HelperEntry, None);
                    trivia::insert(self.cx, self.policy, &id, BoundaryAt::HelperExit, None);
                    let child = self.atom(body, &format!("{role}/0"));
                    self.finish(j, ExprNode::Seq { items: vec![child] })
                };
                self.depth -= 1;
                let (min, max) = match atom {
                    A::Bounded { min, max, .. } => (*min, *max),
                    _ => (1, None),
                };
                ExprNode::Repeat {
                    child,
                    min,
                    max,
                    terminator: None,
                }
            }
            A::Separated {
                element, separator, ..
            } => {
                // 原構文の site は一つ。element Expr は先頭と反復 pair で共有する。
                trivia::chain(self.cx, self.policy, &self.id(i), 2);
                self.depth += 1;
                let elem = self.atom(element, &format!("{role}/0"));
                let repeat = self.reserve(atom.span(), &format!("{role_node}/lower/separated/1"));
                let pair = self.reserve(atom.span(), &format!("{role_node}/lower/separated/2"));
                trivia::chain(self.cx, self.policy, &self.id(pair), 2);
                let sep = self.atom(separator, &format!("{role}/1"));
                let pair_id = self.finish(
                    pair,
                    ExprNode::Seq {
                        items: vec![sep, elem.clone()],
                    },
                );
                let repeat_id = self.finish(
                    repeat,
                    ExprNode::Repeat {
                        child: pair_id,
                        min: 0,
                        max: None,
                        terminator: None,
                    },
                );
                self.depth -= 1;
                ExprNode::Seq {
                    items: vec![elem, repeat_id],
                }
            }
        };
        self.finish(i, node)
    }
}
pub(super) fn seq_extent(seq: &[ResolvedElement]) -> a::Span {
    a::Span {
        start: seq.first().map_or(0, |e| e.atom.span().start),
        end: seq.last().map_or(0, |e| e.atom.span().end),
    }
}
pub(super) fn body_extent(body: &ResolvedBody) -> a::Span {
    let first = body.alternatives.first().and_then(|s| s.first());
    let last = body.alternatives.last().and_then(|s| s.last());
    a::Span {
        start: first.map_or(body.span.start, |e| e.atom.span().start),
        end: last.map_or(body.span.end, |e| e.atom.span().end),
    }
}
// resolve は各空 alternative の span を保持しないため、元 AST の span を使う。
fn original_sequence_span(
    cx: &Context<'_>,
    rule: &ResolvedRule,
    span: a::Span,
    index: usize,
) -> Option<a::Span> {
    fn find(body: &a::RuleBody, span: a::Span, index: usize) -> Option<a::Span> {
        if body.span() == span {
            return body.alternatives().get(index).map(|s| s.span);
        }
        body.alternatives()
            .iter()
            .flat_map(|s| &s.elements)
            .find_map(|e| {
                atom(&e.atom, span, index)
                    .or_else(|| e.separator.as_ref().and_then(|s| atom(s, span, index)))
            })
    }
    fn atom(a: &a::Atom, span: a::Span, index: usize) -> Option<a::Span> {
        match a {
            a::Atom::Group { body, .. }
            | a::Atom::Optional { body, .. }
            | a::Atom::Repeat { body, .. } => find(body, span, index),
            _ => None,
        }
    }
    cx.grammar.units[rule.unit.0]
        .ast
        .rules
        .iter()
        .find(|r| r.span == rule.span)
        .and_then(|r| find(&r.body, span, index))
}
fn single_atom(body: &ResolvedBody) -> Option<&A> {
    if let [seq] = body.alternatives.as_slice() {
        if let [e] = seq.as_slice() {
            return Some(&e.atom);
        }
    }
    None
}
fn value_ids(expressions: &[Expr], id: &ExprId) -> Vec<ExprId> {
    let e = expressions.iter().find(|e| &e.id == id).unwrap();
    if id.0.ends_with("/separated/lower/separated/0") {
        if let ExprNode::Seq { items } = &e.node {
            return value_ids(expressions, &items[0]);
        }
    }
    match &e.node {
        ExprNode::Optional { child } | ExprNode::Repeat { child, .. } => {
            value_ids(expressions, child)
        }
        ExprNode::Seq { items } if items.len() == 1 => value_ids(expressions, &items[0]),
        _ => vec![id.clone()],
    }
}
pub(super) fn execution(node: &ExprNode) -> Execution {
    let mut e = Execution {
        child_mode: ExecutionChildMode::Inherit,
        entry_flag: ExecutionEntryFlag::Preserve,
        failure_cursor: ExecutionFailureCursor::RestoreTransaction,
        primitive_profile: None,
        transaction: ExecutionTransaction::Once,
    };
    match node {
        ExprNode::Seq { .. } => e.entry_flag = ExecutionEntryFlag::SetFalse,
        ExprNode::Choice { .. } => e.transaction = ExecutionTransaction::PerAlternative,
        ExprNode::Optional { .. } | ExprNode::Repeat { .. } | ExprNode::Eof {} => {
            e.transaction = ExecutionTransaction::Occurs
        }
        ExprNode::RuleRef { .. } | ExprNode::TokenRef { .. } | ExprNode::Group { .. } => {
            e.transaction = ExecutionTransaction::Delegate;
            e.failure_cursor = ExecutionFailureCursor::Delegate;
        }
        ExprNode::Literal { .. } | ExprNode::Any {} => {
            e.transaction = ExecutionTransaction::None;
            e.failure_cursor = ExecutionFailureCursor::PrimitiveMayResetMatched;
            e.primitive_profile = Some(
                if matches!(node, ExprNode::Literal { .. }) {
                    "legacyWord"
                } else {
                    "legacyAny"
                }
                .into(),
            );
        }
        ExprNode::Lookahead { .. } | ExprNode::NegativeLookahead { .. } => {
            e.child_mode = ExecutionChildMode::MatchOnly
        }
        _ => {}
    }
    e
}
