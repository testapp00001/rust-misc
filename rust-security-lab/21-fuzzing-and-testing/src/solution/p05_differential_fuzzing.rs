//! # Lesson 05: Differential Fuzzing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::Digest;
use proptest::prelude::*;

/// Implementation A: SHA-256 using the sha2 crate.
pub fn sha256_a(data: &[u8]) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Implementation B: SHA-256 using ring.
pub fn sha256_b(data: &[u8]) -> Vec<u8> {
    ring::digest::digest(&ring::digest::SHA256, data).as_ref().to_vec()
}

/// Differential test for SHA-256.
pub fn diff_sha256(data: &[u8]) -> Result<(), String> {
    let a = sha256_a(data);
    let b = sha256_b(data);
    if a == b {
        Ok(())
    } else {
        Err(format!(
            "SHA-256 diverged on {} bytes: sha2={:?}, ring={:?}",
            data.len(),
            &a[..8],
            &b[..8]
        ))
    }
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
pub fn diff_hex_encode(data: &[u8]) -> Result<(), String> {
    let a = hex_encode_a(data);
    let b = hex_encode_b(data);
    if a == b {
        Ok(())
    } else {
        Err(format!("Hex diverged: a={}, b={}", a, b))
    }
}

/// A "naive" XOR implementation (reference).
pub fn xor_a(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// A "chunked" XOR implementation.
pub fn xor_b(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// Differential test for XOR implementations.
pub fn diff_xor(a: &[u8], b: &[u8]) -> Result<(), String> {
    let r_a = xor_a(a, b);
    let r_b = xor_b(a, b);
    if r_a == r_b {
        Ok(())
    } else {
        Err(format!("XOR diverged: a={:?}, b={:?}", &r_a[..8], &r_b[..8]))
    }
}

/// Implementation A: Base64 encoding using the base64 crate.
pub fn base64_encode_a(data: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}

/// Implementation B: Manual base64 encoding.
pub fn base64_encode_b(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let chunks = data.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

/// Differential test for base64 encoding.
pub fn diff_base64_encode(data: &[u8]) -> Result<(), String> {
    let a = base64_encode_a(data);
    let b = base64_encode_b(data);
    if a == b {
        Ok(())
    } else {
        Err(format!("Base64 diverged: a={}, b={}", a, b))
    }
}

/// Generic differential test runner.
pub fn run_differential<T: PartialEq + std::fmt::Debug>(
    input: &[u8],
    f_a: fn(&[u8]) -> T,
    f_b: fn(&[u8]) -> T,
) -> Result<T, String> {
    let a = f_a(input);
    let b = f_b(input);
    if a == b {
        Ok(a)
    } else {
        Err(format!(
            "Implementations diverged on {} bytes: a={:?}, b={:?}",
            input.len(),
            a,
            b
        ))
    }
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
        fn always_true(_data: &[u8]) -> bool { true }
        fn always_false(_data: &[u8]) -> bool { false }
        let result = run_differential(b"test", always_true, always_false);
        assert!(result.is_err());
    }
}
