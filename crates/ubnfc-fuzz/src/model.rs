//! 生成した UBNF の構造モデル。原文への描画、入力の導出、縮小候補の列挙を担う。
//! 構造を保つことで「文法を縮小する」を文字列編集ではなく木の操作として行える。

use crate::rng::Rng;
use std::collections::BTreeSet;
use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quant {
    Plus,
    Question,
    Star,
    /// `{min}` は max=Some(min)、`{min,}` は max=None。
    Bounded(u32, Option<u32>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Atom {
    Literal(String),
    /// ルール名・token 名、または `alias.Rule`。
    Ref(String),
    Group(Body),
    Optional(Body),
    Repeat(Body),
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    /// 候補の**先頭**に置く注釈（UBNF v2 の `@mapping` / `@leftAssoc` / `@rightAssoc`）。
    /// 候補の 2 番目以降の要素に付けると構文エラーになるので、生成側が先頭にだけ置く。
    pub lead: Vec<String>,
    pub typeof_capture: Option<String>,
    pub atom: Atom,
    pub quant: Option<Quant>,
    pub separator: Option<Atom>,
    pub capture: Option<String>,
}

impl Rule {
    /// 雛形の綴りを持たない通常の規則。
    pub fn plain(annotations: Vec<String>, name: String, body: Body) -> Self {
        Self {
            annotations,
            name,
            body,
            template: None,
        }
    }
}

impl Element {
    pub fn new(atom: Atom) -> Self {
        Self {
            lead: Vec::new(),
            typeof_capture: None,
            atom,
            quant: None,
            separator: None,
            capture: None,
        }
    }
    pub fn captured(atom: Atom, name: &str) -> Self {
        Self {
            capture: Some(name.into()),
            ..Self::new(atom)
        }
    }
}

pub type Seq = Vec<Element>;

/// UBNF の RuleBody。`alternatives` は常に 1 つ以上（空の Seq は epsilon）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    pub alternatives: Vec<Seq>,
}

impl Body {
    pub fn single(elements: Seq) -> Self {
        Self {
            alternatives: vec![elements],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub name: String,
    /// `= ` の右辺の原文。
    pub declaration: String,
    /// 導出で使う受理例。空なら導出はゼロ幅として扱う。
    pub samples: Vec<String>,
    /// 生成器の被覆集計に使う機能名。
    pub feature: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// `@mapping(...)` などの描画済み注釈。縮小は要素の削除で行う。
    pub annotations: Vec<String>,
    pub name: String,
    pub body: Body,
    /// パラメータ化ルール（D-043）の実体化を**原文として**書くときの本体
    /// （例 `Tpl<Op0, ','>`）。`body` は常に展開後の形を持つので、モデル・導出・
    /// 期待値はすべて展開後を見る。つまりこの欄は「同じ文法の別の綴り」であり、
    /// front の展開がその綴りと一致することを両 backend が突き合わせる。
    pub template: Option<String>,
}

/// `@import` で参照される 2 つ目のファイル。規則は最小限に保つ。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub alias: String,
    pub file: String,
    pub grammar: String,
    pub tokens: Vec<Token>,
    pub rules: Vec<Rule>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grammar {
    pub name: String,
    /// `@package: ...` などのグローバル設定（描画済み 1 行）。
    pub settings: Vec<String>,
    /// パラメータ化ルールの宣言（描画済み 1 行）。
    pub templates: Vec<String>,
    pub tokens: Vec<Token>,
    pub rules: Vec<Rule>,
    pub import: Option<Import>,
    /// 生成時に投入した機能名。discard 率を機能別に出すために保持する。
    pub features: BTreeSet<&'static str>,
}

fn quote(text: &str) -> String {
    let mut out = String::from("'");
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out.push('\'');
    out
}

fn render_body(body: &Body, out: &mut String) {
    for (index, alternative) in body.alternatives.iter().enumerate() {
        if index > 0 {
            out.push_str(" | ");
        }
        for (position, element) in alternative.iter().enumerate() {
            if position > 0 {
                out.push(' ');
            }
            render_element(element, out);
        }
    }
}

fn render_atom(atom: &Atom, out: &mut String) {
    match atom {
        Atom::Literal(text) => out.push_str(&quote(text)),
        Atom::Ref(name) => out.push_str(name),
        Atom::Group(body) => {
            out.push_str("( ");
            render_body(body, out);
            out.push_str(" )");
        }
        Atom::Optional(body) => {
            out.push_str("[ ");
            render_body(body, out);
            out.push_str(" ]");
        }
        Atom::Repeat(body) => {
            out.push_str("{ ");
            render_body(body, out);
            out.push_str(" }");
        }
        Atom::Error(message) => {
            let _ = write!(out, "ERROR({})", quote(message));
        }
    }
}

fn render_element(element: &Element, out: &mut String) {
    for annotation in &element.lead {
        out.push_str(annotation);
        out.push(' ');
    }
    if let Some(target) = &element.typeof_capture {
        let _ = write!(out, "@typeof({target}) ");
    }
    render_atom(&element.atom, out);
    match element.quant {
        None => {}
        Some(Quant::Plus) => out.push('+'),
        Some(Quant::Question) => out.push('?'),
        Some(Quant::Star) => out.push('*'),
        Some(Quant::Bounded(min, None)) => {
            let _ = write!(out, "{{{min},}}");
        }
        Some(Quant::Bounded(min, Some(max))) => {
            if min == max {
                let _ = write!(out, "{{{min}}}");
            } else {
                let _ = write!(out, "{{{min},{max}}}");
            }
        }
    }
    if let Some(separator) = &element.separator {
        out.push_str(" % ");
        render_atom(separator, out);
    }
    if let Some(name) = &element.capture {
        let _ = write!(out, " @{name}");
    }
}

fn render_unit(
    name: &str,
    imports: &[String],
    settings: &[String],
    templates: &[String],
    tokens: &[Token],
    rules: &[Rule],
) -> String {
    let mut out = format!("grammar {name} {{\n");
    for line in imports {
        let _ = writeln!(out, "  {line}");
    }
    for line in settings {
        let _ = writeln!(out, "  {line}");
    }
    for token in tokens {
        let _ = writeln!(out, "  token {} = {}", token.name, token.declaration);
    }
    // 実体化されている雛形だけを宣言する（縮小で使われなくなったら消える）。
    for declaration in templates {
        let used = format!("{}<", declaration.split('<').next().unwrap_or(""));
        if rules
            .iter()
            .any(|r| r.template.as_deref().is_some_and(|t| t.contains(&used)))
        {
            let _ = writeln!(out, "  {declaration}");
        }
    }
    for rule in rules {
        out.push_str("  ");
        for annotation in &rule.annotations {
            let _ = write!(out, "{annotation} ");
        }
        let _ = write!(out, "{} ::= ", rule.name);
        match &rule.template {
            Some(body) => out.push_str(body),
            None => render_body(&rule.body, &mut out),
        }
        out.push_str(";\n");
    }
    out.push_str("}\n");
    out
}

impl Grammar {
    /// main ファイルの原文。
    pub fn render(&self) -> String {
        let imports = match &self.import {
            None => Vec::new(),
            Some(import) => vec![format!(
                "@import {} from {}",
                import.alias,
                quote(&import.file)
            )],
        };
        render_unit(
            &self.name,
            &imports,
            &self.settings,
            &self.templates,
            &self.tokens,
            &self.rules,
        )
    }

    /// import ファイルの (ファイル名, 原文)。
    pub fn imported_source(&self) -> Option<(String, String)> {
        let import = self.import.as_ref()?;
        Some((
            import.file.clone(),
            render_unit(
                &import.grammar,
                &[],
                &[],
                &[],
                &import.tokens,
                &import.rules,
            ),
        ))
    }

    /// 内容だけで決まる識別子。ビルド成果物のキャッシュ鍵に使う。
    pub fn digest(&self) -> String {
        let mut text = self.render();
        if let Some((file, source)) = self.imported_source() {
            text.push_str(&file);
            text.push_str(&source);
        }
        let hash = text.bytes().fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
        format!("{hash:016x}")
    }

    pub fn root_rule(&self) -> Option<&Rule> {
        self.rules
            .iter()
            .find(|rule| rule.annotations.iter().any(|a| a == "@root"))
            .or_else(|| self.rules.last())
    }

    fn lookup(&self, name: &str) -> Resolved<'_> {
        if let Some((alias, target)) = name.split_once('.') {
            if let Some(import) = self.import.as_ref().filter(|i| i.alias == alias) {
                if let Some(rule) = import.rules.iter().find(|r| r.name == target) {
                    return Resolved::Rule(rule);
                }
                if let Some(token) = import.tokens.iter().find(|t| t.name == target) {
                    return Resolved::Token(token);
                }
            }
            return Resolved::Unknown;
        }
        if let Some(rule) = self.rules.iter().find(|r| r.name == name) {
            return Resolved::Rule(rule);
        }
        if let Some(token) = self.tokens.iter().find(|t| t.name == name) {
            return Resolved::Token(token);
        }
        Resolved::Unknown
    }

    /// 根から 1 つの文を導出する。受理される入力を多く作るのが目的で、
    /// 先読み・EOF・左再帰などの都合で拒否される導出もそのまま使う。
    pub fn derive(&self, rng: &mut Rng, trivia: &[&str]) -> String {
        let Some(root) = self.root_rule() else {
            return String::new();
        };
        let mut out = String::new();
        let mut budget = 400usize;
        self.derive_body(&root.body, rng, trivia, &mut out, &mut budget, 0);
        out
    }

    fn derive_body(
        &self,
        body: &Body,
        rng: &mut Rng,
        trivia: &[&str],
        out: &mut String,
        budget: &mut usize,
        depth: usize,
    ) {
        if depth > 12 || *budget == 0 {
            return;
        }
        let alternative = &body.alternatives[rng.below(body.alternatives.len())];
        for (position, element) in alternative.iter().enumerate() {
            if position > 0 && !trivia.is_empty() && rng.chance(25) {
                out.push_str(rng.pick(trivia));
            }
            self.derive_element(element, rng, trivia, out, budget, depth);
        }
    }

    fn derive_element(
        &self,
        element: &Element,
        rng: &mut Rng,
        trivia: &[&str],
        out: &mut String,
        budget: &mut usize,
        depth: usize,
    ) {
        let count = match element.quant {
            None => 1,
            Some(Quant::Plus) => rng.between(1, 3),
            Some(Quant::Question) => rng.below(2),
            Some(Quant::Star) => rng.below(3),
            Some(Quant::Bounded(min, max)) => {
                let min = min as usize;
                let max = max.map_or(min + 2, |m| m as usize).min(min + 2).max(min);
                rng.between(min, max)
            }
        };
        for index in 0..count {
            if index > 0 {
                if let Some(separator) = &element.separator {
                    self.derive_atom(separator, rng, trivia, out, budget, depth);
                }
            }
            self.derive_atom(&element.atom, rng, trivia, out, budget, depth);
        }
    }

    fn derive_atom(
        &self,
        atom: &Atom,
        rng: &mut Rng,
        trivia: &[&str],
        out: &mut String,
        budget: &mut usize,
        depth: usize,
    ) {
        if *budget == 0 || depth > 12 {
            return;
        }
        *budget -= 1;
        match atom {
            Atom::Literal(text) => out.push_str(text),
            Atom::Error(_) => {}
            Atom::Group(body) => self.derive_body(body, rng, trivia, out, budget, depth + 1),
            Atom::Optional(body) => {
                if rng.chance(60) {
                    self.derive_body(body, rng, trivia, out, budget, depth + 1);
                }
            }
            Atom::Repeat(body) => {
                for _ in 0..rng.below(3) {
                    self.derive_body(body, rng, trivia, out, budget, depth + 1);
                }
            }
            Atom::Ref(name) => match self.lookup(name) {
                Resolved::Rule(rule) => {
                    self.derive_body(&rule.body, rng, trivia, out, budget, depth + 1)
                }
                Resolved::Token(token) => {
                    if !token.samples.is_empty() {
                        out.push_str(rng.pick(&token.samples));
                    }
                }
                Resolved::Unknown => {}
            },
        }
    }
}

enum Resolved<'a> {
    Rule(&'a Rule),
    Token(&'a Token),
    Unknown,
}

/// 差分が保たれる限り採用される縮小候補を、単純なものから順に返す。
pub fn shrink_candidates(grammar: &Grammar) -> Vec<Grammar> {
    let mut out = Vec::new();
    if grammar.import.is_some() {
        let mut candidate = grammar.clone();
        candidate.import = None;
        out.push(candidate);
    }
    for index in 0..grammar.settings.len() {
        let mut candidate = grammar.clone();
        candidate.settings.remove(index);
        out.push(candidate);
    }
    // 「雛形の綴りをやめて展開後を直接書く」候補。乖離が雛形の展開に由来するか
    // どうかを縮小の 1 手で切り分けられる。
    if grammar.rules.iter().any(|r| r.template.is_some()) {
        let mut candidate = grammar.clone();
        candidate.templates.clear();
        for rule in &mut candidate.rules {
            rule.template = None;
        }
        out.push(candidate);
    }
    for index in 0..grammar.rules.len() {
        if grammar.rules.len() > 1 {
            let mut candidate = grammar.clone();
            candidate.rules.remove(index);
            out.push(candidate);
        }
        for annotation in 0..grammar.rules[index].annotations.len() {
            let mut candidate = grammar.clone();
            candidate.rules[index].annotations.remove(annotation);
            candidate.rules[index].template = None;
            out.push(candidate);
        }
        let alternatives = grammar.rules[index].body.alternatives.len();
        if alternatives > 1 {
            for alternative in 0..alternatives {
                let mut candidate = grammar.clone();
                candidate.rules[index].body.alternatives.remove(alternative);
                candidate.rules[index].template = None;
                out.push(candidate);
            }
        }
        for alternative in 0..alternatives {
            let elements = grammar.rules[index].body.alternatives[alternative].len();
            for element in 0..elements {
                let mut candidate = grammar.clone();
                candidate.rules[index].body.alternatives[alternative].remove(element);
                candidate.rules[index].template = None;
                out.push(candidate);
                for simpler in
                    simplify_element(&grammar.rules[index].body.alternatives[alternative][element])
                {
                    let mut candidate = grammar.clone();
                    candidate.rules[index].body.alternatives[alternative][element] = simpler;
                    candidate.rules[index].template = None;
                    out.push(candidate);
                }
            }
        }
    }
    for index in 0..grammar.tokens.len() {
        let mut candidate = grammar.clone();
        candidate.tokens.remove(index);
        out.push(candidate);
    }
    out
}

fn simplify_element(element: &Element) -> Vec<Element> {
    let mut out = Vec::new();
    if !element.lead.is_empty() {
        out.push(Element {
            lead: Vec::new(),
            ..element.clone()
        });
    }
    if element.typeof_capture.is_some() {
        out.push(Element {
            typeof_capture: None,
            ..element.clone()
        });
    }
    if element.separator.is_some() {
        out.push(Element {
            separator: None,
            ..element.clone()
        });
    }
    if element.quant.is_some() {
        out.push(Element {
            quant: None,
            ..element.clone()
        });
    }
    if element.capture.is_some() {
        out.push(Element {
            capture: None,
            ..element.clone()
        });
    }
    // 入れ子の本体を 1 つの代替へ落とす。参照は残すので未定義参照にならない。
    match &element.atom {
        Atom::Group(body) | Atom::Optional(body) | Atom::Repeat(body) => {
            if body.alternatives.len() > 1 {
                out.push(Element {
                    atom: match &element.atom {
                        Atom::Group(_) => Atom::Group(Body::single(body.alternatives[0].clone())),
                        Atom::Optional(_) => {
                            Atom::Optional(Body::single(body.alternatives[0].clone()))
                        }
                        _ => Atom::Repeat(Body::single(body.alternatives[0].clone())),
                    },
                    ..element.clone()
                });
            }
            if body.alternatives.len() == 1 && body.alternatives[0].len() == 1 {
                out.push(Element {
                    atom: body.alternatives[0][0].atom.clone(),
                    ..element.clone()
                });
            }
        }
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grammar() -> Grammar {
        Grammar {
            name: "Fuzz".into(),
            settings: vec!["@whitespace: javaStyle".into()],
            templates: Vec::new(),
            tokens: vec![Token {
                name: "N".into(),
                declaration: "NumberParser".into(),
                samples: vec!["1".into()],
                feature: "token:number",
            }],
            rules: vec![Rule {
                template: None,
                annotations: vec!["@root".into()],
                name: "Root".into(),
                body: Body::single(vec![
                    Element::captured(Atom::Ref("N".into()), "head"),
                    Element {
                        quant: Some(Quant::Bounded(1, Some(2))),
                        separator: Some(Atom::Literal(",".into())),
                        ..Element::new(Atom::Optional(Body::single(vec![Element::new(
                            Atom::Literal("x".into()),
                        )])))
                    },
                ]),
            }],
            import: None,
            features: BTreeSet::new(),
        }
    }

    #[test]
    fn rendering_round_trips_through_the_real_ubnf_parser() {
        let source = grammar().render();
        let file = ubnfc_front::syntax::parse_file(&source).expect(&source);
        assert_eq!(file.grammars.len(), 1);
        assert_eq!(file.grammars[0].rules.len(), 1);
        assert!(source.contains("[ 'x' ]{1,2} % ','"), "{source}");
        assert!(source.contains("N @head"), "{source}");
    }

    #[test]
    fn escapes_survive_a_parse_and_derivation_only_emits_declared_text() {
        let mut g = grammar();
        g.rules[0].body.alternatives[0][1].atom = Atom::Literal("a'b\\c\nd".into());
        let source = g.render();
        let file = ubnfc_front::syntax::parse_file(&source).expect(&source);
        let text = format!("{:?}", file.grammars[0].rules[0]);
        assert!(text.contains("a'b\\\\c\\nd"), "{text}");
        let mut rng = Rng::new(3);
        for _ in 0..50 {
            let input = g.derive(&mut rng, &[]);
            assert!(input.starts_with('1'), "{input:?}");
        }
    }

    #[test]
    fn shrink_candidates_are_strictly_smaller_and_still_render() {
        let g = grammar();
        let candidates = shrink_candidates(&g);
        assert!(!candidates.is_empty());
        for candidate in candidates {
            assert_ne!(candidate, g);
            let source = candidate.render();
            assert!(ubnfc_front::syntax::parse_file(&source).is_ok(), "{source}");
        }
    }
}
