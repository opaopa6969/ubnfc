//! Power.ubnf と corpus が指定する派生文法の静的な目標生成物。
use crate::api::{number, NumberMapper, ParseOptions, ParseResult};
use crate::support::{RecipeKind, Session, Step};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    #[default]
    Text,
    Mapped,
    Unicode,
    Grouped,
}
impl Mode {
    pub const ALL: [Self; 4] = [Self::Text, Self::Mapped, Self::Unicode, Self::Grouped];
    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Mapped => "mapped",
            Self::Unicode => "unicode",
            Self::Grouped => "grouped",
        }
    }
}

const RULES: &[(u32, &str)] = &[
    (20, "Expr"),
    (21, "Atom"),
    (22, "Base"),
    (23, "ExprRecursive"),
    (24, "BaseGroup"),
];

pub fn parse(input: &str) -> ParseResult {
    parse_in_mode(input, Mode::Text)
}
pub fn parse_in_mode(input: &str, mode: Mode) -> ParseResult {
    parse_with_options(input, mode, ParseOptions::default())
}
pub fn parse_with_options(input: &str, mode: Mode, options: ParseOptions) -> ParseResult {
    parse_with_number_mapper(input, mode, options, number)
}
pub fn parse_with_number_mapper(
    input: &str,
    mode: Mode,
    options: ParseOptions,
    mapper: NumberMapper,
) -> ParseResult {
    let mut parser = Parser {
        s: Session::new(input, options, mode as u8, RULES),
        mode,
    };
    let end = parser.parse_expr(0, 0);
    parser.s.finish(end, mapper)
}

struct Parser<'a> {
    s: Session<'a>,
    mode: Mode,
}
impl Parser<'_> {
    fn parse_expr(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(20, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(20);
        let result = self.expr(consumed, matched);
        self.s.leave(key, result)
    }
    fn expr(&mut self, consumed: usize, matched: usize) -> Step {
        // 宣言順 PEG: (SPACE Base@left SPACE '^'@op SPACE Expr@right SPACE) | Base@left。
        let checkpoint = self.s.checkpoint();
        if let Some(end) = self.parse_recursive(consumed, matched) {
            return Some(end);
        }
        self.s.rollback(checkpoint);
        // fallback に Expr 側の SPACE を足さない。Base / Atom 自身の chain に任せる。
        let (c, m) = self.parse_base(consumed, matched)?;
        let left = self.s.last;
        self.s.capture(20, "left", consumed, c);
        if self.s.options.build_ast {
            self.s.recipe(
                20,
                [consumed, c],
                RecipeKind::Power {
                    left: left.expect("left"),
                    tail: None,
                },
            );
        }
        Some((c, m))
    }

    fn parse_atom(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(21, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(21);
        let result = self.atom(consumed, matched);
        self.s.leave(key, result)
    }
    fn atom(&mut self, consumed: usize, matched: usize) -> Step {
        let (mut c, mut m) = self.s.trivia(consumed, matched);
        if self.mode == Mode::Unicode {
            // Optional の単一 literal は helper なし。optional の前後は Atom chain の境界。
            if let Some(end) = self.s.literal(c, m, "😀", 8) {
                (c, m) = end;
            }
            (c, m) = self.s.trivia(c, m);
        }
        let start = c;
        let start_matched = m;
        let (end, m) = if self.mode == Mode::Text {
            self.s.parse_number(c, m)?
        } else {
            self.s.parse_number_group(c, m)?
        };
        if self.mode != Mode::Text {
            self.s.capture(23, "value", start, end);
        }
        let (c, m) = self.s.trivia(end, m);
        if self.s.options.build_ast {
            let token = [start, self.s.number_end(start, start_matched)];
            let kind = if self.mode == Mode::Text {
                // Atom は一要素 chain。@left の text は Atom 全体を trim した値。
                RecipeKind::Text {
                    value: [consumed, c],
                    token,
                }
            } else {
                RecipeKind::Number {
                    value: [start, end],
                    token,
                }
            };
            self.s.recipe(21, [consumed, c], kind);
        }
        Some((c, m))
    }

    fn parse_base(&mut self, consumed: usize, matched: usize) -> Step {
        // Text/Mapped/Unicode では Base はソースに存在しないので Atom へ静的に委譲。
        if self.mode != Mode::Grouped {
            return self.parse_atom(consumed, matched);
        }
        let key = self.s.key(22, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(22);
        let checkpoint = self.s.checkpoint();
        let result = self.parse_atom(consumed, matched).or_else(|| {
            self.s.rollback(checkpoint);
            self.parse_base_group(consumed, matched)
        });
        self.s.leave(key, result)
    }

    fn parse_recursive(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(23, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(23);
        let result = self.recursive(consumed, matched);
        self.s.leave(key, result)
    }
    fn recursive(&mut self, consumed: usize, matched: usize) -> Step {
        let (start, m) = self.s.trivia(consumed, matched);
        let (end, m) = self.parse_base(start, m)?;
        let left = self.s.last;
        self.s.capture(20, "left", start, end);
        let (start, m) = self.s.trivia(end, m);
        let (end, m) = self.s.literal(start, m, "^", 7)?;
        let operator = [start, end];
        self.s.capture(21, "op", start, end);
        let (start, m) = self.s.trivia(end, m);
        let (end, m) = self.parse_expr(start, m)?;
        let right = self.s.last;
        self.s.capture(22, "right", start, end);
        let (c, m) = self.s.trivia(end, m);
        if self.s.options.build_ast {
            self.s.recipe(
                20,
                [consumed, c],
                RecipeKind::Power {
                    left: left.expect("left"),
                    tail: Some((operator, right.expect("right"))),
                },
            );
        }
        Some((c, m))
    }

    fn parse_base_group(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(24, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(24);
        let result = self.base_group(consumed, matched);
        self.s.leave(key, result)
    }
    fn base_group(&mut self, consumed: usize, matched: usize) -> Step {
        let (c, m) = self.s.trivia(consumed, matched);
        let (c, m) = self.s.literal(c, m, "(", 5)?;
        let (c, m) = self.s.trivia(c, m);
        let (c, m) = self.parse_expr(c, m)?;
        let (c, m) = self.s.trivia(c, m);
        let (c, m) = self.s.literal(c, m, ")", 6)?;
        Some(self.s.trivia(c, m))
    }
}
