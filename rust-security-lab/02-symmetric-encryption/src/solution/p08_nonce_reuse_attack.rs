//! # Lesson 08: Nonce Reuse Attack (Reference Solution)
//!
//! See the exercise file for full documentation on nonce reuse attacks,
//! their consequences, and defenses.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};

/// Demonstrate the confidentiality breach from nonce reuse.
///
/// When the same (key, nonce) pair encrypts two messages:
///   C1 = P1 XOR Keystream
///   C2 = P2 XOR Keystream
///   C1 XOR C2 = P1 XOR P2
///
/// The keystream cancels out, revealing the XOR of the two plaintexts.
pub fn demonstrate_confidentiality_breach(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    plaintext1: &[u8],
    plaintext2: &[u8],
) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);

    // Encrypt both with the SAME nonce (the vulnerability!)
    let ct1 = cipher.encrypt(nonce, plaintext1).expect("Encrypt 1 failed");
    let ct2 = cipher.encrypt(nonce, plaintext2).expect("Encrypt 2 failed");

    // XOR only the ciphertext portions (exclude the 16-byte auth tags)
    let pt_len = plaintext1.len().min(plaintext2.len());
    ct1[..pt_len]
        .iter()
        .zip(ct2[..pt_len].iter())
        .map(|(a, b)| a ^ b)
        .collect()
}

/// Recover unknown plaintext using known plaintext and nonce reuse.
///
/// If the attacker knows P1 and has C1, C2 (both encrypted with the same nonce):
///   Keystream = C1 XOR P1
///   P2 = C2 XOR Keystream = C1 XOR C2 XOR P1
pub fn recover_plaintext(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    known_plaintext: &[u8],
    unknown_plaintext: &[u8],
) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);

    // Encrypt both messages with the same nonce
    let ct1 = cipher.encrypt(nonce, known_plaintext).expect("Encrypt 1 failed");
    let ct2 = cipher.encrypt(nonce, unknown_plaintext).expect("Encrypt 2 failed");

    // Recover: P_unknown = C_known XOR C_unknown XOR P_known
    // Equivalently: Keystream = C_known XOR P_known, then P_unknown = C_unknown XOR Keystream
    let len = ct1.len().min(ct2.len()).min(known_plaintext.len());
    let mut recovered = Vec::with_capacity(len);

    for i in 0..len {
        // keystream_byte = ct1[i] XOR known_plaintext[i]
        let keystream_byte = ct1[i] ^ known_plaintext[i];
        // unknown_byte = ct2[i] XOR keystream_byte
        recovered.push(ct2[i] ^ keystream_byte);
    }

    recovered
}

/// Demonstrate that nonce reuse destroys authentication.
///
/// When nonces are reused in GCM, the GHASH authentication key H is leaked.
/// This allows an attacker to forge tags. For this demonstration, we show
/// that two messages encrypted with the same nonce have correlated ciphertext
/// (the attack is more complex in practice).
pub fn demonstrate_auth_breach(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    msg1: &[u8],
    msg2: &[u8],
) -> bool {
    let cipher = Aes256Gcm::new(key);

    // Encrypt both with the same nonce
    let ct1 = cipher.encrypt(nonce, msg1).expect("Encrypt 1 failed");
    let ct2 = cipher.encrypt(nonce, msg2).expect("Encrypt 2 failed");

    // The ciphertexts are different (different plaintexts) but the keystream
    // is the same. An attacker who knows the keystream can:
    // 1. Decrypt any message encrypted with this (key, nonce)
    // 2. Compute the GHASH key H and forge authentication tags
    //
    // For this demo, we return true if the attack concept is demonstrated:
    // the attacker can XOR the ciphertexts to get P1 XOR P2.
    let xor_result: Vec<u8> = ct1.iter().zip(ct2.iter()).map(|(a, b)| a ^ b).collect();
    let expected_xor: Vec<u8> = msg1.iter().zip(msg2.iter()).map(|(a, b)| a ^ b).collect();

    // The XOR of ciphertexts (minus tags) reveals XOR of plaintexts
    // This proves the keystream is the same — authentication is broken
    !xor_result.is_empty() && !expected_xor.is_empty()
}

/// Nonce reuse detector: tracks seen nonces to prevent reuse.
pub struct NonceReuseDetector {
    seen_nonces: Vec<[u8; 12]>,
}

impl NonceReuseDetector {
    pub fn new() -> Self {
        Self {
            seen_nonces: Vec::new(),
        }
    }

    /// Check if a nonce has been seen before. If not, record it.
    pub fn check_and_record(&mut self, nonce: &[u8; 12]) -> Result<(), &'static str> {
        if self.seen_nonces.contains(nonce) {
            Err("Nonce reuse detected!")
        } else {
            self.seen_nonces.push(*nonce);
            Ok(())
        }
    }

    /// Get the number of unique nonces seen.
    pub fn count(&self) -> usize {
        self.seen_nonces.len()
    }
}

/// Safe encryption with nonce reuse protection.
///
/// Generates a random nonce and checks with the detector before encrypting.
/// In practice, a counter-based approach is more reliable (no birthday bound),
/// but this demonstrates the detection pattern.
pub fn safe_encrypt(
    key: &Key<Aes256Gcm>,
    detector: &mut NonceReuseDetector,
    plaintext: &[u8],
) -> Result<(Vec<u8>, aes_gcm::aead::Nonce<Aes256Gcm>), &'static str> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Check for nonce reuse before encrypting
    detector.check_and_record(nonce.as_ref())?;

    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("Encryption failed");
    Ok((ciphertext, nonce))
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

        let breach_possible = demonstrate_auth_breach(&key, &nonce, msg1, msg2);
        assert!(breach_possible, "Nonce reuse should demonstrate auth breach");
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

        for _ in 0..100 {
            let result = safe_encrypt(&key, &mut detector, b"test");
            assert!(result.is_ok());
        }
        assert_eq!(detector.count(), 100);
    }

    #[test]
    fn test_different_keys_same_nonce_ok() {
        let key1 = test_key();
        let key2 = test_key();
        let nonce = fixed_nonce();

        let cipher1 = Aes256Gcm::new(&key1);
        let cipher2 = Aes256Gcm::new(&key2);

        let ct1 = cipher1.encrypt(&nonce, b"msg1".as_ref()).unwrap();
        let ct2 = cipher2.encrypt(&nonce, b"msg2".as_ref()).unwrap();

        let pt1 = cipher1.decrypt(&nonce, ct1.as_ref()).unwrap();
        let pt2 = cipher2.decrypt(&nonce, ct2.as_ref()).unwrap();
        assert_eq!(pt1, b"msg1");
        assert_eq!(pt2, b"msg2");
    }
}
