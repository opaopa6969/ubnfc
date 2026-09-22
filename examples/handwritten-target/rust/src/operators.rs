//! Operators.ubnf の静的な目標生成物。各 precedence 段をそのまま関数にする。
use crate::api::{number, NumberMapper, ParseOptions, ParseResult};
use crate::support::{repeat_more, RecipeKind, Session, Step};

const RULES: &[(u32, &str)] = &[
    (1, "Expression"),
    (2, "Term"),
    (3, "AddOp"),
    (4, "MulOp"),
    (5, "Factor"),
    (6, "NumericLiteral"),
    (7, "ExpressionRepeat"),
    (8, "TermRepeat"),
    (9, "FactorGroup"),
];

pub fn parse(input: &str) -> ParseResult {
    parse_with_options(input, ParseOptions::default())
}
pub fn parse_with_options(input: &str, options: ParseOptions) -> ParseResult {
    parse_with_number_mapper(input, options, number)
}
pub fn parse_with_number_mapper(
    input: &str,
    options: ParseOptions,
    mapper: NumberMapper,
) -> ParseResult {
    let mut parser = Parser {
        s: Session::new(input, options, 0, RULES),
    };
    let end = parser.parse_expression(0, 0);
    parser.s.finish(end, mapper)
}

struct Parser<'a> {
    s: Session<'a>,
}
impl Parser<'_> {
    fn parse_expression(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(1, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(1);
        let result = self.expression(consumed, matched);
        self.s.leave(key, result)
    }
    fn expression(&mut self, consumed: usize, matched: usize) -> Step {
        // Chain: SPACE Term@left SPACE Repeat SPACE。
        let (c, m) = self.s.trivia(consumed, matched);
        let (end, matched_end) = self.parse_term(c, m)?;
        let left = self.s.last;
        self.s.capture(1, "left", c, end);
        let (mut c, mut m) = self.s.trivia(end, matched_end);
        let mut tail = Vec::new();
        let mut count = 0;
        loop {
            let checkpoint = self.s.checkpoint();
            let Some((end, matched_end)) = self.parse_expression_tail(c, m) else {
                self.s.rollback(checkpoint);
                break;
            };
            if self.s.options.build_ast {
                tail.push(self.s.tail());
            }
            let more = repeat_more(&mut count, c, end, usize::MAX);
            (c, m) = (end, matched_end);
            if !more {
                break;
            }
        }
        let (c, m) = self.s.trivia(c, m);
        if self.s.options.build_ast {
            self.s.recipe(
                1,
                [consumed, c],
                RecipeKind::Binary {
                    left: left.expect("left"),
                    tail,
                },
            );
        }
        Some((c, m))
    }

    fn parse_term(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(2, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(2);
        let result = self.term(consumed, matched);
        self.s.leave(key, result)
    }
    fn term(&mut self, consumed: usize, matched: usize) -> Step {
        let (c, m) = self.s.trivia(consumed, matched);
        let (end, matched_end) = self.parse_factor(c, m)?;
        let left = self.s.last;
        self.s.capture(4, "left", c, end);
        let (mut c, mut m) = self.s.trivia(end, matched_end);
        let mut tail = Vec::new();
        let mut count = 0;
        loop {
            let checkpoint = self.s.checkpoint();
            let Some((end, matched_end)) = self.parse_term_tail(c, m) else {
                self.s.rollback(checkpoint);
                break;
            };
            if self.s.options.build_ast {
                tail.push(self.s.tail());
            }
            let more = repeat_more(&mut count, c, end, usize::MAX);
            (c, m) = (end, matched_end);
            if !more {
                break;
            }
        }
        let (c, m) = self.s.trivia(c, m);
        if self.s.options.build_ast {
            self.s.recipe(
                2,
                [consumed, c],
                RecipeKind::Binary {
                    left: left.expect("left"),
                    tail,
                },
            );
        }
        Some((c, m))
    }

    fn parse_add_op(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(3, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(3);
        // 一要素ずつの Choice: trivia を挿入しない。宣言順 PEG。
        let result = self
            .s
            .literal(consumed, matched, "+", 1)
            .or_else(|| self.s.literal(consumed, matched, "-", 2));
        self.s.leave(key, result)
    }
    fn parse_mul_op(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(4, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(4);
        let result = self
            .s
            .literal(consumed, matched, "*", 3)
            .or_else(|| self.s.literal(consumed, matched, "/", 4));
        self.s.leave(key, result)
    }

    fn parse_factor(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(5, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(5);
        let checkpoint = self.s.checkpoint();
        let result = self.parse_numeric_literal(consumed, matched).or_else(|| {
            self.s.rollback(checkpoint);
            self.parse_factor_group(consumed, matched)
        });
        self.s.leave(key, result)
    }

    fn parse_numeric_literal(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(6, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(6);
        let result = self.numeric_literal(consumed, matched);
        self.s.leave(key, result)
    }
    fn numeric_literal(&mut self, consumed: usize, matched: usize) -> Step {
        let (start, m) = self.s.trivia(consumed, matched);
        let (end, matched_end) = self.s.parse_number_group(start, m)?;
        self.s.capture(7, "value", start, end);
        let (c, matched_end) = self.s.trivia(end, matched_end);
        if self.s.options.build_ast {
            let token_end = self.s.number_end(start, m);
            self.s.recipe(
                6,
                [consumed, c],
                RecipeKind::Number {
                    value: [start, end],
                    token: [start, token_end],
                },
            );
        }
        Some((c, matched_end))
    }

    fn parse_expression_tail(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(7, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(7);
        let result = self.expression_tail(consumed, matched);
        self.s.leave(key, result)
    }
    fn expression_tail(&mut self, consumed: usize, matched: usize) -> Step {
        // Repeat helper: SPACE AddOp@op SPACE Term@right SPACE。
        let (start, m) = self.s.trivia(consumed, matched);
        let (end, m) = self.parse_add_op(start, m)?;
        self.s.capture(2, "op", start, end);
        let operator = [start, end];
        let (start, m) = self.s.trivia(end, m);
        let (end, m) = self.parse_term(start, m)?;
        let right = self.s.last;
        self.s.capture(3, "right", start, end);
        let (c, m) = self.s.trivia(end, m);
        if self.s.options.build_ast {
            self.s.recipe(
                7,
                [consumed, c],
                RecipeKind::Tail {
                    operator,
                    right: right.expect("right"),
                },
            );
        }
        Some((c, m))
    }
    fn parse_term_tail(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(8, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(8);
        let result = self.term_tail(consumed, matched);
        self.s.leave(key, result)
    }
    fn term_tail(&mut self, consumed: usize, matched: usize) -> Step {
        let (start, m) = self.s.trivia(consumed, matched);
        let (end, m) = self.parse_mul_op(start, m)?;
        self.s.capture(5, "op", start, end);
        let operator = [start, end];
        let (start, m) = self.s.trivia(end, m);
        let (end, m) = self.parse_factor(start, m)?;
        let right = self.s.last;
        self.s.capture(6, "right", start, end);
        let (c, m) = self.s.trivia(end, m);
        if self.s.options.build_ast {
            self.s.recipe(
                8,
                [consumed, c],
                RecipeKind::Tail {
                    operator,
                    right: right.expect("right"),
                },
            );
        }
        Some((c, m))
    }

    fn parse_factor_group(&mut self, consumed: usize, matched: usize) -> Step {
        let key = self.s.key(9, consumed, matched);
        if let Some(hit) = self.s.replay(key) {
            return hit;
        }
        self.s.enter(9);
        let result = self.factor_group(consumed, matched);
        self.s.leave(key, result)
    }
    fn factor_group(&mut self, consumed: usize, matched: usize) -> Step {
        // Choice の複数要素枝だけが chain。括弧は AST ノードを増やさない。
        let (c, m) = self.s.trivia(consumed, matched);
        let (c, m) = self.s.literal(c, m, "(", 5)?;
        let (c, m) = self.s.trivia(c, m);
        let (c, m) = self.parse_expression(c, m)?;
        let (c, m) = self.s.trivia(c, m);
        let (c, m) = self.s.literal(c, m, ")", 6)?;
        Some(self.s.trivia(c, m))
    }
}
