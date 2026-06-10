//! # Lesson 02: ChaCha20-Poly1305
//!
//! ## What is ChaCha20-Poly1305?
//!
//! ChaCha20-Poly1305 is an AEAD cipher designed by Daniel Bernstein. It combines:
//! - **ChaCha20**: A stream cipher (replaces the Salsa20 cipher)
//! - **Poly1305**: A one-time authenticator (MAC)
//!
//! Unlike AES, it requires NO hardware acceleration to be fast. On devices without
//! AES-NI (many ARM mobile chips), ChaCha20-Poly1305 is 3-4x faster than AES-GCM.
//!
//! ## ChaCha20 vs AES-GCM
//!
//! | Property | ChaCha20-Poly1305 | AES-256-GCM |
//! |----------|-------------------|-------------|
//! | Design | Software-first | Hardware-first |
//! | Without HW accel | ~1-3 GB/s | ~100-200 MB/s |
//! | With HW accel | ~1-3 GB/s | ~1-4 GB/s |
//! | Side-channel resistance | Excellent (no cache timing) | Needs AES-NI to avoid leaks |
//! | Nonce size | 96 bits | 96 bits |
//! | Standard | RFC 8439 | NIST SP 800-38D |
//!
//! ## When to Use ChaCha20-Poly1305
//!
//! - Mobile devices (Android, iOS) where AES-NI may not be available
//! - Network protocols (TLS 1.3, WireGuard, QUIC)
//! - Software-only environments (embedded, WASM)
//! - When you want strong side-channel resistance by design
//!
//! ## Attack Scenario: Nonce Reuse (Same as GCM)
//!
//! ChaCha20-Poly1305 has the SAME catastrophic failure on nonce reuse as AES-GCM.
//! If the same (key, nonce) pair is used twice, the Poly1305 one-time key is leaked,
//! allowing the attacker to forge authentication tags for arbitrary messages.

use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Key, Nonce},
    ChaCha20Poly1305,
};

/// Exercise 1: Generate a random ChaCha20-Poly1305 key.
///
/// ChaCha20 uses a 256-bit (32-byte) key.
///
/// Hints:
/// - Use `ChaCha20Poly1305::generate_key(&mut OsRng)`
pub fn generate_key() -> Key<ChaCha20Poly1305> {
    todo!("Generate a random ChaCha20-Poly1305 key")
}

/// Exercise 2: Generate a random 96-bit nonce for ChaCha20-Poly1305.
///
/// Hints:
/// - Use `ChaCha20Poly1305::generate_nonce(&mut OsRng)`
pub fn generate_nonce() -> Nonce<ChaCha20Poly1305> {
    todo!("Generate a random 96-bit nonce")
}

/// Exercise 3: Encrypt plaintext using ChaCha20-Poly1305.
///
/// Returns (ciphertext_with_tag, nonce).
///
/// Hints:
/// - Create cipher: `ChaCha20Poly1305::new(&key)`
/// - Generate a fresh nonce
/// - Encrypt: `cipher.encrypt(&nonce, plaintext)`
pub fn encrypt(key: &Key<ChaCha20Poly1305>, plaintext: &[u8]) -> (Vec<u8>, Nonce<ChaCha20Poly1305>) {
    todo!("Encrypt with ChaCha20-Poly1305")
}

/// Exercise 4: Decrypt ciphertext using ChaCha20-Poly1305.
///
/// Hints:
/// - Create cipher: `ChaCha20Poly1305::new(&key)`
/// - Decrypt: `cipher.decrypt(&nonce, ciphertext)`
/// - Returns `Result<Vec<u8>, Error>`
pub fn decrypt(
    key: &Key<ChaCha20Poly1305>,
    nonce: &Nonce<ChaCha20Poly1305>,
    ciphertext: &[u8],
) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
    todo!("Decrypt with ChaCha20-Poly1305")
}

/// Exercise 5: Demonstrate interoperability between ring and chacha20poly1305 crates.
///
/// Both implement the same algorithm (RFC 8439), so they must produce the same output.
///
/// Hints:
/// - Encrypt with the `chacha20poly1305` crate
/// - Decrypt with `ring::aead::ChaCha20Poly1305`
/// - They should produce identical results
///
/// Note: This function takes pre-encrypted data from the chacha20poly1305 crate
/// and decrypts it with ring. The key format is the same (32 bytes).
pub fn decrypt_with_ring(key_bytes: &[u8; 32], nonce_bytes: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    todo!("Decrypt chacha20poly1305 ciphertext using ring")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = generate_key();
        assert_eq!(key.len(), 32, "ChaCha20 key must be 32 bytes");
    }

    #[test]
    fn test_nonce_generation() {
        let nonce = generate_nonce();
        assert_eq!(nonce.len(), 12, "Nonce must be 12 bytes");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_key();
        let plaintext = b"Hello from ChaCha20-Poly1305!";
        let (ciphertext, nonce) = encrypt(&key, plaintext);

        assert_ne!(&ciphertext[..plaintext.len()], plaintext);
        assert_eq!(ciphertext.len(), plaintext.len() + 16, "Poly1305 adds 16-byte tag");

        let decrypted = decrypt(&key, &nonce, &ciphertext).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_large_plaintext() {
        let key = generate_key();
        let plaintext = vec![0xABu8; 10_000]; // 10KB
        let (ciphertext, nonce) = encrypt(&key, &plaintext);
        assert_eq!(ciphertext.len(), plaintext.len() + 16);
        let decrypted = decrypt(&key, &nonce, &ciphertext).expect("Failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key();
        let key2 = generate_key();
        let (ciphertext, nonce) = encrypt(&key1, b"secret");
        assert!(decrypt(&key2, &nonce, &ciphertext).is_err());
    }

    #[test]
    fn test_wrong_nonce_fails() {
        let key = generate_key();
        let (ciphertext, _) = encrypt(&key, b"secret");
        let wrong_nonce = generate_nonce();
        assert!(decrypt(&key, &wrong_nonce, &ciphertext).is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = generate_key();
        let (mut ciphertext, nonce) = encrypt(&key, b"integrity check");
        ciphertext[0] ^= 0x01;
        assert!(decrypt(&key, &nonce, &ciphertext).is_err(), "Must detect tampering");
    }

    #[test]
    fn test_ring_interop() {
        let key = generate_key();
        let key_bytes: [u8; 32] = key.into();
        let nonce = generate_nonce();
        let nonce_bytes: [u8; 12] = nonce.into();
        let plaintext = b"interop test";

        // Encrypt with chacha20poly1305 crate
        let (ciphertext, _) = encrypt(Key::<ChaCha20Poly1305>::from_slice(&key_bytes), plaintext);

        // Decrypt with ring
        let decrypted = decrypt_with_ring(&key_bytes, &nonce_bytes, &ciphertext)
            .expect("Ring decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }
}
