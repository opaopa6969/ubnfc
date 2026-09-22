//! Grammar IR v1（`ir/schema/grammar-ir.v1.schema.json`）。
//!
//! `from_json` は serde の構造・必須キー・判別子・定数を検査する。
//! 横断的な ID 参照と機能宣言の検査は `validate_references` で明示的に行う。
//! `to_canonical_json` は値の順序を保って ubnfc-json.v1 形式へ直列化する。

mod constants;
mod json;
mod references;
mod types;

pub use constants::*;
pub use json::{from_json, to_canonical_json};
pub use references::validate_references;
pub use types::*;

use serde::{Deserialize, Serialize};
use std::fmt;

/// 論理 sourceId、code point 区間、構文上の role から構成する安定 ID。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ExprId(pub String);

impl ExprId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ExprId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<_> = value.split(':').collect();
        let valid = parts.len() == 5
            && parts[0] == "expr"
            && !parts[1].is_empty()
            && parts[1]
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._~%/-".contains(&c))
            && parts[2..4]
                .iter()
                .all(|s| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit()))
            && !parts[4].is_empty()
            && parts[4]
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._/-".contains(&c));
        if valid {
            Ok(Self(value))
        } else {
            Err(format!("invalid ExprId: {value}"))
        }
    }
}

impl From<ExprId> for String {
    fn from(value: ExprId) -> Self {
        value.0
    }
}

impl fmt::Display for ExprId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// JSON の構造エラー、または JSON Pointer で場所を示す横断検証エラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrError {
    Json {
        line: usize,
        column: usize,
        message: String,
    },
    Validation {
        path: String,
        message: String,
    },
}

impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json {
                line,
                column,
                message,
            } => write!(f, "{line}:{column}: {message}"),
            Self::Validation { path, message } => write!(f, "{path}: {message}"),
        }
    }
}

impl std::error::Error for IrError {}

impl From<serde_json::Error> for IrError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json {
            line: error.line(),
            column: error.column(),
            message: error.to_string(),
        }
    }
}
