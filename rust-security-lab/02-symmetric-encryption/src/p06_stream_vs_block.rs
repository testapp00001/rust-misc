//! # Lesson 06: Stream vs Block Ciphers
//!
//! ## Block Ciphers
//!
//! Block ciphers (AES) encrypt fixed-size blocks (128 bits for AES). To encrypt
//! arbitrary-length data, they use a **mode of operation**:
//!
//! | Mode | Description | Secure? |
//! |------|-------------|---------|
//! | ECB | Each block independently | NO — identical blocks → identical ciphertext |
//! | CBC | Each block XORed with previous ciphertext | Needs MAC for integrity |
//! | CTR | Turns block cipher into stream cipher | Secure with unique nonces |
//! | GCM | CTR + GHASH authentication | AEAD — secure and authenticated |
//!
//! ## Stream Ciphers
//!
//! Stream ciphers (ChaCha20) generate a keystream and XOR it with the plaintext:
//!
//! ```text
//! Keystream = ChaCha20(key, nonce, counter)
//! Ciphertext = Plaintext XOR Keystream
//! ```
//!
//! They naturally handle arbitrary-length data without padding.
//!
//! ## Block Cipher as Stream Cipher (CTR Mode)
//!
//! AES-CTR is effectively a stream cipher built from a block cipher:
//! ```text
//! Keystream = AES(key, nonce||1) || AES(key, nonce||2) || AES(key, nonce||3) || ...
//! Ciphertext = Plaintext XOR Keystream
//! ```
//!
//! ## Chunked Encryption for Large Data
//!
//! For large files, you can't load everything into memory. Instead:
//! 1. Split data into chunks
//! 2. Encrypt each chunk with a unique nonce (counter-based)
//! 3. Store chunk index + nonce alongside each chunk
//!
//! ## Attack Scenario: ECB Mode
//!
//! ECB encrypts identical blocks to identical ciphertext, leaking patterns:
//! - The famous "ECB penguin" shows patterns in an encrypted image
//! - An attacker can detect repeated blocks without decrypting
//!
//! ## Defense
//!
//! - Use GCM or ChaCha20-Poly1305 (stream-like, no block patterns)
//! - If you must use CBC, always include a random IV
//! - For chunked encryption, use a monotonically increasing counter as nonce

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};
use chacha20poly1305::{
    aead::{Aead as ChaChaAead, AeadCore as ChaChaAeadCore, KeyInit as ChaChaKeyInit, OsRng as ChaChaOsRng, Key as ChaChaKey, Nonce as ChaChaNonce},
    ChaCha20Poly1305,
};

/// Exercise 1: Demonstrate ECB mode weakness (pattern leakage).
///
/// ECB mode encrypts each block independently. If two plaintext blocks are identical,
/// their ciphertext blocks are also identical. This leaks patterns.
///
/// This function encrypts the same 16-byte block twice with AES-GCM using different
/// nonces, producing different ciphertexts. Returns true if the ciphertexts differ,
/// proving that GCM (unlike ECB) hides patterns.
///
/// Hints:
/// - Encrypt the same 16 bytes twice with different random nonces
/// - Compare the ciphertexts — they should be different
/// - GCM uses CTR mode internally, so identical plaintext with different nonces
///   produces different ciphertext
pub fn demonstrate_ecb_pattern_leakage(key: &Key<Aes256Gcm>) -> bool {
    todo!("Show that GCM (unlike ECB) hides repeated blocks")
}

/// Exercise 2: Implement chunked encryption for large data.
///
/// Encrypt data in fixed-size chunks, each with a unique nonce derived
/// from a chunk counter. This allows streaming encryption without loading
/// all data into memory.
///
/// Chunk format: [chunk_index: 8 bytes] [nonce: 12 bytes] [ciphertext+tag]
///
/// Hints:
/// - Split plaintext into chunks of `chunk_size` bytes
/// - For each chunk, generate a nonce from the chunk index
/// - Concatenate all encrypted chunks
pub fn encrypt_chunked(
    key: &Key<Aes256Gcm>,
    plaintext: &[u8],
    chunk_size: usize,
) -> Vec<u8> {
    todo!("Encrypt data in chunks with counter-based nonces")
}

/// Exercise 3: Decrypt chunked data.
///
/// Parse and decrypt data produced by `encrypt_chunked`.
///
/// Hints:
/// - Parse each chunk: [index: 8] [nonce: 12] [ciphertext+tag]
/// - Decrypt each chunk and concatenate the results
/// - The ciphertext+tag length is: total_chunk_size - 8 - 12 = total - 20
pub fn decrypt_chunked(
    key: &Key<Aes256Gcm>,
    data: &[u8],
    chunk_size: usize,
) -> Result<Vec<u8>, &'static str> {
    todo!("Decrypt chunked data")
}

/// Exercise 4: Compare AES-GCM and ChaCha20-Poly1305 performance characteristics.
///
/// This function encrypts the same data with both ciphers and returns
/// (gcm_ciphertext_len, chacha_ciphertext_len). Both should be the same
/// since both add a 16-byte authentication tag.
///
/// Hints:
/// - Encrypt with AES-256-GCM
/// - Encrypt with ChaCha20-Poly1305
/// - Return both ciphertext lengths
pub fn compare_ciphertext_sizes(
    aes_key: &Key<Aes256Gcm>,
    chacha_key: &ChaChaKey<ChaCha20Poly1305>,
    plaintext: &[u8],
) -> (usize, usize) {
    todo!("Encrypt with both ciphers and compare output sizes")
}

/// Exercise 5: Demonstrate the "nonce as counter" pattern for streaming.
///
/// Show that using a sequential counter for nonces guarantees uniqueness.
/// Generate 100 nonces using a counter and verify they are all unique.
///
/// Hints:
/// - Use a loop from 0..100
/// - For each iteration, create a nonce from the counter value
/// - Collect all nonces and verify no duplicates
pub fn demonstrate_streaming_nonces() -> Vec<[u8; 12]> {
    todo!("Generate 100 sequential nonces and verify uniqueness")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_aes_key() -> Key<Aes256Gcm> {
        Aes256Gcm::generate_key(&mut OsRng)
    }

    fn test_chacha_key() -> ChaChaKey<ChaCha20Poly1305> {
        ChaCha20Poly1305::generate_key(&mut ChaChaOsRng)
    }

    #[test]
    fn test_ecb_pattern_leakage_gcm_hides_patterns() {
        let key = test_aes_key();
        let hides_patterns = demonstrate_ecb_pattern_leakage(&key);
        assert!(hides_patterns, "GCM should produce different ciphertext for identical plaintext blocks");
    }

    #[test]
    fn test_chunked_encrypt_decrypt_small() {
        let key = test_aes_key();
        let plaintext = b"Hello, chunked encryption!";
        let chunk_size = 10;

        let encrypted = encrypt_chunked(&key, plaintext, chunk_size);
        let decrypted = decrypt_chunked(&key, &encrypted, chunk_size).expect("Chunked decrypt failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chunked_encrypt_decrypt_exact_chunks() {
        let key = test_aes_key();
        let plaintext = vec![0xAAu8; 100]; // Exactly 10 chunks of 10
        let chunk_size = 10;

        let encrypted = encrypt_chunked(&key, &plaintext, chunk_size);
        let decrypted = decrypt_chunked(&key, &encrypted, chunk_size).expect("Failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chunked_encrypt_decrypt_partial_last_chunk() {
        let key = test_aes_key();
        let plaintext = vec![0xBBu8; 55]; // 5 full chunks + 1 partial
        let chunk_size = 10;

        let encrypted = encrypt_chunked(&key, &plaintext, chunk_size);
        let decrypted = decrypt_chunked(&key, &encrypted, chunk_size).expect("Failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chunked_empty_plaintext() {
        let key = test_aes_key();
        let encrypted = encrypt_chunked(&key, b"", 10);
        let decrypted = decrypt_chunked(&key, &encrypted, 10).expect("Failed");
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_compare_ciphertext_sizes() {
        let aes_key = test_aes_key();
        let chacha_key = test_chacha_key();
        let plaintext = b"size comparison test";

        let (gcm_len, chacha_len) = compare_ciphertext_sizes(&aes_key, &chacha_key, plaintext);
        assert_eq!(gcm_len, chacha_len, "Both AEAD ciphers add 16-byte tag");
        assert_eq!(gcm_len, plaintext.len() + 16);
    }

    #[test]
    fn test_streaming_nonces_unique() {
        let nonces = demonstrate_streaming_nonces();
        assert_eq!(nonces.len(), 100);
        // Check all nonces are unique
        let mut sorted = nonces.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 100, "All 100 nonces must be unique");
    }

    #[test]
    fn test_chunked_tampered_data_fails() {
        let key = test_aes_key();
        let plaintext = b"tamper detection in chunks";
        let mut encrypted = encrypt_chunked(&key, plaintext, 10);

        // Tamper with a byte in the first chunk's ciphertext
        if encrypted.len() > 25 {
            encrypted[25] ^= 0xff;
        }

        let result = decrypt_chunked(&key, &encrypted, 10);
        assert!(result.is_err(), "Tampered chunked data must fail");
    }
}
