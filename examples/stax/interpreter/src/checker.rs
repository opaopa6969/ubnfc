use crate::{generated::ast::Ast, model::*};
use std::{
    cell::Cell,
    collections::{BTreeMap, VecDeque},
};

struct State {
    stack: Vec<Ty>,
    required: VecDeque<Ty>,
    locals: BTreeMap<String, Ty>,
    infer: bool,
}
#[derive(Default)]
struct Checker {
    bindings: Vec<Option<Ty>>,
    ranks: Vec<u8>,
    work: Cell<usize>,
    binding_types: BTreeMap<String, Ty>,
    numeric: Vec<Ty>,
    equality: Vec<Ty>,
    steps: usize,
}
impl Checker {
    fn charge(&self, n: usize) -> Result<()> {
        let used = self
            .work
            .get()
            .checked_add(n)
            .ok_or_else(|| Error("型検査の操作上限".into()))?;
        if used > 1_000_000 {
            return fail("型検査の操作上限");
        }
        self.work.set(used);
        Ok(())
    }
    fn fresh(&mut self) -> Ty {
        let id = self.bindings.len();
        self.bindings.push(None);
        self.ranks.push(0);
        Ty::Var(id)
    }
    fn resolve(&self, t: &Ty) -> Result<Ty> {
        self.resolve_at(t, 0)
    }
    fn resolve_at<'a>(&'a self, mut t: &'a Ty, depth: usize) -> Result<Ty> {
        if depth > 64 {
            return fail("引用型の深さ上限");
        }
        self.charge(1)?;
        // 型変数の親は host の再帰を使わずにたどる。
        while let Ty::Var(id) = t {
            self.charge(1)?;
            match &self.bindings[*id] {
                Some(parent) => t = parent,
                None => return Ok(t.clone()),
            }
        }
        Ok(match t {
            Ty::Quote(e) => Ty::Quote(Effect {
                input: e
                    .input
                    .iter()
                    .map(|t| self.resolve_at(t, depth + 1))
                    .collect::<Result<_>>()?,
                output: e
                    .output
                    .iter()
                    .map(|t| self.resolve_at(t, depth + 1))
                    .collect::<Result<_>>()?,
            }),
            _ => t.clone(),
        })
    }
    fn occurs(&self, id: usize, t: &Ty) -> Result<bool> {
        let t = self.resolve(t)?;
        let mut pending = vec![&t];
        while let Some(t) = pending.pop() {
            match t {
                Ty::Var(other) if *other == id => return Ok(true),
                Ty::Quote(e) => pending.extend(e.input.iter().chain(&e.output)),
                _ => {}
            }
        }
        Ok(false)
    }
    fn unify(&mut self, a: &Ty, b: &Ty) -> Result<()> {
        let (a, b) = (self.resolve(a)?, self.resolve(b)?);
        if a == b {
            return Ok(());
        }
        match (&a, &b) {
            (Ty::Var(a), Ty::Var(b)) => {
                // 小さい木を大きい木へ結合し、変数連鎖の二乗時間を避ける。
                let (a, b) = (*a, *b);
                if self.ranks[a] < self.ranks[b] {
                    self.bindings[a] = Some(Ty::Var(b));
                } else {
                    self.bindings[b] = Some(Ty::Var(a));
                    if self.ranks[a] == self.ranks[b] {
                        self.ranks[a] += 1;
                    }
                }
                Ok(())
            }
            (Ty::Var(id), t) | (t, Ty::Var(id)) => {
                if self.occurs(*id, t)? {
                    return fail("無限の引用型は作れない");
                }
                self.bindings[*id] = Some(t.clone());
                Ok(())
            }
            (Ty::Quote(a), Ty::Quote(b)) => {
                self.same(&a.input, &b.input)?;
                self.same(&a.output, &b.output)
            }
            _ => fail(format!("型不一致: {a} と {b}")),
        }
    }
    fn assign(&mut self, actual: &Ty, expected: &Ty) -> Result<()> {
        let (a, b) = (self.resolve(actual)?, self.resolve(expected)?);
        if let (Ty::Quote(a), Ty::Quote(b)) = (&a, &b) {
            // より少ない入力で動く値を、接頭辞を保存する限定的な型へ渡す向きだけ許す。
            if a.input.len() > b.input.len() {
                return fail("引用の入力効果が不一致");
            }
            let mut a = a.clone();
            let prefix: Vec<_> = (a.input.len()..b.input.len())
                .map(|_| self.fresh())
                .collect();
            a.input.splice(0..0, prefix.clone());
            a.output.splice(0..0, prefix);
            self.same(&a.input, &b.input)?;
            self.same(&a.output, &b.output)
        } else {
            self.unify(&a, &b)
        }
    }
    fn combine(&mut self, a: &Effect, b: &Effect) -> Result<Effect> {
        // 両方の枝が要求する共通の入力を残す。空引用側の型だけを返すと入力を失う。
        let mut a = a.clone();
        let mut b = b.clone();
        let count = a.input.len().max(b.input.len());
        for e in [&mut a, &mut b] {
            let prefix: Vec<_> = (e.input.len()..count).map(|_| self.fresh()).collect();
            e.input.splice(0..0, prefix.clone());
            e.output.splice(0..0, prefix);
        }
        self.same(&a.input, &b.input)?;
        self.same(&a.output, &b.output)?;
        Ok(Effect {
            input: a
                .input
                .iter()
                .map(|t| self.resolve(t))
                .collect::<Result<_>>()?,
            output: a
                .output
                .iter()
                .map(|t| self.resolve(t))
                .collect::<Result<_>>()?,
        })
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
            s.required.push_front(t.clone());
            Ok(t)
        } else {
            fail("スタック深さが不足")
        }
    }
    fn expect(&mut self, s: &mut State, t: &Ty) -> Result<()> {
        let actual = self.pop(s)?;
        self.assign(&actual, t)
    }
    fn apply(&mut self, s: &mut State, e: &Effect) -> Result<()> {
        for t in e.input.iter().rev() {
            self.expect(s, t)?;
        }
        s.stack.extend(e.output.clone());
        Ok(())
    }
    fn quote(&self, t: Ty) -> Result<Effect> {
        if let Ty::Quote(e) = self.resolve(&t)? {
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
                        self.charge(s.locals.len())?;
                        let mut q = State {
                            stack: vec![],
                            required: VecDeque::new(),
                            locals: s.locals.clone(),
                            infer: true,
                        };
                        self.body(&n.g_body, &mut q, w, depth + 1)?;
                        let inferred = Ty::Quote(Effect {
                            input: q.required.into_iter().collect(),
                            output: q.stack,
                        });
                        self.resolve(&inferred)?;
                        s.stack.push(inferred);
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
                        // 引用ごとの可視性と、同一呼出し枠での単型性を分ける。
                        let declared = if let Some(old) = self.binding_types.get(&n.g_name).cloned()
                        {
                            self.assign(&t, &old)?;
                            old
                        } else {
                            self.binding_types.insert(n.g_name.clone(), t.clone());
                            t
                        };
                        s.locals.insert(n.g_name.clone(), declared);
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
                        self.assign(&value, t)?;
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
        if let Ty::Struct(s) = self.resolve(t)? {
            return Ok(s);
        }
        if matches!(self.resolve(t)?, Ty::Var(_)) {
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
                let a = self.quote(a)?;
                let b = self.quote(b)?;
                let common = self.combine(&a, &b)?;
                self.expect(s, &Ty::Bool)?;
                s.stack.push(Ty::Quote(common));
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
        required: VecDeque::new(),
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
        if !matches!(c.resolve(t)?, Ty::Int | Ty::Float | Ty::Var(_)) {
            return fail("算術には同じ数値型が必要");
        }
    }
    for t in &c.equality {
        if !matches!(
            c.resolve(t)?,
            Ty::Int | Ty::Float | Ty::Bool | Ty::Str | Ty::Enum(_) | Ty::Var(_)
        ) {
            return fail("この型は等価比較できない");
        }
    }
    Ok(())
}
