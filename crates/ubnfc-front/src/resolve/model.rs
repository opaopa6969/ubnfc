//! 解決済み構文木。ID は resolve 呼出し内で一意、宣言・import 順で安定する。

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::syntax::ast::{Annotation, GlobalSetting, GrammarDecl, Span, TokenValue};

macro_rules! id {
    ($($name:ident),*) => {$ (
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub usize);
    )*};
}
id!(GrammarId, UnitId, SymbolId);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedGrammar {
    pub grammars: Vec<Grammar>,
    /// 元の宣言を保持する出所表。import の別名ごとの複製ではない。
    pub units: Vec<GrammarUnit>,
    pub imports: Vec<ImportEdge>,
    pub symbols: Vec<Symbol>,
    /// grammar ごとに閉じた名前空間。import は alias.Rule の形でここに統合する。
    pub symbol_table: BTreeMap<(GrammarId, String), SymbolId>,
}

impl ResolvedGrammar {
    pub fn lookup(&self, grammar: GrammarId, name: &str) -> Option<SymbolId> {
        self.symbol_table.get(&(grammar, name.to_owned())).copied()
    }

    pub fn symbol(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id.0]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarUnit {
    pub id: UnitId,
    pub path: PathBuf,
    pub declaration_index: usize,
    pub ast: GrammarDecl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEdge {
    pub grammar: GrammarId,
    pub from: UnitId,
    pub to: UnitId,
    pub alias: String,
    /// 多段 import を含めた統合先 prefix。
    pub namespace: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grammar {
    pub id: GrammarId,
    pub name: String,
    pub unit: UnitId,
    pub settings: Vec<GlobalSetting>,
    pub rules: Vec<ResolvedRule>,
    pub tokens: Vec<ResolvedToken>,
    /// ローカル root を優先。ない場合は既存生成器と同じく最初の imported root。
    pub root: Option<SymbolId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Rule,
    Token,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub grammar: GrammarId,
    pub unit: UnitId,
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRule {
    pub id: SymbolId,
    pub unit: UnitId,
    pub name: String,
    pub annotations: Vec<Annotation>,
    pub body: ResolvedBody,
    pub captures: Vec<CaptureSite>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedToken {
    pub id: SymbolId,
    pub unit: UnitId,
    pub name: String,
    /// memoSafeToken の Simple 判定などに使う、解決前の値。
    pub declaration: TokenValue,
    pub value: ResolvedTokenValue,
    pub unresolved: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedTokenValue {
    Declarative {
        expression: crate::declarative::LexicalExpression,
    },
    Builtin {
        kind: BuiltinKind,
    },
    Extern {
        class_name: String,
    },
    Until {
        terminator: String,
    },
    Negation {
        chars: String,
    },
    Lookahead {
        pattern: String,
    },
    NegativeLookahead {
        pattern: String,
    },
    CharRange {
        min: String,
        max: String,
    },
    CaseInsensitive {
        word: String,
    },
    Regex {
        pattern: String,
    },
    Any,
    Eof,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinKind {
    Number,
    Identifier,
    SingleQuoted,
    Quoted,
    EndOfSource,
    Digit,
    /// docs/design/02-facts §A の posix/ASCII 1文字集合。
    CharacterSet(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBody {
    pub alternatives: Vec<Vec<ResolvedElement>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedElement {
    pub atom: ResolvedAtom,
    pub capture: Option<String>,
    pub typeof_constraint: Option<String>,
    pub span: Span,
}

/// 量指定子を明示した木。Ref は名前文字列でなく必ず定義済み SymbolId を指す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedAtom {
    Ref {
        symbol: SymbolId,
        span: Span,
    },
    Literal {
        value: String,
        raw: String,
        span: Span,
    },
    Error {
        message: String,
        span: Span,
    },
    Group {
        body: ResolvedBody,
        span: Span,
    },
    Optional {
        body: ResolvedBody,
        /// `[ ... ]` と書かれたか（`X?` は false）。v1 では lower が
        /// 「span の先頭文字が `[` か」で判定していたが、雛形（D-043）を展開すると
        /// 範囲が実体化の位置へ移るので、構文木の形そのものを持ち回る。
        bracket: bool,
        span: Span,
    },
    Repeat {
        body: ResolvedBody,
        span: Span,
    },
    OneOrMore {
        body: Box<ResolvedAtom>,
        span: Span,
    },
    Bounded {
        body: Box<ResolvedAtom>,
        min: u64,
        max: Option<u64>,
        span: Span,
    },
    Separated {
        element: Box<ResolvedAtom>,
        separator: Box<ResolvedAtom>,
        span: Span,
    },
}

impl ResolvedAtom {
    pub fn span(&self) -> Span {
        match self {
            Self::Ref { span, .. }
            | Self::Literal { span, .. }
            | Self::Error { span, .. }
            | Self::Group { span, .. }
            | Self::Optional { span, .. }
            | Self::Repeat { span, .. }
            | Self::OneOrMore { span, .. }
            | Self::Bounded { span, .. }
            | Self::Separated { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathStep {
    Alternative(usize),
    Element(usize),
    Group,
    Optional,
    Repeat,
    OneOrMore,
    Bounded,
    SeparatedElement,
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureSite {
    /// ルール内の構文木 preorder。実行時の完了順ではない。
    pub id: usize,
    pub name: String,
    pub span: Span,
    pub path: Vec<PathStep>,
    /// capture の外側に Optional/Repeat/量指定子/Separated があるか。
    /// `X+ @x` 自体は false、`(X @x)+` の x は true。
    pub inside_quantifier: bool,
    pub quantifier_depth: usize,
    /// capture される要素そのものが量指定されているか。
    pub quantified: bool,
    pub occurrences: usize,
}
