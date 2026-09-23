//! 深い文法・巡回の多い文法で front-end が異常終了・停止しないことの回帰。
//!
//! どちらも issue #2（2026-09-23 の panic 掃き出しで再現手順だけ残していた 2 件）。
//! 再現手順は `docs/reports/2026-09-23-crosscheck-fixes.md` の「直していない 2 件」。

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ubnfc_front::analysis::analyze_with_choice_issues;
use ubnfc_front::lower::{lower, SourceInput};
use ubnfc_front::resolve::resolve;
use ubnfc_front::syntax::parse_file;
use ubnfc_front::validate::validate;

/// parse → resolve → validate → lower → analyze。深い文法でどの段も落ちないことを見る。
fn compile(text: &str) -> usize {
    let path = PathBuf::from("deep.ubnf");
    let grammar = resolve(&[(path.clone(), parse_file(text).unwrap())], &|_| {
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    })
    .unwrap();
    let issues = validate(&grammar);
    assert!(
        issues.iter().all(|i| i.severity() != "ERROR"),
        "深い連鎖は正しい文法のはず: {issues:?}"
    );
    let lowered = lower(&grammar, &[SourceInput::new(path, "deep.ubnf", text)])
        .unwrap_or_else(|e| panic!("{e:?}"));
    let (analysis, _, _) =
        analyze_with_choice_issues(&lowered.semantics).unwrap_or_else(|e| panic!("{e:?}"));
    analysis.rules.len()
}

/// `A ::= 'a' B; B ::= 'a' C; …` の連鎖。規則参照グラフの深さだけが効く。
fn chain(depth: usize) -> String {
    let mut text = String::from("grammar G {\n  @root R0 ::= 'a' R1;\n");
    for i in 1..depth {
        text.push_str(&format!("  R{i} ::= 'a' R{};\n", i + 1));
    }
    text.push_str(&format!("  R{depth} ::= 'x';\n}}\n"));
    text
}

/// 左端が rule 参照の連鎖 + 順序付き選択。`analysis::reachability` の
/// `lead` / `literal_prefix` はこの形でだけ rule 本体へ降りる。
fn leftmost_chain(depth: usize) -> String {
    let mut text =
        String::from("grammar G {\n  @root S ::= A | B;\n  A ::= R0;\n  B ::= R0 'z';\n");
    for i in 0..depth {
        text.push_str(&format!("  R{i} ::= R{} 'a';\n", i + 1));
    }
    text.push_str(&format!("  R{depth} ::= 'x';\n}}\n"));
    text
}

/// リテラル始まりの連鎖。`literal_prefix` は Seq の先頭リテラルを繋ぎながら降りる。
fn literal_chain(depth: usize) -> String {
    let mut text = String::from("grammar G {\n  @root S ::= 'a' | B;\n  B ::= R0;\n");
    for i in 0..depth {
        text.push_str(&format!("  R{i} ::= 'a' R{};\n", i + 1));
    }
    text.push_str(&format!("  R{depth} ::= 'x';\n}}\n"));
    text
}

/// 小さい stack の thread で走らせる。明示スタック化してあれば深さは heap にしか乗らない。
/// 再帰のままだと（このサイズでは release でも）stack overflow で **プロセスごと**落ちる。
fn in_small_stack(name: &'static str, text: String, expected_rules: usize) {
    let done = std::thread::Builder::new()
        .name(name.into())
        .stack_size(512 * 1024)
        .spawn(move || compile(&text))
        .expect("thread")
        .join()
        .expect("解析が stack overflow せずに終わること");
    assert_eq!(done, expected_rules);
}

/// 規則参照グラフを辿る再帰に深さ上限が無く、release ビルドでも約 16,000 段の
/// 規則連鎖で exit 134（stack overflow）になっていた。再帰だったのは
/// `analysis::graph::scc`（Tarjan）と `analysis::planning::effects`。
#[test]
fn deep_rule_reference_chain_does_not_overflow_the_stack() {
    in_small_stack("deep-chain", chain(4_000), 4_001);
}

/// 同じ連鎖を `analysis::reachability::lead` が辿る形。左端が rule 参照だと
/// choice の到達性判定が rule 本体へ降りる。
#[test]
fn deep_leftmost_chain_does_not_overflow_the_stack() {
    in_small_stack("deep-leftmost", leftmost_chain(600), 604);
}

/// 同じ連鎖を `analysis::reachability::literal_prefix` が辿る形。
#[test]
fn deep_literal_prefix_chain_does_not_overflow_the_stack() {
    in_small_stack("deep-literal", literal_chain(4_000), 4_003);
}

/// 互いを左端参照する N 規則。旧 `validate::operators::find_cycles` は
/// visited を持たず全単純経路を数え上げていたので、9 規則（505 byte）で
/// 986,400 件・9 秒、10 規則で 120 秒超だった。いまは規則ごとに 1 件。
#[test]
fn mutually_left_recursive_rules_are_reported_in_linear_time() {
    let n = 12;
    let mut text = String::from("grammar G {\n");
    for i in 0..n {
        let alternatives: Vec<_> = (0..n)
            .filter(|j| *j != i)
            .map(|j| format!("R{j}"))
            .collect();
        let root = if i == 0 { "@root " } else { "" };
        text.push_str(&format!(
            "  {root}R{i} ::= {} | 'x';\n",
            alternatives.join(" | ")
        ));
    }
    text.push_str("}\n");
    assert!(text.len() < 1024, "1 KB 未満の文法であること");

    let grammar = resolve(&[("clique.ubnf", parse_file(&text).unwrap())], &|_| {
        panic!("unexpected import")
    })
    .unwrap();
    let started = Instant::now();
    let issues = validate(&grammar);
    let elapsed = started.elapsed();

    let cycles: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "W-LEFT-RECURSION")
        .collect();
    assert_eq!(cycles.len(), n, "左再帰の規則ごとに 1 件");
    assert!(
        elapsed < Duration::from_millis(500),
        "左端巡回の検出が線形時間であること: {elapsed:?}"
    );
}

/// 巡回の報告内容の契約。Java bootstrap（`tests/fixtures/validation-java.json` の
/// `testValidateWithWarningsIndirectLeftRecursion`）と同じく、巡回に載る規則ごとに
/// その規則から始まる巡回を 1 件ずつ出す。
#[test]
fn left_recursive_cycles_are_reported_once_per_rule() {
    let cases = [
        (
            "grammar G { @root Expr ::= Expr | 'n'; }",
            vec!["left-recursive cycle detected: Expr -> Expr"],
        ),
        (
            "grammar G { @root A ::= B; B ::= A; }",
            vec![
                "left-recursive cycle detected: A -> B -> A",
                "left-recursive cycle detected: B -> A -> B",
            ],
        ),
        (
            "grammar G { @root A ::= B | 'x'; B ::= C | A; C ::= B | 'y'; }",
            // B の左端参照は宣言順に C, A。最短の巡回は B -> C -> B。
            vec![
                "left-recursive cycle detected: A -> B -> A",
                "left-recursive cycle detected: B -> C -> B",
                "left-recursive cycle detected: C -> B -> C",
            ],
        ),
    ];
    for (source, expected) in cases {
        let grammar = resolve(&[("t.ubnf", parse_file(source).unwrap())], &|_| {
            panic!("unexpected import")
        })
        .unwrap();
        let messages: Vec<_> = validate(&grammar)
            .into_iter()
            .filter(|i| i.code == "W-LEFT-RECURSION")
            .map(|i| i.message)
            .collect();
        assert_eq!(messages, expected, "{source}");
    }
}
