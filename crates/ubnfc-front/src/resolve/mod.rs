//! @import の相対パス・名前空間・token クラス解決。
//!
//! ファイルは字句的に正規化したパスでキャッシュし、loader は必要な import だけに
//! 呼び出す。入力済みファイルは再読込しない。import 先が複数 grammar を含む場合は
//! UBNFMapper と同じく最後の宣言を公開する。多段 import とその内部参照も解決する。

mod captures;
mod model;
mod tokens;

pub use model::*;
pub use tokens::builtin_class;

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::syntax::{ast::*, parse_file};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveError {
    pub code: String,
    pub message: String,
    pub hint: String,
    pub rule: Option<String>,
    pub grammar: String,
    pub path: PathBuf,
    pub span: Span,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [code: {}] [hint: {}]",
            self.message, self.code, self.hint
        )
    }
}
impl std::error::Error for ResolveError {}

/// 入力順・宣言順を保持する。エラーは (grammar, rule, code, message, path) 順。
pub fn resolve<P: AsRef<Path>>(
    files: &[(P, GrammarAst)],
    loader: &dyn Fn(&Path) -> io::Result<String>,
) -> Result<ResolvedGrammar, Vec<ResolveError>> {
    let mut resolver = Resolver {
        loader,
        files: BTreeMap::new(),
        units: BTreeMap::new(),
        checked: BTreeSet::new(),
        result: ResolvedGrammar::default(),
        errors: Vec::new(),
    };
    for (path, ast) in files {
        let path = normalize(path.as_ref());
        if resolver.files.insert(path.clone(), ast.clone()).is_some() {
            resolver.error(
                "",
                None,
                &path,
                ast.span,
                "E-SOURCE-DUPLICATE",
                format!("duplicate input file: {}", path.display()),
                "Supply each source path once.",
            );
        }
    }
    let mut names = BTreeSet::new();
    let mut pending = Vec::new();
    for (path, file) in files {
        let path = normalize(path.as_ref());
        for (index, decl) in file.grammars.iter().enumerate() {
            if !names.insert(decl.name.clone()) {
                resolver.error(
                    &decl.name,
                    None,
                    &path,
                    decl.span,
                    "E-GRAMMAR-DUPLICATE",
                    format!("duplicate grammar name: {}", decl.name),
                    "Resolve different versions in separate calls.",
                );
            }
            let id = GrammarId(pending.len());
            let unit = resolver.unit(&path, index, decl);
            let mut grammar = PendingGrammar {
                id,
                unit,
                name: decl.name.clone(),
                settings: decl.settings.clone(),
                rules: Vec::new(),
                tokens: Vec::new(),
            };
            resolver.expand(
                &mut grammar,
                &path,
                index,
                decl,
                "",
                &mut vec![path.clone()],
            );
            pending.push(grammar);
        }
    }
    // import 失敗から生じる未定義参照の二次エラーは返さない。
    if resolver.errors.is_empty() {
        for pending in pending {
            resolver.finish(pending);
        }
    }
    if resolver.errors.is_empty() {
        Ok(resolver.result)
    } else {
        resolver.errors.sort_by(|a, b| {
            (&a.grammar, &a.rule, &a.code, &a.message, &a.path)
                .cmp(&(&b.grammar, &b.rule, &b.code, &b.message, &b.path))
        });
        resolver.errors.dedup_by(|a, b| {
            a.grammar == b.grammar
                && a.rule == b.rule
                && a.code == b.code
                && a.message == b.message
                && a.path == b.path
        });
        Err(resolver.errors)
    }
}

struct PendingGrammar {
    id: GrammarId,
    unit: UnitId,
    name: String,
    settings: Vec<GlobalSetting>,
    rules: Vec<(UnitId, String, RuleDecl)>,
    tokens: Vec<(UnitId, String, TokenDecl)>,
}

struct Resolver<'a> {
    loader: &'a dyn Fn(&Path) -> io::Result<String>,
    files: BTreeMap<PathBuf, GrammarAst>,
    units: BTreeMap<(PathBuf, usize), UnitId>,
    checked: BTreeSet<UnitId>,
    result: ResolvedGrammar,
    errors: Vec<ResolveError>,
}

impl Resolver<'_> {
    #[allow(clippy::too_many_arguments)]
    fn error(
        &mut self,
        grammar: &str,
        rule: Option<&str>,
        path: &Path,
        span: Span,
        code: &str,
        message: String,
        hint: &str,
    ) {
        self.errors.push(ResolveError {
            code: code.into(),
            message,
            hint: hint.into(),
            rule: rule.map(str::to_owned),
            grammar: grammar.into(),
            path: path.into(),
            span,
        });
    }

    fn unit(&mut self, path: &Path, index: usize, ast: &GrammarDecl) -> UnitId {
        let key = (path.to_path_buf(), index);
        if let Some(id) = self.units.get(&key) {
            return *id;
        }
        let id = UnitId(self.result.units.len());
        self.result.units.push(GrammarUnit {
            id,
            path: path.into(),
            declaration_index: index,
            ast: ast.clone(),
        });
        self.units.insert(key, id);
        id
    }

    #[allow(clippy::too_many_arguments)]
    fn expand(
        &mut self,
        pending: &mut PendingGrammar,
        path: &Path,
        index: usize,
        ast: &GrammarDecl,
        prefix: &str,
        stack: &mut Vec<PathBuf>,
    ) {
        let unit = self.unit(path, index, ast);
        if self.checked.insert(unit) {
            let roots: Vec<_> = ast
                .rules
                .iter()
                .filter(|r| {
                    r.annotations
                        .iter()
                        .any(|a| matches!(a, Annotation::Root { .. }))
                })
                .collect();
            if roots.len() > 1 {
                self.error(
                    &ast.name,
                    None,
                    path,
                    roots[1].span,
                    "E-ROOT-MULTIPLE",
                    format!(
                        "grammar {} has multiple @root rules: [{}]",
                        ast.name,
                        roots
                            .iter()
                            .map(|r| r.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    "Keep a single @root rule per grammar.",
                );
            }
        }
        pending.rules.extend(
            ast.rules
                .iter()
                .cloned()
                .map(|r| (unit, prefix.to_owned(), r)),
        );
        pending.tokens.extend(
            ast.tokens
                .iter()
                .cloned()
                .map(|t| (unit, prefix.to_owned(), t)),
        );
        let mut aliases = BTreeSet::new();
        for import in &ast.imports {
            if !aliases.insert(&import.alias) {
                self.error(
                    &pending.name,
                    None,
                    path,
                    import.span,
                    "E-IMPORT-DUPLICATE-ALIAS",
                    format!("duplicate import alias: {}", import.alias),
                    "Use a distinct alias for each import.",
                );
                continue;
            }
            let target = normalize(&path.parent().unwrap_or(Path::new("")).join(&import.path));
            if stack.contains(&target) {
                self.error(
                    &pending.name,
                    None,
                    path,
                    import.span,
                    "E-IMPORT-CYCLE",
                    format!(
                        "循環 import を検出しました: {} (スタック: [{}])",
                        target.display(),
                        stack
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    "Remove the circular import.",
                );
                continue;
            }
            if !self.files.contains_key(&target) {
                match (self.loader)(&target) {
                    Ok(source) => match parse_file(&source) {
                        Ok(file) => {
                            self.files.insert(target.clone(), file);
                        }
                        Err(error) => {
                            self.error(
                                &pending.name,
                                None,
                                &target,
                                error.span,
                                "E-IMPORT-SYNTAX",
                                format!("invalid imported grammar '{}': {error}", target.display()),
                                "Fix the imported UBNF source.",
                            );
                            continue;
                        }
                    },
                    Err(error) => {
                        self.error(
                            &pending.name,
                            None,
                            path,
                            import.span,
                            "E-IMPORT-UNRESOLVED",
                            format!("import が解決できません: '{}'", import.path),
                            &format!("{}: {error}", target.display()),
                        );
                        continue;
                    }
                }
            }
            let file = self.files[&target].clone();
            let Some((index, imported)) = file.grammars.iter().enumerate().next_back() else {
                self.error(
                    &pending.name,
                    None,
                    &target,
                    file.span,
                    "E-IMPORT-EMPTY",
                    format!("import contains no grammar: '{}'", import.path),
                    "Declare a grammar in the imported file.",
                );
                continue;
            };
            let to = self.unit(&target, index, imported);
            let namespace = qualify(prefix, &import.alias);
            self.result.imports.push(ImportEdge {
                grammar: pending.id,
                from: unit,
                to,
                alias: import.alias.clone(),
                namespace: namespace.clone(),
                span: import.span,
            });
            stack.push(target.clone());
            self.expand(pending, &target, index, imported, &namespace, stack);
            stack.pop();
        }
    }

    fn register(
        &mut self,
        pending: &PendingGrammar,
        unit: UnitId,
        name: String,
        kind: SymbolKind,
        span: Span,
    ) -> SymbolId {
        if let Some(id) = self.result.lookup(pending.id, &name) {
            let path = self.result.units[unit.0].path.clone();
            self.error(
                &pending.name,
                None,
                &path,
                span,
                "E-SYMBOL-DUPLICATE",
                format!("duplicate rule or token name: {name}"),
                "Use distinct names for rules and tokens.",
            );
            return id;
        }
        let id = SymbolId(self.result.symbols.len());
        self.result.symbols.push(Symbol {
            id,
            grammar: pending.id,
            unit,
            name: name.clone(),
            kind,
            span,
        });
        self.result.symbol_table.insert((pending.id, name), id);
        id
    }

    fn finish(&mut self, pending: PendingGrammar) {
        for (unit, prefix, rule) in &pending.rules {
            self.register(
                &pending,
                *unit,
                qualify(prefix, &rule.name),
                SymbolKind::Rule,
                rule.span,
            );
        }
        for (unit, prefix, token) in &pending.tokens {
            self.register(
                &pending,
                *unit,
                qualify(prefix, &token.name),
                SymbolKind::Token,
                token.span,
            );
        }
        let mut grammar = Grammar {
            id: pending.id,
            unit: pending.unit,
            name: pending.name.clone(),
            settings: pending.settings.clone(),
            rules: Vec::new(),
            tokens: Vec::new(),
            root: None,
        };
        for (unit, prefix, rule) in &pending.rules {
            let name = qualify(prefix, &rule.name);
            let id = self.result.lookup(pending.id, &name).unwrap();
            let body = self.body(&pending, *unit, prefix, &name, &rule.body);
            if grammar.root.is_none()
                && rule
                    .annotations
                    .iter()
                    .any(|a| matches!(a, Annotation::Root { .. }))
            {
                grammar.root = Some(id);
            }
            grammar.rules.push(ResolvedRule {
                id,
                unit: *unit,
                name,
                annotations: rule.annotations.clone(),
                captures: captures::collect(&body),
                body,
                span: rule.span,
            });
        }
        for (unit, prefix, token) in pending.tokens {
            let name = qualify(&prefix, &token.name);
            let id = self.result.lookup(pending.id, &name).unwrap();
            let (value, unresolved) = tokens::classify(&token.value);
            grammar.tokens.push(ResolvedToken {
                id,
                unit,
                name,
                value,
                unresolved,
                declaration: token.value,
                span: token.span,
            });
        }
        self.result.grammars.push(grammar);
    }

    fn body(
        &mut self,
        pending: &PendingGrammar,
        unit: UnitId,
        prefix: &str,
        rule: &str,
        body: &RuleBody,
    ) -> ResolvedBody {
        let alternatives = body
            .alternatives()
            .iter()
            .map(|seq| {
                seq.elements
                    .iter()
                    .filter_map(|element| {
                        let atom = self.atom(pending, unit, prefix, rule, &element.atom);
                        // 本体が未定義でも separator の診断を収集する。
                        let separator = element
                            .separator
                            .as_ref()
                            .map(|separator| self.atom(pending, unit, prefix, rule, separator));
                        let mut atom = atom?;
                        for quantifier in [&element.postfix_quantifier, &element.quantifier]
                            .into_iter()
                            .flatten()
                        {
                            atom = quantify(atom, quantifier);
                        }
                        if let Some(separator) = separator {
                            let separator = separator?;
                            let span = Span {
                                start: atom.span().start,
                                end: separator.span().end,
                            };
                            atom = ResolvedAtom::Separated {
                                element: Box::new(atom),
                                separator: Box::new(separator),
                                span,
                            };
                        }
                        Some(ResolvedElement {
                            atom,
                            capture: element.capture.clone(),
                            typeof_constraint: element.typeof_constraint.clone(),
                            span: element.span,
                        })
                    })
                    .collect()
            })
            .collect();
        ResolvedBody {
            alternatives,
            span: body.span(),
        }
    }

    fn atom(
        &mut self,
        pending: &PendingGrammar,
        unit: UnitId,
        prefix: &str,
        rule: &str,
        atom: &Atom,
    ) -> Option<ResolvedAtom> {
        use ResolvedAtom as R;
        let span = atom.span();
        Some(match atom {
            Atom::Ref {
                namespace, name, ..
            } => {
                let local = namespace
                    .as_ref()
                    .map_or_else(|| name.clone(), |ns| format!("{ns}.{name}"));
                let qualified = qualify(prefix, &local);
                let Some(symbol) = self.result.lookup(pending.id, &qualified) else {
                    let known = self
                        .result
                        .symbols
                        .iter()
                        .filter(|s| s.grammar == pending.id)
                        .map(|s| s.name.as_str());
                    let hint = suggest_similar_name(&qualified, known).map_or_else(
                        || format!("Define a rule or token named '{qualified}'."),
                        |name| format!("Did you mean '{name}'?"),
                    );
                    let path = self.result.units[unit.0].path.clone();
                    self.error(
                        &pending.name,
                        Some(rule),
                        &path,
                        span,
                        "E-RULE-UNDEFINED",
                        format!("rule {rule} references undefined rule or token '{qualified}'"),
                        &hint,
                    );
                    return None;
                };
                R::Ref { symbol, span }
            }
            Atom::Literal { value, raw, .. } => R::Literal {
                value: value.clone(),
                raw: raw.clone(),
                span,
            },
            Atom::Error { message, .. } => R::Error {
                message: message.clone(),
                span,
            },
            Atom::Group { body, .. } => R::Group {
                body: self.body(pending, unit, prefix, rule, body),
                span,
            },
            Atom::Optional { body, .. } => R::Optional {
                body: self.body(pending, unit, prefix, rule, body),
                span,
            },
            Atom::Repeat { body, .. } => R::Repeat {
                body: self.body(pending, unit, prefix, rule, body),
                span,
            },
        })
    }
}

fn qualify(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}

fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(result.components().next_back(), Some(Component::Normal(_))) =>
            {
                result.pop();
            }
            Component::ParentDir if result.has_root() => {}
            _ => result.push(part.as_os_str()),
        }
    }
    result
}

fn quantify(atom: ResolvedAtom, quantifier: &Quantifier) -> ResolvedAtom {
    let inner_span = atom.span();
    let span = Span {
        start: inner_span.start,
        end: quantifier.span().end,
    };
    match quantifier {
        Quantifier::Plus { .. } => ResolvedAtom::OneOrMore {
            body: Box::new(atom),
            span,
        },
        Quantifier::Bounded { min, max, .. } => ResolvedAtom::Bounded {
            body: Box::new(atom),
            min: *min,
            max: *max,
            span,
        },
        Quantifier::Question { .. } | Quantifier::Star { .. } => {
            let body = ResolvedBody {
                alternatives: vec![vec![ResolvedElement {
                    atom,
                    capture: None,
                    typeof_constraint: None,
                    span: inner_span,
                }]],
                span: inner_span,
            };
            if matches!(quantifier, Quantifier::Question { .. }) {
                ResolvedAtom::Optional { body, span }
            } else {
                ResolvedAtom::Repeat { body, span }
            }
        }
    }
}

/// Java と同じ UTF-16 編集距離。距離2以内で最小、同距離なら宣言順の最初。
pub fn suggest_similar_name<'a>(
    unknown: &str,
    known: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let mut best = None;
    let mut distance = 3;
    for candidate in known {
        let b: Vec<_> = candidate.encode_utf16().collect();
        let mut row: Vec<_> = (0..=b.len()).collect();
        for (i, a) in unknown.encode_utf16().enumerate() {
            let mut previous = row[0];
            row[0] = i + 1;
            for (j, &b) in b.iter().enumerate() {
                let old = row[j + 1];
                row[j + 1] = (row[j] + 1)
                    .min(old + 1)
                    .min(previous + usize::from(a != b));
                previous = old;
            }
        }
        if row[b.len()] < distance {
            distance = row[b.len()];
            best = Some(candidate);
        }
    }
    best
}
