// driver 専用 JSON。数値の字句を保持し、id を f64 に丸めない。
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}
impl Json {
    pub fn object(fields: impl IntoIterator<Item = (impl Into<String>, Json)>) -> Self {
        Self::Object(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
    pub fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(map) => map.get(key),
            _ => None,
        }
    }
    pub fn string(&self) -> Result<&str, &'static str> {
        match self {
            Self::String(s) => Ok(s),
            _ => Err("expected string"),
        }
    }
    pub fn boolean(&self) -> Result<bool, &'static str> {
        match self {
            Self::Bool(b) => Ok(*b),
            _ => Err("expected boolean"),
        }
    }
    pub fn write(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Self::Number(n) => out.push_str(n),
            Self::String(s) => {
                out.push('"');
                for ch in s.chars() {
                    match ch {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        '\u{8}' => out.push_str("\\b"),
                        '\u{c}' => out.push_str("\\f"),
                        ch if ch < ' ' => {
                            write!(out, "\\u{:04x}", ch as u32).unwrap();
                        }
                        ch => out.push(ch),
                    }
                }
                out.push('"');
            }
            Self::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write(out);
                }
                out.push(']');
            }
            Self::Object(fields) => {
                out.push('{');
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    Self::String(key.clone()).write(out);
                    out.push(':');
                    value.write(out);
                }
                out.push('}');
            }
        }
    }
}
pub trait ToJson {
    fn to_json(&self) -> Json;
}
impl ToJson for str {
    fn to_json(&self) -> Json {
        Json::String(self.into())
    }
}
impl ToJson for String {
    fn to_json(&self) -> Json {
        self.as_str().to_json()
    }
}
impl ToJson for usize {
    fn to_json(&self) -> Json {
        Json::Number(self.to_string())
    }
}
impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> Json {
        self.as_ref().map_or(Json::Null, ToJson::to_json)
    }
}
impl<T: ToJson> ToJson for Box<T> {
    fn to_json(&self) -> Json {
        self.as_ref().to_json()
    }
}
impl<T: ToJson> ToJson for [T] {
    fn to_json(&self) -> Json {
        Json::Array(self.iter().map(ToJson::to_json).collect())
    }
}
impl<T: ToJson + ?Sized> ToJson for &T {
    fn to_json(&self) -> Json {
        (*self).to_json()
    }
}

pub fn read(text: &str) -> Result<Json, &'static str> {
    let mut p = Reader { text, pos: 0 };
    let value = p.value(0)?;
    p.space();
    if p.pos != text.len() {
        return Err("trailing JSON input");
    }
    Ok(value)
}
struct Reader<'a> {
    text: &'a str,
    pos: usize,
}
impl Reader<'_> {
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }
    fn take(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }
    fn value(&mut self, depth: usize) -> Result<Json, &'static str> {
        if depth > 128 {
            return Err("JSON nesting limit");
        }
        self.space();
        match self.peek() {
            Some(b'"') => Ok(Json::String(self.string()?)),
            Some(b'[') => {
                self.pos += 1;
                let mut values = Vec::new();
                self.space();
                if self.take(b']') {
                    return Ok(Json::Array(values));
                }
                loop {
                    values.push(self.value(depth + 1)?);
                    self.space();
                    if self.take(b']') {
                        break;
                    }
                    if !self.take(b',') {
                        return Err("expected comma in array");
                    }
                }
                Ok(Json::Array(values))
            }
            Some(b'{') => {
                self.pos += 1;
                let mut fields = BTreeMap::new();
                self.space();
                if self.take(b'}') {
                    return Ok(Json::Object(fields));
                }
                loop {
                    self.space();
                    let key = self.string()?;
                    self.space();
                    if !self.take(b':') {
                        return Err("expected colon");
                    }
                    if fields.insert(key, self.value(depth + 1)?).is_some() {
                        return Err("duplicate JSON key");
                    }
                    self.space();
                    if self.take(b'}') {
                        break;
                    }
                    if !self.take(b',') {
                        return Err("expected comma in object");
                    }
                }
                Ok(Json::Object(fields))
            }
            Some(b'n') => {
                self.literal("null")?;
                Ok(Json::Null)
            }
            Some(b't') => {
                self.literal("true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.literal("false")?;
                Ok(Json::Bool(false))
            }
            Some(b'-' | b'0'..=b'9') => {
                let start = self.pos;
                self.take(b'-');
                if !self.take(b'0') {
                    self.digits()?;
                }
                if self.take(b'.') {
                    self.digits()?;
                }
                if self.take(b'e') || self.take(b'E') {
                    if !self.take(b'+') {
                        self.take(b'-');
                    }
                    self.digits()?;
                }
                Ok(Json::Number(self.text[start..self.pos].into()))
            }
            _ => Err("invalid JSON value"),
        }
    }
    fn digits(&mut self) -> Result<(), &'static str> {
        let start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.pos == start {
            Err("expected digit")
        } else {
            Ok(())
        }
    }
    fn literal(&mut self, value: &str) -> Result<(), &'static str> {
        if self.text[self.pos..].starts_with(value) {
            self.pos += value.len();
            Ok(())
        } else {
            Err("invalid literal")
        }
    }
    fn hex(&mut self) -> Result<u32, &'static str> {
        let mut n = 0;
        for _ in 0..4 {
            let digit = self
                .peek()
                .and_then(|b| (b as char).to_digit(16))
                .ok_or("invalid unicode escape")?;
            self.pos += 1;
            n = n * 16 + digit;
        }
        Ok(n)
    }
    fn string(&mut self) -> Result<String, &'static str> {
        if !self.take(b'"') {
            return Err("expected string");
        }
        let mut out = String::new();
        loop {
            let ch = self.text[self.pos..]
                .chars()
                .next()
                .ok_or("unterminated string")?;
            self.pos += ch.len_utf8();
            match ch {
                '"' => return Ok(out),
                '\\' => {
                    let escape = self.peek().ok_or("unterminated escape")?;
                    self.pos += 1;
                    out.push(match escape {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let mut code = self.hex()?;
                            if (0xd800..=0xdbff).contains(&code) {
                                self.literal("\\u")?;
                                let low = self.hex()?;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err("invalid surrogate pair");
                                }
                                code = 0x10000 + ((code - 0xd800) << 10) + low - 0xdc00;
                            }
                            char::from_u32(code).ok_or("invalid unicode scalar")?
                        }
                        _ => return Err("invalid escape"),
                    });
                }
                ch if ch < ' ' => return Err("unescaped control character"),
                ch => out.push(ch),
            }
        }
    }
}
