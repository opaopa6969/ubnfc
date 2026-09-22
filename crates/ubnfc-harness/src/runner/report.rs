use super::{compare, unobserved, Actual, RunCase};
use crate::{canonical::Difference, divergence::Verdict};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Counts {
    pub pass: usize,
    pub known_divergence: usize,
    pub fail: usize,
    /// ケース数ではなく、比較できなかった観測項目の件数。
    pub unobserved: usize,
    pub unobserved_cases: usize,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct Report {
    pub pass: usize,
    pub known_divergence: usize,
    pub fail: Vec<(String, Vec<Difference>)>,
    pub known_diffs: Vec<(String, Vec<Difference>)>,
    pub unobserved: usize,
    pub unobserved_cases: usize,
    pub missing: Vec<(String, Vec<String>)>,
    pub families: BTreeMap<String, Counts>,
}
impl Report {
    pub fn from_actual(cases: &[RunCase], actual: &[Actual]) -> super::Result<Self> {
        if cases.len() != actual.len() {
            return Err(super::Error(format!(
                "ケース数 {} と結果数 {} が一致しません",
                cases.len(),
                actual.len()
            )));
        }
        let mut report = Self::default();
        for (case, actual) in cases.iter().zip(actual) {
            report.record(case, actual);
        }
        Ok(report)
    }
    pub fn record(&mut self, case: &RunCase, actual: &Actual) {
        let label = case.label();
        let count = self.families.entry(case.family.clone()).or_default();
        let missing = unobserved(case, actual);
        if !missing.is_empty() {
            self.unobserved += missing.len();
            self.unobserved_cases += 1;
            count.unobserved += missing.len();
            count.unobserved_cases += 1;
            self.missing.push((label.clone(), missing));
        }
        match compare(case, actual, &case.case.known_divergences) {
            Verdict::Pass => {
                self.pass += 1;
                count.pass += 1;
            }
            Verdict::KnownDivergence(diffs) => {
                self.known_divergence += 1;
                count.known_divergence += 1;
                self.known_diffs.push((label, diffs));
            }
            Verdict::Fail(diffs) => {
                count.fail += 1;
                self.fail.push((label, diffs));
            }
        }
    }
    pub fn json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Report は JSON 値")
    }
    pub fn markdown(&self) -> String {
        use std::fmt::Write;
        let mut out = String::from("| 系統 | Pass | Known divergence | Fail | 未観測項目 | 未観測ケース |\n|---|---:|---:|---:|---:|---:|\n");
        for (family, count) in &self.families {
            writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} |",
                cell(family),
                count.pass,
                count.known_divergence,
                count.fail,
                count.unobserved,
                count.unobserved_cases
            )
            .unwrap();
        }
        writeln!(
            out,
            "| 合計 | {} | {} | {} | {} | {} |",
            self.pass,
            self.known_divergence,
            self.fail.len(),
            self.unobserved,
            self.unobserved_cases
        )
        .unwrap();
        for (kind, rows) in [
            ("Fail", &self.fail),
            ("Known divergence", &self.known_diffs),
        ] {
            for (case, diffs) in rows {
                writeln!(
                    out,
                    "\n{kind}: {}\n\n| パス | 期待値 | 実測値 |\n|---|---|---|",
                    cell(case)
                )
                .unwrap();
                for diff in diffs {
                    writeln!(
                        out,
                        "| {} | {} | {} |",
                        cell(&diff.path),
                        display(&diff.expected),
                        display(&diff.actual)
                    )
                    .unwrap();
                }
            }
        }
        if !self.missing.is_empty() {
            out.push_str("\n未観測（Pass は観測できた項目の一致を表します）:\n\n");
            for (case, paths) in &self.missing {
                writeln!(
                    out,
                    "- {}: {}",
                    cell(case),
                    paths
                        .iter()
                        .map(|path| cell(path))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .unwrap();
            }
        }
        out
    }
}
fn display(value: &Option<serde_json::Value>) -> String {
    value
        .as_ref()
        .map_or_else(|| "欠落".into(), |value| cell(&value.to_string()))
}
fn cell(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace(['\r', '\n'], " ")
}
