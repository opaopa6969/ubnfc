use super::*;

pub(super) fn run(plan: Plan, config: &Config, directory: &Path) -> Result<Report> {
    let mut results = Vec::new();
    let mut generated = Vec::new();
    let mut pending = Vec::new();
    for work in &plan.work {
        let path = directory.join(&work.build.id);
        let mut build = work.build.clone();
        let rows = work.rows.clone();
        let (ir_path, grammar_name) = match prepare_ir(work, config, &path) {
            Ok(prepared) => {
                build.ir = StageResult::success();
                prepared
            }
            Err(error) => {
                build.ir = error;
                results.push(stopped(build, rows, Outcome::IrFailure, &path));
                continue;
            }
        };
        let artifact = path.join("generated");
        build.generation = StageResult::command(
            "generation",
            Command::new(config.rust_generator.as_ref().expect("resolved generator"))
                .arg("--ir")
                .arg(ir_path)
                .arg("--out")
                .arg(&artifact)
                .arg("--emit-driver")
                .output(),
        );
        if build.generation.status == StageStatus::Failed {
            results.push(stopped(build, rows, Outcome::GenerationRejected, &path));
            continue;
        }
        if let Err(error) = runner::scanners::rust(&artifact) {
            build.cargo = Some(StageResult::failed("build", error.to_string()));
            results.push(stopped(build, rows, Outcome::BuildFailure, &path));
            continue;
        }
        let mut cases = work.cases.clone();
        for case in &mut cases {
            case.grammar.clone_from(&grammar_name);
        }
        generated.push(artifact);
        pending.push((build, rows, cases, path));
    }
    let build_dir = directory.join("build");
    let batch = runner::rust::build_batch(&generated, &build_dir, config.cargo_jobs)?;
    let runner = Runner {
        backend: Backend::Rust,
        build_dir,
    };
    for ((mut build, mut rows, cases, path), result) in pending.into_iter().zip(batch.results) {
        match result {
            Err(error) => {
                build.cargo = Some(StageResult::failed("build", error.to_string()));
                results.push(stopped(build, rows, Outcome::BuildFailure, &path));
            }
            Ok(executable) => {
                build.cargo = Some(StageResult::success());
                for ((row, case), actual) in rows
                    .iter_mut()
                    .zip(&cases)
                    .zip(runner.run(&executable, &cases))
                {
                    row.observe(case, actual);
                }
                normalize_logs(&mut build, &path);
                results.push((build, rows));
            }
        }
    }
    // 元の plan 順に戻す。段階失敗で report の build 順が変わらないようにする。
    let mut results: BTreeMap<_, _> = results
        .into_iter()
        .map(|(b, r)| (b.id.clone(), (b, r)))
        .collect();
    let mut builds = Vec::new();
    let mut cases = plan.without_grammar;
    for work in plan.work {
        let (mut build, rows) = results.remove(&work.build.id).expect("recorded build");
        // 一括 workspace と target のランダムパスも取り除く。
        normalize_logs(&mut build, directory);
        builds.push(build);
        cases.extend(rows);
    }
    let mut report = Report::new(
        Backend::Rust,
        config.jobs,
        plan.files,
        plan.variants,
        builds,
        cases,
    );
    report.rust_build = Some(batch.stats);
    Ok(report)
}
