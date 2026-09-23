//! UBNF 4 実装比較のプローブ（ubnfc 側 2 実装ぶん）。
//!
//! 引数の各 `.ubnf` を ubnfc の手書き frontend（`ubnfc_front::syntax`）と
//! 自己ホスト生成 frontend（`ubnfc_selfhost`）の両方へ通し、1 入力 1 行の JSON を出す。
//! `--stdin-cases` を付けると TSV（`id<TAB>source`、`\n` `\t` は 2 文字表記）を標準入力から読む。
//!
//! `ast` は `unlaxer-parser/rust/unlaxer-ubnf/tests/support/canonical.rs` と
//! **同じ綴り**へ射影したもの。ubnfc の構文木は量指定子を要素の属性として持ち、
//! unlaxer 系は要素種別として持つので、次の正規化だけを行う（差を消す目的の正規化は他に無い）。
//!
//! - `X?` -> OptionalElement(body\[\[X\]\])、`X*` -> RepeatElement(body\[\[X\]\])
//! - `X+` -> OneOrMoreElement(X)、`X{n,m}` -> BoundedRepeatElement(X,n,m)
//! - `X % sep` -> SeparatedElement(X, sep)（量指定子の外側）
//! - 量指定子が 2 つある `X+{n,m}` は postfix を内側、bounded を外側に包む
//!
//! ubnfc にしか無い構文（v2 の Builtin/Extern token、雛形、`@symbolRef` 等）は
//! 専用の綴りで出し、対応物が無いことが差分として見えるようにする。

use ubnfc_front::syntax;
use ubnfc_front::syntax::ast::*;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("--stdin-cases") {
        let mut text = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut text).unwrap();
        for line in text.lines() {
            let (id, source) = line.split_once('\t').unwrap_or((line, ""));
            let source = source.replace("\\t", "\t").replace("\\n", "\n");
            println!("{}", report(id, &source));
        }
        return;
    }
    for path in arguments {
        let source = std::fs::read_to_string(&path).unwrap();
        println!("{}", report(&path, &source));
    }
}

fn report(id: &str, source: &str) -> String {
    let handwritten = match syntax::parse_file(source) {
        Ok(file) => format!("{{\"ok\":true,\"ast\":{}}}", file_json(&file)),
        Err(error) => format!(
            "{{\"ok\":false,\"offset\":{},\"message\":{}}}",
            error.span.start,
            quote(&error.to_string())
        ),
    };
    let selfhost = match ubnfc_selfhost::parse_file(source) {
        Ok(file) => format!("{{\"ok\":true,\"ast\":{}}}", file_json(&file)),
        Err(error) => format!(
            "{{\"ok\":false,\"offset\":{},\"message\":{}}}",
            error.span().map_or(-1, |span| span.start as i64),
            quote(&error.to_string())
        ),
    };
    format!(
        "{{\"input\":{},\"handwritten\":{handwritten},\"selfhost\":{selfhost}}}",
        quote(id)
    )
}

fn quote(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn list<I: IntoIterator<Item = String>>(values: I) -> String {
    format!("[{}]", values.into_iter().collect::<Vec<_>>().join(","))
}
fn tag<I: IntoIterator<Item = String>>(name: &str, values: I) -> String {
    list(std::iter::once(quote(name)).chain(values))
}
fn optional(value: Option<&String>) -> String {
    value.map_or_else(|| "null".to_owned(), |s| quote(s))
}
fn strings(values: &[String]) -> String {
    list(values.iter().map(|s| quote(s)))
}

fn file_json(file: &UbnfFile) -> String {
    tag("UBNFFile", [list(file.grammars.iter().map(grammar))])
}

fn grammar(g: &GrammarDecl) -> String {
    let mut fields = vec![
        quote(&g.name),
        list(
            g.imports
                .iter()
                .map(|i| tag("ImportDecl", [quote(&i.alias), quote(&i.path)])),
        ),
        list(g.settings.iter().map(setting)),
        list(g.tokens.iter().map(token)),
        list(g.rules.iter().map(rule)),
    ];
    // 雛形は ubnfc 固有（D-043）。空でなければ差分として見えるように追加する。
    if !g.templates.is_empty() {
        fields.push(list(g.templates.iter().map(|t| {
            tag(
                "TemplateDecl",
                [quote(&t.name), strings(&t.params), body(&t.body)],
            )
        })));
    }
    tag("GrammarDecl", fields)
}

fn setting(s: &GlobalSetting) -> String {
    match s {
        GlobalSetting::String { key, value, .. } => tag(
            "GlobalSetting",
            [quote(key), tag("StringSettingValue", [quote(value)])],
        ),
        GlobalSetting::Block { key, entries, .. } => tag(
            "GlobalSetting",
            [
                quote(key),
                tag(
                    "BlockSettingValue",
                    [list(entries.iter().map(|e| {
                        tag("KeyValuePair", [quote(&e.key), quote(&e.value)])
                    }))],
                ),
            ],
        ),
    }
}

fn token(t: &TokenDecl) -> String {
    let (name, args) = match &t.value {
        TokenValue::ClassRef { dotted, .. } => ("Simple", vec![quote(dotted)]),
        TokenValue::Until { terminator, .. } => ("Until", vec![quote(terminator)]),
        TokenValue::Negation { chars, .. } => ("Negation", vec![quote(chars)]),
        TokenValue::Lookahead { pattern, .. } => ("Lookahead", vec![quote(pattern)]),
        TokenValue::NegativeLookahead { pattern, .. } => {
            ("NegativeLookahead", vec![quote(pattern)])
        }
        TokenValue::Any { .. } => ("Any", vec![]),
        TokenValue::Eof { .. } => ("Eof", vec![]),
        TokenValue::Empty { .. } => ("Empty", vec![]),
        TokenValue::CharRange { min, max, .. } => ("CharRange", vec![quote(min), quote(max)]),
        TokenValue::CaseInsensitive { word, .. } => ("CaseInsensitive", vec![quote(word)]),
        TokenValue::Regex { pattern, .. } => ("Regex", vec![quote(pattern)]),
        // v2 のみ（D-041）。unlaxer 系に対応物が無いので専用の綴りにする。
        TokenValue::Builtin { name, .. } => ("BuiltinVocabulary", vec![quote(name)]),
        TokenValue::Extern { name, .. } => ("ExternScanner", vec![quote(name)]),
    };
    tag(name, std::iter::once(quote(&t.name)).chain(args))
}

fn rule(r: &RuleDecl) -> String {
    tag(
        "RuleDecl",
        [
            list(r.annotations.iter().map(annotation)),
            quote(&r.name),
            body(&r.body),
        ],
    )
}

fn annotation(a: &Annotation) -> String {
    let (name, args) = match a {
        Annotation::Root { .. } => ("RootAnnotation", vec![]),
        Annotation::Mapping {
            class_name, params, ..
        } => (
            "MappingAnnotation",
            vec![quote(class_name), strings(params)],
        ),
        // ubnfc は kind/strategy を持たず key=value の並びで持つ。構造差をそのまま残す。
        Annotation::Eval { entries, .. } => (
            "EvalAnnotation",
            vec![list(entries.iter().map(|e| {
                tag("KeyValuePair", [quote(&e.key), quote(&e.value)])
            }))],
        ),
        Annotation::Whitespace { style, .. } => {
            ("WhitespaceAnnotation", vec![optional(style.as_ref())])
        }
        Annotation::Interleave { profile, .. } => ("InterleaveAnnotation", vec![quote(profile)]),
        Annotation::Backref { name, mode, .. } => match mode {
            None => ("BackrefAnnotation", vec![quote(name)]),
            Some(BackrefMode::Symbol) => ("SymbolRefAnnotation", vec![quote(name)]),
            Some(BackrefMode::SameText) => ("SameTextAnnotation", vec![quote(name)]),
        },
        Annotation::ScopeTree { mode, .. } => ("ScopeTreeAnnotation", vec![quote(mode)]),
        Annotation::Declares {
            symbol,
            description,
            ..
        } => (
            "DeclaresAnnotation",
            vec![quote(symbol), optional(description.as_ref())],
        ),
        Annotation::Catalog { context, .. } => ("CatalogAnnotation", vec![quote(context)]),
        Annotation::LeftAssoc { .. } => ("LeftAssocAnnotation", vec![]),
        Annotation::RightAssoc { .. } => ("RightAssocAnnotation", vec![]),
        Annotation::LongestChoice { .. } => ("LongestChoiceAnnotation", vec![]),
        Annotation::PredictiveChoice { .. } => ("PredictiveChoiceAnnotation", vec![]),
        Annotation::Precedence { level, .. } => ("PrecedenceAnnotation", vec![level.to_string()]),
        Annotation::Doc { text, .. } => ("DocAnnotation", vec![quote(text)]),
        Annotation::Recovery { mode, sync, .. } => {
            // unlaxer 側は (RecoveryMode, syncTokens[]) の 2 引数。
            let (mode, tokens) = match (mode.as_deref(), sync.as_deref()) {
                (Some(mode), _) => (mode.to_uppercase(), Vec::new()),
                (None, Some(token)) => ("SYNC".to_owned(), vec![token.to_owned()]),
                (None, None) => ("SYNC".to_owned(), Vec::new()),
            };
            ("RecoveryAnnotation", vec![quote(&mode), strings(&tokens)])
        }
        Annotation::Skip { .. } => ("SkipAnnotation", vec![]),
        Annotation::CommonField { fields, .. } => ("CommonFieldAnnotation", vec![strings(fields)]),
        Annotation::Enum { .. } => ("EnumAnnotation", vec![]),
        Annotation::Simple { name, .. } => ("SimpleAnnotation", vec![quote(name)]),
    };
    tag(name, args)
}

fn body(b: &RuleBody) -> String {
    tag(
        "body",
        [list(
            b.alternatives()
                .iter()
                .map(|s| list(s.elements.iter().map(element))),
        )],
    )
}

/// 量指定子の中身を unlaxer 綴りの AnnotatedElement で包む（capture/typeof は付かない）。
fn bare(atom: String) -> String {
    tag(
        "AnnotatedElement",
        [atom, "null".to_owned(), "null".to_owned()],
    )
}

fn wrap(inner: String, quantifier: &Quantifier) -> String {
    match quantifier {
        Quantifier::Plus { .. } => tag("OneOrMoreElement", [inner]),
        Quantifier::Question { .. } => tag(
            "OptionalElement",
            [tag("body", [list([list([bare(inner)])])])],
        ),
        Quantifier::Star { .. } => tag(
            "RepeatElement",
            [tag("body", [list([list([bare(inner)])])])],
        ),
        Quantifier::Bounded { min, max, .. } => tag(
            "BoundedRepeatElement",
            [
                inner,
                min.to_string(),
                max.unwrap_or(i32::MAX as u64).to_string(),
            ],
        ),
    }
}

fn element(e: &Element) -> String {
    let mut value = atom(&e.atom);
    if let Some(quantifier) = &e.postfix_quantifier {
        value = wrap(value, quantifier);
    }
    if let Some(quantifier) = &e.quantifier {
        value = wrap(value, quantifier);
    }
    if let Some(separator) = &e.separator {
        value = tag("SeparatedElement", [value, atom(separator)]);
    }
    tag(
        "AnnotatedElement",
        [
            value,
            optional(e.capture.as_ref()),
            e.typeof_constraint
                .as_ref()
                .map_or_else(|| "null".to_owned(), |s| tag("TypeofElement", [quote(s)])),
        ],
    )
}

fn atom(a: &Atom) -> String {
    match a {
        Atom::Group { body: b, .. } => tag("GroupElement", [body(b)]),
        Atom::Optional { body: b, .. } => tag("OptionalElement", [body(b)]),
        Atom::Repeat { body: b, .. } => tag("RepeatElement", [body(b)]),
        Atom::Error { message, .. } => tag("ErrorElement", [quote(message)]),
        Atom::Literal { value, .. } => tag("TerminalElement", [quote(value)]),
        Atom::Ref {
            namespace, name, ..
        } => tag(
            "RuleRefElement",
            [optional(namespace.as_ref()), quote(name)],
        ),
        // v2 のみ（D-043）。unlaxer 系に対応物が無い。
        Atom::Instantiate { name, args, .. } => tag(
            "InstantiateElement",
            [quote(name), list(args.iter().map(atom))],
        ),
    }
}
