//! ランダムな UBNF 文法の生成。検証を通る形を構成的に作り、通らなかったものは
//! campaign 側が discard として機能別に数える（被覆が見えるようにするため）。

use crate::model::*;
use crate::rng::Rng;
use std::collections::BTreeSet;

/// 文法に投入し得る機能。discard 率の集計単位でもある。
pub const FEATURES: &[&str] = &[
    "literal",
    "token:number",
    "token:identifier",
    "token:singleQuoted",
    "token:charRange",
    "token:negation",
    "token:ci",
    "token:any",
    "token:eof",
    "token:empty",
    "token:until",
    "token:lookahead",
    "token:negativeLookahead",
    "sequence",
    "choice",
    "optional",
    "repeat",
    "group",
    "quant:plus",
    "quant:star",
    "quant:question",
    "quant:bounded",
    "separated",
    "capture",
    "capture:nested",
    "capture:parallel",
    "mapping",
    "mapping:nested",
    "mapping:shared",
    "enum",
    "commonField",
    "skip",
    "whitespace",
    "interleave",
    "comment",
    "leftAssoc",
    "rightAssoc",
    "precedence",
    "longestChoice",
    "predictiveChoice",
    "scopeTree",
    "declares",
    "backref",
    "typeof",
    "recovery",
    "error",
    "import",
    "eval",
    "catalog",
    "doc",
    "ubnf2",
    "leftRecursion",
    "wordBoundary",
    "token:neutral",
    "token:extern",
    "template",
];

/// UBNF v2（`@ubnf: 2`）を生成する seed の帯。
///
/// 既存 seed の生成物は 1 バイトも変えない（seed 269 は A-1 の回帰試験、
/// 4001..4004 は CI の固定 campaign）。したがって v2 は乱数を引かず、
/// **seed の値だけ**で決める。
pub const V2_SEED_BASE: u64 = 100_000;

/// UBNF v2 の token 綴り（D-041）。v2 の seed だけ、同じ述語を**言語中立な語彙**か
/// **明示 extern** で綴り直す。トークン数も乱数の消費も変えないので、既存 seed の
/// 生成物は 1 バイトも変わらない（v2 帯そのものは `V2_SEED_BASE` 以降）。
///
/// `extern 'SingleQuotedParser'` は scanner registry（`scanners/{java,rust}`）が
/// 両言語で提供している ID なので、**extern の表層も両 backend で突き合わせられる**。
/// seed の偶奇で切り替えるので、どちらの綴りも固定 seed campaign に現れる。
fn v2_spelling(seed: u64, declaration: &str) -> Option<(&'static str, &'static str)> {
    Some(match declaration {
        "NumberParser" => ("NUMBER", "token:neutral"),
        "org.unlaxer.parser.clang.IdentifierParser" => ("IDENTIFIER", "token:neutral"),
        "org.unlaxer.parser.elementary.SingleQuotedParser" if seed % 2 == 1 => {
            ("extern 'SingleQuotedParser'", "token:extern")
        }
        "org.unlaxer.parser.elementary.SingleQuotedParser" => ("SINGLE_QUOTED", "token:neutral"),
        _ => return None,
    })
}

/// `@enum` の選択肢。variant 名が Java / Rust の識別子になるため、
/// 記号や非 ASCII は使えない（Java 生成器の `Validation.identifier`）。
const ENUM_LITERALS: &[&str] = &["alpha", "beta", "gamma", "delta"];

const LITERALS: &[&str] = &[
    "a", "b", "c", "ab", "x", "(", ")", ",", ";", "+", "-", "*", ":", "=", "😀", "あ", "\n",
];

/// v2 だけで使うリテラル。**末尾が識別子構成文字**の語（D-040 の語境界が効く形）と、
/// 互いに接頭辞になる組（`let`/`letter`、`in`/`int`）を足して、境界が受理を変える
/// 位置を両 backend に踏ませる。v1 の seed は `LITERALS` のままなので生成物は不変。
const V2_LITERALS: &[&str] = &[
    "a", "b", "c", "ab", "x", "(", ")", ",", ";", "+", "-", "*", ":", "=", "😀", "あ", "\n", "let",
    "letter", "in", "int", "if", "x1", "_",
];

struct TokenSpec {
    declaration: &'static str,
    samples: &'static [&'static str],
    feature: &'static str,
}

const TOKEN_PALETTE: &[TokenSpec] = &[
    TokenSpec {
        declaration: "NumberParser",
        samples: &["0", "1", "2", "7", "42"],
        feature: "token:number",
    },
    TokenSpec {
        declaration: "org.unlaxer.parser.clang.IdentifierParser",
        samples: &["ab", "x1", "zz"],
        feature: "token:identifier",
    },
    TokenSpec {
        declaration: "org.unlaxer.parser.elementary.SingleQuotedParser",
        samples: &["'q'", "'😀'", "''"],
        feature: "token:singleQuoted",
    },
    TokenSpec {
        declaration: "CHAR_RANGE('a','f')",
        samples: &["a", "c", "f"],
        feature: "token:charRange",
    },
    TokenSpec {
        declaration: "NEGATION('ab\\n')",
        samples: &["z", "q", "😀"],
        feature: "token:negation",
    },
    TokenSpec {
        declaration: "CI('select')",
        samples: &["select", "SELECT", "SeLeCt"],
        feature: "token:ci",
    },
    TokenSpec {
        declaration: "ANY",
        samples: &["k", "😀", " "],
        feature: "token:any",
    },
    TokenSpec {
        declaration: "EOF",
        samples: &[],
        feature: "token:eof",
    },
    TokenSpec {
        declaration: "EMPTY",
        samples: &[],
        feature: "token:empty",
    },
    TokenSpec {
        declaration: "UNTIL('#')",
        samples: &["zz#", "#"],
        feature: "token:until",
    },
    TokenSpec {
        declaration: "LOOKAHEAD('a')",
        samples: &[],
        feature: "token:lookahead",
    },
    TokenSpec {
        declaration: "NEGATIVE_LOOKAHEAD('#')",
        samples: &[],
        feature: "token:negativeLookahead",
    },
];

struct Builder<'a> {
    rng: &'a mut Rng,
    features: BTreeSet<&'static str>,
    /// `@ubnf: 2`（D-038）。語境界つきリテラル（D-040）を引くかどうかを決める。
    v2: bool,
    /// 参照できる、より深い層の名前（ルール名または token 名）。
    deeper: Vec<String>,
    tokens: Vec<Token>,
    captures: usize,
}

impl Builder<'_> {
    fn mark(&mut self, feature: &'static str) {
        self.features.insert(feature);
    }

    fn literal(&mut self) -> Atom {
        self.mark("literal");
        let pool = if self.v2 { V2_LITERALS } else { LITERALS };
        let value = (*self.rng.pick(pool)).to_owned();
        // 末尾が識別子構成文字なら、v2 の既定でこのリテラルに語境界が付く。
        if self.v2
            && value
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.mark("wordBoundary");
        }
        Atom::Literal(value)
    }

    fn reference(&mut self) -> Atom {
        if self.deeper.is_empty() {
            return self.literal();
        }
        let index = self.rng.below(self.deeper.len());
        Atom::Ref(self.deeper[index].clone())
    }

    fn capture_name(&mut self) -> String {
        self.captures += 1;
        format!("v{}", self.captures)
    }

    /// 深さ制限つきの atom。深い側ほど終端に寄せる。
    fn atom(&mut self, depth: usize) -> Atom {
        if depth == 0 {
            return if self.rng.chance(55) {
                self.literal()
            } else {
                self.reference()
            };
        }
        match self.rng.weighted(&[40, 25, 10, 10, 10, 5]) {
            0 => self.literal(),
            1 => self.reference(),
            2 => {
                self.mark("group");
                Atom::Group(self.body(depth - 1, 2))
            }
            3 => {
                self.mark("optional");
                Atom::Optional(self.body(depth - 1, 1))
            }
            4 => {
                self.mark("repeat");
                Atom::Repeat(self.body(depth - 1, 1))
            }
            _ => {
                self.mark("error");
                Atom::Error("fuzz error".into())
            }
        }
    }

    fn quantifier(&mut self) -> Option<Quant> {
        match self.rng.weighted(&[60, 8, 8, 8, 16]) {
            0 => None,
            1 => {
                self.mark("quant:plus");
                Some(Quant::Plus)
            }
            2 => {
                self.mark("quant:question");
                Some(Quant::Question)
            }
            3 => {
                self.mark("quant:star");
                Some(Quant::Star)
            }
            _ => {
                self.mark("quant:bounded");
                let min = self.rng.below(3) as u32;
                Some(match self.rng.weighted(&[2, 1, 1]) {
                    0 => Quant::Bounded(min, Some(min + self.rng.below(3) as u32)),
                    1 => Quant::Bounded(min, Some(min)),
                    _ => Quant::Bounded(min, None),
                })
            }
        }
    }

    fn element(&mut self, depth: usize, allow_capture: bool) -> Element {
        let atom = self.atom(depth);
        let quant = self.quantifier();
        let separator = if quant.is_some() && self.rng.chance(20) {
            self.mark("separated");
            Some(self.literal())
        } else {
            None
        };
        let capture = if allow_capture && self.rng.chance(35) {
            self.mark("capture");
            Some(self.capture_name())
        } else {
            None
        };
        normalize(Element {
            lead: Vec::new(),
            typeof_capture: None,
            atom,
            quant,
            separator,
            capture,
        })
    }

    fn body(&mut self, depth: usize, max_alternatives: usize) -> Body {
        let alternatives = self.rng.between(1, max_alternatives.max(1));
        if alternatives > 1 {
            self.mark("choice");
        }
        Body {
            alternatives: (0..alternatives)
                .map(|_| {
                    let count = self.rng.between(1, 3);
                    if count > 1 {
                        self.mark("sequence");
                    }
                    (0..count).map(|_| self.element(depth, true)).collect()
                })
                .collect(),
        }
    }
}

/// 同じ seed からは必ず同じ文法を作る。
pub fn generate(seed: u64) -> Grammar {
    let mut rng = Rng::new(seed);
    // v2 は乱数を消費せずに seed だけで決める（既存 seed の生成物を保つため）。
    let v2 = seed >= V2_SEED_BASE;
    let mut builder = Builder {
        rng: &mut rng,
        features: BTreeSet::new(),
        v2,
        deeper: Vec::new(),
        tokens: Vec::new(),
        captures: 0,
    };

    let mut settings = Vec::new();
    if v2 {
        builder.mark("ubnf2");
        settings.push("@ubnf: 2".to_owned());
    }
    let whitespace = builder.rng.weighted(&[35, 45, 20]);
    match whitespace {
        0 => {}
        1 => {
            builder.mark("whitespace");
            settings.push("@whitespace: javaStyle".to_owned());
        }
        _ => {
            builder.mark("whitespace");
            settings.push("@whitespace: none".to_owned());
        }
    }
    let comments = builder.rng.chance(30);
    if comments {
        builder.mark("comment");
        settings.push("@comment: { line: \"//\" blockOpen: \"/*\" blockClose: \"*/\" }".to_owned());
    }
    if builder.rng.chance(40) {
        settings.push("@package: fuzz.generated".to_owned());
    }

    for index in 0..builder.rng.between(1, 3) {
        let spec = builder.rng.pick(TOKEN_PALETTE);
        let (declaration, samples, feature) = (spec.declaration, spec.samples, spec.feature);
        builder.mark(feature);
        // 綴りだけ v2 の語彙へ差し替える（述語・サンプル・乱数は同じ）。
        let declaration = match v2.then(|| v2_spelling(seed, declaration)).flatten() {
            Some((spelling, marker)) => {
                builder.mark(marker);
                spelling
            }
            None => declaration,
        };
        builder.tokens.push(Token {
            name: format!("T{index}"),
            declaration: declaration.to_owned(),
            samples: samples.iter().map(|s| (*s).to_owned()).collect(),
            feature,
        });
    }
    let token_names: Vec<String> = builder.tokens.iter().map(|t| t.name.clone()).collect();

    let import = if builder.rng.chance(15) {
        builder.mark("import");
        Some(Import {
            alias: "imp".into(),
            file: "imported.ubnf".into(),
            grammar: "Imported".into(),
            tokens: Vec::new(),
            rules: vec![Rule::plain(
                Vec::new(),
                "Piece".into(),
                Body {
                    alternatives: vec![
                        vec![Element::new(Atom::Literal("i".into()))],
                        vec![Element::new(Atom::Literal("j".into()))],
                    ],
                },
            )],
        })
    } else {
        None
    };

    let mut rules: Vec<Rule> = Vec::new();

    // 最深層: 終端だけの小さな規則。@enum / @skip の対象にもする。
    builder.deeper = token_names.clone();
    if import.is_some() {
        builder.deeper.push("imp.Piece".into());
    }
    for index in 0..builder.rng.between(1, 3) {
        let name = format!("L{index}");
        let enumerated = builder.rng.chance(25);
        let body = if enumerated {
            let count = builder.rng.between(2, 3);
            Body {
                alternatives: (0..count)
                    .map(|index| {
                        vec![Element::new(Atom::Literal(
                            ENUM_LITERALS[index % ENUM_LITERALS.len()].to_owned(),
                        ))]
                    })
                    .collect(),
            }
        } else {
            builder.body(1, 2)
        };
        let mut annotations = Vec::new();
        if enumerated {
            builder.mark("enum");
            annotations.push("@enum".to_owned());
        } else if builder.rng.chance(15) {
            builder.mark("skip");
            annotations.push("@skip".to_owned());
        }
        rules.push(Rule::plain(annotations, name.clone(), body));
        builder.deeper.push(name);
    }

    // 共有クラス族: Node.A / Node.B と、その dispatcher。@commonField もここで付ける。
    if builder.rng.chance(35) {
        builder.mark("mapping:shared");
        builder.mark("mapping:nested");
        let common = builder.rng.chance(50);
        for (index, variant) in ["A", "B"].iter().enumerate() {
            // 変異部は capture を持たない atom に限る。@mapping の params は
            // 本体の capture 全体と一致していなければならない。
            let tail = Element {
                quant: builder.quantifier(),
                ..Element::new(if builder.rng.chance(50) {
                    builder.literal()
                } else {
                    builder.reference()
                })
            };
            let mut elements = vec![Element::captured(
                Atom::Literal(if index == 0 { "p" } else { "q" }.into()),
                "w",
            )];
            elements.push(normalize(tail));
            rules.push(Rule::plain(
                vec![format!("@mapping(Node.{variant}, params=[w])")],
                format!("V{variant}"),
                Body::single(elements),
            ));
        }
        let mut annotations = vec!["@mapping(Node)".to_owned()];
        if common {
            builder.mark("commonField");
            annotations.push("@commonField(w)".to_owned());
        }
        rules.push(Rule::plain(
            annotations,
            "Node".into(),
            Body {
                alternatives: vec![
                    vec![Element::new(Atom::Ref("VA".into()))],
                    vec![Element::new(Atom::Ref("VB".into()))],
                ],
            },
        ));
        builder.deeper.push("Node".into());
    }

    // 中間層: capture / mapping / scope / typeof / recovery などを載せる。
    let middle = builder.rng.between(1, 3);
    for index in 0..middle {
        let name = format!("M{index}");
        let mut body = builder.body(2, 3);
        let mut annotations = Vec::new();
        let names = capture_names(&body);
        if builder.rng.chance(45) {
            builder.mark("mapping");
            if names.is_empty() {
                annotations.push(format!("@mapping(C{index})"));
            } else {
                annotations.push(format!("@mapping(C{index}, params=[{}])", names.join(", ")));
            }
        }
        let mapped = annotations.iter().any(|a| a.starts_with("@mapping("));
        if mapped && !names.is_empty() && builder.rng.chance(35) {
            builder.mark("typeof");
            let target = names[0].clone();
            let extra = builder.capture_name();
            let atom = builder.literal();
            body.alternatives[0].push(Element {
                lead: Vec::new(),
                typeof_capture: Some(target),
                atom,
                quant: None,
                separator: None,
                capture: Some(extra.clone()),
            });
            // @mapping の params は本体の capture と完全一致でなければならない。
            if let Some(mapping) = annotations.iter_mut().find(|a| a.starts_with("@mapping(")) {
                let all = capture_names(&body);
                *mapping = format!("@mapping(C{index}, params=[{}])", all.join(", "));
            }
        }
        if builder.rng.chance(20) {
            builder.mark("whitespace");
            annotations.push(if builder.rng.chance(50) {
                "@whitespace".to_owned()
            } else {
                "@whitespace(none)".to_owned()
            });
        }
        if builder.rng.chance(20) {
            // D-046: v2 で `@interleave` は非推奨なので、同じ位置に `@whitespace` を置く
            // （同一規則に `@whitespace` が既にあるときは書かない＝重複エラーを避ける）。
            // 乱数は v1 と同じ回数だけ引く。
            let comments = builder.rng.chance(50);
            if builder.v2 {
                if !annotations.iter().any(|a| a.starts_with("@whitespace")) {
                    builder.mark("whitespace");
                    annotations.push("@whitespace(javaStyle)".to_owned());
                }
            } else {
                builder.mark("interleave");
                annotations.push(if comments {
                    "@interleave(profile=javaStyle)".to_owned()
                } else {
                    "@interleave(profile=commentsAndSpaces)".to_owned()
                });
            }
        }
        if builder.rng.chance(20) {
            builder.mark("recovery");
            annotations.push(
                (*builder
                    .rng
                    .pick(&["@recovery(AUTO)", "@recovery(SKIP)", "@recovery(sync=';')"]))
                .to_owned(),
            );
        }
        if body.alternatives.len() > 1 && builder.rng.chance(25) {
            if builder.rng.chance(50) {
                builder.mark("longestChoice");
                annotations.push("@longestChoice".to_owned());
            } else if !builder.v2 {
                // D-047: v2 で `@predictiveChoice` は非推奨（front が自力で predict を選ぶ）。
                // `@longestChoice` は導出できない意味論の選択なので残す。
                builder.mark("predictiveChoice");
                annotations.push("@predictiveChoice".to_owned());
            }
        }
        if !names.is_empty() && builder.rng.chance(25) {
            builder.mark("scopeTree");
            annotations.push(
                (*builder
                    .rng
                    .pick(&["@scopeTree(mode=lexical)", "@scopeTree(mode=dynamic)"]))
                .to_owned(),
            );
            if builder.rng.chance(70) {
                builder.mark("declares");
                annotations.push(format!("@declares(symbol={})", names[0]));
            }
            if builder.rng.chance(50) {
                // D-046: v2 は意味ごとの綴りを使う。生成側は必ず `@scopeTree` と同居させて
                // いるので scope 検索側（`@symbolRef`）になる。
                builder.mark(if builder.v2 { "symbolRef" } else { "backref" });
                let spelling = if builder.v2 { "symbolRef" } else { "backref" };
                annotations.push(format!("@{spelling}(name={})", names[names.len() - 1]));
            }
        }
        if !names.is_empty() && builder.rng.chance(12) {
            builder.mark("catalog");
            annotations.push("@catalog(context='fuzz')".to_owned());
        }
        if builder.rng.chance(10) {
            builder.mark("doc");
            annotations.push("@doc('fuzz 😀')".to_owned());
        }
        rules.push(Rule::plain(annotations, name.clone(), body));
        builder.deeper.push(name);
    }

    // 演算子塔: 内側ほど強く束縛する（precedence は外側が小さい）。
    let tower = builder.rng.between(0, 2);
    let mut operand = {
        let index = builder.rng.below(builder.deeper.len());
        builder.deeper[index].clone()
    };
    // v2: 同じ塔を「候補順＝優先順位」の左再帰ルール 1 本で書く（D-039）。
    // 段ごとの規則は脱糖が作るので、両 backend は塔と同じ IR を受け取る。
    if v2 && tower > 0 {
        builder.mark("leftRecursion");
        builder.mark("precedence");
        builder.mark("leftAssoc");
        let name = "OpLr".to_owned();
        let mut alternatives = Vec::new();
        for step in 0..tower {
            let operator = Atom::Literal((*builder.rng.pick(&["+", "-", "*"])).to_owned());
            // 段ごとに別の AST クラスを使う（v1 の塔の Bin{step} と同じ形）。
            // 候補先頭の @mapping はその段だけに効く。
            let mut left = Element::captured(Atom::Ref(name.clone()), "left");
            left.lead = vec![format!("@mapping(BinLr{step}, params=[left, op, right])")];
            alternatives.push(vec![
                left,
                Element::captured(operator, "op"),
                Element::captured(Atom::Ref(name.clone()), "right"),
            ]);
        }
        alternatives.push(vec![Element::new(Atom::Ref(operand.clone()))]);
        rules.push(Rule::plain(Vec::new(), name.clone(), Body { alternatives }));
        operand = name.clone();
        builder.deeper.push(name);
    }
    for step in 0..if v2 { 0 } else { tower } {
        let name = format!("Op{step}");
        let level = 90 - 10 * step as u64;
        let right = builder.rng.chance(40);
        let operator = Atom::Literal((*builder.rng.pick(&["+", "-", "*"])).to_owned());
        let body = if right {
            builder.mark("rightAssoc");
            Body::single(vec![
                Element::captured(Atom::Ref(operand.clone()), "left"),
                Element::new(Atom::Repeat(Body::single(vec![
                    Element::captured(operator, "op"),
                    Element::captured(Atom::Ref(name.clone()), "right"),
                ]))),
            ])
        } else {
            builder.mark("leftAssoc");
            Body::single(vec![
                Element::captured(Atom::Ref(operand.clone()), "left"),
                Element::new(Atom::Repeat(Body::single(vec![
                    Element::captured(operator, "op"),
                    Element::captured(Atom::Ref(operand.clone()), "right"),
                ]))),
            ])
        };
        builder.mark("precedence");
        rules.push(Rule::plain(
            vec![
                format!("@mapping(Bin{step}, params=[left, op, right])"),
                if right { "@rightAssoc" } else { "@leftAssoc" }.to_owned(),
                format!("@precedence(level={level})"),
            ],
            name.clone(),
            body,
        ));
        operand = name.clone();
        builder.deeper.push(name);
    }

    // v2: パラメータ化ルール（D-043）。雛形は規則を作らず実体化の位置へ展開されるので、
    // **モデルは展開後の形をそのまま持ち**、原文だけ `Tpl<...>` と綴る。front の展開が
    // その綴りと一致していなければ、両 backend が同時にモデルと食い違って赤くなる。
    // 乱数は 1 つも引かないので、既存 seed の生成物は 1 バイトも変わらない。
    let mut templates: Vec<String> = Vec::new();
    if v2 {
        builder.mark("template");
        templates.push("Tpl<A, S> ::= A @head { S A @tail } ;".to_owned());
        let expansion = |separator: &str| -> Seq {
            vec![
                Element::captured(Atom::Ref(operand.clone()), "head"),
                Element::new(Atom::Repeat(Body::single(vec![
                    Element::new(Atom::Literal(separator.to_owned())),
                    Element::captured(Atom::Ref(operand.clone()), "tail"),
                ]))),
            ]
        };
        // 実体化は 2 箇所: 列の途中（括弧の中）と、候補まるごと。
        let mut first = vec![Element::new(Atom::Literal("(".into()))];
        first.extend(expansion(","));
        first.push(Element::new(Atom::Literal(")".into())));
        let name = "TplWrap".to_owned();
        let mut rule = Rule::plain(
            vec!["@mapping(TplExpr, params=[head, tail])".to_owned()],
            name.clone(),
            Body {
                alternatives: vec![first, expansion(";")],
            },
        );
        rule.template = Some(format!("'(' Tpl<{operand}, ','> ')' | Tpl<{operand}, ';'>"));
        rules.push(rule);
        operand = name.clone();
        builder.deeper.push(name);
    }

    // 根: 直下に量指定子・区切り・並行 capture を置いて、prefix と全入力の差を出す。
    let mut root_elements = vec![Element::captured(Atom::Ref(operand.clone()), "head")];
    if builder.rng.chance(60) {
        builder.mark("capture:parallel");
        let atom = builder.atom(1);
        let quant = builder.quantifier();
        root_elements.push(normalize(Element {
            lead: Vec::new(),
            typeof_capture: None,
            atom,
            quant,
            separator: None,
            capture: Some("head".into()),
        }));
    }
    if builder.rng.chance(45) {
        builder.mark("capture:nested");
        let inner = builder.element(1, false);
        root_elements.push(Element {
            lead: Vec::new(),
            typeof_capture: None,
            atom: Atom::Group(Body::single(vec![
                Element::captured(Atom::Literal(",".into()), "tail"),
                inner,
            ])),
            quant: Some(Quant::Star),
            separator: None,
            capture: Some("tail".into()),
        });
    }
    let root_captures = capture_names(&Body::single(root_elements.clone()));
    let mut root_annotations = vec!["@root".to_owned()];
    if builder.rng.chance(60) {
        builder.mark("mapping");
        root_annotations.push(format!(
            "@mapping(Top, params=[{}])",
            root_captures.join(", ")
        ));
    }
    if builder.rng.chance(12) {
        builder.mark("eval");
        root_annotations.push("@eval(kind='binary', strategy='manual')".to_owned());
    }
    rules.push(Rule::plain(
        root_annotations,
        "Root".into(),
        Body::single(root_elements),
    ));

    let features = builder.features.clone();
    let tokens = builder.tokens.clone();
    Grammar {
        name: "Fuzz".into(),
        settings,
        templates,
        tokens,
        rules,
        import,
        features,
    }
}

/// 生成器が避ける量指定子の形を潰す。
///
/// - `{n,0}`（決して一致しない反復）は front/backend の契約が定まっていないので使わない。
/// - capture 付きの `{0}` / `{1}` / `{0,1}` は front 自身が拒否する: lowering は Bounded を
///   List と宣言し、analysis は Scalar/Optional と推論して `CaptureCardinality` で落ちる
///   （`docs/reports/2026-09-23-differential-fuzzing.md` の F1）。上限を外して避ける。
fn normalize(mut element: Element) -> Element {
    element.quant = match (element.capture.is_some(), element.quant) {
        (_, Some(Quant::Bounded(min, Some(0)))) => Some(Quant::Bounded(min, None)),
        (true, Some(Quant::Bounded(min, Some(max)))) if max <= 1 => Some(Quant::Bounded(min, None)),
        (_, quant) => quant,
    };
    element
}

/// 本体に現れる capture 名を、重複を除いた出現順で返す。
pub fn capture_names(body: &Body) -> Vec<String> {
    let mut names = Vec::new();
    collect(body, &mut names);
    names
}

fn collect(body: &Body, out: &mut Vec<String>) {
    for element in body.alternatives.iter().flatten() {
        if let Some(name) = &element.capture {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        match &element.atom {
            Atom::Group(inner) | Atom::Optional(inner) | Atom::Repeat(inner) => collect(inner, out),
            _ => {}
        }
    }
}

/// 導出時に挿入してよい trivia。文法の設定に合わせて決める。
pub fn trivia_for(grammar: &Grammar) -> Vec<&'static str> {
    let text = grammar.render();
    if text.contains("@whitespace: none") && !text.contains("@interleave") {
        return Vec::new();
    }
    if !text.contains("@whitespace") && !text.contains("@interleave") {
        return Vec::new();
    }
    let mut trivia = vec![" ", "\t", "\n", "  "];
    if text.contains("@comment") || text.contains("commentsAndSpaces") {
        trivia.push("// c\n");
        trivia.push("/* c */");
    }
    trivia
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D-043: 雛形は v2 の帯にだけ現れ、v1 の帯には 1 つも現れない。
    /// 実体化は 2 箇所あり、展開後の形をモデルが持っている。
    #[test]
    fn parameterized_rules_only_appear_in_the_v2_seed_band() {
        for seed in 0..60u64 {
            let text = generate(seed).render();
            assert!(!text.contains("Tpl<"), "seed {seed} に雛形がある");
            assert!(
                !text.contains("::= A @head"),
                "seed {seed} に雛形宣言がある"
            );
        }
        for seed in V2_SEED_BASE..V2_SEED_BASE + 60 {
            let grammar = generate(seed);
            let text = grammar.render();
            assert_eq!(
                text.matches("Tpl<").count(),
                3,
                "seed {seed}: 宣言 1 + 実体化 2 のはず\n{text}"
            );
            assert!(grammar.features.contains("template"), "seed {seed}");
            // モデルは展開後の形を持つ（原文だけが雛形の綴り）。
            let wrap = grammar
                .rules
                .iter()
                .find(|r| r.name == "TplWrap")
                .expect("TplWrap");
            assert_eq!(wrap.body.alternatives.len(), 2);
            assert!(wrap.template.is_some());
        }
    }

    #[test]
    fn generation_is_deterministic_and_always_parses_as_ubnf() {
        for seed in (0..200u64).chain(V2_SEED_BASE..V2_SEED_BASE + 200) {
            let grammar = generate(seed);
            assert_eq!(grammar, generate(seed));
            let source = grammar.render();
            ubnfc_front::syntax::parse_file(&source)
                .unwrap_or_else(|e| panic!("seed {seed}: {e}\n{source}"));
            if let Some((_, imported)) = grammar.imported_source() {
                ubnfc_front::syntax::parse_file(&imported).unwrap();
            }
        }
    }

    /// D-040: v2 の seed 帯は語境界つきリテラルをほぼ必ず含み、v1 の帯は 1 つも含まない。
    #[test]
    fn word_boundary_literals_only_appear_in_the_v2_seed_band() {
        let v1 = (0..200u64).filter(|s| generate(*s).features.contains("wordBoundary"));
        assert_eq!(v1.count(), 0);
        let v2 = (V2_SEED_BASE..V2_SEED_BASE + 60)
            .filter(|s| generate(*s).features.contains("wordBoundary"))
            .count();
        assert!(v2 >= 50, "v2 帯の被覆が足りない: {v2}/60");
    }

    /// v1 の seed は host クラス名の綴りのまま（v1 は凍結、D-041）。
    #[test]
    fn v1_seeds_keep_the_host_class_spelling() {
        for seed in 0..400u64 {
            let source = generate(seed).render();
            for line in source
                .lines()
                .filter(|l| l.trim_start().starts_with("token "))
            {
                assert!(
                    !line.contains("extern ")
                        && !line.contains("= NUMBER")
                        && !line.contains("= IDENTIFIER")
                        && !line.contains("= SINGLE_QUOTED"),
                    "seed {seed}: {line}"
                );
            }
        }
    }

    /// v2 の seed は中立語彙か明示 extern で綴る。どちらも固定 seed campaign
    /// （`tests/campaign.rs` の c）で両 backend に渡る。
    #[test]
    fn v2_seeds_spell_tokens_with_the_neutral_vocabulary_or_explicit_extern() {
        let (mut neutral, mut external) = (0, 0);
        for seed in V2_SEED_BASE..V2_SEED_BASE + 400 {
            let source = generate(seed).render();
            for line in source
                .lines()
                .filter(|l| l.trim_start().starts_with("token "))
            {
                assert!(
                    !line.contains("org.unlaxer"),
                    "seed {seed} に host クラス名が残っている: {line}"
                );
                neutral += usize::from(
                    line.contains("= NUMBER")
                        || line.contains("= IDENTIFIER")
                        || line.contains("= SINGLE_QUOTED"),
                );
                external += usize::from(line.contains("extern 'SingleQuotedParser'"));
            }
        }
        assert!(neutral > 0 && external > 0, "{neutral} / {external}");
    }

    /// `tests/campaign.rs` の c が回す固定 seed 帯が、**両方の綴り**を含むこと。
    /// これが崩れると「両 backend で突き合わせた」と言えなくなる。
    #[test]
    fn the_fixed_v2_campaign_band_covers_both_spellings() {
        let features: BTreeSet<&str> = (V2_SEED_BASE + 1..V2_SEED_BASE + 4)
            .flat_map(|seed| generate(seed).features)
            .collect();
        assert!(features.contains("token:neutral"), "{features:?}");
        assert!(features.contains("token:extern"), "{features:?}");
    }

    #[test]
    fn every_declared_feature_name_is_reachable_from_some_seed() {
        let mut seen = BTreeSet::new();
        for seed in (0..800u64).chain(V2_SEED_BASE..V2_SEED_BASE + 800) {
            seen.extend(generate(seed).features);
        }
        let missing: Vec<_> = FEATURES
            .iter()
            .filter(|f| !seen.contains(*f) && **f != "token:eof")
            .copied()
            .collect();
        assert!(missing.is_empty(), "未到達の機能: {missing:?}");
    }
}
