//! # Lesson 10: Hybrid Encryption
//!
//! ## What is Hybrid Encryption?
//!
//! Hybrid encryption combines the best of both worlds:
//! - **Asymmetric (public-key) encryption**: Used to securely exchange a symmetric key
//! - **Symmetric encryption**: Used to encrypt the actual data (fast, efficient)
//!
//! ```text
//! Sender:
//!   1. Generate random symmetric key K
//!   2. Encrypt data with K using AES-GCM (fast)
//!   3. Encrypt K with recipient's public key (RSA/ECIES)
//!   4. Send: encrypted_K || encrypted_data
//!
//! Recipient:
//!   1. Decrypt K with their private key
//!   2. Decrypt data with K using AES-GCM
//! ```
//!
//! ## Why Hybrid?
//!
//! | Property | Symmetric Only | Asymmetric Only | Hybrid |
//! |----------|---------------|-----------------|--------|
//! | Key exchange | Hard (need secure channel) | Easy (public key) | Easy |
//! | Speed | Fast | Very slow | Fast |
//! | Data size limit | None | Small (~200 bytes for RSA) | None |
//! | Used in | Pre-shared keys | Key exchange only | TLS, PGP, age, Signal |
//!
//! ## Real-World Hybrid Encryption
//!
//! - **TLS 1.3**: ECDHE for key exchange + AES-GCM/ChaCha20 for data
//! - **PGP/GPG**: RSA for key + AES for message
//! - **age**: X25519 for key exchange + ChaCha20-Poly1305 for data
//! - **Signal Protocol**: X3DH for key exchange + AES-256-CBC + HMAC-SHA256
//!
//! ## Attack Scenario: RSA Alone for Large Data
//!
//! RSA-2048 can only encrypt ~245 bytes. For larger data, you'd need to:
//! 1. Split into chunks and encrypt each with RSA (very slow)
//! 2. Use ECB-like mode (leaks patterns)
//! 3. Risk padding oracle attacks on each chunk
//!
//! Hybrid encryption avoids all these problems.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};
use ring::agreement;
use ring::rand as ring_rand;

/// Exercise 1: Implement a simple hybrid encryption scheme.
///
/// This simulates hybrid encryption without full asymmetric crypto.
/// Instead of RSA, we use a "key wrapping" pattern: the symmetric key
/// is XORed with a shared secret (simulating key exchange).
///
/// In a real system:
/// - The "key_wrap" would be RSA-OAEP or ECIES encryption of the symmetric key
/// - The "shared_secret" would be the recipient's public key
///
/// Returns: [wrapped_key: 32 bytes] [nonce: 12 bytes] [ciphertext+tag]
pub fn hybrid_encrypt(
    shared_secret: &[u8; 32],
    plaintext: &[u8],
) -> Vec<u8> {
    todo!("Implement hybrid encryption: wrap key + encrypt data")
}

/// Exercise 2: Implement hybrid decryption.
///
/// Unwrap the symmetric key and decrypt the data.
///
/// Hints:
/// - Split the input: wrapped_key (32), nonce (12), ciphertext (rest)
/// - Unwrap the key by XORing with the shared_secret
/// - Decrypt with the unwrapped key
pub fn hybrid_decrypt(
    shared_secret: &[u8; 32],
    data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    todo!("Implement hybrid decryption: unwrap key + decrypt data")
}

/// Exercise 3: Implement ECDH-based key agreement (simulated).
///
/// In a real hybrid system, the symmetric key is derived from a
/// Diffie-Hellman key exchange. This function simulates ECDH by
/// using ring's X25519 agreement.
///
/// Returns a 32-byte shared secret derived from the agreement.
///
/// Hints:
/// - Generate an ephemeral keypair with `agreement::EphemeralPrivateKey::generate`
/// - The "recipient" also generates a keypair
/// - Compute the shared secret via `agreement::agree_ephemeral`
/// - Return the raw key bytes (in production, derive via HKDF)
///
/// Note: For this exercise, return the raw bytes as a [u8; 32].
/// In production, ALWAYS run the shared secret through HKDF.
pub fn simulate_ecdh_key_exchange() -> [u8; 32] {
    todo!("Simulate ECDH key exchange to produce a shared secret")
}

/// Exercise 4: Implement hybrid encryption with ephemeral keys.
///
/// Each message uses a fresh ephemeral key pair. The sender:
/// 1. Generates an ephemeral X25519 keypair
/// 2. Computes shared secret with recipient's public key
/// 3. Derives a symmetric key from the shared secret
/// 4. Encrypts with AES-GCM
///
/// Returns: [ephemeral_public_key: 32 bytes] [nonce: 12 bytes] [ciphertext+tag]
///
/// For simplicity, we simulate the key exchange using XOR of a
/// "public key" with a "private key" to produce the shared secret.
pub fn ephemeral_hybrid_encrypt(
    recipient_public: &[u8; 32],
    plaintext: &[u8],
) -> Vec<u8> {
    todo!("Encrypt with ephemeral key pair")
}

/// Exercise 5: Decrypt ephemeral hybrid ciphertext.
///
/// Parse the ephemeral public key from the message, compute the
/// shared secret, and decrypt.
pub fn ephemeral_hybrid_decrypt(
    recipient_private: &[u8; 32],
    data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    todo!("Decrypt using ephemeral hybrid scheme")
}

/// Exercise 6: Demonstrate forward secrecy concept.
///
/// Forward secrecy means that compromise of long-term keys doesn't
/// compromise past session keys. In ephemeral hybrid encryption:
/// - Each message uses a fresh ephemeral key
/// - The ephemeral private key is deleted after use
/// - Even if the long-term key leaks, past messages can't be decrypted
///
/// This function returns true if the demonstration succeeds:
/// 1. Encrypt two messages with different ephemeral keys
/// 2. Show that knowing one shared secret doesn't help decrypt the other
///
/// Hints:
/// - Encrypt message 1 with shared_secret_1
/// - Encrypt message 2 with shared_secret_2
/// - Show that decrypting message 1 with shared_secret_2 fails
pub fn demonstrate_forward_secrecy() -> bool {
    todo!("Show that ephemeral keys provide forward secrecy")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_secret() -> [u8; 32] {
        let mut secret = [0u8; 32];
        for byte in secret.iter_mut() {
            *byte = rand::random();
        }
        secret
    }

    #[test]
    fn test_hybrid_encrypt_decrypt_roundtrip() {
        let shared_secret = random_secret();
        let plaintext = b"hybrid encryption test message";

        let encrypted = hybrid_encrypt(&shared_secret, plaintext);
        let decrypted = hybrid_decrypt(&shared_secret, &encrypted).expect("Hybrid decrypt failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_hybrid_wrong_secret_fails() {
        let secret1 = random_secret();
        let secret2 = random_secret();
        let encrypted = hybrid_encrypt(&secret1, b"secret data");

        let result = hybrid_decrypt(&secret2, &encrypted);
        assert!(result.is_err(), "Wrong shared secret must fail");
    }

    #[test]
    fn test_hybrid_encrypt_large_data() {
        let shared_secret = random_secret();
        let plaintext = vec![0xCDu8; 100_000]; // 100KB

        let encrypted = hybrid_encrypt(&shared_secret, &plaintext);
        let decrypted = hybrid_decrypt(&shared_secret, &encrypted).expect("Failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_simulate_ecdh() {
        let secret = simulate_ecdh_key_exchange();
        assert_eq!(secret.len(), 32);
        // Should not be all zeros
        assert!(secret.iter().any(|&b| b != 0), "Shared secret should have entropy");
    }

    #[test]
    fn test_ephemeral_hybrid_roundtrip() {
        let recipient_private = random_secret();
        // In this simulation, "public key" = private key XOR a fixed derivation
        // (In real ECDH, public = scalar_mult(private, base_point))
        let mut recipient_public = recipient_private;
        for byte in recipient_public.iter_mut() {
            *byte ^= 0xAA;
        }

        let plaintext = b"ephemeral key test";
        let encrypted = ephemeral_hybrid_encrypt(&recipient_public, plaintext);
        let decrypted = ephemeral_hybrid_decrypt(&recipient_private, &encrypted)
            .expect("Ephemeral decrypt failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ephemeral_wrong_key_fails() {
        let recipient_private = random_secret();
        let mut recipient_public = recipient_private;
        for byte in recipient_public.iter_mut() {
            *byte ^= 0xAA;
        }

        let encrypted = ephemeral_hybrid_encrypt(&recipient_public, b"secret");

        let wrong_private = random_secret();
        let result = ephemeral_hybrid_decrypt(&wrong_private, &encrypted);
        assert!(result.is_err(), "Wrong private key must fail");
    }

    #[test]
    fn test_forward_secrecy() {
        let result = demonstrate_forward_secrecy();
        assert!(result, "Forward secrecy should be demonstrated");
    }

    #[test]
    fn test_hybrid_empty_plaintext() {
        let shared_secret = random_secret();
        let encrypted = hybrid_encrypt(&shared_secret, b"");
        let decrypted = hybrid_decrypt(&shared_secret, &encrypted).expect("Failed");
        assert_eq!(decrypted, b"");
    }
}
