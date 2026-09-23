//! UBNF v2 のバージョン門と、左再帰ルールの優先順位塔への脱糖。
//!
//! `@ubnf: 1|2`（既定 1）。v1 の意味論は凍結されているので、この module は
//! **v2 を宣言した grammar でだけ**規則を書き換える。v1 の grammar に対しては
//! 「候補先頭注釈が置かれていないこと」を検査するだけで、AST を変更しない。
//!
//! 脱糖は resolve より前（構文木の段階）で行う。したがって下流（resolve /
//! validate / lower / analysis / backend）は v1 の正規形しか見ない。これが
//! 「v1 の挙動が bit 単位で不変」を構造的に保証する方法である（D-038 / D-039）。

use std::collections::BTreeSet;

use crate::syntax::ast::*;
use crate::validate::ValidationIssue;

pub const DEFAULT_LANGUAGE_VERSION: u64 = 1;
pub const LATEST_LANGUAGE_VERSION: u64 = 2;

fn issue(
    code: &str,
    message: String,
    hint: &str,
    rule: Option<&str>,
    grammar: &str,
) -> ValidationIssue {
    ValidationIssue {
        code: code.into(),
        message,
        hint: hint.into(),
        rule: rule.map(ToOwned::to_owned),
        grammar: grammar.into(),
    }
}

/// `@ubnf` 設定の値。宣言が無ければ 1。不正値は issue を積んで 1 を返す。
pub fn language_version(decl: &GrammarDecl, issues: &mut Vec<ValidationIssue>) -> u64 {
    language_version_with(&decl.name, &decl.settings, issues)
}

/// resolve 後の `Grammar.settings` からも同じ値を得るための形。
pub fn language_version_of(settings: &[GlobalSetting], issues: &mut Vec<ValidationIssue>) -> u64 {
    language_version_with("", settings, issues)
}

fn language_version_with(
    decl_name: &str,
    settings: &[GlobalSetting],
    issues: &mut Vec<ValidationIssue>,
) -> u64 {
    let mut version = DEFAULT_LANGUAGE_VERSION;
    let mut seen = false;
    for setting in settings {
        let (GlobalSetting::String { key, .. } | GlobalSetting::Block { key, .. }) = setting;
        if key != "ubnf" {
            continue;
        }
        if seen {
            issues.push(issue(
                "E-UBNF-VERSION-DUPLICATE",
                "@ubnf は grammar ごとに一度だけ宣言できます".into(),
                "余分な @ubnf 設定を削除してください。",
                None,
                decl_name,
            ));
            continue;
        }
        seen = true;
        let value = match setting {
            GlobalSetting::String { value, .. } => value.parse::<u64>().ok(),
            GlobalSetting::Block { .. } => None,
        };
        match value {
            Some(v) if (1..=LATEST_LANGUAGE_VERSION).contains(&v) => version = v,
            _ => issues.push(issue(
                "E-UBNF-VERSION-UNSUPPORTED",
                format!(
                    "@ubnf は 1..{LATEST_LANGUAGE_VERSION} のいずれかです: {}",
                    match setting {
                        GlobalSetting::String { value, .. } => value.clone(),
                        GlobalSetting::Block { .. } => "{...}".into(),
                    }
                ),
                "'@ubnf: 1' か '@ubnf: 2' を宣言してください。",
                None,
                decl_name,
            )),
        }
    }
    version
}

/// ファイル内の全 grammar を脱糖し、宣言された最大の言語バージョンを返す。
pub fn file(ast: &mut UbnfFile, issues: &mut Vec<ValidationIssue>) -> u64 {
    let mut max = DEFAULT_LANGUAGE_VERSION;
    for decl in &mut ast.grammars {
        let version = language_version(decl, issues);
        crate::boundary::validate(decl, version, issues);
        max = max.max(version);
        grammar(decl, version, issues);
    }
    max
}

fn grammar(decl: &mut GrammarDecl, version: u64, issues: &mut Vec<ValidationIssue>) {
    let names: BTreeSet<String> = decl
        .rules
        .iter()
        .map(|r| r.name.clone())
        .chain(decl.tokens.iter().map(|t| t.name.clone()))
        .collect();
    let grammar_name = decl.name.clone();
    let mut rules = Vec::with_capacity(decl.rules.len());
    for rule in std::mem::take(&mut decl.rules) {
        match tiers(&rule, version, &names, &grammar_name, issues) {
            Some(expanded) => rules.extend(expanded),
            None => rules.push(rule),
        }
    }
    decl.rules = rules;
}

fn self_reference(element: &Element, name: &str) -> bool {
    matches!(&element.atom, Atom::Ref { namespace: None, name: n, .. } if n == name)
}

fn plain(element: &Element) -> bool {
    element.quantifier.is_none()
        && element.postfix_quantifier.is_none()
        && element.separator.is_none()
        && element.typeof_constraint.is_none()
}

fn optional_quantifier(element: &Element) -> bool {
    matches!(
        element.quantifier,
        Some(
            Quantifier::Question { .. }
                | Quantifier::Star { .. }
                | Quantifier::Bounded { min: 0, .. }
        )
    )
}

/// 左再帰ルールでも保持できる rule 注釈。ここに無いものは第一段では拒否する。
fn portable(annotation: &Annotation) -> bool {
    matches!(
        annotation,
        Annotation::Doc { .. } | Annotation::Whitespace { .. } | Annotation::Interleave { .. }
    )
}

fn annotation_name(annotation: &Annotation) -> &str {
    match annotation {
        Annotation::Root { .. } => "@root",
        Annotation::Mapping { .. } => "@mapping",
        Annotation::Eval { .. } => "@eval",
        Annotation::Whitespace { .. } => "@whitespace",
        Annotation::Interleave { .. } => "@interleave",
        Annotation::Backref { .. } => "@backref",
        Annotation::ScopeTree { .. } => "@scopeTree",
        Annotation::Declares { .. } => "@declares",
        Annotation::Catalog { .. } => "@catalog",
        Annotation::LeftAssoc { .. } => "@leftAssoc",
        Annotation::RightAssoc { .. } => "@rightAssoc",
        Annotation::LongestChoice { .. } => "@longestChoice",
        Annotation::PredictiveChoice { .. } => "@predictiveChoice",
        Annotation::Precedence { .. } => "@precedence",
        Annotation::Doc { .. } => "@doc",
        Annotation::Recovery { .. } => "@recovery",
        Annotation::Skip { .. } => "@skip",
        Annotation::CommonField { .. } => "@commonField",
        Annotation::Enum { .. } => "@enum",
        Annotation::Simple { name, .. } => name,
    }
}

fn reject_alternative_annotations(
    rule: &RuleDecl,
    grammar_name: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    for alternative in rule.body.alternatives() {
        for annotation in &alternative.annotations {
            issues.push(issue(
                "E-ALT-ASSOC-UNSUPPORTED",
                format!(
                    "{}: 候補先頭の {} は @ubnf: 2 の左再帰ルールでだけ使えます",
                    rule.name,
                    annotation_name(annotation)
                ),
                "grammar に '@ubnf: 2' を宣言し、候補の先頭要素をそのルール自身にしてください。",
                Some(&rule.name),
                grammar_name,
            ));
        }
    }
}

/// 直接左再帰ルールを優先順位塔へ展開する。対象外なら `None`（元の rule を残す）。
fn tiers(
    rule: &RuleDecl,
    version: u64,
    names: &BTreeSet<String>,
    grammar_name: &str,
    issues: &mut Vec<ValidationIssue>,
) -> Option<Vec<RuleDecl>> {
    let alternatives = rule.body.alternatives();
    let operators: Vec<usize> = alternatives
        .iter()
        .enumerate()
        .filter(|(_, a)| {
            a.elements
                .first()
                .is_some_and(|e| self_reference(e, &rule.name))
        })
        .map(|(i, _)| i)
        .collect();
    if version < LATEST_LANGUAGE_VERSION || operators.is_empty() {
        reject_alternative_annotations(rule, grammar_name, issues);
        return None;
    }
    let mut ok = true;
    let fail = |code: &str, message: String, hint: &str, issues: &mut Vec<ValidationIssue>| {
        issues.push(issue(code, message, hint, Some(&rule.name), grammar_name));
    };
    for annotation in &rule.annotations {
        if portable(annotation)
            || matches!(
                annotation,
                Annotation::Root { .. } | Annotation::Mapping { .. }
            )
        {
            continue;
        }
        ok = false;
        fail(
            "E-LEFTREC-ANNOTATION",
            format!(
                "{}: 左再帰ルールで {} は未対応です",
                rule.name,
                annotation_name(annotation)
            ),
            "その注釈が要る場合は塔を手書きしてください（@ubnf: 1 の書き方）。",
            issues,
        );
    }
    // 演算子候補は `Self <op> Self` の 3 要素に限る。塔の正規形
    // `Base @left { Op @op Right @right }` へ 1 対 1 で写せる形だけを受け入れる。
    let mut assoc = Vec::new();
    for &i in &operators {
        let alternative = &alternatives[i];
        let mut right = None;
        for annotation in &alternative.annotations {
            match annotation {
                Annotation::LeftAssoc { .. } => right = Some(false),
                Annotation::RightAssoc { .. } => right = Some(true),
                // 段ごとに AST クラスを変える場合の per-alternative @mapping。
                Annotation::Mapping { .. } => {}
                other => {
                    ok = false;
                    fail(
                        "E-LEFTREC-ALT-ANNOTATION",
                        format!(
                            "{}: 候補先頭に置けるのは @leftAssoc / @rightAssoc / @mapping だけです: {}",
                            rule.name,
                            annotation_name(other)
                        ),
                        "他の注釈は rule に付けてください。",
                        issues,
                    );
                }
            }
        }
        let shape = alternative.elements.len() == 3
            && plain(&alternative.elements[0])
            && plain(&alternative.elements[2])
            && self_reference(&alternative.elements[2], &rule.name)
            && !self_reference(&alternative.elements[1], &rule.name);
        if !shape {
            ok = false;
            fail(
                "E-LEFTREC-ALT-SHAPE",
                format!(
                    "{}: 左再帰の候補は `{} <演算子> {}` の 3 要素に限ります",
                    rule.name, rule.name, rule.name
                ),
                "前置・後置・三項の候補は第一段では未対応です。塔を手書きしてください。",
                issues,
            );
        } else if right == Some(true) && matches!(alternative.elements[1].atom, Atom::Ref { .. }) {
            // v1 の右結合正規形は二つの解析で食い違っている:
            // `lower/annotations.rs::operator_elements` は `Base @left { Op @op Self @right }`
            // を、`analysis/operators.rs::canonical` は `Base Op Self | Base` を右結合と見る。
            // 演算子が**規則参照**のときだけ operator metadata が作られてこの食い違いが
            // 表に出る（v1 でも同じ文法は E-ANALYSIS-... で落ちる）。第一段では
            // 生成し得ない形なので、黙って別の AST を作らずここで拒否する。
            ok = false;
            fail(
                "E-LEFTREC-RIGHTASSOC-OP-REF",
                format!(
                    "{}: 右結合の演算子に規則参照は使えません（v1 の右結合正規形の制約）",
                    rule.name
                ),
                "演算子をリテラルか括弧つきの選択（例 ('^'|'**')）で書いてください。",
                issues,
            );
        } else if optional_quantifier(&alternative.elements[1]) {
            ok = false;
            fail(
                "E-LEFTREC-NULLABLE-OP",
                format!("{}: 演算子が空一致し得ます", rule.name),
                "演算子要素から ? / * / {0,n} を外してください。",
                issues,
            );
        }
        assoc.push(right.unwrap_or(false));
    }
    let primaries: Vec<&Sequence> = alternatives
        .iter()
        .enumerate()
        .filter(|(i, _)| !operators.contains(i))
        .map(|(_, a)| a)
        .collect();
    for primary in &primaries {
        for annotation in &primary.annotations {
            ok = false;
            fail(
                "E-LEFTREC-ALT-ANNOTATION",
                format!(
                    "{}: 演算子候補でない候補に {} は置けません",
                    rule.name,
                    annotation_name(annotation)
                ),
                "候補の先頭要素をそのルール自身にしてください。",
                issues,
            );
        }
        if primary
            .elements
            .last()
            .is_some_and(|e| self_reference(e, &rule.name))
        {
            issues.push(issue(
                "W-LEFTREC-TRAILING-SELF-REF",
                format!(
                    "{}: 演算子候補でない候補の末尾の {} は式全体（最低優先）を指します",
                    rule.name, rule.name
                ),
                "ANTLR 風の前置演算子の優先順位が要る場合は塔を手書きしてください。",
                Some(&rule.name),
                grammar_name,
            ));
        }
    }
    if primaries.is_empty() {
        ok = false;
        fail(
            "E-LEFTREC-NO-BASE",
            format!("{}: 左再帰ルールに基底候補がありません", rule.name),
            "自身で始まらない候補を少なくとも一つ書いてください。",
            issues,
        );
    }
    let depth_names: Vec<String> = (0..=operators.len())
        .map(|d| tier_name(&rule.name, d, operators.len()))
        .collect();
    for name in &depth_names[1..] {
        if names.contains(name) {
            ok = false;
            fail(
                "E-LEFTREC-NAME-COLLISION",
                format!("{}: 脱糖が使う名前 {name} が既に存在します", rule.name),
                "既存の規則の名前を変えてください。",
                issues,
            );
        }
    }
    if !ok {
        return None;
    }
    let count = operators.len();
    let mut expanded = Vec::with_capacity(count + 1);
    for depth in 0..count {
        let index = operators[count - 1 - depth];
        let alternative = &alternatives[index];
        let next = &depth_names[depth + 1];
        let right_assoc = assoc[count - 1 - depth];
        let left = redirect(&alternative.elements[0], next);
        let right = redirect(
            &alternative.elements[2],
            if right_assoc {
                &depth_names[depth]
            } else {
                next
            },
        );
        let inner = Span {
            start: alternative.elements[1].span.start,
            end: alternative.elements[2].span.end,
        };
        let repeat = Element {
            typeof_constraint: None,
            atom: Atom::Repeat {
                body: RuleBody::Choice {
                    alternatives: vec![Sequence {
                        annotations: Vec::new(),
                        elements: vec![alternative.elements[1].clone(), right],
                        span: inner,
                    }],
                    span: inner,
                },
                span: inner,
            },
            quantifier: None,
            postfix_quantifier: None,
            separator: None,
            capture: None,
            span: inner,
        };
        expanded.push(RuleDecl {
            annotations: tier_annotations(rule, alternative, depth, right_assoc),
            name: depth_names[depth].clone(),
            body: RuleBody::Choice {
                alternatives: vec![Sequence {
                    annotations: Vec::new(),
                    elements: vec![left, repeat],
                    span: alternative.span,
                }],
                span: alternative.span,
            },
            span: alternative.span,
        });
    }
    let span = Span {
        start: primaries.first().unwrap().span.start,
        end: primaries.last().unwrap().span.end,
    };
    expanded.push(RuleDecl {
        annotations: rule
            .annotations
            .iter()
            .filter(|a| portable(a))
            .map(|a| {
                respan(
                    a,
                    Span {
                        start: span.start,
                        end: span.start,
                    },
                )
            })
            .collect(),
        name: depth_names[count].clone(),
        body: RuleBody::Choice {
            alternatives: primaries
                .into_iter()
                .map(|a| Sequence {
                    annotations: Vec::new(),
                    elements: a.elements.clone(),
                    span: a.span,
                })
                .collect(),
            span,
        },
        span,
    });
    Some(expanded)
}

/// depth 0 は元の名前（最低優先＝この規則の入口）。最深は基底候補の段。
fn tier_name(rule: &str, depth: usize, count: usize) -> String {
    if depth == 0 {
        rule.to_owned()
    } else if depth == count {
        format!("{rule}__lrBase")
    } else {
        format!("{rule}__lr{depth}")
    }
}

fn redirect(element: &Element, name: &str) -> Element {
    Element {
        typeof_constraint: None,
        atom: Atom::Ref {
            namespace: None,
            name: name.to_owned(),
            span: element.atom.span(),
        },
        quantifier: None,
        postfix_quantifier: None,
        separator: None,
        capture: element.capture.clone(),
        span: element.span,
    }
}

/// 合成した規則の注釈は、その規則の span（＝元の候補）の**直前 0 幅**に置く。
/// lower は「最後の注釈の直後」を symbol の開始位置にするので、元の位置のまま
/// 運ぶと symbol span が候補の外へ出てしまう。
fn respan(annotation: &Annotation, span: Span) -> Annotation {
    macro_rules! respan {
        ($($variant:ident { $($field:ident),* }),+ $(,)?) => {
            match annotation {
                $(Annotation::$variant { $($field,)* .. } => Annotation::$variant {
                    $($field: $field.clone(),)* span
                },)+
            }
        };
    }
    respan!(
        Root {},
        Mapping { class_name, params },
        Eval { entries },
        Whitespace { style },
        Interleave { profile },
        Backref { name },
        ScopeTree { mode },
        Declares {
            symbol,
            description
        },
        Catalog { context },
        LeftAssoc {},
        RightAssoc {},
        LongestChoice {},
        PredictiveChoice {},
        Precedence { level },
        Doc { text },
        Recovery { mode, sync },
        Skip {},
        CommonField { fields },
        Enum {},
        Simple { name },
    )
}

fn tier_annotations(
    rule: &RuleDecl,
    alternative: &Sequence,
    depth: usize,
    right_assoc: bool,
) -> Vec<Annotation> {
    let span = Span {
        start: alternative.span.start,
        end: alternative.span.start,
    };
    let mut annotations = Vec::new();
    if depth == 0 {
        annotations.extend(
            rule.annotations
                .iter()
                .filter(|a| matches!(a, Annotation::Root { .. }))
                .map(|a| respan(a, span)),
        );
    }
    annotations.extend(
        rule.annotations
            .iter()
            .filter(|a| portable(a))
            .map(|a| respan(a, span)),
    );
    // 候補先頭の @mapping はその段だけ rule の @mapping を上書きする。
    let mapping = alternative
        .annotations
        .iter()
        .find(|a| matches!(a, Annotation::Mapping { .. }))
        .or_else(|| {
            rule.annotations
                .iter()
                .find(|a| matches!(a, Annotation::Mapping { .. }))
        });
    annotations.extend(mapping.map(|a| respan(a, span)));
    // capture 名が left/op/right のときだけ既存の結合性正規形に一致する。
    // その場合だけ operator metadata（@leftAssoc / @precedence）を付ける。
    let captures: Vec<Option<&str>> = alternative
        .elements
        .iter()
        .map(|e| e.capture.as_deref())
        .collect();
    if captures == [Some("left"), Some("op"), Some("right")] {
        annotations.push(if right_assoc {
            Annotation::RightAssoc { span }
        } else {
            Annotation::LeftAssoc { span }
        });
        annotations.push(Annotation::Precedence {
            level: ((depth + 1) as u64) * 10,
            span,
        });
    }
    annotations
}
