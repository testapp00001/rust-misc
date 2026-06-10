//! # Lesson 01: RSA Basics — Key Generation, Encrypt, Decrypt (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rsa::{RsaPrivateKey, RsaPublicKey, Oaep, traits::PublicKeyParts};
use rand::rngs::OsRng;

/// Generate an RSA key pair with the given bit size.
///
/// `RsaPrivateKey::new` generates two random primes of the appropriate size,
/// computes N = p*q, and derives the public/private key components.
///
/// # Security Notes
/// - Always use >= 2048 bits (NIST recommendation through 2030)
/// - `OsRng` uses the operating system's CSPRNG (/dev/urandom, CryptGenRandom)
/// - Key generation is the slowest RSA operation (~1s for 4096-bit keys)
pub fn generate_keypair(bits: usize) -> (RsaPublicKey, RsaPrivateKey) {
    let private_key = RsaPrivateKey::new(&mut OsRng, bits).expect("key generation failed");
    let public_key = private_key.to_public_key();
    (public_key, private_key)
}

/// Encrypt a message using RSA-OAEP with SHA-256.
///
/// OAEP (Optimal Asymmetric Encryption Padding) adds:
/// - Randomness (same plaintext -> different ciphertexts)
/// - Structure (detects tampering)
/// - CCA security (resists chosen-ciphertext attacks)
///
/// The padding scheme `Oaep::new::<sha2::Sha256>()` uses SHA-256 for both
/// the hash and the mask generation function (MGF1).
pub fn encrypt(public_key: &RsaPublicKey, message: &[u8]) -> Vec<u8> {
    let padding = Oaep::new::<sha2::Sha256>();
    public_key
        .encrypt(&mut OsRng, padding, message)
        .expect("encryption failed")
}

/// Decrypt a message using RSA-OAEP with SHA-256.
///
/// If the ciphertext has been tampered with, or if the wrong key is used,
/// decryption fails with an error. OAEP ensures that the error gives the
/// attacker NO information about the plaintext (unlike PKCS#1 v1.5).
pub fn decrypt(private_key: &RsaPrivateKey, ciphertext: &[u8]) -> Vec<u8> {
    let padding = Oaep::new::<sha2::Sha256>();
    private_key
        .decrypt(padding, ciphertext)
        .expect("decryption failed — wrong key or tampered ciphertext")
}

/// Demonstrate that OAEP is non-malleable.
///
/// Flipping a bit in the ciphertext should cause decryption to fail entirely,
/// not produce corrupted plaintext. This is a critical security property —
/// without it, attackers can modify encrypted messages.
pub fn detect_tampering(public_key: &RsaPublicKey, private_key: &RsaPrivateKey, message: &[u8]) -> bool {
    let ciphertext = encrypt(public_key, message);

    // Tamper with the ciphertext (flip a bit)
    let mut tampered = ciphertext.clone();
    if !tampered.is_empty() {
        tampered[0] ^= 0x01;
    }

    // Try to decrypt — should fail
    let padding = Oaep::new::<sha2::Sha256>();
    private_key.decrypt(padding, &tampered).is_err()
}

/// Get the key size in bits from an RSA public key.
///
/// `RsaPublicKey::size()` returns the size in bytes (the byte length of the modulus N).
/// Multiply by 8 to get bits.
pub fn key_size_bits(public_key: &RsaPublicKey) -> usize {
    public_key.size() * 8
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
