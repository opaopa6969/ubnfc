//! 生成 UBNF パーサの採用結果から構文木へのフロント専用変換 (D-012)。
//!
//! 型付き AST の値と、未 capture の suffix を含む rule occurrence を使う。
//! ソース全体を字句・構文解析し直す経路は持たない。

use crate::syntax::ast::*;

/// 生成バックエンドに依存しない、型付き AST の借用投影。
/// ノードの構造は occurrence が保持し、ここには mapping 済みの文字列を置く。
///
/// ノード・field・値をそれぞれ 1 本の配列に詰める。ノードごと・field ごとに
/// `Vec` を持つと投影だけで確保がノード数に比例して増えるため（実測 764 ノードで 885 回）、
/// 範囲の始点だけを持ち、終点は次の要素の始点から求める。
#[derive(Debug, Default)]
pub struct MappedNodes<'a> {
    nodes: Vec<MappedNode<'a>>,
    fields: Vec<MappedField<'a>>,
    values: Vec<&'a str>,
}

#[derive(Debug)]
struct MappedNode<'a> {
    kind: &'a str,
    span_cp: [usize; 2],
    field_start: u32,
}

#[derive(Debug)]
struct MappedField<'a> {
    name: &'a str,
    value_start: u32,
}

impl<'a> MappedNodes<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    /// `nodes` はノード数の上限の目安。field / 値も同じ目安で確保しておく。
    pub fn with_capacity(nodes: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(nodes),
            fields: Vec::with_capacity(nodes),
            values: Vec::with_capacity(nodes),
        }
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    /// ノードを 1 つ足す。field は返り値の `field` で続けて足す。
    pub fn node<'s>(&'s mut self, kind: &'a str, span_cp: [usize; 2]) -> NodeFields<'s, 'a> {
        let field_start = self.fields.len() as u32;
        self.nodes.push(MappedNode {
            kind,
            span_cp,
            field_start,
        });
        NodeFields { nodes: self }
    }
    fn kind(&self, node: usize) -> &'a str {
        self.nodes[node].kind
    }
    fn span_cp(&self, node: usize) -> [usize; 2] {
        self.nodes[node].span_cp
    }
    /// ノードの field 値。名前が無ければ `None`（呼び手は capture 経路に落ちる）。
    fn values_of(&self, node: usize, field: &str) -> Option<&[&'a str]> {
        let start = self.nodes[node].field_start as usize;
        let end = self.nodes[node + 1..]
            .first()
            .map_or(self.fields.len(), |next| next.field_start as usize);
        let at = start
            + self.fields[start..end]
                .iter()
                .position(|f| f.name == field)?;
        let from = self.fields[at].value_start as usize;
        let to = self.fields[at + 1..]
            .first()
            .map_or(self.values.len(), |next| next.value_start as usize);
        Some(&self.values[from..to])
    }
}

/// 直前に足したノードへ field を足す借用。値は `&str` の並びをそのまま渡せる。
pub struct NodeFields<'s, 'a> {
    nodes: &'s mut MappedNodes<'a>,
}
impl<'a> NodeFields<'_, 'a> {
    pub fn field(&mut self, name: &'a str, values: impl IntoIterator<Item = &'a str>) -> &mut Self {
        let value_start = self.nodes.values.len() as u32;
        self.nodes.fields.push(MappedField { name, value_start });
        self.nodes.values.extend(values);
        self
    }
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
        // code point 数は byte 数を超えないので、1 回の確保で表を作り切る。
        let mut bytes = Vec::with_capacity(source.len() + 1);
        bytes.extend(source.char_indices().map(|(i, _)| i));
        bytes.push(source.len());
        Self { source, bytes }
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
    ///
    /// 走査は byte 単位で行う。判定に使う文字（空白・`/`・`\\`・`'`）はすべて ASCII で、
    /// 多 byte 文字の各 byte は 0x80 以上なのでどれにも一致せず「内容」として扱われる。
    /// 先頭 byte で `start` が、末尾の後続 byte で `end` が決まるので、
    /// 文字単位で走るのと同じ結果になる（UTF-8 の復号ぶんだけ速い）。
    fn content_span(&self, cp: [usize; 2]) -> Result<Span> {
        let raw = self.byte_span(cp)?;
        let source = self.source.as_bytes();
        let mut start = None;
        let mut end = raw.start;
        let mut at = raw.start;
        let mut quoted = false;
        let mut escaped = false;
        while at < raw.end {
            let byte = source[at];
            if !quoted {
                if is_space(byte) {
                    at += 1;
                    continue;
                }
                if byte == b'/' && at + 1 < raw.end && source[at + 1] == b'/' {
                    at += 2;
                    while at < raw.end && !matches!(source[at], b'\r' | b'\n') {
                        at += 1;
                    }
                    // 行末の改行自体も trivia として読み飛ばす。
                    at += 1;
                    continue;
                }
            }
            start.get_or_insert(at);
            end = at + 1;
            if escaped {
                escaped = false;
            } else if quoted && byte == b'\\' {
                escaped = true;
            } else if byte == b'\'' {
                quoted = !quoted;
            }
            at += 1;
        }
        // epsilon は次の構文トークンの位置。rule 開始時の trivia 消費後の位置を使う。
        Ok(Span {
            start: start.unwrap_or(raw.end),
            end: if start.is_some() { end } else { raw.end },
        })
    }
}

/// 境界 trivia を除いた `IDENTIFIER '.'` の原文から識別子だけを取る。
/// IDENTIFIER は C 識別子（`[A-Za-z_][A-Za-z0-9_]*`。手書き lexer の
/// `syntax::lexer` と同じ文字集合）なので、先頭からの識別子文字の連なりが
/// そのまま成分で、続く trivia と `'.'` は落ちる。
fn leading_identifier(raw: &str) -> &str {
    let end = raw
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(raw.len());
    &raw[..end]
}

fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 0x0b | 0x0c)
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
    // escape の無い文字列（大多数）は 1 回の確保で複製する。
    if !inner.contains('\\') {
        return Ok(inner.to_owned());
    }
    let mut result = String::with_capacity(inner.len());
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
    nodes: &MappedNodes<'_>,
    captures: &[Capture<'_>],
    occurrences: &[Occurrence<'_>],
) -> Result<UbnfFile> {
    Converter::new(source, nodes, captures, occurrences)?.file()
}

/// occurrence / node 索引の「無し」。索引は u32 に収める（超えたら変換を拒否する）。
const NONE: u32 = u32::MAX;

/// 圧縮隣接リスト（CSR）。`start[k]..start[k + 1]` が k 番の要素範囲。
struct Csr {
    start: Vec<u32>,
    index: Vec<u32>,
}
impl Csr {
    /// `owner[k]` が要素 k の所有者（`NONE` は所有者なし）。元の push 順（k の昇順）を保つ。
    fn from_owners(owner: &[u32], groups: usize) -> Self {
        let mut start = vec![0u32; groups + 1];
        for &o in owner {
            if o != NONE {
                start[o as usize + 1] += 1;
            }
        }
        for k in 0..groups {
            start[k + 1] += start[k];
        }
        let mut index = vec![0u32; start[groups] as usize];
        let mut cursor = start[..groups].to_vec();
        for (k, &o) in owner.iter().enumerate() {
            if o != NONE {
                let slot = &mut cursor[o as usize];
                index[*slot as usize] = k as u32;
                *slot += 1;
            }
        }
        Self { start, index }
    }
    fn slice(&self, k: usize) -> &[u32] {
        &self.index[self.start[k] as usize..self.start[k + 1] as usize]
    }
    fn slice_mut(&mut self, k: usize) -> &mut [u32] {
        &mut self.index[self.start[k] as usize..self.start[k + 1] as usize]
    }
}

/// 生成物の静的文字列は同じ番地で来るので、まず番地で照合する。
fn same_str(left: &str, right: &str) -> bool {
    std::ptr::eq(left.as_ptr(), right.as_ptr()) && left.len() == right.len() || left == right
}

/// 反復子を 1 度だけ走査して要素数と先頭要素を採る。`Vec` に集めない。
/// 要素の誤りは元の `collect::<Result<Vec<_>>>` と同じく個数検査より先に返す。
fn count_and_first<'a, I: Iterator<Item = Result<&'a str>>>(
    values: I,
) -> Result<(usize, Option<&'a str>)> {
    let mut count = 0usize;
    let mut first = None;
    for value in values {
        let value = value?;
        if count == 0 {
            first = Some(value);
        }
        count += 1;
    }
    Ok((count, first))
}

struct Converter<'a> {
    offsets: SourceOffsets<'a>,
    nodes: &'a MappedNodes<'a>,
    captures: &'a [Capture<'a>],
    occurrences: &'a [Occurrence<'a>],
    /// occurrence -> 子 occurrence（採用順 = occurrence 添字の昇順）。
    children: Csr,
    /// occurrence -> 所有 capture（completion_order 昇順）。
    owned_captures: Csr,
    /// occurrence -> 同じ kind・同じ span を持つ mapping ノード（無ければ `NONE`）。
    node_of: Vec<u32>,
    /// occurrence -> `rule_id` 末尾の kind。
    kind_of: Vec<&'a str>,
}
impl<'a> Converter<'a> {
    fn new(
        source: &'a str,
        nodes: &'a MappedNodes<'a>,
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
        let count = occurrences.len();
        if count.max(nodes.len()).max(captures.len()) >= NONE as usize {
            return Err(invalid("occurrence / capture / node が多すぎます"));
        }
        // occurrence ID -> 添字。整列した表の二分探索で、要素ごとの節点確保を避ける。
        let mut ids: Vec<(usize, u32)> = Vec::with_capacity(count);
        for (i, o) in occurrences.iter().enumerate() {
            offsets.byte_span(o.span_cp)?;
            if o.expr_id.is_empty() {
                return Err(invalid("採用 expression ID がありません"));
            }
            ids.push((o.id, i as u32));
        }
        ids.sort_unstable();
        if ids.windows(2).any(|w| w[0].0 == w[1].0) {
            return Err(invalid("occurrence ID が重複しています"));
        }
        let index_of = |id: usize| {
            ids.binary_search_by_key(&id, |&(key, _)| key)
                .ok()
                .map(|at| ids[at].1 as usize)
        };

        let mut parent_of = vec![NONE; count];
        let mut depth = vec![0u32; count];
        for (i, o) in occurrences.iter().enumerate() {
            if let Some(parent) = o.parent {
                let p = index_of(parent).ok_or_else(|| invalid("親 occurrence がありません"))?;
                let parent = &occurrences[p];
                if p >= i
                    || parent.span_cp[0] > o.span_cp[0]
                    || parent.span_cp[1] < o.span_cp[1]
                    || parent.completion_order <= o.completion_order
                {
                    return Err(invalid("occurrence の親子関係・範囲・完了順が不正です"));
                }
                parent_of[i] = p as u32;
                depth[i] = depth[p] + 1;
            }
        }
        let children = Csr::from_owners(&parent_of, count);

        // capture の所有 occurrence は「rule_id が一致し span を含むもののうち最も深いもの」。
        // rule_id ごとに occurrence をまとめ、深さ昇順に並べて後ろから探すことで、
        // 元の「全 occurrence の線形走査 × capture 数」（1,383 × 1,286）を避ける。
        // 同じ深さが複数一致するときは元の `max_by_key`（最後の最大値）と同じく添字の大きい方を採る。
        // rule_id は生成物の静的文字列なので、まず番地で照合して文字列比較を避ける。
        let mut rule_ids: Vec<&'a str> = Vec::new();
        let mut rule_group = vec![0u32; count];
        for (i, o) in occurrences.iter().enumerate() {
            let group = rule_ids
                .iter()
                .position(|rule| same_str(rule, o.rule_id))
                .unwrap_or_else(|| {
                    rule_ids.push(o.rule_id);
                    rule_ids.len() - 1
                });
            rule_group[i] = group as u32;
        }
        let mut by_rule = Csr::from_owners(&rule_group, rule_ids.len());
        // 走査は「所属 occurrence の span を順に読む」だけなので、span を並べて持つ。
        // 1 グループぶんが cache に乗り、`occurrences` への飛び飛びの参照が消える。
        let mut by_rule_span: Vec<(u32, [usize; 2])> = Vec::with_capacity(count);
        for group in 0..rule_ids.len() {
            by_rule.slice_mut(group).sort_by_key(|&i| depth[i as usize]);
            by_rule_span.extend(
                by_rule
                    .slice(group)
                    .iter()
                    .map(|&i| (i, occurrences[i as usize].span_cp)),
            );
        }

        let mut capture_owner: Vec<u32> = Vec::with_capacity(captures.len());
        // capture 側の rule_id は別の静的表から来るので番地が一致しない。
        // 一度引いた結果を番地で覚え、文字列比較を rule 種類の数だけに抑える。
        let mut resolved: Vec<(*const u8, usize, u32)> = Vec::new();
        for capture in captures {
            offsets.byte_span(capture.span_cp)?;
            let key = (capture.rule_id.as_ptr(), capture.rule_id.len());
            let group = match resolved.iter().find(|r| (r.0, r.1) == key) {
                Some(&(_, _, group)) => group,
                None => {
                    let group = rule_ids
                        .iter()
                        .position(|rule| *rule == capture.rule_id)
                        .map_or(NONE, |group| group as u32);
                    resolved.push((key.0, key.1, group));
                    group
                }
            };
            let owner = if group == NONE {
                None
            } else {
                let group = group as usize;
                by_rule_span[by_rule.start[group] as usize..by_rule.start[group + 1] as usize]
                    .iter()
                    .rev()
                    .find(|(_, span)| {
                        span[0] <= capture.span_cp[0] && capture.span_cp[1] <= span[1]
                    })
                    .map(|&(i, _)| i)
            };
            capture_owner
                .push(owner.ok_or_else(|| invalid("capture の所有 occurrence がありません"))?);
        }
        let mut owned_captures = Csr::from_owners(&capture_owner, count);
        for i in 0..count {
            // 安定整列なので、completion_order が同じなら capture 添字の昇順が残る。
            owned_captures
                .slice_mut(i)
                .sort_by_key(|&c| captures[c as usize].completion_order);
        }

        // (span, kind) -> mapping ノード。occurrence ごとに 1 度だけ引き当てておく。
        // span の始点で直接引ける表を作り、二分探索の飛び飛びの参照を避ける。
        let mut keys: Vec<([usize; 2], &'a str, u32)> = Vec::with_capacity(nodes.len());
        for node in 0..nodes.len() {
            let span_cp = nodes.span_cp(node);
            offsets.byte_span(span_cp)?;
            keys.push((span_cp, nodes.kind(node), node as u32));
        }
        keys.sort_unstable_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        if keys
            .windows(2)
            .any(|w| w[0].0 == w[1].0 && w[0].1 == w[1].1)
        {
            return Err(invalid("mapping ノードの対応が曖昧です"));
        }
        let mut first_at = vec![NONE; offsets.bytes.len()];
        for (at, key) in keys.iter().enumerate().rev() {
            first_at[key.0[0]] = at as u32;
        }
        // `rule_id` 末尾の kind は木を辿るたびに要るので、ここで 1 度だけ切り出す。
        let mut kind_of: Vec<&'a str> = Vec::with_capacity(count);
        kind_of.extend(
            occurrences
                .iter()
                .map(|o| o.rule_id.rsplit("::").next().unwrap()),
        );
        let mut node_of = vec![NONE; count];
        for (i, o) in occurrences.iter().enumerate() {
            let mut at = first_at[o.span_cp[0]];
            while at != NONE {
                let key = &keys[at as usize];
                if key.0[0] != o.span_cp[0] {
                    break;
                }
                if key.0[1] == o.span_cp[1] && key.1 == kind_of[i] {
                    node_of[i] = key.2;
                    break;
                }
                at = if (at as usize + 1) < keys.len() {
                    at + 1
                } else {
                    NONE
                };
            }
        }

        Ok(Self {
            offsets,
            nodes,
            captures,
            occurrences,
            children,
            owned_captures,
            node_of,
            kind_of,
        })
    }
    fn kind(&self, i: usize) -> &'a str {
        self.kind_of[i]
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
    /// 子 occurrence を kind で絞る。中間の `Vec` を作らない。
    fn children<'s>(&'s self, i: usize, kind: &'s str) -> impl Iterator<Item = usize> + 's {
        self.children
            .slice(i)
            .iter()
            .map(|&j| j as usize)
            .filter(move |&j| self.kind(j) == kind)
    }
    fn child(&self, i: usize, kind: &str) -> Result<usize> {
        let mut found = None;
        let mut count = 0usize;
        for j in self.children(i, kind) {
            if count == 0 {
                found = Some(j);
            }
            count += 1;
        }
        match found {
            Some(j) if count == 1 => Ok(j),
            _ => Err(self.error(i, format!("{} に {kind} が一つ必要です", self.kind(i)))),
        }
    }
    fn branch(&self, i: usize) -> Result<usize> {
        match *self.children.slice(i) {
            [only] => Ok(only as usize),
            _ => Err(self.error(i, "採用枝が一つ必要です")),
        }
    }
    /// 型付き AST 側の値（借用のまま）。field が無ければ `None` で capture 経路に落ちる。
    fn node_values(&self, i: usize, field: &str) -> Option<&'a [&'a str]> {
        let at = self.node_of[i];
        if at == NONE {
            return None;
        }
        let nodes: &'a MappedNodes<'a> = self.nodes;
        nodes.values_of(at as usize, field)
    }
    /// capture 経路の値。`raw` のときは引用符込みの原文（境界 trivia を除いたもの）。
    fn capture_values<'s>(
        &'s self,
        i: usize,
        field: &'s str,
        raw: bool,
    ) -> impl Iterator<Item = Result<&'a str>> + 's {
        let captures: &'a [Capture<'a>] = self.captures;
        self.owned_captures
            .slice(i)
            .iter()
            .map(move |&c| &captures[c as usize])
            .filter(move |c| c.name == field)
            .map(move |c| {
                if raw {
                    let span = self.offsets.content_span(c.span_cp)?;
                    Ok(&self.offsets.source[span.start..span.end])
                } else {
                    self.offsets.text(c.span_cp)
                }
            })
    }
    /// 型付き AST 側にあればそれを、無ければ capture 経路を反復子として渡す。
    /// `Vec` に集めないので、値の複製はここでは起きない。
    fn with_values<R>(
        &self,
        i: usize,
        field: &str,
        use_values: impl FnOnce(&mut dyn Iterator<Item = Result<&'a str>>) -> R,
    ) -> R {
        match self.node_values(i, field) {
            Some(values) => use_values(&mut values.iter().copied().map(Ok)),
            None => use_values(&mut self.capture_values(i, field, false)),
        }
    }
    fn scalar_error(&self, i: usize, field: &str) -> ConversionError {
        self.error(
            i,
            format!("{}.{field} に scalar capture が必要です", self.kind(i)),
        )
    }
    fn optional_error(&self, i: usize, field: &str) -> ConversionError {
        self.error(i, format!("{field} の optional capture が重複しています"))
    }
    fn value(&self, i: usize, field: &str) -> Result<String> {
        self.with_values(i, field, |values| match count_and_first(values)? {
            (1, Some(value)) => Ok(value.to_owned()),
            _ => Err(self.scalar_error(i, field)),
        })
    }
    fn optional(&self, i: usize, field: &str) -> Result<Option<String>> {
        self.with_values(i, field, |values| match count_and_first(values)? {
            (0, _) => Ok(None),
            (1, Some(value)) => Ok(Some(value.to_owned())),
            _ => Err(self.optional_error(i, field)),
        })
    }
    /// `QuantifiedRef ::= { IDENTIFIER '.' } @namespace IDENTIFIER @name` の
    /// `@namespace` は反復 1 回ぶんずつの capture（`IDENTIFIER '.'`）の並びなので、
    /// 各成分の識別子を取り出して `.` で連結し、手書き frontend と同じ
    /// `namespace` 文字列（`a.b`）にする。0 個なら namespace 無し、1 個なら `a`。
    /// 連結先の `String` は一つだけで、成分ごとの中間 `String` は作らない。
    fn namespace(&self, i: usize) -> Result<Option<String>> {
        let mut joined: Option<String> = None;
        for value in self.capture_values(i, "namespace", true) {
            let segment = leading_identifier(value?);
            match &mut joined {
                None => joined = Some(segment.to_owned()),
                Some(out) => {
                    out.push('.');
                    out.push_str(segment);
                }
            }
        }
        Ok(joined)
    }
    fn strings(&self, i: usize, field: &str) -> Result<Vec<String>> {
        self.with_values(i, field, |values| {
            let mut out = Vec::new();
            for value in values {
                out.push(value?.to_owned());
            }
            Ok(out)
        })
    }
    /// STRING capture の原文 (引用符付き)。型付き AST の値は引用符を除いた
    /// 内容なので、raw 保持と escape 復号は capture span の原文から行う。
    fn raw_scalar(&self, i: usize, field: &str) -> Result<&'a str> {
        match count_and_first(self.capture_values(i, field, true))? {
            (1, Some(value)) => Ok(value),
            _ => Err(self.scalar_error(i, field)),
        }
    }
    fn raw_string(&self, i: usize, field: &str) -> Result<String> {
        Ok(self.raw_scalar(i, field)?.to_owned())
    }
    fn string(&self, i: usize, field: &str) -> Result<String> {
        decode_string(self.raw_scalar(i, field)?, self.span(i)?)
    }
    /// STRING capture の原文から引用符を落としたもの。v2 の `\uXXXX` 展開（D-063）が
    /// 使う。手書き parser の `string_with_raw` と同じ文字列になる。
    fn inner_raw(&self, i: usize, field: &str) -> Result<String> {
        let raw = self.raw_scalar(i, field)?;
        let trimmed = raw
            .strip_prefix(['\'', '"'])
            .and_then(|rest| rest.strip_suffix(['\'', '"']))
            .unwrap_or(raw);
        Ok(trimmed.to_owned())
    }
    fn optional_string(&self, i: usize, field: &str) -> Result<Option<String>> {
        let raw = match count_and_first(self.capture_values(i, field, true))? {
            (0, _) => None,
            (1, Some(value)) => Some(value),
            _ => return Err(self.optional_error(i, field)),
        };
        raw.map(|value| decode_string(value, self.span(i)?))
            .transpose()
    }
    fn file(&self) -> Result<UbnfFile> {
        let mut root = None;
        let mut roots = 0usize;
        for (i, o) in self.occurrences.iter().enumerate() {
            if o.parent.is_none() {
                if roots == 0 {
                    root = Some(i);
                }
                roots += 1;
            }
        }
        let root = match root {
            Some(root) if roots == 1 && self.kind(root) == "UBNFFile" => root,
            _ => {
                return Err(ConversionError {
                    message: "UBNFFile の採用 root が一つ必要です".into(),
                    span: Span::default(),
                })
            }
        };
        if self.node_of[root] == NONE {
            return Err(self.error(root, "生成結果に型付き UBNFFile がありません"));
        }
        Ok(UbnfFile {
            grammars: self
                .children(root, "GrammarDecl")
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
            // 自己記述文法（v1）はパラメータ化ルールを綴れない（D-043）。
            templates: Vec::new(),
            imports: self
                .children(i, "ImportDecl")
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
                .map(|j| self.setting(j))
                .collect::<Result<_>>()?,
            tokens: self
                .children(i, "TokenDecl")
                .map(|j| {
                    Ok(TokenDecl {
                        name: self.value(j, "name")?,
                        value: if let Some(value) = self.children(j, "DeclarativeTokenValue").next()
                        {
                            TokenValue::Declarative {
                                expression: self.lexical(self.child(value, "LexicalBody")?)?,
                                span: self.span(value)?,
                            }
                        } else {
                            self.token(self.branch(self.child(j, "TokenValue")?)?)?
                        },
                        span: self.span(j)?,
                    })
                })
                .collect::<Result<_>>()?,
            rules: self
                .children(i, "RuleDecl")
                .map(|j| self.rule(j))
                .collect::<Result<_>>()?,
        })
    }
    fn dotted(&self, i: usize) -> Result<String> {
        self.with_values(i, "parts", |values| {
            let mut out = String::new();
            let mut first = true;
            for value in values {
                if !first {
                    out.push('.');
                }
                first = false;
                out.push_str(value?);
            }
            Ok(out)
        })
    }
    /// Build the lexical program from generated-parser occurrences, never by
    /// reparsing a declaration through the bootstrap parser. The lexer is used
    /// only to decode scalar arguments and bounds already recognized by UBNF.
    fn lexical(&self, i: usize) -> Result<crate::declarative::LexicalExpression> {
        use crate::declarative::{LexicalExpression as Lex, Op};
        use crate::syntax::lexer::{lex, Kind};
        match self.kind(i) {
            "LexicalBody" | "LexicalSequence" => {
                let (op, kind) = if self.kind(i) == "LexicalBody" {
                    (Op::CHOICE, "LexicalSequence")
                } else {
                    (Op::SEQUENCE, "LexicalElement")
                };
                Ok(Lex::node(
                    op,
                    self.children(i, kind)
                        .map(|j| self.lexical(j))
                        .collect::<Result<_>>()?,
                ))
            }
            "LexicalElement" => {
                let atom = self.lexical(self.child(i, "LexicalAtom")?)?;
                let Some(suffix) = self.children(i, "LexicalSuffix").next() else {
                    return Ok(atom);
                };
                let raw = self.raw_scalar(suffix, "text")?;
                let bounds = match raw.trim() {
                    "?" => (0, 1),
                    "*" => (0, -1),
                    "+" => (1, -1),
                    _ => {
                        let tokens = lex(raw).map_err(|e| self.error(suffix, e.to_string()))?;
                        let values = tokens
                            .iter()
                            .filter_map(|t| {
                                if let Kind::Digits(v) = t.kind {
                                    Some(v)
                                } else {
                                    None
                                }
                            })
                            .map(|v| {
                                v.parse::<i32>()
                                    .map_err(|_| self.error(suffix, "32-bit repeat bound"))
                            })
                            .collect::<Result<Vec<_>>>()?;
                        let Some(&min) = values.first() else {
                            return Err(self.error(suffix, "repeat bound"));
                        };
                        let comma = tokens.iter().any(|t| matches!(t.kind, Kind::Punct(',')));
                        let max = values
                            .get(1)
                            .copied()
                            .unwrap_or(if comma { -1 } else { min });
                        if max >= 0 && min > max {
                            return Err(self.error(suffix, "ordered repeat bounds"));
                        }
                        (min, max)
                    }
                };
                Ok(Lex::repeat(atom, bounds.0, bounds.1))
            }
            "LexicalAtom" => {
                let raw = self.raw_scalar(i, "value")?;
                let tokens = lex(raw).map_err(|e| self.error(i, e.to_string()))?;
                let nested = || self.lexical(self.child(i, "LexicalBody")?);
                let name_arg = || match tokens.get(2).map(|t| &t.kind) {
                    Some(Kind::Identifier(name)) => Ok((*name).to_owned()),
                    _ => Err(self.error(i, "capture name")),
                };
                match &tokens[0].kind {
                    Kind::String { value, .. } => Ok(Lex::leaf(Op::LITERAL, value.clone())),
                    Kind::Punct('(') => nested(),
                    Kind::Punct('[') => Ok(Lex::repeat(nested()?, 0, 1)),
                    Kind::Punct('{') => Ok(Lex::repeat(nested()?, 0, -1)),
                    Kind::Identifier(name)
                        if matches!(tokens.get(1).map(|t| &t.kind), Some(Kind::Punct('('))) =>
                    {
                        match *name {
                            "LOOKAHEAD" | "NEGATIVE_LOOKAHEAD" => Ok(Lex::node(
                                if *name == "LOOKAHEAD" {
                                    Op::LOOK
                                } else {
                                    Op::NOT
                                },
                                vec![nested()?],
                            )),
                            "CAPTURE" => Ok(Lex {
                                text: name_arg()?,
                                ..Lex::node(Op::CAPTURE, vec![nested()?])
                            }),
                            "SAME_AS" => Ok(Lex::leaf(Op::BACKREF, name_arg()?)),
                            "NEGATION" | "CHAR_RANGE" => {
                                let strings = tokens
                                    .iter()
                                    .filter_map(|t| {
                                        if let Kind::String { value, .. } = &t.kind {
                                            Some(value.as_str())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<_>>();
                                if *name == "NEGATION" && strings.len() == 1 {
                                    return Ok(Lex::leaf(Op::EXCEPT, strings[0].into()));
                                }
                                if strings.len() != 2
                                    || strings.iter().any(|s| s.chars().count() != 1)
                                {
                                    return Err(self.error(i, "one Unicode scalar"));
                                }
                                let min = strings[0].chars().next().unwrap() as i32;
                                let max = strings[1].chars().next().unwrap() as i32;
                                if min > max {
                                    return Err(self.error(i, "ordered scalar range"));
                                }
                                Ok(Lex {
                                    min,
                                    max,
                                    ..Lex::leaf(Op::RANGE, String::new())
                                })
                            }
                            _ => Err(self.error(i, "unknown lexical operator")),
                        }
                    }
                    Kind::Identifier(_) => {
                        let name = self.dotted(self.child(i, "DottedIdentifier")?)?;
                        let op = match name.as_str() {
                            "ANY" => Op::ANY,
                            "EOF" => Op::EOF,
                            "BOF" => Op::BOF,
                            "BOL" => Op::BOL,
                            "EOL" => Op::EOL,
                            _ => Op::REF,
                        };
                        Ok(Lex::leaf(
                            op,
                            if op == Op::REF { name } else { String::new() },
                        ))
                    }
                    _ => Err(self.error(i, "lexical atom")),
                }
            }
            _ => Err(self.error(i, "lexical occurrence")),
        }
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
                chars_raw: self.inner_raw(i, "chars")?,
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
                min_raw: self.inner_raw(i, "min")?,
                max_raw: self.inner_raw(i, "max")?,
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
                // ubnf.ubnf は v1 のメタ文法で旧綴りしか持たない（D-046）。
                mode: None,
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
            .map(|j| {
                let elements = self
                    .children(j, "AnnotatedElement")
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
        if let Some(j) = self.children(i, "BoundedQuantifier").next() {
            let span = self.span(j)?;
            let mut numbers = Vec::new();
            for k in self.children(j, "Digits") {
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
            .next()
            .map(|j| self.atom(self.child(j, "AtomicElement")?))
            .transpose()?;
        let typeof_constraint = self
            .children(i, "TypeofElement")
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
                namespace: self.namespace(j)?,
                name: self.value(j, "name")?,
                span,
            },
            kind => return Err(self.error(j, format!("未知の atom: {kind}"))),
        })
    }
}
