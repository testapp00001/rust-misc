//! # Lesson 09: Additional Authenticated Data (Reference Solution)
//!
//! See the exercise file for full documentation on AAD, context binding,
//! and replay attack prevention.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce, Payload},
    Aes256Gcm, Key,
};

/// Encrypt plaintext with additional authenticated data (AAD).
///
/// The AAD is authenticated but NOT encrypted — it's visible to everyone.
/// Any modification of AAD causes decryption to fail.
pub fn encrypt_with_aad(
    key: &Key<Aes256Gcm>,
    plaintext: &[u8],
    aad: &[u8],
) -> (Vec<u8>, aes_gcm::aead::Nonce<Aes256Gcm>) {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let payload = Payload {
        msg: plaintext,
        aad,
    };

    let ciphertext = cipher.encrypt(&nonce, payload).expect("Encryption failed");
    (ciphertext, nonce)
}

/// Decrypt ciphertext with AAD verification.
///
/// If the AAD doesn't match what was used during encryption, decryption fails.
pub fn decrypt_with_aad(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(key);

    let payload = Payload {
        msg: ciphertext,
        aad,
    };

    cipher.decrypt(nonce, payload)
}

/// Demonstrate that modified AAD causes decryption to fail.
pub fn demonstrate_aad_tamper_detection(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> bool {
    let aad = b"original context";
    let (ciphertext, nonce) = encrypt_with_aad(key, plaintext, aad);

    // Try to decrypt with modified AAD
    let tampered_aad = b"modified context";
    let result = decrypt_with_aad(key, &nonce, &ciphertext, tampered_aad);

    // Should fail — AAD mismatch
    result.is_err()
}

/// Seal a message bound to a specific context (AAD).
///
/// Format: [nonce: 12 bytes] [ciphertext+tag]
/// The context is NOT stored — it must be provided during decryption.
pub fn seal_with_context(key: &Key<Aes256Gcm>, plaintext: &[u8], context: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let payload = Payload {
        msg: plaintext,
        aad: context,
    };

    let ciphertext = cipher.encrypt(&nonce, payload).expect("Encryption failed");

    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    output
}

/// Open a sealed message and verify context.
///
/// The context must match exactly — any change causes decryption to fail.
pub fn open_with_context(
    key: &Key<Aes256Gcm>,
    sealed: &[u8],
    context: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if sealed.len() < 12 {
        return Err("Sealed data too short");
    }

    let (nonce_bytes, ciphertext) = sealed.split_at(12);
    let nonce = Nonce::<Aes256Gcm>::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(key);

    let payload = Payload {
        msg: ciphertext,
        aad: context,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|_| "Decryption failed — context mismatch or tampered data")
}

/// Demonstrate replay attack prevention with AAD.
///
/// A ciphertext encrypted for one context cannot be used in another.
pub fn demonstrate_replay_prevention(key: &Key<Aes256Gcm>) -> bool {
    let plaintext = b"$100 transfer";
    let context_alice = b"user:alice";
    let context_bob = b"user:bob";

    // Seal for Alice's context
    let sealed = seal_with_context(key, plaintext, context_alice);

    // Try to open in Bob's context — must fail
    let result = open_with_context(key, &sealed, context_bob);
    result.is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Key<Aes256Gcm> {
        Aes256Gcm::generate_key(&mut OsRng)
    }

    #[test]
    fn test_encrypt_decrypt_with_aad() {
        let key = test_key();
        let plaintext = b"secret message";
        let aad = b"context: user=alice, txn=42";

        let (ciphertext, nonce) = encrypt_with_aad(&key, plaintext, aad);
        let decrypted = decrypt_with_aad(&key, &nonce, &ciphertext, aad).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_aad_fails() {
        let key = test_key();
        let plaintext = b"secret message";
        let aad = b"correct context";

        let (ciphertext, nonce) = encrypt_with_aad(&key, plaintext, aad);
        let wrong_aad = b"wrong context";
        let result = decrypt_with_aad(&key, &nonce, &ciphertext, wrong_aad);
        assert!(result.is_err(), "Wrong AAD must fail decryption");
    }

    #[test]
    fn test_no_aad_works() {
        let key = test_key();
        let plaintext = b"no aad message";
        let aad = b"";

        let (ciphertext, nonce) = encrypt_with_aad(&key, plaintext, aad);
        let decrypted = decrypt_with_aad(&key, &nonce, &ciphertext, aad).expect("Failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aad_tamper_detection() {
        let key = test_key();
        let detected = demonstrate_aad_tamper_detection(&key, b"AAD protects context");
        assert!(detected, "Modified AAD must be detected");
    }

    #[test]
    fn test_seal_open_with_context() {
        let key = test_key();
        let plaintext = b"context-bound data";
        let context = b"row_id=12345";

        let sealed = seal_with_context(&key, plaintext, context);
        let opened = open_with_context(&key, &sealed, context).expect("Open failed");
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn test_seal_open_wrong_context() {
        let key = test_key();
        let plaintext = b"context-bound data";
        let context = b"row_id=12345";

        let sealed = seal_with_context(&key, plaintext, context);
        let result = open_with_context(&key, &sealed, b"row_id=99999");
        assert!(result.is_err(), "Wrong context must fail");
    }

    #[test]
    fn test_replay_prevention() {
        let key = test_key();
        let prevented = demonstrate_replay_prevention(&key);
        assert!(prevented, "AAD should prevent replay across contexts");
    }

    #[test]
    fn test_aad_visible_in_transit() {
        let key = test_key();
        let plaintext = b"encrypted content";
        let aad = b"public metadata";

        let (ciphertext, nonce) = encrypt_with_aad(&key, plaintext, aad);

        // AAD is not in the ciphertext
        assert_eq!(ciphertext.len(), plaintext.len() + 16);
        let decrypted = decrypt_with_aad(&key, &nonce, &ciphertext, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
