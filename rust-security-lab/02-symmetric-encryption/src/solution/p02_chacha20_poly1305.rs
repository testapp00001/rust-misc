//! # Lesson 02: ChaCha20-Poly1305 (Reference Solution)
//!
//! See the exercise file for full documentation.

use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Key, Nonce},
    ChaCha20Poly1305,
};

/// Generate a random 256-bit key for ChaCha20-Poly1305.
pub fn generate_key() -> Key<ChaCha20Poly1305> {
    ChaCha20Poly1305::generate_key(&mut OsRng)
}

/// Generate a random 96-bit nonce for ChaCha20-Poly1305.
pub fn generate_nonce() -> Nonce<ChaCha20Poly1305> {
    ChaCha20Poly1305::generate_nonce(&mut OsRng)
}

/// Encrypt plaintext with ChaCha20-Poly1305.
///
/// The 16-byte Poly1305 authentication tag is appended to the ciphertext.
pub fn encrypt(key: &Key<ChaCha20Poly1305>, plaintext: &[u8]) -> (Vec<u8>, Nonce<ChaCha20Poly1305>) {
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = generate_nonce();
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("Encryption should not fail");
    (ciphertext, nonce)
}

/// Decrypt ciphertext with ChaCha20-Poly1305.
///
/// Returns an error if the Poly1305 tag doesn't verify.
pub fn decrypt(
    key: &Key<ChaCha20Poly1305>,
    nonce: &Nonce<ChaCha20Poly1305>,
    ciphertext: &[u8],
) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
    let cipher = ChaCha20Poly1305::new(key);
    cipher.decrypt(nonce, ciphertext)
}

/// Demonstrate interoperability: decrypt chacha20poly1305 ciphertext with ring.
///
/// Both crates implement RFC 8439 (ChaCha20-Poly1305) identically.
/// ring's `aead::CHACHA20_POLY1305` expects the nonce and ciphertext
/// concatenated as `[nonce || ciphertext_with_tag]`.
pub fn decrypt_with_ring(key_bytes: &[u8; 32], nonce_bytes: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    let unbound_key = ring::aead::UnboundKey::new(&ring::aead::CHACHA20_POLY1305, key_bytes)
        .map_err(|_| ())?;
    let sealing_key = ring::aead::LessSafeKey::new(unbound_key);

    // ring expects nonce || ciphertext || tag as one input for opening
    let mut in_out = nonce_bytes.to_vec();
    in_out.extend_from_slice(ciphertext);

    let nonce = ring::aead::Nonce::assume_unique_for_key(*nonce_bytes);
    let tag_len = 16; // Poly1305 tag is 16 bytes

    // Separate ciphertext and tag
    if in_out.len() < 12 + tag_len {
        return Err(());
    }
    let ciphertext_with_tag = &in_out[12..];
    let mut ct_and_tag = ciphertext_with_tag.to_vec();

    let plaintext = sealing_key
        .open_in_place(
            ring::aead::Nonce::assume_unique_for_key(*nonce_bytes),
            ring::aead::Aad::empty(),
            &mut ct_and_tag,
        )
        .map_err(|_| ())?;

    let pt_len = plaintext.len();
    Ok(ct_and_tag[..pt_len].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = generate_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_nonce_generation() {
        let nonce = generate_nonce();
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_key();
        let plaintext = b"Hello from ChaCha20-Poly1305!";
        let (ciphertext, nonce) = encrypt(&key, plaintext);
        assert_ne!(&ciphertext[..plaintext.len()], plaintext);
        assert_eq!(ciphertext.len(), plaintext.len() + 16);
        let decrypted = decrypt(&key, &nonce, &ciphertext).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_large_plaintext() {
        let key = generate_key();
        let plaintext = vec![0xABu8; 10_000];
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
        assert!(decrypt(&key, &nonce, &ciphertext).is_err());
    }

    #[test]
    fn test_ring_interop() {
        let key = generate_key();
        let key_bytes: [u8; 32] = key.into();
        let plaintext = b"interop test";

        let key2 = Key::<ChaCha20Poly1305>::from_slice(&key_bytes);
        let (ciphertext, used_nonce) = encrypt(key2, plaintext);
        let nonce_bytes: [u8; 12] = used_nonce.into();

        let decrypted = decrypt_with_ring(&key_bytes, &nonce_bytes, &ciphertext)
            .expect("Ring decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }
}
