//! # Lesson 01: Fuzzing Fundamentals — cargo-fuzz and libFuzzer
//!
//! ## What is Fuzzing?
//!
//! Fuzzing is an automated testing technique that provides random, unexpected, or
//! malformed inputs to a program to find crashes, assertion failures, memory errors,
//! and security vulnerabilities. It is one of the most effective techniques for
//! discovering bugs in parsers, protocol handlers, and cryptographic code.
//!
//! ## cargo-fuzz Architecture
//!
//! ```
//! cargo-fuzz (Rust CLI)
//!   └── libFuzzer (C engine, LLVM)
//!        ├── Generates/mutates inputs
//!        ├── Instruments code for coverage
//!        ├── Tracks which inputs explore new paths
//!        └── Shrinks crashing inputs to minimal reproducers
//! ```
//!
//! ## Anatomy of a Fuzz Target
//!
//! ```rust,ignore
//! // fuzz_targets/fuzz_parse.rs
//! #![no_main]
//! use libfuzzer_sys::fuzz_target;
//!
//! fuzz_target!(|data: &[u8]| {
//!     // This function receives arbitrary bytes from the fuzzer.
//!     // It must NEVER panic on any input.
//!     let _ = my_parser::parse(data);
//! });
//! ```
//!
//! ## Security Perspective
//!
//! ### Attack: Crash via Malformed Input
//! Attackers send crafted inputs to public-facing services. If the parser panics or
//! has undefined behavior, it can cause denial-of-service or code execution.
//!
//! ### Defense: Fuzz Your Parsers
//! Every parser that handles untrusted input MUST be fuzzed. Run cargo-fuzz for at
//! least 24 hours before shipping a parser to production.
//!
//! ### Audit Checklist
//! - [ ] Every parser has a fuzz target
//! - [ ] Fuzz targets run in CI (even briefly)
//! - [ ] Crashes are captured as regression tests
//! - [ ] Corpus is checked into version control

/// A simple parser that reads a length-prefixed message from bytes.
///
/// Format: [length: u8][payload: length bytes]
///
/// Returns the payload as a Vec<u8>.
///
/// Hints:
/// - Return an error (don't panic!) if data is empty
/// - Return an error if data[0] > remaining bytes
/// - Use `data.get(1..1+len)` for safe slicing (returns Option)
pub fn parse_length_prefixed(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    todo!("Implement a safe length-prefixed parser")
}

/// A parser for a simple key-value format.
///
/// Format: [key_len: u8][key: key_len bytes][value_len: u8][value: value_len bytes]
///
/// Returns (key, value) as a tuple of Vec<u8>.
///
/// Hints:
/// - Parse key_len from data[0]
/// - Parse key from data[1..1+key_len]
/// - Parse value_len from data[1+key_len]
/// - Parse value from remaining bytes
/// - Return errors at each step if bounds are exceeded
pub fn parse_key_value(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), &'static str> {
    todo!("Implement a safe key-value parser")
}

/// A "fuzz-ready" wrapper that catches panics from an arbitrary closure.
///
/// This simulates what a fuzz harness does: call code with arbitrary input
/// and ensure it doesn't panic.
///
/// Hints:
/// - Use `std::panic::catch_unwind` to catch panics
/// - Return Ok(result) on success
/// - Return Err(message) on panic
pub fn fuzz_catch<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    todo!("Implement panic-catching fuzz wrapper")
}

/// Parse an unsigned 32-bit integer from a byte slice (big-endian).
///
/// Must handle slices shorter than 4 bytes gracefully.
///
/// Hints:
/// - Check length first, return error if < 4
/// - Use `u32::from_be_bytes` with a fixed-size array
/// - Copy bytes into a `[u8; 4]` array
pub fn parse_u32_be(data: &[u8]) -> Result<u32, &'static str> {
    todo!("Implement safe u32 big-endian parser")
}

/// Parse a sequence of length-prefixed messages.
///
/// Format: [count: u8][msg1_len: u8][msg1...][msg2_len: u8][msg2...]...
///
/// Returns a Vec of message payloads.
///
/// Hints:
/// - Read count from data[0]
/// - Loop count times, parsing each length-prefixed message
/// - Track current position with an index variable
/// - Return errors if data runs out
pub fn parse_message_sequence(data: &[u8]) -> Result<Vec<Vec<u8>>, &'static str> {
    todo!("Implement message sequence parser")
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
        // Empty input should return error, not panic
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
        // Length says 5 but only 2 bytes follow
        let data = [5, b'a', b'b'];
        assert!(parse_length_prefixed(&data).is_err());
    }

    // --- parse_key_value tests ---

    #[test]
    fn test_key_value_basic() {
        let mut data = vec![3, b'k', b'e', b'y', 5, b'v', b'a', b'l', b'u', b'e'];
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
        // Key length says 10 but only 2 bytes available
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
        let data = [0]; // count = 0
        let msgs = parse_message_sequence(&data).unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_message_sequence_truncated() {
        // Count says 3 but data only has enough for 1 message
        let data = [3, 2, b'a', b'b'];
        assert!(parse_message_sequence(&data).is_err());
    }

    #[test]
    fn test_no_panics_on_arbitrary_bytes() {
        // Simulate what a fuzzer does: try many random byte sequences
        let test_inputs: Vec<&[u8]> = vec![
            b"",
            b"\x00",
            b"\xff",
            b"\xff\xff\xff\xff",
            b"\x00\x00\x00\x00\x00\x00",
            b"\xff\x00\xff\x00\xff",
        ];
        for input in test_inputs {
            // These must not panic -- errors are OK
            let _ = parse_length_prefixed(input);
            let _ = parse_key_value(input);
            let _ = parse_u32_be(input);
            let _ = parse_message_sequence(input);
        }
    }
}
