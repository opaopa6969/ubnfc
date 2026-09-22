//! tinyexpression の fixture 16件に対する受理・消費量・canonical AST golden の固定。
//! 期待値は docs/reports/2026-09-22-p4-java-bench.md（Java 側）と同じ。
//! `UBNFC_P4_WRITE_GOLDEN=1` で golden を書き出す（通常は比較のみ）。
use std::path::{Path, PathBuf};
use ubnfc_p4_rust::{parse_entry, parse_formula, ParseOptions, ParseResult};

/// Formula 入口で全消費する fixture と、その code point 数。
const ACCEPTED: &[(&str, usize)] = &[
    ("complex", 325),
    ("complex-x4", 1293),
    ("complex-x16", 5179),
    ("complex-x64", 20923),
    ("flat-arithmetic", 1426),
    ("large-match", 1380),
    ("valid-basic", 6),
    ("valid-multiline", 40),
    ("valid-unicode", 9),
];
/// Formula 入口が拒否する fixture（comparison-heavy 系は Expression の選択順序、他は意図的な不正入力）。
const REJECTED: &[&str] = &[
    "comparison-heavy",
    "comparison-heavy-x4",
    "comparison-heavy-x16",
    "comparison-heavy-x64",
    "complex-half",
    "complex-tail",
    "invalid-syntax",
];
/// BooleanExpression 入口なら全消費する fixture と、その code point 数。
const BOOLEAN_ENTRY: &[(&str, usize)] = &[
    ("comparison-heavy", 179),
    ("comparison-heavy-x4", 716),
    ("comparison-heavy-x16", 2864),
    ("comparison-heavy-x64", 11456),
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn fixture(name: &str) -> String {
    std::fs::read_to_string(root().join("tests/fixtures").join(format!("{name}.tiny"))).unwrap()
}
/// 既定の診断予算（200 万 node）では complex-x4 以上・large-match・comparison-heavy-x4 以上が
/// `diagnostic budget exceeded` になった。M5 で失敗 1 件が O(1) 件になり既定予算で通る
/// （`default_diagnostic_budget_accepts_every_fixture_and_small_budget_fails_as_resource` で固定）。
/// 受理検証は予算だけ広げる。予算は事前確保ではない。
fn options(build_ast: bool, memo: bool) -> ParseOptions {
    let mut options = ParseOptions {
        build_ast,
        memo,
        ..Default::default()
    };
    options.limits.diagnostics = 1 << 30;
    options.limits.arena_entries = 1 << 30;
    options.limits.memo_entries = 1 << 30;
    options
}
fn strip_statistics(mut result: ParseResult) -> ParseResult {
    // memo の hash は session ごとの RandomState なので memo_max_probe は実行ごとに変わる。
    result.statistics = Default::default();
    result
}
fn check_golden(sub: &str, name: &str, result: &ParseResult) -> String {
    let json = result.ast.as_ref().expect("AST").canonical_json();
    let path = root()
        .join("tests/golden")
        .join(sub)
        .join(format!("{name}.tiny.json"));
    if std::env::var_os("UBNFC_P4_WRITE_GOLDEN").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, format!("{json}\n")).unwrap();
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("golden {} を読めない: {e}", path.display()));
    assert_eq!(
        expected.trim_end(),
        json,
        "canonical AST が golden と異なる: {name}"
    );
    json
}
fn assert_full(name: &str, result: &ParseResult, cp: usize) {
    assert!(
        result.ok,
        "{name}: 受理されるべき: {:?}",
        result.diagnostics
    );
    assert_eq!(result.consumed_cp, cp, "{name}: consumed");
    assert_eq!(result.matched_cp, cp, "{name}: matched");
    assert!(
        result.diagnostics.is_empty(),
        "{name}: {:?}",
        result.diagnostics
    );
}

#[test]
fn formula_accepts_and_matches_golden() {
    for &(name, cp) in ACCEPTED {
        let text = fixture(name);
        assert_eq!(text.chars().count(), cp, "{name}: fixture の code point 数");
        let with_ast = parse_formula(&text, options(true, true));
        assert_full(name, &with_ast, cp);
        let json = check_golden("", name, &with_ast);
        // 認識のみ: 同じ受理・消費で AST と capture は省略。
        let recognition = parse_formula(&text, options(false, true));
        assert_full(name, &recognition, cp);
        assert!(recognition.ast.is_none());
        assert!(recognition.captures.is_empty());
        // memo 無効でも同じ AST。complex 以上は memo 無効だと rule 評価が 168,000 回を超え
        // release でも 4 秒かかるため、小さい fixture だけ確認する。
        if cp <= 64 {
            let no_memo = parse_formula(&text, options(true, false));
            assert_full(name, &no_memo, cp);
            assert_eq!(
                no_memo.ast.unwrap().canonical_json(),
                json,
                "{name}: memo off"
            );
            assert_eq!(no_memo.statistics.memo_entries, 0);
        }
    }
}

#[test]
fn default_diagnostic_budget_accepts_every_fixture_and_small_budget_fails_as_resource() {
    // M5: `limits.diagnostics` は「記録した診断の件数」の上限。主診断 DAG の node 数と、
    // 公開 farthest 候補（label）の記録数をそれぞれ数え、どちらかが超えると Resource 診断で
    // 全体失敗になる。失敗 1 件が O(1) 件になったので、既定 2,000,000 で全 fixture が通る。
    let default = ParseOptions::default();
    let run = |name: &str, entry: Option<&str>, options: ParseOptions| match entry {
        None => parse_formula(&fixture(name), options),
        Some(entry) => parse_entry(Some(entry), &fixture(name), options).unwrap(),
    };
    for (name, entry) in [
        ("complex", None),
        ("complex-x4", None),
        ("complex-x16", None),
        ("complex-x64", None),
        ("flat-arithmetic", None),
        ("large-match", None),
        ("valid-basic", None),
        ("valid-multiline", None),
        ("valid-unicode", None),
        ("comparison-heavy", Some("BooleanExpression")),
        ("comparison-heavy-x4", Some("BooleanExpression")),
        ("comparison-heavy-x16", Some("BooleanExpression")),
        ("comparison-heavy-x64", Some("BooleanExpression")),
    ] {
        let result = run(name, entry, default);
        assert!(
            result.ok,
            "{name}: 既定予算で受理されるはず: {:?}",
            result.diagnostics
        );
    }
    // memo 無効でも complex は既定予算に収まる（rule 評価 168,262 回）。
    let no_memo = run(
        "complex",
        None,
        ParseOptions {
            memo: false,
            ..Default::default()
        },
    );
    assert!(no_memo.ok, "{:?}", no_memo.diagnostics);
    // 予算を小さくすると、認識は正しく進んでも Resource 診断 1 件で全体失敗になる。
    let mut small = ParseOptions::default();
    small.limits.diagnostics = 200;
    for (name, entry) in [
        ("complex", None),
        ("comparison-heavy", Some("BooleanExpression")),
    ] {
        let result = run(name, entry, small);
        assert!(!result.ok, "{name}: 予算 200 で失敗するはず");
        assert_eq!(result.consumed_cp, 0, "{name}");
        assert_eq!(
            result.diagnostics.len(),
            1,
            "{name}: {:?}",
            result.diagnostics
        );
        let diagnostic = &result.diagnostics[0];
        assert_eq!(
            diagnostic.kind,
            ubnfc_p4_rust::DiagnosticKind::Resource,
            "{name}"
        );
        assert_eq!(
            diagnostic.expected,
            vec!["diagnostic budget exceeded"],
            "{name}"
        );
        assert!(result.ast.is_none(), "{name}");
    }
}

#[test]
fn formula_rejects_known_inputs() {
    for &name in REJECTED {
        let text = fixture(name);
        for build_ast in [true, false] {
            let result = parse_formula(&text, options(build_ast, true));
            assert!(!result.ok, "{name}: 拒否されるべき");
            assert!(result.ast.is_none());
            assert!(!result.diagnostics.is_empty(), "{name}: 診断なし");
        }
    }
}

#[test]
fn formula_stops_comparison_heavy_before_comparison_operator() {
    // `1+2*3>=...`: Expression は NumberExpression を先に選び、`>=` の手前 (offset 5) で
    // Formula の EOF に届かない。Java 報告と同じ位置。
    for name in ["comparison-heavy", "comparison-heavy-x4"] {
        let result = parse_formula(&fixture(name), options(true, true));
        assert!(!result.ok);
        assert_eq!(result.consumed_cp, 0, "{name}: root は巻き戻す");
        let offsets: Vec<usize> = result.diagnostics.iter().map(|d| d.offset_cp).collect();
        assert!(
            offsets.contains(&5),
            "{name}: 主診断が offset 5 にない: {offsets:?}"
        );
    }
    let minimal = parse_formula("1>=1", options(true, true));
    assert!(!minimal.ok);
    assert!(
        parse_entry(Some("BooleanExpression"), "1>=1", options(true, true))
            .unwrap()
            .ok
    );
}

#[test]
fn boolean_entry_accepts_comparison_heavy() {
    for &(name, cp) in BOOLEAN_ENTRY {
        let text = fixture(name);
        assert_eq!(text.chars().count(), cp, "{name}: fixture の code point 数");
        let result = parse_entry(Some("BooleanExpression"), &text, options(true, true)).unwrap();
        assert_full(name, &result, cp);
        check_golden("boolean-entry", name, &result);
        let recognition =
            parse_entry(Some("BooleanExpression"), &text, options(false, true)).unwrap();
        assert_full(name, &recognition, cp);
        assert!(recognition.ast.is_none());
    }
}

#[test]
fn entry_selection_matches_root_parser() {
    // build.rs の複製 parser（parse_entry）と commit 済み生成物（parse_formula）が同じ結果を返す。
    for name in ["complex", "valid-unicode", "large-match"] {
        let text = fixture(name);
        let root_result = strip_statistics(parse_formula(&text, options(true, true)));
        for entry in [None, Some("Formula")] {
            let entry_result =
                strip_statistics(parse_entry(entry, &text, options(true, true)).unwrap());
            assert_eq!(entry_result, root_result, "{name}: entry={entry:?}");
        }
    }
    assert!(parse_entry(Some("NoSuchRule"), "1", options(true, true)).is_err());
}

#[test]
fn extern_scanners_are_reached() {
    // STRING は複数 fixture に現れる。scanner 未登録（RejectExtern）だと文字列宣言で失敗する。
    let text = fixture("complex");
    let rejected = ubnfc_p4_rust::parse_with_options(&text, options(true, true));
    assert!(!rejected.ok, "RejectExtern では文字列を受理しない");
    let accepted = parse_formula(&text, options(true, true));
    assert!(accepted.ok);
    let json = accepted.ast.unwrap().canonical_json();
    // scanners/rust の移植集合は単引用符だけ value から除く（QuotedParser 由来の規約）。
    // Java 側の生成物は字句のまま `'jp'` を返すので、この点だけ golden が異なる。
    assert!(json.contains("\"jp\""), "{json}");
    assert!(!json.contains("'jp'"), "{json}");
    let double = parse_formula("var $c as string set \"jp\";\n1", options(true, true));
    assert!(double.ok, "{:?}", double.diagnostics);
    assert!(double
        .ast
        .unwrap()
        .canonical_json()
        .contains("\"\\\"jp\\\"\""));
    // CODE_START / CODE_END は行頭の ``` 識別子:識別子 と ``` で囲む。
    let code = "```java:org.example.Fn\nreturn 1;\n```\n1";
    let result = parse_formula(code, options(true, true));
    assert!(result.ok, "{:?}", result.diagnostics);
    assert_eq!(result.consumed_cp, code.chars().count());
    assert!(!parse_formula("```java:org.example.Fn\nreturn 1;\n1", options(true, true)).ok);
}

/// Java golden の単引用符付き文字列値 `'jp'` を `jp` に正規化し、正規化した個数を返す。
fn strip_single_quotes(value: &mut serde_json::Value) -> usize {
    match value {
        serde_json::Value::String(s)
            if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') =>
        {
            *s = s[1..s.len() - 1].to_owned();
            1
        }
        serde_json::Value::Array(items) => items.iter_mut().map(strip_single_quotes).sum(),
        serde_json::Value::Object(fields) => fields.values_mut().map(strip_single_quotes).sum(),
        _ => 0,
    }
}

#[test]
fn java_goldens_agree_except_single_quoted_string_values() {
    // Java 側 golden（examples/p4-java）と serde_json::Value として比較する。key の順序は無視。
    // 唯一の差は文字列値の単引用符（Rust scanner 集合は除く、Java は字句のまま）。
    let java = root().join("../p4-java/src/test/resources/golden");
    if !java.is_dir() {
        eprintln!("Java golden がないため比較を省略: {}", java.display());
        return;
    }
    let compare = |sub: &str, name: &str, quoted: usize| {
        let rust: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                root()
                    .join("tests/golden")
                    .join(sub)
                    .join(format!("{name}.tiny.json")),
            )
            .unwrap(),
        )
        .unwrap();
        let mut java: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(java.join(sub).join(format!("{name}.tiny.json"))).unwrap(),
        )
        .unwrap();
        if quoted == 0 {
            assert_eq!(rust, java, "{sub}/{name}: Java golden と構造が異なる");
        } else {
            assert_ne!(rust, java, "{sub}/{name}: 生の golden は一致しないはず");
        }
        assert_eq!(
            strip_single_quotes(&mut java),
            quoted,
            "{sub}/{name}: 単引用符の文字列値の数"
        );
        assert_eq!(
            rust, java,
            "{sub}/{name}: 単引用符を除いても Java golden と異なる"
        );
    };
    // 単引用符付き文字列値の個数（complex 3、x4 12、x16 48、x64 192、valid-unicode 1）。
    for (name, quoted) in [
        ("complex", 3),
        ("complex-x4", 12),
        ("complex-x16", 48),
        ("complex-x64", 192),
        ("flat-arithmetic", 0),
        ("large-match", 0),
        ("valid-basic", 0),
        ("valid-multiline", 0),
        ("valid-unicode", 1),
    ] {
        compare("", name, quoted);
    }
    for &(name, _) in BOOLEAN_ENTRY {
        compare("boolean-entry", name, 0);
    }
}

#[test]
fn fixtures_are_unmodified_copies() {
    // 収録した 16 fixture の一覧。出典は tinyexpression（README 参照）。
    let dir = root().join("tests/fixtures");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    let mut expected: Vec<String> = ACCEPTED
        .iter()
        .map(|(n, _)| *n)
        .chain(REJECTED.iter().copied())
        .map(|n| format!("{n}.tiny"))
        .collect();
    expected.sort();
    assert_eq!(names, expected);
    assert!(Path::new(&dir).join("complex.tiny").is_file());
}

/// §3.4 の候補除外（`ParseOptions::predict`）は参照経路（全候補を試行）と結果・診断が同じ。
/// 全 fixture の完全入力に加え、切り詰め（各 byte 境界）と 1 文字破壊（`#`）で失敗経路を通す。
/// 大きい fixture は間引く。memo の有無とも掛け合わせる。
#[test]
fn predict_matches_reference_path_on_truncated_and_corrupted_inputs() {
    let all: Vec<&str> = ACCEPTED
        .iter()
        .map(|(n, _)| *n)
        .chain(REJECTED.iter().copied())
        .collect();
    let mut compared = 0usize;
    for name in all {
        let text = fixture(name);
        let entry = if name.starts_with("comparison-heavy") {
            Some("BooleanExpression")
        } else {
            None
        };
        let stride = match text.len() {
            0..=400 => 1,
            401..=1600 => 7,
            1601..=6000 => 97,
            _ => 1201,
        };
        let mut inputs: Vec<String> = vec![text.clone()];
        let mut i = 0;
        while i < text.len() {
            if text.is_char_boundary(i) {
                inputs.push(text[..i].to_owned());
                let mut corrupted = text[..i].to_owned();
                corrupted.push('#');
                let rest = &text[i..];
                let next = rest.chars().next().map_or(0, char::len_utf8);
                corrupted.push_str(&rest[next..]);
                inputs.push(corrupted);
            }
            i += stride;
        }
        for (index, input) in inputs.iter().enumerate() {
            for memo in [true, false] {
                // memo 無効の参照経路は遅い（complex で rule 評価 16 万回）ので間引く。
                if !memo && (input.len() > 400 || index % 13 != 0) {
                    continue;
                }
                let run = |predict: bool| {
                    let mut options = options(true, memo);
                    options.predict = predict;
                    strip_statistics(match entry {
                        None => parse_formula(input, options),
                        Some(entry) => parse_entry(Some(entry), input, options).unwrap(),
                    })
                };
                let (on, off) = (run(true), run(false));
                assert!(
                    on == off,
                    "{name}: predict on/off differ (memo {memo}) for input {:?}\non:  {:?}\noff: {:?}",
                    input,
                    on.diagnostics,
                    off.diagnostics
                );
                compared += 1;
            }
        }
    }
    assert!(compared > 1000, "{compared}");
}
