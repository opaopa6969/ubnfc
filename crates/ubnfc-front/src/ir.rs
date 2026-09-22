//! Grammar IR（バックエンドとの契約）。スキーマは docs/design/00-design-v0.md §3。
//! 争点が確定するまで最小の骨格のみ。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarIr {
    pub version: u32,
}
