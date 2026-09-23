//! 生成器の乱択文法に対する到達不能候補検査（D-037）の掃き出し。
//!
//! 目的は 2 つ。(1) 乱択文法でも停止し、同じ seed で同じ結果になること。
//! (2) 実際に鳴った警告が本当に「先行候補が成功する位置でしか後続が成功しない」形か、
//! 件数を固定して退行を検出すること。受理・IR は変わらないことも確かめる。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ubnfc_fuzz::gen;

fn choice_codes(seed: u64) -> Option<Vec<String>> {
    let grammar = gen::generate(seed);
    let main = PathBuf::from("/virtual/main.ubnf");
    let imported = grammar.imported_source();
    let text = grammar.render();
    let compiled = ubnfc_front::pipeline::compile(&main, &|path: &Path| {
        if path == main {
            return Ok(text.clone());
        }
        match &imported {
            Some((_, source)) => Ok(source.clone()),
            None => Err(std::io::Error::other("no import")),
        }
    });
    // front が拒否する文法（左再帰・未対応機能など）は campaign と同じく捨てる。
    let compiled = compiled.ok()?;
    assert!(
        !ubnfc_front::ir::to_canonical_json(&compiled.ir).contains("W-CHOICE"),
        "seed {seed}: 警告が IR に混入した"
    );
    Some(
        compiled
            .warnings
            .into_iter()
            .filter(|i| i.code.starts_with("W-CHOICE-"))
            .map(|i| i.code)
            .collect(),
    )
}

#[test]
fn a_thousand_generated_grammars_terminate_and_replay_the_same_choice_warnings() {
    let mut total = BTreeMap::new();
    let mut compiled = 0usize;
    let mut flagged = 0usize;
    for seed in 0..1000 {
        let Some(codes) = choice_codes(seed) else {
            continue;
        };
        compiled += 1;
        assert_eq!(Some(codes.clone()), choice_codes(seed), "seed {seed}");
        if !codes.is_empty() {
            flagged += 1;
        }
        for code in codes {
            *total.entry(code).or_insert(0usize) += 1;
        }
    }
    // 生成器は候補の順序を無作為に選ぶので、死んだ候補は必ず一定数出る。
    assert!(compiled > 800, "compiled={compiled}");
    assert_eq!(
        (compiled, flagged, total),
        (
            SWEEP.0,
            SWEEP.1,
            SWEEP.2.iter().map(|(c, n)| ((*c).to_owned(), *n)).collect()
        )
    );
}

/// 実測値（`docs/reports/2026-09-23-choice-reachability.md`）。
const SWEEP: (usize, usize, &[(&str, usize)]) = (
    1000,
    461,
    &[
        ("W-CHOICE-NULLABLE-NOT-LAST", 594),
        ("W-CHOICE-PREFIX", 3),
        ("W-CHOICE-UNREACHABLE", 9),
    ],
);

/// 鳴った形を目で確かめられるようにする。`--nocapture` で本文を出す。
#[test]
fn the_first_flagged_generated_grammar_of_each_code_is_printed_with_its_source() {
    let mut seen: BTreeMap<String, u64> = BTreeMap::new();
    for seed in 0..1000 {
        let grammar = gen::generate(seed);
        let main = PathBuf::from("/virtual/main.ubnf");
        let imported = grammar.imported_source();
        let text = grammar.render();
        let Ok(compiled) = ubnfc_front::pipeline::compile(&main, &|path: &Path| {
            if path == main {
                return Ok(text.clone());
            }
            match &imported {
                Some((_, source)) => Ok(source.clone()),
                None => Err(std::io::Error::other("no import")),
            }
        }) else {
            continue;
        };
        for issue in compiled
            .warnings
            .iter()
            .filter(|i| i.code.starts_with("W-CHOICE-"))
        {
            if seen.contains_key(&issue.code) {
                continue;
            }
            seen.insert(issue.code.clone(), seed);
            println!("--- seed {seed}\n{text}\n{}", issue.format());
        }
    }
    assert_eq!(seen.len(), 3, "{seen:?}");
}
