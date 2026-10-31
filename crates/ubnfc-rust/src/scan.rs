//! D-072 — `plan.scans` の領域を parser の機構を通さない走査関数へ落とす。
//!
//! front が「領域は終端だけでできていて観測を 1 つも残さない」ことを証明している
//! （`crates/ubnfc-front/src/analysis/scan.rs`）。ここで生成するのは
//! `fn s{i}(&self, p: usize) -> Option<usize>`（消費 byte 数）だけの純関数で、
//! Step・事象 arena・診断・mark・transaction・memo を一切触らない。
//!
//! 呼び出しは `b{i}_c` の先頭に置き、次の条件でだけ走査へ入る:
//!
//! * `self.options.scan` — 参照経路へ戻す脱出口（`predict` / `factor` と同じ作法）。
//! * `!DIAG` — 診断 session は従来どおり式単位に評価する。fast session が失敗したら
//!   診断 session が入力を解析し直す（D-023）ので、走査は**成功した解析にしか効かない**。
//!   主診断 DAG・display・expected は構成上 1 bit も変わらない。
//! * `!self.options.lexical && !self.options.occurrences` — 出現表は領域内の
//!   rule / token 事象を数えるので、要求されているときは畳まない。
//! * `!state.invert` — `Not` の下では終端の意味が反転する。
//!
//! 事象は「走査した範囲ちょうど 1 件の token 事象」に畳む。領域の終端は隙間なく
//! 範囲を敷き詰めるので、`selected_extent`（先頭 token の開始と末尾 token の終了）は
//! 同じ span になる。capture site は領域内に無いことを front が証明している。

use crate::emit::Generator;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use ubnfc_front::ir::*;

pub(crate) struct Scans {
    /// 走査関数を生成する式（領域の根）。
    pub roots: BTreeSet<usize>,
    /// 走査関数を生成する全式（根と、その下の領域）。
    pub members: BTreeSet<usize>,
    /// 反復ごとの一括走査表。
    pub classes: BTreeMap<usize, ScanClass>,
}

impl Scans {
    pub fn empty() -> Self {
        Self {
            roots: BTreeSet::new(),
            members: BTreeSet::new(),
            classes: BTreeMap::new(),
        }
    }

    pub fn new(g: &Generator<'_>) -> Self {
        let mut roots = BTreeSet::new();
        let mut classes = BTreeMap::new();
        for plan in &g.ir.plan.scans {
            let Some(&i) = g.expr.get(plan.expr_id.as_str()) else {
                continue;
            };
            if plan.root {
                roots.insert(i);
            }
            if let Some(class) = &plan.class {
                classes.insert(i, class.clone());
            }
        }
        // 領域の中に memo を置く式があると、memo 表の使用量（資源上限）が経路で変わる。
        // そのような根は落とす（JSON のように D-065 で終端走査規則の memo が消えた
        // 文法では起きない）。
        roots.retain(|&root| {
            region(g, root)
                .into_iter()
                .all(|i| i == root || !crate::expressions::memoized(g, i))
        });
        let mut members = BTreeSet::new();
        for &root in &roots {
            members.extend(region(g, root));
        }
        classes.retain(|i, _| members.contains(i));
        Self {
            roots,
            members,
            classes,
        }
    }
}

fn region(g: &Generator<'_>, root: usize) -> BTreeSet<usize> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(i) = pending.pop() {
        if !seen.insert(i) {
            continue;
        }
        let expr = &g.ir.semantics.expressions[i];
        match &expr.node {
            ExprNode::RuleRef { symbol_id } => {
                let rule = &g.ir.semantics.rules[g.rules[symbol_id.as_str()]];
                pending.push(g.expr[rule.body.as_str()]);
            }
            node => pending.extend(node.children().iter().map(|id| g.expr[id.as_str()])),
        }
    }
    seen
}

/// 単一 code point の終端が真になる条件（`c` は `char`、`p` は byte 位置）。
/// `tokens.rs` の生成と同じ式を使う（同じ判定を二度書き分けない）。
fn leaf(g: &Generator<'_>, i: usize) -> Option<String> {
    let expr = &g.ir.semantics.expressions[i];
    match &expr.node {
        ExprNode::Any {} => Some("let _=c;true".into()),
        ExprNode::TokenRef { symbol_id } => {
            let token = &g.ir.semantics.tokens[g.tokens[symbol_id.as_str()]];
            token_condition(token)
        }
        _ => None,
    }
}

fn token_condition(token: &Token) -> Option<String> {
    Some(match &token.spec {
        TokenSpec::CharRange { min, max } => format!("({min}..={max}).contains(&(c as u64))"),
        TokenSpec::Negation { excluded } => format!("!{excluded:?}.contains(c)"),
        TokenSpec::Any {} => "let _=c;true".into(),
        TokenSpec::Builtin { which, code_points } => match which {
            BuiltinToken::Digit => "c.is_ascii_digit()".into(),
            BuiltinToken::Alphabet => "c.is_ascii_alphabetic()".into(),
            BuiltinToken::AlphabetUnderScore => "c.is_ascii_alphabetic() || c=='_'".into(),
            BuiltinToken::AlphabetNumericUnderScore => "c.is_ascii_alphanumeric() || c=='_'".into(),
            // `tokens.rs` は最初の byte を 9..=13 | 32 で見る。どれも ASCII なので同値。
            BuiltinToken::Space => "matches!(c,'\\t'..='\\r'|' ')".into(),
            BuiltinToken::CodePointSet => format!("{code_points:?}.contains(&(c as u64))"),
            _ => return None,
        },
        _ => return None,
    })
}

/// `{ X }` の一括走査表を 1 文字ぶん判定する式（`c`: char）。
fn class_condition(class: &ScanClass) -> String {
    let mut s = format!(
        "{{let cp=c as u32;if cp<64 {{({:#x}u64>>cp)&1!=0}} else if cp<128 {{({:#x}u64>>(cp-64))&1!=0}} else {{",
        class.ascii_low, class.ascii_high
    );
    if class.ranges.is_empty() {
        s.push_str("false");
    } else {
        let arms: Vec<String> = class
            .ranges
            .iter()
            .map(|r| format!("{}..={}", r.start, r.end))
            .collect();
        write!(s, "matches!(cp,{})", arms.join("|")).unwrap();
    }
    s.push_str("}}");
    s
}

/// 各 byte の上位 bit に区間内判定を置く。`x` は上位 bit を除いた word。
/// 各 lane に番兵 bit を足してから引くので、隣の lane へ borrow しない。
fn lane_range(lo: u8, hi: u8) -> String {
    let repeat = |b: u8| u64::from(b) * 0x0101_0101_0101_0101;
    match (lo, hi) {
        (0, 127) => "0x8080808080808080u64".into(),
        (0, hi) => format!("{:#x}u64-x", repeat(hi) | 0x8080_8080_8080_8080),
        (lo, 127) => format!("(x|0x8080808080808080u64)-{:#x}u64", repeat(lo)),
        (lo, hi) => format!(
            "((x|0x8080808080808080u64)-{:#x}u64)&({:#x}u64-x)",
            repeat(lo),
            repeat(hi) | 0x8080_8080_8080_8080
        ),
    }
}

/// ASCII 表の連続区間か、その補集合のうち短い方を SWAR 判定へ落とす。
/// 非 ASCII は常に止め、従来の code point 判定へ渡す。CPU 固有命令は使わない。
/// 返す式の各 lane 上位 bit が不一致を表す。little endian 読み込みと
/// trailing_zeros で、停止文字の前にある 0..8 byte だけを消費する。
fn word_failure_mask(class: &ScanClass) -> Option<String> {
    let ranges = |accepted: bool| {
        let mut result = Vec::new();
        let mut b = 0u16;
        while b < 128 {
            let member = |b: u16| {
                let bits = if b < 64 {
                    class.ascii_low
                } else {
                    class.ascii_high
                };
                (bits >> (b % 64)) & 1 != 0
            };
            if member(b) != accepted {
                b += 1;
                continue;
            }
            let lo = b;
            while b + 1 < 128 && member(b + 1) == accepted {
                b += 1;
            }
            result.push((lo as u8, b as u8));
            b += 1;
        }
        result
    };
    let yes = ranges(true);
    let no = ranges(false);
    if yes.is_empty() {
        return Some("0x8080808080808080u64".into());
    }
    if no.is_empty() {
        return Some("w&0x8080808080808080u64".into());
    }
    let complement = no.len() < yes.len();
    let intervals = if complement { no } else { yes };
    // 複雑な集合では短い走査に対して演算が増えるので既存の表引きを保つ。
    if intervals.len() > 3 {
        return None;
    }
    let mask = if intervals.len() == 1 {
        lane_range(intervals[0].0, intervals[0].1)
    } else {
        intervals
            .into_iter()
            .map(|(lo, hi)| format!("({})", lane_range(lo, hi)))
            .collect::<Vec<_>>()
            .join("|")
    };
    Some(if complement {
        format!("(w|({mask}))&0x8080808080808080u64")
    } else {
        format!("(w|!({mask}))&0x8080808080808080u64")
    })
}

/// 走査関数の本体。`p` から読み、消費した byte 数を返す。
fn body(g: &Generator<'_>, scans: &Scans, i: usize) -> Option<String> {
    let expr = &g.ir.semantics.expressions[i];
    if let Some(condition) = leaf(g, i) {
        return Some(format!(
            "match self.input.cp_at(p) {{Some((c,n)) if {{{condition}}}=>Some(n),_=>None}}"
        ));
    }
    Some(match &expr.node {
        ExprNode::Seq { items } => {
            let mut s = String::from("let start=p;\n");
            for item in items {
                writeln!(s, "p+=self.s{}(p)?;", g.expr[item.as_str()]).unwrap();
            }
            s.push_str("Some(p-start)");
            s
        }
        ExprNode::Group { child } => format!("self.s{}(p)", g.expr[child.as_str()]),
        ExprNode::RuleRef { symbol_id } => {
            let rule = &g.ir.semantics.rules[g.rules[symbol_id.as_str()]];
            format!("self.s{}(p)", g.expr[rule.body.as_str()])
        }
        ExprNode::Optional { child } => {
            format!("Some(self.s{}(p).unwrap_or(0))", g.expr[child.as_str()])
        }
        ExprNode::Choice { alternatives, .. } => {
            let plan = g.ir.plan.choices.iter().find(|p| p.expr_id == expr.id);
            let order = plan
                .map(|p| &p.candidate_order)
                .filter(|o| !o.is_empty())
                .unwrap_or(alternatives);
            let mut s = String::new();
            for alt in order {
                writeln!(
                    s,
                    "if let Some(n)=self.s{}(p) {{return Some(n);}}",
                    g.expr[alt.as_str()]
                )
                .unwrap();
            }
            s.push_str("None");
            s
        }
        ExprNode::Repeat {
            child, min, max, ..
        } => {
            let child = g.expr[child.as_str()];
            let counted = *min > 0 || max.is_some();
            // 一括走査: 1 文字で確定して 1 code point だけ消費する部分集合は、
            // 表引きだけの内側ループで食べる（ASCII は byte 1 個の判定）。上限つきの
            // 反復には出さない（周回数を数えながら回すので表引きの得が出ない）。
            let table = max.is_none().then(|| scans.classes.get(&i)).flatten();
            // 表が本体の成功をすべて覆うなら、一括走査が止まった時点で本体も失敗するので、
            // 外側の周回そのものが要らない。
            let complete = table.is_some_and(|class| class.complete);
            let mut s = String::from("let start=p;\n");
            if counted {
                s.push_str("let mut count=0usize;\n");
            }
            let bulk = table.map(|class| {
                let word = word_failure_mask(class).map(|condition| format!(
                    "if first<0x80 {{if (if first<64 {{{:#x}u64>>first}} else {{{:#x}u64>>(first-64)}})&1==0 {{break;}}\nwhile let Some(chunk)=bytes.get(p..).and_then(|tail|tail.get(..8)) {{\nlet w=u64::from_le_bytes(chunk.try_into().expect(\"8 bytes\"));\nlet x=w&0x7f7f7f7f7f7f7f7fu64;let _=x;\nlet bad={condition};let n=(bad.trailing_zeros()/8) as usize;p+=n;{}if bad!=0 {{break;}}\n}}}}\n",
                    class.ascii_low, class.ascii_high,
                    if counted {"count+=n;"} else {""}
                )).unwrap_or_default();
                let byte = if word.is_empty() { "let b=first;" } else { "let Some(&b)=bytes.get(p) else {break;};" };
                format!("let bytes=self.text.as_bytes();while let Some(&first)=bytes.get(p) {{\n{word}{byte}\nif b<0x80 {{if (if b<64 {{{:#x}u64>>b}} else {{{:#x}u64>>(b-64)}})&1==0 {{break;}}p+=1;}}\nelse {{let Some((c,n))=self.input.cp_at(p) else {{break;}};if !{} {{break;}}p+=n;}}\n{}}}\n",class.ascii_low,class.ascii_high,class_condition(class),if counted {"count+=1;"} else {""})
            });
            if complete {
                s.push_str(bulk.as_deref().expect("complete implies a table"));
            } else {
                if let Some(bulk) = &bulk {
                    s.push_str("loop {\n");
                    s.push_str(bulk);
                    writeln!(s, "let Some(n)=self.s{child}(p) else {{break;}};").unwrap();
                } else {
                    writeln!(s, "while let Some(n)=self.s{child}(p) {{").unwrap();
                }
                s.push_str("if n==0 {break;}\np+=n;\n");
                if counted {
                    s.push_str("count+=1;\n");
                }
                if let Some(max) = max {
                    writeln!(s, "if count>={} {{break;}}", (*max).max(1)).unwrap();
                }
                s.push_str("}\n");
            }
            let bound = match (min, max) {
                (0, None) => None,
                (0, Some(max)) => Some(format!("count>{max}")),
                (min, None) => Some(format!("count<{min}")),
                (min, Some(max)) => Some(format!("!({min}..={max}).contains(&count)")),
            };
            if let Some(bound) = bound {
                writeln!(s, "if {bound} {{return None;}}").unwrap();
            }
            s.push_str("Some(p-start)");
            s
        }
        ExprNode::Literal {
            value,
            case_sensitive: true,
            word_boundary: false,
        } => format!(
            "if self.input.starts_with(p,{value:?}) {{Some({})}} else {{None}}",
            value.len()
        ),
        _ => return None,
    })
}

/// 領域内の全式ぶんの走査関数。生成できない節点があれば、その根は使わない。
pub(crate) fn generate(g: &Generator<'_>, scans: &mut Scans) -> String {
    let mut bodies = BTreeMap::new();
    for &i in &scans.members {
        match body(g, scans, i) {
            Some(text) => {
                bodies.insert(i, text);
            }
            None => {
                // 生成できない節点を含む根は落とす（front の証明より backend の
                // 実装範囲が狭い場合の安全側）。
                scans.roots.retain(|&root| !region(g, root).contains(&i));
            }
        }
    }
    let mut members = BTreeSet::new();
    for &root in &scans.roots {
        members.extend(region(g, root));
    }
    scans.members = members;
    let mut s = String::new();
    for (&i, text) in &bodies {
        if !scans.members.contains(&i) {
            continue;
        }
        writeln!(
            s,
            "/// D-072: 観測を残さない終端領域の走査。消費 byte 数を返す。\n#[allow(unused_mut)]\nfn s{i}(&self,mut p:usize)->Option<usize> {{\n{text}\n}}"
        )
        .unwrap();
    }
    s
}

/// `b{i}_c` の先頭に置く走査経路。`out` / `state` は生成側の変数名。
pub(crate) fn fast_path(g: &Generator<'_>, scans: &Scans, i: usize, mode: bool) -> Option<String> {
    if mode || !scans.roots.contains(&i) {
        return None;
    }
    let label = crate::expressions::label_of(g, &g.ir.semantics.expressions[i]);
    Some(format!(
        "if self.options.scan && !DIAG && !self.options.lexical && !self.options.occurrences && !out.state.invert {{\n\
         let start=out.state.consumed;\n\
         match self.s{i}(start) {{\n\
         Some(n)=>{{out.state.advance::<false>(n);if n>0 {{out.events=self.event(Event::Token{{text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,start+n]}});}}}}\n\
         None=>{{out.ok=false;out.state=state;out.diag=self.diag_fail(start,{label:?});}}\n\
         }}\n\
         }} else {{\n"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swar_predicates_match_scalar_tables_in_every_byte_lane() {
        let output = std::process::Command::new("mktemp")
            .arg("-d")
            .output()
            .unwrap();
        assert!(output.status.success());
        let dir = std::path::PathBuf::from(String::from_utf8(output.stdout).unwrap().trim());
        let mut source = String::from("fn main(){\n");
        let mut seed = 3u64;
        let mut tested = 0;
        for n in 0..256 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let lo = (seed % 128) as u8;
            let hi = lo + ((seed >> 8) % (128 - u64::from(lo))) as u8;
            let mut class = ScanClass {
                ascii_low: 0,
                ascii_high: 0,
                complete: true,
                ranges: vec![],
            };
            for b in 0..128u8 {
                if (lo..=hi).contains(&b) ^ (n % 2 == 0) {
                    if b < 64 {
                        class.ascii_low |= 1u64 << b;
                    } else {
                        class.ascii_high |= 1u64 << (b - 64);
                    }
                }
            }
            let condition = word_failure_mask(&class).unwrap();
            // 各 lane に全 256 byte。背景には許容文字・非許容文字・非 ASCII を使い、
            // 隣接 lane の carry / borrow で結果が汚染されないことも確認する。
            writeln!(source, "{{let predicate=|w:u64|{{let x=w&0x7f7f7f7f7f7f7f7fu64;let _=x;({condition}).trailing_zeros()/8}}; let low={:#x}u64;let high={:#x}u64;for background in [0u8,{lo},{hi},127,128,255] {{for lane in 0..8 {{for b in 0..=255u8 {{let mut bytes=[background;8];bytes[lane]=b;let expected=bytes.iter().take_while(|&&v|v<128 && (if v<64 {{low>>v}} else {{high>>(v-64)}})&1!=0).count() as u32;assert_eq!(predicate(u64::from_le_bytes(bytes)),expected,\"class={n} lane={{lane}} byte={{b}} background={{background}}\");}}}}}}}}",class.ascii_low,class.ascii_high).unwrap();
            tested += 1;
        }
        source.push_str("}\n");
        std::fs::write(dir.join("check.rs"), source).unwrap();
        let compiled = std::process::Command::new("rustc")
            .args(["--edition=2021", "check.rs", "-o", "check"])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let run = std::process::Command::new(dir.join("check"))
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        std::fs::remove_dir_all(dir).unwrap();
        assert_eq!(tested, 256);
    }
}
