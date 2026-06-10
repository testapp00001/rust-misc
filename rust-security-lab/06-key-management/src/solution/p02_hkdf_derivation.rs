//! # Lesson 02: HKDF — HMAC-based Key Derivation Function (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use hkdf::Hkdf;
use sha2::Sha256;

/// HKDF-Extract: Condense input key material into a pseudorandom key.
///
/// The extract step ensures that even if the input key material has non-uniform
/// distribution (e.g., from a DH key exchange), the PRK is uniformly random.
pub fn extract(salt: &[u8], ikm: &[u8]) -> Vec<u8> {
    // HKDF-Extract is HMAC(salt, ikm).
    // The hkdf crate does this internally in Hkdf::new(), but for direct PRK access
    // we compute it ourselves using hmac.
    use hmac::{Hmac, Mac};
    use sha2::Sha256 as S256;
    type HmacSha256 = Hmac<S256>;

    let mut mac = HmacSha256::new_from_slice(salt).expect("HMAC accepts any key length");
    mac.update(ikm);
    let result = mac.finalize();
    result.into_bytes().to_vec()
}

/// HKDF-Expand: Generate output key material from a PRK.
///
/// The expand step uses HMAC iteratively to produce as many bytes as needed.
/// The `info` parameter binds the output to a specific context/purpose.
pub fn expand(prk: &[u8], info: &[u8], length: usize) -> Vec<u8> {
    let hk = Hkdf::<Sha256>::from_prk(prk).expect("Valid PRK");
    let mut okm = vec![0u8; length];
    hk.expand(info, &mut okm).expect("Valid expand length");
    okm
}

/// One-shot HKDF: Extract + Expand in one step.
///
/// This is the most common usage pattern. The `hkdf` crate's `new()` does extract
/// internally, and `expand()` does the expand step.
pub fn derive_key(salt: &[u8], ikm: &[u8], info: &[u8], length: usize) -> Vec<u8> {
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut okm = vec![0u8; length];
    hk.expand(info, &mut okm).expect("Valid expand length");
    okm
}

/// Derive multiple independent keys from one master key.
///
/// Each key uses a unique info string (e.g., "key-0", "key-1") to ensure
/// cryptographic independence. The HKDF expand step guarantees that outputs
/// with different info values are computationally independent.
pub fn derive_multiple_keys(master: &[u8], salt: &[u8], count: usize, key_length: usize) -> Vec<Vec<u8>> {
    let hk = Hkdf::<Sha256>::new(Some(salt), master);
    (0..count)
        .map(|i| {
            let info = format!("key-{}", i);
            let mut okm = vec![0u8; key_length];
            hk.expand(info.as_bytes(), &mut okm).expect("Valid expand");
            okm
        })
        .collect()
}

/// Demonstrate HKDF info string semantics.
///
/// Returns: (key_with_info_a, key_with_info_b, key_same_info_1, key_same_info_2)
///
/// Security note: The info string is not a secret — it's a context label.
/// It's often a human-readable string like "encryption-key" or "v1:auth".
pub fn demonstrate_info_importance(master: &[u8], salt: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let key_a = derive_key(salt, master, b"context-a", 32);
    let key_b = derive_key(salt, master, b"context-b", 32);
    let same1 = derive_key(salt, master, b"same-context", 32);
    let same2 = derive_key(salt, master, b"same-context", 32);
    (key_a, key_b, same1, same2)
}

/// Derive encryption and MAC keys from a shared secret.
///
/// After ECDH or similar key exchange, the raw shared secret should never be
/// used directly as a key. HKDF extracts full entropy and derives purpose-specific
/// keys.
///
/// This pattern is used in TLS 1.3, Signal Protocol, and WireGuard.
pub fn derive_enc_and_mac_keys(shared_secret: &[u8], salt: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let enc_key = derive_key(salt, shared_secret, b"encryption", 32);
    let mac_key = derive_key(salt, shared_secret, b"authentication", 32);
    (enc_key, mac_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_master() -> Vec<u8> {
        vec![0xABu8; 32]
    }

    fn test_salt() -> Vec<u8> {
        vec![0xCDu8; 16]
    }

    #[test]
    fn test_extract_length() {
        let prk = extract(&test_salt(), &test_master());
        assert_eq!(prk.len(), 32, "PRK from SHA-256 HKDF should be 32 bytes");
    }

    #[test]
    fn test_extract_deterministic() {
        let prk1 = extract(&test_salt(), &test_master());
        let prk2 = extract(&test_salt(), &test_master());
        assert_eq!(prk1, prk2);
    }

    #[test]
    fn test_expand_length() {
        let prk = extract(&test_salt(), &test_master());
        let okm = expand(&prk, b"test", 48);
        assert_eq!(okm.len(), 48);
    }

    #[test]
    fn test_expand_different_info() {
        let prk = extract(&test_salt(), &test_master());
        let okm1 = expand(&prk, b"encryption", 32);
        let okm2 = expand(&prk, b"authentication", 32);
        assert_ne!(okm1, okm2, "Different info must produce different keys");
    }

    #[test]
    fn test_expand_same_info_deterministic() {
        let prk = extract(&test_salt(), &test_master());
        let okm1 = expand(&prk, b"test", 32);
        let okm2 = expand(&prk, b"test", 32);
        assert_eq!(okm1, okm2, "Same info must produce same key");
    }

    #[test]
    fn test_derive_key_length() {
        let key = derive_key(&test_salt(), &test_master(), b"test", 64);
        assert_eq!(key.len(), 64);
    }

    #[test]
    fn test_derive_multiple_keys_count() {
        let keys = derive_multiple_keys(&test_master(), &test_salt(), 3, 32);
        assert_eq!(keys.len(), 3);
        for key in &keys {
            assert_eq!(key.len(), 32);
        }
    }

    #[test]
    fn test_derive_multiple_keys_independent() {
        let keys = derive_multiple_keys(&test_master(), &test_salt(), 3, 32);
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[1], keys[2]);
        assert_ne!(keys[0], keys[2]);
    }

    #[test]
    fn test_info_importance() {
        let master = test_master();
        let salt = test_salt();
        let (key_a, key_b, same1, same2) = demonstrate_info_importance(&master, &salt);
        assert_ne!(key_a, key_b, "Keys with different info must differ");
        assert_eq!(same1, same2, "Keys with same info must match");
    }

    #[test]
    fn test_enc_mac_keys_independent() {
        let (enc_key, mac_key) = derive_enc_and_mac_keys(&test_master(), &test_salt());
        assert_eq!(enc_key.len(), 32);
        assert_eq!(mac_key.len(), 32);
        assert_ne!(enc_key, mac_key, "Encryption and MAC keys must be independent");
    }

    #[test]
    fn test_derive_key_empty_salt() {
        let key = derive_key(b"", &test_master(), b"test", 32);
        assert_eq!(key.len(), 32);
    }
}
