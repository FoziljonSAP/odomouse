//! Minimal JSON value, parser and writer (no dependencies).
//! Object key order is preserved, so files stay diff-friendly.

use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn obj() -> Json {
        Json::Obj(Vec::new())
    }

    /// Builder: insert or replace a key (objects only).
    pub fn set(mut self, key: &str, value: impl Into<Json>) -> Json {
        self.insert(key, value);
        self
    }

    pub fn insert(&mut self, key: &str, value: impl Into<Json>) {
        if let Json::Obj(items) = self {
            let value = value.into();
            if let Some(slot) = items.iter_mut().find(|(k, _)| k == key) {
                slot.1 = value;
            } else {
                items.push((key.to_string(), value));
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(items) => items.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_obj(&self) -> Option<&Vec<(String, Json)>> {
        match self {
            Json::Obj(o) => Some(o),
            _ => None,
        }
    }

    /// Number at `key`, or 0 if missing / not a number / negative / not finite.
    pub fn num_or_zero(&self, key: &str) -> f64 {
        match self.get(key).and_then(Json::as_f64) {
            Some(n) if n.is_finite() && n > 0.0 => n,
            _ => 0.0,
        }
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        write_value(self, &mut out);
        out
    }
}

impl From<f64> for Json {
    fn from(v: f64) -> Json {
        Json::Num(v)
    }
}
impl From<i64> for Json {
    fn from(v: i64) -> Json {
        Json::Num(v as f64)
    }
}
impl From<u64> for Json {
    fn from(v: u64) -> Json {
        Json::Num(v as f64)
    }
}
impl From<u32> for Json {
    fn from(v: u32) -> Json {
        Json::Num(v as f64)
    }
}
impl From<i32> for Json {
    fn from(v: i32) -> Json {
        Json::Num(v as f64)
    }
}
impl From<usize> for Json {
    fn from(v: usize) -> Json {
        Json::Num(v as f64)
    }
}
impl From<bool> for Json {
    fn from(v: bool) -> Json {
        Json::Bool(v)
    }
}
impl From<&str> for Json {
    fn from(v: &str) -> Json {
        Json::Str(v.to_string())
    }
}
impl From<String> for Json {
    fn from(v: String) -> Json {
        Json::Str(v)
    }
}
impl<T: Into<Json>> From<Option<T>> for Json {
    fn from(v: Option<T>) -> Json {
        match v {
            Some(x) => x.into(),
            None => Json::Null,
        }
    }
}
impl<T: Into<Json>> From<Vec<T>> for Json {
    fn from(v: Vec<T>) -> Json {
        Json::Arr(v.into_iter().map(Into::into).collect())
    }
}

// ---------------------------------------------------------------- writer

fn write_value(v: &Json, out: &mut String) {
    match v {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Num(n) => write_num(*n, out),
        Json::Str(s) => write_str(s, out),
        Json::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(item, out);
            }
            out.push(']');
        }
        Json::Obj(items) => {
            out.push('{');
            for (i, (k, item)) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_str(k, out);
                out.push(':');
                write_value(item, out);
            }
            out.push('}');
        }
    }
}

fn write_num(n: f64, out: &mut String) {
    if !n.is_finite() {
        out.push_str("null");
    } else if n.fract() == 0.0 && n.abs() < 1e15 {
        let _ = write!(out, "{}", n as i64);
    } else {
        let _ = write!(out, "{}", n);
    }
}

fn write_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

// ---------------------------------------------------------------- parser

pub fn parse(input: &str) -> Result<Json, String> {
    let mut p = Parser { s: input.as_bytes(), i: 0, depth: 0 };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.i != p.s.len() {
        return Err(format!("trailing data at {}", p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\n' | b'\r' | b'\t') {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn expect(&mut self, lit: &str) -> Result<(), String> {
        if self.s[self.i..].starts_with(lit.as_bytes()) {
            self.i += lit.len();
            Ok(())
        } else {
            Err(format!("expected {} at {}", lit, self.i))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.depth += 1;
        if self.depth > 200 {
            return Err("nesting too deep".into());
        }
        let v = match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Json::Str),
            Some(b't') => self.expect("true").map(|_| Json::Bool(true)),
            Some(b'f') => self.expect("false").map(|_| Json::Bool(false)),
            Some(b'n') => self.expect("null").map(|_| Json::Null),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.number(),
            _ => Err(format!("unexpected input at {}", self.i)),
        };
        self.depth -= 1;
        v
    }

    fn object(&mut self) -> Result<Json, String> {
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Json::Obj(items));
        }
        loop {
            self.ws();
            if self.peek() != Some(b'"') {
                return Err(format!("expected key at {}", self.i));
            }
            let k = self.string()?;
            self.ws();
            self.expect(":")?;
            self.ws();
            let v = self.value()?;
            items.push((k, v));
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Obj(items));
                }
                _ => return Err(format!("expected , or }} at {}", self.i)),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            self.ws();
            items.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(format!("expected , or ] at {}", self.i)),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        if self.i + 4 > self.s.len() {
            return Err("short \\u escape".into());
        }
        let h = std::str::from_utf8(&self.s[self.i..self.i + 4]).map_err(|e| e.to_string())?;
        let v = u32::from_str_radix(h, 16).map_err(|_| format!("bad \\u escape at {}", self.i))?;
        self.i += 4;
        Ok(v)
    }

    fn string(&mut self) -> Result<String, String> {
        self.i += 1; // opening quote
        let mut out = String::new();
        loop {
            let start = self.i;
            while self.i < self.s.len() && self.s[self.i] != b'"' && self.s[self.i] != b'\\' {
                self.i += 1;
            }
            out.push_str(std::str::from_utf8(&self.s[start..self.i]).map_err(|e| e.to_string())?);
            match self.peek() {
                Some(b'"') => {
                    self.i += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.i += 1;
                    let c = self.peek().ok_or("bad escape")?;
                    self.i += 1;
                    match c {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let mut cp = self.hex4()?;
                            if (0xD800..0xDC00).contains(&cp) && self.s[self.i..].starts_with(b"\\u") {
                                self.i += 2;
                                let lo = self.hex4()?;
                                cp = 0x10000 + ((cp - 0xD800) << 10) + (lo.wrapping_sub(0xDC00) & 0x3FF);
                            }
                            out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                        }
                        _ => return Err(format!("bad escape at {}", self.i)),
                    }
                }
                _ => return Err("unterminated string".into()),
            }
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        while self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
            self.i += 1;
        }
        let t = std::str::from_utf8(&self.s[start..self.i]).map_err(|e| e.to_string())?;
        t.parse::<f64>().map(Json::Num).map_err(|_| format!("bad number {}", t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_keeps_order_and_values() {
        let src = r#"{"b":1,"a":[true,false,null,-2.5,1e3],"s":"q\"\\\né😀"}"#;
        let v = parse(src).unwrap();
        assert_eq!(v.get("b").unwrap().as_f64(), Some(1.0));
        assert_eq!(v.get("s").unwrap().as_str(), Some("q\"\\\né😀"));
        let out = v.to_string();
        assert!(out.starts_with(r#"{"b":1,"a":[true,false,null,-2.5,1000]"#), "{}", out);
        assert_eq!(parse(&out).unwrap(), v);
    }

    #[test]
    fn rejects_garbage() {
        for bad in ["{ broken", "[1,]x", "{\"a\" 1}", "", "nul", "\"abc"] {
            assert!(parse(bad).is_err(), "{}", bad);
        }
    }

    #[test]
    fn writes_integers_without_decimal_and_nan_as_null() {
        assert_eq!(Json::Num(42.0).to_string(), "42");
        assert_eq!(Json::Num(0.25).to_string(), "0.25");
        assert_eq!(Json::Num(f64::NAN).to_string(), "null");
    }

    #[test]
    fn builder_set_replaces() {
        let v = Json::obj().set("a", 1i64).set("b", "x").set("a", 2i64);
        assert_eq!(v.to_string(), r#"{"a":2,"b":"x"}"#);
        assert_eq!(v.num_or_zero("a"), 2.0);
        assert_eq!(v.num_or_zero("b"), 0.0);
    }
}
