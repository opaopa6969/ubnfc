//! capture occurrence と mapping が取り出す意味値の個数を区別する。
//! 未 mapping helper は複数/optional の値を返せる。mapped rule では探索を止める。
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn join(a: Cardinality, b: Cardinality) -> Cardinality {
    if a == Cardinality::List || b == Cardinality::List {
        Cardinality::List
    } else if a == Cardinality::Optional || b == Cardinality::Optional {
        Cardinality::Optional
    } else {
        Cardinality::Scalar
    }
}

pub(super) struct Values<'a> {
    expressions: BTreeMap<&'a ExprId, &'a Expr>,
    rules: BTreeMap<&'a str, &'a Rule>,
}

impl<'a> Values<'a> {
    pub(super) fn new(s: &'a Semantics) -> Self {
        Self {
            expressions: s.expressions.iter().map(|e| (&e.id, e)).collect(),
            rules: s.rules.iter().map(|r| (r.symbol_id.as_str(), r)).collect(),
        }
    }

    pub(super) fn captured(&self, capture: &Capture) -> Cardinality {
        join(
            capture.cardinality,
            self.count(&capture.expr_id, &mut BTreeSet::new())
                .unwrap_or(Cardinality::Scalar),
        )
    }

    // None は text のみ。区切り文字は mapped 値に加算せず、text のみの
    // helper は一致範囲全体が一つの値になる（lower::captures::shape_body と同じ）。
    fn count(&self, id: &ExprId, visiting: &mut BTreeSet<ExprId>) -> Option<Cardinality> {
        let e = self.expressions.get(id)?;
        if !visiting.insert(id.clone()) {
            return None;
        }
        let count = match &e.node {
            ExprNode::RuleRef { symbol_id } => self.rules.get(symbol_id.as_str()).and_then(|r| {
                if r.skip {
                    None
                } else if r.mapping_type_id.is_some() {
                    Some(Cardinality::Scalar)
                } else {
                    self.count(&r.body, visiting)
                }
            }),
            ExprNode::Seq { items } => items
                .iter()
                .filter_map(|id| self.count(id, visiting))
                .reduce(|_, _| Cardinality::List),
            ExprNode::Choice { alternatives, .. } => {
                let counts: Vec<_> = alternatives
                    .iter()
                    .map(|id| self.count(id, visiting))
                    .collect();
                counts
                    .iter()
                    .any(Option::is_some)
                    .then(|| counts.into_iter().flatten().fold(Cardinality::Scalar, join))
            }
            ExprNode::Group { child } => self.count(child, visiting),
            ExprNode::Optional { child } => self
                .count(child, visiting)
                .map(|c| join(c, Cardinality::Optional)),
            ExprNode::Repeat { child, .. } | ExprNode::Separated { element: child, .. } => {
                self.count(child, visiting).map(|_| Cardinality::List)
            }
            _ => None,
        };
        visiting.remove(id);
        count
    }
}
