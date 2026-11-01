use crate::{generated::ast::Ast, model::*};
use std::collections::BTreeMap;

struct State {
    stack: Vec<Ty>,
    required: Vec<Ty>,
    locals: BTreeMap<String, Ty>,
    infer: bool,
}
#[derive(Default)]
struct Checker {
    bindings: Vec<Option<Ty>>,
    numeric: Vec<Ty>,
    equality: Vec<Ty>,
    steps: usize,
}
impl Checker {
    fn fresh(&mut self) -> Ty {
        let id = self.bindings.len();
        self.bindings.push(None);
        Ty::Var(id)
    }
    fn resolve(&self, t: &Ty) -> Ty {
        match t {
            Ty::Var(i) => self.bindings[*i]
                .as_ref()
                .map(|t| self.resolve(t))
                .unwrap_or(t.clone()),
            Ty::Quote(e) => Ty::Quote(Effect {
                input: e.input.iter().map(|t| self.resolve(t)).collect(),
                output: e.output.iter().map(|t| self.resolve(t)).collect(),
            }),
            _ => t.clone(),
        }
    }
    fn occurs(&self, id: usize, t: &Ty) -> bool {
        match self.resolve(t) {
            Ty::Var(i) => i == id,
            Ty::Quote(e) => e.input.iter().chain(&e.output).any(|t| self.occurs(id, t)),
            _ => false,
        }
    }
    fn unify(&mut self, a: &Ty, b: &Ty) -> Result<()> {
        let (a, b) = (self.resolve(a), self.resolve(b));
        if a == b {
            return Ok(());
        }
        match (&a, &b) {
            (Ty::Var(id), t) | (t, Ty::Var(id)) => {
                if self.occurs(*id, t) {
                    return fail("無限の引用型は作れない");
                }
                self.bindings[*id] = Some(t.clone());
                Ok(())
            }
            (Ty::Quote(a), Ty::Quote(b)) => {
                // 共通のスタック接頭辞は保存する。空引用は任意の接頭辞上の恒等関数。
                let mut a = a.clone();
                let mut b = b.clone();
                let count = a.input.len().max(b.input.len());
                for e in [&mut a, &mut b] {
                    let prefix: Vec<_> = (e.input.len()..count).map(|_| self.fresh()).collect();
                    e.input.splice(0..0, prefix.clone());
                    e.output.splice(0..0, prefix);
                }
                self.same(&a.input, &b.input)?;
                self.same(&a.output, &b.output)
            }
            _ => fail(format!("型不一致: {a} と {b}")),
        }
    }
    fn same(&mut self, a: &[Ty], b: &[Ty]) -> Result<()> {
        if a.len() != b.len() {
            return fail(format!(
                "宣言効果の深さが不一致: {} と {}",
                a.len(),
                b.len()
            ));
        }
        for (a, b) in a.iter().zip(b) {
            self.unify(a, b)?;
        }
        Ok(())
    }
    fn pop(&mut self, s: &mut State) -> Result<Ty> {
        if let Some(t) = s.stack.pop() {
            Ok(t)
        } else if s.infer {
            let t = self.fresh();
            s.required.insert(0, t.clone());
            Ok(t)
        } else {
            fail("スタック深さが不足")
        }
    }
    fn expect(&mut self, s: &mut State, t: &Ty) -> Result<()> {
        let actual = self.pop(s)?;
        self.unify(&actual, t)
    }
    fn apply(&mut self, s: &mut State, e: &Effect) -> Result<()> {
        for t in e.input.iter().rev() {
            self.expect(s, t)?;
        }
        s.stack.extend(e.output.clone());
        Ok(())
    }
    fn quote(&self, t: Ty) -> Result<Effect> {
        if let Ty::Quote(e) = self.resolve(&t) {
            Ok(e)
        } else {
            fail("call / choose には効果が分かる引用が必要")
        }
    }
    fn body(&mut self, body: &[Ast], s: &mut State, w: &Word, depth: usize) -> Result<()> {
        if depth > 64 {
            return fail("型検査の引用深さ上限");
        }
        use Ast::*;
        for term in body {
            self.steps += 1;
            if self.steps > 100_000 {
                return fail("型検査のステップ上限");
            }
            let result: Result<()> = (|| {
                match term {
                    g_StaxAST_2e_NumLit(n) => s.stack.push(match number(&n.g_text)? {
                        Number::Int(_) => Ty::Int,
                        Number::Float(_) => Ty::Float,
                    }),
                    g_StaxAST_2e_StrLit(n) => {
                        string(&n.g_text)?;
                        s.stack.push(Ty::Str);
                    }
                    g_StaxAST_2e_BoolLit(_) => s.stack.push(Ty::Bool),
                    g_StaxAST_2e_Quotation(n) => {
                        let mut q = State {
                            stack: vec![],
                            required: vec![],
                            locals: s.locals.clone(),
                            infer: true,
                        };
                        self.body(&n.g_body, &mut q, w, depth + 1)?;
                        s.stack.push(Ty::Quote(Effect {
                            input: q.required,
                            output: q.stack,
                        }));
                    }
                    g_StaxAST_2e_LetBind(n) => {
                        if n.g_name.len() > 256 {
                            return fail("局所名の長さ上限");
                        }
                        if crate::modules::reserved(&n.g_name) {
                            return fail("予約名は局所束縛に使えない");
                        }
                        if matches!(w.env.get(&n.g_name), Some(Symbol::Macro(_))) {
                            return fail("macro 名は局所束縛で隠せない");
                        }
                        let t = self.pop(s)?;
                        if let Some(old) = s.locals.get(&n.g_name) {
                            self.unify(old, &t)?;
                        } else {
                            s.locals.insert(n.g_name.clone(), t);
                        }
                    }
                    g_StaxAST_2e_FieldGet(n) => {
                        let t = self.pop(s)?;
                        let def = self.structure(&t, &n.g_name, &w.env)?;
                        let t = def
                            .fields
                            .iter()
                            .find(|(name, _)| name == &n.g_name)
                            .ok_or_else(|| Error(format!("未知のフィールド: {}", n.g_name)))?
                            .1
                            .clone();
                        s.stack.push(t);
                    }
                    g_StaxAST_2e_FieldSet(n) => {
                        let value = self.pop(s)?;
                        let owner = self.pop(s)?;
                        let def = self.structure(&owner, &n.g_name, &w.env)?;
                        let t = &def
                            .fields
                            .iter()
                            .find(|(name, _)| name == &n.g_name)
                            .ok_or_else(|| Error(format!("未知のフィールド: {}", n.g_name)))?
                            .1;
                        self.unify(t, &value)?;
                        s.stack.push(Ty::Struct(def));
                    }
                    g_StaxAST_2e_Prim(n) => self.primitive(&n.g_name, s, w)?,
                    g_StaxAST_2e_WordRef(n) => {
                        if let Some(t) = s.locals.get(&n.g_name) {
                            s.stack.push(t.clone());
                        } else {
                            match w.env.get(&n.g_name) {
                                Some(Symbol::Word(word) | Symbol::Macro(word)) => {
                                    self.apply(s, &word.effect)?
                                }
                                Some(Symbol::Constructor(def)) => {
                                    self.apply(
                                        s,
                                        &Effect {
                                            input: def
                                                .fields
                                                .iter()
                                                .map(|(_, t)| t.clone())
                                                .collect(),
                                            output: vec![Ty::Struct(def.clone())],
                                        },
                                    )?;
                                }
                                Some(Symbol::Variant(t, _)) => s.stack.push(t.clone()),
                                _ => {
                                    return fail(format!(
                                        "未定義の語または未束縛の局所名: {}",
                                        n.g_name
                                    ))
                                }
                            }
                        }
                    }
                    _ => return fail("実行できない項 AST"),
                }
                Ok(())
            })();
            result.map_err(|e| {
                Error(format!(
                    "{}:{}（{}）: {}",
                    w.path.display(),
                    term.span().unwrap_or([0, 0])[0],
                    w.name,
                    e
                ))
            })?;
        }
        Ok(())
    }
    fn structure(&mut self, t: &Ty, field: &str, env: &Env) -> Result<std::rc::Rc<Structure>> {
        if let Ty::Struct(s) = self.resolve(t) {
            return Ok(s);
        }
        if matches!(self.resolve(t), Ty::Var(_)) {
            let mut candidates = BTreeMap::new();
            for symbol in env.values() {
                if let Symbol::Constructor(s) = symbol {
                    if s.fields.iter().any(|(name, _)| name == field) {
                        candidates.insert(s.id.clone(), s.clone());
                    }
                }
            }
            if candidates.len() == 1 {
                let s = candidates.into_values().next().unwrap();
                self.unify(t, &Ty::Struct(s.clone()))?;
                return Ok(s);
            }
        }
        fail(format!(
            "フィールド {field} の所有 struct を型から決定できない"
        ))
    }
    fn primitive(&mut self, name: &str, s: &mut State, w: &Word) -> Result<()> {
        match name {
            "dup" => {
                let a = self.pop(s)?;
                s.stack.extend([a.clone(), a]);
            }
            "drop" => {
                self.pop(s)?;
            }
            "swap" | "over" | "nip" => {
                let b = self.pop(s)?;
                let a = self.pop(s)?;
                match name {
                    "swap" => s.stack.extend([b, a]),
                    "over" => s.stack.extend([a.clone(), b, a]),
                    _ => s.stack.push(b),
                }
            }
            "rot" => {
                let c = self.pop(s)?;
                let b = self.pop(s)?;
                let a = self.pop(s)?;
                s.stack.extend([b, c, a]);
            }
            "+" | "-" | "*" | "/" => {
                let b = self.pop(s)?;
                let a = self.pop(s)?;
                self.unify(&a, &b)?;
                self.numeric.push(a.clone());
                s.stack.push(a);
            }
            "neg" => {
                let a = self.pop(s)?;
                self.numeric.push(a.clone());
                s.stack.push(a);
            }
            "mod" => self.apply(
                s,
                &Effect {
                    input: vec![Ty::Int, Ty::Int],
                    output: vec![Ty::Int],
                },
            )?,
            "=" | "<>" => {
                let b = self.pop(s)?;
                let a = self.pop(s)?;
                self.unify(&a, &b)?;
                self.equality.push(a);
                s.stack.push(Ty::Bool);
            }
            "<" | "<=" | ">" | ">=" => self.apply(
                s,
                &Effect {
                    input: vec![Ty::Int, Ty::Int],
                    output: vec![Ty::Bool],
                },
            )?,
            "and" | "or" => self.apply(
                s,
                &Effect {
                    input: vec![Ty::Bool, Ty::Bool],
                    output: vec![Ty::Bool],
                },
            )?,
            "not" => self.apply(
                s,
                &Effect {
                    input: vec![Ty::Bool],
                    output: vec![Ty::Bool],
                },
            )?,
            "concat" => self.apply(
                s,
                &Effect {
                    input: vec![Ty::Str, Ty::Str],
                    output: vec![Ty::Str],
                },
            )?,
            "str_len" => self.apply(
                s,
                &Effect {
                    input: vec![Ty::Str],
                    output: vec![Ty::Int],
                },
            )?,
            "int_to_str" => self.apply(
                s,
                &Effect {
                    input: vec![Ty::Int],
                    output: vec![Ty::Str],
                },
            )?,
            "print" => self.expect(s, &Ty::Str)?,
            "newline" => {}
            "recurse" => self.apply(s, &w.effect)?,
            "call" => {
                let t = self.pop(s)?;
                let e = self.quote(t)?;
                self.apply(s, &e)?;
            }
            "choose" => {
                let b = self.pop(s)?;
                let a = self.pop(s)?;
                self.quote(a.clone())?;
                self.quote(b.clone())?;
                self.unify(&a, &b)?;
                self.expect(s, &Ty::Bool)?;
                s.stack.push(a);
            }
            _ => return fail(format!("未知の primitive: {name}")),
        }
        Ok(())
    }
}
pub(crate) fn check(word: &Word) -> Result<()> {
    let mut c = Checker::default();
    let mut s = State {
        stack: word.effect.input.clone(),
        required: vec![],
        locals: BTreeMap::new(),
        infer: false,
    };
    c.body(&word.body, &mut s, word, 0)?;
    c.same(&s.stack, &word.effect.output).map_err(|e| {
        Error(format!(
            "{}: {} の宣言効果と本体が不一致: {e}",
            word.path.display(),
            word.name
        ))
    })?;
    for t in &c.numeric {
        if !matches!(c.resolve(t), Ty::Int | Ty::Float | Ty::Var(_)) {
            return fail("算術には同じ数値型が必要");
        }
    }
    for t in &c.equality {
        if !matches!(
            c.resolve(t),
            Ty::Int | Ty::Float | Ty::Bool | Ty::Str | Ty::Enum(_) | Ty::Var(_)
        ) {
            return fail("この型は等価比較できない");
        }
    }
    Ok(())
}
