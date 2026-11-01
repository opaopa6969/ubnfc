use crate::generated::ast::Ast;
use std::{collections::BTreeMap, fmt, path::PathBuf, rc::Rc};

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone)]
pub struct Error(pub String);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
pub(crate) fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(Error(message.into()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Ty {
    Int,
    Float,
    Bool,
    Str,
    Struct(Rc<Structure>),
    Enum(Rc<str>),
    Quote(Effect),
    Var(usize),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Effect {
    pub input: Vec<Ty>,
    pub output: Vec<Ty>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Structure {
    pub id: String,
    pub fields: Vec<(String, Ty)>,
}
#[derive(Clone)]
pub(crate) enum Symbol {
    Word(Rc<Word>),
    Macro(Rc<Word>),
    Constructor(Rc<Structure>),
    Type(Ty),
    Variant(Ty, Rc<str>),
    Field,
}
pub(crate) type Env = BTreeMap<String, Symbol>;
#[derive(Clone)]
pub(crate) struct Word {
    pub name: String,
    pub path: PathBuf,
    pub effect: Effect,
    pub body: Rc<Vec<Ast>>,
    pub env: Rc<Env>,
}

pub(crate) fn type_ref(ast: &Ast, env: &Env) -> Result<Ty> {
    use Ast::*;
    match ast {
        g_StaxAST_2e_BuiltinType(n) => match n.g_name.as_str() {
            "Int" => Ok(Ty::Int),
            "Float" => Ok(Ty::Float),
            "Bool" => Ok(Ty::Bool),
            "Str" => Ok(Ty::Str),
            _ => unreachable!(),
        },
        g_StaxAST_2e_NamedType(n) => match env.get(&n.g_name) {
            Some(Symbol::Type(t)) => Ok(t.clone()),
            Some(Symbol::Constructor(s)) => Ok(Ty::Struct(s.clone())),
            _ => fail(format!("未知の型: {}", n.g_name)),
        },
        g_StaxAST_2e_QuotType(n) => {
            let ty = Ty::Quote(effect(&n.g_inputs, &n.g_outputs, env)?);
            ty.check_depth(0)?;
            Ok(ty)
        }
        _ => fail("型の AST が必要"),
    }
}
pub(crate) fn effect(input: &[Ast], output: &[Ast], env: &Env) -> Result<Effect> {
    Ok(Effect {
        input: input
            .iter()
            .map(|a| type_ref(a, env))
            .collect::<Result<_>>()?,
        output: output
            .iter()
            .map(|a| type_ref(a, env))
            .collect::<Result<_>>()?,
    })
}
pub(crate) fn declared_effect(ast: &Ast, env: &Env) -> Result<Effect> {
    if let Ast::g_StaxAST_2e_Effect(n) = ast {
        effect(&n.g_inputs, &n.g_outputs, env)
    } else {
        fail("Effect AST が必要")
    }
}

pub(crate) fn number(text: &str) -> Result<Number> {
    if text.contains(['.', 'e', 'E']) {
        let v = text
            .parse::<f64>()
            .map_err(|_| Error("Float リテラルが不正".into()))?;
        if v.is_finite() {
            Ok(Number::Float(v))
        } else {
            fail("Float リテラルが範囲外")
        }
    } else {
        text.parse::<i64>()
            .map(Number::Int)
            .map_err(|_| Error("Int リテラルが i64 の範囲外".into()))
    }
}
pub(crate) enum Number {
    Int(i64),
    Float(f64),
}

pub(crate) fn string(text: &str) -> Result<String> {
    let mut chars = text.chars();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        out.push(match chars.next() {
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            Some('0') => '\0',
            Some('"') => '"',
            Some('\\') => '\\',
            _ => return fail(r#"文字列で未対応の escape"#),
        });
    }
    Ok(out)
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => f.write_str("Int"),
            Ty::Float => f.write_str("Float"),
            Ty::Bool => f.write_str("Bool"),
            Ty::Str => f.write_str("Str"),
            Ty::Struct(s) => f.write_str(&s.id),
            Ty::Enum(id) => f.write_str(id),
            Ty::Var(id) => write!(f, "T{id}"),
            Ty::Quote(e) => {
                f.write_str("[")?;
                for t in &e.input {
                    write!(f, " {t}")?;
                }
                f.write_str(" --")?;
                for t in &e.output {
                    write!(f, " {t}")?;
                }
                f.write_str(" ]")
            }
        }
    }
}

impl Ty {
    fn check_depth(&self, depth: usize) -> Result<()> {
        if depth > 64 {
            return fail("引用型の深さ上限");
        }
        if let Ty::Quote(e) = self {
            for t in e.input.iter().chain(&e.output) {
                t.check_depth(depth + 1)?;
            }
        }
        Ok(())
    }
}
