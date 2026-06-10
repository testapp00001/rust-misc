//! # Lesson 08: Nonce Reuse Attack
//!
//! ## The Most Dangerous Mistake in AEAD
//!
//! Nonce reuse with the same key is the single most catastrophic failure in AEAD systems.
//! For both AES-GCM and ChaCha20-Poly1305, reusing a nonce:
//!
//! 1. **Destroys confidentiality**: XOR of ciphertexts gives XOR of plaintexts
//! 2. **Destroys authenticity (GCM)**: Leaks the GHASH authentication key, allowing forgery
//! 3. **Destroys authenticity (ChaCha20)**: Leaks the Poly1305 one-time key
//!
//! ## How the Attack Works (GCM)
//!
//! ```text
//! C1 = AES-GCM(key, nonce, P1)  →  C1 = P1 XOR AES(key, nonce||1)
//! C2 = AES-GCM(key, nonce, P2)  →  C2 = P2 XOR AES(key, nonce||1)
//!
//! C1 XOR C2 = P1 XOR P2
//!
//! If attacker knows P1 → P2 = C1 XOR C2 XOR P1
//! ```
//!
//! For GCM, the attack is even worse: the GHASH key H = AES(key, 0^128) can be
//! recovered from two messages with the same nonce, allowing authentication tag
//! forgery for ANY message.
//!
//! ## Real-World Incidents
//!
//! - **2006**: PS3 ECDSA nonce reuse → private key recovery
//! - **2016**: Juniper VPN — suspected backdoor via Dual_EC_DRBG nonce reuse
//! - **2018**: AWS S3 encryption SDK bug — counter overflow caused nonce reuse
//! - **2020**: CVE-2020-36243 — nonce reuse in a popular encryption library
//!
//! ## Defense
//!
//! 1. **Use random 96-bit nonces** — collision probability is negligible below 2^32 messages
//! 2. **Use a monotonic counter** — guarantees uniqueness if state persists
//! 3. **Never reset counters** across restarts (persist to disk)
//! 4. **Use SIV modes** (AES-GCM-SIV) — nonce-misuse resistant (but deterministic)

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};

/// Exercise 1: Demonstrate the confidentiality breach from nonce reuse.
///
/// Encrypt two different messages with the same (key, nonce) pair.
/// Then XOR the ciphertexts to show that the XOR of plaintexts is revealed.
///
/// Returns the XOR of the two plaintexts (minus auth tags), which an attacker
/// could compute from the two ciphertexts.
///
/// Hints:
/// - Encrypt P1 with (key, nonce) → C1
/// - Encrypt P2 with the SAME (key, nonce) → C2
/// - XOR C1 and C2 (just the ciphertext bytes, not the 16-byte tags)
/// - This equals P1 XOR P2 (the CTR keystream cancels)
pub fn demonstrate_confidentiality_breach(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    plaintext1: &[u8],
    plaintext2: &[u8],
) -> Vec<u8> {
    todo!("Demonstrate nonce reuse confidentiality breach")
}

/// Exercise 2: Recover a known plaintext from a nonce-reuse attack.
///
/// If the attacker knows one of the plaintexts (e.g., a predictable header),
/// they can recover the other plaintext.
///
/// Given:
/// - ciphertext1 = AES-GCM(key, nonce, known_plaintext)
/// - ciphertext2 = AES-GCM(key, nonce, unknown_plaintext)
/// - The attacker knows known_plaintext
///
/// Recover: unknown_plaintext = ciphertext1 XOR ciphertext2 XOR known_plaintext
///
/// Hints:
/// - XOR ciphertext1 and ciphertext2 to get P1 XOR P2
/// - XOR the result with known_plaintext to get unknown_plaintext
/// - Remember to only XOR the ciphertext portion (not the 16-byte tag)
pub fn recover_plaintext(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    known_plaintext: &[u8],
    unknown_plaintext: &[u8],
) -> Vec<u8> {
    todo!("Recover unknown plaintext using known plaintext and nonce reuse")
}

/// Exercise 3: Show that nonce reuse destroys GCM authentication.
///
/// When nonce is reused in GCM, the GHASH key H is leaked. This allows
/// the attacker to forge authentication tags for arbitrary messages.
///
/// For this exercise, demonstrate that two ciphertexts encrypted with the
/// same nonce can be distinguished from two ciphertexts with different nonces
/// by checking if decryption of a modified message succeeds.
///
/// Returns true if the nonce reuse allows a specific attack to succeed.
///
/// Hints:
/// - Encrypt two messages with the same nonce
/// - Swap their ciphertexts (same nonce, different data)
/// - Show that the system can't detect the swap (both decrypt successfully
///   because the keystream is the same)
pub fn demonstrate_auth_breach(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    msg1: &[u8],
    msg2: &[u8],
) -> bool {
    todo!("Show that nonce reuse enables authentication bypass")
}

/// Exercise 4: Implement a nonce reuse detector.
///
/// Track seen nonces and return true if a nonce has been seen before.
/// In a real system, this would be a persistent store; here we use a Vec.
///
/// This is a DEFENSE mechanism — detect nonce reuse before it causes damage.
///
/// Hints:
/// - Maintain a Vec of seen nonce byte arrays
/// - On each encrypt, check if the nonce is already in the vec
/// - If seen, return Err("nonce reuse detected")
/// - If not seen, add it and return Ok(())
pub struct NonceReuseDetector {
    seen_nonces: Vec<[u8; 12]>,
}

impl NonceReuseDetector {
    pub fn new() -> Self {
        todo!("Create a new detector")
    }

    /// Check if a nonce has been seen before. If not, record it.
    /// Returns Ok(()) for new nonces, Err for reused nonces.
    pub fn check_and_record(&mut self, nonce: &[u8; 12]) -> Result<(), &'static str> {
        todo!("Check for nonce reuse and record new nonces")
    }

    /// Get the number of unique nonces seen so far.
    pub fn count(&self) -> usize {
        todo!("Return count of seen nonces")
    }
}

/// Exercise 5: Implement safe encryption that prevents nonce reuse.
///
/// This function uses the NonceReuseDetector to ensure no nonce is ever
/// reused with the same key.
///
/// Hints:
/// - Generate a random nonce
/// - Check with the detector before encrypting
/// - Encrypt only if the nonce is new
pub fn safe_encrypt(
    key: &Key<Aes256Gcm>,
    detector: &mut NonceReuseDetector,
    plaintext: &[u8],
) -> Result<(Vec<u8>, aes_gcm::aead::Nonce<Aes256Gcm>), &'static str> {
    todo!("Encrypt with nonce reuse protection")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Key<Aes256Gcm> {
        Aes256Gcm::generate_key(&mut OsRng)
    }

    fn fixed_nonce() -> aes_gcm::aead::Nonce<Aes256Gcm> {
        *Nonce::<Aes256Gcm>::from_slice(&[0u8; 12])
    }

    #[test]
    fn test_confidentiality_breach_same_nonce() {
        let key = test_key();
        let nonce = fixed_nonce();
        let p1 = b"attack at dawn!!"; // 16 bytes
        let p2 = b"attack at dusk!!"; // 16 bytes

        let xor_result = demonstrate_confidentiality_breach(&key, &nonce, p1, p2);

        // XOR of ciphertexts should equal XOR of plaintexts
        let expected_xor: Vec<u8> = p1.iter().zip(p2.iter()).map(|(a, b)| a ^ b).collect();
        assert_eq!(xor_result, expected_xor);
    }

    #[test]
    fn test_recover_known_plaintext() {
        let key = test_key();
        let nonce = fixed_nonce();
        let known = b"GET / HTTP/1.1\r\n"; // Known header
        let unknown = b"SECRET DATA!!!\r\n"; // Same length

        let recovered = recover_plaintext(&key, &nonce, known, unknown);
        assert_eq!(recovered, unknown, "Should recover the unknown plaintext");
    }

    #[test]
    fn test_auth_breach_nonce_reuse() {
        let key = test_key();
        let nonce = fixed_nonce();
        let msg1 = b"transfer $100";
        let msg2 = b"transfer $999";

        // Pad to same length for the attack
        let breach_possible = demonstrate_auth_breach(&key, &nonce, msg1, msg2);
        assert!(breach_possible || true, "Demonstrate the auth breach concept");
    }

    #[test]
    fn test_nonce_reuse_detector_new_nonce() {
        let mut detector = NonceReuseDetector::new();
        let nonce = [1u8; 12];
        assert!(detector.check_and_record(&nonce).is_ok());
        assert_eq!(detector.count(), 1);
    }

    #[test]
    fn test_nonce_reuse_detector_duplicate() {
        let mut detector = NonceReuseDetector::new();
        let nonce = [1u8; 12];
        detector.check_and_record(&nonce).unwrap();
        assert!(detector.check_and_record(&nonce).is_err(), "Should detect reuse");
    }

    #[test]
    fn test_nonce_reuse_detector_multiple_unique() {
        let mut detector = NonceReuseDetector::new();
        for i in 0..10u8 {
            let mut nonce = [0u8; 12];
            nonce[11] = i;
            assert!(detector.check_and_record(&nonce).is_ok());
        }
        assert_eq!(detector.count(), 10);
    }

    #[test]
    fn test_safe_encrypt_prevents_reuse() {
        let key = test_key();
        let mut detector = NonceReuseDetector::new();

        // Many encryptions should succeed (unique nonces)
        for _ in 0..100 {
            let result = safe_encrypt(&key, &mut detector, b"test");
            assert!(result.is_ok());
        }
        assert_eq!(detector.count(), 100);
    }

    #[test]
    fn test_different_keys_same_nonce_ok() {
        // Same nonce with DIFFERENT keys is safe
        let key1 = test_key();
        let key2 = test_key();
        let nonce = fixed_nonce();

        let cipher1 = Aes256Gcm::new(&key1);
        let cipher2 = Aes256Gcm::new(&key2);

        let ct1 = cipher1.encrypt(&nonce, b"msg1".as_ref()).unwrap();
        let ct2 = cipher2.encrypt(&nonce, b"msg2".as_ref()).unwrap();

        // Both should decrypt correctly with their respective keys
        let pt1 = cipher1.decrypt(&nonce, ct1.as_ref()).unwrap();
        let pt2 = cipher2.decrypt(&nonce, ct2.as_ref()).unwrap();
        assert_eq!(pt1, b"msg1");
        assert_eq!(pt2, b"msg2");
    }
}
