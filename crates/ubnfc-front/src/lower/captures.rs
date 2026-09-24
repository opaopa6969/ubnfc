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
/// 量指定子が一度の成功解析で共存させる出現の個数。`{1}` は 1 個・`{0,1}` は 0/1 個で、
/// List になるのは上限が 2 以上のときだけ（`docs/nested-capture-migration.md`）。
pub(super) fn quantifier_count(atom: &A) -> Cardinality {
    match atom {
        A::Optional { .. } => Cardinality::Optional,
        A::Bounded { min, max, .. } => Cardinality::quantified(*min, *max),
        A::Repeat { .. } | A::OneOrMore { .. } | A::Separated { .. } => Cardinality::List,
        _ => Cardinality::Scalar,
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
    body_captures(cx, &rule.body, true)
}
/// capture の出現回数は、参照先 helper が返す意味値の個数とは別。
/// AST field/recipe は fields の意味的 cardinality を保持する。
pub(super) fn occurrences(cx: &Context<'_>, rule: &r::ResolvedRule) -> BTreeMap<String, Shape> {
    body_captures(cx, &rule.body, false)
}
fn body_captures(cx: &Context<'_>, body: &ResolvedBody, semantic: bool) -> BTreeMap<String, Shape> {
    let alternatives: Vec<_> = body
        .alternatives
        .iter()
        .map(|seq| {
            let mut out = BTreeMap::new();
            for e in seq {
                if let Some(name) = &e.capture {
                    let shape = if semantic {
                        captured(cx, &e.atom)
                    } else {
                        Shape::text().wrap(quantifier_count(&e.atom))
                    };
                    add(&mut out, name.clone(), shape, true);
                }
                for (name, shape) in atom_captures(cx, &e.atom, semantic) {
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
fn atom_captures(cx: &Context<'_>, atom: &A, semantic: bool) -> BTreeMap<String, Shape> {
    let (mut out, count) = match atom {
        A::Group { body, .. } => (body_captures(cx, body, semantic), Cardinality::Scalar),
        A::Optional { body, .. } => (body_captures(cx, body, semantic), Cardinality::Optional),
        A::Repeat { body, .. } => (body_captures(cx, body, semantic), Cardinality::List),
        A::OneOrMore { body, .. } => (atom_captures(cx, body, semantic), Cardinality::List),
        A::Bounded { body, .. } => (atom_captures(cx, body, semantic), quantifier_count(atom)),
        A::Separated {
            element, separator, ..
        } => {
            let mut out = atom_captures(cx, element, semantic);
            for (name, shape) in atom_captures(cx, separator, semantic) {
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
    captured_at(cx, atom, true)
}
/// `site` は capture が直接貼られた原子。そこの量指定子は出現回数そのものなので必ず数える。
/// 内側の量指定子は**意味値**の個数だけを増やす。text だけの領域は何回繰り返しても
/// 一致範囲全体で一つの値なので増えない（`shape_atom` と同じ規則）。
fn captured_at(cx: &Context<'_>, atom: &A, site: bool) -> Shape {
    let wrap = |shape: Shape, count| {
        if site {
            shape.wrap(count)
        } else {
            semantic_wrap(shape, count)
        }
    };
    match atom {
        A::Optional { body, .. } => wrap(capture_body(cx, body), Cardinality::Optional),
        A::Repeat { body, .. } => wrap(capture_body(cx, body), Cardinality::List),
        A::Bounded { body, .. } => wrap(captured_at(cx, body, false), quantifier_count(atom)),
        A::OneOrMore { body, .. } | A::Separated { element: body, .. } => {
            wrap(captured_at(cx, body, false), Cardinality::List)
        }
        // #75 / D-074: token の直接 capture（`NUMBER @x`）も原文の文字列。数値 token を
        // 言語別の数値型（Java int / Rust 字句）へ変える特例は、同じ文法の canonical AST を
        // backend ごとに変えていたので廃止した。型付けは `@mapping` の利用側が行う。
        _ => shape_atom(cx, atom, &mut BTreeSet::new()),
    }
}
fn capture_body(cx: &Context<'_>, body: &ResolvedBody) -> Shape {
    if let [seq] = body.alternatives.as_slice() {
        if let [e] = seq.as_slice() {
            return captured_at(cx, &e.atom, false);
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
        A::Bounded { body, .. } => {
            semantic_wrap(shape_atom(cx, body, visiting), quantifier_count(atom))
        }
        A::OneOrMore { body, .. } | A::Separated { element: body, .. } => {
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
