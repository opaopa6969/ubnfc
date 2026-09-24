// D-077: 採用結果の AST を arena（SoA）で持つ形。所有 `Ast` は常にこの木から写す。
//
// - 節点は `nodes: Vec<TreeNode>`（24 byte）に作った順で並ぶ。子は `slots: Vec<u32>` の添字で指す。
// - 文字列は持たない。Text 節点は入力の byte 範囲 `[a, b)` で、値は常に入力の部分文字列
//   （semantic text は trim 済み範囲か token の content 範囲で、どちらも入力の中にある）。
// - record の field は宣言順に slot を持つ: scalar / optional は 1 slot（空は `NONE`）、
//   list は 2 slot（`slots` 内の開始位置と個数。要素は record より先に `slots` へ積む）。
// - `#[doc(hidden)] pub` の構築用メソッドは生成 runtime 専用（driver が runtime を別 crate へ
//   `#[path]` で取り込むため crate 内可視にできない）。利用者は呼ばない。
// - 作ったが採用されなかった節点（値の包み直しで捨てた Text 等）も `nodes` に残る。
//   根から辿れないだけで観測には出ない（所有 `Ast` と同じ）。
use super::super::api::{NodeSpan, Span, ValueSpan};
/// 空の slot（optional の無値）と、根の無い木。
pub const NONE: u32 = u32::MAX;
pub const KIND_TEXT: u16 = 0;
pub const KIND_NULL: u16 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TreeNode {
    kind: u16,
    /// 節点を作った規則（`RULE_IDS` の添字）。Text / Null は 0。
    rule: u16,
    /// 型付き節点の作成順（`NodeSpan::node_id`）。Text / Null は 0。
    node_id: u32,
    /// code point の範囲。Text は値の出自（extent）。
    span: [u32; 2],
    /// Text: byte 範囲。record: 最初の slot。enum: variant の番号。
    a: u32,
    b: u32,
}
/// 1 回の parse の AST を arena に持つ木（`ParseOptions::ast_tree`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstTree {
    source: String,
    nodes: Vec<TreeNode>,
    slots: Vec<u32>,
    typed: u32,
    root: u32,
}
impl Default for AstTree {
    fn default() -> Self {
        Self {
            source: String::new(),
            nodes: Vec::new(),
            slots: Vec::new(),
            typed: 0,
            root: NONE,
        }
    }
}
fn to_u32(value: usize) -> u32 {
    u32::try_from(value).expect("AST tree offsets fit in u32 (input_bytes <= u32::MAX)")
}
fn span_of(node: &TreeNode) -> Span {
    [node.span[0] as usize, node.span[1] as usize]
}
impl AstTree {
    /// 解析した入力（Text 節点の値はこの部分文字列）。
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn root(&self) -> Option<NodeRef<'_>> {
        (self.root != NONE).then_some(NodeRef {
            tree: self,
            id: self.root,
        })
    }
    /// 任意の節点（`NodeRef::id` の値）。範囲外は `None`。
    pub fn node(&self, id: u32) -> Option<NodeRef<'_>> {
        ((id as usize) < self.nodes.len()).then_some(NodeRef { tree: self, id })
    }
    /// arena の節点数（Text と、根から辿れない節点を含む）。
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    /// 子の参照に使っている slot（u32）の数。
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }
    /// 型付き節点の数（`Statistics::ast_nodes` と同じ）。
    pub fn typed_node_count(&self) -> usize {
        self.typed as usize
    }
    /// 所有 `Ast` へ写す（`ParseOptions::build_ast` が作るものと同じ値）。
    pub fn to_ast(&self) -> Option<super::Ast> {
        self.root().map(|root| root.to_ast())
    }
    /// `ParseResult::node_spans` と同じ表（作成順）。
    pub fn node_spans(&self) -> Vec<NodeSpan> {
        let mut out = Vec::with_capacity(self.typed as usize);
        self.push_node_spans(&mut out);
        out
    }
    /// `ParseOptions::value_spans` と同じ表（canonical AST の JSON pointer 順）。
    pub fn value_spans(&self) -> Vec<ValueSpan> {
        let mut out = Vec::new();
        if self.root != NONE {
            let mut path = String::with_capacity(128);
            self.collect_value_spans(&self.source, self.root, &mut path, &mut out);
        }
        out
    }
    /// この木が確保している byte 数（容量ベース）。
    pub fn heap_bytes(&self) -> usize {
        self.source.capacity()
            + self.nodes.capacity() * std::mem::size_of::<TreeNode>()
            + self.slots.capacity() * std::mem::size_of::<u32>()
    }
    #[doc(hidden)]
    pub fn push_node_spans(&self, out: &mut Vec<NodeSpan>) {
        for node in &self.nodes {
            if node.kind > KIND_NULL {
                out.push(NodeSpan {
                    node_id: node.node_id as usize,
                    rule_id: RULE_IDS[node.rule as usize],
                    node_type: TYPE_NAMES[node.kind as usize],
                    span: span_of(node),
                });
            }
        }
    }
    #[doc(hidden)]
    pub fn clear(&mut self) {
        self.source.clear();
        self.nodes.clear();
        self.slots.clear();
        self.typed = 0;
        self.root = NONE;
    }
    #[doc(hidden)]
    pub fn reserve(&mut self, nodes: usize, slots: usize) {
        self.nodes.reserve(nodes);
        self.slots.reserve(slots);
    }
    #[doc(hidden)]
    pub fn set_root(&mut self, root: u32) {
        self.root = root;
    }
    #[doc(hidden)]
    pub fn set_source(&mut self, source: &str) {
        self.source.clear();
        self.source.push_str(source);
    }
    #[inline]
    fn push(&mut self, node: TreeNode) -> u32 {
        let id = to_u32(self.nodes.len());
        self.nodes.push(node);
        id
    }
    /// 入力 `[bytes[0], bytes[1])` を値とする Text 節点。`extent` は値の出自（code point）。
    #[inline]
    #[doc(hidden)]
    pub fn text(&mut self, extent: Span, bytes: [usize; 2]) -> u32 {
        self.push(TreeNode {
            kind: KIND_TEXT,
            rule: 0,
            node_id: 0,
            span: [to_u32(extent[0]), to_u32(extent[1])],
            a: to_u32(bytes[0]),
            b: to_u32(bytes[1]),
        })
    }
    /// Text の出自を付け直す（leaf 型へ昇格した Text は capture の範囲を出自にする）。
    #[doc(hidden)]
    pub fn set_extent(&mut self, id: u32, extent: Span) {
        self.nodes[id as usize].span = [to_u32(extent[0]), to_u32(extent[1])];
    }
    /// list field の要素を積み、record の 2 slot（開始, 個数）を返す。
    #[inline]
    #[doc(hidden)]
    pub fn list(&mut self, items: &[u32]) -> [u32; 2] {
        let start = to_u32(self.slots.len());
        self.slots.extend_from_slice(items);
        [start, to_u32(items.len())]
    }
    #[inline]
    #[doc(hidden)]
    pub fn record(&mut self, kind: u16, rule: u16, span: Span, fields: &[u32]) -> u32 {
        let first = to_u32(self.slots.len());
        self.slots.extend_from_slice(fields);
        let node_id = self.typed;
        self.typed += 1;
        self.push(TreeNode {
            kind,
            rule,
            node_id,
            span: [to_u32(span[0]), to_u32(span[1])],
            a: first,
            b: to_u32(fields.len()),
        })
    }
    #[inline]
    #[doc(hidden)]
    pub fn enumeration(&mut self, kind: u16, rule: u16, span: Span, value: u32) -> u32 {
        let node_id = self.typed;
        self.typed += 1;
        self.push(TreeNode {
            kind,
            rule,
            node_id,
            span: [to_u32(span[0]), to_u32(span[1])],
            a: value,
            b: 0,
        })
    }
    #[inline]
    #[doc(hidden)]
    pub fn kind(&self, id: u32) -> u16 {
        self.nodes[id as usize].kind
    }
    #[doc(hidden)]
    pub fn type_id_of(&self, id: u32) -> &'static str {
        TYPE_IDS[self.kind(id) as usize]
    }
    #[doc(hidden)]
    pub fn type_name_of(&self, id: u32) -> &'static str {
        TYPE_NAMES[self.kind(id) as usize]
    }
    #[inline]
    fn slot(&self, id: u32, index: usize) -> u32 {
        self.slots[self.nodes[id as usize].a as usize + index]
    }
    #[inline]
    fn items(&self, id: u32, index: usize) -> &[u32] {
        let first = self.nodes[id as usize].a as usize + index;
        let (start, len) = (self.slots[first] as usize, self.slots[first + 1] as usize);
        &self.slots[start..start + len]
    }
    /// list field の要素（`slots` の開始位置と個数）。
    #[inline]
    fn list_at(&self, start: u32, len: u32) -> &[u32] {
        &self.slots[start as usize..start as usize + len as usize]
    }
    #[inline]
    fn text_in<'s>(&self, source: &'s str, id: u32) -> &'s str {
        let node = &self.nodes[id as usize];
        &source[node.a as usize..node.b as usize]
    }
    fn text_value_span(&self, id: u32, path: &str, out: &mut Vec<ValueSpan>) {
        out.push(ValueSpan {
            path: path.to_owned(),
            span: span_of(&self.nodes[id as usize]),
            text: None,
        });
    }
}
/// 木の 1 節点（Text / Null / 型付き節点）。
#[derive(Clone, Copy)]
pub struct NodeRef<'t> {
    tree: &'t AstTree,
    id: u32,
}
impl PartialEq for NodeRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.tree, other.tree) && self.id == other.id
    }
}
impl Eq for NodeRef<'_> {}
impl std::fmt::Debug for NodeRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRef")
            .field("id", &self.id)
            .field("type", &self.type_name())
            .finish()
    }
}
impl<'t> NodeRef<'t> {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn tree(&self) -> &'t AstTree {
        self.tree
    }
    /// 型付き節点の範囲（code point）。Text / Null は `None`（`Ast::span` と同じ）。
    pub fn span(&self) -> Option<Span> {
        let node = &self.tree.nodes[self.id as usize];
        (node.kind > KIND_NULL).then(|| span_of(node))
    }
    pub fn type_name(&self) -> &'static str {
        self.tree.type_name_of(self.id)
    }
    pub fn type_id(&self) -> &'static str {
        self.tree.type_id_of(self.id)
    }
    /// Text 節点の値（入力の部分文字列、複製しない）。
    pub fn text(&self) -> Option<&'t str> {
        (self.tree.kind(self.id) == KIND_TEXT)
            .then(|| self.tree.text_in(&self.tree.source, self.id))
    }
    pub fn is_null(&self) -> bool {
        self.tree.kind(self.id) == KIND_NULL
    }
    pub fn to_ast(&self) -> super::Ast {
        self.tree.project(&self.tree.source, self.id)
    }
}
/// list field（要素は節点）。
#[derive(Clone, Copy)]
pub struct NodeList<'t> {
    tree: &'t AstTree,
    ids: &'t [u32],
}
impl std::fmt::Debug for NodeList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<'t> NodeList<'t> {
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<NodeRef<'t>> {
        let tree = self.tree;
        self.ids.get(index).map(|&id| NodeRef { tree, id })
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = NodeRef<'t>> + 't {
        let tree = self.tree;
        self.ids.iter().map(move |&id| NodeRef { tree, id })
    }
}
/// list field（要素は文字列。入力の部分文字列を返す）。
#[derive(Clone, Copy)]
pub struct TextList<'t> {
    tree: &'t AstTree,
    ids: &'t [u32],
}
impl std::fmt::Debug for TextList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<'t> TextList<'t> {
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<&'t str> {
        let tree = self.tree;
        self.ids
            .get(index)
            .map(|&id| tree.text_in(&tree.source, id))
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &'t str> + 't {
        let tree = self.tree;
        self.ids
            .iter()
            .map(move |&id| tree.text_in(&tree.source, id))
    }
}
