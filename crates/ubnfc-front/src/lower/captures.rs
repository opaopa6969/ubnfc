use super::*;
use r::{ResolvedAtom as A, ResolvedBody};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Shape {
    pub value: ValueType,
    pub count: Cardinality,
}
impl Shape {
    fn text() -> Self {
        Self {
            value: ValueType::Text {},
            count: Cardinality::Scalar,
        }
    }
    fn wrap(mut self, count: Cardinality) -> Self {
        self.count = join_count(self.count, count);
        self
    }
}
fn join_count(a: Cardinality, b: Cardinality) -> Cardinality {
    if a == Cardinality::List || b == Cardinality::List {
        Cardinality::List
    } else if a == Cardinality::Optional || b == Cardinality::Optional {
        Cardinality::Optional
    } else {
        Cardinality::Scalar
    }
}
pub(super) fn join(a: ValueType, b: ValueType) -> ValueType {
    if a == b {
        return a;
    }
    let mut members = match a {
        ValueType::Mixed { members } => members,
        other => vec![other],
    };
    for t in match b {
        ValueType::Mixed { members } => members,
        other => vec![other],
    } {
        if !members.contains(&t) {
            members.push(t);
        }
    }
    ValueType::Mixed { members }
}
fn merge(a: Shape, b: Shape, sequence: bool) -> Shape {
    Shape {
        value: join(a.value, b.value),
        count: if sequence {
            Cardinality::List
        } else {
            join_count(a.count, b.count)
        },
    }
}
pub(super) fn conversion(value: &ValueType) -> Conversion {
    match value {
        ValueType::Text {} => Conversion::Text,
        ValueType::Number { .. } => Conversion::Number,
        ValueType::Node { .. } => Conversion::Node,
        _ => Conversion::Mixed,
    }
}
pub(super) fn fields(cx: &Context<'_>, rule: &r::ResolvedRule) -> BTreeMap<String, Shape> {
    body_captures(cx, &rule.body)
}
fn body_captures(cx: &Context<'_>, body: &ResolvedBody) -> BTreeMap<String, Shape> {
    let alternatives: Vec<_> = body
        .alternatives
        .iter()
        .map(|seq| {
            let mut out = BTreeMap::new();
            for e in seq {
                if let Some(name) = &e.capture {
                    add(&mut out, name.clone(), captured(cx, &e.atom), true);
                }
                for (name, shape) in atom_captures(cx, &e.atom) {
                    add(&mut out, name, shape, true);
                }
            }
            out
        })
        .collect();
    let mut out = BTreeMap::new();
    for seq in &alternatives {
        for (name, shape) in seq {
            add(&mut out, name.clone(), shape.clone(), false);
        }
    }
    for (name, shape) in &mut out {
        if alternatives.iter().any(|seq| !seq.contains_key(name)) {
            shape.count = join_count(shape.count, Cardinality::Optional);
        }
    }
    out
}
fn add(out: &mut BTreeMap<String, Shape>, name: String, shape: Shape, sequence: bool) {
    if let Some(old) = out.remove(&name) {
        out.insert(name, merge(old, shape, sequence));
    } else {
        out.insert(name, shape);
    }
}
fn atom_captures(cx: &Context<'_>, atom: &A) -> BTreeMap<String, Shape> {
    let (mut out, count) = match atom {
        A::Group { body, .. } => (body_captures(cx, body), Cardinality::Scalar),
        A::Optional { body, .. } => (body_captures(cx, body), Cardinality::Optional),
        A::Repeat { body, .. } => (body_captures(cx, body), Cardinality::List),
        A::OneOrMore { body, .. } | A::Bounded { body, .. } => {
            (atom_captures(cx, body), Cardinality::List)
        }
        A::Separated {
            element, separator, ..
        } => {
            let mut out = atom_captures(cx, element);
            for (name, shape) in atom_captures(cx, separator) {
                add(&mut out, name, shape, true);
            }
            (out, Cardinality::List)
        }
        _ => return BTreeMap::new(),
    };
    for shape in out.values_mut() {
        shape.count = join_count(shape.count, count);
    }
    out
}
pub(super) fn captured(cx: &Context<'_>, atom: &A) -> Shape {
    match atom {
        A::Optional { body, .. } => capture_body(cx, body).wrap(Cardinality::Optional),
        A::Repeat { body, .. } => capture_body(cx, body).wrap(Cardinality::List),
        A::OneOrMore { body, .. }
        | A::Bounded { body, .. }
        | A::Separated { element: body, .. } => captured(cx, body).wrap(Cardinality::List),
        A::Ref { symbol, .. } => {
            if let Some(token) = cx
                .grammar
                .grammars
                .iter()
                .flat_map(|g| &g.tokens)
                .find(|t| t.id == *symbol)
            {
                if matches!(
                    token.value,
                    r::ResolvedTokenValue::Builtin {
                        kind: r::BuiltinKind::Number | r::BuiltinKind::Digit
                    }
                ) {
                    return Shape {
                        value: ValueType::Number {
                            java: ValueTypeNumberJava,
                            rust: ValueTypeNumberRust,
                        },
                        count: Cardinality::Scalar,
                    };
                }
            }
            shape_atom(cx, atom, &mut BTreeSet::new())
        }
        _ => shape_atom(cx, atom, &mut BTreeSet::new()),
    }
}
fn capture_body(cx: &Context<'_>, body: &ResolvedBody) -> Shape {
    if let [seq] = body.alternatives.as_slice() {
        if let [e] = seq.as_slice() {
            return captured(cx, &e.atom);
        }
    }
    shape_body(cx, body, &mut BTreeSet::new())
}
fn shape_atom(cx: &Context<'_>, atom: &A, visiting: &mut BTreeSet<r::SymbolId>) -> Shape {
    match atom {
        A::Ref { symbol, .. } => {
            if let Some(rule) = cx.rule(*symbol) {
                if skipped(rule) {
                    return Shape::text();
                }
                if let Some(type_id) = cx.types.get(symbol) {
                    return Shape {
                        value: ValueType::Node {
                            type_id: type_id.clone(),
                        },
                        count: Cardinality::Scalar,
                    };
                }
                if !visiting.insert(*symbol) {
                    return Shape::text();
                }
                let shape = shape_body(cx, &rule.body, visiting);
                visiting.remove(symbol);
                shape
            } else {
                Shape::text()
            }
        }
        A::Group { body, .. } => shape_body(cx, body, visiting),
        A::Optional { body, .. } => {
            semantic_wrap(shape_body(cx, body, visiting), Cardinality::Optional)
        }
        A::Repeat { body, .. } => semantic_wrap(shape_body(cx, body, visiting), Cardinality::List),
        A::OneOrMore { body, .. }
        | A::Bounded { body, .. }
        | A::Separated { element: body, .. } => {
            semantic_wrap(shape_atom(cx, body, visiting), Cardinality::List)
        }
        _ => Shape::text(),
    }
}
fn semantic_wrap(shape: Shape, count: Cardinality) -> Shape {
    if shape.value == (ValueType::Text {}) {
        shape
    } else {
        shape.wrap(count)
    }
}
fn shape_body(
    cx: &Context<'_>,
    body: &ResolvedBody,
    visiting: &mut BTreeSet<r::SymbolId>,
) -> Shape {
    body.alternatives
        .iter()
        .map(|seq| {
            seq.iter()
                .map(|e| shape_atom(cx, &e.atom, visiting))
                .filter(|s| s.value != ValueType::Text {})
                .reduce(|a, b| merge(a, b, true))
                .unwrap_or_else(Shape::text)
        })
        .reduce(|a, b| merge(a, b, false))
        .unwrap_or_else(Shape::text)
}
/// 共有結合 schema の operand 探索。mapped rule で停止し、その内部 field は辿らない。
pub(super) fn mapped_types(cx: &Context<'_>, atom: &A) -> Vec<String> {
    fn atom_visit(
        cx: &Context<'_>,
        a: &A,
        seen: &mut BTreeSet<r::SymbolId>,
        out: &mut Vec<String>,
    ) {
        match a {
            A::Ref { symbol, .. } => {
                if let Some(t) = cx.types.get(symbol) {
                    if !out.contains(t) {
                        out.push(t.clone());
                    }
                } else if seen.insert(*symbol) {
                    if let Some(r) = cx.rule(*symbol) {
                        body_visit(cx, &r.body, seen, out);
                    }
                }
            }
            A::Group { body, .. } | A::Optional { body, .. } | A::Repeat { body, .. } => {
                body_visit(cx, body, seen, out)
            }
            A::OneOrMore { body, .. }
            | A::Bounded { body, .. }
            | A::Separated { element: body, .. } => atom_visit(cx, body, seen, out),
            _ => {}
        }
    }
    fn body_visit(
        cx: &Context<'_>,
        b: &ResolvedBody,
        s: &mut BTreeSet<r::SymbolId>,
        o: &mut Vec<String>,
    ) {
        for e in b.alternatives.iter().flatten() {
            atom_visit(cx, &e.atom, s, o);
        }
    }
    let mut out = vec![];
    atom_visit(cx, atom, &mut BTreeSet::new(), &mut out);
    out
}
