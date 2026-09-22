//! 自己記述文法 ubnf.ubnf に沿う再帰下降。fixture との互換拡張は各箇所に記す。

use super::ast::*;
use super::lexer::{Kind, Token};
use super::SyntaxError;

type ParseResult<T> = Result<T, SyntaxError>;

pub(super) fn parse(source: &str, tokens: Vec<Token<'_>>) -> ParseResult<UbnfFile> {
    let mut parser = Parser {
        tokens,
        cursor: 0,
        depth: 0,
    };
    let mut grammars = Vec::new();
    while !matches!(parser.peek(0).kind, Kind::Eof) {
        grammars.push(parser.grammar()?);
    }
    Ok(UbnfFile {
        grammars,
        span: Span {
            start: 0,
            end: source.len(),
        },
    })
}

struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    cursor: usize,
    depth: usize,
}

impl Parser<'_> {
    fn peek(&self, ahead: usize) -> &Token<'_> {
        &self.tokens[(self.cursor + ahead).min(self.tokens.len() - 1)]
    }

    fn is_at(&self, ahead: usize, text: &str) -> bool {
        match &self.peek(ahead).kind {
            Kind::Identifier(word) => *word == text,
            Kind::Punct(ch) => text.len() == 1 && text.as_bytes()[0] == *ch as u8,
            Kind::Define => text == "::=",
            _ => false,
        }
    }

    fn consume(&mut self, text: &str) -> bool {
        if self.is_at(0, text) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn error(&self, expected: &[&'static str]) -> SyntaxError {
        SyntaxError {
            message: format!("予期しないトークン: {:?}", self.peek(0).kind),
            span: self.peek(0).span,
            expected: expected.to_vec(),
        }
    }

    fn require(&mut self, text: &'static str) -> ParseResult<()> {
        if self.consume(text) {
            Ok(())
        } else {
            Err(self.error(&[text]))
        }
    }

    fn start(&self) -> usize {
        self.peek(0).span.start
    }

    fn span_from(&self, start: usize) -> Span {
        let end = self
            .cursor
            .checked_sub(1)
            .map_or(start, |i| self.tokens[i].span.end);
        Span {
            start,
            end: end.max(start),
        }
    }

    fn identifier(&mut self) -> ParseResult<String> {
        if let Kind::Identifier(value) = self.peek(0).kind {
            let value = value.to_owned();
            self.cursor += 1;
            Ok(value)
        } else {
            Err(self.error(&["identifier"]))
        }
    }

    fn dotted(&mut self) -> ParseResult<String> {
        let mut name = self.identifier()?;
        while self.consume(".") {
            name.push('.');
            name.push_str(&self.identifier()?);
        }
        Ok(name)
    }

    fn string(&mut self) -> ParseResult<String> {
        if let Kind::String { value, .. } = &self.peek(0).kind {
            let value = value.clone();
            self.cursor += 1;
            Ok(value)
        } else {
            Err(self.error(&["single-quoted string"]))
        }
    }

    fn integer(&mut self) -> ParseResult<u64> {
        let Kind::Digits(digits) = self.peek(0).kind else {
            return Err(self.error(&["unsigned integer"]));
        };
        let value = digits.parse().map_err(|_| SyntaxError {
            message: "整数が u64 の範囲を超えています".into(),
            span: self.peek(0).span,
            expected: vec!["unsigned integer within u64"],
        })?;
        self.cursor += 1;
        Ok(value)
    }

    fn name_list(&mut self) -> ParseResult<Vec<String>> {
        let mut names = vec![self.identifier()?];
        while self.consume(",") {
            names.push(self.identifier()?);
        }
        Ok(names)
    }

    fn named_argument(&mut self, name: &'static str) -> ParseResult<()> {
        self.require(name)?;
        self.require("=")
    }

    fn grammar(&mut self) -> ParseResult<GrammarDecl> {
        let start = self.start();
        self.require("grammar")?;
        let name = self.identifier()?;
        self.require("{")?;
        let mut imports = Vec::new();
        // @import: value はグローバル設定なので import 宣言と区別する。
        while self.is_at(0, "@")
            && self.is_at(1, "import")
            && !self.is_at(2, ":")
            && matches!(self.peek(2).kind, Kind::Identifier(_))
            && self.is_at(3, "from")
        {
            let start = self.start();
            self.require("@")?;
            self.require("import")?;
            let alias = self.identifier()?;
            self.require("from")?;
            let path = self.string()?;
            imports.push(ImportDecl {
                alias,
                path,
                span: self.span_from(start),
            });
        }
        let mut settings = Vec::new();
        while self.is_at(0, "@") && self.is_at(2, ":") {
            settings.push(self.setting()?);
        }
        let mut tokens = Vec::new();
        while self.is_at(0, "token") && matches!(self.peek(1).kind, Kind::Identifier(_)) {
            tokens.push(self.token_decl()?);
        }
        let mut rules = Vec::new();
        while !self.is_at(0, "}") {
            if matches!(self.peek(0).kind, Kind::Eof) {
                return Err(self.error(&["}"]));
            }
            rules.push(self.rule()?);
        }
        self.require("}")?;
        Ok(GrammarDecl {
            name,
            imports,
            settings,
            tokens,
            rules,
            span: self.span_from(start),
        })
    }

    fn setting(&mut self) -> ParseResult<GlobalSetting> {
        let start = self.start();
        self.require("@")?;
        let key = self.identifier()?;
        self.require(":")?;
        if self.consume("{") {
            let mut entries = Vec::new();
            while !self.consume("}") {
                let start = self.start();
                let key = self.identifier()?;
                self.require(":")?;
                // examples/tinycalc.ubnf の line: "//" を受理する最小の拡張。
                // token/annotation/rule の STRING はシングルクォートに限定する。
                let value = if let Kind::SettingString(value) = &self.peek(0).kind {
                    let value = value.clone();
                    self.cursor += 1;
                    value
                } else {
                    self.string()?
                };
                entries.push(KeyValuePair {
                    key,
                    value,
                    span: self.span_from(start),
                });
            }
            Ok(GlobalSetting::Block {
                key,
                entries,
                span: self.span_from(start),
            })
        } else {
            let value = self.dotted()?;
            Ok(GlobalSetting::String {
                key,
                value,
                span: self.span_from(start),
            })
        }
    }

    fn token_decl(&mut self) -> ParseResult<TokenDecl> {
        let start = self.start();
        self.require("token")?;
        let name = self.identifier()?;
        self.require("=")?;
        let value = self.token_value()?;
        Ok(TokenDecl {
            name,
            value,
            span: self.span_from(start),
        })
    }

    fn token_value(&mut self) -> ParseResult<TokenValue> {
        let start = self.start();
        let dotted = self.dotted()?;
        let constructor = matches!(
            dotted.as_str(),
            "UNTIL"
                | "NEGATION"
                | "LOOKAHEAD"
                | "NEGATIVE_LOOKAHEAD"
                | "CHAR_RANGE"
                | "CI"
                | "REGEX"
        );
        if constructor && self.consume("(") {
            let argument = self.string()?;
            if dotted == "CHAR_RANGE" {
                self.require(",")?;
                let max = self.string()?;
                self.require(")")?;
                return Ok(TokenValue::CharRange {
                    min: argument,
                    max,
                    span: self.span_from(start),
                });
            }
            self.require(")")?;
            let span = self.span_from(start);
            Ok(match dotted.as_str() {
                "UNTIL" => TokenValue::Until {
                    terminator: argument,
                    span,
                },
                "NEGATION" => TokenValue::Negation {
                    chars: argument,
                    span,
                },
                "LOOKAHEAD" => TokenValue::Lookahead {
                    pattern: argument,
                    span,
                },
                "NEGATIVE_LOOKAHEAD" => TokenValue::NegativeLookahead {
                    pattern: argument,
                    span,
                },
                "CI" => TokenValue::CaseInsensitive {
                    word: argument,
                    span,
                },
                "REGEX" => TokenValue::Regex {
                    pattern: argument,
                    span,
                },
                _ => unreachable!("CHAR_RANGE は上で処理済み"),
            })
        } else {
            let span = self.span_from(start);
            Ok(match dotted.as_str() {
                "ANY" => TokenValue::Any { span },
                "EOF" => TokenValue::Eof { span },
                "EMPTY" => TokenValue::Empty { span },
                _ => TokenValue::ClassRef { dotted, span },
            })
        }
    }

    fn rule(&mut self) -> ParseResult<RuleDecl> {
        let start = self.start();
        let mut annotations = Vec::new();
        while self.is_at(0, "@") {
            annotations.push(self.annotation()?);
        }
        let name = self.identifier()?;
        self.require("::=")?;
        let body = self.body()?;
        self.require(";")?;
        Ok(RuleDecl {
            annotations,
            name,
            body,
            span: self.span_from(start),
        })
    }

    fn annotation(&mut self) -> ParseResult<Annotation> {
        let start = self.start();
        self.require("@")?;
        let name = self.identifier()?;
        macro_rules! finish {
            ($variant:ident $(, $field:ident)*) => {
                Ok(Annotation::$variant { $($field,)* span: self.span_from(start) })
            };
        }
        match name.as_str() {
            "root" => finish!(Root),
            "leftAssoc" => finish!(LeftAssoc),
            "rightAssoc" => finish!(RightAssoc),
            "longestChoice" => finish!(LongestChoice),
            "predictiveChoice" => finish!(PredictiveChoice),
            "skip" => finish!(Skip),
            "enum" => finish!(Enum),
            "mapping" => {
                self.require("(")?;
                // CLASS_NAME は自己記述では識別子だが Node.Item を fixture が使う。
                let class_name = self.dotted()?;
                let params = if self.consume(",") {
                    self.named_argument("params")?;
                    self.require("[")?;
                    let params = self.name_list()?;
                    self.require("]")?;
                    params
                } else {
                    Vec::new()
                };
                self.require(")")?;
                finish!(Mapping, class_name, params)
            }
            "eval" => {
                self.require("(")?;
                let mut entries = Vec::new();
                loop {
                    let start = self.start();
                    let key = self.identifier()?;
                    // ubnf.ubnf の KeyValuePair は ':'、既存 fixture は '='。
                    if !(self.consume(":") || self.consume("=")) {
                        return Err(self.error(&[":", "="]));
                    }
                    let value = self.string()?;
                    entries.push(KeyValuePair {
                        key,
                        value,
                        span: self.span_from(start),
                    });
                    if !self.consume(",") {
                        break;
                    }
                }
                self.require(")")?;
                finish!(Eval, entries)
            }
            "whitespace" => {
                let style = if self.consume("(") {
                    let style = self.identifier()?;
                    self.require(")")?;
                    Some(style)
                } else {
                    None
                };
                finish!(Whitespace, style)
            }
            "interleave" => {
                let profile = self.identifier_argument("profile")?;
                finish!(Interleave, profile)
            }
            "backref" => {
                let name = self.identifier_argument("name")?;
                finish!(Backref, name)
            }
            "scopeTree" => {
                let mode = self.identifier_argument("mode")?;
                finish!(ScopeTree, mode)
            }
            "declares" => {
                self.require("(")?;
                self.named_argument("symbol")?;
                let symbol = self.identifier()?;
                let description = if self.consume(",") {
                    self.named_argument("description")?;
                    // 自己記述の STRING と、P4/annotations.ubnf の capture 名を受理。
                    Some(if matches!(self.peek(0).kind, Kind::String { .. }) {
                        self.string()?
                    } else {
                        self.identifier()?
                    })
                } else {
                    None
                };
                self.require(")")?;
                finish!(Declares, symbol, description)
            }
            "catalog" => {
                self.require("(")?;
                self.named_argument("context")?;
                let context = self.string()?;
                self.require(")")?;
                finish!(Catalog, context)
            }
            "precedence" => {
                self.require("(")?;
                self.named_argument("level")?;
                let level = self.integer()?;
                self.require(")")?;
                finish!(Precedence, level)
            }
            "doc" => {
                self.require("(")?;
                let text = self.string()?;
                self.require(")")?;
                finish!(Doc, text)
            }
            "recovery" => {
                self.require("(")?;
                let (mode, sync) = if self.is_at(0, "sync") && self.is_at(1, "=") {
                    self.named_argument("sync")?;
                    (None, Some(self.string()?))
                } else {
                    (Some(self.identifier()?), None)
                };
                self.require(")")?;
                finish!(Recovery, mode, sync)
            }
            "commonField" => {
                self.require("(")?;
                let fields = self.name_list()?;
                self.require(")")?;
                finish!(CommonField, fields)
            }
            _ => finish!(Simple, name),
        }
    }

    fn identifier_argument(&mut self, name: &'static str) -> ParseResult<String> {
        self.require("(")?;
        self.named_argument(name)?;
        let value = self.identifier()?;
        self.require(")")?;
        Ok(value)
    }

    fn body(&mut self) -> ParseResult<RuleBody> {
        // 異常な入れ子でもプロセスを stack overflow で落とさず位置付きエラーにする。
        if self.depth == 128 {
            return Err(SyntaxError {
                message: "ルール本体の入れ子が上限128を超えています".into(),
                span: self.peek(0).span,
                expected: vec!["shallower rule body"],
            });
        }
        self.depth += 1;
        let start = self.start();
        let mut alternatives = Vec::new();
        loop {
            let start = self.start();
            let mut elements = Vec::new();
            while matches!(
                self.peek(0).kind,
                Kind::Identifier(_) | Kind::String { .. } | Kind::Punct('(' | '[' | '{' | '@')
            ) {
                elements.push(self.element()?);
            }
            // SequenceBody ::= { AnnotatedElement } に従い epsilon も保持する。
            alternatives.push(Sequence {
                elements,
                span: self.span_from(start),
            });
            if !self.consume("|") {
                break;
            }
        }
        self.depth -= 1;
        // 末尾の空 Sequence は区切りの直前の位置。空白を挟んでも親に含める。
        let end = alternatives.last().unwrap().span.end;
        Ok(RuleBody::Choice {
            alternatives,
            span: Span { start, end },
        })
    }

    fn at_typeof(&self) -> bool {
        self.is_at(0, "@") && self.is_at(1, "typeof") && self.is_at(2, "(")
    }

    fn element(&mut self) -> ParseResult<Element> {
        let start = self.start();
        let typeof_constraint = if self.at_typeof() {
            self.require("@")?;
            self.require("typeof")?;
            self.require("(")?;
            let name = self.identifier()?;
            self.require(")")?;
            Some(name)
        } else {
            None
        };
        let atom = self.atom()?;
        let mut quantifier = match self.peek(0).kind {
            Kind::Punct(ch @ ('+' | '?' | '*')) => {
                let span = self.peek(0).span;
                self.cursor += 1;
                Some(match ch {
                    '+' => Quantifier::Plus { span },
                    '?' => Quantifier::Question { span },
                    _ => Quantifier::Star { span },
                })
            }
            _ => None,
        };
        let mut postfix_quantifier = None;
        // '{' の直後が数字でなければ次の RepeatElement。'{ }' も repeat。
        if self.is_at(0, "{") && matches!(self.peek(1).kind, Kind::Digits(_)) {
            let start = self.start();
            self.require("{")?;
            let min = self.integer()?;
            let max = if self.consume(",") {
                if self.is_at(0, "}") {
                    None
                } else {
                    Some(self.integer()?)
                }
            } else {
                Some(min)
            };
            self.require("}")?;
            postfix_quantifier = quantifier.take();
            quantifier = Some(Quantifier::Bounded {
                min,
                max,
                span: self.span_from(start),
            });
        }
        let separator = if self.consume("%") {
            Some(self.atom()?)
        } else {
            None
        };
        // 後続 @typeof(...) は次の要素の制約。単独 @typeof は capture 名。
        let capture = if !self.at_typeof() && self.consume("@") {
            Some(self.identifier()?)
        } else {
            None
        };
        Ok(Element {
            typeof_constraint,
            atom,
            quantifier,
            postfix_quantifier,
            separator,
            capture,
            span: self.span_from(start),
        })
    }

    fn atom(&mut self) -> ParseResult<Atom> {
        let start = self.start();
        if let Kind::Punct(open @ ('(' | '[' | '{')) = self.peek(0).kind {
            self.cursor += 1;
            let body = self.body()?;
            self.require(match open {
                '(' => ")",
                '[' => "]",
                _ => "}",
            })?;
            let span = self.span_from(start);
            return Ok(match open {
                '(' => Atom::Group { body, span },
                '[' => Atom::Optional { body, span },
                _ => Atom::Repeat { body, span },
            });
        }
        if let Kind::String { value, raw } = &self.peek(0).kind {
            let literal = Atom::Literal {
                value: value.clone(),
                raw: (*raw).to_owned(),
                span: self.peek(0).span,
            };
            self.cursor += 1;
            return Ok(literal);
        }
        // ERROR の完全な形だけを特別扱いし、ERROR (Item) 等は Ref + Group。
        if self.is_at(0, "ERROR")
            && self.is_at(1, "(")
            && matches!(self.peek(2).kind, Kind::String { .. })
            && self.is_at(3, ")")
        {
            self.require("ERROR")?;
            self.require("(")?;
            let message = self.string()?;
            self.require(")")?;
            return Ok(Atom::Error {
                message,
                span: self.span_from(start),
            });
        }
        if !matches!(self.peek(0).kind, Kind::Identifier(_)) {
            return Err(self.error(&["identifier", "single-quoted string", "(", "[", "{"]));
        }
        let dotted = self.dotted()?;
        // known/namespace.ubnf は a.b.Value。最後の成分以外を namespace とする。
        let (namespace, name) = if let Some((namespace, name)) = dotted.rsplit_once('.') {
            (Some(namespace.to_owned()), name.to_owned())
        } else {
            (None, dotted)
        };
        Ok(Atom::Ref {
            namespace,
            name,
            span: self.span_from(start),
        })
    }
}
