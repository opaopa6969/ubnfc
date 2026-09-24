//! Grammar IR v1 から、外部 runtime に依存しない Rust モジュールを生成する。
mod ast;
mod diagnostics;
mod driver;
mod emit;
mod evaluator;
mod expressions;
mod factor;
mod lexical;
mod mapping;
mod metadata;
mod predict;
mod scan;
mod tokens;
mod validate;

use std::{collections::BTreeMap, fmt, path::Path};
use ubnfc_front::ir::GrammarIr;

#[derive(Clone, Debug)]
pub struct EmitOptions {
    /// rustfmt は呼ばず、決定的なソースを返す。呼び出し側が必要に応じて整形する。
    pub documentation: Option<String>,
    /// JSON Lines driver を生成する（既定 true）。
    pub emit_driver: bool,
}

impl Default for EmitOptions {
    fn default() -> Self {
        Self {
            documentation: None,
            emit_driver: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedCrate {
    /// out をモジュールディレクトリとして利用する。crate root にも mod.rs を指定できる。
    pub files: BTreeMap<String, String>,
}
impl GeneratedCrate {
    pub fn write_to(&self, directory: impl AsRef<Path>) -> std::io::Result<()> {
        for (name, source) in &self.files {
            let path = directory.as_ref().join(name);
            std::fs::create_dir_all(path.parent().expect("generated relative path"))?;
            std::fs::write(path, source)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmitError {
    InvalidIr(String),
    Unsupported {
        feature: String,
        detail: String,
    },
    ResourceLimit {
        resource: &'static str,
        limit: usize,
    },
}
impl fmt::Display for EmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIr(message) => write!(f, "invalid Grammar IR: {message}"),
            Self::Unsupported { feature, detail } => {
                write!(f, "unsupported IR feature {feature}: {detail}")
            }
            Self::ResourceLimit { resource, limit } => {
                write!(f, "generation resource limit: {resource} ({limit})")
            }
        }
    }
}
impl std::error::Error for EmitError {}

pub fn emit(ir: &GrammarIr, options: &EmitOptions) -> Result<GeneratedCrate, EmitError> {
    validate::budget(ir)?;
    validate::validate(ir)?;
    emit::generate(ir, options)
}
