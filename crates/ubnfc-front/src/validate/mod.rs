//! GrammarValidator の診断契約。コード・文言は仕様表より Java 実装を優先する。
//!
//! 未定義参照は resolve が E-RULE-UNDEFINED として拒否する。負の precedence は
//! syntax の u64 では表現不能なので、E-PRECEDENCE-NEGATIVE には到達しない。
//! @eval の和型拒否は EvaluatorGenerator の例外を E-EVAL-SUM-TYPE に構造化する。

mod annotations;
mod general;
mod mapping;
mod operators;

use crate::resolve::{Grammar, ResolvedAtom, ResolvedBody, ResolvedGrammar, ResolvedRule};
use crate::syntax::ast::Annotation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    pub hint: String,
    pub rule: Option<String>,
    pub grammar: String,
}

impl ValidationIssue {
    pub fn severity(&self) -> &'static str {
        if self.code.starts_with("W-") {
            "WARNING"
        } else {
            "ERROR"
        }
    }

    pub fn category(&self) -> &'static str {
        for (prefix, category) in [
            ("W-CHOICE-", "CHOICE"),
            ("W-ANNOTATION-", "ANNOTATION"),
            ("E-MAPPING-", "MAPPING"),
            ("E-ASSOC-", "ASSOCIATIVITY"),
            ("E-RIGHTASSOC-", "ASSOCIATIVITY"),
            ("E-WHITESPACE-", "WHITESPACE"),
            ("E-MEMO-", "MEMOIZATION"),
            ("E-PRECEDENCE-", "PRECEDENCE"),
            ("E-ANNOTATION-", "ANNOTATION"),
            ("E-RULE-", "RULE"),
        ] {
            if self.code.starts_with(prefix) {
                return category;
            }
        }
        "GENERAL"
    }

    pub fn format(&self) -> String {
        format!(
            "{} [code: {}] [hint: {}]",
            self.message, self.code, self.hint
        )
    }
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format())
    }
}

pub fn validate(resolved: &ResolvedGrammar) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    for grammar in &resolved.grammars {
        let mut context = Context {
            grammar,
            issues: &mut issues,
        };
        let v2 = general::declares_v2(grammar);
        general::validate(&mut context);
        mapping::validate_shapes(&mut context);
        let scope = grammar.rules.iter().any(|r| {
            r.annotations
                .iter()
                .any(|a| matches!(a, Annotation::ScopeTree { .. }))
        });
        for rule in &grammar.rules {
            mapping::validate_mapping(rule, &mut context);
            operators::validate_rule(rule, &mut context);
            annotations::validate(rule, scope, v2, &mut context);
        }
        operators::validate_topology(&mut context);
        operators::left_recursion(&mut context);
    }
    issues.sort_by(|a, b| {
        (&a.grammar, &a.rule, &a.code, &a.message).cmp(&(&b.grammar, &b.rule, &b.code, &b.message))
    });
    issues
}

struct Context<'a> {
    grammar: &'a Grammar,
    issues: &'a mut Vec<ValidationIssue>,
}

impl Context<'_> {
    fn add(
        &mut self,
        rule: Option<&str>,
        code: &str,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) {
        self.issues.push(ValidationIssue {
            code: code.into(),
            message: message.into(),
            hint: hint.into(),
            rule: rule.map(str::to_owned),
            grammar: self.grammar.name.clone(),
        });
    }
    fn rule(
        &mut self,
        rule: &ResolvedRule,
        code: &str,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) {
        self.add(Some(&rule.name), code, message, hint);
    }
}

fn captures(rule: &ResolvedRule) -> Vec<&str> {
    let mut names = Vec::new();
    for site in &rule.captures {
        if !names.contains(&site.name.as_str()) {
            names.push(site.name.as_str());
        }
    }
    names
}

fn first_mapping(rule: &ResolvedRule) -> Option<(&str, &[String])> {
    rule.annotations.iter().find_map(mapping_annotation)
}
fn last_mapping(rule: &ResolvedRule) -> Option<(&str, &[String])> {
    rule.annotations.iter().rev().find_map(mapping_annotation)
}
fn mapping_annotation(annotation: &Annotation) -> Option<(&str, &[String])> {
    if let Annotation::Mapping {
        class_name, params, ..
    } = annotation
    {
        Some((class_name, params))
    } else {
        None
    }
}

fn single_sequence(body: &ResolvedBody) -> Option<&[crate::resolve::ResolvedElement]> {
    if body.alternatives.len() == 1 {
        Some(&body.alternatives[0])
    } else {
        None
    }
}

/// Java の各検査が辿る従来の Group/Optional/Repeat だけを選ぶ。
/// OneOrMore/Bounded/Separated の参照解決と capture 列挙は resolve が全て辿る。
fn nested_body(atom: &ResolvedAtom) -> Option<&ResolvedBody> {
    match atom {
        ResolvedAtom::Group { body, .. }
        | ResolvedAtom::Optional { body, .. }
        | ResolvedAtom::Repeat { body, .. } => Some(body),
        _ => None,
    }
}

fn trim(value: &str) -> &str {
    value.trim_matches(|c: char| c <= '\u{20}')
}
