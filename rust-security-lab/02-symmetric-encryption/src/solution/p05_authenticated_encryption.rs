//! # Lesson 05: Authenticated Encryption (Reference Solution)
//!
//! See the exercise file for full documentation on AEAD, EtM, and attacks.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};
use ring::hmac;

/// Encrypt-then-MAC: encrypt first, then compute MAC over the ciphertext.
///
/// This is the CORRECT composition. The MAC covers the ciphertext (not plaintext),
/// so the receiver verifies integrity before attempting decryption.
pub fn encrypt_then_mac(
    enc_key: &Key<Aes256Gcm>,
    mac_key: &[u8; 32],
    plaintext: &[u8],
) -> Vec<u8> {
    // Step 1: Encrypt with AES-GCM (nonce = zeros for simplicity; in production use random)
    let cipher = Aes256Gcm::new(enc_key);
    let nonce = Nonce::<Aes256Gcm>::from_slice(&[0u8; 12]);
    let ciphertext = cipher.encrypt(nonce, plaintext).expect("Encryption failed");

    // Step 2: Compute HMAC-SHA256 over the ciphertext
    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, mac_key);
    let tag = hmac::sign(&hmac_key, &ciphertext);

    // Step 3: Return ciphertext || tag
    let mut output = Vec::with_capacity(ciphertext.len() + 32);
    output.extend_from_slice(&ciphertext);
    output.extend_from_slice(tag.as_ref());
    output
}

/// Verify-then-decrypt: check MAC first, then decrypt.
///
/// CRITICAL: Always verify the MAC BEFORE decryption to prevent
/// chosen-ciphertext attacks.
pub fn verify_then_decrypt(
    enc_key: &Key<Aes256Gcm>,
    mac_key: &[u8; 32],
    data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if data.len() < 32 {
        return Err("Data too short");
    }

    // Step 1: Split ciphertext and MAC tag (last 32 bytes for SHA256)
    let (ciphertext, tag_bytes) = data.split_at(data.len() - 32);

    // Step 2: Verify HMAC FIRST
    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, mac_key);
    hmac::verify(&hmac_key, ciphertext, tag_bytes)
        .map_err(|_| "MAC verification failed — data may be tampered")?;

    // Step 3: Only decrypt if MAC is valid
    let cipher = Aes256Gcm::new(enc_key);
    let nonce = Nonce::<Aes256Gcm>::from_slice(&[0u8; 12]);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed")
}

/// Demonstrate that AEAD (GCM) detects any tampering.
pub fn demonstrate_aead_tamper_detection(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> bool {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let mut ciphertext = cipher.encrypt(&nonce, plaintext).expect("Encrypt failed");

    // Tamper with one byte
    if !ciphertext.is_empty() {
        ciphertext[0] ^= 0xff;
    }

    // AEAD decryption must fail
    cipher.decrypt(&nonce, ciphertext.as_ref()).is_err()
}

/// Demonstrate bit-flip attack and AEAD defense.
///
/// In CBC without MAC, an attacker can flip bits to produce controlled changes.
/// AEAD catches this immediately.
pub fn demonstrate_bit_flip_attack(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> (Vec<u8>, bool) {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("Encrypt failed");

    // In a non-authenticated cipher, flipping bits in the ciphertext block
    // would produce controlled changes in the corresponding plaintext block.
    // With AEAD, the entire decryption fails.
    let mut tampered = ciphertext.clone();
    if !tampered.is_empty() {
        tampered[0] ^= 0xff;
    }

    let aead_detects = cipher.decrypt(&nonce, tampered.as_ref()).is_err();
    (b"tampered_simulated".to_vec(), aead_detects)
}

/// High-level seal: encrypt with a fresh nonce, prepend nonce to output.
pub fn seal(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("Encrypt failed");

    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    output
}

/// High-level open: extract nonce, verify tag, decrypt.
pub fn open(key: &Key<Aes256Gcm>, sealed: &[u8]) -> Result<Vec<u8>, &'static str> {
    if sealed.len() < 12 {
        return Err("Sealed data too short: need at least 12 bytes for nonce");
    }

    let (nonce_bytes, ciphertext) = sealed.split_at(12);
    let nonce = Nonce::<Aes256Gcm>::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(key);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed — data may be tampered or key is wrong")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::hmac;

    fn test_key() -> Key<Aes256Gcm> {
        Aes256Gcm::generate_key(&mut OsRng)
    }

    fn test_mac_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        for (i, byte) in key.iter_mut().enumerate() {
            *byte = i as u8;
        }
        key
    }

    #[test]
    fn test_encrypt_then_mac_roundtrip() {
        let enc_key = test_key();
        let mac_key = test_mac_key();
        let plaintext = b"authenticated encryption test";

        let sealed = encrypt_then_mac(&enc_key, &mac_key, plaintext);
        let decrypted = verify_then_decrypt(&enc_key, &mac_key, &sealed).expect("EtM failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_verify_then_decrypt_tampered() {
        let enc_key = test_key();
        let mac_key = test_mac_key();
        let plaintext = b"integrity test";

        let mut sealed = encrypt_then_mac(&enc_key, &mac_key, plaintext);
        if sealed.len() > 32 {
            sealed[0] ^= 0xff;
        }
        let result = verify_then_decrypt(&enc_key, &mac_key, &sealed);
        assert!(result.is_err(), "Tampered data must fail MAC verification");
    }

    #[test]
    fn test_verify_then_decrypt_wrong_mac_key() {
        let enc_key = test_key();
        let mac_key = test_mac_key();
        let plaintext = b"wrong key test";

        let sealed = encrypt_then_mac(&enc_key, &mac_key, plaintext);

        let wrong_mac_key = [0xABu8; 32];
        let result = verify_then_decrypt(&enc_key, &wrong_mac_key, &sealed);
        assert!(result.is_err(), "Wrong MAC key must fail verification");
    }

    #[test]
    fn test_aead_tamper_detection() {
        let key = test_key();
        let detected = demonstrate_aead_tamper_detection(&key, b"AEAD protects this");
        assert!(detected, "AEAD must detect tampering");
    }

    #[test]
    fn test_bit_flip_attack_demo() {
        let key = test_key();
        let (_, aead_catches_it) = demonstrate_bit_flip_attack(&key, b"secret message");
        assert!(aead_catches_it, "AEAD should detect bit-flip attacks");
    }

    #[test]
    fn test_seal_open_roundtrip() {
        let key = test_key();
        let plaintext = b"seal and open test";
        let sealed = seal(&key, plaintext);
        let opened = open(&key, &sealed).expect("Open failed");
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn test_seal_open_tampered() {
        let key = test_key();
        let mut sealed = seal(&key, b"tamper test");
        if !sealed.is_empty() {
            sealed[0] ^= 0x01;
        }
        assert!(open(&key, &sealed).is_err(), "Tampered seal must fail");
    }

    #[test]
    fn test_seal_different_nonces() {
        let key = test_key();
        let s1 = seal(&key, b"same message");
        let s2 = seal(&key, b"same message");
        assert_ne!(s1, s2, "Sealed packages should differ (different nonces)");
    }
}
