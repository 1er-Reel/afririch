// ===== AFRI-JSON — Notre propre parser/serializer JSON from scratch =====
// 100% souverain — zéro dépendance externe
// Type JsonValue + parser (string -> JsonValue) + serializer (JsonValue -> string)
// Compatible avec le format JSON standard

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Int(i) => Some(*i),
            JsonValue::UInt(u) => Some(*u as i64),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            JsonValue::UInt(u) => Some(*u),
            JsonValue::Int(i) => Some(*i as u64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Float(f) => Some(*f),
            JsonValue::Int(i) => Some(*i as f64),
            JsonValue::UInt(u) => Some(*u as f64),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }
}

// ===== SERIALIZER =====

pub fn to_string(value: &JsonValue) -> String {
    let mut out = String::new();
    serialize_value(value, &mut out, false, 0);
    out
}

pub fn to_string_pretty(value: &JsonValue) -> String {
    let mut out = String::new();
    serialize_value(value, &mut out, true, 0);
    out
}

fn serialize_value(value: &JsonValue, out: &mut String, pretty: bool, indent: usize) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        JsonValue::Int(i) => out.push_str(&i.to_string()),
        JsonValue::UInt(u) => out.push_str(&u.to_string()),
        JsonValue::Float(f) => {
            if f.is_finite() {
                out.push_str(&f.to_string());
            } else {
                out.push_str("null");
            }
        }
        JsonValue::Str(s) => serialize_string(s, out),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in arr.iter().enumerate() {
                if pretty {
                    out.push('\n');
                    push_indent(out, indent + 1);
                } else if i > 0 {
                    out.push(',');
                }
                if pretty && i > 0 { /* already indented */ }
                if !pretty && i > 0 { /* already comma */ }
                if pretty { } // handled above
                serialize_value(item, out, pretty, indent + 1);
                if pretty && i < arr.len() - 1 { out.push(','); }
                if !pretty && i < arr.len() - 1 { /* comma already pushed */ }
            }
            if pretty {
                out.push('\n');
                push_indent(out, indent);
            }
            out.push(']');
        }
        JsonValue::Object(map) => {
            if map.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            let mut sorted: Vec<_> = map.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));
            for (i, (k, v)) in sorted.iter().enumerate() {
                if pretty {
                    out.push('\n');
                    push_indent(out, indent + 1);
                } else if i > 0 {
                    out.push(',');
                }
                serialize_string(k, out);
                out.push(':');
                if pretty { out.push(' '); }
                serialize_value(v, out, pretty, indent + 1);
                if pretty && i < sorted.len() - 1 { out.push(','); }
            }
            if pretty {
                out.push('\n');
                push_indent(out, indent);
            }
            out.push('}');
        }
    }
}

fn serialize_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn push_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

// ===== PARSER =====

pub fn from_str(input: &str) -> Result<JsonValue, String> {
    let mut parser = Parser::new(input);
    parser.skip_whitespace();
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return Err(format!("Unexpected character at position {}", parser.pos));
    }
    Ok(value)
}

pub fn from_slice(input: &[u8]) -> Result<JsonValue, String> {
    let s = std::str::from_utf8(input).map_err(|e| e.to_string())?;
    from_str(s)
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input: input.as_bytes(), pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let c = self.peek();
        if c.is_some() { self.pos += 1; }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => Ok(JsonValue::Str(self.parse_string()?)),
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(b'n') => self.parse_null(),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_number(),
            _ => Err(format!("Unexpected character '{}' at position {}", self.peek().unwrap_or(b'?') as char, self.pos)),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.next(); // consume '{'
        let mut map = HashMap::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.next();
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(format!("Expected string key at position {}", self.pos));
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.next() != Some(b':') {
                return Err(format!("Expected ':' at position {}", self.pos));
            }
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.next() {
                Some(b',') => continue,
                Some(b'}') => break,
                _ => return Err(format!("Expected ',' or '}}' at position {}", self.pos)),
            }
        }
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.next(); // consume '['
        let mut arr = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.next();
            return Ok(JsonValue::Array(arr));
        }
        loop {
            let value = self.parse_value()?;
            arr.push(value);
            self.skip_whitespace();
            match self.next() {
                Some(b',') => continue,
                Some(b']') => break,
                _ => return Err(format!("Expected ',' or ']' at position {}", self.pos)),
            }
        }
        Ok(JsonValue::Array(arr))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.next(); // consume '"'
        let mut s = String::new();
        loop {
            match self.next() {
                Some(b'"') => break,
                Some(b'\\') => {
                    match self.next() {
                        Some(b'"') => s.push('"'),
                        Some(b'\\') => s.push('\\'),
                        Some(b'/') => s.push('/'),
                        Some(b'n') => s.push('\n'),
                        Some(b'r') => s.push('\r'),
                        Some(b't') => s.push('\t'),
                        Some(b'b') => s.push('\x08'),
                        Some(b'f') => s.push('\x0c'),
                        Some(b'u') => {
                            let mut code = 0u32;
                            for _ in 0..4 {
                                let c = self.next().ok_or("Unexpected end of string")?;
                                code = code * 16 + match c {
                                    b'0'..=b'9' => (c - b'0') as u32,
                                    b'a'..=b'f' => (c - b'a' + 10) as u32,
                                    b'A'..=b'F' => (c - b'A' + 10) as u32,
                                    _ => return Err("Invalid unicode escape".to_string()),
                                };
                            }
                            if let Some(ch) = char::from_u32(code) {
                                s.push(ch);
                            }
                        }
                        _ => return Err("Invalid escape sequence".to_string()),
                    }
                }
                Some(c) => {
                    // Handle UTF-8 multi-byte
                    if c < 0x80 {
                        s.push(c as char);
                    } else {
                        // Multi-byte UTF-8
                        let mut bytes = vec![c];
                        let extra = if c >= 0xF0 { 3 } else if c >= 0xE0 { 2 } else { 1 };
                        for _ in 0..extra {
                            if let Some(b) = self.next() {
                                bytes.push(b);
                            } else {
                                return Err("Unexpected end of UTF-8 sequence".to_string());
                            }
                        }
                        if let Ok(st) = std::str::from_utf8(&bytes) {
                            s.push_str(st);
                        }
                    }
                }
                None => return Err("Unexpected end of string".to_string()),
            }
        }
        Ok(s)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(format!("Invalid boolean at position {}", self.pos))
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(format!("Invalid null at position {}", self.pos))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut is_float = false;
        if self.peek() == Some(b'-') { self.pos += 1; }
        while let Some(c) = self.peek() {
            match c {
                b'0'..=b'9' => self.pos += 1,
                b'.' | b'e' | b'E' | b'+' | b'-' => { is_float = true; self.pos += 1; }
                _ => break,
            }
        }
        let num_str = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|e| e.to_string())?;
        if is_float {
            num_str.parse::<f64>()
                .map(JsonValue::Float)
                .map_err(|e| e.to_string())
        } else if num_str.starts_with('-') {
            num_str.parse::<i64>()
                .map(JsonValue::Int)
                .map_err(|e| e.to_string())
        } else {
            num_str.parse::<u64>()
                .map(JsonValue::UInt)
                .map_err(|e| e.to_string())
        }
    }
}

// ===== Helper macros for building JSON =====

pub fn obj() -> HashMap<String, JsonValue> {
    HashMap::new()
}

pub fn arr() -> Vec<JsonValue> {
    Vec::new()
}
