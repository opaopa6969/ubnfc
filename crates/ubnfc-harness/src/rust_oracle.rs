//! D-011 の言語別 oracle（Rust 側）。既存 Rust 実装（`unlaxer-parser/rust`）を実走した
//! 診断を `corpus/oracle/rust/diagnostics.json` に固定し、Rust backend の比較でだけ
//! Java 採取 JSON の `diagnostics[i].offset`（主位置）を置換する。
//! `farthestOffset` は D-014 の最遠到達（共有契約）なので Java 採取値のまま。
//! 既存 Rust runtime の `farthest.offset` は主 offset の複製であり、最遠到達の oracle ではない。
//! Java 採取ファイルは変更しない。kind / expected / origin は置換しない。
use crate::{
    corpus::{decode, oracle::invalid, read_text},
    CorpusFile, Family, LoadError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

/// 1 ケース分の Rust 実走診断（`ParseDiagnostic::canonical_json` の行そのまま）。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustOracle {
    pub diagnostics: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Entry {
    file: String,
    case_id: String,
    input: String,
    diagnostics: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sidecar {
    backend: String,
    observation: String,
    source: String,
    cases: Vec<Entry>,
}

/// sidecar が無ければ言語別 oracle は無い（Java 採取値をそのまま使う）。
/// あれば全エントリの参照・入力一致・診断形を検証してから付与する。
pub fn apply(path: &Path, files: &mut [CorpusFile]) -> Result<(), LoadError> {
    for file in files.iter_mut().filter(|f| f.family == Family::Oracle) {
        for case in &mut file.cases {
            case.rust_oracle = None;
        }
    }
    if !path.is_file() {
        return Ok(());
    }
    let sidecar: Sidecar = decode(path, &read_text(path)?)?;
    if sidecar.backend != "rust" || sidecar.observation != "diagnostics" {
        return Err(invalid(
            path,
            "rust oracle は backend=rust, observation=diagnostics のみ",
        ));
    }
    if sidecar.source.trim().is_empty() {
        return Err(invalid(path, "source（採取元 revision）が空です"));
    }
    let mut seen = BTreeSet::new();
    for entry in sidecar.cases {
        let name = Path::new(&entry.file);
        if name.file_name().and_then(|s| s.to_str()) != Some(entry.file.as_str()) {
            return Err(invalid(path, format!("不正な file: {}", entry.file)));
        }
        if !seen.insert((entry.file.clone(), entry.case_id.clone())) {
            return Err(invalid(
                path,
                format!("rust oracle が重複: {} {}", entry.file, entry.case_id),
            ));
        }
        for row in &entry.diagnostics {
            let ok = row.get("kind").is_some_and(Value::is_string)
                && row.get("offset").is_some_and(Value::is_u64)
                && row.get("farthestOffset").is_some_and(Value::is_u64);
            if !ok {
                return Err(invalid(
                    path,
                    format!(
                        "rust oracle の診断に kind/offset/farthestOffset がありません: {} {}",
                        entry.file, entry.case_id
                    ),
                ));
            }
        }
        let targets: Vec<_> = files
            .iter_mut()
            .filter(|file| {
                file.family == Family::Oracle && file.path.file_name() == Some(name.as_os_str())
            })
            .flat_map(|file| file.cases.iter_mut().filter(|c| c.id == entry.case_id))
            .collect();
        let [case] = targets
            .into_iter()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| {
                invalid(
                    path,
                    format!(
                        "rust oracle の file/caseId が一意に存在しません: {} {}",
                        entry.file, entry.case_id
                    ),
                )
            })?;
        if case.input != entry.input {
            return Err(invalid(
                path,
                format!(
                    "rust oracle の入力が Java 採取と異なります: {} {}",
                    entry.file, entry.case_id
                ),
            ));
        }
        case.rust_oracle = Some(RustOracle {
            diagnostics: entry.diagnostics,
        });
    }
    Ok(())
}

/// Java 採取の diagnostics 列に Rust 実走の主 offset を重ねる。
/// 件数と kind が index ごとに一致する場合だけ主位置を置換し、それ以外は
/// Rust 実走の列で置き換える（origin は Java 側の値を引き継ぐ）。
pub fn project_diagnostics(java: &mut Value, rust: &RustOracle) {
    let Some(rows) = java.as_array_mut() else {
        return;
    };
    let aligned = rows.len() == rust.diagnostics.len()
        && rows
            .iter()
            .zip(&rust.diagnostics)
            .all(|(j, r)| j.get("kind") == r.get("kind"));
    if aligned {
        for (row, captured) in rows.iter_mut().zip(&rust.diagnostics) {
            if let Some(row) = row.as_object_mut() {
                row.insert("offset".into(), captured["offset"].clone());
            }
        }
        return;
    }
    let origin = rows
        .first()
        .and_then(|r| r.get("origin"))
        .cloned()
        .unwrap_or_else(|| Value::String("Mapper.diagnose".into()));
    *rows = rust
        .diagnostics
        .iter()
        .map(|captured| {
            let mut row = captured.clone();
            if let Some(row) = row.as_object_mut() {
                row.entry("origin").or_insert_with(|| origin.clone());
            }
            row
        })
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn aligned_rows_replace_only_the_primary_offset() {
        let mut java = json!([{"kind":"syntax","offset":4,"expected":["DigitParser"],
            "farthestOffset":4,"farthestExpected":["DigitParser"],"origin":"Mapper.diagnose"}]);
        let rust = RustOracle {
            diagnostics: vec![
                json!({"kind":"syntax","offset":2,"expected":[";"],"farthestOffset":2,"farthestExpected":[";"]}),
            ],
        };
        project_diagnostics(&mut java, &rust);
        assert_eq!(
            java,
            json!([{"kind":"syntax","offset":2,"expected":["DigitParser"],
                "farthestOffset":4,"farthestExpected":["DigitParser"],"origin":"Mapper.diagnose"}])
        );
    }

    #[test]
    fn misaligned_rows_take_the_rust_capture_with_java_origin() {
        let mut java =
            json!([{"kind":"syntax","offset":4,"expected":["x"],"origin":"Mapper.diagnose"}]);
        let rust = RustOracle {
            diagnostics: vec![],
        };
        project_diagnostics(&mut java, &rust);
        assert_eq!(java, json!([]));
        let mut java = json!([]);
        let rust = RustOracle {
            diagnostics: vec![
                json!({"kind":"syntax","offset":0,"expected":["x"],"farthestOffset":0,"farthestExpected":["x"]}),
            ],
        };
        project_diagnostics(&mut java, &rust);
        assert_eq!(
            java,
            json!([{"kind":"syntax","offset":0,"expected":["x"],"farthestOffset":0,"farthestExpected":["x"],"origin":"Mapper.diagnose"}])
        );
    }

    #[test]
    fn default_sidecar_attaches_to_the_documented_cases_only() {
        let files = crate::corpus::oracle::load_all(crate::corpus::oracle::default_root()).unwrap();
        let attached: Vec<_> = files
            .iter()
            .flat_map(|f| {
                let name = f.path.file_name().unwrap().to_string_lossy().into_owned();
                f.cases
                    .iter()
                    .filter(|c| c.rust_oracle.is_some())
                    .map(move |c| format!("{name}#{}", c.id))
            })
            .collect();
        assert_eq!(
            attached,
            [
                "lookahead-failure.json#lookahead-failure/1",
                "numeric.json#numeric/1",
                "numeric.json#numeric/2",
                "numeric.json#numeric/3",
                "numeric.json#numeric/4",
                "numeric.json#numeric/5",
                "numeric.json#numeric/6",
                "numeric.json#numeric/7",
                "predictive-nullable.json#predictive-nullable/1",
                "predictive-nullable.json#predictive-nullable/2",
                "predictive-nullable.json#predictive-nullable/3",
                "reobserved-lookahead-failure.json#lookahead-failure/1",
                "reobserved-numeric.json#numeric/1",
                "reobserved-numeric.json#numeric/6",
                "reobserved-predictive-nullable.json#predictive-nullable/3",
            ]
        );
        let numeric7 = files
            .iter()
            .find(|f| f.path.file_name().unwrap() == "numeric.json")
            .unwrap()
            .cases
            .iter()
            .find(|c| c.id == "numeric/7")
            .unwrap();
        assert_eq!(
            numeric7.rust_oracle.as_ref().unwrap().diagnostics[0]["offset"],
            json!(2)
        );
    }

    #[test]
    fn missing_sidecar_means_no_override_and_bad_entries_are_rejected() {
        let mut files =
            crate::corpus::oracle::load_all(crate::corpus::oracle::default_root()).unwrap();
        apply(Path::new("/nonexistent/rust/diagnostics.json"), &mut files).unwrap();
        assert!(files
            .iter()
            .flat_map(|f| &f.cases)
            .all(|c| c.rust_oracle.is_none()));
        let scratch = crate::runner::Scratch::new().unwrap();
        let path = scratch.0.join("diagnostics.json");
        for (body, message) in [
            (
                json!({"backend":"java","observation":"diagnostics","source":"x","cases":[]}),
                "backend=rust",
            ),
            (
                json!({"backend":"rust","observation":"diagnostics","source":"x","cases":[
                    {"file":"numeric.json","caseId":"numeric/9","input":"","diagnostics":[]}]}),
                "一意に存在しません",
            ),
            (
                json!({"backend":"rust","observation":"diagnostics","source":"x","cases":[
                    {"file":"numeric.json","caseId":"numeric/7","input":"n1e+","diagnostics":[]}]}),
                "入力が Java 採取と異なります",
            ),
            (
                json!({"backend":"rust","observation":"diagnostics","source":"x","cases":[
                    {"file":"numeric.json","caseId":"numeric/7","input":"n1e+;","diagnostics":[{"kind":"syntax"}]}]}),
                "farthestOffset",
            ),
        ] {
            std::fs::write(&path, body.to_string()).unwrap();
            let error = apply(&path, &mut files).unwrap_err();
            assert!(error.message.contains(message), "{error}");
        }
    }
}
