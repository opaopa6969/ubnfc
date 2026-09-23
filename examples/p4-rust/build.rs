//! 委託 API（scanner 付き entry 選択）と、`counters` feature の計測フックを、
//! commit 済み生成物 `src/generated/{parser,runtime}.rs` の複製に文字列置換で加える。
//! 生成物そのものは変更しない（tests/regenerate.rs が byte 一致を検証する）。
use std::{env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("patched");
    let generated = manifest.join("src/generated");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/generated/parser.rs");
    println!("cargo:rerun-if-changed=src/generated/runtime.rs");
    println!("cargo:rerun-if-changed=src/generated/rt/diag.rs");
    let counters = env::var_os("CARGO_FEATURE_COUNTERS").is_some();
    fs::create_dir_all(&out).unwrap();

    let mut parser = fs::read_to_string(generated.join("parser.rs")).unwrap();
    let entry = entry_with_scanner(&parser);
    parser.push_str(&entry);
    let mut runtime = fs::read_to_string(generated.join("runtime.rs")).unwrap();
    let mut diag = fs::read_to_string(generated.join("rt/diag.rs")).unwrap();
    if counters {
        instrument_parser(&mut parser);
        instrument_runtime(&mut runtime);
        instrument_diag(&mut diag);
    }
    fs::create_dir_all(out.join("rt")).unwrap();
    fs::write(out.join("parser.rs"), parser).unwrap();
    fs::write(out.join("runtime.rs"), runtime).unwrap();
    fs::write(out.join("rt/diag.rs"), diag).unwrap();
    // rt::diag だけ複製し、cursor/input/memo/scope（公開 API の型）は commit 済み生成物を共有する。
    let rt_mod = format!(
        "pub use crate::generated::rt::{{cursor, input, lexcache, memo, scope}};\n\
         #[path = {diag:?}] pub mod diag;\n",
        diag = out.join("rt/diag.rs").display().to_string(),
    );
    fs::write(out.join("rt/mod.rs"), rt_mod).unwrap();
    let mod_rs = format!(
        "// build.rs が生成した複製。api/ast/metadata は commit 済み生成物と同一の型を共有する。\n\
         pub use crate::generated::{{api, ast, evaluator, metadata}};\n\
         #[path = {rt:?}] pub mod rt;\n\
         #[path = {parser:?}] mod parser;\n\
         #[path = {runtime:?}] mod runtime;\n\
         pub use parser::parse_entry_with_scanner;\n",
        rt = out.join("rt/mod.rs").display().to_string(),
        parser = out.join("parser.rs").display().to_string(),
        runtime = out.join("runtime.rs").display().to_string(),
    );
    fs::write(out.join("mod.rs"), mod_rs).unwrap();
}

/// crates/ubnfc-rust/src/driver.rs の scanner_hook と同じ置換。
/// `parse_entry_with_options` の scanner を RejectExtern から引数へ差し替えた関数を追加する。
fn entry_with_scanner(parser: &str) -> String {
    let start = parser
        .find("pub fn parse_entry_with_options(")
        .expect("entry API");
    let end = start
        + parser[start..]
            .find("\nimpl<const DIAG: bool> Session")
            .expect("entry 終端");
    parser[start..end]
        .replacen("parse_entry_with_options(", "parse_entry_with_scanner(", 1)
        .replacen(
            "options:ParseOptions)",
            "options:ParseOptions,scanner:&mut dyn TokenScanner)",
            1,
        )
        .replacen("let mut scanner=RejectExtern;\n", "", 1)
        .replace("&mut scanner", "&mut *scanner")
}

/// 行頭 `fn rN_c(` / `fn rN_m(` / `fn eN_c(` / `fn eN_m(` の本体先頭にフックを挿入し、
/// 文の位置にある display 呼出しの前に計数を挿入する。
fn instrument_parser(parser: &mut String) {
    let signature = "(&mut self,state:State)->Step {";
    let mut out = String::with_capacity(parser.len() + parser.len() / 8);
    for line in parser.split_inclusive('\n') {
        let mut line = line.to_owned();
        if let Some(rest) = line.strip_prefix("fn guard_e") {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            let tail = &rest[digits.len()..];
            if !digits.is_empty()
                && (tail.starts_with("_c") || tail.starts_with("_m"))
                && tail[2..].starts_with(signature)
            {
                let head_len = 10 + digits.len() + 2 + signature.len();
                line.insert_str(head_len, " let _guard=crate::counters::Guard::guard();");
            }
        } else if let Some(rest) = line
            .strip_prefix("fn r")
            .or_else(|| line.strip_prefix("fn e"))
        {
            let kind = line.as_bytes()[3];
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            let tail = &rest[digits.len()..];
            if !digits.is_empty()
                && (tail.starts_with("_c") || tail.starts_with("_m"))
                && tail[2..].starts_with(signature)
            {
                let match_only = tail.starts_with("_m");
                let head_len = 4 + digits.len() + 2 + signature.len();
                let hook = if kind == b'r' {
                    format!(" let _guard=crate::counters::Guard::rule({digits},{match_only});")
                } else {
                    format!(" crate::counters::expression({digits},{match_only});")
                };
                line.insert_str(head_len, &hook);
            }
        }
        for (call, key) in [
            ("self.display_enter(", "DISPLAY_ENTER"),
            ("self.display_failures(", "DISPLAY_FAILURES"),
            ("self.display_reach(", "DISPLAY_REACH"),
            ("self.display_leave(", "DISPLAY_LEAVE"),
        ] {
            line = statement_hook(&line, call, key);
        }
        out.push_str(&line);
    }
    *parser = out;
}

/// `call` が文の先頭（直前が `;` `{` `}` か行頭）にある箇所だけ計数を前置する。
fn statement_hook(line: &str, call: &str, key: &str) -> String {
    let mut out = String::with_capacity(line.len() + 64);
    let mut rest = line;
    while let Some(index) = rest.find(call) {
        let prefix = &rest[..index];
        let statement = prefix.is_empty()
            || prefix.ends_with(';')
            || prefix.ends_with('{')
            || prefix.ends_with('}');
        out.push_str(prefix);
        if statement {
            out.push_str(&format!("crate::counters::bump(crate::counters::{key});"));
        }
        out.push_str(call);
        rest = &rest[index + call.len()..];
    }
    out.push_str(rest);
    out
}

/// runtime.rs の Session メソッド本体の先頭に計数を挿入する。
fn instrument_runtime(runtime: &mut String) {
    for (name, key) in [
        ("limit", "LIMIT"),
        ("enter_rule", "ENTER_RULE"),
        ("mark", "MARK"),
        ("restore", "RESTORE"),
        ("save_effects", "SAVE_EFFECTS"),
        ("replay_effects", "REPLAY_EFFECTS"),
        ("effect_at", "EFFECT_AT"),
        ("lookup", "LOOKUP"),
        ("store", "STORE"),
        ("event", "EVENT"),
        ("join", "JOIN"),
        ("combine", "COMBINE"),
        ("fail", "FAIL"),
        ("primitive", "PRIMITIVE"),
        ("literal", "LITERAL"),
        ("any", "ANY"),
        ("eof", "EOF"),
        ("number", "NUMBER"),
        ("identifier", "IDENTIFIER"),
        ("quoted", "QUOTED"),
        ("until", "UNTIL"),
        ("scan", "SCAN"),
        ("mapped_text", "MAPPED_TEXT"),
        ("node", "NODE"),
    ] {
        let needle_plain = format!("    pub fn {name}(");
        let needle_generic = format!("    pub fn {name}<");
        let start = runtime
            .find(&needle_plain)
            .or_else(|| runtime.find(&needle_generic))
            .unwrap_or_else(|| panic!("runtime.rs に {name} がない"));
        let body = start + runtime[start..].find("{\n").expect("本体") + 2;
        runtime.insert_str(
            body,
            &format!("        crate::counters::bump(crate::counters::{key});\n"),
        );
    }
    let hit = "        self.statistics.memo_hits += 1;\n";
    let index = runtime.find(hit).expect("memo_hits");
    runtime.insert_str(
        index + hit.len(),
        "        crate::counters::bump(crate::counters::MEMO_HIT);\n",
    );
    let insert = "        self.memo.insert(key, Saved::new(";
    let index = runtime.find(insert).expect("memo.insert");
    runtime.insert_str(
        index,
        "        crate::counters::bump(crate::counters::MEMO_INSERT);\n",
    );
}

/// rt/diag.rs: 診断 DAG の node 追加数、join/rule 呼出し、display frame の走査回数と最大深さ。
fn instrument_diag(diag: &mut String) {
    let hooks = [
        (
            "    fn push(&mut self, pos: usize, node: Node) -> Diag {\n",
            "        crate::counters::bump(crate::counters::DIAG_PUSH);\n",
        ),
        (
            "    pub fn join(&mut self, a: Diag, b: Diag) -> Diag {\n",
            "        crate::counters::bump(crate::counters::DIAG_JOIN);\n",
        ),
        (
            "    pub fn rule(&mut self, rule: usize, child: Diag) -> Diag {\n",
            "        crate::counters::bump(crate::counters::DIAG_RULE);\n",
        ),
        (
            "    pub fn reach(&mut self, reached: usize) {\n",
            "        crate::counters::add(crate::counters::FRAME_ITER, 1);\n",
        ),
        (
            "    pub fn merge(&mut self, snapshot: DisplaySnapshot) {\n",
            "        crate::counters::add(crate::counters::FRAME_ITER, 1);\n",
        ),
        (
            "    pub fn enter(&mut self, reached: usize) {\n",
            "        crate::counters::max(crate::counters::MAX_FRAMES, self.frames.len() as u64 + 1);\n",
        ),
    ];
    for (signature, hook) in hooks {
        let index = diag
            .find(signature)
            .unwrap_or_else(|| panic!("rt/diag.rs に {signature:?} がない"));
        diag.insert_str(index + signature.len(), hook);
    }
}
