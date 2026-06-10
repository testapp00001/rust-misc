//! # Lesson 05: Differential Fuzzing
//!
//! ## What is Differential Fuzzing?
//!
//! Differential fuzzing compares two (or more) implementations of the same function.
//! If they produce different outputs for the same input, at least one is buggy.
//!
//! ```
//! Input → [Implementation A] → output_a
//! Input → [Implementation B] → output_b
//!
//! assert_eq!(output_a, output_b)  // If this fails, we found a bug!
//! ```
//!
//! ## Why It Works
//!
//! Two independently-written implementations are unlikely to have the SAME bug.
//! Any divergence reveals a correctness issue in one of them.
//!
//! ## Classic Applications
//!
//! - Comparing two JSON parsers
//! - Comparing two SHA-256 implementations (ring vs sha2)
//! - Comparing an optimized vs reference implementation
//! - Comparing a Rust implementation vs a C reference
//!
//! ## Security Perspective
//!
//! ### Attack: Implementation Divergence
//! If two TLS implementations handle malformed certificates differently, an attacker
//! can exploit the one that's more permissive.
//!
//! ### Defense: Cross-Check Implementations
//! Use differential testing to ensure all implementations of a standard agree.

/// Implementation A: SHA-256 using the sha2 crate.
pub fn sha256_a(data: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Implementation B: SHA-256 using ring.
pub fn sha256_b(data: &[u8]) -> Vec<u8> {
    ring::digest::digest(&ring::digest::SHA256, data).as_ref().to_vec()
}

/// Differential test: run two SHA-256 implementations and check they agree.
///
/// Returns Ok(()) if both agree, Err with details if they diverge.
///
/// Hints:
/// - Compute sha256_a and sha256_b on the same input
/// - Compare the results
/// - Return Err with a descriptive message if they differ
pub fn diff_sha256(data: &[u8]) -> Result<(), String> {
    todo!("Implement differential SHA-256 test")
}

/// Implementation A: Hex encoding using the hex crate.
pub fn hex_encode_a(data: &[u8]) -> String {
    hex::encode(data)
}

/// Implementation B: Hex encoding using manual implementation.
pub fn hex_encode_b(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Differential test for hex encoding.
///
/// Returns Ok(()) if both agree, Err if they diverge.
///
/// Hints:
/// - Compute both encodings
/// - Compare strings
pub fn diff_hex_encode(data: &[u8]) -> Result<(), String> {
    todo!("Implement differential hex encoding test")
}

/// A "naive" XOR implementation (reference).
pub fn xor_a(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// A "chunked" XOR implementation (optimized).
///
/// XOR in 8-byte chunks using u64, then handle remainder byte-by-byte.
///
/// Hints:
/// - Process min(a.len(), b.len()) bytes
/// - For the fast path: read u64 from each slice, XOR, write back
/// - For the remainder: XOR remaining bytes individually
/// - This is safe to implement with byte-by-byte for the exercise
pub fn xor_b(a: &[u8], b: &[u8]) -> Vec<u8> {
    todo!("Implement chunked XOR")
}

/// Differential test for XOR implementations.
///
/// Returns Ok(()) if both produce identical output.
pub fn diff_xor(a: &[u8], b: &[u8]) -> Result<(), String> {
    todo!("Implement differential XOR test")
}

/// Implementation A: Base64 encoding using the base64 crate.
pub fn base64_encode_a(data: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}

/// Implementation B: Manual base64 encoding.
///
/// Implements the base64 encoding algorithm manually:
/// 1. Process input 3 bytes at a time
/// 2. Split into 4 groups of 6 bits
/// 3. Map each 6-bit group to a base64 character
/// 4. Pad with '=' if input length is not a multiple of 3
///
/// Hints:
/// - Use the base64 alphabet: A-Z, a-z, 0-9, +, /
/// - Handle padding: 1 remaining byte → 2 output chars + "==", 2 remaining → 3 chars + "="
pub fn base64_encode_b(data: &[u8]) -> String {
    todo!("Implement manual base64 encoding")
}

/// Differential test for base64 encoding.
pub fn diff_base64_encode(data: &[u8]) -> Result<(), String> {
    todo!("Implement differential base64 test")
}

/// Generic differential test runner.
///
/// Takes two functions and input, runs both, compares outputs.
/// Returns Ok(output) if they agree, Err if they diverge.
///
/// Hints:
/// - Call f_a(input) and f_b(input)
/// - Compare the results
/// - Return Ok with one of the outputs if they match
/// - Return Err with a message listing both outputs if they differ
pub fn run_differential<T: PartialEq + std::fmt::Debug>(
    input: &[u8],
    f_a: fn(&[u8]) -> T,
    f_b: fn(&[u8]) -> T,
) -> Result<T, String> {
    todo!("Implement generic differential test runner")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SHA-256 differential ---

    #[test]
    fn test_diff_sha256_agrees() {
        assert!(diff_sha256(b"hello").is_ok());
        assert!(diff_sha256(b"").is_ok());
    }

    proptest::proptest! {
        #[test]
        fn test_diff_sha256_always_agrees(data in prop::collection::vec(prop::num::u8::ANY, 0..1000)) {
            prop_assert!(
                diff_sha256(&data).is_ok(),
                "SHA-256 implementations diverged on input of length {}",
                data.len()
            );
        }
    }

    // --- Hex differential ---

    #[test]
    fn test_diff_hex_agrees() {
        assert!(diff_hex_encode(b"hello").is_ok());
        assert!(diff_hex_encode(b"").is_ok());
    }

    proptest::proptest! {
        #[test]
        fn test_diff_hex_always_agrees(data in prop::collection::vec(prop::num::u8::ANY, 0..500)) {
            prop_assert!(
                diff_hex_encode(&data).is_ok(),
                "Hex implementations diverged"
            );
        }
    }

    // --- XOR differential ---

    #[test]
    fn test_diff_xor_agrees() {
        assert!(diff_xor(b"hello", b"world").is_ok());
    }

    proptest::proptest! {
        #[test]
        fn test_diff_xor_always_agrees(
            a in prop::collection::vec(prop::num::u8::ANY, 1..500),
            b in prop::collection::vec(prop::num::u8::ANY, 1..500)
        ) {
            prop_assert!(
                diff_xor(&a, &b).is_ok(),
                "XOR implementations diverged"
            );
        }
    }

    // --- Base64 differential ---

    #[test]
    fn test_diff_base64_agrees() {
        assert!(diff_base64_encode(b"hello").is_ok());
        assert!(diff_base64_encode(b"").is_ok());
        // Test padding cases
        assert!(diff_base64_encode(b"a").is_ok());
        assert!(diff_base64_encode(b"ab").is_ok());
        assert!(diff_base64_encode(b"abc").is_ok());
    }

    proptest::proptest! {
        #[test]
        fn test_diff_base64_always_agrees(data in prop::collection::vec(prop::num::u8::ANY, 0..500)) {
            prop_assert!(
                diff_base64_encode(&data).is_ok(),
                "Base64 implementations diverged"
            );
        }
    }

    // --- Generic differential runner ---

    #[test]
    fn test_generic_runner_sha256() {
        let result = run_differential(b"test", sha256_a, sha256_b);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_runner_hex() {
        let result = run_differential(b"test", hex_encode_a, hex_encode_b);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_runner_detects_divergence() {
        // Two functions that always return different values
        fn always_true(_data: &[u8]) -> bool { true }
        fn always_false(_data: &[u8]) -> bool { false }
        let result = run_differential(b"test", always_true, always_false);
        assert!(result.is_err());
    }
}
