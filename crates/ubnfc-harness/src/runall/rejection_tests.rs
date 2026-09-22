use super::*;
use serde_json::json;

#[test]
fn only_the_28_documented_rejections_are_registered_for_both_backends() {
    let files = crate::load_all(crate::default_root()).unwrap();
    let originals = files.clone();
    for backend in [Backend::Java, Backend::Rust] {
        let plan = plan(&files, &[], backend).unwrap();
        let mut counts = BTreeMap::new();
        for work in &plan.work {
            for (case, row) in work.cases.iter().zip(&work.rows) {
                assert_eq!(case.case.expect, row.expected);
                let Expectation::ExpectedRejection {
                    stage,
                    code,
                    evidence,
                    original,
                } = &row.expected
                else {
                    continue;
                };
                *counts.entry((stage.as_str(), code.as_str())).or_insert(0) += 1;
                match code.as_str() {
                    "E-PRECEDENCE-ORDER" => {
                        assert_eq!(row.corpus, "associative/corpus.json");
                        assert_eq!(row.variant, "reversed-metadata");
                        assert!(matches!(**original, Expectation::RuleValue(_)));
                    }
                    "E-LOWER-COMMON-FIELD" => {
                        assert!(matches!(
                            row.case_id.as_str(),
                            "common-field-missing/1" | "common-field-type/1"
                        ));
                        assert!(matches!(**original, Expectation::GenerationFailure { .. }));
                    }
                    "E-LOWER-SUM-EVAL" => {
                        assert_eq!(row.case_id, "eval-sum/1");
                        assert!(matches!(**original, Expectation::Oracle { .. }));
                    }
                    "E-IMPORT-CYCLE" => {
                        assert_eq!(row.case_id, "import-cycle/1");
                        assert!(matches!(**original, Expectation::GenerationFailure { .. }));
                    }
                    other => panic!("未登録の拒否: {other}"),
                }
                assert!(!evidence.is_empty());
                if code != "E-IMPORT-CYCLE" {
                    assert!(evidence.contains("docs/reports/2026-09-22-triage.md"));
                }
            }
        }
        assert_eq!(
            counts,
            BTreeMap::from([
                (("validate", "E-PRECEDENCE-ORDER"), 24),
                (("ir", "E-LOWER-COMMON-FIELD"), 2),
                (("ir", "E-LOWER-SUM-EVAL"), 1),
                (("ir", "E-IMPORT-CYCLE"), 1),
            ])
        );
    }
    assert_eq!(files, originals);
}

#[test]
fn expected_rejections_pass_but_wrong_codes_stages_and_runtime_fail() {
    let files = crate::load_all(crate::default_root()).unwrap();
    let plan = plan(&files, &[], Backend::Java).unwrap();
    let mut builds = Vec::new();
    let mut passed = Vec::new();
    for work in plan.work.iter().filter(|work| {
        work.rows
            .iter()
            .any(|row| matches!(row.expected, Expectation::ExpectedRejection { .. }))
    }) {
        let Expectation::ExpectedRejection { stage, code, .. } = &work.rows[0].expected else {
            panic!()
        };
        let mut build = work.build.clone();
        build.ir = StageResult::failed("ir", format!("[code: {code}]"));
        assert_eq!(
            rejection_verdict(stage, code, stage, &build.ir),
            Verdict::Pass
        );
        let (kept, rows) = stopped(
            build.clone(),
            work.rows.clone(),
            Outcome::IrFailure,
            Path::new("/unused"),
        );
        assert_eq!(kept.ir.status, StageStatus::Failed);
        assert_eq!(kept.ir.reasons.as_slice(), std::slice::from_ref(code));
        assert!(rows.iter().all(|row| {
            row.outcome == Outcome::Pass && row.differences.is_empty() && row.missing.is_empty()
        }));
        builds.push(kept);
        passed.extend(rows);

        for output in [
            "[code: E-UNRELATED]".to_string(),
            format!("[code: {code}]\n[code: E-UNRELATED]"),
        ] {
            build.ir = StageResult::failed("ir", output);
            assert!(matches!(
                rejection_verdict(stage, code, stage, &build.ir),
                Verdict::Fail(_)
            ));
            let (_, rows) = stopped(
                build.clone(),
                work.rows.clone(),
                Outcome::IrFailure,
                Path::new("/unused"),
            );
            assert!(rows.iter().all(|row| row.outcome == Outcome::Fail));
            assert_eq!(rows[0].differences[0].path, "/rejection");
        }
        build.generation = StageResult::failed("generation", code.clone());
        let (_, rows) = stopped(
            build,
            work.rows.clone(),
            Outcome::GenerationRejected,
            Path::new("/unused"),
        );
        assert!(rows.iter().all(|row| row.outcome == Outcome::Fail));
        assert!(matches!(
            rejection_verdict(stage, code, stage, &StageResult::success()),
            Verdict::Fail(_)
        ));
        let mut row = work.rows[0].clone();
        row.observe(
            &work.cases[0],
            Actual {
                backend: Backend::Java,
                variant: row.variant.clone(),
                response: json!({"allConsumed": true}),
            },
        );
        assert_eq!(row.outcome, Outcome::Fail);
    }
    let report = Report::new(Backend::Java, 1, 5, 5, builds, passed);
    assert_eq!(report.totals.expected_rejection_pass, 28);
    assert_eq!(report.totals.pass, 0);
    assert_eq!(report.totals.ir_failure, 0);
    assert_eq!(report.totals.total(), 28);
    assert_eq!(report.families["associative"].expected_rejection_pass, 24);
    assert_eq!(report.families["oracle"].expected_rejection_pass, 4);
    for groups in [&report.families, &report.kinds, &report.grammars] {
        assert_eq!(groups.values().map(Counts::total).sum::<usize>(), 28);
    }
    assert_eq!(report.refusals["ir/E-PRECEDENCE-ORDER"].executions, 24);
    let json: Value = serde_json::from_str(&report.json()).unwrap();
    assert_eq!(json["totals"]["expected_rejection_pass"], 28);
    assert!(report.markdown().contains("| 期待拒否 Pass |"));
    assert!(report
        .markdown()
        .contains("| 合計 | 0 | 0 | 0 | 0 | 0 | 28 | 0 | 0 | 0 |"));
}
