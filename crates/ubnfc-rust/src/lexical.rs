//! 統合 DFA（全 literal 終端の byte trie）と、その結果を位置ごとに保存する cache の生成。
//!
//! 対象は「大文字小文字を区別する ASCII の非空 literal」だけにする。理由:
//! - CI literal は `equal_ignore_case`（Java `equalsIgnoreCase` 近似）で、非 ASCII の入力が
//!   ASCII の文字と一致し得る（`'İ'` と `'i'`）。byte trie では再現できないので走査に残す。
//! - 非 ASCII を含む literal は先頭 byte が継続 byte と衝突し得るため、文字境界の検査を
//!   trie では省けない。ASCII だけなら「その位置の byte が ASCII」＝「文字境界」なので、
//!   `Input::starts_with` の境界検査を DFA が自然に満たす。
//! - 空 literal は EOF を含む任意の文字境界で一致する（位置が境界かどうかは byte trie では
//!   判定しない）ので除く。
//!
//! 長さは literal ごとに静的（word の byte 長）なので、cache は「一致した literal の集合」
//! だけを持てばよい（位置ごとの長さ配列は要らない）。
use crate::emit::Generator;
use std::collections::BTreeMap;
use std::fmt::Write;
use ubnfc_front::ir::*;

/// 文法が使う literal → 統合 DFA の id。
pub(crate) struct Lexicon {
    pub ids: BTreeMap<String, u32>,
}

/// 統合 DFA に入れる literal の最小 byte 長。1 byte の literal は `starts_with` が
/// 1 命令で終わるので、cache を引くほうが高くつく（報告の crossover 参照）。
const MIN_LITERAL_BYTES: usize = 1;

fn cacheable(value: &str, case_sensitive: bool) -> bool {
    case_sensitive && value.len() >= MIN_LITERAL_BYTES.max(1) && value.is_ascii()
}

impl Lexicon {
    pub fn new(ir: &GrammarIr) -> Self {
        let mut words: Vec<&str> = Vec::new();
        for expr in &ir.semantics.expressions {
            if let ExprNode::Literal {
                value,
                case_sensitive,
                ..
            } = &expr.node
            {
                if cacheable(value, *case_sensitive) && !words.contains(&value.as_str()) {
                    words.push(value);
                }
            }
        }
        // id 順は宣言順に依存しない（決定的な生成のため辞書順）。
        words.sort_unstable();
        Self {
            ids: words
                .into_iter()
                .enumerate()
                .map(|(i, w)| (w.to_owned(), u32::try_from(i).expect("literal id")))
                .collect(),
        }
    }
    /// bitset の u64 語数。
    pub fn words(&self) -> usize {
        self.ids.len().div_ceil(64)
    }
    /// 生成コードが `literal` へ渡す id（cache 対象外は u32::MAX）。
    pub fn id(&self, value: &str, case_sensitive: bool) -> String {
        if !cacheable(value, case_sensitive) {
            return "u32::MAX".to_owned();
        }
        match self.ids.get(value) {
            Some(id) => id.to_string(),
            None => "u32::MAX".to_owned(),
        }
    }
}

/// trie node: byte → 子 node。accept は「この node に到達したら一致する literal の id」。
#[derive(Default)]
struct Node {
    edges: BTreeMap<u8, usize>,
    accept: Option<u32>,
}

/// `lex_fill`（統合 DFA）と語数の定数を生成する。
pub(crate) fn generate(g: &Generator<'_>) -> String {
    let lexicon = &g.lexicon;
    let mut s = String::new();
    writeln!(
        s,
        "/// 統合 DFA の bitset 語数（literal {} 個）。0 なら cache を作らない。",
        lexicon.ids.len()
    )
    .unwrap();
    writeln!(s, "const LEX_WORDS:usize={};", lexicon.words()).unwrap();
    writeln!(
        s,
        "/// 診断を記録しない読み飛ばし cache を持つ trivia policy 数（区切り集合で正規化済み）。\nconst TRIVIA_POLICIES:usize={};",
        crate::predict::canonical_policies(g.ir)
            .iter()
            .max()
            .map_or(0, |m| m + 1)
    )
    .unwrap();
    if lexicon.ids.is_empty() {
        s.push_str("#[inline]\nfn lex_fill(_bytes:&[u8],_pos:usize,_out:&mut [u64]) {}\n");
        return s;
    }
    let mut nodes: Vec<Node> = vec![Node::default()];
    for (word, id) in &lexicon.ids {
        let mut current = 0usize;
        for &byte in word.as_bytes() {
            current = match nodes[current].edges.get(&byte) {
                Some(&next) => next,
                None => {
                    nodes.push(Node::default());
                    let next = nodes.len() - 1;
                    nodes[current].edges.insert(byte, next);
                    next
                }
            };
        }
        nodes[current].accept = Some(*id);
    }
    s.push_str("/// 全 literal 終端を 1 本にまとめた DFA（trie）。位置 `pos` から 1 回走らせ、\n");
    s.push_str("/// 一致した literal の id を `out` の bitset に立てる。長さは id ごとに静的。\n");
    s.push_str("fn lex_fill(bytes:&[u8],pos:usize,out:&mut [u64]) {\nlet mut p=pos;let mut state=0usize;\nloop {\nlet Some(&c)=bytes.get(p) else {return;};\nstate=match state {\n");
    for (index, node) in nodes.iter().enumerate() {
        if node.edges.is_empty() {
            continue;
        }
        writeln!(s, "{index}=>match c {{").unwrap();
        for (&byte, &target) in &node.edges {
            match nodes[target].accept {
                Some(id) => writeln!(
                    s,
                    "{byte}=>{{out[{}]|={};{target}}},",
                    id as usize / 64,
                    1u64 << (id % 64)
                )
                .unwrap(),
                None => writeln!(s, "{byte}=>{target},").unwrap(),
            }
        }
        s.push_str("_=>return,\n},\n");
    }
    s.push_str("_=>return,\n};\np+=1;\n}\n}\n");
    s
}
