use super::*;
use a::Annotation as A;

fn json(value: impl Serialize) -> JsonValue {
    serde_json::from_value(serde_json::to_value(value).unwrap()).unwrap()
}
pub(super) fn annotation(cx: &Context<'_>, unit: r::UnitId, a: &A) -> Annotation {
    let (name, args): (&str, Vec<(&str, JsonValue)>) = match a {
        A::Root { .. } => ("root", vec![]),
        A::Mapping {
            class_name, params, ..
        } => (
            "mapping",
            vec![("className", json(class_name)), ("params", json(params))],
        ),
        A::Eval { entries, .. } => (
            "eval",
            entries
                .iter()
                .map(|e| (e.key.as_str(), json(&e.value)))
                .collect(),
        ),
        A::Whitespace { style, .. } => (
            "whitespace",
            vec![("style", json(style.as_deref().unwrap_or("javaStyle")))],
        ),
        A::Interleave { profile, .. } => ("interleave", vec![("profile", json(profile))]),
        // 表層の綴りをそのまま metadata に残す（D-044）。意味は `scope.effects[].mode`
        // が正本で、旧綴り `@backref` と新綴りは同じ field へ落ちる。
        A::Backref { name, mode, .. } => (
            match mode {
                None => "backref",
                Some(a::BackrefMode::Symbol) => "symbolRef",
                Some(a::BackrefMode::SameText) => "sameText",
            },
            vec![("name", json(name))],
        ),
        A::ScopeTree { mode, .. } => ("scopeTree", vec![("mode", json(mode))]),
        A::Declares {
            symbol,
            description,
            ..
        } => (
            "declares",
            vec![("symbol", json(symbol)), ("description", json(description))],
        ),
        A::Catalog { context, .. } => ("catalog", vec![("context", json(context))]),
        A::LeftAssoc { .. } => ("leftAssoc", vec![]),
        A::RightAssoc { .. } => ("rightAssoc", vec![]),
        A::LongestChoice { .. } => ("longestChoice", vec![]),
        A::PredictiveChoice { .. } => ("predictiveChoice", vec![]),
        A::Precedence { level, .. } => ("precedence", vec![("level", json(level))]),
        A::Doc { text, .. } => ("doc", vec![("text", json(text))]),
        A::Recovery { mode, sync, .. } => {
            ("recovery", vec![("mode", json(mode)), ("sync", json(sync))])
        }
        A::Skip { .. } => ("skip", vec![]),
        A::CommonField { fields, .. } => ("commonField", vec![("fields", json(fields))]),
        A::Enum { .. } => ("enum", vec![]),
        A::Simple { name, .. } => (name, vec![]),
    };
    Annotation {
        name: name.into(),
        arguments: args.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        source: cx.span(unit, a.span()),
    }
}
pub(super) fn rule(
    cx: &mut Context<'_>,
    grammar: &r::Grammar,
    rule: &r::ResolvedRule,
    body: ExprId,
    policy: String,
    sites: Vec<String>,
) {
    let rule_id = cx.sid(rule.id);
    let eval = rule.annotations.iter().find_map(|a| {
        if let A::Eval { entries, .. } = a {
            let mut params: BTreeMap<_, _> = entries
                .iter()
                .map(|e| (e.key.clone(), json(&e.value)))
                .collect();
            let kind = entries
                .iter()
                .find(|e| e.key == "kind")
                .map_or("", |e| e.value.as_str())
                .to_owned();
            let manual = entries
                .iter()
                .find(|e| e.key == "strategy")
                .is_some_and(|e| e.value == "manual");
            params.remove("kind");
            params.remove("strategy");
            Some(Eval {
                kind: kind.clone(),
                params,
                strategy: if manual
                    || ![
                        "binary_arithmetic",
                        "variable_ref",
                        "conditional",
                        "passthrough",
                        "literal",
                    ]
                    .contains(&kind.as_str())
                {
                    EvalStrategy::AbstractCallback
                } else {
                    EvalStrategy::Auto
                },
            })
        } else {
            None
        }
    });
    let operator = operator(cx, rule);
    cx.out.semantics.rules.push(Rule {
        annotations: rule
            .annotations
            .iter()
            .map(|a| annotation(cx, rule.unit, a))
            .collect(),
        body,
        eval,
        mapping_params: mapping(rule).map_or(vec![], |(_, p)| p.to_vec()),
        mapping_type_id: cx.types.get(&rule.id).cloned(),
        operator,
        skip: skipped(rule),
        symbol_id: rule_id.clone(),
        trivia_policy_id: policy,
    });
    for a in &rule.annotations {
        match a {
            A::Catalog { context, .. } => {
                for capture_site_id in &sites {
                    cx.out.semantics.catalogs.push(Catalog {
                        capture_site_id: capture_site_id.clone(),
                        context: context.clone(),
                        rule_id: rule_id.clone(),
                    });
                }
            }
            A::Recovery { mode, sync, .. } => {
                // `@recovery(auto)` と `@recovery(AUTO)` は同じ注釈。綴りの大小で
                // IR の mode が変わると backend ごとに受理が変わる（2026-09-23 A-1）。
                let spelled = mode.as_deref().map(str::to_ascii_uppercase);
                let mode = match spelled.as_deref() {
                    Some("AUTO") => RecoveryMode::Auto,
                    Some("SKIP") => RecoveryMode::Skip,
                    _ => RecoveryMode::Sync,
                };
                cx.out
                    .recovery_follow_sync_patterns
                    .insert(rule_id.clone(), None);
                cx.out.semantics.recovery.push(Recovery {
                    rule_id: rule_id.clone(),
                    mode,
                    resolved_sync_patterns: if mode == RecoveryMode::Sync {
                        vec![sync.clone().unwrap_or_else(|| ";".into())]
                    } else {
                        vec![]
                    },
                    // 明示 SYNC だけが同期文字列自体を消費する（design §3.10）。
                    // AUTO の同期文字列は FOLLOW、つまり外側が待っている字句なので
                    // 消費しない（D-017「AUTO の過剰消費」は再現しない）。
                    consume_sync: mode == RecoveryMode::Sync,
                    no_sync_action: if mode == RecoveryMode::Skip {
                        RecoveryNoSyncAction::ConsumeToEof
                    } else {
                        RecoveryNoSyncAction::Fail
                    },
                    progress_rule: if mode == RecoveryMode::Skip {
                        RecoveryProgressRule::LegacySkip
                    } else {
                        RecoveryProgressRule::LegacySync
                    },
                    application_sites: vec![],
                    ast_projection: RecoveryAstProjection::LegacyMarker,
                    recipe_id: None,
                });
            }
            _ => {}
        }
    }
    scope(cx, grammar, rule);
}
pub(super) fn operator(cx: &mut Context<'_>, rule: &r::ResolvedRule) -> Option<Operator> {
    let associativity = assoc(rule)?;
    let Some((left, op, right)) = operator_elements(rule) else {
        cx.errors.push(error(
            "E-LOWER-OPERATOR-SHAPE",
            format!("{}: canonical Base {{ Op Right }} が必要", rule.name),
        ));
        return None;
    };
    // 右結合の `operator` metadata は作らない（D-045）。v1 の右結合正規形は表層の
    // `Base @left { Op @op Self @right }` のままで IR に載るが、`analysis::operators::canonical`
    // が右結合の段として認めるのは正規化後の `Base Op Self | Base` の形だけなので、
    // metadata を作ると `AnalysisError::InvalidOperator` で文法ごと落ちる（演算子が
    // 規則参照のときだけ発火していた。演算子がリテラルなら下の `else` で None になり、
    // 同じ形の `@leftAssoc` は通るという結合性の非対称になっていた）。
    if associativity == OperatorAssociativity::Right {
        return None;
    }
    let (r::ResolvedAtom::Ref { symbol: base, .. }, r::ResolvedAtom::Ref { symbol: oper, .. }) =
        (&left.atom, &op.atom)
    else {
        // Java は literal operator / operand も許す。Operator の rule ID 欄で
        // 表現できない場合は recipe + 原 annotation に意味を保持する。
        return None;
    };
    if cx.grammar.symbol(*base).kind != r::SymbolKind::Rule
        || cx.grammar.symbol(*oper).kind != r::SymbolKind::Rule
    {
        return None;
    }
    let site = |name: &Option<String>| {
        name.as_ref()
            .and_then(|n| {
                cx.out
                    .semantics
                    .captures
                    .iter()
                    .find(|c| c.rule_id == cx.sid(rule.id) && &c.name == n)
            })
            .map(|c| c.site_id.clone())
    };
    Some(Operator {
        associativity,
        base_rule_id: cx.sid(*base),
        operator_rule_id: cx.sid(*oper),
        canonical_form: if associativity == OperatorAssociativity::Left {
            OperatorCanonicalForm::LeftOpRightLists
        } else {
            OperatorCanonicalForm::BaseOpSelfOrBase
        },
        left_site_id: site(&left.capture)?,
        op_site_id: site(&op.capture)?,
        right_site_id: site(&right.capture)?,
        precedence: rule
            .annotations
            .iter()
            .find_map(|a| {
                if let A::Precedence { level, .. } = a {
                    Some(*level as i64)
                } else {
                    None
                }
            })
            .unwrap_or(0),
    })
}
pub(super) fn operator_elements(
    rule: &r::ResolvedRule,
) -> Option<(
    &r::ResolvedElement,
    &r::ResolvedElement,
    &r::ResolvedElement,
)> {
    let [seq] = rule.body.alternatives.as_slice() else {
        return None;
    };
    let [left, repeat] = seq.as_slice() else {
        return None;
    };
    let r::ResolvedAtom::Repeat { body, .. } = &repeat.atom else {
        return None;
    };
    let [seq] = body.alternatives.as_slice() else {
        return None;
    };
    let [op, right] = seq.as_slice() else {
        return None;
    };
    Some((left, op, right))
}
fn scope(cx: &mut Context<'_>, grammar: &r::Grammar, rule: &r::ResolvedRule) {
    let scope = rule.annotations.iter().find_map(|a| {
        if let A::ScopeTree { mode, .. } = a {
            Some(mode.as_str())
        } else {
            None
        }
    });
    let any_scope = grammar.rules.iter().any(|r| {
        r.annotations
            .iter()
            .any(|a| matches!(a, A::ScopeTree { .. }))
    });
    let mut events = vec![];
    if scope.is_some() {
        events.push(ScopeEvent {
            action: ScopeEffectEventsAction::Enter,
            capture_site_id: None,
            name: Some(rule.name.clone()),
            when: ScopeEffectEventsWhen::Entry,
        });
        events.push(ScopeEvent {
            action: ScopeEffectEventsAction::Leave,
            capture_site_id: None,
            name: Some(rule.name.clone()),
            when: ScopeEffectEventsWhen::SuccessBeforeLeave,
        });
    }
    for declare in [true, false] {
        for a in &rule.annotations {
            let name = match (declare, a) {
                (true, A::Declares { symbol, .. }) => symbol,
                (false, A::Backref { name, .. }) => name,
                _ => continue,
            };
            // 同名 nested を含めて静的完了順。実行時も occurrence の完了順を使う。
            for binding in &cx.out.capture_bindings {
                let Some(c) = cx.out.semantics.captures.iter().find(|c| {
                    c.site_id == binding.site_id && c.rule_id == cx.sid(rule.id) && &c.name == name
                }) else {
                    continue;
                };
                events.push(ScopeEvent {
                    action: if declare {
                        ScopeEffectEventsAction::Declare
                    } else {
                        ScopeEffectEventsAction::Use
                    },
                    capture_site_id: Some(c.site_id.clone()),
                    name: Some(name.clone()),
                    when: if declare {
                        ScopeEffectEventsWhen::SuccessAfterLeave
                    } else {
                        ScopeEffectEventsWhen::AfterDeclarations
                    },
                });
            }
        }
    }
    if !events.is_empty() {
        // D-044: v2 の明示綴りは「文法のどこかに `@scopeTree` があるか」という非局所な
        // 判断より優先する。旧 `@backref`（mode=None）だけが従来の非局所規則で決まる。
        let explicit = rule.annotations.iter().find_map(|a| match a {
            A::Backref { mode, .. } => mode.as_ref(),
            _ => None,
        });
        let symbol_mode = if scope == Some("dynamic") {
            ScopeEffectMode::Dynamic
        } else {
            ScopeEffectMode::Lexical
        };
        cx.out.semantics.scope.effects.push(ScopeEffect {
            events,
            mode: match explicit {
                Some(a::BackrefMode::SameText) => ScopeEffectMode::SameRuleText,
                Some(a::BackrefMode::Symbol) => symbol_mode,
                None if !any_scope => ScopeEffectMode::SameRuleText,
                None => symbol_mode,
            },
            rule_id: cx.sid(rule.id),
        });
    }
}
pub(super) fn recovery_sites(cx: &mut Context<'_>) {
    for recovery in &mut cx.out.semantics.recovery {
        recovery.application_sites = cx
            .out
            .semantics
            .expressions
            .iter()
            .filter(
                |e| matches!(&e.node,ExprNode::RuleRef {symbol_id} if symbol_id==&recovery.rule_id),
            )
            .map(|e| e.id.clone())
            .collect();
    }
}
