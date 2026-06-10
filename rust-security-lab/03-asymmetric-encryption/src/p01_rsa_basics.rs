//! # Lesson 01: RSA Basics — Key Generation, Encrypt, Decrypt
//!
//! ## What is RSA?
//!
//! RSA (Rivest-Shamir-Adleman) is the oldest widely-used public-key cryptosystem.
//! Its security rests on the difficulty of factoring large numbers: given N = p*q
//! (where p and q are large primes), it's computationally infeasible to recover p and q.
//!
//! ## How RSA Works
//!
//! 1. **Key Generation**: Pick two large primes p, q. Compute N = p*q.
//!    Choose e (commonly 65537). Compute d = e^(-1) mod phi(N).
//!    Public key: (N, e). Private key: (N, d).
//!
//! 2. **Encryption**: c = m^e mod N (but NEVER use textbook RSA!)
//!
//! 3. **Decryption**: m = c^d mod N
//!
//! ## Why OAEP Padding Matters
//!
//! **Textbook RSA** (m^e mod N) is DANGEROUS:
//! - Deterministic: same plaintext always produces same ciphertext
//! - Malleable: attacker can modify ciphertext to change plaintext
//! - Vulnerable to chosen-ciphertext attacks (Bleichenbacher)
//!
//! **OAEP** (Optimal Asymmetric Encryption Padding) adds randomness and structure:
//! - Randomized: same plaintext produces different ciphertexts each time
//! - Non-malleable: modifying ciphertext makes decryption fail completely
//! - CCA-secure: resistant to chosen-ciphertext attacks
//!
//! ## Attack Demo: Textbook RSA is Malleable
//!
//! Given c = m^e mod N, an attacker can compute c' = c * 2^e mod N.
//! Decryption of c' gives 2m — the attacker doubled the plaintext without knowing m!
//! OAEP prevents this because any modification causes decryption to fail entirely.

use rsa::{RsaPrivateKey, RsaPublicKey, Oaep};
use rand::rngs::OsRng;

/// Exercise 1: Generate an RSA key pair with the given bit size.
///
/// Hints:
/// - Use `RsaPrivateKey::new(&mut rng, bits)` where bits is the key size
/// - This returns an `RsaPrivateKey` which contains the public key
/// - Use `.to_public_key()` to extract the public key
/// - Use `OsRng` for the random number generator (it's cryptographically secure)
pub fn generate_keypair(bits: usize) -> (RsaPublicKey, RsaPrivateKey) {
    todo!("Generate RSA keypair with the given bit size")
}

/// Exercise 2: Encrypt a message using RSA-OAEP with SHA-256.
///
/// Hints:
/// - Create an OAEP padding scheme: `Oaep::new::<sha2::Sha256>()`
/// - Use `public_key.encrypt(&mut rng, padding, message)`
/// - Use `OsRng` for randomness
/// - Returns `Vec<u8>` ciphertext
pub fn encrypt(public_key: &RsaPublicKey, message: &[u8]) -> Vec<u8> {
    todo!("Encrypt message using RSA-OAEP with SHA-256")
}

/// Exercise 3: Decrypt a message using RSA-OAEP with SHA-256.
///
/// Hints:
/// - Create the same OAEP padding scheme used for encryption
/// - Use `private_key.decrypt(padding, ciphertext)`
/// - Returns `Vec<u8>` plaintext, or an error if decryption fails
pub fn decrypt(private_key: &RsaPrivateKey, ciphertext: &[u8]) -> Vec<u8> {
    todo!("Decrypt ciphertext using RSA-OAEP with SHA-256")
}

/// Exercise 4: Demonstrate that OAEP is non-malleable.
///
/// Given a ciphertext, try to tamper with it (flip a bit) and verify that
/// decryption fails (returns an error, not corrupted plaintext).
///
/// Hints:
/// - Encrypt a message, flip a bit in the ciphertext, try to decrypt
/// - The decryption should return an `Err`, not corrupted data
/// - Return `true` if tampering is detected (decryption fails)
pub fn detect_tampering(public_key: &RsaPublicKey, private_key: &RsaPrivateKey, message: &[u8]) -> bool {
    todo!("Demonstrate OAEP tamper detection")
}

/// Exercise 5: Get the key size in bits from an RSA public key.
///
/// Hints:
/// - Use `public_key.size()` to get the key size in bytes
/// - Multiply by 8 to get bits
pub fn key_size_bits(public_key: &RsaPublicKey) -> usize {
    todo!("Return key size in bits")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair_2048() {
        let (pub_key, _priv_key) = generate_keypair(2048);
        assert_eq!(key_size_bits(&pub_key), 2048);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (pub_key, priv_key) = generate_keypair(2048);
        let message = b"Hello, RSA!";
        let ciphertext = encrypt(&pub_key, message);
        let decrypted = decrypt(&priv_key, &ciphertext);
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let (pub_key, priv_key) = generate_keypair(2048);
        let message = b"";
        let ciphertext = encrypt(&pub_key, message);
        let decrypted = decrypt(&priv_key, &ciphertext);
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_oaep_randomization() {
        let (pub_key, _priv_key) = generate_keypair(2048);
        let message = b"same message";
        let ct1 = encrypt(&pub_key, message);
        let ct2 = encrypt(&pub_key, message);
        // OAEP is randomized — same plaintext should produce different ciphertexts
        assert_ne!(ct1, ct2, "OAEP should produce different ciphertexts for same plaintext");
    }

    #[test]
    fn test_tampering_detected() {
        let (pub_key, priv_key) = generate_keypair(2048);
        let message = b"sensitive data";
        assert!(detect_tampering(&pub_key, &priv_key, message),
            "OAEP should detect ciphertext tampering");
    }

    #[test]
    fn test_wrong_key_fails() {
        let (pub_key1, _priv_key1) = generate_keypair(2048);
        let (_pub_key2, priv_key2) = generate_keypair(2048);
        let message = b"secret";
        let ciphertext = encrypt(&pub_key1, message);
        // Decryption with wrong key should fail
        let result = std::panic::catch_unwind(|| {
            decrypt(&priv_key2, &ciphertext)
        });
        assert!(result.is_err(), "Decryption with wrong key should fail");
    }

    #[test]
    fn test_key_size_2048() {
        let (pub_key, _) = generate_keypair(2048);
        assert_eq!(key_size_bits(&pub_key), 2048);
    }

    #[test]
    fn test_encrypt_larger_message() {
        let (pub_key, priv_key) = generate_keypair(2048);
        // RSA-2048 with OAEP SHA-256 can encrypt up to 190 bytes
        let message = vec![42u8; 100];
        let ciphertext = encrypt(&pub_key, &message);
        let decrypted = decrypt(&priv_key, &ciphertext);
        assert_eq!(decrypted, message);
    }
}
