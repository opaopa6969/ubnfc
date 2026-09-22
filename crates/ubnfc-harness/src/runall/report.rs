use super::{BuildResult, CaseResult};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

/// case × variant ごとの排他的な結果。段階失敗後は比較しない。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pass,
    Unobserved,
    KnownDivergence,
    Fail,
    DriverError,
    JavacFailure,
    GenerationRejected,
    IrFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Counts {
    pub ir_failure: usize,
    pub generation_rejected: usize,
    pub javac_failure: usize,
    pub driver_error: usize,
    pub pass: usize,
    pub known_divergence: usize,
    pub fail: usize,
    pub unobserved: usize,
}
impl Counts {
    fn record(&mut self, outcome: Outcome) {
        *match outcome {
            Outcome::IrFailure => &mut self.ir_failure,
            Outcome::GenerationRejected => &mut self.generation_rejected,
            Outcome::JavacFailure => &mut self.javac_failure,
            Outcome::DriverError => &mut self.driver_error,
            Outcome::Pass => &mut self.pass,
            Outcome::KnownDivergence => &mut self.known_divergence,
            Outcome::Fail => &mut self.fail,
            Outcome::Unobserved => &mut self.unobserved,
        } += 1;
    }
    pub fn total(&self) -> usize {
        self.ir_failure
            + self.generation_rejected
            + self.javac_failure
            + self.driver_error
            + self.pass
            + self.known_divergence
            + self.fail
            + self.unobserved
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct RefusalCounts {
    pub builds: usize,
    pub cases: usize,
    pub executions: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub backend: String,
    pub generator_sha256: Option<String>,
    pub front_sha256: Option<String>,
    pub jobs: usize,
    pub corpus_files: usize,
    pub grammar_variants: usize,
    pub unique_builds: usize,
    pub source_cases: usize,
    pub totals: Counts,
    pub families: BTreeMap<String, Counts>,
    pub kinds: BTreeMap<String, Counts>,
    pub grammars: BTreeMap<String, Counts>,
    pub refusals: BTreeMap<String, RefusalCounts>,
    pub missing_observations: BTreeMap<String, usize>,
    pub builds: Vec<BuildResult>,
    pub cases: Vec<CaseResult>,
}
impl Report {
    pub(super) fn new(
        jobs: usize,
        corpus_files: usize,
        grammar_variants: usize,
        builds: Vec<BuildResult>,
        mut cases: Vec<CaseResult>,
    ) -> Self {
        cases.sort_by(|a, b| {
            (&a.corpus, &a.case_id, &a.grammar).cmp(&(&b.corpus, &b.case_id, &b.grammar))
        });
        let mut report = Self {
            backend: "java".into(),
            generator_sha256: None,
            front_sha256: None,
            jobs,
            corpus_files,
            grammar_variants,
            unique_builds: builds.len(),
            source_cases: 0,
            totals: Counts::default(),
            families: BTreeMap::new(),
            kinds: BTreeMap::new(),
            grammars: BTreeMap::new(),
            refusals: BTreeMap::new(),
            missing_observations: BTreeMap::new(),
            builds,
            cases,
        };
        let mut unique = BTreeSet::new();
        for row in &report.cases {
            let key = (&row.corpus, &row.case_id);
            unique.insert(key);
            report.totals.record(row.outcome);
            report
                .families
                .entry(row.family.clone())
                .or_default()
                .record(row.outcome);
            report
                .kinds
                .entry(row.kind.clone())
                .or_default()
                .record(row.outcome);
            report
                .grammars
                .entry(row.grammar.clone())
                .or_default()
                .record(row.outcome);
            for missing in &row.missing {
                *report
                    .missing_observations
                    .entry(missing.clone())
                    .or_default() += 1;
            }
        }
        report.source_cases = unique.len();
        type Refusals<'a> =
            BTreeMap<String, (BTreeSet<&'a str>, BTreeSet<(&'a str, &'a str)>, usize)>;
        let mut refusals = Refusals::new();
        for build in &report.builds {
            for (stage, result) in [
                ("ir", &build.ir),
                ("generation", &build.generation),
                ("javac", &build.javac),
            ] {
                for feature in &result.reasons {
                    let counts = refusals.entry(format!("{stage}/{feature}")).or_default();
                    counts.0.insert(&build.id);
                    for row in report
                        .cases
                        .iter()
                        .filter(|row| row.build_id.as_deref() == Some(&build.id))
                    {
                        counts.1.insert((&row.corpus, &row.case_id));
                        counts.2 += 1;
                    }
                }
            }
        }
        report.refusals = refusals
            .into_iter()
            .map(|(key, (builds, cases, executions))| {
                (
                    key,
                    RefusalCounts {
                        builds: builds.len(),
                        cases: cases.len(),
                        executions,
                    },
                )
            })
            .collect();
        report
    }
    pub fn json(&self) -> String {
        serde_json::to_string_pretty(self).expect("run-all report is JSON") + "\n"
    }
    pub fn summary_markdown(&self) -> String {
        let mut out = format!("Java corpus run-all\n\n元 corpus: {} ファイル、{} ケース。文法 variant: {}、hash キャッシュ: {} build、展開後: {} 件（文法なし操作列を含む）。jobs={}。\n\n",
            self.corpus_files, self.source_cases, self.grammar_variants, self.unique_builds, self.totals.total(), self.jobs);
        out.push_str("集計単位は case × variant（文法なし操作列は各1件）。段階失敗後は比較しない。比較に差分がある場合は Fail / Known、差分がなく欠落観測がある場合は未観測、それ以外は Pass。Known/Fail に併存する未観測は JSON の missing と未観測項目表に残す。\n\n");
        if let Some(sha) = &self.generator_sha256 {
            writeln!(out, "Java generator SHA-256: `{sha}`\n").unwrap();
        }
        if let Some(sha) = &self.front_sha256 {
            writeln!(out, "front SHA-256: `{sha}`\n").unwrap();
        }
        let rejections = self
            .cases
            .iter()
            .filter(|row| {
                row.outcome == Outcome::Pass
                    && matches!(row.expected, crate::Expectation::ExpectedRejection { .. })
            })
            .count();
        if rejections > 0 {
            writeln!(out, "Pass のうち明示的な期待拒否: {rejections}件。build の実測失敗と診断は拒否理由表に保持する。\n").unwrap();
        }
        table(&mut out, "系統", &self.families, &self.totals);
        out
    }
    pub fn markdown(&self) -> String {
        let mut out = self.summary_markdown();
        out.push_str("\n6 系統別（case × variant 単位）\n\n");
        table(&mut out, "loader 系統", &self.kinds, &self.totals);
        out.push_str("\n文法別（case × variant 単位。文法なしは操作列）\n\n");
        table(&mut out, "文法", &self.grammars, &self.totals);
        out.push_str("\n拒否理由（M3 作業リスト）\n\n同一文法の拒否はキャッシュ単位に数える。ケース数は理由ごとに重複除去する。生成器が最初の拒否で停止するため、後続の未対応機能を網羅する件数ではない。IR 診断が複数あると理由間で重複する。\n\n| 段階 / 機能・診断コード | build 数 | 元ケース数 | 展開後件数 |\n|---|---:|---:|---:|\n");
        for (reason, count) in &self.refusals {
            writeln!(
                out,
                "| {} | {} | {} | {} |",
                cell(reason),
                count.builds,
                count.cases,
                count.executions
            )
            .unwrap();
        }
        out.push_str(
            "\nFail 詳細（各実行の差分先頭3行。実測応答・全差分・期待値は JSON に保持）\n",
        );
        for row in self.cases.iter().filter(|row| row.outcome == Outcome::Fail) {
            writeln!(
                out,
                "\n{} / {} / {}\n\n| 観測 | 期待 | 実測 |\n|---|---|---|",
                cell(&row.corpus),
                cell(&row.case_id),
                cell(&row.grammar)
            )
            .unwrap();
            for diff in row.differences.iter().take(3) {
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
        out.push_str("\n未観測項目（展開後件数。欠落を Pass に読み替えない）\n\n| 観測 | 件数 |\n|---|---:|\n");
        for (path, count) in &self.missing_observations {
            writeln!(out, "| {} | {count} |", cell(path)).unwrap();
        }
        out.push_str("\n段階失敗の原文（完全な標準出力・標準エラーは JSON）\n");
        for build in &self.builds {
            for (stage, result) in [
                ("ir", &build.ir),
                ("generation", &build.generation),
                ("javac", &build.javac),
            ] {
                if !result.reasons.is_empty() {
                    writeln!(
                        out,
                        "\n{} / {}: {}\n",
                        build.id,
                        stage,
                        cell(&build.grammars.join(", "))
                    )
                    .unwrap();
                    for line in result
                        .output
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .take(3)
                    {
                        writeln!(out, "> {}", cell(line)).unwrap();
                    }
                }
            }
        }
        for row in self
            .cases
            .iter()
            .filter(|row| row.outcome == Outcome::DriverError)
        {
            writeln!(
                out,
                "\n実行エラー: {} / {} / {}: {}",
                cell(&row.corpus),
                cell(&row.case_id),
                cell(&row.grammar),
                row.actual
                    .as_ref()
                    .map_or_else(String::new, |v| cell(&v.to_string()))
            )
            .unwrap();
        }
        out
    }
}
fn table(out: &mut String, label: &str, rows: &BTreeMap<String, Counts>, total: &Counts) {
    writeln!(out, "| {label} | ir 失敗 | 生成拒否 | javac 失敗 | 実行エラー | Pass | Known | Fail | 未観測 |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|").unwrap();
    for (name, c) in rows
        .iter()
        .map(|(n, c)| (n.as_str(), c))
        .chain([("合計", total)])
    {
        writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            cell(name),
            c.ir_failure,
            c.generation_rejected,
            c.javac_failure,
            c.driver_error,
            c.pass,
            c.known_divergence,
            c.fail,
            c.unobserved
        )
        .unwrap();
    }
}
fn display(value: &Option<serde_json::Value>) -> String {
    value
        .as_ref()
        .map_or_else(|| "欠落".into(), |v| cell(&v.to_string()))
}
fn cell(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace(['\n', '\r'], " ")
}
