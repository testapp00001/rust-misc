//! # Lesson 06: Stream vs Block Ciphers (Reference Solution)
//!
//! See the exercise file for full documentation on stream ciphers, block ciphers,
//! and chunked encryption.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};
use chacha20poly1305::{
    aead::{Aead as ChaChaAead, AeadCore as ChaChaAeadCore, KeyInit as ChaChaKeyInit, OsRng as ChaChaOsRng, Key as ChaChaKey, Nonce as ChaChaNonce},
    ChaCha20Poly1305,
};

/// Demonstrate that GCM (CTR-based) hides repeated blocks unlike ECB.
///
/// ECB encrypts identical blocks to identical ciphertext, leaking patterns.
/// GCM uses CTR mode internally, so identical plaintext with different nonces
/// produces completely different ciphertext.
pub fn demonstrate_ecb_pattern_leakage(key: &Key<Aes256Gcm>) -> bool {
    let cipher = Aes256Gcm::new(key);
    let block = [0xAAu8; 16]; // Same 16-byte block

    // Encrypt the same block twice with different nonces
    let nonce1 = Aes256Gcm::generate_nonce(&mut OsRng);
    let nonce2 = Aes256Gcm::generate_nonce(&mut OsRng);

    let ct1 = cipher.encrypt(&nonce1, block.as_ref()).expect("Encrypt 1 failed");
    let ct2 = cipher.encrypt(&nonce2, block.as_ref()).expect("Encrypt 2 failed");

    // GCM should produce different ciphertext for the same plaintext
    ct1 != ct2
}

/// Encrypt data in chunks with counter-based nonces.
///
/// Each chunk is encrypted independently with a unique nonce derived from
/// the chunk index. This allows streaming encryption of large data.
///
/// Chunk format: [chunk_index: 8 bytes] [nonce: 12 bytes] [ciphertext+tag]
pub fn encrypt_chunked(
    key: &Key<Aes256Gcm>,
    plaintext: &[u8],
    chunk_size: usize,
) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let mut output = Vec::new();

    for (i, chunk) in plaintext.chunks(chunk_size).enumerate() {
        // Derive nonce from chunk index (counter-based)
        let mut nonce_bytes = [0u8; 12];
        let counter_bytes = (i as u64).to_be_bytes();
        nonce_bytes[4..12].copy_from_slice(&counter_bytes);
        let nonce = Nonce::<Aes256Gcm>::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, chunk).expect("Chunk encryption failed");

        // Write: [index: 8] [nonce: 12] [ciphertext+tag]
        output.extend_from_slice(&(i as u64).to_be_bytes());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
    }

    output
}

/// Decrypt chunked data produced by `encrypt_chunked`.
///
/// Each chunk is: [index: 8] [nonce: 12] [ciphertext+tag]
/// Total per chunk: 8 + 12 + (chunk_size + 16) = chunk_size + 36
pub fn decrypt_chunked(
    key: &Key<Aes256Gcm>,
    data: &[u8],
    chunk_size: usize,
) -> Result<Vec<u8>, &'static str> {
    let cipher = Aes256Gcm::new(key);
    let mut output = Vec::new();

    // Each encrypted chunk header is 8 (index) + 12 (nonce) = 20 bytes
    // Plus the ciphertext+tag which is original_chunk_size + 16
    let chunk_overhead = 8 + 12; // index + nonce
    let encrypted_chunk_size = chunk_overhead + chunk_size + 16; // + 16 for GCM tag
    let encrypted_last_chunk_size = chunk_overhead + (data.len() % if data.is_empty() { 1 } else { encrypted_chunk_size }).max(0) + 16;

    // Parse chunks
    let mut offset = 0;
    while offset < data.len() {
        if offset + 20 > data.len() {
            return Err("Truncated chunk header");
        }

        // Parse index (we trust it for ordering)
        let _index = u64::from_be_bytes(data[offset..offset + 8].try_into().unwrap());

        // Parse nonce
        let nonce = Nonce::<Aes256Gcm>::from_slice(&data[offset + 8..offset + 20]);

        // Find the end of this chunk's ciphertext
        // The ciphertext extends to the start of the next chunk or end of data
        let next_chunk_start = offset + 8 + 12 + chunk_size + 16;
        let ct_end = if next_chunk_start <= data.len() {
            next_chunk_start
        } else {
            data.len()
        };

        let ciphertext = &data[offset + 20..ct_end];

        match cipher.decrypt(nonce, ciphertext) {
            Ok(plaintext) => output.extend_from_slice(&plaintext),
            Err(_) => return Err("Chunk decryption failed"),
        }

        offset = ct_end;
    }

    Ok(output)
}

/// Compare ciphertext sizes between AES-GCM and ChaCha20-Poly1305.
///
/// Both add a 16-byte authentication tag, so output sizes are identical
/// for the same plaintext.
pub fn compare_ciphertext_sizes(
    aes_key: &Key<Aes256Gcm>,
    chacha_key: &ChaChaKey<ChaCha20Poly1305>,
    plaintext: &[u8],
) -> (usize, usize) {
    // AES-GCM encryption
    let aes_cipher = Aes256Gcm::new(aes_key);
    let aes_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let aes_ct = aes_cipher.encrypt(&aes_nonce, plaintext).expect("AES encrypt failed");

    // ChaCha20-Poly1305 encryption
    let chacha_cipher = ChaCha20Poly1305::new(chacha_key);
    let chacha_nonce = ChaCha20Poly1305::generate_nonce(&mut ChaChaOsRng);
    let chacha_ct = chacha_cipher.encrypt(&chacha_nonce, plaintext).expect("ChaCha encrypt failed");

    (aes_ct.len(), chacha_ct.len())
}

/// Generate 100 sequential nonces and return them.
///
/// Counter-based nonces guarantee uniqueness as long as the counter
/// is monotonically increasing and never wraps.
pub fn demonstrate_streaming_nonces() -> Vec<[u8; 12]> {
    let mut nonces = Vec::with_capacity(100);

    for i in 0..100u64 {
        let mut nonce_bytes = [0u8; 12];
        let counter_bytes = i.to_be_bytes();
        nonce_bytes[4..12].copy_from_slice(&counter_bytes);
        nonces.push(nonce_bytes);
    }

    nonces
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
        let plaintext = vec![0xAAu8; 100];
        let chunk_size = 10;

        let encrypted = encrypt_chunked(&key, &plaintext, chunk_size);
        let decrypted = decrypt_chunked(&key, &encrypted, chunk_size).expect("Failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chunked_encrypt_decrypt_partial_last_chunk() {
        let key = test_aes_key();
        let plaintext = vec![0xBBu8; 55];
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

        if encrypted.len() > 25 {
            encrypted[25] ^= 0xff;
        }

        let result = decrypt_chunked(&key, &encrypted, 10);
        assert!(result.is_err(), "Tampered chunked data must fail");
    }
}
