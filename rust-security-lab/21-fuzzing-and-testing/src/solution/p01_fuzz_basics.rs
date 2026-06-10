//! # Lesson 01: Fuzzing Fundamentals — cargo-fuzz and libFuzzer (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// A simple parser that reads a length-prefixed message from bytes.
///
/// Format: [length: u8][payload: length bytes]
pub fn parse_length_prefixed(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.is_empty() {
        return Err("empty input");
    }
    let len = data[0] as usize;
    let payload = data
        .get(1..1 + len)
        .ok_or("truncated payload")?;
    Ok(payload.to_vec())
}

/// A parser for a simple key-value format.
///
/// Format: [key_len: u8][key: key_len bytes][value_len: u8][value: value_len bytes]
pub fn parse_key_value(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), &'static str> {
    if data.is_empty() {
        return Err("empty input");
    }
    let key_len = data[0] as usize;
    let key = data
        .get(1..1 + key_len)
        .ok_or("truncated key")?;

    let val_offset = 1 + key_len;
    if data.len() <= val_offset {
        return Err("missing value length");
    }
    let val_len = data[val_offset] as usize;
    let val_start = val_offset + 1;
    let value = data
        .get(val_start..val_start + val_len)
        .ok_or("truncated value")?;

    Ok((key.to_vec(), value.to_vec()))
}

/// A "fuzz-ready" wrapper that catches panics from an arbitrary closure.
pub fn fuzz_catch<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(result) => Ok(result),
        Err(panic_info) => {
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            Err(msg)
        }
    }
}

/// Parse an unsigned 32-bit integer from a byte slice (big-endian).
pub fn parse_u32_be(data: &[u8]) -> Result<u32, &'static str> {
    if data.len() < 4 {
        return Err("need at least 4 bytes");
    }
    let bytes = [data[0], data[1], data[2], data[3]];
    Ok(u32::from_be_bytes(bytes))
}

/// Parse a sequence of length-prefixed messages.
///
/// Format: [count: u8][msg1_len: u8][msg1...][msg2_len: u8][msg2...]...
pub fn parse_message_sequence(data: &[u8]) -> Result<Vec<Vec<u8>>, &'static str> {
    if data.is_empty() {
        return Err("empty input");
    }
    let count = data[0] as usize;
    let mut messages = Vec::with_capacity(count);
    let mut pos = 1usize;

    for _ in 0..count {
        if pos >= data.len() {
            return Err("truncated message sequence");
        }
        let msg_len = data[pos] as usize;
        pos += 1;
        let msg = data
            .get(pos..pos + msg_len)
            .ok_or("truncated message payload")?;
        messages.push(msg.to_vec());
        pos += msg_len;
    }

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_length_prefixed tests ---

    #[test]
    fn test_length_prefixed_basic() {
        let data = [3, b'h', b'e', b'l'];
        let result = parse_length_prefixed(&data).unwrap();
        assert_eq!(result, b"hel");
    }

    #[test]
    fn test_length_prefixed_empty() {
        assert!(parse_length_prefixed(b"").is_err());
    }

    #[test]
    fn test_length_prefixed_zero_length() {
        let data = [0];
        let result = parse_length_prefixed(&data).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_length_prefixed_truncated() {
        let data = [5, b'a', b'b'];
        assert!(parse_length_prefixed(&data).is_err());
    }

    // --- parse_key_value tests ---

    #[test]
    fn test_key_value_basic() {
        let data = vec![3, b'k', b'e', b'y', 5, b'v', b'a', b'l', b'u', b'e'];
        let (key, value) = parse_key_value(&data).unwrap();
        assert_eq!(key, b"key");
        assert_eq!(value, b"value");
    }

    #[test]
    fn test_key_value_empty_key() {
        let data = [0, 3, b'a', b'b', b'c'];
        let (key, value) = parse_key_value(&data).unwrap();
        assert!(key.is_empty());
        assert_eq!(value, b"abc");
    }

    #[test]
    fn test_key_value_truncated() {
        let data = [10, b'a', b'b'];
        assert!(parse_key_value(&data).is_err());
    }

    // --- fuzz_catch tests ---

    #[test]
    fn test_fuzz_catch_no_panic() {
        let result = fuzz_catch(|| 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_fuzz_catch_with_panic() {
        let result: Result<(), String> = fuzz_catch(|| {
            panic!("something went wrong");
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("something went wrong"));
    }

    // --- parse_u32_be tests ---

    #[test]
    fn test_parse_u32_be_basic() {
        assert_eq!(parse_u32_be(&[0, 0, 0, 1]).unwrap(), 1);
        assert_eq!(parse_u32_be(&[0, 0, 1, 0]).unwrap(), 256);
        assert_eq!(parse_u32_be(&[0xFF, 0xFF, 0xFF, 0xFF]).unwrap(), u32::MAX);
    }

    #[test]
    fn test_parse_u32_be_short() {
        assert!(parse_u32_be(&[0, 0]).is_err());
        assert!(parse_u32_be(&[]).is_err());
    }

    // --- parse_message_sequence tests ---

    #[test]
    fn test_message_sequence_basic() {
        let data = [2, 3, b'a', b'b', b'c', 2, b'x', b'y'];
        let msgs = parse_message_sequence(&data).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0], b"abc");
        assert_eq!(msgs[1], b"xy");
    }

    #[test]
    fn test_message_sequence_empty() {
        let data = [0];
        let msgs = parse_message_sequence(&data).unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_message_sequence_truncated() {
        let data = [3, 2, b'a', b'b'];
        assert!(parse_message_sequence(&data).is_err());
    }

    #[test]
    fn test_no_panics_on_arbitrary_bytes() {
        let test_inputs: Vec<&[u8]> = vec![
            b"",
            b"\x00",
            b"\xff",
            b"\xff\xff\xff\xff",
            b"\x00\x00\x00\x00\x00\x00",
            b"\xff\x00\xff\x00\xff",
        ];
        for input in test_inputs {
            let _ = parse_length_prefixed(input);
            let _ = parse_key_value(input);
            let _ = parse_u32_be(input);
            let _ = parse_message_sequence(input);
        }
    }
}
