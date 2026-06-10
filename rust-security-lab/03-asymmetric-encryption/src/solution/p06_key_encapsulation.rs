//! # Lesson 06: Key Encapsulation (KEM) — Wrapping Secrets with Public Keys (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.
//!
//! This implements a KEM-like pattern using X25519:
//! - Keygen: Generate X25519 keypair
//! - Encapsulate: Generate ephemeral key, ECDH -> shared secret
//! - Decapsulate: Use private key + ephemeral public key -> same shared secret

use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};
use ring::{aead, hkdf};
use rand::rngs::OsRng;
use rand::RngCore;

/// A KEM ciphertext and shared secret.
pub struct KemOutput {
    /// The ephemeral public key (sent to recipient as "ciphertext").
    pub ciphertext: Vec<u8>,
    /// The derived shared secret (32 bytes).
    pub shared_secret: [u8; 32],
}

/// Generate a KEM keypair using X25519.
///
/// Returns (public_key_bytes, secret_key_bytes).
/// The public key is shared with anyone who wants to send you encapsulated secrets.
/// The private key is used to decapsulate.
pub fn kem_keygen() -> ([u8; 32], [u8; 32]) {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    (*public.as_bytes(), secret.to_bytes())
}

/// Encapsulate a shared secret using the recipient's public key.
///
/// The sender:
/// 1. Generates an ephemeral X25519 keypair
/// 2. Performs ECDH with recipient's public key
/// 3. Derives a shared secret using HKDF
/// 4. Returns the ephemeral public key (as "ciphertext") and the shared secret
///
/// The recipient can recover the same shared secret using their private key
/// and the ephemeral public key.
pub fn kem_encapsulate(recipient_public: &[u8; 32]) -> KemOutput {
    // Generate ephemeral keypair
    let ephemeral_secret = EphemeralSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_secret);

    // ECDH with recipient's public key
    let recipient_pub = PublicKey::from(*recipient_public);
    let shared = ephemeral_secret.diffie_hellman(&recipient_pub);

    // Derive shared secret using HKDF
    let shared_secret = derive_kem_key(shared.as_bytes(), ephemeral_public.as_bytes());

    KemOutput {
        ciphertext: ephemeral_public.as_bytes().to_vec(),
        shared_secret,
    }
}

/// Decapsulate the shared secret using the recipient's private key.
///
/// The recipient:
/// 1. Parses the ciphertext as the ephemeral public key
/// 2. Performs ECDH with their private key
/// 3. Derives the same shared secret using HKDF
///
/// If the wrong private key is used, a different (wrong) shared secret is produced.
/// This will cause any subsequent decryption to fail.
pub fn kem_decapsulate(recipient_secret: &[u8; 32], ciphertext: &[u8]) -> [u8; 32] {
    // Parse ephemeral public key from ciphertext
    let ephemeral_public = PublicKey::from(
        <[u8; 32]>::try_from(ciphertext).expect("ciphertext must be 32 bytes")
    );

    // ECDH with recipient's secret
    let secret = StaticSecret::from(*recipient_secret);
    let shared = secret.diffie_hellman(&ephemeral_public);

    // Derive same shared secret
    derive_kem_key(shared.as_bytes(), ephemeral_public.as_bytes())
}

/// Verify KEM correctness: encapsulate and decapsulate produce matching secrets.
pub fn verify_kem_correctness() -> bool {
    let (pub_key, sec_key) = kem_keygen();
    let output = kem_encapsulate(&pub_key);
    let recovered = kem_decapsulate(&sec_key, &output.ciphertext);
    output.shared_secret == recovered
}

/// Show that wrong key produces wrong shared secret.
pub fn wrong_key_wrong_secret() -> bool {
    let (pub_key1, _sec_key1) = kem_keygen();
    let (_pub_key2, sec_key2) = kem_keygen();

    let output = kem_encapsulate(&pub_key1);
    let wrong_recovery = kem_decapsulate(&sec_key2, &output.ciphertext);

    output.shared_secret != wrong_recovery
}

/// Use KEM + AES-GCM to encrypt a message.
///
/// Returns (aes_ciphertext, kem_ciphertext).
/// The kem_ciphertext is sent alongside the AES ciphertext so the recipient
/// can recover the key and decrypt.
pub fn kem_encrypt_message(
    recipient_public: &[u8; 32],
    plaintext: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    let output = kem_encapsulate(recipient_public);

    // Derive AES key from shared secret
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &output.shared_secret)
        .expect("invalid key");
    let key = aead::LessSafeKey::new(unbound_key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
        .expect("encryption failed");

    // Prepend nonce to ciphertext
    let mut aes_ct = nonce_bytes.to_vec();
    aes_ct.extend_from_slice(&in_out);

    (aes_ct, output.ciphertext)
}

/// Derive a 32-byte key from KEM shared secret using HKDF.
fn derive_kem_key(shared_secret: &[u8], ephemeral_public: &[u8]) -> [u8; 32] {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, ephemeral_public);
    let prk = salt.extract(shared_secret);
    let okm = prk.expand(&[b"kem-shared-secret"], hkdf::HKDF_SHA256).unwrap();
    let mut key = [0u8; 32];
    okm.fill(&mut key).unwrap();
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kem_keygen() {
        let (pub_key, _sec_key) = kem_keygen();
        assert_eq!(pub_key.len(), 32, "Public key should be 32 bytes");
    }

    #[test]
    fn test_kem_roundtrip() {
        assert!(verify_kem_correctness(), "KEM encapsulate/decapsulate should produce matching secrets");
    }

    #[test]
    fn test_kem_shared_secret_is_32_bytes() {
        let (pub_key, _) = kem_keygen();
        let output = kem_encapsulate(&pub_key);
        assert_eq!(output.shared_secret.len(), 32);
    }

    #[test]
    fn test_kem_ciphertext_is_ephemeral_public_key() {
        let (pub_key, _) = kem_keygen();
        let output = kem_encapsulate(&pub_key);
        assert_eq!(output.ciphertext.len(), 32);
    }

    #[test]
    fn test_wrong_key_fails() {
        assert!(wrong_key_wrong_secret(), "Wrong key should produce wrong shared secret");
    }

    #[test]
    fn test_two_encapsulations_differ() {
        let (pub_key, _) = kem_keygen();
        let out1 = kem_encapsulate(&pub_key);
        let out2 = kem_encapsulate(&pub_key);
        assert_ne!(out1.shared_secret, out2.shared_secret,
            "Two encapsulations should produce different secrets");
        assert_ne!(out1.ciphertext, out2.ciphertext,
            "Two encapsulations should produce different ciphertexts");
    }

    #[test]
    fn test_kem_encrypt_decrypt() {
        let (pub_key, sec_key) = kem_keygen();
        let plaintext = b"KEM-encrypted message";
        let (aes_ct, kem_ct) = kem_encrypt_message(&pub_key, plaintext);

        // Decrypt: recover key from KEM, then decrypt AES
        let shared = kem_decapsulate(&sec_key, &kem_ct);
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &shared).unwrap();
        let key = aead::LessSafeKey::new(unbound_key);

        let nonce_bytes: [u8; 12] = aes_ct[..12].try_into().unwrap();
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = aes_ct[12..].to_vec();
        let decrypted = key.open_in_place(nonce, aead::Aad::empty(), &mut in_out).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
