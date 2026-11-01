use crate::{generated::ast::Ast, model::*};
use std::{collections::BTreeMap, rc::Rc};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub steps: u64,
    pub depth: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            steps: 100_000,
            depth: 1024,
        }
    }
}
#[derive(Clone)]
enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Rc<str>),
    Struct(Rc<Structure>, Rc<Vec<Value>>),
    Enum(Rc<str>, Rc<str>),
    Quote(Rc<Quotation>),
}
#[derive(Clone)]
struct Quotation {
    body: Rc<Vec<Ast>>,
    frame: usize,
    owner: Rc<Word>,
}
struct Task {
    body: Rc<Vec<Ast>>,
    pc: usize,
    frame: usize,
    owner: Rc<Word>,
}
pub(crate) struct Machine {
    tasks: Vec<Task>,
    stack: Vec<Value>,
    frames: Vec<BTreeMap<String, Value>>,
    output: String,
    limits: Limits,
    steps: u64,
    bytes: usize,
    nodes: usize,
    slots: usize,
    compile: bool,
}
impl Machine {
    fn new(limits: Limits, compile: bool) -> Self {
        Self {
            tasks: vec![],
            stack: vec![],
            frames: vec![],
            output: String::new(),
            limits,
            steps: 0,
            bytes: 0,
            nodes: 0,
            slots: 0,
            compile,
        }
    }
    fn pop(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or_else(|| Error("実行時のスタック深さが不足".into()))
    }
    fn integer(&mut self) -> Result<i64> {
        match self.pop()? {
            Value::Int(n) => Ok(n),
            _ => fail("実行時に Int が必要"),
        }
    }
    fn boolean(&mut self) -> Result<bool> {
        match self.pop()? {
            Value::Bool(n) => Ok(n),
            _ => fail("実行時に Bool が必要"),
        }
    }
    fn text(&mut self) -> Result<Rc<str>> {
        match self.pop()? {
            Value::Str(n) => Ok(n),
            _ => fail("実行時に Str が必要"),
        }
    }
    fn quote(&mut self) -> Result<Rc<Quotation>> {
        match self.pop()? {
            Value::Quote(n) => Ok(n),
            _ => fail("実行時に引用が必要"),
        }
    }
    fn string(&mut self, text: String) -> Result<Value> {
        self.bytes += text.len();
        if self.bytes > 8 * 1024 * 1024 {
            return fail("文字列の累積割当上限");
        }
        Ok(Value::Str(text.into()))
    }
    fn frame(&mut self) -> Result<usize> {
        if self.frames.len() >= 4096 {
            return fail("呼出し枠の累積上限");
        }
        let id = self.frames.len();
        self.frames.push(BTreeMap::new());
        Ok(id)
    }
    fn call(&mut self, word: Rc<Word>) -> Result<()> {
        if self.tasks.len() >= self.limits.depth {
            return fail("呼出し深さ上限");
        }
        let frame = self.frame()?;
        self.tasks.push(Task {
            body: word.body.clone(),
            pc: 0,
            frame,
            owner: word,
        });
        Ok(())
    }
    fn invoke(&mut self, q: Rc<Quotation>) -> Result<()> {
        if self.tasks.len() >= self.limits.depth {
            return fail("呼出し深さ上限");
        }
        self.tasks.push(Task {
            body: q.body.clone(),
            pc: 0,
            frame: q.frame,
            owner: q.owner.clone(),
        });
        Ok(())
    }
    fn charge_nodes(&mut self, body: &[Ast]) -> Result<()> {
        for t in body {
            self.nodes += 1;
            if self.nodes > 100_000 {
                return fail("引用 AST の累積割当上限");
            }
            if let Ast::g_StaxAST_2e_Quotation(q) = t {
                self.charge_nodes(&q.g_body)?;
            }
        }
        Ok(())
    }
    fn charge_slots(&mut self, n: usize) -> Result<()> {
        self.slots += n;
        if self.slots > 1_000_000 {
            return fail("構造体の累積割当上限");
        }
        Ok(())
    }
    fn drive(&mut self) -> Result<()> {
        use Ast::*;
        while let Some(task) = self.tasks.last_mut() {
            if task.pc == task.body.len() {
                self.tasks.pop();
                continue;
            }
            let (body, pc, frame, owner) =
                (task.body.clone(), task.pc, task.frame, task.owner.clone());
            task.pc += 1;
            let term = &body[pc];
            if self.steps >= self.limits.steps {
                return fail("実行ステップ上限");
            }
            self.steps += 1;
            let result: Result<()> = (|| {
                match term {
                    g_StaxAST_2e_NumLit(n) => self.stack.push(match number(&n.g_text)? {
                        Number::Int(n) => Value::Int(n),
                        Number::Float(n) => Value::Float(n),
                    }),
                    g_StaxAST_2e_StrLit(n) => {
                        let v = self.string(string(&n.g_text)?)?;
                        self.stack.push(v);
                    }
                    g_StaxAST_2e_BoolLit(n) => self.stack.push(Value::Bool(n.g_text == "true")),
                    g_StaxAST_2e_Quotation(n) => {
                        self.charge_nodes(&n.g_body)?;
                        self.stack.push(Value::Quote(Rc::new(Quotation {
                            body: Rc::new(n.g_body.clone()),
                            frame,
                            owner: owner.clone(),
                        })));
                    }
                    g_StaxAST_2e_LetBind(n) => {
                        let v = self.pop()?;
                        self.frames[frame].insert(n.g_name.clone(), v);
                    }
                    g_StaxAST_2e_FieldGet(n) => {
                        let Value::Struct(def, values) = self.pop()? else {
                            return fail("フィールド取得には struct が必要");
                        };
                        let i = def
                            .fields
                            .iter()
                            .position(|(f, _)| f == &n.g_name)
                            .ok_or_else(|| Error("未知のフィールド".into()))?;
                        self.stack.push(values[i].clone());
                    }
                    g_StaxAST_2e_FieldSet(n) => {
                        let v = self.pop()?;
                        let Value::Struct(def, values) = self.pop()? else {
                            return fail("フィールド更新には struct が必要");
                        };
                        let i = def
                            .fields
                            .iter()
                            .position(|(f, _)| f == &n.g_name)
                            .ok_or_else(|| Error("未知のフィールド".into()))?;
                        self.charge_slots(values.len())?;
                        let mut values = values.as_ref().clone();
                        values[i] = v;
                        self.stack.push(Value::Struct(def, Rc::new(values)));
                    }
                    g_StaxAST_2e_Prim(n) => self.primitive(&n.g_name, owner.clone())?,
                    g_StaxAST_2e_WordRef(n) => {
                        if let Some(v) = self.frames[frame].get(&n.g_name) {
                            self.stack.push(v.clone());
                        } else {
                            match owner.env.get(&n.g_name).cloned() {
                                Some(Symbol::Word(w)) => self.call(w)?,
                                Some(Symbol::Macro(_)) => {
                                    return fail("展開されていない macro は実行できない")
                                }
                                Some(Symbol::Variant(Ty::Enum(ty), tag)) => {
                                    self.stack.push(Value::Enum(ty, tag))
                                }
                                Some(Symbol::Constructor(def)) => {
                                    let n = def.fields.len();
                                    if self.stack.len() < n {
                                        return fail("struct 構築のスタック深さが不足");
                                    }
                                    self.charge_slots(n)?;
                                    let values = self.stack.split_off(self.stack.len() - n);
                                    self.stack.push(Value::Struct(def, Rc::new(values)));
                                }
                                _ => return fail(format!("実行時に未束縛の名前: {}", n.g_name)),
                            }
                        }
                    }
                    _ => return fail("実行できない AST"),
                }
                Ok(())
            })();
            result.map_err(|e| {
                Error(format!(
                    "{}:{}（{}）: {e}",
                    owner.path.display(),
                    term.span().unwrap_or([0, 0])[0],
                    owner.name
                ))
            })?;
            if self.stack.len() > 65_536 {
                return fail("値スタック上限");
            }
        }
        Ok(())
    }
    fn primitive(&mut self, name: &str, owner: Rc<Word>) -> Result<()> {
        match name {
            "dup" => {
                let a = self.pop()?;
                self.stack.extend([a.clone(), a]);
            }
            "drop" => {
                self.pop()?;
            }
            "swap" | "over" | "nip" => {
                let b = self.pop()?;
                let a = self.pop()?;
                match name {
                    "swap" => self.stack.extend([b, a]),
                    "over" => self.stack.extend([a.clone(), b, a]),
                    _ => self.stack.push(b),
                }
            }
            "rot" => {
                let c = self.pop()?;
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.extend([b, c, a]);
            }
            "+" | "-" | "*" | "/" | "mod" => {
                let b = self.pop()?;
                let a = self.pop()?;
                let v = match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        if matches!(name, "/" | "mod") && b == 0 {
                            return fail("0 除算");
                        }
                        let n = match name {
                            "+" => a.checked_add(b),
                            "-" => a.checked_sub(b),
                            "*" => a.checked_mul(b),
                            "/" => a.checked_div(b),
                            _ => a.checked_rem(b),
                        };
                        Value::Int(n.ok_or_else(|| Error("整数 overflow".into()))?)
                    }
                    (Value::Float(a), Value::Float(b)) if name != "mod" => {
                        if name == "/" && b == 0.0 {
                            return fail("0 除算");
                        }
                        let n = match name {
                            "+" => a + b,
                            "-" => a - b,
                            "*" => a * b,
                            _ => a / b,
                        };
                        if !n.is_finite() {
                            return fail("Float の結果が有限でない");
                        }
                        Value::Float(n)
                    }
                    _ => return fail("同じ数値型が必要"),
                };
                self.stack.push(v);
            }
            "neg" => {
                let v = match self.pop()? {
                    Value::Int(n) => Value::Int(
                        n.checked_neg()
                            .ok_or_else(|| Error("整数 overflow".into()))?,
                    ),
                    Value::Float(n) => Value::Float(-n),
                    _ => return fail("数値が必要"),
                };
                self.stack.push(v);
            }
            "=" | "<>" => {
                let b = self.pop()?;
                let a = self.pop()?;
                let same = match (a, b) {
                    (Value::Int(a), Value::Int(b)) => a == b,
                    (Value::Float(a), Value::Float(b)) => a == b,
                    (Value::Bool(a), Value::Bool(b)) => a == b,
                    (Value::Str(a), Value::Str(b)) => a == b,
                    (Value::Enum(a, x), Value::Enum(b, y)) => a == b && x == y,
                    _ => return fail("比較できない値"),
                };
                self.stack
                    .push(Value::Bool(if name == "=" { same } else { !same }));
            }
            "<" | "<=" | ">" | ">=" => {
                let b = self.integer()?;
                let a = self.integer()?;
                self.stack.push(Value::Bool(match name {
                    "<" => a < b,
                    "<=" => a <= b,
                    ">" => a > b,
                    _ => a >= b,
                }));
            }
            "and" | "or" => {
                let b = self.boolean()?;
                let a = self.boolean()?;
                self.stack
                    .push(Value::Bool(if name == "and" { a && b } else { a || b }));
            }
            "not" => {
                let a = self.boolean()?;
                self.stack.push(Value::Bool(!a));
            }
            "concat" => {
                let b = self.text()?;
                let a = self.text()?;
                if a.len() + b.len() > 1024 * 1024 {
                    return fail("文字列長上限");
                }
                let v = self.string(format!("{a}{b}"))?;
                self.stack.push(v);
            }
            "str_len" => {
                let s = self.text()?;
                self.stack.push(Value::Int(s.chars().count() as i64));
            }
            "int_to_str" => {
                let i = self.integer()?;
                let v = self.string(i.to_string())?;
                self.stack.push(v);
            }
            "print" | "newline" => {
                if self.compile {
                    return fail("macro 展開時の I/O は禁止");
                }
                let text: Rc<str> = if name == "print" {
                    self.text()?
                } else {
                    "\n".into()
                };
                if self.output.len() + text.len() > 1024 * 1024 {
                    return fail("標準出力長上限");
                }
                self.output.push_str(&text);
            }
            "call" => {
                let q = self.quote()?;
                self.invoke(q)?;
            }
            "choose" => {
                let b = self.quote()?;
                let a = self.quote()?;
                let cond = self.boolean()?;
                self.stack.push(Value::Quote(if cond { a } else { b }));
            }
            "recurse" => self.call(owner)?,
            _ => return fail("未知の primitive"),
        }
        Ok(())
    }
    fn reify(
        &self,
        body: &[Ast],
        frame: usize,
        budget: &mut usize,
        depth: usize,
    ) -> Result<Vec<Ast>> {
        if depth > 64 {
            return fail("macro 展開深さ上限");
        }
        let mut out = vec![];
        for term in body {
            *budget += 1;
            if *budget > 100_000 {
                return fail("macro 展開ノード上限");
            }
            let mut term = term.clone();
            match &mut term {
                Ast::g_StaxAST_2e_LetBind(n) if self.frames[frame].contains_key(&n.g_name) => {
                    return fail("macro の捕捉名を返す引用内で再束縛することは未対応")
                }
                Ast::g_StaxAST_2e_WordRef(n) => {
                    if let Some(v) = self.frames[frame].get(&n.g_name) {
                        if let Value::Quote(q) = v {
                            term = Ast::g_StaxAST_2e_Quotation(
                                crate::generated::ast::g_StaxAST_2e_Quotation {
                                    span: n.span,
                                    node_id: n.node_id,
                                    g_body: self.reify(&q.body, q.frame, budget, depth + 1)?,
                                },
                            );
                        } else {
                            return fail("macro が返す引用の捕捉値は引用に限定する");
                        }
                    }
                }
                Ast::g_StaxAST_2e_Quotation(n) => {
                    n.g_body = self.reify(&n.g_body, frame, budget, depth + 1)?
                }
                _ => {}
            }
            out.push(term);
        }
        Ok(out)
    }
}
pub(crate) fn execute(word: Rc<Word>, limits: Limits) -> Result<String> {
    if limits.steps == 0 || limits.steps > 1_000_000 || limits.depth == 0 || limits.depth > 4096 {
        return fail("資源上限の設定が範囲外");
    }
    let mut vm = Machine::new(limits, false);
    vm.call(word)?;
    vm.drive()?;
    if !vm.stack.is_empty() {
        return fail("main の終了時スタックが空でない");
    }
    Ok(vm.output)
}
pub(crate) fn expand(
    body: &[Ast],
    env: &Env,
    budget: &mut usize,
    depth: usize,
) -> Result<Vec<Ast>> {
    if depth > 64 {
        return fail("macro 展開深さ上限");
    }
    let mut out = vec![];
    for term in body {
        *budget += 1;
        if *budget > 100_000 {
            return fail("macro 展開ノード上限");
        }
        if let Ast::g_StaxAST_2e_WordRef(n) = term {
            if let Some(Symbol::Macro(word)) = env.get(&n.g_name) {
                let count = word.effect.input.len();
                if out.len() < count {
                    return fail("macro の直前に引用リテラルが必要");
                }
                let args = out.split_off(out.len() - count);
                let mut vm = Machine::new(
                    Limits {
                        steps: 10_000,
                        depth: 64,
                    },
                    true,
                );
                let frame = vm.frame()?;
                for arg in args {
                    let Ast::g_StaxAST_2e_Quotation(q) = arg else {
                        return fail("macro 引数は直前の引用リテラルに限定する");
                    };
                    vm.charge_nodes(&q.g_body)?;
                    vm.stack.push(Value::Quote(Rc::new(Quotation {
                        body: Rc::new(q.g_body),
                        frame,
                        owner: word.clone(),
                    })));
                }
                vm.call(word.clone())?;
                vm.drive()?;
                if vm.stack.len() != 1 {
                    return fail("macro は引用を1つ返す必要がある");
                }
                let q = vm.quote()?;
                let expanded = vm.reify(&q.body, q.frame, budget, 0)?;
                let expanded = expand(&expanded, env, budget, depth + 1)?;
                out.push(Ast::g_StaxAST_2e_Quotation(
                    crate::generated::ast::g_StaxAST_2e_Quotation {
                        span: n.span,
                        node_id: n.node_id,
                        g_body: expanded,
                    },
                ));
                continue;
            }
        }
        let mut term = term.clone();
        if let Ast::g_StaxAST_2e_Quotation(q) = &mut term {
            q.g_body = expand(&q.g_body, env, budget, depth + 1)?;
        }
        out.push(term);
    }
    Ok(out)
}
