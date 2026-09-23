use ubnfc_front::analysis::{
    analyze, plan_required_features, scope_schedule, AnalysisError, ScopeStep,
};
use ubnfc_front::ir::*;

fn empty_semantics() -> Semantics {
    Semantics {
        ast_types: vec![],
        captures: vec![],
        catalogs: vec![],
        common_fields: vec![],
        contracts: Contracts {
            choice: ContractsChoice,
            cursor: CursorContract {
                begin: CursorContractBegin,
                commit: CursorContractCommit,
                consume: CursorContractConsume,
                coordinate: CursorContractCoordinate,
                frame_flag: CursorContractFrameFlag,
                internal_java: CursorContractInternalJava,
                internal_rust: CursorContractInternalRust,
                match_only: CursorContractMatchOnly,
                mode: CursorContractMode,
                positions: CursorContractPositions,
                rollback: CursorContractRollback,
            },
            left_recursion: ContractsLeftRecursion,
            longest: ContractsLongest,
            mapping_failure: ContractsMappingFailure,
            mapping_phase: ContractsMappingPhase,
            occurrences: OccurrenceContract {
                capture_fields: OccurrenceContractCaptureFields,
                diagnostics: OccurrenceContractDiagnostics,
                lexical_fields: OccurrenceContractLexicalFields,
                lexical_output: OccurrenceContractLexicalOutput,
                node_spans: OccurrenceContractNodeSpans,
                scope_events: OccurrenceContractScopeEvents,
                self_host_transform: OccurrenceContractSelfHostTransform,
            },
            predictive: ContractsPredictive,
            regex_policy: ContractsRegexPolicy,
            repeat: ContractsRepeat,
            repeat_zero_max: ContractsRepeatZeroMax,
            span_unit: ContractsSpanUnit,
        },
        diagnostics: Diagnostics {
            deepest_rule: DiagnosticsDeepestRule,
            expected_display: vec![],
            java_syntax_offset: DiagnosticsJavaSyntaxOffset,
            merge: DiagnosticsMerge,
            negative_lookahead: DiagnosticsNegativeLookahead,
            offset_unit: DiagnosticsOffsetUnit,
            retain_farthest_separately: DiagnosticsRetainFarthestSeparately,
            rust_syntax_offset: DiagnosticsRustSyntaxOffset,
            successful_rule_failures: DiagnosticsSuccessfulRuleFailures,
            trailing_input_offset: DiagnosticsTrailingInputOffset,
        },
        expressions: vec![],
        normalizations: vec![],
        recipes: vec![],
        recovery: vec![],
        rules: vec![],
        scope: Scope {
            effects: vec![],
            rollback: ScopeRollback,
            trim_max_code_point: ScopeTrimMaxCodePoint,
            version_changes: ScopeVersionChanges,
        },
        symbols: vec![],
        tokens: vec![],
        trivia: Trivia {
            insertion_plan: vec![],
            policies: vec![],
        },
        typeof_constraints: vec![],
    }
}
fn span() -> Span {
    Span {
        source_id: "test.ubnf".into(),
        start: 0,
        end: 1,
    }
}
fn expr(s: &mut Semantics, r: &str, node: ExprNode) -> ExprId {
    let id = ExprId(format!("expr:test.ubnf:0:1:e{}", s.expressions.len()));
    let execution = Execution {
        child_mode: ExecutionChildMode::Inherit,
        entry_flag: ExecutionEntryFlag::Preserve,
        failure_cursor: ExecutionFailureCursor::Delegate,
        primitive_profile: None,
        transaction: ExecutionTransaction::None,
    };
    s.expressions.push(Expr {
        id: id.clone(),
        rule_id: r.into(),
        source: span(),
        node,
        execution,
        capture_site_ids: vec![],
        diagnostic_label: None,
    });
    id
}
fn rule(s: &mut Semantics, r: &str, body: ExprId) {
    s.rules.push(Rule {
        symbol_id: r.into(),
        body,
        annotations: vec![],
        eval: None,
        mapping_params: vec![],
        mapping_type_id: None,
        operator: None,
        skip: false,
        trivia_policy_id: "none".into(),
    });
}
fn lit(s: &mut Semantics, r: &str, text: &str) -> ExprId {
    expr(
        s,
        r,
        ExprNode::Literal {
            value: text.into(),
            case_sensitive: true,
            word_boundary: false,
        },
    )
}
fn call(s: &mut Semantics, r: &str, target: &str) -> ExprId {
    expr(
        s,
        r,
        ExprNode::RuleRef {
            symbol_id: target.into(),
        },
    )
}
fn seq(s: &mut Semantics, r: &str, items: Vec<ExprId>) -> ExprId {
    expr(s, r, ExprNode::Seq { items })
}
fn choice(s: &mut Semantics, r: &str, alternatives: Vec<ExprId>) -> ExprId {
    expr(
        s,
        r,
        ExprNode::Choice {
            alternatives,
            selection: ChoiceSelection::Ordered,
            tie_break: ExprNodeChoiceTieBreak,
        },
    )
}
fn token(s: &mut Semantics, r: &str, name: &str, spec: TokenSpec) -> ExprId {
    s.tokens.push(Token {
        symbol_id: name.into(),
        spec,
        annotations: vec![],
        diagnostic_profiles: TokenDiagnosticProfiles {
            java: "test".into(),
            rust: "test".into(),
        },
        memo_safety: MemoSafety {
            asserted_alias: None,
            original_declaration: None,
            success: false,
            failure: false,
            provenance: MemoSafetyProvenance::None,
            purity: MemoSafetyPurity::Unknown,
            reason: "テスト用。組込みは解析器が既知と判定する。".into(),
        },
    });
    expr(
        s,
        r,
        ExprNode::TokenRef {
            symbol_id: name.into(),
        },
    )
}
fn fact<'a>(a: &'a Analysis, id: &ExprId) -> &'a ExprAnalysis {
    a.expressions.iter().find(|e| &e.expr_id == id).unwrap()
}
fn contains(f: &MayStart, cp: u64) -> bool {
    f.ranges.iter().any(|r| r.start <= cp && cp <= r.end)
}
fn check_properties(s: &Semantics, a: &Analysis, p: &Plan) {
    assert_eq!(a.expressions.len(), s.expressions.len());
    for f in &a.expressions {
        if f.nullable {
            assert_eq!(f.min_consume, 0, "{}", f.expr_id);
        }
    }
    for c in &p.choices {
        if c.strategy != ChoicePlanStrategy::Predict || c.fallback != ChoicePlanFallback::None {
            continue;
        }
        for (i, id) in c.candidate_order.iter().enumerate() {
            let left = fact(a, id);
            assert!(!left.nullable);
            assert_ne!(left.may_start.certainty, MayStartCertainty::Unknown);
            for other in &c.candidate_order[i + 1..] {
                let right = fact(a, other);
                assert!(!(left.may_start.eof && right.may_start.eof));
                assert!(left.may_start.ranges.iter().all(|x| right
                    .may_start
                    .ranges
                    .iter()
                    .all(|y| x.end < y.start || y.end < x.start)));
            }
        }
    }
}

#[test]
fn disjoint_choice_predicts_and_records_omission_proof() {
    let mut s = empty_semantics();
    let x = lit(&mut s, "A", "x");
    let y = lit(&mut s, "A", "y");
    let body = choice(&mut s, "A", vec![x.clone(), y.clone()]);
    rule(&mut s, "A", body.clone());
    let (a, p) = analyze(&s).unwrap();
    check_properties(&s, &a, &p);
    assert_eq!(p.choices[0].strategy, ChoicePlanStrategy::Predict);
    assert_eq!(p.choices[0].fallback, ChoicePlanFallback::None);
    assert_eq!(
        p.memo
            .entries
            .iter()
            .find(|m| m.expr_id == body)
            .unwrap()
            .reason,
        MemoEntryReason::DefaultSafe
    );
    for id in [x, y] {
        let m = p.memo.entries.iter().find(|m| m.expr_id == id).unwrap();
        assert_eq!(m.reason, MemoEntryReason::ProvedOmission);
        assert_eq!(
            m.omission_proof.as_ref().unwrap().kind,
            ProofKind::ExclusiveCandidates
        );
    }
}

/// 再帰 SCC でも rule 本体の memo が守るので、本体の局所木で単一所有の部分式は省略できる。
/// 2 つの候補の `'a'` は別 Expr（別 memo key）なので、全 memo でも候補間で hit は起きない。
#[test]
fn recursive_overlapping_choice_keeps_rule_body_memoized_and_omits_owned_subexpressions() {
    let mut s = empty_semantics();
    let mut branches = vec![];
    let mut leaves = vec![];
    for end in ["b", "c"] {
        let a = lit(&mut s, "S", "a");
        let rec = call(&mut s, "S", "S");
        let b = lit(&mut s, "S", end);
        leaves.extend([a.clone(), rec.clone(), b.clone()]);
        branches.push(seq(&mut s, "S", vec![a, rec, b]));
    }
    branches.push(expr(&mut s, "S", ExprNode::Empty {}));
    let body = choice(&mut s, "S", branches.clone());
    rule(&mut s, "S", body.clone());
    let (a, p) = analyze(&s).unwrap();
    check_properties(&s, &a, &p);
    assert!(fact(&a, &body).nullable);
    assert_eq!(fact(&a, &body).min_consume, 0);
    assert!(a.sccs[0].recursive);
    assert_eq!(a.sccs[0].memo_policy, SccMemoPolicy::ProvedOmissions);
    let entry = |id: &ExprId| p.memo.entries.iter().find(|m| &m.expr_id == id).unwrap();
    assert!(entry(&body).cache_success && entry(&body).cache_failure);
    assert_eq!(entry(&body).reason, MemoEntryReason::DefaultSafe);
    for id in &branches {
        assert_eq!(entry(id).reason, MemoEntryReason::ProvedOmission);
        assert_eq!(
            entry(id).omission_proof.as_ref().unwrap().kind,
            ProofKind::ExclusiveCandidates
        );
    }
    for id in &leaves {
        assert!(!entry(id).cache_success && !entry(id).cache_failure);
        assert_eq!(entry(id).reason, MemoEntryReason::ProvedOmission);
    }
    assert_eq!(p.choices[0].strategy, ChoicePlanStrategy::Ordered);
}

#[test]
fn direct_and_indirect_left_recursion_are_explicit_errors() {
    let mut s = empty_semantics();
    let a = call(&mut s, "A", "B");
    rule(&mut s, "A", a);
    let empty = expr(&mut s, "B", ExprNode::Empty {});
    let b = call(&mut s, "B", "A");
    let body = seq(&mut s, "B", vec![empty, b]);
    rule(&mut s, "B", body);
    assert!(
        matches!(&analyze(&s).unwrap_err()[0],AnalysisError::IndirectLeftRecursion { rule_ids } if rule_ids == &["A","B"])
    );
    let mut s = empty_semantics();
    let a = call(&mut s, "A", "A");
    rule(&mut s, "A", a);
    assert!(
        matches!(&analyze(&s).unwrap_err()[0],AnalysisError::DirectLeftRecursion { rule_id } if rule_id == "A")
    );
}

#[test]
fn cursor_contract_and_negation_are_not_boolean_first_complements() {
    let mut s = empty_semantics();
    let x = lit(&mut s, "A", "x");
    let look = expr(&mut s, "A", ExprNode::Lookahead { child: x.clone() });
    let not = expr(
        &mut s,
        "A",
        ExprNode::NegativeLookahead { child: x.clone() },
    );
    let empty = expr(&mut s, "A", ExprNode::Empty {});
    let eof = expr(&mut s, "A", ExprNode::Eof {});
    let neg = token(
        &mut s,
        "A",
        "NEG",
        TokenSpec::Negation {
            excluded: "aé😀".into(),
        },
    );
    let until = token(
        &mut s,
        "A",
        "UNTIL",
        TokenSpec::Until {
            terminator: "#".into(),
            empty_terminator: TokenSpecUntilEmptyTerminator,
            missing_terminator: TokenSpecUntilMissingTerminator,
            preserve_matched: TokenSpecUntilPreserveMatched,
        },
    );
    let body = choice(
        &mut s,
        "A",
        vec![
            x.clone(),
            look.clone(),
            not.clone(),
            empty.clone(),
            eof.clone(),
            neg.clone(),
            until.clone(),
        ],
    );
    rule(&mut s, "A", body);
    let (a, p) = analyze(&s).unwrap();
    check_properties(&s, &a, &p);
    for id in [&look, &not, &empty, &eof] {
        assert_eq!(
            fact(&a, id).cursor_transition.consumed_mode,
            CursorProgress::Preserve
        );
    }
    assert_eq!(
        fact(&a, &look).cursor_transition.matched_mode,
        CursorProgress::Advance
    );
    assert_eq!(
        fact(&a, &not).cursor_transition.matched_mode,
        CursorProgress::Preserve
    );
    assert_eq!(
        fact(&a, &empty).cursor_transition.matched_mode,
        CursorProgress::MayAdvance
    );
    assert_eq!(
        fact(&a, &eof).cursor_transition.matched_mode,
        CursorProgress::Preserve
    );
    assert_eq!(
        fact(&a, &x).cursor_transition.failure,
        CursorTransitionFailure::MayResetMatched
    );
    assert_eq!(
        fact(&a, &look).cursor_transition.failure,
        CursorTransitionFailure::Restore
    );
    assert_eq!(
        fact(&a, &not).may_start.zero_consume_condition,
        MayStartZeroConsumeCondition::Predicate
    );
    assert!(fact(&a, &not).may_start.eof);
    assert!(contains(&fact(&a, &not).may_start, 'x' as u64));
    assert!(fact(&a, &eof).may_start.eof);
    assert!(fact(&a, &eof).may_start.ranges.is_empty());
    let n = &fact(&a, &neg).may_start;
    assert!(!n.eof);
    for cp in ['a', 'é', '😀'] {
        assert!(!contains(n, cp as u64));
    }
    for cp in ['b', 'ê', '😁', '\0'] {
        assert!(contains(n, cp as u64));
    }
    assert_eq!(fact(&a, &until).min_consume, 0);
    assert_eq!(
        fact(&a, &until).cursor_transition.consumed_mode,
        CursorProgress::MayAdvance
    );
    assert!(fact(&a, &until).may_start.eof);
}

#[test]
fn extern_closure_is_fail_closed_and_alias_guarantee_is_respected() {
    let mut s = empty_semantics();
    let a = call(&mut s, "A", "B");
    rule(&mut s, "A", a);
    let ext = token(
        &mut s,
        "B",
        "G::X",
        TokenSpec::Extern {
            first_chars: None,
            logical_id: "X".into(),
            original_class_name: "custom.X".into(),
            java_binding: None,
            rust_binding: None,
            input_state: vec![],
            result_fields: TokenSpecExternResultFields,
        },
    );
    rule(&mut s, "B", ext.clone());
    let c = lit(&mut s, "C", "c");
    rule(&mut s, "C", c.clone());
    let (a, p) = analyze(&s).unwrap();
    check_properties(&s, &a, &p);
    for m in &p.memo.entries {
        assert_eq!(m.cache_success, m.expr_id == c);
        assert_eq!(m.cache_failure, m.expr_id == c);
    }
    assert_eq!(
        fact(&a, &ext).may_start.certainty,
        MayStartCertainty::Unknown
    );
    assert!(fact(&a, &ext).may_start.eof);
    assert!(fact(&a, &ext).nullable);
    s.tokens[0].memo_safety = MemoSafety {
        asserted_alias: Some("X".into()),
        original_declaration: None,
        success: true,
        failure: true,
        provenance: MemoSafetyProvenance::UserAliasAssertion,
        purity: MemoSafetyPurity::StateRead,
        reason: "@memoSafeToken".into(),
    };
    let (_, p) = analyze(&s).unwrap();
    assert!(p
        .memo
        .entries
        .iter()
        .all(|m| m.cache_success && m.cache_failure));
    s.tokens[0].memo_safety.asserted_alias = Some("別名".into());
    assert!(analyze(&s)
        .unwrap()
        .1
        .memo
        .entries
        .iter()
        .any(|m| m.reason == MemoEntryReason::UnsafeExternClosure));
}

#[test]
fn explicit_choice_policies_and_trie_keep_short_and_unknown_candidates() {
    let mut s = empty_semantics();
    let short = lit(&mut s, "A", "var");
    let long = lit(&mut s, "A", "variable");
    let any = expr(&mut s, "A", ExprNode::Any {});
    let body = choice(&mut s, "A", vec![short.clone(), long.clone(), any.clone()]);
    rule(&mut s, "A", body.clone());
    let (a, p) = analyze(&s).unwrap();
    assert_eq!(p.choices[0].strategy, ChoicePlanStrategy::Ordered);
    let trie = &a.prefix_tries[0];
    assert_eq!(trie.max_depth, 8);
    for n in trie.nodes.iter().skip(3) {
        assert!(n.completed_candidate_expr_ids.contains(&short));
        assert!(n.unknown_candidate_expr_ids.contains(&any));
    }
    let e = s.expressions.iter_mut().find(|e| e.id == body).unwrap();
    if let ExprNode::Choice { selection, .. } = &mut e.node {
        *selection = ChoiceSelection::Predictive;
    }
    let (_, p) = analyze(&s).unwrap();
    assert_eq!(p.choices[0].strategy, ChoicePlanStrategy::Predict);
    assert_eq!(p.choices[0].fallback, ChoicePlanFallback::FullChoice);
    if let ExprNode::Choice { selection, .. } = &mut s.expressions.last_mut().unwrap().node {
        *selection = ChoiceSelection::Longest;
    }
    assert_eq!(
        analyze(&s).unwrap().1.choices[0].strategy,
        ChoicePlanStrategy::Longest
    );
}

#[test]
fn canonical_operator_tower_uses_reference_order_and_rejects_trivia() {
    let mut s = empty_semantics();
    let atom = lit(&mut s, "T", "x");
    rule(&mut s, "T", atom);
    let base = call(&mut s, "E", "T");
    let plus = lit(&mut s, "E", "+");
    let minus = lit(&mut s, "E", "-");
    let op = choice(&mut s, "E", vec![plus, minus]);
    let right = call(&mut s, "E", "T");
    let pair = seq(&mut s, "E", vec![op, right]);
    let repeat = expr(
        &mut s,
        "E",
        ExprNode::Repeat {
            child: pair,
            min: 0,
            max: None,
            terminator: None,
        },
    );
    let body = seq(&mut s, "E", vec![base, repeat]);
    rule(&mut s, "E", body.clone());
    let (a, p) = analyze(&s).unwrap();
    check_properties(&s, &a, &p);
    assert_eq!(p.choices[0].strategy, ChoicePlanStrategy::Predict);
    assert_eq!(
        p.operators[0].strategy,
        OperatorPlanStrategy::PrecedenceClimbing
    );
    assert_eq!(p.operators[0].rule_ids, vec!["E"]);
    s.trivia.policies.push(TriviaPolicy {
        id: "ws".into(),
        effective_delimiter_set: vec![Delimiter::Characters {
            code_points: vec![32],
        }],
        precedence: TriviaPolicyPrecedence,
        raw_settings: Settings {
            comment: None,
            interleave: None,
            whitespace: Some(SettingsWhitespace::JavaStyle),
        },
    });
    s.trivia.insertion_plan.push(TriviaInsertion {
        boundary: Boundary {
            at: BoundaryAt::Entry,
            child_index: None,
            owner_expr_id: body.clone(),
        },
        policy_id: "ws".into(),
    });
    let (a, p) = analyze(&s).unwrap();
    assert_eq!(p.operators[0].strategy, OperatorPlanStrategy::TierFunctions);
    assert!(contains(&fact(&a, &body).may_start, 32));
}

fn auto_recovery(rule: &str, sites: Vec<ExprId>) -> Recovery {
    Recovery {
        rule_id: rule.into(),
        application_sites: sites,
        mode: RecoveryMode::Auto,
        ast_projection: RecoveryAstProjection::LegacyMarker,
        consume_sync: true,
        no_sync_action: RecoveryNoSyncAction::Fail,
        progress_rule: RecoveryProgressRule::LegacySync,
        recipe_id: None,
        resolved_sync_patterns: vec![],
    }
}
#[test]
fn follow_uses_terminal_strings_and_stays_empty_for_a_rule_nothing_follows() {
    let mut s = empty_semantics();
    let atom = lit(&mut s, "A", "x");
    rule(&mut s, "A", atom);
    let a = call(&mut s, "S", "A");
    let end = lit(&mut s, "S", "end");
    let comma = lit(&mut s, "S", ",");
    let opt = expr(&mut s, "S", ExprNode::Optional { child: comma });
    let body = seq(&mut s, "S", vec![a.clone(), opt, end]);
    rule(&mut s, "S", body);
    s.recovery.push(auto_recovery("A", vec![a]));
    s.recovery.push(auto_recovery("S", vec![]));
    let (a, _) = analyze(&s).unwrap();
    assert_eq!(
        a.rules[0].follow_sync_patterns,
        Some(vec![",".into(), "end".into()])
    );
    // FOLLOW そのものを返す。空 AUTO への既定 `;` は pipeline の回復方針
    // （`pipeline_api::auto_recovery_follow_is_integrated_without_changing_sync_or_skip`）。
    assert_eq!(a.rules[1].follow_sync_patterns, Some(vec![]));
    s.recovery[1].mode = RecoveryMode::Sync;
    assert_eq!(analyze(&s).unwrap().0.rules[1].follow_sync_patterns, None);
}

#[test]
fn scope_schedule_preserves_rule_body_boundary_and_validates_event_order() {
    let mut s = empty_semantics();
    let body = lit(&mut s, "A", "x");
    rule(&mut s, "A", body.clone());
    let events = [
        (ScopeEffectEventsAction::Enter, ScopeEffectEventsWhen::Entry),
        (
            ScopeEffectEventsAction::Leave,
            ScopeEffectEventsWhen::SuccessBeforeLeave,
        ),
        (
            ScopeEffectEventsAction::Declare,
            ScopeEffectEventsWhen::SuccessAfterLeave,
        ),
        (
            ScopeEffectEventsAction::Use,
            ScopeEffectEventsWhen::AfterDeclarations,
        ),
    ]
    .into_iter()
    .map(|(action, when)| ScopeEvent {
        action,
        when,
        capture_site_id: None,
        name: Some("x".into()),
    })
    .collect();
    s.scope.effects.push(ScopeEffect {
        rule_id: "A".into(),
        mode: ScopeEffectMode::Lexical,
        events,
    });
    let schedule = scope_schedule(&s, "A").unwrap();
    assert_eq!(schedule.len(), 5);
    assert_eq!(
        schedule[1],
        ScopeStep::Evaluate {
            expr_id: body.clone()
        }
    );
    let (a, p) = analyze(&s).unwrap();
    assert_eq!(
        fact(&a, &body).effects,
        vec!["scope:A:0", "scope:A:1", "scope:A:2", "scope:A:3"]
    );
    assert!(p.memo.entries[0].cache_success);
    assert_eq!(
        fact(&a, &body).memo_safety.purity,
        MemoSafetyPurity::Effectful
    );
    s.scope.effects[0].events.swap(1, 2);
    assert!(analyze(&s).is_err());
}

#[test]
fn capture_cardinality_checks_aggregate_same_name_and_quantifiers() {
    let mut s = empty_semantics();
    let x = lit(&mut s, "A", "x");
    let body = expr(&mut s, "A", ExprNode::Optional { child: x.clone() });
    rule(&mut s, "A", body);
    s.captures.push(Capture {
        site_id: "site:x".into(),
        expr_id: x,
        rule_id: "A".into(),
        name: "x".into(),
        cardinality: Cardinality::Scalar,
        conversion: Conversion::Text,
        occurrence_order: CaptureOccurrenceOrder,
        skip: false,
        span_rule: CaptureSpanRule::ConsumedExtent,
        value_type: ValueType::Text {},
    });
    assert!(matches!(
        analyze(&s).unwrap_err()[0],
        AnalysisError::CaptureCardinality {
            inferred: Cardinality::Optional,
            ..
        }
    ));
    s.captures[0].cardinality = Cardinality::Optional;
    analyze(&s).unwrap();
}

#[test]
fn tinycalc_compare_recorded_differences() {
    let ir = from_json(include_str!("../../../ir/examples/tinycalc.ir.json")).unwrap();
    let (a, p) = analyze(&ir.semantics).unwrap();
    check_properties(&ir.semantics, &a, &p);
    // tinycalc は全 memo / ordered の保守計画。以下だけを契約に基づく精密化として許可する。
    let mut expected_analysis = ir.analysis.clone();
    let mut expected_plan = ir.plan.clone();
    for (expected, actual) in expected_analysis.expressions.iter_mut().zip(&a.expressions) {
        let node = &ir
            .semantics
            .expressions
            .iter()
            .find(|e| e.id == expected.expr_id)
            .unwrap()
            .node;
        // 葉の開始集合は既知、複合式は保守的なまま。範囲・EOF・zero は一切変更しない。
        if matches!(node, ExprNode::Literal { .. } | ExprNode::TokenRef { .. }) {
            expected.may_start.certainty = MayStartCertainty::Exact;
        }
        expected.cursor_transition.failure = match node {
            ExprNode::Seq { .. }
            | ExprNode::Choice { .. }
            | ExprNode::Optional { .. }
            | ExprNode::Repeat { .. } => CursorTransitionFailure::Restore,
            _ => expected.cursor_transition.failure,
        };
        expected.cursor_transition.flag_effect = match node {
            ExprNode::Literal { .. }
            | ExprNode::Choice { .. }
            | ExprNode::Optional { .. }
            | ExprNode::Repeat { .. } => CursorTransitionFlagEffect::Preserve,
            _ => expected.cursor_transition.flag_effect,
        };
        assert_eq!(expected, actual, "{}", actual.expr_id);
    }
    for (expected, actual) in expected_plan.choices.iter_mut().zip(&p.choices) {
        let e = ir
            .semantics
            .expressions
            .iter()
            .find(|e| e.id == expected.expr_id)
            .unwrap();
        if ["TinyCalc::AddOp", "TinyCalc::MulOp"].contains(&e.rule_id.as_str()) {
            expected.strategy = ChoicePlanStrategy::Predict;
            let proof = actual.proof.as_ref().unwrap();
            assert_eq!(proof.kind, ProofKind::ExclusiveCandidates);
            assert_eq!(proof.expr_ids, expected.candidate_order);
            assert!(proof.explanation.contains("consume(0)"));
            expected.proof = Some(proof.clone());
        }
        expected.trie_id = expected_analysis
            .expressions
            .iter()
            .find(|a| a.expr_id == e.id)
            .unwrap()
            .prefix_trie_id
            .clone();
    }
    for (expected, actual) in expected_plan.memo.entries.iter_mut().zip(&p.memo.entries) {
        let e = ir
            .semantics
            .expressions
            .iter()
            .find(|e| e.id == expected.expr_id)
            .unwrap();
        if ["TinyCalc::AddOp", "TinyCalc::MulOp"].contains(&e.rule_id.as_str())
            && matches!(e.node, ExprNode::Literal { .. })
        {
            expected.cache_failure = false;
            expected.cache_success = false;
            expected.reason = MemoEntryReason::ProvedOmission;
            let proof = actual.omission_proof.as_ref().unwrap();
            assert_eq!(proof.kind, ProofKind::ExclusiveCandidates);
            assert_eq!(proof.expr_ids.last(), Some(&e.id));
            assert!(proof.explanation.contains("rule 本体の memo を保持"));
            expected.omission_proof = Some(proof.clone());
        }
    }
    for expected in &mut expected_analysis.sccs {
        if expected.rule_ids == ["TinyCalc::AddOp"] || expected.rule_ids == ["TinyCalc::MulOp"] {
            expected.memo_policy = SccMemoPolicy::ProvedOmissions;
        }
    }
    // trivia があるので climbing を有効にしない。拒否理由だけ追加。
    assert_eq!(p.operators[0].strategy, OperatorPlanStrategy::TierFunctions);
    let proof = p.operators[0].proof.as_ref().unwrap();
    assert_eq!(proof.kind, ProofKind::None);
    assert!(proof.explanation.contains("trivia"));
    assert_eq!(proof.expr_ids, p.operators[0].boundary_expr_ids);
    expected_plan.operators[0].proof = Some(proof.clone());
    expected_plan.profile = PlanProfile::Optimized;
    assert_eq!(a, expected_analysis);
    assert_eq!(p, expected_plan);
    let mut generated = ir.clone();
    generated.analysis = a;
    generated.plan = p;
    for feature in plan_required_features(&generated.plan) {
        if !generated.required_features.contains(&feature) {
            generated.required_features.push(feature);
        }
    }
    generated
        .required_features
        .sort_by_cached_key(|f| serde_json::to_string(f).unwrap());
    let errors = validate_references(&generated);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn generated_small_grammars_obey_nullable_and_predict_properties() {
    // 依存追加なしの有限全探索。異なる初期固定点・nullable prefix・未知 token を混ぜる。
    for left in 0..6 {
        for right in 0..6 {
            for wrapping in 0..4 {
                let mut s = empty_semantics();
                let mut leaves = Vec::new();
                for kind in [left, right] {
                    leaves.push(match kind {
                        0 => lit(&mut s, "S", "a"),
                        1 => lit(&mut s, "S", "b"),
                        2 => expr(&mut s, "S", ExprNode::Empty {}),
                        3 => expr(&mut s, "S", ExprNode::Eof {}),
                        4 => expr(&mut s, "S", ExprNode::Any {}),
                        _ => {
                            let x = lit(&mut s, "S", "x");
                            expr(&mut s, "S", ExprNode::NegativeLookahead { child: x })
                        }
                    });
                }
                let combined = if wrapping % 2 == 0 {
                    choice(&mut s, "S", leaves)
                } else {
                    seq(&mut s, "S", leaves)
                };
                let body = if wrapping >= 2 {
                    expr(
                        &mut s,
                        "S",
                        ExprNode::Repeat {
                            child: combined,
                            min: 0,
                            max: None,
                            terminator: None,
                        },
                    )
                } else {
                    combined
                };
                rule(&mut s, "S", body);
                let (a, p) = analyze(&s).unwrap();
                check_properties(&s, &a, &p);
            }
        }
    }
}

#[test]
fn builtin_acceptance_ranges_and_literal_code_point_lengths() {
    let cases = [
        (BuiltinToken::Number, "+-.0123456789", 1),
        (
            BuiltinToken::Identifier,
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz",
            1,
        ),
        (BuiltinToken::SingleQuoted, "'", 2),
        (BuiltinToken::Quoted, "\"", 2),
        (BuiltinToken::Digit, "0123456789", 1),
        (
            BuiltinToken::Alphabet,
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
            1,
        ),
        (
            BuiltinToken::AlphabetUnderScore,
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz",
            1,
        ),
        (
            BuiltinToken::AlphabetNumericUnderScore,
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz0123456789",
            1,
        ),
        (BuiltinToken::Space, " \t\n\r\u{b}\u{c}", 1),
    ];
    for (which, chars, min) in cases {
        let mut s = empty_semantics();
        let body = token(
            &mut s,
            "A",
            "X",
            TokenSpec::Builtin {
                which,
                code_points: vec![],
            },
        );
        rule(&mut s, "A", body.clone());
        let (a, p) = analyze(&s).unwrap();
        check_properties(&s, &a, &p);
        let f = fact(&a, &body);
        assert_eq!(f.min_consume, min);
        assert!(!f.may_start.eof);
        for cp in 0..128 {
            assert_eq!(
                contains(&f.may_start, cp),
                chars.chars().any(|c| c as u64 == cp),
                "{which:?}: {cp}"
            );
        }
        assert!(!contains(&f.may_start, 0xe9));
    }
    let mut s = empty_semantics();
    let body = lit(&mut s, "A", "😀é");
    rule(&mut s, "A", body.clone());
    assert_eq!(fact(&analyze(&s).unwrap().0, &body).min_consume, 2);
}

#[test]
fn normalization_record_excludes_saved_original_from_left_corner_graph() {
    let mut s = empty_semantics();
    let old = call(&mut s, "A", "A");
    let body = lit(&mut s, "A", "x");
    rule(&mut s, "A", body.clone());
    s.normalizations.push(Rewrite {
        rule_id: "A".into(),
        kind: RewriteKind,
        original_body: old.clone(),
        rewritten_body: body.clone(),
        proof: Proof {
            kind: ProofKind::LeftRecursionNormalization,
            expr_ids: vec![old, body],
            explanation: "lowering の正規化記録".into(),
        },
    });
    let (a, _) = analyze(&s).unwrap();
    assert_eq!(
        a.rules[0].left_recursion,
        RuleAnalysisLeftRecursion::DirectRewritten
    );
    assert!(!a.sccs[0].recursive);
}

#[test]
fn repeat_cursor_progress_and_zero_max_are_conservative() {
    let mut s = empty_semantics();
    let empty = expr(&mut s, "A", ExprNode::Empty {});
    let repeat = expr(
        &mut s,
        "A",
        ExprNode::Repeat {
            child: empty,
            min: 0,
            max: Some(0),
            terminator: None,
        },
    );
    rule(&mut s, "A", repeat.clone());
    let (a, p) = analyze(&s).unwrap();
    check_properties(&s, &a, &p);
    assert_eq!(
        fact(&a, &repeat).may_start.zero_consume_condition,
        MayStartZeroConsumeCondition::Predicate
    );
    assert_eq!(
        fact(&a, &repeat).cursor_transition.consumed_mode,
        CursorProgress::Preserve
    );
    let mut s = empty_semantics();
    let a = lit(&mut s, "A", "a");
    let repeat = expr(
        &mut s,
        "A",
        ExprNode::Repeat {
            child: a.clone(),
            min: 0,
            max: None,
            terminator: None,
        },
    );
    rule(&mut s, "A", repeat);
    let (_, p) = analyze(&s).unwrap();
    assert_eq!(
        p.memo
            .entries
            .iter()
            .find(|e| e.expr_id == a)
            .unwrap()
            .omission_proof
            .as_ref()
            .unwrap()
            .kind,
        ProofKind::MonotoneNoRollback
    );
}

#[test]
fn malformed_semantics_is_rejected_without_panicking() {
    let mut s = empty_semantics();
    let body = call(&mut s, "A", "MISSING");
    rule(&mut s, "A", body);
    assert!(matches!(
        analyze(&s).unwrap_err()[0],
        AnalysisError::InvalidSemantics { .. }
    ));
    let mut s = empty_semantics();
    let body = lit(&mut s, "A", "x");
    rule(&mut s, "A", body.clone());
    s.expressions[0].node = ExprNode::Group { child: body };
    assert!(analyze(&s).is_err());
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OpTree {
    Atom {
        start: usize,
        end: usize,
        nested: Option<Box<OpTree>>,
    },
    Tier {
        level: usize,
        start: usize,
        end: usize,
        left: Box<OpTree>,
        ops: Vec<u8>,
        right: Vec<OpTree>,
    },
}
impl OpTree {
    fn span(&self) -> (usize, usize) {
        match self {
            Self::Atom { start, end, .. } | Self::Tier { start, end, .. } => (*start, *end),
        }
    }
}
fn wrap(tree: OpTree, level: usize) -> OpTree {
    let (start, end) = tree.span();
    OpTree::Tier {
        level,
        start,
        end,
        left: Box::new(tree),
        ops: vec![],
        right: vec![],
    }
}
fn append(tree: &mut OpTree, op: u8, rhs: OpTree) {
    if let OpTree::Tier {
        end, ops, right, ..
    } = tree
    {
        *end = rhs.span().1;
        ops.push(op);
        right.push(rhs);
    } else {
        panic!("段ノードが必要");
    }
}
fn oracle_atom(input: &[u8], pos: usize, assocs: &[bool; 2], climbing: bool) -> Option<OpTree> {
    match input.get(pos) {
        Some(b'x') => Some(OpTree::Atom {
            start: pos,
            end: pos + 1,
            nested: None,
        }),
        Some(b'(') => {
            let inner = if climbing {
                oracle_climb(input, pos + 1, 0, assocs)?
            } else {
                oracle_tier(input, pos + 1, 0, assocs)?
            };
            let end = inner.span().1;
            (input.get(end) == Some(&b')')).then(|| OpTree::Atom {
                start: pos,
                end: end + 1,
                nested: Some(Box::new(inner)),
            })
        }
        _ => None,
    }
}
// 参照側は段別の PEG。右結合の第1候補失敗では base を初めから再試行する。
fn oracle_tier(input: &[u8], pos: usize, level: usize, assocs: &[bool; 2]) -> Option<OpTree> {
    if level == 2 {
        return oracle_atom(input, pos, assocs, false);
    }
    let base = oracle_tier(input, pos, level + 1, assocs)?;
    let operator = b"+*"[level];
    let mut tree = wrap(base, level);
    loop {
        let end = tree.span().1;
        if input.get(end) != Some(&operator) {
            break;
        }
        let rhs = oracle_tier(
            input,
            end + 1,
            if assocs[level] { level } else { level + 1 },
            assocs,
        );
        if let Some(rhs) = rhs {
            append(&mut tree, operator, rhs);
        } else {
            if assocs[level] {
                return oracle_tier(input, pos, level + 1, assocs).map(|base| wrap(base, level));
            }
            break;
        }
        if assocs[level] {
            break;
        }
    }
    Some(tree)
}
// D-004 の lift と ceiling を備えた独立した climbing 評価器。
fn oracle_climb(input: &[u8], pos: usize, min: usize, assocs: &[bool; 2]) -> Option<OpTree> {
    let mut x = oracle_atom(input, pos, assocs, true)?;
    let mut tier = 2;
    let mut ceiling = 1isize;
    loop {
        let end = x.span().1;
        let k = match input.get(end) {
            Some(b'+') => 0,
            Some(b'*') => 1,
            _ => break,
        };
        if k < min || k as isize > ceiling {
            break;
        }
        let Some(y) = oracle_climb(input, end + 1, if assocs[k] { k } else { k + 1 }, assocs)
        else {
            break;
        };
        for level in (k..tier).rev() {
            x = wrap(x, level);
        }
        append(&mut x, input[end], y);
        tier = k;
        ceiling = if assocs[k] {
            k as isize - 1
        } else {
            k as isize
        };
    }
    for level in (min..tier).rev() {
        x = wrap(x, level);
    }
    Some(x)
}
fn operator_grammar(assocs: [bool; 2]) -> Semantics {
    let mut s = empty_semantics();
    for (level, name) in ["E", "T"].into_iter().enumerate() {
        let base_name = ["T", "Atom"][level];
        let base = call(&mut s, name, base_name);
        let op = lit(&mut s, name, ["+", "*"][level]);
        let rhs = call(&mut s, name, if assocs[level] { name } else { base_name });
        let body = if assocs[level] {
            let recursive = seq(&mut s, name, vec![base, op, rhs]);
            let fallback = call(&mut s, name, base_name);
            choice(&mut s, name, vec![recursive, fallback])
        } else {
            let pair = seq(&mut s, name, vec![op, rhs]);
            let repeat = expr(
                &mut s,
                name,
                ExprNode::Repeat {
                    child: pair,
                    min: 0,
                    max: None,
                    terminator: None,
                },
            );
            seq(&mut s, name, vec![base, repeat])
        };
        rule(&mut s, name, body);
        s.rules.last_mut().unwrap().annotations = vec![
            Annotation {
                name: if assocs[level] {
                    "rightAssoc"
                } else {
                    "leftAssoc"
                }
                .into(),
                arguments: Default::default(),
                source: span(),
            },
            Annotation {
                name: "precedence".into(),
                arguments: [(
                    "level".into(),
                    JsonValue::Number(serde_json::Number::from(100 - level as i64)),
                )]
                .into_iter()
                .collect(),
                source: span(),
            },
        ];
    }
    let x = lit(&mut s, "Atom", "x");
    let open = lit(&mut s, "Atom", "(");
    let inner = call(&mut s, "Atom", "E");
    let close = lit(&mut s, "Atom", ")");
    let parens = seq(&mut s, "Atom", vec![open, inner, close]);
    let body = choice(&mut s, "Atom", vec![x, parens]);
    rule(&mut s, "Atom", body);
    s
}

#[test]
fn climbing_matches_tier_ast_spans_and_consumption_on_complete_and_incomplete_inputs() {
    let alphabet = b"x+*()";
    for left in [false, true] {
        for right in [false, true] {
            let assocs = [left, right];
            let s = operator_grammar(assocs);
            let (_, p) = analyze(&s).unwrap();
            assert_eq!(p.operators.len(), 1);
            assert_eq!(
                p.operators[0].strategy,
                OperatorPlanStrategy::PrecedenceClimbing
            );
            assert_eq!(p.operators[0].rule_ids, vec!["E", "T"]); // precedence の数値は逆順でも参照順を守る。
            for len in 0..=6 {
                for mut code in 0..5usize.pow(len) {
                    let mut input = vec![0; len as usize];
                    for c in &mut input {
                        *c = alphabet[code % 5];
                        code /= 5;
                    }
                    assert_eq!(
                        oracle_tier(&input, 0, 0, &assocs),
                        oracle_climb(&input, 0, 0, &assocs),
                        "{assocs:?}: {}",
                        String::from_utf8_lossy(&input)
                    );
                }
            }
            for input in [
                b"x*x+x*x+x".as_slice(),
                b"(x+x)*(x*x)",
                b"x*(x+)",
                b"x*x*+x",
                b"((x+x)*x)+x",
            ] {
                assert_eq!(
                    oracle_tier(input, 0, 0, &assocs),
                    oracle_climb(input, 0, 0, &assocs)
                );
            }
        }
    }
}

#[test]
fn recovery_follow_crosses_nullable_separated_elements() {
    let mut s = empty_semantics();
    let comma = lit(&mut s, "B", ",");
    rule(&mut s, "B", comma);
    let x = lit(&mut s, "S", "x");
    let element = expr(&mut s, "S", ExprNode::Optional { child: x });
    let separator = call(&mut s, "S", "B");
    let list = expr(
        &mut s,
        "S",
        ExprNode::Separated {
            element,
            separator: separator.clone(),
            min: 1,
            pair_rollback: ExprNodeSeparatedPairRollback,
        },
    );
    let end = lit(&mut s, "S", "end");
    let body = seq(&mut s, "S", vec![list, end]);
    rule(&mut s, "S", body);
    s.recovery.push(auto_recovery("B", vec![separator]));
    let (a, _) = analyze(&s).unwrap();
    assert_eq!(
        a.rules[0].follow_sync_patterns,
        Some(vec![",".into(), "end".into(), "x".into()])
    );
}

#[test]
fn operator_plan_rejects_scope_recovery_lookahead_and_ambiguous_operators() {
    for obstacle in 0..4 {
        let mut s = operator_grammar([false, false]);
        match obstacle {
            0 => s.scope.effects.push(ScopeEffect {
                rule_id: "Atom".into(),
                mode: ScopeEffectMode::Lexical,
                events: vec![ScopeEvent {
                    action: ScopeEffectEventsAction::Use,
                    when: ScopeEffectEventsWhen::AfterDeclarations,
                    capture_site_id: None,
                    name: None,
                }],
            }),
            1 => s.recovery.push(auto_recovery("Atom", vec![])),
            2 => {
                let atom = s
                    .rules
                    .iter()
                    .find(|r| r.symbol_id == "Atom")
                    .unwrap()
                    .body
                    .clone();
                let guard = expr(
                    &mut s,
                    "Atom",
                    ExprNode::Lookahead {
                        child: atom.clone(),
                    },
                );
                let body = seq(&mut s, "Atom", vec![guard, atom]);
                s.rules
                    .iter_mut()
                    .find(|r| r.symbol_id == "Atom")
                    .unwrap()
                    .body = body;
            }
            _ => {
                for e in &mut s.expressions {
                    if let ExprNode::Literal { value, .. } = &mut e.node {
                        if value == "*" {
                            *value = "+".into();
                        }
                    }
                }
            }
        }
        let (_, p) = analyze(&s).unwrap();
        assert!(p
            .operators
            .iter()
            .all(|p| p.strategy == OperatorPlanStrategy::TierFunctions));
    }
}
