//! # Fuzzing
//!
//! Fuzzing tests code with random, unexpected, or malformed inputs to find
//! crashes, panics, and security vulnerabilities. Rust has excellent fuzzing
//! support through `cargo-fuzz` (libFuzzer) and `afl.rs`.
//!
//! Key concepts:
//! - Fuzz targets: functions that accept arbitrary bytes
//! - Corpus management: seed inputs and crash reproducers
//! - Coverage-guided fuzzing: the fuzzer learns which inputs explore new code
//! - Integration with CI/CD

/// A parser that demonstrates fuzzing-friendly design.
/// The key principle: never panic on malformed input.
#[derive(Debug, Clone, PartialEq)]
pub struct SafeParser {
    input: Vec<u8>,
    position: usize,
}

impl SafeParser {
    pub fn new(input: &[u8]) -> Self {
        SafeParser {
            input: input.to_vec(),
            position: 0,
        }
    }

    pub fn remaining(&self) -> &[u8] {
        &self.input[self.position..]
    }

    pub fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Read a single byte. Returns None at EOF.
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.position < self.input.len() {
            let byte = self.input[self.position];
            self.position += 1;
            Some(byte)
        } else {
            None
        }
    }

    /// Read a big-endian u16. Returns None if not enough bytes.
    pub fn read_u16(&mut self) -> Option<u16> {
        if self.position + 2 > self.input.len() {
            return None;
        }
        let val = u16::from_be_bytes([self.input[self.position], self.input[self.position + 1]]);
        self.position += 2;
        Some(val)
    }

    /// Read a big-endian u32. Returns None if not enough bytes.
    pub fn read_u32(&mut self) -> Option<u32> {
        if self.position + 4 > self.input.len() {
            return None;
        }
        let bytes: [u8; 4] = self.input[self.position..self.position + 4].try_into().ok()?;
        self.position += 4;
        Some(u32::from_be_bytes(bytes))
    }

    /// Read a length-prefixed string. Returns None on any error.
    pub fn read_string(&mut self) -> Option<String> {
        let len = self.read_u16()? as usize;
        if self.position + len > self.input.len() {
            return None;
        }
        let bytes = &self.input[self.position..self.position + len];
        self.position += len;
        String::from_utf8(bytes.to_vec()).ok()
    }

    /// Read a sequence of items with a count prefix.
    pub fn read_sequence<T, F>(&mut self, read_item: F) -> Option<Vec<T>>
    where
        F: Fn(&mut Self) -> Option<T>,
    {
        let count = self.read_u16()? as usize;
        let mut items = Vec::with_capacity(count.min(1024)); // Cap allocation
        for _ in 0..count {
            items.push(read_item(self)?);
        }
        Some(items)
    }
}

/// A safe JSON-like parser that never panics on malformed input.
#[derive(Debug, Clone, PartialEq)]
pub enum SafeValue {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<SafeValue>),
    Object(Vec<(String, SafeValue)>),
}

pub fn safe_parse_value(input: &str) -> Option<SafeValue> {
    let mut parser = SafeJsonParser::new(input);
    parser.parse_value()
}

struct SafeJsonParser {
    chars: Vec<char>,
    pos: usize,
}

impl SafeJsonParser {
    fn new(input: &str) -> Self {
        SafeJsonParser {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Option<SafeValue> {
        self.skip_whitespace();
        match self.peek()? {
            '"' => self.parse_string().map(SafeValue::Str),
            't' | 'f' => self.parse_bool().map(SafeValue::Bool),
            'n' => self.parse_null(),
            '-' | '0'..='9' => self.parse_number().map(SafeValue::Number),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            _ => None,
        }
    }

    fn parse_string(&mut self) -> Option<String> {
        if self.advance()? != '"' {
            return None;
        }
        let mut s = String::new();
        loop {
            match self.advance()? {
                '"' => return Some(s),
                '\\' => {
                    let escaped = self.advance()?;
                    s.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        other => other,
                    });
                }
                ch => s.push(ch),
            }
        }
    }

    fn parse_bool(&mut self) -> Option<bool> {
        if self.starts_with("true") {
            self.pos += 4;
            Some(true)
        } else if self.starts_with("false") {
            self.pos += 5;
            Some(false)
        } else {
            None
        }
    }

    fn parse_null(&mut self) -> Option<SafeValue> {
        if self.starts_with("null") {
            self.pos += 4;
            Some(SafeValue::Null)
        } else {
            None
        }
    }

    fn parse_number(&mut self) -> Option<f64> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.advance();
        }
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some('.') {
            self.advance();
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        if start == self.pos {
            return None;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse().ok()
    }

    fn parse_array(&mut self) -> Option<SafeValue> {
        if self.advance()? != '[' {
            return None;
        }
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.advance();
            return Some(SafeValue::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_whitespace();
            match self.advance()? {
                ',' => continue,
                ']' => return Some(SafeValue::Array(items)),
                _ => return None,
            }
        }
    }

    fn parse_object(&mut self) -> Option<SafeValue> {
        if self.advance()? != '{' {
            return None;
        }
        let mut entries = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some('}') {
            self.advance();
            return Some(SafeValue::Object(entries));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.advance()? != ':' {
                return None;
            }
            let value = self.parse_value()?;
            entries.push((key, value));
            self.skip_whitespace();
            match self.advance()? {
                ',' => continue,
                '}' => return Some(SafeValue::Object(entries)),
                _ => return None,
            }
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        let remaining: String = self.chars[self.pos..].iter().collect();
        remaining.starts_with(s)
    }
}

/// A fuzzing-friendly binary protocol parser.
pub fn parse_binary_packet(data: &[u8]) -> Option<BinaryPacket> {
    if data.len() < 4 {
        return None;
    }

    let magic = u16::from_be_bytes([data[0], data[1]]);
    if magic != 0xABCD {
        return None;
    }

    let packet_type = data[2];
    let payload_len = data[3] as usize;

    if data.len() < 4 + payload_len {
        return None;
    }

    let payload = &data[4..4 + payload_len];

    Some(BinaryPacket {
        packet_type,
        payload: payload.to_vec(),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryPacket {
    pub packet_type: u8,
    pub payload: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_parser_read_byte() {
        let mut parser = SafeParser::new(&[1, 2, 3]);
        assert_eq!(parser.read_byte(), Some(1));
        assert_eq!(parser.read_byte(), Some(2));
        assert_eq!(parser.read_byte(), Some(3));
        assert_eq!(parser.read_byte(), None);
    }

    #[test]
    fn test_safe_parser_read_u16() {
        let mut parser = SafeParser::new(&[0x00, 0x2A]); // 42
        assert_eq!(parser.read_u16(), Some(42));
        assert_eq!(parser.read_u16(), None);
    }

    #[test]
    fn test_safe_parser_read_u32() {
        let mut parser = SafeParser::new(&[0x00, 0x00, 0x00, 0x2A]); // 42
        assert_eq!(parser.read_u32(), Some(42));
    }

    #[test]
    fn test_safe_parser_read_string() {
        let data = b"\x00\x05hello";
        let mut parser = SafeParser::new(data);
        assert_eq!(parser.read_string(), Some("hello".to_string()));
    }

    #[test]
    fn test_safe_parser_truncated() {
        let mut parser = SafeParser::new(&[0x00]);
        assert_eq!(parser.read_u16(), None);
    }

    #[test]
    fn test_safe_json_parse_null() {
        assert_eq!(safe_parse_value("null"), Some(SafeValue::Null));
    }

    #[test]
    fn test_safe_json_parse_bool() {
        assert_eq!(safe_parse_value("true"), Some(SafeValue::Bool(true)));
        assert_eq!(safe_parse_value("false"), Some(SafeValue::Bool(false)));
    }

    #[test]
    fn test_safe_json_parse_number() {
        assert_eq!(safe_parse_value("42"), Some(SafeValue::Number(42.0)));
        assert_eq!(safe_parse_value("-3.14"), Some(SafeValue::Number(-3.14)));
    }

    #[test]
    fn test_safe_json_parse_string() {
        assert_eq!(
            safe_parse_value("\"hello\""),
            Some(SafeValue::Str("hello".to_string()))
        );
    }

    #[test]
    fn test_safe_json_parse_array() {
        let result = safe_parse_value("[1, 2, 3]");
        assert!(result.is_some());
    }

    #[test]
    fn test_safe_json_parse_object() {
        let result = safe_parse_value("{\"key\": \"value\"}");
        assert!(result.is_some());
    }

    #[test]
    fn test_safe_json_malformed() {
        // These should all return None, never panic
        assert!(safe_parse_value("").is_none());
        assert!(safe_parse_value("{").is_none());
        assert!(safe_parse_value("[1,").is_none());
        assert!(safe_parse_value("\"unterminated").is_none());
        assert!(safe_parse_value("tru").is_none());
        assert!(safe_parse_value("nul").is_none());
    }

    #[test]
    fn test_binary_packet_valid() {
        let data = [0xAB, 0xCD, 0x01, 0x03, 0x01, 0x02, 0x03];
        let packet = parse_binary_packet(&data).unwrap();
        assert_eq!(packet.packet_type, 1);
        assert_eq!(packet.payload, vec![1, 2, 3]);
    }

    #[test]
    fn test_binary_packet_invalid_magic() {
        let data = [0x00, 0x00, 0x01, 0x00];
        assert!(parse_binary_packet(&data).is_none());
    }

    #[test]
    fn test_binary_packet_truncated() {
        let data = [0xAB, 0xCD, 0x01, 0x10]; // Claims 16 bytes payload
        assert!(parse_binary_packet(&data).is_none());
    }

    #[test]
    fn test_binary_packet_empty() {
        assert!(parse_binary_packet(&[]).is_none());
    }

    #[test]
    fn test_fuzz_safety_never_panics() {
        // Simulate fuzzing with various inputs
        let inputs: Vec<&[u8]> = vec![
            &[],
            &[0],
            &[0xFF; 100],
            &[0xAB, 0xCD],
            b"not json at all",
            b"{invalid json}",
            &[0, 0, 0, 0, 0, 0, 0, 0],
        ];

        for input in inputs {
            // These should never panic
            let _ = SafeParser::new(input).read_byte();
            let _ = SafeParser::new(input).read_u16();
            let _ = SafeParser::new(input).read_u32();
            let _ = SafeParser::new(input).read_string();
            let _ = parse_binary_packet(input);
        }
    }
}
