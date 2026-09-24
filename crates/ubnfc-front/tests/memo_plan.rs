//! rule 本体 memo の要否（issue #17）。memo が当たるのは同じ `(rule, position)` が
//! 2 回試されたときだけなので、巻戻しで再入し得ない規則には付けない。
//!
//! Operators / Power は 1 度も当たらない（512 atom の rule 呼出し 2,561 件すべて miss、
//! 認識専用で 0.221 → 0.180 ms）。P4 は外すと 67 倍（0.219 → 14.69 ms）悪化するので、
//! 塔の規則に memo が残ることをここで固定する。
use ubnfc_front::analysis::analyze;
use ubnfc_front::ir::{from_json, GrammarIr, MemoEntry, MemoEntryReason, ProofKind};

fn fixture(name: &str) -> GrammarIr {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../ir/fixtures")
            .join(name),
    )
    .unwrap_or_else(|e| panic!("{name}: {e}"));
    from_json(&text).unwrap()
}

/// golden の plan と、その semantics を解析し直した plan の両方から rule 本体の memo を読む。
fn bodies(ir: &GrammarIr) -> Vec<(String, MemoEntry, MemoEntry)> {
    let (_, plan) = analyze(&ir.semantics).unwrap();
    ir.semantics
        .rules
        .iter()
        .map(|r| {
            let pick = |entries: &[MemoEntry]| {
                entries
                    .iter()
                    .find(|m| m.expr_id == r.body)
                    .unwrap_or_else(|| panic!("{}: memo entry がない", r.symbol_id))
                    .clone()
            };
            (
                r.symbol_id.clone(),
                pick(&ir.plan.memo.entries),
                pick(&plan.memo.entries),
            )
        })
        .collect()
}

const OPERATORS: &str = "unlaxer-dsl__src__test__resources__associative__Operators.ubnf.ir.json";
const POWER: &str = "unlaxer-dsl__src__test__resources__right-associative__Power.ubnf.ir.json";
const P4: &str = "tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json";

#[test]
fn operators_and_power_drop_every_rule_body_memo() {
    for (name, count) in [(OPERATORS, 6), (POWER, 2)] {
        let ir = fixture(name);
        let rules = bodies(&ir);
        assert_eq!(rules.len(), count, "{name}");
        for (rule, golden, fresh) in rules {
            assert_eq!(golden, fresh, "{rule}: golden と再解析が一致しない");
            assert!(
                !golden.cache_success && !golden.cache_failure,
                "{rule}: memo が残っている"
            );
            assert_eq!(golden.reason, MemoEntryReason::ProvedOmission, "{rule}");
            assert_eq!(
                golden.omission_proof.as_ref().unwrap().kind,
                ProofKind::UnreachableRevisit,
                "{rule}"
            );
        }
    }
}

/// 外すと 67 倍になる塔。`Expression` は 6 候補が同じ位置から下の段を順に試すので、
/// 候補間で同じ `(rule, position)` が何度も再試行される（complex 1 parse あたり hit 464 件）。
#[test]
fn p4_keeps_the_expression_tower_memoized() {
    let ir = fixture(P4);
    let rules = bodies(&ir);
    assert_eq!(rules.len(), 124);
    let tower = [
        "Expression",
        "ArgumentExpression",
        "NumberExpression",
        "NumberTerm",
        "NumberFactor",
        "BooleanExpression",
        "BooleanAndExpression",
        "BooleanXorExpression",
        "BooleanFactor",
        "BooleanComparable",
        "StringExpression",
        "StringTerm",
        "ComparisonExpression",
        "StringComparisonExpression",
        "VariableRef",
        "MethodInvocation",
    ];
    for name in tower {
        let id = format!("TinyExpressionP4::{name}");
        let (_, golden, fresh) = rules
            .iter()
            .find(|(rule, _, _)| *rule == id)
            .unwrap_or_else(|| panic!("{id} がない"));
        assert_eq!(golden, fresh, "{id}: golden と再解析が一致しない");
        assert!(
            golden.cache_success && golden.cache_failure,
            "{id}: memo が外れている（外すと 67 倍）"
        );
        assert_eq!(golden.reason, MemoEntryReason::DefaultSafe, "{id}");
    }
    // 全部残すわけではない。型ヒントのような葉は再入しないので落ち（unreachableRevisit）、
    // 再入し得ても本体が終端走査だけの規則は割に合わないので落ちる（unprofitableTerminalScan）。
    let dropped: Vec<_> = rules
        .iter()
        .filter(|(_, golden, _)| !golden.cache_success)
        .map(|(rule, golden, _)| {
            (
                rule.rsplit("::").next().unwrap(),
                golden.omission_proof.as_ref().unwrap().kind,
            )
        })
        .collect();
    let revisit = ProofKind::UnreachableRevisit;
    let terminal = ProofKind::UnprofitableTerminalScan;
    assert_eq!(
        dropped,
        [
            ("ImportDeclaration", revisit),
            ("ClassName", terminal),
            ("TypeHint", revisit),
            ("NumberTypeHint", revisit),
            ("StringTypeHint", revisit),
            ("BooleanTypeHint", revisit),
            ("ObjectTypeHint", revisit),
            ("OnlyIfAbsent", terminal),
            ("NumberReturnType", terminal),
            ("StringReturnType", terminal),
            ("BooleanReturnType", terminal),
            ("ObjectReturnType", terminal),
            ("MethodInvocationHeader", terminal),
            ("AddOp", terminal),
            ("MulOp", terminal),
            ("RandomFunction", terminal),
            ("DayOfWeek", terminal),
            ("StringCastVariable", terminal),
            ("StringTypedVariable", terminal),
            ("EqualityOp", terminal),
            ("CompareOp", terminal),
            ("TypeKeyword", terminal),
        ]
    );
}
