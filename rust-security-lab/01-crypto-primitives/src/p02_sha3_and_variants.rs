//! # Lesson 02: SHA-3 Family
//!
//! ## SHA-3: The New Standard
//!
//! SHA-3 (Keccak) was selected by NIST in 2012 after a public competition. It uses a
//! completely different construction (sponge) than SHA-2 (Merkle-Damgård), providing
//! diversity in case a fundamental break is found in one family.
//!
//! ## SHA-3 Variants
//!
//! | Variant | Output Size | Security Level | Use Case |
//! |---------|-------------|----------------|----------|
//! | SHA3-224 | 28 bytes | 112-bit | Lightweight integrity |
//! | SHA3-256 | 32 bytes | 128-bit | General-purpose hashing |
//! | SHA3-384 | 48 bytes | 192-bit | High security |
//! | SHA3-512 | 64 bytes | 256-bit | Maximum security |
//! | SHAKE128 |任意 | min(d/2, 128) | XOF — extendable output |
//! | SHAKE256 |任意 | min(d/2, 256) | XOF — extendable output |
//!
//! ## SHA-2 vs SHA-3
//!
//! | Property | SHA-2 | SHA-3 |
//! |----------|-------|-------|
//! | Construction | Merkle-Damgård | Sponge (Keccak) |
//! | Length extension | Vulnerable | Immune |
//! | Performance | Faster in hardware | Faster in software (some) |
//! | Maturity | Since 2001 | Since 2015 |
//! | Use case | Existing systems, TLS | New systems, future-proofing |
//!
//! ## SHAKE: Extendable Output Functions
//!
//! SHAKE128/256 can produce arbitrary-length output (like a hash that never stops).
//! Useful for key derivation, random number generation, and lattice-based crypto.

use sha3::{Digest, Sha3_256, Sha3_512, Sha3_224, Sha3_384, Shake128, Shake256};
use sha3::digest::{ExtendableOutput, Update, XofReader};

/// Exercise 1: Compute SHA3-256 hash.
///
/// Hints:
/// - Create hasher: `Sha3_256::new()`
/// - Feed data: `.update(data)`
/// - Finalize: `.finalize()`
/// - The result implements `AsRef<[u8]>` — convert to Vec<u8>
pub fn sha3_256(data: &[u8]) -> Vec<u8> {
    todo!("Implement SHA3-256 hashing")
}

/// Exercise 2: Compute SHA3-512 hash.
pub fn sha3_512(data: &[u8]) -> Vec<u8> {
    todo!("Implement SHA3-512 hashing")
}

/// Exercise 3: Compute SHAKE256 with custom output length.
///
/// SHAKE is an XOF (Extendable Output Function) — it can produce any output length.
///
/// Hints:
/// - Create: `Shake256::default()`
/// - Feed: `.update(data)`
/// - Finalize: `.finalize_xof()`
/// - Read: `.read(&mut output_buffer)`
pub fn shake256(data: &[u8], output_len: usize) -> Vec<u8> {
    todo!("Implement SHAKE256 with variable output length")
}

/// Exercise 4: Demonstrate that SHA-3 is immune to length extension attacks.
///
/// Unlike SHA-256, knowing `sha3_256(msg)` does NOT let an attacker compute
/// `sha3_256(msg || padding || extension)`.
///
/// This function should:
/// 1. Hash a message with SHA3-256
/// 2. Return the hash
/// 3. (Tests will verify that length extension doesn't work)
pub fn sha3_no_length_extension(data: &[u8]) -> Vec<u8> {
    todo!("Implement SHA3-256 (immune to length extension by design)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha3_256_empty() {
        let hash = sha3_256(b"");
        assert_eq!(hash.len(), 32);
        // Known SHA3-256 of empty string
        let expected = "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a";
        assert_eq!(hex::encode(&hash), expected);
    }

    #[test]
    fn test_sha3_256_hello() {
        let hash = sha3_256(b"hello");
        assert_eq!(hash.len(), 32);
        // SHA3-256 of "hello" differs from SHA-256 of "hello"
        let sha256_hello = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert_ne!(hex::encode(&hash), sha256_hello, "SHA3 and SHA2 should produce different hashes");
    }

    #[test]
    fn test_sha3_512() {
        let hash = sha3_512(b"hello");
        assert_eq!(hash.len(), 64, "SHA3-512 produces 64 bytes");
    }

    #[test]
    fn test_shake256_variable_length() {
        let hash_32 = shake256(b"test", 32);
        let hash_64 = shake256(b"test", 64);
        let hash_100 = shake256(b"test", 100);

        assert_eq!(hash_32.len(), 32);
        assert_eq!(hash_64.len(), 64);
        assert_eq!(hash_100.len(), 100);

        // First 32 bytes of all should match (same input, same initial output)
        assert_eq!(&hash_32[..], &hash_64[..32]);
        assert_eq!(&hash_32[..], &hash_100[..32]);
    }

    #[test]
    fn test_sha3_different_from_sha2() {
        let data = b"identical input";
        let sha2 = ring::digest::digest(&ring::digest::SHA256, data);
        let sha3 = sha3_256(data);
        assert_ne!(sha2.as_ref(), &sha3[..], "SHA-2 and SHA-3 should produce different outputs");
    }

    #[test]
    fn test_sha3_immune_to_length_extension() {
        // With SHA-256, knowing hash(msg) lets attacker compute hash(msg || padding || ext)
        // With SHA-3, this is NOT possible due to sponge construction
        let msg = b"secret message";
        let hash1 = sha3_256(msg);

        // Attacker tries to extend — the hash of msg||extension should be completely different
        // and NOT derivable from hash1
        let extended = [msg.as_ref(), b"evil extension"].concat();
        let hash2 = sha3_256(&extended);

        // These should be completely unrelated (not just different)
        assert_ne!(hash1, hash2);
        // In a real length extension attack on SHA-256, the attacker could predict hash2
        // from hash1 without knowing msg. With SHA-3, they can't.
    }

    #[test]
    fn test_deterministic() {
        let h1 = sha3_256(b"consistency check");
        let h2 = sha3_256(b"consistency check");
        assert_eq!(h1, h2);
    }
}
