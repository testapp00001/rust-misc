//! # Lesson 10: Hybrid Encryption (Reference Solution)
//!
//! See the exercise file for full documentation on hybrid encryption,
//! ECDH key exchange, and forward secrecy.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};
use ring::agreement;
use ring::rand as ring_rand;

/// Hybrid encryption: wrap a symmetric key with a shared secret, then encrypt data.
///
/// In a real system, the "wrapping" would use RSA-OAEP or ECIES.
/// Here we simulate it with XOR of a random key with the shared secret.
///
/// Format: [wrapped_key: 32 bytes] [nonce: 12 bytes] [ciphertext+tag]
pub fn hybrid_encrypt(
    shared_secret: &[u8; 32],
    plaintext: &[u8],
) -> Vec<u8> {
    // Step 1: Generate a random symmetric key
    let sym_key = Aes256Gcm::generate_key(&mut OsRng);

    // Step 2: "Wrap" the symmetric key by XORing with the shared secret
    // In production, this would be RSA-OAEP or ECIES encryption
    let mut wrapped_key = [0u8; 32];
    for i in 0..32 {
        wrapped_key[i] = sym_key[i] ^ shared_secret[i];
    }

    // Step 3: Encrypt the data with the symmetric key
    let cipher = Aes256Gcm::new(&sym_key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("Encryption failed");

    // Step 4: Package: wrapped_key || nonce || ciphertext
    let mut output = Vec::with_capacity(32 + 12 + ciphertext.len());
    output.extend_from_slice(&wrapped_key);
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    output
}

/// Hybrid decryption: unwrap the symmetric key, then decrypt data.
pub fn hybrid_decrypt(
    shared_secret: &[u8; 32],
    data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if data.len() < 44 {
        return Err("Data too short: need at least 32 (key) + 12 (nonce) bytes");
    }

    // Step 1: Parse wrapped key, nonce, ciphertext
    let wrapped_key = &data[..32];
    let nonce = Nonce::<Aes256Gcm>::from_slice(&data[32..44]);
    let ciphertext = &data[44..];

    // Step 2: Unwrap the symmetric key
    let mut sym_key_bytes = [0u8; 32];
    for i in 0..32 {
        sym_key_bytes[i] = wrapped_key[i] ^ shared_secret[i];
    }
    let sym_key = Key::<Aes256Gcm>::from_slice(&sym_key_bytes);

    // Step 3: Decrypt
    let cipher = Aes256Gcm::new(sym_key);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed — wrong key or tampered data")
}

/// Simulate ECDH key exchange using ring's X25519.
///
/// In a real system, this produces a shared secret from two parties'
/// keypairs. Here we demonstrate the concept with ring's agreement API.
///
/// Returns a 32-byte shared secret. In production, derive via HKDF.
pub fn simulate_ecdh_key_exchange() -> [u8; 32] {
    let rng = ring_rand::SystemRandom::new();

    // Alice generates an ephemeral keypair
    let alice_private = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng)
        .expect("Key generation failed");
    let alice_public = alice_private.compute_public_key().expect("Public key computation failed");

    // Bob generates an ephemeral keypair
    let bob_private = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng)
        .expect("Key generation failed");
    let bob_public = bob_private.compute_public_key().expect("Public key computation failed");

    // Alice computes shared secret using Bob's public key
    let shared_secret = agreement::agree_ephemeral(
        alice_private,
        &agreement::UnparsedPublicKey::new(&agreement::X25519, bob_public),
        |key_material| {
            // In production, use HKDF here
            let mut secret = [0u8; 32];
            let len = key_material.len().min(32);
            secret[..len].copy_from_slice(&key_material[..len]);
            secret
        },
    ).expect("Key agreement failed");

    shared_secret
}

/// Encrypt with ephemeral key pair.
///
/// Each message uses a fresh "public key" for the recipient.
/// Format: [ephemeral_public: 32 bytes] [nonce: 12 bytes] [ciphertext+tag]
///
/// This simulation uses XOR for key derivation (real ECDH uses scalar multiplication).
pub fn ephemeral_hybrid_encrypt(
    recipient_public: &[u8; 32],
    plaintext: &[u8],
) -> Vec<u8> {
    // Generate ephemeral "private key"
    let mut ephemeral_private = [0u8; 32];
    for byte in ephemeral_private.iter_mut() {
        *byte = rand::random();
    }

    // Derive shared secret: XOR of ephemeral private with recipient public
    // (In real ECDH: shared_secret = scalar_mult(ephemeral_private, recipient_public))
    let mut shared_secret = [0u8; 32];
    for i in 0..32 {
        shared_secret[i] = ephemeral_private[i] ^ recipient_public[i];
    }

    // Compute "ephemeral public key" from private (simulation)
    let mut ephemeral_public = ephemeral_private;
    for byte in ephemeral_public.iter_mut() {
        *byte ^= 0xAA;
    }

    // Encrypt with the derived key
    let sym_key = Key::<Aes256Gcm>::clone_from_slice(&shared_secret);
    let cipher = Aes256Gcm::new(&sym_key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("Encryption failed");

    // Package: ephemeral_public || nonce || ciphertext
    let mut output = Vec::with_capacity(32 + 12 + ciphertext.len());
    output.extend_from_slice(&ephemeral_public);
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    output
}

/// Decrypt ephemeral hybrid ciphertext.
///
/// Recipient uses their private key to derive the shared secret from
/// the sender's ephemeral public key.
pub fn ephemeral_hybrid_decrypt(
    recipient_private: &[u8; 32],
    data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if data.len() < 44 {
        return Err("Data too short");
    }

    // Parse: ephemeral_public (32) || nonce (12) || ciphertext (rest)
    let ephemeral_public = &data[..32];
    let nonce = Nonce::<Aes256Gcm>::from_slice(&data[32..44]);
    let ciphertext = &data[44..];

    // Derive shared secret from recipient private and ephemeral public
    // (In real ECDH: shared_secret = scalar_mult(recipient_private, ephemeral_public))
    let mut shared_secret = [0u8; 32];
    for i in 0..32 {
        // Reverse the XOR: private XOR (public XOR 0xAA) = private XOR private XOR 0xAA = 0xAA... wait
        // Actually we need to match the encrypt side:
        // Encrypt: ephemeral_public = ephemeral_private ^ 0xAA
        //          shared_secret = ephemeral_private ^ recipient_public
        // Decrypt: recipient has recipient_private, sees ephemeral_public
        //          needs to recover shared_secret
        // In our simulation: recipient_private IS the recipient_public XOR 0xAA derivation
        // So: shared_secret = (ephemeral_public ^ 0xAA) ^ recipient_public
        // But recipient only has recipient_private...
        //
        // Simpler approach: the "private key" is what the recipient knows.
        // In our XOR simulation: shared_secret = recipient_private XOR ephemeral_public
        shared_secret[i] = recipient_private[i] ^ ephemeral_public[i];
    }

    let sym_key = Key::<Aes256Gcm>::clone_from_slice(&shared_secret);
    let cipher = Aes256Gcm::new(&sym_key);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed")
}

/// Demonstrate forward secrecy with ephemeral keys.
///
/// Forward secrecy means: if the long-term key leaks, past sessions are safe.
/// With ephemeral keys, each session uses a fresh key that's deleted after use.
pub fn demonstrate_forward_secrecy() -> bool {
    // Two independent "sessions" with different shared secrets
    let secret1: [u8; 32] = rand::random();
    let secret2: [u8; 32] = rand::random();

    // Encrypt messages in different sessions
    let ct1 = hybrid_encrypt(&secret1, b"session 1 message");
    let ct2 = hybrid_encrypt(&secret2, b"session 2 message");

    // Knowing secret1 doesn't help decrypt ct2
    let result1_with_wrong_key = hybrid_decrypt(&secret1, &ct2);
    assert!(result1_with_wrong_key.is_err(), "secret1 should not decrypt session 2");

    // Each session's key is independent
    let result1 = hybrid_decrypt(&secret1, &ct1);
    let result2 = hybrid_decrypt(&secret2, &ct2);

    result1.is_ok() && result2.is_ok()
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
        assert!(secret.iter().any(|&b| b != 0), "Shared secret should have entropy");
    }

    #[test]
    fn test_ephemeral_hybrid_roundtrip() {
        let recipient_private = random_secret();
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
