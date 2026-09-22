//! corpus の AST JSON。span は Unicode code point の半開区間。
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Value", into = "Value")]
pub enum AstValue {
    Node {
        r#type: String,
        span: [usize; 2],
        fields: BTreeMap<String, AstValue>,
    },
    Text(String),
    Int(i64),
    List(Vec<AstValue>),
    Null,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstError {
    /// RFC 6901 JSON Pointer。空文字はルート。
    pub path: String,
    pub message: String,
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}
impl std::error::Error for AstError {}

fn pointer(parent: &str, key: &str) -> String {
    format!("{parent}/{}", key.replace('~', "~0").replace('/', "~1"))
}

impl TryFrom<Value> for AstValue {
    type Error = AstError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        fn convert(value: Value, path: &str) -> Result<AstValue, AstError> {
            let error = |message: &str| AstError {
                path: path.into(),
                message: message.into(),
            };
            Ok(match value {
                Value::Null => AstValue::Null,
                Value::String(s) => AstValue::Text(s),
                Value::Number(n) => {
                    AstValue::Int(n.as_i64().ok_or_else(|| error("i64 整数が必要です"))?)
                }
                Value::Array(values) => AstValue::List(
                    values
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| convert(v, &pointer(path, &i.to_string())))
                        .collect::<Result<_, _>>()?,
                ),
                Value::Object(mut object) => {
                    // 未知のキーを黙って捨てず、canonical の範囲外として呼出側へ返す。
                    if object.len() != 3
                        || !["type", "span", "fields"]
                            .iter()
                            .all(|k| object.contains_key(*k))
                    {
                        return Err(error("node には type, span, fields のみが必要です"));
                    }
                    let kind = object.remove("type").unwrap();
                    let kind = kind
                        .as_str()
                        .ok_or_else(|| AstError {
                            path: pointer(path, "type"),
                            message: "文字列が必要です".into(),
                        })?
                        .to_owned();
                    let span: [usize; 2] = serde_json::from_value(object.remove("span").unwrap())
                        .map_err(|e| AstError {
                        path: pointer(path, "span"),
                        message: e.to_string(),
                    })?;
                    if span[0] > span[1] {
                        return Err(AstError {
                            path: pointer(path, "span"),
                            message: "開始位置が終了位置を超えています".into(),
                        });
                    }
                    let fields = object.remove("fields").unwrap();
                    let Value::Object(fields) = fields else {
                        return Err(AstError {
                            path: pointer(path, "fields"),
                            message: "object が必要です".into(),
                        });
                    };
                    let fields = fields
                        .into_iter()
                        .map(|(k, v)| {
                            convert(v, &pointer(&pointer(path, "fields"), &k)).map(|v| (k, v))
                        })
                        .collect::<Result<_, _>>()?;
                    AstValue::Node {
                        r#type: kind,
                        span,
                        fields,
                    }
                }
                Value::Bool(_) => return Err(error("boolean は canonical AST 値ではありません")),
            })
        }
        convert(value, "")
    }
}

impl TryFrom<&Value> for AstValue {
    type Error = AstError;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Self::try_from(value.clone())
    }
}

impl From<AstValue> for Value {
    fn from(value: AstValue) -> Self {
        match value {
            AstValue::Node {
                r#type,
                span,
                fields,
            } => serde_json::json!({
                "type": r#type, "span": span, "fields": fields,
            }),
            AstValue::Text(s) => Value::String(s),
            AstValue::Int(n) => n.into(),
            AstValue::List(values) => Value::Array(values.into_iter().map(Value::from).collect()),
            AstValue::Null => Value::Null,
        }
    }
}

impl From<&AstValue> for Value {
    fn from(value: &AstValue) -> Self {
        value.clone().into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Difference {
    /// RFC 6901 JSON Pointer。配列の添字・span の両端も個別に報告。
    pub path: String,
    /// None はキー/要素の欠落、Some(Null) は明示的 null。
    pub expected: Option<Value>,
    pub actual: Option<Value>,
}

/// 共通の構造は再帰比較し、欠落した部分木や型違いはその位置で一件として報告する。
pub fn diff(expected: &AstValue, actual: &AstValue) -> Vec<Difference> {
    fn walk(path: &str, expected: &Value, actual: &Value, out: &mut Vec<Difference>) {
        match (expected, actual) {
            (Value::Object(e), Value::Object(a)) => {
                for key in e.keys().chain(a.keys()).collect::<BTreeSet<_>>() {
                    compare(&pointer(path, key), e.get(key), a.get(key), out);
                }
            }
            (Value::Array(e), Value::Array(a)) => {
                for i in 0..e.len().max(a.len()) {
                    compare(&pointer(path, &i.to_string()), e.get(i), a.get(i), out);
                }
            }
            _ if expected != actual => out.push(Difference {
                path: path.into(),
                expected: Some(expected.clone()),
                actual: Some(actual.clone()),
            }),
            _ => {}
        }
    }
    fn compare(path: &str, e: Option<&Value>, a: Option<&Value>, out: &mut Vec<Difference>) {
        match (e, a) {
            (Some(e), Some(a)) => walk(path, e, a, out),
            _ => out.push(Difference {
                path: path.into(),
                expected: e.cloned(),
                actual: a.cloned(),
            }),
        }
    }
    let mut out = Vec::new();
    walk("", &expected.into(), &actual.into(), &mut out);
    out
}
