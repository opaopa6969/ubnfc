//! D-014 の独立 Java 実測と同じ表示候補。解析の Rust fail_at とは分離する。
//! ParseContext.java:1097,1111-1148: 子を二段まで探索し、空なら表示名を使う。
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use ubnfc_front::ir::GrammarIr;

#[derive(Default)]
struct Node {
    name: String,
    terminal: Vec<String>,
    children: Vec<usize>,
}
pub(crate) struct Plan {
    exprs: BTreeMap<String, Value>,
    tokens: BTreeMap<String, Value>,
    names: BTreeMap<String, String>,
    rules: BTreeMap<String, usize>,
    nodes: BTreeMap<String, usize>,
    bases: BTreeMap<String, usize>,
    helpers: BTreeMap<String, String>,
    bodies: BTreeSet<String>,
    assigned: BTreeSet<String>,
    displays: BTreeMap<String, String>,
    recovery: BTreeSet<String>,
    arena: Vec<Node>,
}
fn string(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().into()
}
fn children(n: &Value) -> Vec<String> {
    ["items", "alternatives"]
        .iter()
        .flat_map(|k| n[k].as_array().into_iter().flatten())
        .chain(
            ["child", "element", "separator", "terminator"]
                .iter()
                .filter_map(|k| n.get(k)),
        )
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
impl Plan {
    pub fn new(ir: &GrammarIr) -> Self {
        let v = serde_json::to_value(ir).unwrap();
        let s = &v["semantics"];
        let index = |key: &str, id: &str| {
            s[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| (string(v, id), v.clone()))
                .collect()
        };
        let mut p = Self {
            exprs: index("expressions", "id"),
            tokens: index("tokens", "symbolId"),
            names: s["symbols"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| (string(v, "id"), string(v, "name")))
                .collect(),
            rules: BTreeMap::new(),
            nodes: BTreeMap::new(),
            bases: BTreeMap::new(),
            helpers: BTreeMap::new(),
            bodies: BTreeSet::new(),
            assigned: BTreeSet::new(),
            displays: s["diagnostics"]["expectedDisplay"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| (string(v, "exprId"), string(v, "rust")))
                .collect(),
            recovery: s["recovery"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|r| r["applicationSites"].as_array().unwrap())
                .map(|v| v.as_str().unwrap().into())
                .collect(),
            arena: vec![],
        };
        // IR の Rust literal label は字句そのもの。公開候補は引用符付きの
        // 表示文字列にし、明示的に別名を指定した rust label は保持する。
        for (id, display) in &mut p.displays {
            let n = &p.exprs[id]["node"];
            if n["kind"] == "literal" && n["value"].as_str() == Some(display.as_str()) {
                *display = format!("'{display}'");
            }
        }
        for r in s["rules"].as_array().unwrap() {
            let id = string(r, "symbolId");
            let body = string(r, "body");
            let n = &p.exprs[&body]["node"];
            let (name, term) = if body.contains("/lower/operator/") && n["kind"] == "literal" {
                (
                    "WordParser".into(),
                    vec![format!("'{}'", string(n, "value"))],
                )
            } else {
                (format!("{}Parser", p.names[&id]), vec![])
            };
            let node = p.add(name, term, vec![]);
            p.rules.insert(id, node);
            p.bodies.insert(body.clone());
            p.assign(&body, &mut [0; 4]);
        }
        for r in s["rules"].as_array().unwrap() {
            let id = string(r, "symbolId");
            let children = p.child_nodes(&string(r, "body"));
            let i = p.rules[&id];
            p.arena[i].children = children;
        }
        for id in p.exprs.keys().cloned().collect::<Vec<_>>() {
            p.node(&id);
        }
        p
    }
    fn add(&mut self, name: String, terminal: Vec<String>, children: Vec<usize>) -> usize {
        let i = self.arena.len();
        self.arena.push(Node {
            name,
            terminal,
            children,
        });
        i
    }
    fn plain(&mut self, name: &str) -> usize {
        self.add(name.into(), vec![], vec![])
    }
    fn wrap(&mut self, name: &str, child: usize) -> usize {
        self.add(name.into(), vec![], vec![child])
    }
    fn terminal(&mut self, name: &str, text: String) -> usize {
        self.add(name.into(), vec![text], vec![])
    }
    fn assign(&mut self, id: &str, c: &mut [usize; 4]) {
        if !self.assigned.insert(id.into()) {
            return;
        }
        let e = self.exprs[id].clone();
        let n = &e["node"];
        let name = &self.names[&string(&e, "ruleId")];
        if id.ends_with("/lower/separated/0") {
            let h = format!("{name}Sep{}", c[3]);
            c[3] += 1;
            self.helpers.insert(id.into(), format!("{h}Parser"));
            self.helpers.insert(
                format!("{}2", &id[..id.len() - 1]),
                format!("{h}BodyParser"),
            );
        }
        let child = string(n, "child");
        if n["kind"] == "group" {
            self.helpers
                .insert(child.clone(), format!("{name}Group{}Parser", c[0]));
            c[0] += 1;
        }
        if n["kind"] == "repeat"
            && !matches!(
                self.exprs[&child]["node"]["kind"].as_str(),
                Some("ruleRef" | "tokenRef")
            )
            && !id.contains("/lower/separated/")
        {
            let kind = match n["min"].as_u64().unwrap() {
                0 => "Repeat",
                1 => "OneOrMore",
                _ => "Bounded",
            };
            self.helpers
                .insert(child.clone(), format!("{name}{kind}{}Parser", c[1]));
            c[1] += 1;
        }
        if n["kind"] == "optional" && child.contains("/lower/helper/") {
            self.helpers
                .insert(child, format!("{name}Opt{}Parser", c[2]));
            c[2] += 1;
        }
        if n["kind"] == "separated" {
            self.helpers
                .insert(id.into(), format!("{name}Sep{}Parser", c[3]));
            c[3] += 1;
        }
        for child in children(n) {
            self.assign(&child, c);
        }
    }
    fn child_nodes(&mut self, id: &str) -> Vec<usize> {
        let n = self.exprs[id]["node"].clone();
        if matches!(n["kind"].as_str(), Some("seq" | "choice")) {
            children(&n).iter().map(|c| self.node(c)).collect()
        } else {
            vec![self.node(id)]
        }
    }
    fn node(&mut self, id: &str) -> usize {
        if let Some(&i) = self.nodes.get(id) {
            return i;
        }
        let e = self.exprs[id].clone();
        let n = &e["node"];
        let mut base = match n["kind"].as_str().unwrap() {
            "literal" => self.terminal("WordParser", format!("'{}'", string(n, "value"))),
            "ruleRef" => self.rules[&string(n, "symbolId")],
            "tokenRef" => self.token(&string(n, "symbolId")),
            "group" => {
                let child = string(n, "child");
                let kids = self.child_nodes(&child);
                self.add(self.helpers[&child].clone(), vec![], kids)
            }
            "seq" | "choice" => {
                let kids = self.child_nodes(id);
                self.add(
                    self.helpers.get(id).cloned().unwrap_or_default(),
                    vec![],
                    kids,
                )
            }
            "repeat" | "optional" => {
                let c = self.node(&string(n, "child"));
                self.wrap(
                    if n["kind"] == "optional" {
                        "Optional"
                    } else {
                        "Repeat"
                    },
                    c,
                )
            }
            "separated" => {
                let element = self.node(&string(n, "element"));
                let sep = self.node(&string(n, "separator"));
                let name = self.helpers[id].clone();
                let pair = self.add(
                    name.replace("Parser", "BodyParser"),
                    vec![],
                    vec![sep, element],
                );
                let tail = self.wrap("ZeroOrMore", pair);
                self.add(name, vec![], vec![element, tail])
            }
            "eof" | "empty" | "any" => self.special(n["kind"].as_str().unwrap()),
            "lookahead" | "negativeLookahead" => {
                let c = self.node(&string(n, "child"));
                self.wrap("MatchOnly", c)
            }
            "error" => self.terminal("ErrorMessageParser", string(n, "message")),
            kind => self.plain(kind),
        };
        if let Some(display) = self.displays.get(id).cloned() {
            base = self.terminal(&self.arena[base].name.clone(), display);
        }
        if self.recovery.contains(id) {
            base = self.wrap("RecoveryParser", base);
        }
        self.bases.insert(id.into(), base);
        let result = if e["captureSiteIds"].as_array().unwrap().is_empty() {
            base
        } else {
            self.wrap("__CaptureSite", base)
        };
        self.nodes.insert(id.into(), result);
        result
    }
    fn special(&mut self, kind: &str) -> usize {
        let any = self.plain("WildCardCharacterParser");
        match kind {
            "eof" => self.wrap("EndOfSourceParser", any),
            "empty" => {
                let m = self.wrap("MatchOnly", any);
                self.wrap("EmptyParser", m)
            }
            "until" => self.wrap("WildCardStringTerminatorParser", any),
            _ => any,
        }
    }
    fn token(&mut self, id: &str) -> usize {
        let s = self.tokens[id]["spec"].clone();
        match s["kind"].as_str().unwrap() {
            "caseInsensitive" => {
                self.terminal("IgnoreCaseWordParser", format!("'{}'", string(&s, "word")))
            }
            "regex" => self.terminal(
                "RegexTokenParser",
                format!("REGEX('{}')", string(&s, "pattern")),
            ),
            "lookahead" | "negativeLookahead" => {
                let word = self.terminal("WordParser", format!("'{}'", string(&s, "pattern")));
                self.wrap("MatchOnly", word)
            }
            "until" | "eof" | "empty" | "any" => self.special(s["kind"].as_str().unwrap()),
            "builtin" => match s["which"].as_str().unwrap() {
                "Number" => {
                    let sign = self.plain("SignParser");
                    let sign = self.wrap("Optional", sign);
                    let digit = self.plain("DigitParser");
                    let digits = self.wrap("OneOrMore", digit);
                    let point = self.terminal("PointParser", "'.'".into());
                    let chain = self.add("Chain".into(), vec![], vec![digits, point]);
                    let choice = self.wrap("Choice", chain);
                    let exponent = self.plain("ExponentParser");
                    let exponent = self.wrap("Optional", exponent);
                    self.add("NumberParser".into(), vec![], vec![sign, choice, exponent])
                }
                "Identifier" => {
                    let head = self.plain("AlphabetUnderScoreParser");
                    let tail = self.plain("AlphabetNumericUnderScoreParser");
                    let tail = self.wrap("ZeroOrMore", tail);
                    self.add("IdentifierParser".into(), vec![], vec![head, tail])
                }
                "EndOfSource" => self.plain("EndOfSourceParser"),
                _ => self.plain(&format!("{}Parser", self.names[id])),
            },
            "extern" => self.plain(
                s["originalClassName"]
                    .as_str()
                    .unwrap()
                    .rsplit('.')
                    .next()
                    .unwrap(),
            ),
            _ => self.plain(&format!("{}Parser", self.names[id])),
        }
    }
    fn collect(&self, i: usize, depth: usize, out: &mut Vec<String>) {
        let n = &self.arena[i];
        for t in &n.terminal {
            if !out.contains(t) {
                out.push(t.clone());
            }
        }
        if depth > 0 && n.terminal.is_empty() {
            for &child in &n.children {
                self.collect(child, depth - 1, out);
            }
        }
    }
    fn hints(&self, i: usize) -> Vec<String> {
        let mut hints = vec![];
        self.collect(i, 2, &mut hints);
        if hints.is_empty() && !self.arena[i].name.is_empty() {
            hints.push(self.arena[i].name.clone());
        }
        hints
    }
    pub fn rule(&self, id: &str) -> Vec<String> {
        self.hints(self.rules[id])
    }
    pub fn expression(&self, id: &str) -> Vec<String> {
        let e = &self.exprs[id];
        let n = &e["node"];
        if n["kind"] == "tokenRef"
            && self.tokens[&string(n, "symbolId")]["spec"]["kind"] == "extern"
        {
            return vec![];
        }
        let mut out = vec![];
        if !self.bodies.contains(id) && n["kind"] != "ruleRef" {
            out.extend(self.hints(self.bases[id]));
        }
        if !e["captureSiteIds"].as_array().unwrap().is_empty() {
            out.extend(self.hints(self.nodes[id]));
        }
        out.sort();
        out.dedup();
        out
    }
    pub fn token_label(&self, id: &str) -> String {
        self.hints(self.bases[id])
            .into_iter()
            .next()
            .unwrap_or_else(|| "token".into())
    }
}
