//! Family C: 同じ UBNF 文法に対する 3 つの Rust parser を 1 プロセスで交互に測る。
//!
//! - `ubnfc-front`（手書き lexer + 再帰下降）… AST（`UbnfFile`）のみ
//! - `ubnfc-selfhost`（`ubnf.ubnf` から生成した self-host front-end）… 認識のみ / 生成 AST / `UbnfFile` 射影
//! - `unlaxer-ubnf`（sibling repo の手書き Rust UBNF parser）… AST（`UbnfFile`）のみ
//!
//! 入力は引数（`name=path` の並び）。全 parser に同じ byte 列を渡す。
include!("bench_util.rs");

#[global_allocator]
static ALLOC: Counting = Counting;

fn main() {
    let inputs: Vec<(String, String)> = std::env::args()
        .skip(1)
        .map(|arg| {
            let (name, path) = arg.split_once('=').expect("name=path");
            (
                name.to_owned(),
                std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}")),
            )
        })
        .collect();
    header();
    for (name, text) in &inputs {
        let bytes = text.len();
        // 受理可否を先に出す（測らない実装があることを報告に書くため）。
        let front_ok = ubnfc_front::syntax::parse_file(text).is_ok();
        let self_ok = ubnfc_selfhost::parse_file(text).is_ok();
        let unlaxer_ok = unlaxer_ubnf::parse(text).is_ok();
        eprintln!("# {name} accept: front={front_ok} selfhost={self_ok} unlaxer-ubnf={unlaxer_ok}");

        if self_ok {
            let recognize = ubnfc_selfhost::generated::ParseOptions {
                lexical: true,
                build_ast: false,
                ..Default::default()
            };
            row("C", "ubnfc-selfhost", name, bytes, "認識", || {
                let r = ubnfc_selfhost::generated::parse_with_options(
                    std::hint::black_box(text.as_str()),
                    recognize,
                );
                assert!(r.ok);
                let _ = std::hint::black_box(r);
            });
            let ast = ubnfc_selfhost::generated::ParseOptions {
                lexical: true,
                build_ast: true,
                ..Default::default()
            };
            row("C", "ubnfc-selfhost", name, bytes, "AST(生成)", || {
                let r = ubnfc_selfhost::generated::parse_with_options(
                    std::hint::black_box(text.as_str()),
                    ast,
                );
                assert!(r.ok && r.ast.is_some());
                let _ = std::hint::black_box(r);
            });
            row("C", "ubnfc-selfhost", name, bytes, "AST(UbnfFile 射影)", || {
                let r = ubnfc_selfhost::parse_file(std::hint::black_box(text.as_str()));
                assert!(r.is_ok());
                let _ = std::hint::black_box(r);
            });
        }
        if front_ok {
            row("C", "ubnfc-front(手書き)", name, bytes, "AST(UbnfFile)", || {
                let r = ubnfc_front::syntax::parse_file(std::hint::black_box(text.as_str()));
                assert!(r.is_ok());
                let _ = std::hint::black_box(r);
            });
        }
        if unlaxer_ok {
            row("C", "unlaxer-ubnf(手書き)", name, bytes, "AST(UbnfFile)", || {
                let r = unlaxer_ubnf::parse(std::hint::black_box(text.as_str()));
                assert!(r.is_ok());
                let _ = std::hint::black_box(r);
            });
        }
    }
}
