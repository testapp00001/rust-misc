//! # Lesson 05: Authenticated Encryption
//!
//! ## What is Authenticated Encryption?
//!
//! Authenticated Encryption with Associated Data (AEAD) provides three guarantees:
//! - **Confidentiality**: Plaintext is hidden from attackers
//! - **Integrity**: Any modification of ciphertext is detected
//! - **Authenticity**: The ciphertext came from someone who knows the key
//!
//! ## Encrypt-then-MAC vs AEAD
//!
//! Before AEAD modes existed, engineers had to compose encryption and MAC manually:
//!
//! ### Encrypt-then-MAC (EtM) — The Correct Composition
//! ```text
//! Encrypt:  ciphertext = Encrypt(key_e, plaintext)
//!           tag = MAC(key_m, ciphertext)
//!           output = ciphertext || tag
//!
//! Decrypt:  verify MAC first, THEN decrypt
//! ```
//!
//! ### MAC-then-Encrypt (MtE) — INSECURE (used by TLS 1.2)
//! ```text
//! Encrypt:  tag = MAC(key_m, plaintext)
//!           ciphertext = Encrypt(key_e, plaintext || tag)
//!           → Attacker can modify IV to create valid-looking ciphertexts!
//! ```
//!
//! ### Encrypt-and-MAC (E&M) — INSECURE (used by SSH)
//! ```text
//! Encrypt:  ciphertext = Encrypt(key_e, plaintext)
//!           tag = MAC(key_m, plaintext)
//!           → MAC leaks information about plaintext!
//! ```
//!
//! AEAD modes (GCM, Poly1305) handle all this internally and correctly.
//!
//! ## Attack Scenario: MAC-then-Encrypt (Padding Oracle)
//!
//! In TLS 1.2 with CBC mode:
//! 1. Server decrypts, THEN checks MAC
//! 2. Different error for "bad padding" vs "bad MAC" leaks plaintext info
//! 3. Attacker sends ~128 ciphertexts per byte to recover plaintext
//!
//! AEAD modes (GCM, Poly1305) verify the tag BEFORE returning plaintext,
//! making this attack impossible.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};

/// Exercise 1: Demonstrate encrypt-then-MAC manually.
///
/// Implement a manual encrypt-then-MAC scheme using AES-CBC for encryption
/// and HMAC-SHA256 for the MAC. This shows why AEAD is better — you have
/// to get the composition right yourself.
///
/// Steps:
/// 1. Encrypt plaintext with AES-256 (we'll use AES-GCM in CTR-like mode for simplicity)
/// 2. Compute HMAC-SHA256 over the ciphertext
/// 3. Return ciphertext || tag
///
/// For simplicity, we use AES-GCM with a zeroed nonce to simulate basic encryption,
/// then apply HMAC on top. This demonstrates the EtM pattern.
///
/// Hints:
/// - Encrypt with AES-GCM (nonce = 12 zero bytes)
/// - Use `ring::hmac` to compute HMAC-SHA256 over the ciphertext
/// - Return ciphertext || hmac_tag (32 bytes for SHA256)
pub fn encrypt_then_mac(
    enc_key: &Key<Aes256Gcm>,
    mac_key: &[u8; 32],
    plaintext: &[u8],
) -> Vec<u8> {
    todo!("Implement encrypt-then-MAC pattern")
}

/// Exercise 2: Verify and decrypt encrypt-then-MAC.
///
/// Steps:
/// 1. Split the input into ciphertext and tag (last 32 bytes)
/// 2. Verify HMAC-SHA256 over the ciphertext using the MAC key
/// 3. If MAC is valid, decrypt and return plaintext
/// 4. If MAC is invalid, return an error
///
/// CRITICAL: Always verify the MAC BEFORE decrypting. This prevents
/// chosen-ciphertext attacks.
///
/// Hints:
/// - Split: `let (ciphertext, tag) = data.split_at(data.len() - 32);`
/// - Verify HMAC with `ring::hmac::verify`
/// - Decrypt only if MAC is valid
pub fn verify_then_decrypt(
    enc_key: &Key<Aes256Gcm>,
    mac_key: &[u8; 32],
    data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    todo!("Verify MAC, then decrypt (EtM)")
}

/// Exercise 3: Demonstrate that AEAD rejects tampered ciphertext.
///
/// This function encrypts plaintext, tampers with the ciphertext, and
/// verifies that AEAD decryption fails. Returns true if tampering was detected.
///
/// Hints:
/// - Encrypt with AES-256-GCM
/// - Flip bits in the ciphertext
/// - Attempt decryption — should return Err
pub fn demonstrate_aead_tamper_detection(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> bool {
    todo!("Show that AEAD detects any tampering")
}

/// Exercise 4: Demonstrate a "forbidden attack" on non-authenticated encryption.
///
/// Without authentication, an attacker can flip specific bits in the ciphertext
/// to produce controlled changes in the decrypted plaintext (CBC bit-flipping attack).
///
/// This function simulates the attack: encrypt with AES-GCM (which IS authenticated),
/// then show that WITHOUT the authentication tag check, bit flipping would succeed.
///
/// Returns (modified_plaintext, would_aead_detect_it) where:
/// - modified_plaintext is what you'd get if you skipped authentication
/// - would_aead_detect_it is always true (GCM catches it)
///
/// Hints:
/// - Encrypt plaintext with GCM
/// - Tamper with the ciphertext
/// - Show that decryption fails with GCM (authentication catches it)
/// - Return (b"tampered_simulated", true)
pub fn demonstrate_bit_flip_attack(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> (Vec<u8>, bool) {
    todo!("Demonstrate bit-flip attack and AEAD defense")
}

/// Exercise 5: Implement "seal" and "open" — a high-level AEAD API.
///
/// This is the kind of API you'd build for a real application:
/// - `seal` encrypts and authenticates, returning a self-contained package
/// - `open` verifies and decrypts, rejecting any tampering
///
/// The sealed package format: [nonce: 12 bytes] [ciphertext_with_tag]
///
/// Hints:
/// - Generate a fresh nonce for each seal
/// - Prepend the nonce to the ciphertext
/// - On open, split nonce from ciphertext, then decrypt
pub fn seal(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    todo!("High-level seal: nonce + encrypted data")
}

pub fn open(key: &Key<Aes256Gcm>, sealed: &[u8]) -> Result<Vec<u8>, &'static str> {
    todo!("High-level open: verify nonce + decrypt")
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
        // Tamper with the ciphertext portion (before the MAC)
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
        // Different nonces → different sealed packages
        // (even though plaintext is the same)
        assert_ne!(s1, s2, "Sealed packages should differ (different nonces)");
    }
}
