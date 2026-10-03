//! Declarative lexical token programs. Matching semantics mirror UBNF format 2.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
    LITERAL,
    ANY,
    EOF,
    BOF,
    BOL,
    EOL,
    RANGE,
    EXCEPT,
    SEQUENCE,
    CHOICE,
    REPEAT,
    LOOK,
    NOT,
    CAPTURE,
    BACKREF,
    REF,
    SCOPE,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexicalExpression {
    pub op: Op,
    pub text: String,
    pub min: i32,
    pub max: i32,
    pub children: Vec<LexicalExpression>,
}
impl LexicalExpression {
    pub fn validate(&self) -> Result<(), String> {
        self.validate_inner(&mut std::collections::BTreeSet::new(), 0, &mut 0)
    }
    fn validate_inner(
        &self,
        bound: &mut std::collections::BTreeSet<String>,
        depth: usize,
        nodes: &mut usize,
    ) -> Result<(), String> {
        *nodes += 1;
        if depth > 128 || *nodes > 4096 {
            return Err("lexical program exceeds expansion limit".into());
        }
        let arity = match self.op {
            Op::CAPTURE | Op::LOOK | Op::NOT | Op::SCOPE | Op::REPEAT => 1,
            Op::SEQUENCE | Op::CHOICE => {
                if self.children.is_empty() {
                    return Err("empty lexical sequence/choice".into());
                }
                self.children.len()
            }
            _ => 0,
        };
        if self.children.len() != arity {
            return Err("invalid lexical arity".into());
        }
        match self.op {
            Op::REF => return Err("unresolved lexical reference".into()),
            Op::RANGE => {
                if self.min < 0
                    || self.min > self.max
                    || self.max > 0x10ffff
                    || (0xd800..=0xdfff).contains(&self.min)
                    || (0xd800..=0xdfff).contains(&self.max)
                {
                    return Err("invalid lexical scalar range".into());
                }
            }
            Op::BACKREF => {
                if !bound.contains(&self.text) {
                    return Err("unbound lexical backreference".into());
                }
            }
            Op::CHOICE => {
                let mut definite = None;
                for child in &self.children {
                    let mut branch = bound.clone();
                    child.validate_inner(&mut branch, depth + 1, nodes)?;
                    if let Some(ref mut intersection) = definite {
                        std::collections::BTreeSet::retain(intersection, |name| {
                            branch.contains(name)
                        });
                    } else {
                        definite = Some(branch);
                    }
                }
                bound.extend(definite.unwrap());
            }
            Op::LOOK | Op::NOT | Op::SCOPE => {
                let mut local = if self.op == Op::SCOPE {
                    Default::default()
                } else {
                    bound.clone()
                };
                self.children[0].validate_inner(&mut local, depth + 1, nodes)?;
            }
            Op::REPEAT => {
                if self.min < 0 || self.max < -1 || self.max >= 0 && self.min > self.max {
                    return Err("invalid lexical repeat bounds".into());
                }
                let mut iteration = bound.clone();
                self.children[0].validate_inner(&mut iteration, depth + 1, nodes)?;
                if self.max < 0 && self.children[0].nullable() {
                    return Err("nullable unbounded lexical repeat".into());
                }
                if self.min > 0 {
                    bound.extend(iteration);
                }
            }
            Op::CAPTURE => {
                self.children[0].validate_inner(bound, depth + 1, nodes)?;
                bound.insert(self.text.clone());
            }
            Op::SEQUENCE => {
                for child in &self.children {
                    child.validate_inner(bound, depth + 1, nodes)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub fn leaf(op: Op, text: String) -> Self {
        Self {
            op,
            text,
            min: 0,
            max: 0,
            children: vec![],
        }
    }
    pub fn node(op: Op, children: Vec<Self>) -> Self {
        Self {
            children,
            ..Self::leaf(op, String::new())
        }
    }
    pub fn repeat(child: Self, min: i32, max: i32) -> Self {
        Self {
            min,
            max,
            ..Self::node(Op::REPEAT, vec![child])
        }
    }
    pub fn nullable(&self) -> bool {
        match self.op {
            Op::LITERAL => self.text.is_empty(),
            Op::ANY | Op::RANGE | Op::EXCEPT => false,
            Op::SEQUENCE => self.children.iter().all(Self::nullable),
            Op::CHOICE => self.children.iter().any(Self::nullable),
            Op::REPEAT => self.min == 0 || self.children[0].nullable(),
            Op::CAPTURE | Op::SCOPE => self.children[0].nullable(),
            _ => true,
        }
    }
}

pub fn qualify(expression: &mut LexicalExpression, prefix: &str) {
    if expression.op == Op::REF && !prefix.is_empty() {
        expression.text = format!("{prefix}.{}", expression.text);
    }
    for child in &mut expression.children {
        qualify(child, prefix);
    }
}
pub fn compile(
    definitions: &BTreeMap<String, LexicalExpression>,
    name: &str,
) -> Result<LexicalExpression, String> {
    let program = expand(
        &LexicalExpression::leaf(Op::REF, name.into()),
        definitions,
        &mut BTreeSet::new(),
        0,
        &mut 0,
    )?;
    program.validate()?;
    Ok(program)
}
fn expand(
    e: &LexicalExpression,
    definitions: &BTreeMap<String, LexicalExpression>,
    visiting: &mut BTreeSet<String>,
    depth: usize,
    nodes: &mut usize,
) -> Result<LexicalExpression, String> {
    *nodes += 1;
    if depth > 128 || *nodes > 4096 {
        return Err("lexical expansion exceeds limit".into());
    }
    if e.op == Op::REF {
        let target = definitions
            .get(&e.text)
            .ok_or_else(|| format!("undefined declarative token {}", e.text))?;
        if !visiting.insert(e.text.clone()) {
            return Err(format!("cyclic lexical reference {}", e.text));
        }
        let program = LexicalExpression::node(
            Op::SCOPE,
            vec![expand(target, definitions, visiting, depth + 1, nodes)?],
        );
        visiting.remove(&e.text);
        return Ok(program);
    }
    Ok(LexicalExpression {
        children: e
            .children
            .iter()
            .map(|c| expand(c, definitions, visiting, depth + 1, nodes))
            .collect::<Result<_, _>>()?,
        ..e.clone()
    })
}
