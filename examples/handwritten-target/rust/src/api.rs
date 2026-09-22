//! 生成物の結果 API と独立した評価器。位置はすべて code point。
pub type Span = [usize; 2];
pub type NumberMapper = fn(&str) -> Option<f64>;

#[derive(Clone, Copy, Debug)]
pub struct ParseOptions {
    pub build_ast: bool,
    pub require_eof: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            build_ast: true,
            require_eof: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ast {
    /// Text は mapped node ではないため None。
    pub node_id: Option<usize>,
    pub span: Span,
    pub kind: AstKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstKind {
    Number {
        value: String,
        number: f64,
    },
    Text {
        value: String,
        number: f64,
    },
    Binary {
        left: Box<Ast>,
        op: Vec<String>,
        right: Vec<Ast>,
    },
    Power {
        left: Box<Ast>,
        op: Vec<String>,
        right: Vec<Ast>,
    },
}

impl Ast {
    pub fn type_name(&self) -> &'static str {
        match self.kind {
            AstKind::Number { .. } => "Number",
            AstKind::Text { .. } => "text",
            AstKind::Binary { .. } => "Binary",
            AstKind::Power { .. } => "Power",
        }
    }

    /// Java runtime test / Rust probe と同じ f64 左 fold と右再帰。
    pub fn evaluate(&self) -> f64 {
        match &self.kind {
            AstKind::Number { number, .. } | AstKind::Text { number, .. } => *number,
            AstKind::Binary { left, op, right } => {
                debug_assert_eq!(op.len(), right.len());
                op.iter()
                    .zip(right)
                    .fold(left.evaluate(), |value, (op, rhs)| {
                        let rhs = rhs.evaluate();
                        match op.as_str() {
                            "+" => value + rhs,
                            "-" => value - rhs,
                            "*" => value * rhs,
                            "/" => value / rhs,
                            _ => unreachable!("生成済み演算子集合"),
                        }
                    })
            }
            AstKind::Power { left, op, right } => {
                debug_assert_eq!(op.len(), right.len());
                debug_assert!(right.len() <= 1);
                right.first().map_or_else(
                    || left.evaluate(),
                    |rhs| left.evaluate().powf(rhs.evaluate()),
                )
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeSpan {
    pub node_id: usize,
    pub rule_id: u32,
    pub node_type: &'static str,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capture {
    pub site_id: u32,
    pub name: &'static str,
    pub span: Span,
    pub completion_order: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Syntax,
    Mapping,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub offset_cp: usize,
    pub expected: Vec<&'static str>,
    pub deepest_rule: &'static str,
    pub rule_path: Vec<&'static str>,
    pub message: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub rule_evaluations: usize,
    pub memo_hits: usize,
    pub memo_entries: usize,
    pub recipes: usize,
    pub mapped_numbers: usize,
    pub ast_nodes: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParseResult {
    /// 指定した prefix / EOF 条件での構文成功。mapping 失敗でも true。
    pub ok: bool,
    pub consumed_cp: usize,
    pub matched_cp: usize,
    pub ast: Option<Ast>,
    pub node_spans: Vec<NodeSpan>,
    pub captures: Vec<Capture>,
    /// 成功した rule 内で観測した構文失敗も残す。
    pub diagnostics: Vec<Diagnostic>,
    pub statistics: Statistics,
}

pub(crate) fn number(text: &str) -> Option<f64> {
    text.parse().ok()
}
