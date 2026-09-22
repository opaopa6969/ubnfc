use super::*;
use r::ResolvedAtom as A;

pub(super) fn normalize(grammar: &mut ResolvedGrammar) -> Vec<LowerError> {
    let mut warnings = vec![];
    for grammar_decl in &mut grammar.grammars {
        for rule in &mut grammar_decl.rules {
            let unit = &grammar.units[rule.unit.0];
            if let Some(ast) = unit.ast.rules.iter().find(|r| r.span == rule.span) {
                body(&mut rule.body, &ast.body, &mut warnings);
            }
        }
    }
    warnings
}
fn body(resolved: &mut r::ResolvedBody, ast: &a::RuleBody, warnings: &mut Vec<LowerError>) {
    for (rs, seq) in resolved.alternatives.iter_mut().zip(ast.alternatives()) {
        for (r, e) in rs.iter_mut().zip(&seq.elements) {
            let suffixes = usize::from(e.postfix_quantifier.is_some())
                + usize::from(e.quantifier.is_some())
                + usize::from(e.separator.is_some());
            if suffixes > 1 {
                warnings.push(LowerError::Warning {
                    code: "W-LOWER-SUFFIX-PRIORITY".into(),
                    message: "D-015: postfix → bounded → separated の最初の suffix のみ採用".into(),
                    source: Some(e.span),
                });
                if e.separator.is_some()
                    && (e.quantifier.is_some() || e.postfix_quantifier.is_some())
                {
                    if let A::Separated { element, .. } = &r.atom {
                        r.atom = (**element).clone();
                    }
                }
                if e.postfix_quantifier.is_some() {
                    if let A::Bounded { body, .. } = &r.atom {
                        r.atom = (**body).clone();
                    }
                }
            }
            // 採用された suffix の内側にある元 atom に対応させてから再帰する。
            let has_quant = e.quantifier.is_some() || e.postfix_quantifier.is_some();
            let base = if has_quant {
                unwrap_quantifier(&mut r.atom)
            } else if e.separator.is_some() {
                if let A::Separated { element, .. } = &mut r.atom {
                    element.as_mut()
                } else {
                    unreachable!()
                }
            } else {
                &mut r.atom
            };
            atom(base, &e.atom, warnings);
            if !has_quant {
                if let (Some(s), A::Separated { separator, .. }) = (&e.separator, &mut r.atom) {
                    atom(separator, s, warnings);
                }
            }
        }
    }
}
fn unwrap_quantifier(atom: &mut A) -> &mut A {
    match atom {
        A::OneOrMore { body, .. } | A::Bounded { body, .. } => body,
        A::Optional { body, .. } | A::Repeat { body, .. } => &mut body.alternatives[0][0].atom,
        _ => unreachable!("resolve の suffix 形"),
    }
}
fn atom(resolved: &mut A, ast: &a::Atom, warnings: &mut Vec<LowerError>) {
    match (resolved, ast) {
        (A::Group { body: r, .. }, a::Atom::Group { body: a, .. })
        | (A::Optional { body: r, .. }, a::Atom::Optional { body: a, .. })
        | (A::Repeat { body: r, .. }, a::Atom::Repeat { body: a, .. }) => body(r, a, warnings),
        _ => {}
    }
}
