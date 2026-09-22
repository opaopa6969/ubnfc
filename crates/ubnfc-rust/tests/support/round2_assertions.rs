#[cfg(test)]
mod contracts {
    use super::*;
    use serde_json::json;
    #[test]
    fn scope_completion_and_trimmed_cp_positions() {
        let r = quantified_capture_occurrences::parse("aa|bb|c,c");
        assert!(r.ok);
        assert_eq!(
            r.declarations
                .iter()
                .map(|d| (d.name.as_str(), d.offset_cp))
                .collect::<Vec<_>>(),
            vec![("a", 0), ("a", 1), ("b", 3), ("b", 4), ("c", 6), ("c", 8)]
        );
        let r = repeated_capture_occurrences::parse("a,b;a,b,missing");
        assert!(r.ok);
        assert!(r.references.iter().any(|r| r.name == "b" && r.offset == 6));
        let r = trimmed_group_non_bmp::parse(">  😀x  <? 😀x ");
        assert!(r.ok);
        assert_eq!(r.declarations[0].offset_cp, 3);
        assert_eq!((r.references[0].offset, r.references[0].len), (10, 2));
        let r = nested_same_name_captures::parse("ab");
        assert!(r.ok);
        assert_eq!(
            r.declarations
                .iter()
                .map(|d| d.name.as_str())
                .collect::<Vec<_>>(),
            ["a", "ab"]
        );
    }
    #[test]
    fn lexical_tokens_and_nested_right_span() {
        let r = lexical::parse_with_options(
            "abx",
            lexical::ParseOptions {
                lexical: true,
                ..Default::default()
            },
        );
        assert!(r.ok);
        assert_eq!(r.lexical.len(), 1);
        assert_eq!(r.tokens.len(), 3);
        assert_eq!(
            r.tokens.iter().map(|l| l.span).collect::<Vec<_>>(),
            [[0, 1], [1, 2], [2, 3]]
        );
        assert!(r
            .lexical
            .iter()
            .all(|l| !l.expr_id.is_empty() && !l.rule_id.is_empty()));
        let r = right_associative_local_interleave::parse(" x /*r*/ ^ x ");
        assert!(r.ok);
        let ast = r.ast.unwrap().canonical_value();
        assert_eq!(ast["span"], json!([0, 13]));
        assert_eq!(ast["fields"]["right"][0]["span"], json!([11, 12]));
    }
    #[test]
    fn shared_diagnostic_dag_is_bounded() {
        let start = std::time::Instant::now();
        let r = dag::parse("");
        let elapsed = start.elapsed();
        assert!(!r.ok);
        assert_eq!(r.diagnostics[0].expected, ["x"]);
        assert_eq!(r.diagnostics[0].rule_path.len(), 30);
        assert!(
            elapsed < std::time::Duration::from_millis(50),
            "{elapsed:?}"
        );
    }
    #[test]
    fn resource_limits_fail_without_partial_success() {
        use lexical::*;
        for limit in 0..4 {
            let mut options = ParseOptions::default();
            match limit {
                0 => options.limits.input_bytes = 2,
                1 => options.limits.arena_entries = 2,
                2 => options.limits.memo_entries = 0,
                _ => options.limits.mapping_depth = 0,
            }
            let r = parse_with_options("abx", options);
            if limit == 3 {
                assert!(r.ok);
                assert!(r.ast.is_none());
                assert_eq!(r.diagnostics[0].kind, DiagnosticKind::Mapping);
            } else {
                assert!(!r.ok);
                assert_eq!(r.diagnostics[0].kind, DiagnosticKind::Resource);
                assert_eq!((r.consumed_cp, r.matched_cp), (0, 0));
            }
        }
        let r = parse_with_options(
            "bad",
            ParseOptions {
                limits: ResourceLimits {
                    diagnostics: 0,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        assert!(!r.ok);
        assert_eq!(r.diagnostics[0].kind, DiagnosticKind::Resource);
        let r = quantified_capture_occurrences::parse_with_options(
            "aa|bb|c,c",
            quantified_capture_occurrences::ParseOptions {
                limits: quantified_capture_occurrences::ResourceLimits {
                    effect_entries: 2,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        assert!(!r.ok);
        assert!(r.declarations.is_empty());
    }
    #[test]
    fn catalog_typeof_error_ci_and_enum() {
        let c = catalog::metadata::CATALOGS;
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].context, " variable ");
        assert_eq!(c[0].captures, ["name"]);
        assert_eq!(c[0].site_ids.len(), 2);
        for input in ["a:a", "b:b"] {
            let r = typeof_::parse(input);
            assert!(r.ok);
            assert!(r.ast.is_some());
        }
        for input in ["a:b", "b:a"] {
            let r = typeof_::parse(input);
            assert!(r.ok);
            assert!(r.ast.is_none());
            assert!(r
                .diagnostics
                .iter()
                .any(|d| d.kind == typeof_::DiagnosticKind::Mapping));
        }
        for input in ["x", ""] {
            let r = error::parse(input);
            assert!(!r.ok);
            assert_eq!(r.diagnostics[0].offset_cp, 0);
            assert_eq!(
                r.diagnostics[0]
                    .expected
                    .iter()
                    .filter(|&&e| e == "need a")
                    .count(),
                1
            );
        }
        assert!(error::parse("a").ok);
        for input in ["é", "É"] {
            let r = ci_unicode::parse(input);
            assert!(r.ok);
            assert_eq!((r.consumed_cp, r.matched_cp), (1, 1));
        }
        assert!(!ci_unicode::parse("e").ok);
        for input in ["on", "ON"] {
            let r = enum_collision::parse(input);
            assert!(r.ok);
            assert!(r.ast.is_some());
        }
    }
    #[test]
    fn recovery_projection_and_diagnostics() {
        for memo in [false, true] {
            let r = recovery_sync::parse_with_options(
                "bad;ok;",
                recovery_sync::ParseOptions {
                    memo,
                    ..Default::default()
                },
            );
            assert!(r.ok);
            assert_eq!((r.consumed_cp, r.matched_cp), (7, 7));
            assert_eq!(r.recoveries.len(), 1);
            assert_eq!(r.recoveries[0].span, [0, 4]);
            assert_eq!(r.recoveries[0].sync_span, Some([3, 4]));
            assert_eq!(r.recoveries[0].capture_occurrence_ids, [0]);
            assert_eq!(
                r.ast.unwrap().canonical_value()["fields"]["items"],
                json!([{"type":"Stmt","span":[4,7],"fields":{"value":"ok"}}])
            );
            assert_eq!(r.diagnostics.len(), 1);
            assert_eq!(
                r.diagnostics[0].kind,
                recovery_sync::DiagnosticKind::Recovery
            );
            assert_eq!(
                (r.diagnostics[0].offset_cp, r.diagnostics[0].length_cp),
                (0, 4)
            );
        }
        let r = recovery_auto::parse("bad;ok;");
        assert!(r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (7, 7));
        assert_eq!(r.recoveries[0].span, [0, 4]);
        let r = recovery_root::parse("bad;");
        assert!(r.ok);
        assert!(r.ast.is_none());
        assert!(r.captures.is_empty());
        assert_eq!(r.recoveries[0].span, [0, 4]);
        assert_eq!(r.diagnostics.len(), 1);
        assert!(!recovery_sync::parse("bad").ok);
        assert!(!recovery_root::parse("").ok);
        let r = recovery_sync::parse("😀;x");
        assert!(!r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (2, 2));
        assert_eq!(r.captures.len(), 1);
        assert_eq!(r.captures[0].span, [0, 2]);
        assert_eq!(r.recoveries[0].span, [0, 2]);
        assert_eq!(r.diagnostics.len(), 2);
        assert_eq!(
            r.diagnostics[0].kind,
            recovery_sync::DiagnosticKind::Recovery
        );
        assert_eq!(
            r.diagnostics[1].kind,
            recovery_sync::DiagnosticKind::TrailingInput
        );
        let r = scalar::parse("bad;");
        assert!(r.ok);
        assert!(r.ast.is_none());
        assert_eq!(r.diagnostics.len(), 1);
        let r = optional::parse("bad;");
        assert!(r.ok);
        assert_eq!(
            r.ast.unwrap().canonical_value()["fields"]["item"],
            json!(null)
        );
        let r = rollback::parse("bad;");
        assert!(r.ok);
        assert!(r.recoveries.is_empty());
        let r = memo_recovery::parse("bad;");
        assert!(r.ok);
        assert_eq!(r.recoveries.len(), 1);
        let r = recovery_skip::parse("bad;ok;");
        assert!(r.ok);
        assert_eq!(r.consumed_cp, 7);
        assert!(r.recoveries.iter().all(|r| r.span[1] > r.span[0]));
    }
    #[test]
    fn evaluator_automatic_and_manual_callbacks() {
        use eval_literal::evaluator::*;
        let ast = eval_literal::parse("7").ast.unwrap();
        assert_eq!(
            evaluate(&ast, &mut DefaultEvaluator).unwrap(),
            Value::Text("7".into())
        );
        let ast = eval_passthrough::parse("7").ast.unwrap();
        assert_eq!(
            eval_passthrough::evaluator::evaluate(
                &ast,
                &mut eval_passthrough::evaluator::DefaultEvaluator
            )
            .unwrap(),
            eval_passthrough::evaluator::Value::Text("7".into())
        );
        let ast = eval_manual::parse("7").ast.unwrap();
        assert!(eval_manual::evaluator::evaluate(
            &ast,
            &mut eval_manual::evaluator::DefaultEvaluator
        )
        .is_err());
        let ast = eval_unknown::parse("7").ast.unwrap();
        assert!(eval_unknown::evaluator::evaluate(
            &ast,
            &mut eval_unknown::evaluator::DefaultEvaluator
        )
        .is_err());
        let ast = eval_conditional::parse("0?2:3").ast.unwrap();
        assert_eq!(
            eval_conditional::evaluator::evaluate(
                &ast,
                &mut eval_conditional::evaluator::DefaultEvaluator
            )
            .unwrap(),
            eval_conditional::evaluator::Value::Text("3".into())
        );
    }
    #[test]
    fn evaluator_dispatches_semantic_overrides() {
        use eval_binary_arithmetic::evaluator::{EvalError, EvalResult, Evaluator, Value};
        struct Calc;
        impl Evaluator for Calc {
            fn eval_leaf(&mut self, text: &str) -> EvalResult {
                Ok(Value::Number(text.parse().unwrap()))
            }
            fn apply_binary(&mut self, op: &str, left: Value, right: Value) -> EvalResult {
                let number = |v: Value| match v {
                    Value::Text(s) => s.parse::<f64>().unwrap(),
                    Value::Number(n) => n,
                    _ => panic!("number"),
                };
                if op == "+" {
                    Ok(Value::Number(number(left) + number(right)))
                } else {
                    Err(EvalError("unknown op".into()))
                }
            }
        }
        let ast = eval_binary_arithmetic::parse("1+2+3").ast.unwrap();
        assert_eq!(Calc.eval(&ast).unwrap(), Value::Number(6.0));
        struct Vars;
        impl eval_variable_ref::evaluator::Evaluator for Vars {
            fn resolve_variable(&mut self, name: &str) -> eval_variable_ref::evaluator::EvalResult {
                assert_eq!(name, "x");
                Ok(eval_variable_ref::evaluator::Value::Number(42.0))
            }
        }
        let ast = eval_variable_ref::parse("x").ast.unwrap();
        assert_eq!(
            eval_variable_ref::evaluator::evaluate(&ast, &mut Vars).unwrap(),
            eval_variable_ref::evaluator::Value::Number(42.0)
        );
        struct Manual;
        impl eval_manual::evaluator::Evaluator for Manual {
            fn eval_g_ProbeAST_2e_Literal(
                &mut self,
                node: &eval_manual::ast::g_ProbeAST_2e_Literal,
            ) -> eval_manual::evaluator::EvalResult {
                Ok(eval_manual::evaluator::Value::Text(format!(
                    "manual:{}",
                    node.g_value
                )))
            }
        }
        let ast = eval_manual::parse("7").ast.unwrap();
        assert_eq!(
            eval_manual::evaluator::evaluate(&ast, &mut Manual).unwrap(),
            eval_manual::evaluator::Value::Text("manual:7".into())
        );
    }
    #[test]
    fn dfa_uses_longest_code_point_ranges_and_p4_catalog_builds() {
        let r = dfa::parse("😀😀");
        assert!(r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (2, 2));
        let r = dfa::parse("😀a");
        assert!(!r.ok);
        assert_eq!((r.consumed_cp, r.matched_cp), (1, 1));
        assert!(!dfa::parse("").ok);
        assert!(!p4::metadata::CATALOGS.is_empty());
    }
}
