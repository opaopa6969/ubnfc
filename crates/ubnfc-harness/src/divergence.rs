//! D-017 expectedDivergence: 採取値を変えず、承認済みの観測群だけを既知扱いにする。
use crate::{
    canonical::{diff_values, Difference},
    corpus::{decode, oracle::invalid, read_text},
    CorpusFile, Expectation, Family, LoadError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Observation {
    /// AST と、その構築元となる順序付き capture 出現表。
    Ast,
    Consumed,
    Matched,
    /// prefix 受理 accepted と全入力消費 allConsumed の組。
    Accepted,
    Diagnostics,
    MappingError,
    /// 生成/コンパイルの成否または失敗メッセージ。未実行値を runtime と比較しない。
    Generation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Decision {
    FollowSpec,
    Undecided,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    /// divergences.json と同じディレクトリのファイル名（パスや glob は不可）。
    pub file: String,
    pub case_id: String,
    pub observation: Observation,
    pub reason: String,
    pub decision: Decision,
}

pub fn load(path: &Path) -> Result<Vec<Entry>, LoadError> {
    decode(path, &read_text(path)?)
}

/// 全エントリ（undecided も含む）の参照と重複を検証してから付与する。
/// エラー時は cases を変更せず、再適用時は台帳に合わせて置き換える。
pub fn apply(path: &Path, files: &mut [CorpusFile]) -> Result<(), LoadError> {
    let entries = load(path)?;
    let mut seen = BTreeSet::new();
    let mut resolved = Vec::new();
    for entry in entries {
        let name = Path::new(&entry.file);
        if name.file_name().and_then(|s| s.to_str()) != Some(entry.file.as_str())
            || name.extension().is_none_or(|ext| ext != "json")
            || entry.reason.trim().is_empty()
        {
            return Err(invalid(path, format!("不正な乖離エントリ: {entry:?}")));
        }
        if !seen.insert((entry.file.clone(), entry.case_id.clone(), entry.observation)) {
            return Err(invalid(path, format!("乖離エントリが重複: {entry:?}")));
        }
        let targets: Vec<_> = files
            .iter()
            .enumerate()
            .filter(|(_, file)| {
                file.family == Family::Oracle && file.path.file_name() == Some(name.as_os_str())
            })
            .flat_map(|(f, file)| {
                file.cases
                    .iter()
                    .enumerate()
                    .filter(|(_, case)| case.id == entry.case_id)
                    .map(move |(c, _)| (f, c))
            })
            .collect();
        let [(f, c)] = targets.as_slice() else {
            return Err(invalid(
                path,
                format!("乖離の file/caseId が一意に存在しません: {entry:?}"),
            ));
        };
        if entry.decision == Decision::FollowSpec {
            resolved.push((*f, *c, entry.observation));
        }
    }
    for file in files.iter_mut().filter(|f| f.family == Family::Oracle) {
        for case in &mut file.cases {
            case.known_divergences.clear();
        }
    }
    for (f, c, observation) in resolved {
        files[f].cases[c].known_divergences.push(observation);
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    /// 実際に異なった値と JSON Pointer を保持する。一致した既知項目は含めない。
    KnownDivergence(Vec<Difference>),
    /// 未登録の差分が一つでもあれば Fail。既知の差分も含む全差分を返す。
    Fail(Vec<Difference>),
}

fn observation(path: &str) -> Option<Observation> {
    match path.split('/').nth(1)? {
        "ast" | "captures" => Some(Observation::Ast),
        "consumed" => Some(Observation::Consumed),
        "matched" => Some(Observation::Matched),
        "accepted" | "allConsumed" => Some(Observation::Accepted),
        "diagnostics" => Some(Observation::Diagnostics),
        "mappingError" => Some(Observation::MappingError),
        // declarations は台帳で免除できない。未知項目も自動で既知扱いにしない。
        _ => None,
    }
}

/// actual は runner が同じ Expectation 形式へ正規化して渡す。
/// 既知登録があっても値が一致すれば Pass。undecided は known に含めない。
/// 既存5系統は厳密比較し、oracle の台帳による免除を適用しない。
pub fn compare(expected: &Expectation, actual: &Expectation, known: &[Observation]) -> Verdict {
    let e = serde_json::to_value(expected).expect("Expectation は JSON に直列化可能");
    let a = serde_json::to_value(actual).expect("Expectation は JSON に直列化可能");
    let (diffs, observations) = match (expected, actual) {
        (Expectation::Oracle { .. }, Expectation::Oracle { .. }) => {
            let diffs = diff_values(&e["Oracle"], &a["Oracle"]);
            let observations = diffs
                .iter()
                .map(|d| observation(&d.path))
                .collect::<Vec<_>>();
            (diffs, observations)
        }
        (Expectation::GenerationFailure { .. }, Expectation::GenerationFailure { .. })
        | (Expectation::GenerationFailure { .. }, Expectation::Oracle { .. })
        | (Expectation::Oracle { .. }, Expectation::GenerationFailure { .. }) => {
            // 生成できなかった側には runtime oracle が無い。空配列/null を比較しない。
            let generation = |value: &Value| {
                value
                    .get("GenerationFailure")
                    .cloned()
                    .unwrap_or(Value::Null)
            };
            let e = generation(&e);
            let a = generation(&a);
            let diffs = if e == a {
                Vec::new()
            } else {
                vec![Difference {
                    path: "/generation".into(),
                    expected: Some(e),
                    actual: Some(a),
                }]
            };
            let observations = vec![Some(Observation::Generation); diffs.len()];
            (diffs, observations)
        }
        _ => {
            let diffs = diff_values(&e, &a);
            let observations = vec![None; diffs.len()];
            (diffs, observations)
        }
    };
    if diffs.is_empty() {
        Verdict::Pass
    } else if observations
        .iter()
        .all(|o| o.is_some_and(|o| known.contains(&o)))
    {
        Verdict::KnownDivergence(diffs)
    } else {
        Verdict::Fail(diffs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn oracle() -> Expectation {
        serde_json::from_value(json!({"Oracle": {
            "accepted": true, "allConsumed": true, "consumed": 1, "matched": 1,
            "ast": {"type":"Value", "span":[0,1], "fields":{"value":"a"}},
            "captures": [{"name":"value", "binding":"Root:0", "span":[0,1], "text":"a"}],
            "declarations": [], "diagnostics": [], "mappingError": null
        }}))
        .unwrap()
    }

    fn changed(expected: &Expectation, pointer: &str, value: Value) -> Expectation {
        let mut encoded = serde_json::to_value(expected).unwrap();
        *encoded.pointer_mut(pointer).unwrap() = value;
        serde_json::from_value(encoded).unwrap()
    }

    #[test]
    fn equality_is_pass_even_when_registered() {
        let expected = oracle();
        assert_eq!(compare(&expected, &expected, &[]), Verdict::Pass);
        assert_eq!(
            compare(&expected, &expected, &[Observation::Ast]),
            Verdict::Pass
        );
    }

    #[test]
    fn registered_observation_returns_only_real_differences_with_paths_and_values() {
        let expected = oracle();
        let actual = changed(&expected, "/Oracle/ast/fields/value", json!("b"));
        let Verdict::KnownDivergence(diffs) = compare(
            &expected,
            &actual,
            &[Observation::Ast, Observation::Consumed],
        ) else {
            panic!()
        };
        assert_eq!(
            diffs,
            [Difference {
                path: "/ast/fields/value".into(),
                expected: Some(json!("a")),
                actual: Some(json!("b")),
            }]
        );
    }

    #[test]
    fn unregistered_and_mixed_differences_fail_without_losing_known_diffs() {
        let expected = oracle();
        let actual = changed(&expected, "/Oracle/ast/span/1", json!(2));
        assert!(matches!(compare(&expected, &actual, &[]), Verdict::Fail(_)));
        let actual = changed(&actual, "/Oracle/consumed", json!(2));
        let Verdict::Fail(diffs) = compare(&expected, &actual, &[Observation::Ast]) else {
            panic!()
        };
        assert_eq!(
            diffs.iter().map(|d| d.path.as_str()).collect::<Vec<_>>(),
            ["/ast/span/1", "/consumed"]
        );
        assert!(matches!(
            compare(
                &expected,
                &actual,
                &[Observation::Ast, Observation::Consumed]
            ),
            Verdict::KnownDivergence(_)
        ));
        assert!(matches!(
            compare(&expected, &actual, &[Observation::Matched]),
            Verdict::Fail(_)
        ));
    }

    #[test]
    fn full_acceptance_captures_and_mapping_errors_have_explicit_observation_groups() {
        let expected = oracle();
        for (pointer, value, observation) in [
            ("/Oracle/allConsumed", json!(false), Observation::Accepted),
            ("/Oracle/accepted", Value::Null, Observation::Accepted),
            ("/Oracle/captures/0/span/1", json!(2), Observation::Ast),
            (
                "/Oracle/mappingError",
                json!("mapping failed"),
                Observation::MappingError,
            ),
            ("/Oracle/matched", json!(2), Observation::Matched),
            (
                "/Oracle/diagnostics",
                json!([{"kind":"syntax", "offset":0, "origin":"Mapper.diagnose", "expected":[]}]),
                Observation::Diagnostics,
            ),
        ] {
            let actual = changed(&expected, pointer, value);
            assert!(
                matches!(compare(&expected, &actual, &[]), Verdict::Fail(_)),
                "{pointer}"
            );
            assert!(
                matches!(
                    compare(&expected, &actual, &[observation]),
                    Verdict::KnownDivergence(_)
                ),
                "{pointer}"
            );
        }
        let actual = changed(&expected, "/Oracle/mappingError", json!("mapping failed"));
        assert!(matches!(
            compare(&expected, &actual, &[Observation::Accepted]),
            Verdict::Fail(_)
        ));
    }

    #[test]
    fn declarations_are_never_suppressed_by_the_manifest() {
        let expected = oracle();
        let actual = changed(
            &expected,
            "/Oracle/declarations",
            json!([{"name":"x", "sourceOffset":0}]),
        );
        assert!(matches!(
            compare(
                &expected,
                &actual,
                &[
                    Observation::Accepted,
                    Observation::Ast,
                    Observation::Consumed,
                    Observation::Matched,
                    Observation::Diagnostics,
                    Observation::MappingError,
                    Observation::Generation,
                ]
            ),
            Verdict::Fail(_)
        ));
    }

    #[test]
    fn generation_failure_has_no_runtime_oracle_and_needs_explicit_registration() {
        let expected = Expectation::GenerationFailure {
            message: "compilation_failed: bad API".into(),
        };
        let actual = oracle();
        assert_eq!(
            compare(&expected, &expected, &[Observation::Generation]),
            Verdict::Pass
        );
        for (expected, actual) in [(&expected, &actual), (&actual, &expected)] {
            assert!(matches!(
                compare(expected, actual, &[Observation::Accepted]),
                Verdict::Fail(_)
            ));
            let Verdict::KnownDivergence(diffs) =
                compare(expected, actual, &[Observation::Generation])
            else {
                panic!()
            };
            assert_eq!(diffs.len(), 1);
            assert_eq!(diffs[0].path, "/generation");
        }
        let actual = Expectation::GenerationFailure {
            message: "frontend: invalid grammar".into(),
        };
        assert!(matches!(compare(&expected, &actual, &[]), Verdict::Fail(_)));
        assert!(matches!(
            compare(&expected, &actual, &[Observation::Generation]),
            Verdict::KnownDivergence(_)
        ));
    }

    #[test]
    fn other_corpus_families_do_not_inherit_oracle_waivers() {
        let expected = Expectation::Raw(json!({"ast":null}));
        let actual = Expectation::Raw(json!({"ast":1}));
        assert_eq!(compare(&expected, &expected, &[]), Verdict::Pass);
        assert!(matches!(
            compare(&expected, &actual, &[Observation::Ast]),
            Verdict::Fail(_)
        ));
        assert!(matches!(
            compare(&expected, &oracle(), &[Observation::Generation]),
            Verdict::Fail(_)
        ));
    }
}
