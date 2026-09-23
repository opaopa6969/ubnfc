//! 生成 UBNF パーサの採用結果から構文木へのフロント専用変換 (D-012)。
//!
//! 型付き AST の値と、未 capture の suffix を含む rule occurrence を使う。
//! ソース全体を字句・構文解析し直す経路は持たない。

use std::collections::BTreeMap;

use crate::syntax::ast::*;

/// 生成バックエンドに依存しない、型付き AST の借用投影。
/// ノードの構造は occurrence が保持し、ここには mapping 済みの文字列を置く。
#[derive(Debug)]
pub struct MappedNode<'a> {
    pub kind: &'a str,
    pub span_cp: [usize; 2],
    pub fields: Vec<(&'a str, Vec<&'a str>)>,
}

#[derive(Debug)]
pub struct Capture<'a> {
    pub rule_id: &'a str,
    pub name: &'a str,
    pub span_cp: [usize; 2],
    pub completion_order: usize,
}

#[derive(Debug)]
pub struct Occurrence<'a> {
    pub id: usize,
    pub parent: Option<usize>,
    pub rule_id: &'a str,
    pub expr_id: &'a str,
    pub span_cp: [usize; 2],
    pub completion_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (bytes {}..{})",
            self.message, self.span.start, self.span.end
        )
    }
}
impl std::error::Error for ConversionError {}

type Result<T> = std::result::Result<T, ConversionError>;

/// 範囲外や逆順の span を丸めず拒否する。終端には source.len() を含む。
pub struct SourceOffsets<'a> {
    source: &'a str,
    bytes: Vec<usize>,
}
impl<'a> SourceOffsets<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source
                .char_indices()
                .map(|(i, _)| i)
                .chain([source.len()])
                .collect(),
        }
    }
    pub fn byte_span(&self, cp: [usize; 2]) -> Result<Span> {
        if cp[0] > cp[1] || cp[1] >= self.bytes.len() {
            return Err(ConversionError {
                message: format!("不正な code point span: {cp:?}"),
                span: Span {
                    start: self.source.len(),
                    end: self.source.len(),
                },
            });
        }
        Ok(Span {
            start: self.bytes[cp[0]],
            end: self.bytes[cp[1]],
        })
    }
    pub fn text(&self, cp: [usize; 2]) -> Result<&'a str> {
        let span = self.byte_span(cp)?;
        Ok(&self.source[span.start..span.end])
    }

    /// 採用済み span の境界 trivia だけを除く。内部の構文の採否は決定しない。
    /// 引用文字列の空白・// は残し、javaStyle の6文字と行コメントだけを除く。
    fn content_span(&self, cp: [usize; 2]) -> Result<Span> {
        let raw = self.byte_span(cp)?;
        let mut start = None;
        let mut end = raw.start;
        let mut chars = self.source[raw.start..raw.end].char_indices().peekable();
        let mut quoted = false;
        let mut escaped = false;
        while let Some((at, ch)) = chars.next() {
            if !quoted {
                if is_space(ch) {
                    continue;
                }
                if ch == '/' && chars.peek().is_some_and(|(_, ch)| *ch == '/') {
                    chars.next();
                    for (_, ch) in chars.by_ref() {
                        if matches!(ch, '\r' | '\n') {
                            break;
                        }
                    }
                    continue;
                }
            }
            start.get_or_insert(raw.start + at);
            end = raw.start + at + ch.len_utf8();
            if escaped {
                escaped = false;
            } else if quoted && ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                quoted = !quoted;
            }
        }
        // epsilon は次の構文トークンの位置。rule 開始時の trivia 消費後の位置を使う。
        Ok(Span {
            start: start.unwrap_or(raw.end),
            end: if start.is_some() { end } else { raw.end },
        })
    }
}
fn is_space(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n' | '\u{b}' | '\u{c}')
}

/// STRING capture の引用符を除去し、syntax lexer と同じ未知 escape 保持を行う。
pub fn decode_string(raw: &str, span: Span) -> Result<String> {
    let inner = raw
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .ok_or_else(|| ConversionError {
            message: "STRING capture に引用符がありません".into(),
            span,
        })?;
    let mut result = String::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }
        let next = chars.next().ok_or_else(|| ConversionError {
            message: "未完の escape".into(),
            span,
        })?;
        match next {
            'n' => result.push('\n'),
            't' => result.push('\t'),
            'r' => result.push('\r'),
            '\\' | '\'' => result.push(next),
            _ => {
                result.push('\\');
                result.push(next);
            }
        }
    }
    Ok(result)
}

/// typed AST、capture、採用 rule occurrence から UbnfFile を構築する。
/// occurrence の親子関係が選択枝を表し、span 内文字列から枝を再推測しない。
pub fn convert(
    source: &str,
    nodes: &[MappedNode<'_>],
    captures: &[Capture<'_>],
    occurrences: &[Occurrence<'_>],
) -> Result<UbnfFile> {
    Converter::new(source, nodes, captures, occurrences)?.file()
}

struct Converter<'a> {
    offsets: SourceOffsets<'a>,
    nodes: &'a [MappedNode<'a>],
    captures: &'a [Capture<'a>],
    occurrences: &'a [Occurrence<'a>],
    children: Vec<Vec<usize>>,
    owned_captures: Vec<Vec<usize>>,
    mapped: BTreeMap<(&'a str, [usize; 2]), usize>,
}
impl<'a> Converter<'a> {
    fn new(
        source: &'a str,
        nodes: &'a [MappedNode<'a>],
        captures: &'a [Capture<'a>],
        occurrences: &'a [Occurrence<'a>],
    ) -> Result<Self> {
        let offsets = SourceOffsets::new(source);
        let invalid = |message: &str| ConversionError {
            message: message.into(),
            span: Span {
                start: 0,
                end: source.len(),
            },
        };
        let mut ids = BTreeMap::new();
        for (i, o) in occurrences.iter().enumerate() {
            offsets.byte_span(o.span_cp)?;
            if ids.insert(o.id, i).is_some() {
                return Err(invalid("occurrence ID が重複しています"));
            }
            if o.expr_id.is_empty() {
                return Err(invalid("採用 expression ID がありません"));
            }
        }
        let mut children = vec![vec![]; occurrences.len()];
        let mut depth = vec![0; occurrences.len()];
        for (i, o) in occurrences.iter().enumerate() {
            if let Some(parent) = o.parent {
                let p = *ids
                    .get(&parent)
                    .ok_or_else(|| invalid("親 occurrence がありません"))?;
                let parent = &occurrences[p];
                if p >= i
                    || parent.span_cp[0] > o.span_cp[0]
                    || parent.span_cp[1] < o.span_cp[1]
                    || parent.completion_order <= o.completion_order
                {
                    return Err(invalid("occurrence の親子関係・範囲・完了順が不正です"));
                }
                children[p].push(i);
                depth[i] = depth[p] + 1;
            }
        }
        let mut owned_captures = vec![vec![]; occurrences.len()];
        for (i, capture) in captures.iter().enumerate() {
            offsets.byte_span(capture.span_cp)?;
            let owner = occurrences
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.rule_id == capture.rule_id
                        && o.span_cp[0] <= capture.span_cp[0]
                        && capture.span_cp[1] <= o.span_cp[1]
                })
                .max_by_key(|(j, _)| depth[*j])
                .map(|(j, _)| j)
                .ok_or_else(|| invalid("capture の所有 occurrence がありません"))?;
            owned_captures[owner].push(i);
        }
        for indices in &mut owned_captures {
            indices.sort_by_key(|&i| captures[i].completion_order);
        }
        let mut mapped = BTreeMap::new();
        for (i, node) in nodes.iter().enumerate() {
            offsets.byte_span(node.span_cp)?;
            if mapped.insert((node.kind, node.span_cp), i).is_some() {
                return Err(invalid("mapping ノードの対応が曖昧です"));
            }
        }
        Ok(Self {
            offsets,
            nodes,
            captures,
            occurrences,
            children,
            owned_captures,
            mapped,
        })
    }
    fn kind(&self, i: usize) -> &str {
        self.occurrences[i].rule_id.rsplit("::").next().unwrap()
    }
    fn span(&self, i: usize) -> Result<Span> {
        self.offsets.content_span(self.occurrences[i].span_cp)
    }
    fn error(&self, i: usize, message: impl Into<String>) -> ConversionError {
        ConversionError {
            message: message.into(),
            span: self.span(i).unwrap_or_default(),
        }
    }
    fn children(&self, i: usize, kind: &str) -> Vec<usize> {
        self.children[i]
            .iter()
            .copied()
            .filter(|&j| self.kind(j) == kind)
            .collect()
    }
    fn child(&self, i: usize, kind: &str) -> Result<usize> {
        let found = self.children(i, kind);
        if found.len() != 1 {
            return Err(self.error(i, format!("{} に {kind} が一つ必要です", self.kind(i))));
        }
        Ok(found[0])
    }
    fn branch(&self, i: usize) -> Result<usize> {
        if self.children[i].len() != 1 {
            return Err(self.error(i, "採用枝が一つ必要です"));
        }
        Ok(self.children[i][0])
    }
    fn values(&self, i: usize, field: &str) -> Result<Vec<&str>> {
        if let Some(&n) = self
            .mapped
            .get(&(self.kind(i), self.occurrences[i].span_cp))
        {
            if let Some((_, values)) = self.nodes[n].fields.iter().find(|(name, _)| *name == field)
            {
                return Ok(values.clone());
            }
        }
        self.owned_captures[i]
            .iter()
            .map(|&c| &self.captures[c])
            .filter(|c| c.name == field)
            .map(|c| self.offsets.text(c.span_cp))
            .collect()
    }
    /// STRING capture の原文 (引用符付き)。型付き AST の値は引用符を除いた
    /// 内容なので、raw 保持と escape 復号は capture span の原文から行う。
    fn raw_values(&self, i: usize, field: &str) -> Result<Vec<&str>> {
        self.owned_captures[i]
            .iter()
            .map(|&c| &self.captures[c])
            .filter(|c| c.name == field)
            .map(|c| {
                let span = self.offsets.content_span(c.span_cp)?;
                Ok(&self.offsets.source[span.start..span.end])
            })
            .collect()
    }
    fn scalar(&self, i: usize, field: &str, values: Vec<&str>) -> Result<String> {
        if values.len() != 1 {
            return Err(self.error(
                i,
                format!("{}.{field} に scalar capture が必要です", self.kind(i)),
            ));
        }
        Ok(values[0].to_owned())
    }
    fn at_most_one(&self, i: usize, field: &str, values: Vec<&str>) -> Result<Option<String>> {
        match values.as_slice() {
            [] => Ok(None),
            [value] => Ok(Some((*value).to_owned())),
            _ => Err(self.error(i, format!("{field} の optional capture が重複しています"))),
        }
    }
    fn value(&self, i: usize, field: &str) -> Result<String> {
        self.scalar(i, field, self.values(i, field)?)
    }
    fn optional(&self, i: usize, field: &str) -> Result<Option<String>> {
        self.at_most_one(i, field, self.values(i, field)?)
    }
    fn strings(&self, i: usize, field: &str) -> Result<Vec<String>> {
        Ok(self
            .values(i, field)?
            .into_iter()
            .map(str::to_owned)
            .collect())
    }
    fn raw_string(&self, i: usize, field: &str) -> Result<String> {
        self.scalar(i, field, self.raw_values(i, field)?)
    }
    fn string(&self, i: usize, field: &str) -> Result<String> {
        decode_string(&self.raw_string(i, field)?, self.span(i)?)
    }
    fn optional_string(&self, i: usize, field: &str) -> Result<Option<String>> {
        self.at_most_one(i, field, self.raw_values(i, field)?)?
            .map(|v| decode_string(&v, self.span(i)?))
            .transpose()
    }
    fn file(&self) -> Result<UbnfFile> {
        let roots: Vec<_> = self
            .occurrences
            .iter()
            .enumerate()
            .filter(|(_, o)| o.parent.is_none())
            .map(|(i, _)| i)
            .collect();
        if roots.len() != 1 || self.kind(roots[0]) != "UBNFFile" {
            return Err(ConversionError {
                message: "UBNFFile の採用 root が一つ必要です".into(),
                span: Span::default(),
            });
        }
        let root = roots[0];
        if !self
            .mapped
            .contains_key(&("UBNFFile", self.occurrences[root].span_cp))
        {
            return Err(self.error(root, "生成結果に型付き UBNFFile がありません"));
        }
        Ok(UbnfFile {
            grammars: self
                .children(root, "GrammarDecl")
                .into_iter()
                .map(|i| self.grammar(i))
                .collect::<Result<_>>()?,
            span: Span {
                start: 0,
                end: self.offsets.source.len(),
            },
        })
    }
    fn grammar(&self, i: usize) -> Result<GrammarDecl> {
        Ok(GrammarDecl {
            name: self.value(i, "name")?,
            span: self.span(i)?,
            imports: self
                .children(i, "ImportDecl")
                .into_iter()
                .map(|j| {
                    Ok(ImportDecl {
                        alias: self.value(j, "alias")?,
                        path: self.string(j, "path")?,
                        span: self.span(j)?,
                    })
                })
                .collect::<Result<_>>()?,
            settings: self
                .children(i, "GlobalSetting")
                .into_iter()
                .map(|j| self.setting(j))
                .collect::<Result<_>>()?,
            tokens: self
                .children(i, "TokenDecl")
                .into_iter()
                .map(|j| {
                    Ok(TokenDecl {
                        name: self.value(j, "name")?,
                        value: self.token(self.branch(self.child(j, "TokenValue")?)?)?,
                        span: self.span(j)?,
                    })
                })
                .collect::<Result<_>>()?,
            rules: self
                .children(i, "RuleDecl")
                .into_iter()
                .map(|j| self.rule(j))
                .collect::<Result<_>>()?,
        })
    }
    fn dotted(&self, i: usize) -> Result<String> {
        Ok(self.values(i, "parts")?.join("."))
    }
    fn setting(&self, i: usize) -> Result<GlobalSetting> {
        let key = self.value(i, "key")?;
        let span = self.span(i)?;
        let value = self.branch(self.child(i, "SettingValue")?)?;
        match self.kind(value) {
            "StringSettingValue" => Ok(GlobalSetting::String {
                key,
                value: self.dotted(self.child(value, "DottedIdentifier")?)?,
                span,
            }),
            "BlockSettingValue" => Ok(GlobalSetting::Block {
                key,
                entries: self.entries(value)?,
                span,
            }),
            kind => Err(self.error(value, format!("未知の設定値: {kind}"))),
        }
    }
    fn entries(&self, i: usize) -> Result<Vec<KeyValuePair>> {
        self.children(i, "KeyValuePair")
            .into_iter()
            .map(|j| {
                Ok(KeyValuePair {
                    key: self.value(j, "key")?,
                    value: self.string(j, "value")?,
                    span: self.span(j)?,
                })
            })
            .collect()
    }
    fn token(&self, i: usize) -> Result<TokenValue> {
        let span = self.span(i)?;
        Ok(match self.kind(i) {
            "UntilExpression" => TokenValue::Until {
                terminator: self.string(i, "terminator")?,
                span,
            },
            "NegationExpression" => TokenValue::Negation {
                chars: self.string(i, "chars")?,
                span,
            },
            "LookaheadExpression" => TokenValue::Lookahead {
                pattern: self.string(i, "pattern")?,
                span,
            },
            "NegativeLookaheadExpression" => TokenValue::NegativeLookahead {
                pattern: self.string(i, "pattern")?,
                span,
            },
            "CharRangeExpression" => TokenValue::CharRange {
                min: self.string(i, "min")?,
                max: self.string(i, "max")?,
                span,
            },
            "CIExpression" => TokenValue::CaseInsensitive {
                word: self.string(i, "word")?,
                span,
            },
            "RegexExpression" => TokenValue::Regex {
                pattern: self.string(i, "pattern")?,
                span,
            },
            "AnyExpression" => TokenValue::Any { span },
            "EofExpression" => TokenValue::Eof { span },
            "EmptyExpression" => TokenValue::Empty { span },
            "DottedIdentifier" => TokenValue::ClassRef {
                dotted: self.dotted(i)?,
                span,
            },
            kind => return Err(self.error(i, format!("未知の token 値: {kind}"))),
        })
    }
    fn rule(&self, i: usize) -> Result<RuleDecl> {
        Ok(RuleDecl {
            name: self.value(i, "name")?,
            annotations: self
                .children(i, "Annotation")
                .into_iter()
                .map(|j| self.annotation(self.branch(j)?))
                .collect::<Result<_>>()?,
            body: self.body(self.child(i, "RuleBody")?)?,
            span: self.span(i)?,
        })
    }
    fn annotation(&self, i: usize) -> Result<Annotation> {
        let span = self.span(i)?;
        Ok(match self.kind(i) {
            "RootAnnotation" => Annotation::Root { span },
            "MappingAnnotation" => Annotation::Mapping {
                class_name: self.value(i, "className")?,
                params: self.strings(i, "paramNames")?,
                span,
            },
            "EvalAnnotation" => Annotation::Eval {
                entries: self.entries(i)?,
                span,
            },
            "WhitespaceAnnotation" => Annotation::Whitespace {
                style: self.optional(i, "style")?,
                span,
            },
            "InterleaveAnnotation" => Annotation::Interleave {
                profile: self.value(i, "profile")?,
                span,
            },
            "BackrefAnnotation" => Annotation::Backref {
                name: self.value(i, "name")?,
                span,
            },
            "ScopeTreeAnnotation" => Annotation::ScopeTree {
                mode: self.value(i, "mode")?,
                span,
            },
            "DeclaresAnnotation" => Annotation::Declares {
                symbol: self.value(i, "symbol")?,
                description: self.optional_string(i, "description")?,
                span,
            },
            "CatalogAnnotation" => Annotation::Catalog {
                context: self.string(i, "context")?,
                span,
            },
            "LeftAssocAnnotation" => Annotation::LeftAssoc { span },
            "RightAssocAnnotation" => Annotation::RightAssoc { span },
            "LongestChoiceAnnotation" => Annotation::LongestChoice { span },
            "PredictiveChoiceAnnotation" => Annotation::PredictiveChoice { span },
            "PrecedenceAnnotation" => Annotation::Precedence {
                level: self
                    .value(i, "level")?
                    .parse()
                    .map_err(|_| self.error(i, "整数が u64 の範囲外です"))?,
                span,
            },
            "DocAnnotation" => Annotation::Doc {
                text: self.string(i, "text")?,
                span,
            },
            "RecoveryAnnotation" => Annotation::Recovery {
                mode: self.optional(i, "mode")?,
                sync: self.optional_string(i, "syncTokens")?,
                span,
            },
            "SkipAnnotation" => Annotation::Skip { span },
            "CommonFieldAnnotation" => Annotation::CommonField {
                fields: self.strings(i, "fieldNames")?,
                span,
            },
            "EnumAnnotation" => Annotation::Enum { span },
            "SimpleAnnotation" => Annotation::Simple {
                name: self.value(i, "name")?,
                span,
            },
            kind => return Err(self.error(i, format!("未知の annotation: {kind}"))),
        })
    }
    fn body(&self, i: usize) -> Result<RuleBody> {
        let choice = self.child(i, "ChoiceBody")?;
        let alternatives = self
            .children(choice, "SequenceBody")
            .into_iter()
            .map(|j| {
                let elements = self
                    .children(j, "AnnotatedElement")
                    .into_iter()
                    .map(|k| self.element(k))
                    .collect::<Result<Vec<_>>>()?;
                let span = if let (Some(first), Some(last)) = (elements.first(), elements.last()) {
                    Span {
                        start: first.span.start,
                        end: last.span.end,
                    }
                } else {
                    self.span(j)?
                };
                Ok(Sequence {
                    annotations: Vec::new(),
                    elements,
                    span,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let first = alternatives
            .first()
            .ok_or_else(|| self.error(i, "ChoiceBody に SequenceBody がありません"))?;
        let span = Span {
            start: first.span.start,
            end: alternatives.last().unwrap().span.end,
        };
        Ok(RuleBody::Choice { alternatives, span })
    }
    fn element(&self, i: usize) -> Result<Element> {
        let atom = self.atom(self.child(i, "AtomicElement")?)?;
        let mut quantifier = self
            .children(i, "PostfixQuantifier")
            .into_iter()
            .next()
            .map(|j| {
                let span = self.span(j)?;
                match &self.offsets.source[span.start..span.end] {
                    "+" => Ok(Quantifier::Plus { span }),
                    "?" => Ok(Quantifier::Question { span }),
                    "*" => Ok(Quantifier::Star { span }),
                    _ => Err(self.error(j, "未知の postfix")),
                }
            })
            .transpose()?;
        let mut postfix_quantifier = None;
        if let Some(j) = self.children(i, "BoundedQuantifier").into_iter().next() {
            let span = self.span(j)?;
            let digits = self.children(j, "Digits");
            let mut numbers = Vec::new();
            for k in digits {
                let span = self.span(k)?;
                numbers.push(
                    self.offsets.source[span.start..span.end]
                        .parse::<u64>()
                        .map_err(|_| {
                            self.error(k, "整数が u64 の範囲外、または連続した数字ではありません")
                        })?,
                );
            }
            let min = *numbers
                .first()
                .ok_or_else(|| self.error(j, "bounded の下限がありません"))?;
            let max = if numbers.len() == 2 {
                Some(numbers[1])
            } else if self.offsets.source[span.start..span.end].contains(',') {
                None
            } else {
                Some(min)
            };
            postfix_quantifier = quantifier.take();
            quantifier = Some(Quantifier::Bounded { min, max, span });
        }
        let separator = self
            .children(i, "SeparatedBy")
            .into_iter()
            .next()
            .map(|j| self.atom(self.child(j, "AtomicElement")?))
            .transpose()?;
        let typeof_constraint = self
            .children(i, "TypeofElement")
            .into_iter()
            .next()
            .map(|j| self.value(j, "captureName"))
            .transpose()?;
        Ok(Element {
            typeof_constraint,
            atom,
            quantifier,
            postfix_quantifier,
            separator,
            capture: self.optional(i, "captureName")?,
            span: self.span(i)?,
        })
    }
    fn atom(&self, i: usize) -> Result<Atom> {
        let j = self.branch(i)?;
        let span = self.span(j)?;
        Ok(match self.kind(j) {
            "GroupElement" => Atom::Group {
                body: self.body(self.child(j, "RuleBody")?)?,
                span,
            },
            "OptionalElement" => Atom::Optional {
                body: self.body(self.child(j, "RuleBody")?)?,
                span,
            },
            "RepeatElement" => Atom::Repeat {
                body: self.body(self.child(j, "RuleBody")?)?,
                span,
            },
            "ErrorElement" => Atom::Error {
                message: self.string(j, "message")?,
                span,
            },
            "TerminalElement" => Atom::Literal {
                value: self.string(j, "value")?,
                raw: self.raw_string(j, "value")?,
                span,
            },
            "QuantifiedRef" => Atom::Ref {
                namespace: self.optional(j, "namespace")?,
                name: self.value(j, "name")?,
                span,
            },
            kind => return Err(self.error(j, format!("未知の atom: {kind}"))),
        })
    }
}
