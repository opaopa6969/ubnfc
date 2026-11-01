//! SWAR の word 境界、UTF-8、escape、反復下限と失敗時診断を scan on/off で比較する。
use std::path::PathBuf;
use std::process::Command;
use ubnfc_fuzz::engine::Engine;
use ubnfc_fuzz::model::{Atom, Body, Element, Grammar, Quant, Rule, Token};
use ubnfc_fuzz::observe::differences;
use ubnfc_harness::runner::Backend;

#[test]
fn swar_scan_matches_reference_at_word_and_unicode_boundaries() {
    let output = Command::new("mktemp").arg("-d").output().unwrap();
    assert!(output.status.success());
    let dir = PathBuf::from(String::from_utf8(output.stdout).unwrap().trim());
    let reference = |s: &str| Element::new(Atom::Ref(s.into()));
    let mut digits = reference("DIGIT");
    digits.quant = Some(Quant::Plus);
    let grammar = Grammar {
        name: "Swar".into(),
        settings: vec![
            "@ubnf: 2".into(),
            "@package: fuzz.swar".into(),
            "@whitespace: none".into(),
        ],
        templates: vec![],
        tokens: vec![
            Token {
                name: "SAFE".into(),
                declaration: "NEGATION('\"\\\\')".into(),
                samples: vec![],
                feature: "scan",
            },
            Token {
                name: "DIGIT".into(),
                declaration: "DIGIT".into(),
                samples: vec![],
                feature: "scan",
            },
        ],
        rules: vec![
            Rule::plain(
                vec!["@root".into(), "@mapping(Root, params=[text])".into()],
                "Root".into(),
                Body::single(vec![Element::captured(Atom::Ref("Content".into()), "text")]),
            ),
            Rule::plain(
                vec![],
                "Content".into(),
                Body::single(vec![
                    Element::new(Atom::Literal("\"".into())),
                    Element::new(Atom::Repeat(Body {
                        alternatives: vec![
                            vec![Element::new(Atom::Literal("\\".into())), reference("SAFE")],
                            vec![reference("SAFE")],
                        ],
                    })),
                    Element::new(Atom::Literal("\"".into())),
                    digits,
                ]),
            ),
        ],
        import: None,
        features: Default::default(),
    };
    let mut engine = Engine::new(dir.join("build"));
    let ir_path = engine.compile_front(&grammar).expect("文法を解析");
    let ir: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(ir_path).unwrap()).unwrap();
    assert!(
        ir["plan"]["scans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["class"].is_object()),
        "bulk scan が実際に存在する"
    );
    let built = engine.prepare(&grammar).expect("生成と build");
    let mut inputs = Vec::new();
    // 長さ 0..40 の全境界と各停止文字。ASCII / UTF-8 の後で再び word 走査へ入る。
    for len in 0..=40 {
        for stop in ["", "あ", "😀", "\\a", "\"", "\\", "\u{7f}", "\u{80}", "\n"] {
            let body = format!("{}{}{}", "a".repeat(len), stop, "z".repeat(40 - len));
            for suffix in ["12345678901234567", "", "12345678x"] {
                inputs.push(format!("\"{body}\"{suffix}"));
            }
        }
    }
    // 固定 seed の変異入力。scan の失敗・診断再走査も比較する。
    let alphabet = ['a', '0', '9', '"', '\\', '\n', 'あ', '😀'];
    let mut seed = 3u64;
    for _ in 0..256 {
        let mut s = String::from("\"");
        for _ in 0..64 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            s.push(alphabet[((seed >> 32) as usize) % alphabet.len()]);
        }
        s.push_str("\"123456789");
        inputs.push(s);
    }
    let on = engine.run_scan(&built, Backend::Rust, &inputs, true);
    let off = engine.run_scan(&built, Backend::Rust, &inputs, false);
    for ((input, a), b) in inputs.iter().zip(on).zip(off) {
        assert!(
            a.get("error").is_none() && b.get("error").is_none(),
            "未観測: {a} / {b}"
        );
        assert!(
            differences(&a, &b).is_empty(),
            "{input:?}: {:?}",
            differences(&a, &b)
        );
    }
    assert_eq!(inputs.len(), 1363);
    std::fs::remove_dir_all(dir).unwrap();
}
