use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    process::Command,
    sync::OnceLock,
};
use ubnfc_harness::{
    corpus::oracle,
    divergence::{self, compare, Decision, Observation, Verdict},
    CorpusFile, Expectation, Family, GrammarSource,
};

fn corpus() -> &'static [CorpusFile] {
    static CORPUS: OnceLock<Vec<CorpusFile>> = OnceLock::new();
    CORPUS.get_or_init(|| oracle::load_all(oracle::default_root()).unwrap())
}

#[test]
fn all_63_variants_and_168_cases_keep_their_measured_counts_and_stages() {
    let expected = [
        ("backref", 4),
        ("catalog", 3),
        ("char-range", 5),
        ("ci-unicode", 3),
        ("ci", 4),
        ("comment-block", 3),
        ("comment-line", 3),
        ("comment-whitespace", 3),
        ("common-field-missing", 1),
        ("common-field-type", 1),
        ("common-field", 3),
        ("digit", 4),
        ("empty-one-two", 1),
        ("empty-two", 1),
        ("empty-zero", 1),
        ("empty", 3),
        ("enum-collision", 1),
        ("enum-plain", 3),
        ("enum", 3),
        ("eof", 3),
        ("error", 3),
        ("eval-binary_arithmetic", 1),
        ("eval-conditional", 1),
        ("eval-literal", 1),
        ("eval-manual", 1),
        ("eval-passthrough", 1),
        ("eval-sum", 1),
        ("eval-unknown", 1),
        ("eval-variable_ref", 1),
        ("extern-full", 3),
        ("extern", 4),
        ("import-alias", 3),
        ("import-cycle", 1),
        ("import-two-level", 3),
        ("longest", 4),
        ("lookahead-failure", 1),
        ("lookahead-until", 1),
        ("lookahead", 4),
        ("mapping-alias", 3),
        ("mapping-dotted", 3),
        ("mapping-sum", 3),
        ("negation-empty", 3),
        ("negative-lookahead", 3),
        ("numeric", 7),
        ("posix", 4),
        ("predictive-extern", 2),
        ("predictive-nullable", 3),
        ("predictive", 5),
        ("recovery-auto", 2),
        ("recovery-root", 3),
        ("recovery-skip", 2),
        ("recovery-sync", 3),
        ("regex-ascii", 2),
        ("regex-backref", 1),
        ("regex-emoji", 2),
        ("regex-prefix", 2),
        ("regex-zero", 2),
        ("repeat-unbounded", 4),
        ("repeat-zero", 2),
        ("separated", 5),
        ("skip", 4),
        ("typeof", 5),
        ("until", 5),
    ]
    .into_iter()
    .collect::<BTreeMap<_, _>>();
    let actual = corpus()
        .iter()
        .map(|f| (f.path.file_stem().unwrap().to_str().unwrap(), f.cases.len()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, expected);
    assert_eq!(corpus().len(), 63);
    assert_eq!(actual.values().sum::<usize>(), 168);
    assert!(corpus().windows(2).all(|f| f[0].path < f[1].path));

    let mut stages = BTreeMap::<&str, (usize, usize)>::new();
    let mut outcomes = [0; 4];
    for file in corpus() {
        let stage = file.raw["stage"].as_str().unwrap();
        let count = stages.entry(stage).or_default();
        count.0 += 1;
        count.1 += file.cases.len();
        for case in &file.cases {
            let expect = match &case.expect {
                Expectation::ExpectedRejection {
                    original,
                    stage,
                    code,
                    ..
                } => {
                    assert_eq!(case.id, "import-cycle/1");
                    assert_eq!((stage.as_str(), code.as_str()), ("ir", "E-IMPORT-CYCLE"));
                    original.as_ref()
                }
                other => other,
            };
            match expect {
                Expectation::Oracle {
                    accepted: Some(_), ..
                } => outcomes[0] += 1,
                Expectation::Oracle { accepted: None, .. } => {
                    assert_eq!(case.id, "predictive/5");
                    outcomes[1] += 1;
                }
                Expectation::GenerationFailure { message } => {
                    assert!(message.starts_with(stage));
                    outcomes[if stage == "compilation_failed" { 2 } else { 3 }] += 1;
                }
                _ => panic!("oracle の variant が不正"),
            }
        }
    }
    assert_eq!(
        stages,
        BTreeMap::from([
            ("executed", (55, 153)),
            ("compilation_failed", (7, 14)),
            ("generation_failed", (1, 1)),
        ])
    );
    assert_eq!(outcomes, [152, 1, 14, 1]);
}

#[test]
fn observations_imports_errors_and_source_revision_survive_normalization() {
    for file in corpus() {
        assert_eq!(file.family, Family::Oracle);
        let raw: Value = serde_json::from_str(&fs::read_to_string(&file.path).unwrap()).unwrap();
        assert_eq!(file.raw, raw);
        assert_eq!(file.grammars.len(), 1);
        let GrammarSource::Inline {
            name,
            source,
            metadata,
        } = &file.grammars[0]
        else {
            panic!()
        };
        assert_eq!(name, raw["feature"].as_str().unwrap());
        assert_eq!(source, raw["grammar"].as_str().unwrap());
        assert_eq!(metadata["source"], raw["source"]);
        assert_eq!(metadata.get("imports"), raw.get("imports"));
        for (case, row) in file.cases.iter().zip(raw["cases"].as_array().unwrap()) {
            assert_eq!(case.id, row["id"]);
            assert_eq!(case.input, row["input"]);
            assert_eq!(case.grammar_ref, [0]);
            let expect = match &case.expect {
                Expectation::ExpectedRejection {
                    original,
                    stage,
                    code,
                    ..
                } => {
                    assert_eq!(case.id, "import-cycle/1");
                    assert_eq!((stage.as_str(), code.as_str()), ("ir", "E-IMPORT-CYCLE"));
                    original.as_ref()
                }
                other => other,
            };
            match expect {
                Expectation::Oracle { .. } => {
                    assert_eq!(raw["stage"], "executed");
                    let encoded = serde_json::to_value(&case.expect).unwrap();
                    for field in [
                        "accepted",
                        "allConsumed",
                        "consumed",
                        "matched",
                        "ast",
                        "captures",
                        "declarations",
                        "diagnostics",
                        "mappingError",
                    ] {
                        assert_eq!(encoded["Oracle"][field], row[field], "{}: {field}", case.id);
                    }
                }
                Expectation::GenerationFailure { message } => {
                    assert!(row["accepted"].is_null());
                    let details: Value =
                        serde_json::from_str(message.split_once(": ").unwrap().1).unwrap();
                    let original = if raw["stage"] == "compilation_failed" {
                        &raw["compilation"]["diagnostics"]
                    } else {
                        &raw["generation"]
                    };
                    assert_eq!(&details, original);
                }
                _ => panic!(),
            }
        }
        let restored: CorpusFile =
            serde_json::from_value(serde_json::to_value(file).unwrap()).unwrap();
        assert_eq!(file, &restored);
    }
}

#[test]
fn every_divergence_resolves_and_only_follow_spec_is_attached() {
    let entries = divergence::load(&oracle::default_root().join("divergences.json")).unwrap();
    assert_eq!(entries.len(), 75);
    assert_eq!(
        entries
            .iter()
            .filter(|e| e.decision == Decision::FollowSpec)
            .count(),
        60
    );
    // D-018 で undecided 15 件は名前付き契約（new-contract:*）に決定した。
    assert_eq!(
        entries
            .iter()
            .filter(|e| matches!(e.decision, Decision::NewContract(_)))
            .count(),
        15
    );
    assert_eq!(
        entries
            .iter()
            .filter(|e| e.decision == Decision::Undecided)
            .count(),
        0
    );
    let mut seen = BTreeSet::new();
    let mut case_ids = BTreeSet::new();
    for entry in &entries {
        assert!(seen.insert((&entry.file, &entry.case_id, entry.observation)));
        case_ids.insert((&entry.file, &entry.case_id));
        assert!(entry.reason.starts_with("README.md#疑義 "));
        let file = corpus()
            .iter()
            .find(|f| f.path.file_name().unwrap() == entry.file.as_str())
            .unwrap();
        let case = file.cases.iter().find(|c| c.id == entry.case_id).unwrap();
        assert_eq!(
            case.known_divergences.contains(&entry.observation),
            entry.decision.is_resolved()
        );
    }
    assert_eq!(case_ids.len(), 29);
    let attached: usize = corpus()
        .iter()
        .flat_map(|f| &f.cases)
        .map(|c| c.known_divergences.len())
        .sum();
    assert_eq!(attached, 75);

    // 登録されていない観測（recovery-sync/1 の consumed）を変えると、他に既知登録があっても免除しない。
    let case = corpus()
        .iter()
        .flat_map(|f| &f.cases)
        .find(|c| c.id == "recovery-sync/1")
        .unwrap();
    assert!(!case.known_divergences.contains(&Observation::Consumed));
    let mut actual = case.expect.clone();
    let Expectation::Oracle { consumed, .. } = &mut actual else {
        panic!()
    };
    *consumed = consumed.map(|c| c + 1).or(Some(1));
    assert!(matches!(
        compare(&case.expect, &actual, &case.known_divergences),
        Verdict::Fail(_)
    ));
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let output = Command::new("mktemp").arg("-d").output().unwrap();
        assert!(output.status.success());
        Self(PathBuf::from(
            String::from_utf8(output.stdout).unwrap().trim(),
        ))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn manifest_rejects_bad_references_duplicates_and_typos_without_partial_application() {
    let scratch = Scratch::new();
    let path = scratch.0.join("divergences.json");
    let entry = json!({"file":"predictive.json", "caseId":"predictive/5",
        "observation":"accepted", "reason":"README.md#疑義 10", "decision":"follow-spec"});
    let mut files = corpus().to_vec();
    let before = files.clone();
    for (field, value) in [
        ("file", "missing.json"),
        ("file", "../predictive.json"),
        ("caseId", "predictive/999"),
        ("observation", "acceptd"),
        ("decision", "follow_spce"),
        ("reason", " "),
    ] {
        let mut bad = entry.clone();
        bad[field] = value.into();
        fs::write(&path, json!([entry, bad]).to_string()).unwrap();
        let error = divergence::apply(&path, &mut files).unwrap_err();
        assert_eq!(error.path, path);
        assert_eq!(files, before);
    }
    fs::write(&path, json!([entry, entry]).to_string()).unwrap();
    assert!(divergence::apply(&path, &mut files)
        .unwrap_err()
        .message
        .contains("重複"));
    assert_eq!(files, before);
    fs::write(&path, json!([entry]).to_string()).unwrap();
    divergence::apply(&path, &mut files).unwrap();
    divergence::apply(&path, &mut files).unwrap();
    let cases = files.iter().flat_map(|f| &f.cases).collect::<Vec<_>>();
    assert_eq!(
        cases
            .iter()
            .map(|c| c.known_divergences.len())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        cases
            .iter()
            .find(|c| c.id == "predictive/5")
            .unwrap()
            .known_divergences,
        [Observation::Accepted]
    );
}

#[test]
fn oracle_loader_reports_invalid_json_types_stage_and_duplicate_ids() {
    let scratch = Scratch::new();
    let path = scratch.0.join("probe.json");
    let original: Value = serde_json::from_str(
        &fs::read_to_string(oracle::default_root().join("digit.json")).unwrap(),
    )
    .unwrap();
    for (pointer, value) in [
        ("/stage", json!("unknown")),
        ("/cases/0/accepted", json!("false")),
        ("/cases/0/consumed", json!(-1)),
        ("/cases/0/captures/0/binding", json!(3)),
        ("/cases/1/id", json!("digit/1")),
        ("/stage", json!("compilation_failed")),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        fs::write(&path, serde_json::to_string_pretty(&bad).unwrap()).unwrap();
        let error = oracle::load(&path).unwrap_err();
        assert_eq!(error.path, path);
        if !error.message.contains("重複") && !error.message.contains("失敗診断") {
            assert!(error.line.unwrap() > 0);
            assert!(error.column.unwrap() > 0);
        }
    }
}
