//! stax の生成 typed AST を検査し、上限付きで評価する。JVM は使用しない。
include!(concat!(env!("OUT_DIR"), "/parser.rs"));
mod checker;
mod machine;
mod model;
mod modules;
pub use machine::Limits;
pub use model::{Error, Result};
pub use modules::Program;
