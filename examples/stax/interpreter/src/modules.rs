use crate::{
    checker,
    generated::{self, ast::Ast},
    machine::{self, Limits},
    model::*,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
};

pub struct Program {
    env: Rc<Env>,
}
#[derive(Default)]
struct Loader {
    cache: BTreeMap<PathBuf, Rc<Env>>,
    active: BTreeSet<PathBuf>,
    bytes: usize,
    definitions: usize,
    expansion: usize,
    snapshots: usize,
}
pub(crate) fn reserved(name: &str) -> bool {
    matches!(
        name,
        "Int"
            | "Float"
            | "Bool"
            | "Str"
            | "true"
            | "false"
            | "let"
            | "type"
            | "struct"
            | "use"
            | "macro"
            | "dup"
            | "drop"
            | "swap"
            | "over"
            | "rot"
            | "nip"
            | "and"
            | "or"
            | "not"
            | "mod"
            | "neg"
            | "concat"
            | "str_len"
            | "int_to_str"
            | "print"
            | "newline"
            | "call"
            | "choose"
            | "recurse"
    )
}
fn insert(env: &mut Env, name: &str, symbol: Symbol) -> Result<()> {
    if name.len() > 256 {
        return fail("宣言名の長さ上限");
    }
    if reserved(name) {
        return fail(format!("予約済みの名前は宣言できない: {name}"));
    }
    if env.contains_key(name) && !matches!(env.get(name), Some(Symbol::Field)) {
        return fail(format!("重複した宣言: {name}"));
    }
    if env.len() >= 2048 {
        return fail("モジュールの宣言名上限");
    }
    env.insert(name.into(), symbol);
    Ok(())
}
impl Loader {
    fn load(&mut self, path: &Path) -> Result<Rc<Env>> {
        let path = fs::canonicalize(path)
            .map_err(|e| Error(format!("取込みファイルが欠落: {}: {e}", path.display())))?;
        if self.active.contains(&path) {
            return fail(format!("use の循環: {}", path.display()));
        }
        if let Some(e) = self.cache.get(&path) {
            return Ok(e.clone());
        }
        if self.cache.len() + self.active.len() >= 64 {
            return fail("モジュール数上限");
        }
        let size = fs::metadata(&path).map_err(|e| Error(e.to_string()))?.len();
        if size > 1024 * 1024 {
            return fail("ソースファイル長上限");
        }
        let mut source = String::new();
        fs::File::open(&path)
            .map_err(|e| Error(e.to_string()))?
            .take(1024 * 1024 + 1)
            .read_to_string(&mut source)
            .map_err(|e| Error(e.to_string()))?;
        if source.len() > 1024 * 1024 {
            return fail("ソースファイル長上限");
        }
        self.bytes += source.len();
        if self.bytes > 8 * 1024 * 1024 {
            return fail("全ソース長上限");
        }
        let parsed = generated::parse_with_options(
            &source,
            generated::ParseOptions {
                max_depth: 256,
                ..Default::default()
            },
        );
        if !parsed.ok || parsed.consumed_cp != source.chars().count() {
            return fail(format!(
                "{}: 構文エラー: {:?}",
                path.display(),
                parsed.diagnostics
            ));
        }
        let Some(Ast::g_StaxAST_2e_Program(program)) = parsed.ast else {
            return fail("Program AST が生成されない");
        };
        self.active.insert(path.clone());
        let mut env = Env::new();
        for item in program.g_items {
            self.definitions += 1;
            if self.definitions > 2048 {
                return fail("全宣言数上限");
            }
            match item {
                Ast::g_StaxAST_2e_UseDecl(n) => {
                    let library = self.load(&path.parent().unwrap().join(string(&n.g_path)?))?;
                    for name in n.g_names {
                        let Ast::g_StaxAST_2e_ImportName(name) = name else {
                            return fail("ImportName AST が必要");
                        };
                        let symbol = library.get(&name.g_name).ok_or_else(|| {
                            Error(format!(
                                "取込み名が実在しない: {}（{}）",
                                name.g_name, n.g_path
                            ))
                        })?;
                        // 複数の struct に共有される field 名は metadata として1回だけ取込む。
                        if matches!(symbol, Symbol::Field)
                            && matches!(env.get(&name.g_name), Some(Symbol::Field))
                        {
                            continue;
                        }
                        insert(&mut env, &name.g_name, symbol.clone())?;
                    }
                }
                Ast::g_StaxAST_2e_StructDef(n) => {
                    let mut fields = vec![];
                    let mut names = BTreeSet::new();
                    for f in n.g_fields {
                        let Ast::g_StaxAST_2e_FieldDecl(f) = f else {
                            return fail("FieldDecl AST が必要");
                        };
                        if !names.insert(f.g_name.clone()) {
                            return fail("struct 内のフィールドが重複");
                        }
                        if fields.len() >= 256 {
                            return fail("struct のフィールド数上限");
                        }
                        if f.g_name.len() > 256 {
                            return fail("フィールド名の長さ上限");
                        }
                        fields.push((f.g_name.clone(), type_ref(&f.g_type, &env)?));
                    }
                    let def = Rc::new(Structure {
                        id: format!("{}::{}", path.display(), n.g_name),
                        fields,
                    });
                    insert(&mut env, &n.g_name, Symbol::Constructor(def.clone()))?;
                    for (name, _) in &def.fields {
                        if env.len() >= 2048 && !env.contains_key(name) {
                            return fail("モジュールの宣言名上限");
                        }
                        env.entry(name.clone()).or_insert(Symbol::Field);
                    }
                }
                Ast::g_StaxAST_2e_TypeDef(n) => {
                    let ty = if let Some(sum) = n.g_sum {
                        let ty = Ty::Enum(format!("{}::{}", path.display(), n.g_name).into());
                        let Ast::g_StaxAST_2e_SumBody(sum) = *sum else {
                            return fail("SumBody AST が必要");
                        };
                        for branch in sum.g_variants {
                            let Ast::g_StaxAST_2e_Variant(v) = branch else {
                                return fail("Variant AST が必要");
                            };
                            insert(
                                &mut env,
                                &v.g_name,
                                Symbol::Variant(ty.clone(), v.g_name.clone().into()),
                            )?;
                        }
                        ty
                    } else {
                        let Some(alias) = n.g_alias else {
                            return fail("型の本体が必要");
                        };
                        let Ast::g_StaxAST_2e_AliasBody(a) = *alias else {
                            return fail("AliasBody AST が必要");
                        };
                        type_ref(&a.g_type, &env)?
                    };
                    insert(&mut env, &n.g_name, Symbol::Type(ty))?;
                }
                Ast::g_StaxAST_2e_WordDef(n) => {
                    self.word(&mut env, &path, n.g_name, *n.g_effect, n.g_body, false)?;
                }
                Ast::g_StaxAST_2e_MacroDef(n) => {
                    self.word(&mut env, &path, n.g_name, *n.g_effect, n.g_body, true)?;
                }
                _ => return fail("最上位宣言の AST が不正"),
            }
        }
        self.active.remove(&path);
        let env = Rc::new(env);
        self.cache.insert(path, env.clone());
        Ok(env)
    }
    fn word(
        &mut self,
        env: &mut Env,
        path: &Path,
        name: String,
        effect: Ast,
        body: Vec<Ast>,
        is_macro: bool,
    ) -> Result<()> {
        if env
            .values()
            .filter(|v| matches!(v, Symbol::Word(_) | Symbol::Macro(_)))
            .count()
            >= 256
        {
            return fail("語の定義数上限");
        }
        let effect = declared_effect(&effect, env)?;
        if name == "main" && (is_macro || !effect.input.is_empty() || !effect.output.is_empty()) {
            return fail("main の効果は ( -- ) が必要");
        }
        if is_macro
            && (effect.input.iter().any(|t| !matches!(t, Ty::Quote(_)))
                || effect.output.len() != 1
                || !matches!(effect.output[0], Ty::Quote(_)))
        {
            return fail("macro の入力は引用、出力は引用1つに限定する");
        }
        self.snapshots += env.len();
        if self.snapshots > 100_000 {
            return fail("辞書 snapshot の累積上限");
        }
        let mut word = Word {
            name: name.clone(),
            path: path.into(),
            effect,
            body: Rc::new(body),
            env: Rc::new(env.clone()),
        };
        checker::check(&word)?;
        word.body = Rc::new(machine::expand(&word.body, env, &mut self.expansion, 0)?);
        checker::check(&word)?;
        insert(
            env,
            &name,
            if is_macro {
                Symbol::Macro(Rc::new(word))
            } else {
                Symbol::Word(Rc::new(word))
            },
        )
    }
}
impl Program {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let env = Loader::default().load(path.as_ref())?;
        Ok(Self { env })
    }
    pub fn expansion(&self) -> String {
        let mut expanded = String::new();
        for (name, symbol) in self.env.iter() {
            if let Symbol::Word(word) = symbol {
                expanded.push_str(&format!("{name}: {:?}\n", word.body));
            }
        }
        expanded
    }
    pub fn run(&self, limits: Limits) -> Result<String> {
        match self.env.get("main") {
            Some(Symbol::Word(w)) => machine::execute(w.clone(), limits),
            _ => fail("実行には main が必要（ライブラリの検査は --check）"),
        }
    }
}
