//! 名前解決・意味解析前の構文木。すべての範囲は元ソースの byte offset。

use serde::Serialize;

/// 半開区間 `[start, end)`。空の構文は `start == end`。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UbnfFile {
    pub grammars: Vec<GrammarDecl>,
    pub span: Span,
}

/// 設計文書でいう GrammarAst はファイル全体の構文木。
pub type GrammarAst = UbnfFile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GrammarDecl {
    pub name: String,
    pub imports: Vec<ImportDecl>,
    pub settings: Vec<GlobalSetting>,
    pub tokens: Vec<TokenDecl>,
    pub rules: Vec<RuleDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportDecl {
    pub alias: String,
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum GlobalSetting {
    String {
        key: String,
        value: String,
        span: Span,
    },
    Block {
        key: String,
        entries: Vec<KeyValuePair>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TokenDecl {
    pub name: String,
    pub value: TokenValue,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TokenValue {
    Until {
        terminator: String,
        span: Span,
    },
    Negation {
        chars: String,
        span: Span,
    },
    Lookahead {
        pattern: String,
        span: Span,
    },
    NegativeLookahead {
        pattern: String,
        span: Span,
    },
    /// 境界の文字数・順序の検証は意味解析の責務。
    CharRange {
        min: String,
        max: String,
        span: Span,
    },
    CaseInsensitive {
        word: String,
        span: Span,
    },
    Regex {
        pattern: String,
        span: Span,
    },
    Any {
        span: Span,
    },
    Eof {
        span: Span,
    },
    Empty {
        span: Span,
    },
    /// v1 の綴り: host 言語のクラス名（FQN または既知パッケージの単純名）。
    ClassRef {
        dotted: String,
        span: Span,
    },
    /// 言語中立な組込み語彙（`token NUMBER = NUMBER`）。名前は
    /// [`crate::vocabulary::BUILTIN_VOCABULARY`] の中立名（D-041）。
    Builtin {
        name: String,
        span: Span,
    },
    /// host 実装の明示宣言（`token CODE = extern 'CodeStartParser'`）。
    /// 名前は scanner registry が解決する ID で、FQN でなくてよい（D-041）。
    Extern {
        name: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleDecl {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub body: RuleBody,
    pub span: Span,
}

/// `@backref` を意味ごとに割った v2 の綴り（D-044）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum BackrefMode {
    /// `@symbolRef(name=X)`: scope store の検索。未定義は WARNING。
    Symbol,
    /// `@sameText(name=X)`: 同一 rule 内のテキスト一致。不一致は ERROR。
    SameText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Annotation {
    Root {
        span: Span,
    },
    Mapping {
        class_name: String,
        params: Vec<String>,
        span: Span,
    },
    Eval {
        entries: Vec<KeyValuePair>,
        span: Span,
    },
    Whitespace {
        style: Option<String>,
        span: Span,
    },
    Interleave {
        profile: String,
        span: Span,
    },
    Backref {
        name: String,
        /// v2 の明示綴り。`None` は旧 `@backref`（文法中に `@scopeTree` があるかで意味が決まる）。
        mode: Option<BackrefMode>,
        span: Span,
    },
    ScopeTree {
        mode: String,
        span: Span,
    },
    Declares {
        symbol: String,
        description: Option<String>,
        span: Span,
    },
    Catalog {
        context: String,
        span: Span,
    },
    LeftAssoc {
        span: Span,
    },
    RightAssoc {
        span: Span,
    },
    LongestChoice {
        span: Span,
    },
    PredictiveChoice {
        span: Span,
    },
    Precedence {
        level: u64,
        span: Span,
    },
    Doc {
        text: String,
        span: Span,
    },
    /// 構文上は mode と sync のどちらか一方だけが Some。
    Recovery {
        mode: Option<String>,
        sync: Option<String>,
        span: Span,
    },
    Skip {
        span: Span,
    },
    CommonField {
        fields: Vec<String>,
        span: Span,
    },
    Enum {
        span: Span,
    },
    Simple {
        name: String,
        span: Span,
    },
}

/// Choice と Sequence 自体にも span を付けるため名前付きフィールドを使う。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RuleBody {
    Choice {
        alternatives: Vec<Sequence>,
        span: Span,
    },
}

impl RuleBody {
    pub fn alternatives(&self) -> &[Sequence] {
        let Self::Choice { alternatives, .. } = self;
        alternatives
    }

    pub fn span(&self) -> Span {
        let Self::Choice { span, .. } = self;
        *span
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Sequence {
    /// UBNF v2 の候補先頭注釈（`@leftAssoc` / `@rightAssoc`）。v1 では常に空。
    /// 候補の結合性は左再帰ルールの脱糖だけが読む（`desugar`）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<Annotation>,
    pub elements: Vec<Element>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Element {
    pub typeof_constraint: Option<String>,
    pub atom: Atom,
    pub quantifier: Option<Quantifier>,
    /// 自己記述文法は `A+{2,3}` も許す。両方ある場合、quantifier に
    /// 外側の Bounded、このフィールドに先行する Plus/Question/Star を保持する。
    /// 量化子が一つだけなら quantifier に格納し、ここは None。
    pub postfix_quantifier: Option<Quantifier>,
    pub separator: Option<Atom>,
    pub capture: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Quantifier {
    Plus {
        span: Span,
    },
    Question {
        span: Span,
    },
    Star {
        span: Span,
    },
    /// `{n}` は max=Some(n)、`{n,}` は max=None。
    Bounded {
        min: u64,
        max: Option<u64>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Atom {
    Group {
        body: RuleBody,
        span: Span,
    },
    Optional {
        body: RuleBody,
        span: Span,
    },
    Repeat {
        body: RuleBody,
        span: Span,
    },
    Error {
        message: String,
        span: Span,
    },
    /// raw は引用符を含むソースそのもの、value はエスケープ解決済み。
    Literal {
        value: String,
        raw: String,
        span: Span,
    },
    Ref {
        namespace: Option<String>,
        name: String,
        span: Span,
    },
}

// enum の利用側でも共通の範囲取得を行えるようにする。
macro_rules! impl_span {
    ($name:ident: $($variant:ident),+ $(,)?) => {
        impl $name {
            pub fn span(&self) -> Span {
                match self {
                    $(Self::$variant { span, .. })|+ => *span,
                }
            }
        }
    };
}

impl_span!(GlobalSetting: String, Block);
impl_span!(TokenValue: Until, Negation, Lookahead, NegativeLookahead, CharRange,
    CaseInsensitive, Regex, Any, Eof, Empty, ClassRef, Builtin, Extern);
impl_span!(Annotation: Root, Mapping, Eval, Whitespace, Interleave, Backref, ScopeTree,
    Declares, Catalog, LeftAssoc, RightAssoc, LongestChoice, PredictiveChoice,
    Precedence, Doc, Recovery, Skip, CommonField, Enum, Simple);
impl_span!(Quantifier: Plus, Question, Star, Bounded);
impl_span!(Atom: Group, Optional, Repeat, Error, Literal, Ref);
