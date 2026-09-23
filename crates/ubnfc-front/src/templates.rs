//! パラメータ化ルール（D-043）の単相化。
//!
//! `Name<P1, ..., Pn> ::= body ;` は**規則ではなく規則本体の雛形**で、
//! `Name<A1, ..., An>` の位置へ**その場で展開**される。展開後は雛形も実体化も
//! 残らず、**新しい規則も新しい名前も作らない**。したがって
//!
//! - AST（`@mapping` のクラスと `params`）は手書きしたときと同一、
//! - capture は実体化した規則のフィールドになり、
//! - `rulePath` / `deepestRule` / expected ラベルは 1 文字も変わらない。
//!
//! これが Menhir の `separated_list(sep, X)`（新しい非終端を作る）ではなく
//! `%inline` 側を採った理由である。`@mapping` が規則ごとに AST クラスを決める
//! UBNF では、新しい非終端は**必ず AST を変える**ので既存文法へ後付けできない。
//!
//! 展開は resolve より前の構文木変換なので、下流（resolve / validate / lower /
//! analysis / backend）は v1 の正規形しか見ない（D-038 / D-039 と同じ方法）。

use std::collections::{BTreeMap, BTreeSet};

use crate::syntax::ast::*;
use crate::validate::ValidationIssue;

use crate::desugar::LATEST_LANGUAGE_VERSION;

fn issue(
    code: &str,
    message: String,
    hint: &str,
    rule: Option<&str>,
    grammar: &str,
) -> ValidationIssue {
    ValidationIssue {
        code: code.into(),
        message,
        hint: hint.into(),
        rule: rule.map(ToOwned::to_owned),
        grammar: grammar.into(),
    }
}

/// grammar 1 つ分の雛形を展開し、`decl.templates` を空にする。
pub fn expand(decl: &mut GrammarDecl, version: u64, issues: &mut Vec<ValidationIssue>) {
    let templates = std::mem::take(&mut decl.templates);
    if templates.is_empty() && !decl.rules.iter().any(|r| body_instantiates(&r.body)) {
        return;
    }
    let grammar = decl.name.clone();
    if version < LATEST_LANGUAGE_VERSION {
        issues.push(issue(
            "E-TEMPLATE-UNSUPPORTED",
            "パラメータ化ルールは @ubnf: 2 でだけ使えます".into(),
            "grammar に '@ubnf: 2' を宣言してください。",
            None,
            &grammar,
        ));
        for rule in &mut decl.rules {
            degrade_body(&mut rule.body);
        }
        return;
    }
    let mut ok = true;
    // 名前の衝突。雛形は規則でも token でもないが、同じ綴りは読み手を惑わせる。
    let names: BTreeSet<&str> = decl
        .rules
        .iter()
        .map(|r| r.name.as_str())
        .chain(decl.tokens.iter().map(|t| t.name.as_str()))
        .collect();
    let mut by_name: BTreeMap<String, TemplateDecl> = BTreeMap::new();
    for template in templates {
        if !template.annotations.is_empty() {
            ok = false;
            issues.push(issue(
                "E-TEMPLATE-ANNOTATION",
                format!("{}: 雛形に注釈は置けません", template.name),
                "注釈は実体化する規則の側に書いてください（AST は実体化した規則のものです）。",
                Some(&template.name),
                &grammar,
            ));
        }
        if names.contains(template.name.as_str()) || by_name.contains_key(&template.name) {
            ok = false;
            issues.push(issue(
                "E-TEMPLATE-NAME-COLLISION",
                format!(
                    "{}: 雛形の名前が既存の規則 / token / 雛形と重複しています",
                    template.name
                ),
                "雛形に別の名前を付けてください。",
                Some(&template.name),
                &grammar,
            ));
        }
        // パラメータ名が規則 / token と重ならないことが、名前による代入が
        // 変数捕獲を起こさないための（唯一の）条件である。
        let mut seen = BTreeSet::new();
        for param in &template.params {
            if names.contains(param.as_str()) {
                ok = false;
                issues.push(issue(
                    "E-TEMPLATE-PARAM-SHADOW",
                    format!(
                        "{}: パラメータ {param} が同名の規則 / token を隠します",
                        template.name
                    ),
                    "パラメータに別の名前を付けてください（代入は名前で行います）。",
                    Some(&template.name),
                    &grammar,
                ));
            }
            if !seen.insert(param.as_str()) {
                ok = false;
                issues.push(issue(
                    "E-TEMPLATE-PARAM-DUPLICATE",
                    format!("{}: パラメータ {param} が重複しています", template.name),
                    "パラメータ名は雛形の中で一意にしてください。",
                    Some(&template.name),
                    &grammar,
                ));
            }
        }
        for param in &template.params {
            if !body_mentions(&template.body, param) {
                issues.push(issue(
                    "W-TEMPLATE-PARAM-UNUSED",
                    format!(
                        "{}: パラメータ {param} が本体で使われていません",
                        template.name
                    ),
                    "使わないパラメータは削除してください。",
                    Some(&template.name),
                    &grammar,
                ));
            }
        }
        by_name.insert(template.name.clone(), template);
    }
    // 循環の検出。展開はその場での置換なので、**どんな循環も無限展開**になる。
    // 雛形の依存グラフは有限なので、循環の有無は完全に判定できる（下界ではない）。
    if let Some(found) = cycle(&by_name) {
        issues.push(issue(
            "E-TEMPLATE-RECURSION",
            format!("雛形が循環しています: {}", found.join(" -> ")),
            "雛形は展開されて消えるので再帰できません。再帰が要る箇所は通常の規則にしてください。",
            found.first().map(String::as_str),
            &grammar,
        ));
        for rule in &mut decl.rules {
            degrade_body(&mut rule.body);
        }
        return;
    }
    // 実体化されない雛形は書き間違いのことが多い。
    let mut instantiated: BTreeSet<&str> = BTreeSet::new();
    for rule in &decl.rules {
        collect_body(&rule.body, &by_name, &mut instantiated);
    }
    for template in by_name.values() {
        collect_body(&template.body, &by_name, &mut instantiated);
    }
    for name in by_name.keys() {
        if !instantiated.contains(name.as_str()) {
            issues.push(issue(
                "W-TEMPLATE-UNUSED",
                format!("{name}: 実体化されていない雛形です"),
                "使わない雛形は削除してください（雛形は規則を作りません）。",
                Some(name),
                &grammar,
            ));
        }
    }
    let mut cx = Cx {
        grammar,
        ok: &mut ok,
        issues,
    };
    // DAG なので、依存先の雛形を先に展開しておけば規則の展開は 1 段で済む。
    let mut expanded: BTreeMap<String, TemplateDecl> = BTreeMap::new();
    for name in topological(&by_name) {
        let mut template = by_name[&name].clone();
        let params: BTreeSet<&str> = template.params.iter().map(String::as_str).collect();
        template.body = body(
            &template.body,
            &BTreeMap::new(),
            &expanded,
            &params,
            &mut cx,
        );
        expanded.insert(name, template);
    }
    let empty = BTreeSet::new();
    for rule in &mut decl.rules {
        rule.body = body(&rule.body, &BTreeMap::new(), &expanded, &empty, &mut cx);
    }
    if !ok {
        for rule in &mut decl.rules {
            degrade_body(&mut rule.body);
        }
    }
}

struct Cx<'a> {
    grammar: String,
    ok: &'a mut bool,
    issues: &'a mut Vec<ValidationIssue>,
}

impl Cx<'_> {
    fn fail(&mut self, code: &str, message: String, hint: &str, name: &str) {
        *self.ok = false;
        let grammar = self.grammar.clone();
        self.issues
            .push(issue(code, message, hint, Some(name), &grammar));
    }
}

/// 展開できなかった実体化を `ERROR` 要素へ落とす。すでに E-* を積んであるので
/// コンパイルは必ず失敗する。ここでの目的は、resolve が未定義参照で先に落ちて
/// **本当の理由（E-TEMPLATE-*）を隠さない**ようにすることだけである。
fn degrade_body(target: &mut RuleBody) {
    let RuleBody::Choice { alternatives, .. } = target;
    for alternative in alternatives {
        for element in &mut alternative.elements {
            degrade_atom(&mut element.atom);
        }
    }
}

fn degrade_atom(atom: &mut Atom) {
    match atom {
        Atom::Instantiate { name, span, .. } => {
            *atom = Atom::Error {
                message: format!("未展開の雛形 {name}"),
                span: *span,
            };
        }
        Atom::Group { body, .. } | Atom::Optional { body, .. } | Atom::Repeat { body, .. } => {
            degrade_body(body)
        }
        _ => {}
    }
}

fn body_instantiates(target: &RuleBody) -> bool {
    let RuleBody::Choice { alternatives, .. } = target;
    alternatives
        .iter()
        .flat_map(|a| &a.elements)
        .any(|e| atom_instantiates(&e.atom))
}

fn atom_instantiates(atom: &Atom) -> bool {
    match atom {
        Atom::Instantiate { .. } => true,
        Atom::Group { body, .. } | Atom::Optional { body, .. } | Atom::Repeat { body, .. } => {
            body_instantiates(body)
        }
        _ => false,
    }
}

fn body_mentions(target: &RuleBody, name: &str) -> bool {
    let RuleBody::Choice { alternatives, .. } = target;
    alternatives.iter().flat_map(|a| &a.elements).any(|e| {
        atom_mentions(&e.atom, name) || e.separator.as_ref().is_some_and(|s| atom_mentions(s, name))
    })
}

fn atom_mentions(atom: &Atom, name: &str) -> bool {
    match atom {
        Atom::Ref {
            namespace: None,
            name: n,
            ..
        } => n == name,
        Atom::Instantiate { name: n, args, .. } => {
            n == name || args.iter().any(|a| atom_mentions(a, name))
        }
        Atom::Group { body, .. } | Atom::Optional { body, .. } | Atom::Repeat { body, .. } => {
            body_mentions(body, name)
        }
        _ => false,
    }
}

/// 雛形 -> その本体が実体化する雛形。
fn edges(templates: &BTreeMap<String, TemplateDecl>) -> BTreeMap<&str, BTreeSet<&str>> {
    let mut out: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (name, template) in templates {
        let mut set = BTreeSet::new();
        collect_body(&template.body, templates, &mut set);
        out.insert(name.as_str(), set);
    }
    out
}

fn collect_body<'a>(
    target: &RuleBody,
    templates: &'a BTreeMap<String, TemplateDecl>,
    out: &mut BTreeSet<&'a str>,
) {
    let RuleBody::Choice { alternatives, .. } = target;
    for alternative in alternatives {
        for element in &alternative.elements {
            collect_atom(&element.atom, templates, out);
            if let Some(separator) = &element.separator {
                collect_atom(separator, templates, out);
            }
        }
    }
}

fn collect_atom<'a>(
    atom: &Atom,
    templates: &'a BTreeMap<String, TemplateDecl>,
    out: &mut BTreeSet<&'a str>,
) {
    match atom {
        Atom::Instantiate { name, args, .. } => {
            if let Some((key, _)) = templates.get_key_value(name) {
                out.insert(key.as_str());
            }
            for arg in args {
                collect_atom(arg, templates, out);
            }
        }
        Atom::Group { body, .. } | Atom::Optional { body, .. } | Atom::Repeat { body, .. } => {
            collect_body(body, templates, out)
        }
        _ => {}
    }
}

fn cycle(templates: &BTreeMap<String, TemplateDecl>) -> Option<Vec<String>> {
    let graph = edges(templates);
    let mut state: BTreeMap<&str, u8> = BTreeMap::new();
    let mut path: Vec<&str> = Vec::new();
    for start in graph.keys() {
        if let Some(found) = visit(start, &graph, &mut state, &mut path) {
            return Some(found);
        }
    }
    None
}

fn visit<'a>(
    node: &'a str,
    graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    state: &mut BTreeMap<&'a str, u8>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    match state.get(node) {
        Some(2) => return None,
        Some(_) => {
            let at = path.iter().position(|n| *n == node).unwrap_or(0);
            let mut found: Vec<String> = path[at..].iter().map(|n| (*n).to_owned()).collect();
            found.push(node.to_owned());
            return Some(found);
        }
        None => {}
    }
    state.insert(node, 1);
    path.push(node);
    for next in graph.get(node).into_iter().flatten() {
        if let Some(found) = visit(next, graph, state, path) {
            return Some(found);
        }
    }
    path.pop();
    state.insert(node, 2);
    None
}

/// 循環が無い前提での逆トポロジカル順（依存先が先）。循環があるときも
/// 停止する（訪問済みを二度辿らない）ので、E-TEMPLATE-RECURSION と併用できる。
fn topological(templates: &BTreeMap<String, TemplateDecl>) -> Vec<String> {
    let graph = edges(templates);
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for start in graph.keys() {
        push(start, &graph, &mut seen, &mut out);
    }
    out
}

fn push<'a>(
    node: &'a str,
    graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    seen: &mut BTreeSet<&'a str>,
    out: &mut Vec<String>,
) {
    if !seen.insert(node) {
        return;
    }
    for next in graph.get(node).into_iter().flatten() {
        push(next, graph, seen, out);
    }
    out.push(node.to_owned());
}

type Env = BTreeMap<String, Atom>;

/// 展開に失敗した実体化の置き換え。E-* は既に積んであるのでコンパイルは必ず
/// 失敗する。resolve が未定義参照で先に落ちて理由を隠さないようにするだけ。
fn failed_atom(name: &str, span: Span) -> Atom {
    Atom::Error {
        message: format!("未展開の雛形 {name}"),
        span,
    }
}

fn failed_body(name: &str, span: Span) -> RuleBody {
    RuleBody::Choice {
        alternatives: vec![Sequence {
            annotations: Vec::new(),
            elements: vec![Element {
                typeof_constraint: None,
                atom: failed_atom(name, span),
                quantifier: None,
                postfix_quantifier: None,
                separator: None,
                capture: None,
                span,
            }],
            span,
        }],
        span,
    }
}

/// 量指定子・区切り・capture・`@typeof` が付いていない要素。
fn bare(element: &Element) -> bool {
    element.quantifier.is_none()
        && element.postfix_quantifier.is_none()
        && element.separator.is_none()
        && element.capture.is_none()
        && element.typeof_constraint.is_none()
}

/// 本体を展開する。`env` はパラメータ名 -> 実引数、`params` は「展開せずに
/// 残すパラメータ名」（雛形本体そのものを先に展開するときに使う）。
fn body(
    target: &RuleBody,
    env: &Env,
    templates: &BTreeMap<String, TemplateDecl>,
    params: &BTreeSet<&str>,
    cx: &mut Cx,
) -> RuleBody {
    let RuleBody::Choice { alternatives, span } = target;
    // 本体まるごとが 1 つの実体化なら、雛形の**候補列をそのまま本体にする**。
    // 括弧で括ると候補が 1 段深くなって手書きと形が変わるので、この場合だけ
    // 特別扱いする（規則本体・`( )`・`[ ]`・`{ }` の内側のいずれでも正しい）。
    if let [only] = alternatives.as_slice() {
        if let [element] = only.elements.as_slice() {
            if only.annotations.is_empty() && bare(element) {
                if let Atom::Instantiate {
                    name,
                    args,
                    span: at,
                } = &element.atom
                {
                    return match instantiate(name, args, *at, env, templates, params, cx) {
                        Some(expanded) => expanded,
                        None => failed_body(name, *at),
                    };
                }
            }
        }
    }
    RuleBody::Choice {
        alternatives: alternatives
            .iter()
            .map(|alternative| Sequence {
                annotations: alternative.annotations.clone(),
                elements: elements(&alternative.elements, env, templates, params, cx),
                span: alternative.span,
            })
            .collect(),
        span: *span,
    }
}

fn elements(
    source: &[Element],
    env: &Env,
    templates: &BTreeMap<String, TemplateDecl>,
    params: &BTreeSet<&str>,
    cx: &mut Cx,
) -> Vec<Element> {
    let mut out = Vec::with_capacity(source.len());
    for element in source {
        // 実体化が要素をそのまま占めていて、量指定子・区切り・capture・@typeof が
        // 付いておらず、雛形の本体が単一の候補なら、**要素列をその場へ差し込む**。
        // これが「手書きしたときと 1 文字も変わらない」ことの根拠である。
        if let Atom::Instantiate { name, args, span } = &element.atom {
            let Some(expanded) = instantiate(name, args, *span, env, templates, params, cx) else {
                out.push(Element {
                    atom: failed_atom(name, *span),
                    ..element.clone()
                });
                continue;
            };
            let RuleBody::Choice { alternatives, .. } = &expanded;
            if bare(element) && alternatives.len() == 1 {
                out.extend(alternatives[0].elements.iter().cloned());
                continue;
            }
            // それ以外は括弧で括った形（利用者が手で書くときと同じ）。
            out.push(Element {
                atom: Atom::Group {
                    body: expanded,
                    span: *span,
                },
                ..element.clone()
            });
            continue;
        }
        out.push(Element {
            typeof_constraint: element.typeof_constraint.clone(),
            atom: atom(&element.atom, env, templates, params, cx),
            quantifier: element.quantifier.clone(),
            postfix_quantifier: element.postfix_quantifier.clone(),
            separator: element
                .separator
                .as_ref()
                .map(|s| atom(s, env, templates, params, cx)),
            capture: element.capture.clone(),
            span: element.span,
        });
    }
    out
}

fn atom(
    source: &Atom,
    env: &Env,
    templates: &BTreeMap<String, TemplateDecl>,
    params: &BTreeSet<&str>,
    cx: &mut Cx,
) -> Atom {
    match source {
        // パラメータの参照は実引数へ置き換える。名前でしか置換しないので、
        // パラメータ名が規則 / token と重ならないこと（上で検査済み）が健全性の条件。
        Atom::Ref {
            namespace: None,
            name,
            ..
        } => env.get(name).cloned().unwrap_or_else(|| source.clone()),
        Atom::Instantiate { name, args, span } => {
            match instantiate(name, args, *span, env, templates, params, cx) {
                Some(expanded) => Atom::Group {
                    body: expanded,
                    span: *span,
                },
                None => failed_atom(name, *span),
            }
        }
        Atom::Group { body: b, span } => Atom::Group {
            body: body(b, env, templates, params, cx),
            span: *span,
        },
        Atom::Optional { body: b, span } => Atom::Optional {
            body: body(b, env, templates, params, cx),
            span: *span,
        },
        Atom::Repeat { body: b, span } => Atom::Repeat {
            body: body(b, env, templates, params, cx),
            span: *span,
        },
        _ => source.clone(),
    }
}

/// 1 つの実体化を展開する。失敗したら `None`（呼び出し側が未定義参照へ落とす）。
fn instantiate(
    name: &str,
    args: &[Atom],
    span: Span,
    env: &Env,
    templates: &BTreeMap<String, TemplateDecl>,
    params: &BTreeSet<&str>,
    cx: &mut Cx,
) -> Option<RuleBody> {
    if params.contains(name) || env.contains_key(name) {
        cx.fail(
            "E-TEMPLATE-HIGHER-ORDER",
            format!("{name}: パラメータを雛形として実体化することはできません"),
            "雛形の名前は書かれたとおりに解決されます（高階パラメータは未対応）。",
            name,
        );
        return None;
    }
    let Some(template) = templates.get(name) else {
        cx.fail(
            "E-TEMPLATE-UNKNOWN",
            format!("{name}: そのような雛形はありません"),
            "'Name<P> ::= ...' で宣言してください（規則は実体化できません）。",
            name,
        );
        return None;
    };
    if template.params.len() != args.len() {
        cx.fail(
            "E-TEMPLATE-ARITY",
            format!(
                "{name}: パラメータ {} 個に対して実引数が {} 個です",
                template.params.len(),
                args.len()
            ),
            "実引数の個数を宣言に合わせてください。",
            name,
        );
        return None;
    }
    // 実引数は**呼び出し側の環境で先に**展開する（call-by-value）。
    let inner: Env = template
        .params
        .iter()
        .cloned()
        .zip(args.iter().map(|a| atom(a, env, templates, params, cx)))
        .collect();
    // 雛形本体は逆トポロジカル順で展開済みなので、ここでの再帰は 1 段で止まる。
    let mut expanded = body(&template.body, &inner, templates, &BTreeSet::new(), cx);
    // 展開した木の範囲を**実体化の位置**へ揃える。理由は 2 つ:
    //  (a) `ExprId` は `expr:<source>:<start>:<end>:<role>` なので、同じ雛形を
    //      二箇所で実体化すると雛形本体の範囲のままでは ID が衝突する。
    //  (b) 文法作者への診断は「どの実体化か」を指すほうが役に立つ。
    respan_body(&mut expanded, span);
    Some(expanded)
}

/// 展開した木の範囲を実体化の位置へ揃える。
fn respan_body(target: &mut RuleBody, span: Span) {
    let RuleBody::Choice {
        alternatives,
        span: s,
    } = target;
    *s = span;
    for alternative in alternatives {
        alternative.span = span;
        for annotation in &mut alternative.annotations {
            let _ = annotation;
        }
        for element in &mut alternative.elements {
            element.span = span;
            respan_atom(&mut element.atom, span);
            if let Some(separator) = &mut element.separator {
                respan_atom(separator, span);
            }
        }
    }
}

fn respan_atom(atom: &mut Atom, span: Span) {
    match atom {
        Atom::Group { body, span: s }
        | Atom::Optional { body, span: s }
        | Atom::Repeat { body, span: s } => {
            *s = span;
            respan_body(body, span);
        }
        Atom::Error { span: s, .. }
        | Atom::Literal { span: s, .. }
        | Atom::Ref { span: s, .. }
        | Atom::Instantiate { span: s, .. } => *s = span,
    }
}
