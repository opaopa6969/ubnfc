//! 別置き文法と backend ごとの明示 oracle。比較元の parser は実行しない。
use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsExpectation {
    pub kind: String,
    /// 共に code point。neg(1e+) の 7 / 5 は既存 backend の意図的な相違。
    pub java_offset: usize,
    pub rust_offset: usize,
    #[serde(flatten)]
    pub extra: RawFields,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumericExpectation {
    /// 全行が NumberParser の構文として有効。None は Java AST の
    /// Integer.parseInt が NumberFormatException（構文不受理ではない）。
    pub java_int: Option<i32>,
    /// Rust は符号・先頭ゼロなども含め元の数値字句を文字列で保持する。
    pub rust_lexeme: String,
    pub extra: RawFields,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumericRow {
    pub input: String,
    pub java_int: Option<i32>,
    #[serde(flatten)]
    pub extra: RawFields,
}

pub(crate) fn evolution(path: &Path, diagnostics: bool) -> Result<CorpusFile, LoadError> {
    if diagnostics {
        let (rows, raw) = read::<Vec<Row<DiagnosticsExpectation>>>(path)?;
        let mut out = file(Family::ExternalGrammar, path, raw);
        out.grammars.push(grammar_path(
            path.with_file_name("3").join("Evolution.ubnf"),
            "3",
        )?);
        out.cases = cases(rows, "diagnostics", vec![0], None, Expectation::Diagnostics);
        Ok(out)
    } else {
        let (inputs, raw) = read::<Vec<String>>(path)?;
        let mut out = file(
            Family::ExternalGrammar,
            path,
            serde_json::json!({"inputs":raw,"cases":[]}),
        );
        for stage in 0..4 {
            let stage = stage.to_string();
            let oracle_path = oracle::default_root().join(format!("evolution-v{stage}.json"));
            let measured = oracle::load(&oracle_path)?;
            if measured.cases.len() != inputs.len() {
                return Err(oracle::invalid(
                    &oracle_path,
                    "evolution の入力件数が一致しません",
                ));
            }
            let grammar_file = path.with_file_name(&stage).join("Evolution.ubnf");
            if measured.raw["grammar"].as_str() != Some(read_text(&grammar_file)?.as_str()) {
                return Err(oracle::invalid(
                    &oracle_path,
                    "evolution の採取文法が一致しません",
                ));
            }
            let grammar_index = out.grammars.len();
            out.grammars.push(grammar_path(grammar_file, &stage)?);
            for (i, (mut case, input)) in measured.cases.into_iter().zip(&inputs).enumerate() {
                if &case.input != input || case.id != format!("evolution/{i}") {
                    return Err(oracle::invalid(
                        &oracle_path,
                        "evolution の入力順序/ID が一致しません",
                    ));
                }
                if !matches!(case.expect, Expectation::Oracle { .. }) {
                    return Err(oracle::invalid(
                        &oracle_path,
                        "evolution の実走 oracle がありません",
                    ));
                }
                case.id = format!("evolution/v{stage}/{i}");
                case.grammar_ref = vec![grammar_index];
                let mut row = measured.raw["cases"][i].clone();
                row["id"] = case.id.clone().into();
                out.raw["cases"].as_array_mut().unwrap().push(row);
                out.cases.push(case);
            }
        }
        Ok(out)
    }
}

pub(crate) fn numeric(path: &Path) -> Result<CorpusFile, LoadError> {
    let (rows, raw) = read::<Vec<NumericRow>>(path)?;
    let mut out = file(Family::ExternalGrammar, path, raw);
    out.grammars.push(GrammarSource::Inline {
        name: "Numeric".into(),
        metadata: Value::Null,
        source: "grammar Numeric { @package: numeric @whitespace: javaStyle\n\
            token NUMBER = NumberParser\n@root @mapping(Item, params=[value])\n\
            Item ::= 'n' NUMBER @value ';' ; }"
            .into(),
    });
    out.cases = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| Case {
            id: format!("numeric/{i}"),
            grammar_ref: vec![0],
            entry_rule: Some("Item".into()),
            // NumericCaptureRuntimeTest が parse する実際の入力。字句は期待値と raw に保持。
            input: format!("n{};", row.input),
            expect: Expectation::Numeric(NumericExpectation {
                java_int: row.java_int,
                rust_lexeme: row.input,
                extra: row.extra,
            }),
            known_divergences: Vec::new(),
        })
        .collect();
    Ok(out)
}

pub(crate) fn values(path: &Path, grammar: &str) -> Result<CorpusFile, LoadError> {
    let (rows, raw) = read::<Vec<Row<rule::RuleExpectation>>>(path)?;
    let grammar_file = path.with_file_name(grammar);
    let source = read_text(&grammar_file)?;
    let mut out = file(Family::ExternalGrammar, path, raw);
    let (base_name, entry) = match grammar {
        "Operators.ubnf" => ("normal", "Expression"),
        "Power.ubnf" => ("text", "Expr"),
        _ => ("zero", "Root"),
    };
    out.grammars.push(grammar_path(grammar_file, base_name)?);
    let mut variant = |name: &str, source: String| {
        out.grammars.push(GrammarSource::Inline {
            name: name.into(),
            source,
            metadata: Value::Null,
        })
    };
    // 文法の fixture 展開だけ。生成・コンパイル・実行には踏み込まない。
    match grammar {
        "Operators.ubnf" => variant("reversed-metadata", source.replace("level=10", "level=30")),
        "Power.ubnf" => {
            for mode in ["mapped", "unicode", "grouped"] {
                let mut text = source.replace(
                    "Atom ::= NUMBER;",
                    &format!(
                        "@mapping(Number, params=[value]) Atom ::= {}(NUMBER) @value;",
                        if mode == "unicode" { "[ '😀' ] " } else { "" }
                    ),
                );
                if mode == "grouped" {
                    text = text
                        .replace("Expr ::= Atom @left", "Expr ::= Base @left")
                        .replace("  @root", "  Base ::= Atom | '(' Expr ')';\n  @root");
                }
                variant(mode, text);
            }
        }
        _ => {
            for (name, quantifier) in [
                ("plus", "Item+"),
                ("bounded", "Item{1,2}"),
                ("unbounded", "Item{1,}"),
                ("separated", "Item % ','"),
            ] {
                variant(name, source.replace("{ Item }", quantifier));
            }
        }
    }
    out.cases = cases(
        rows,
        grammar,
        (0..out.grammars.len()).collect(),
        Some(entry),
        Expectation::RuleValue,
    );
    Ok(out)
}
