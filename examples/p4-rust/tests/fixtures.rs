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
/// 出現表と value span は opt-in（D-035）。差分テストはこれらも比較対象に残すため、
/// 参照経路・最適化経路のどちらにも要求する。
fn observed(options: ParseOptions) -> ParseOptions {
    ParseOptions {
        occurrences: true,
        value_spans: true,
        ..options
    }
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

/// D-035: 出現表（`captures` / `lexical` / `tokens`）と `value_spans` は opt-in。
/// 既定では空になり、それ以外の観測（AST・node_spans・診断・scope・recovery・統計）は
/// 要求したときと完全に一致する。要求すれば従来どおりの内容が返る。
#[test]
fn occurrences_and_value_spans_are_opt_in_without_changing_anything_else() {
    let cases: Vec<(&str, Option<&str>)> = ACCEPTED
        .iter()
        .map(|(n, _)| (*n, None))
        .chain(REJECTED.iter().map(|n| (*n, None)))
        .chain([
            ("comparison-heavy", Some("BooleanExpression")),
            ("complex-x4", Some("Formula")),
        ])
        .collect();
    let mut with_tables = 0usize;
    for (name, entry) in cases {
        let text = fixture(name);
        for build_ast in [true, false] {
            for lexical in [false, true] {
                let base = ParseOptions {
                    lexical,
                    ..options(build_ast, true)
                };
                let run = |options: ParseOptions| {
                    strip_statistics(match entry {
                        None => parse_formula(&text, options),
                        Some(entry) => parse_entry(Some(entry), &text, options).unwrap(),
                    })
                };
                let default = run(base);
                let requested = run(observed(base));
                // 既定で空になるのは出現表と value span だけ。value span は常に opt-in。
                assert!(
                    default.value_spans.is_empty(),
                    "{name}: value_spans 既定 off"
                );
                let mut stripped = requested.clone();
                stripped.value_spans.clear();
                if lexical {
                    // lexical mode は出現表そのものが目的なので要求とみなす（D-035）。
                    assert_eq!(default.captures, requested.captures, "{name}: lexical mode");
                    assert_eq!(default.lexical, requested.lexical, "{name}: lexical mode");
                    assert_eq!(default.tokens, requested.tokens, "{name}: lexical mode");
                } else {
                    assert!(default.captures.is_empty(), "{name}: captures 既定 off");
                    assert!(
                        default.lexical.is_empty() && default.tokens.is_empty(),
                        "{name}"
                    );
                    stripped.captures.clear();
                    stripped.lexical.clear();
                    stripped.tokens.clear();
                }
                assert!(
                    default == stripped,
                    "{name}: 出現表と value span 以外が変わった (ast {build_ast} / lexical {lexical})"
                );
                if requested.ok && build_ast {
                    assert!(!requested.captures.is_empty(), "{name}: 要求時は出現が出る");
                    assert!(
                        !requested.value_spans.is_empty(),
                        "{name}: 要求時は value span が出る"
                    );
                    with_tables += 1;
                }
            }
        }
    }
    assert!(with_tables >= 8, "{with_tables}");
}

#[test]
fn default_diagnostic_budget_accepts_every_fixture_and_small_budget_fails_as_resource() {
    // M5: `limits.diagnostics` は「記録した診断の件数」の上限。主診断 DAG の node 数と、
    // 公開 farthest 候補（label）の記録数をそれぞれ数え、どちらかが超えると Resource 診断で
    // 全体失敗になる。失敗 1 件が O(1) 件になったので、既定 2,000,000 で全 fixture が通る。
    // fast mode（既定）は診断を記録しないので予算を消費しない。予算の契約は診断モードで検証する。
    let default = ParseOptions {
        diagnostics: true,
        ..Default::default()
    };
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
            diagnostics: true,
            ..Default::default()
        },
    );
    assert!(no_memo.ok, "{:?}", no_memo.diagnostics);
    // 予算を小さくすると、認識は正しく進んでも Resource 診断 1 件で全体失敗になる。
    let mut small = ParseOptions {
        diagnostics: true,
        ..Default::default()
    };
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
    // scanners/rust の移植集合は単引用符だけ value から除く（QuotedParser 由来の規約、
    // `unlaxer-dsl/specs/generators.md` #132）。Java backend も同じ規則
    // （共有 registry `scanners/java/Registry.java` と `examples/p4-java/P4Scanners.java`）。
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

/// `scanners/first-chars.json` の宣言（M5 round 3）は必要条件: 宣言した集合の外の先頭文字では
/// 同梱 scanner が消費なしに失敗する。生成器はこの前提で Extern で始まる候補を除外する。
#[test]
fn extern_first_chars_declaration_matches_bundled_scanners() {
    use ubnfc_p4_rust::{rt::scope::ScopeStore, scanners::Registry, State, TokenScanner};
    let path = root().join("../../scanners/first-chars.json");
    let text = std::fs::read_to_string(&path).unwrap();
    let decl: serde_json::Value = serde_json::from_str(&text).unwrap();
    let scanners = decl["scanners"].as_object().unwrap();
    let classes: &'static [(&'static str, &'static str)] = Box::leak(
        scanners
            .keys()
            .map(|class| {
                let class: &'static str = Box::leak(class.clone().into_boxed_str());
                (class, class)
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let mut registry = Registry::new(classes).expect("宣言した全クラスは同梱 registry にある");
    let scope = ScopeStore::new();
    let mut checked = 0usize;
    for (class, ranges) in scanners {
        let Some(ranges) = ranges.as_array() else {
            continue;
        };
        let allowed: Vec<(u32, u32)> = ranges
            .iter()
            .map(|r| match r {
                serde_json::Value::String(s) => {
                    let c = s.chars().next().unwrap() as u32;
                    (c, c)
                }
                serde_json::Value::Number(n) => {
                    let c = u32::try_from(n.as_u64().unwrap()).unwrap();
                    (c, c)
                }
                serde_json::Value::Object(o) => {
                    let cp = |v: &serde_json::Value| match v {
                        serde_json::Value::String(s) => s.chars().next().unwrap() as u32,
                        other => u32::try_from(other.as_u64().unwrap()).unwrap(),
                    };
                    (cp(&o["start"]), cp(&o["end"]))
                }
                other => panic!("{other}"),
            })
            .collect();
        let outside = (0x20u32..0x7f)
            .chain([0x0a, 0x09, 0xe9, 0x3042, 0x1f600])
            .filter_map(char::from_u32)
            .filter(|c| {
                !allowed
                    .iter()
                    .any(|&(lo, hi)| (lo..=hi).contains(&(*c as u32)))
            });
        for c in outside {
            for (prefix, consumed) in [("", 0usize), ("ab\n", 3)] {
                let input = format!("{prefix}{c}xyz\n```java:a.b\n");
                for (match_only, reset) in [(false, true), (true, true), (true, false)] {
                    let state = State {
                        consumed,
                        matched: consumed,
                        invert: false,
                        reset,
                    };
                    let result = registry.scan(class, &input, state, match_only, &scope);
                    assert!(
                        !result.ok && result.consumed_end == consumed && result.matched_end == consumed,
                        "{class}: 宣言外の先頭文字 {c:?}（match_only {match_only}, reset {reset}）で消費なしに失敗する: {result:?}"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 1000, "{checked}");
}

/// **両 backend の canonical AST は正規化なしで完全一致する。**
///
/// 2026-09-23 のクロスチェック（`docs/reports/2026-09-23-parser-feature-correctness-comparison.md`
/// §3.2 結果(2)）まで、この比較は「単引用符を除けば一致する」形で書かれており、
/// `strip_single_quotes` が Java golden の `'jp'` を `jp` に直してから比べ、
/// 直した個数（complex 3 / x4 12 / x16 48 / x64 192 / valid-unicode 1 = **合計 256**）を
/// 期待値として固定していた。つまり **backend 間の実際の不一致が、比較の側で
/// 期待値に変換されていた**。corpus と差分 fuzzer が見つけられなかったのは、どちらも
/// 共有 registry（`scanners/java/Registry.java` と `scanners/rust/scanners.rs`、
/// `crates/ubnfc-harness/src/runner/scanners.rs` が両方に注入する）を使い、
/// 共有 registry は両言語とも単引用符を value から除くため。差が出ていたのは
/// `examples/p4-java` が同じ 3 つの Extern scanner を独自に書き直した
/// `P4Scanners.java` だけで、そこは corpus も fuzzer も通らない。
///
/// `P4Scanners.valueSpan` を共有 registry と同じ規則
/// （`unlaxer-dsl/specs/generators.md` #132「外側空白の strip と単引用符だけの除去」）に
/// 直した今、正規化は一切しない。差がまた入れば、ここが落ちる。
#[test]
fn java_and_rust_goldens_are_identical_without_normalization() {
    // key の順序は serde_json::Value の比較で吸収される（Java は type/span/fields 順）。
    let java = root().join("../p4-java/src/test/resources/golden");
    if !java.is_dir() {
        eprintln!("Java golden がないため比較を省略: {}", java.display());
        return;
    }
    let read = |path: std::path::PathBuf| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
        )
        .unwrap()
    };
    let mut compared = 0;
    let mut compare = |sub: &str, name: &str| {
        let file = format!("{name}.tiny.json");
        assert_eq!(
            read(root().join("tests/golden").join(sub).join(&file)),
            read(java.join(sub).join(&file)),
            "{sub}/{name}: Java backend と Rust backend の canonical AST が一致しません"
        );
        compared += 1;
    };
    for &(name, _) in ACCEPTED {
        compare("", name);
    }
    for &(name, _) in BOOLEAN_ENTRY {
        compare("boolean-entry", name);
    }
    assert_eq!(compared, ACCEPTED.len() + BOOLEAN_ENTRY.len());
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

/// §3.4 の候補除外（`ParseOptions::predict`）と位置ごとの字句 cache（`ParseOptions::lexical_cache`）は
/// 参照経路（全候補を試行・cache なし）と結果・診断が同じ。
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
                let run = |predict: bool, factor: bool, lexical_cache: bool| {
                    let mut options = observed(options(true, memo));
                    options.predict = predict;
                    options.factor = factor;
                    options.lexical_cache = lexical_cache;
                    strip_statistics(match entry {
                        None => parse_formula(input, options),
                        Some(entry) => parse_entry(Some(entry), input, options).unwrap(),
                    })
                };
                // fast mode（既定、診断を記録しない）と診断モードの同値。診断が要る結果
                // （失敗・入力の残り・recovery / mapping / resource 診断）は入口が解析し直すので
                // hints まで完全一致し、完全成功では hints（投機的失敗）だけが fast mode で空になる。
                let fast = run(true, true, false);
                let diagnostic = {
                    let mut options = observed(options(true, memo));
                    options.diagnostics = true;
                    strip_statistics(match entry {
                        None => parse_formula(input, options),
                        Some(entry) => parse_entry(Some(entry), input, options).unwrap(),
                    })
                };
                let escalated =
                    !fast.ok || !fast.diagnostics.is_empty() || !fast.recoveries.is_empty();
                if escalated {
                    assert!(
                        fast == diagnostic,
                        "{name}: fast mode の再解析が診断モードと異なる (memo {memo}) for input {:?}",
                        input
                    );
                } else {
                    assert!(
                        fast.hints.is_empty(),
                        "{name}: fast mode が hints を返した for input {:?}",
                        input
                    );
                    let mut expected = diagnostic.clone();
                    expected.hints.clear();
                    assert!(
                        fast == expected,
                        "{name}: fast mode と診断モードが hints 以外で異なる (memo {memo}) for input {:?}",
                        input
                    );
                }
                compared += 1;
                // 参照経路（候補除外なし・因数分解なし・字句 cache なし）と、各最適化の組合せを比較する。
                let reference = run(false, false, false);
                for (predict, factor, lexical_cache) in [
                    (true, true, false),
                    (true, false, false),
                    (false, true, false),
                    // 字句 cache（位置ごとの統合 DFA）の on/off 軸。
                    (false, false, true),
                    (true, true, true),
                    (true, false, true),
                    (false, true, true),
                ] {
                    let optimized = run(predict, factor, lexical_cache);
                    assert!(
                        optimized == reference,
                        "{name}: predict {predict} / factor {factor} / lexical_cache {lexical_cache} differ from reference (memo {memo}) for input {:?}\nopt: {:?}\nref: {:?}",
                        input,
                        optimized.diagnostics,
                        reference.diagnostics
                    );
                    compared += 1;
                }
            }
        }
    }
    assert!(compared > 3000, "{compared}");
}
