//! # Lesson 02: RSA Key Sizes — Security vs Performance (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rsa::{RsaPrivateKey, Oaep};
use rand::rngs::OsRng;
use std::time::Instant;

/// Benchmark RSA key generation at different sizes.
///
/// Key generation involves finding two large primes, which requires many
/// primality tests. Larger keys need larger primes -> more tests -> slower.
///
/// Expected approximate times (modern hardware):
/// - 1024-bit: ~10ms
/// - 2048-bit: ~100ms
/// - 3072-bit: ~500ms
/// - 4096-bit: ~2000ms
pub fn benchmark_key_generation() -> Vec<(usize, u128)> {
    let sizes = [1024, 2048, 3072, 4096];
    sizes
        .iter()
        .map(|&bits| {
            let start = Instant::now();
            let _key = RsaPrivateKey::new(&mut OsRng, bits).expect("keygen failed");
            let elapsed = start.elapsed().as_millis();
            (bits, elapsed)
        })
        .collect()
}

/// Benchmark RSA encrypt/decrypt at different key sizes.
///
/// Encryption uses the public exponent (commonly 65537 = 0x10001),
/// so it's relatively fast. Decryption uses the private exponent,
/// which is much larger, making it slower.
pub fn benchmark_encrypt_decrypt() -> Vec<(usize, u128, u128)> {
    let sizes = [1024, 2048, 3072, 4096];
    let message = vec![0u8; 32]; // Fixed 32-byte message

    sizes
        .iter()
        .map(|&bits| {
            let priv_key = RsaPrivateKey::new(&mut OsRng, bits).expect("keygen failed");
            let pub_key = priv_key.to_public_key();

            // Benchmark encryption
            let start = Instant::now();
            let ciphertext = pub_key
                .encrypt(&mut OsRng, Oaep::new::<sha2::Sha256>(), &message)
                .expect("encrypt failed");
            let encrypt_us = start.elapsed().as_micros();

            // Benchmark decryption
            let start = Instant::now();
            let _plaintext = priv_key.decrypt(Oaep::new::<sha2::Sha256>(), &ciphertext).expect("decrypt failed");
            let decrypt_us = start.elapsed().as_micros();

            (bits, encrypt_us, decrypt_us)
        })
        .collect()
}

/// Calculate maximum plaintext size for RSA-OAEP with SHA-256.
///
/// RSA-OAEP overhead: 2 * hash_len + 2 bytes
/// For SHA-256: 2 * 32 + 2 = 66 bytes overhead
/// Max plaintext = key_bytes - 66
pub fn max_plaintext_size(key_bits: usize) -> usize {
    let key_bytes = key_bits / 8;
    let hash_len = 32; // SHA-256
    key_bytes - 2 * hash_len - 2
}

/// Check if RSA key size meets minimum security requirements.
///
/// NIST SP 800-57 recommends:
/// - 2048-bit minimum through 2030
/// - 3072-bit for protection beyond 2030
/// - 1024-bit is considered insecure (factorizable by nation-states)
pub fn is_secure_key_size(key_bits: usize) -> bool {
    key_bits >= 2048
}

/// Map RSA key size to equivalent ECC key size.
///
/// Based on NIST SP 800-57 security level equivalences:
/// - RSA 2048 ~ ECC 224 (112-bit security)
/// - RSA 3072 ~ ECC 256 (128-bit security)
/// - RSA 7680 ~ ECC 384 (192-bit security)
/// - RSA 15360 ~ ECC 521 (256-bit security)
pub fn equivalent_ecc_key_size(rsa_bits: usize) -> usize {
    match rsa_bits {
        2048 => 224,
        3072 => 256,
        7680 => 384,
        15360 => 521,
        _ => 0, // Below minimum or non-standard size
    }
}

/// Attempt to encrypt a message exceeding RSA-OAEP capacity.
///
/// Returns Ok(ciphertext) if the message fits, Err if it doesn't.
/// This demonstrates the hard limit on RSA plaintext size.
pub fn encrypt_oversized_message(key_bits: usize, message: &[u8]) -> Result<Vec<u8>, String> {
    let max = max_plaintext_size(key_bits);
    if message.len() > max {
        return Err(format!(
            "Message too large: {} bytes exceeds maximum {} bytes for RSA-{}",
            message.len(), max, key_bits
        ));
    }

    let priv_key = RsaPrivateKey::new(&mut OsRng, key_bits).map_err(|e| e.to_string())?;
    let pub_key = priv_key.to_public_key();

    pub_key
        .encrypt(&mut OsRng, Oaep::new::<sha2::Sha256>(), message)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_plaintext_2048() {
        // RSA-2048 with OAEP SHA-256: 256 - 66 = 190 bytes
        assert_eq!(max_plaintext_size(2048), 190);
    }

    #[test]
    fn test_max_plaintext_4096() {
        // RSA-4096 with OAEP SHA-256: 512 - 66 = 446 bytes
        assert_eq!(max_plaintext_size(4096), 446);
    }

    #[test]
    fn test_is_secure_key_size() {
        assert!(!is_secure_key_size(1024), "1024-bit is insecure");
        assert!(is_secure_key_size(2048), "2048-bit is minimum secure");
        assert!(is_secure_key_size(3072), "3072-bit is secure");
        assert!(is_secure_key_size(4096), "4096-bit is secure");
    }

    #[test]
    fn test_equivalent_ecc_sizes() {
        assert_eq!(equivalent_ecc_key_size(2048), 224);
        assert_eq!(equivalent_ecc_key_size(3072), 256);
        assert_eq!(equivalent_ecc_key_size(7680), 384);
        assert_eq!(equivalent_ecc_key_size(15360), 521);
    }

    #[test]
    fn test_insecure_key_returns_zero_ecc() {
        assert_eq!(equivalent_ecc_key_size(512), 0, "512-bit RSA is insecure");
        assert_eq!(equivalent_ecc_key_size(1024), 0, "1024-bit RSA is insecure");
    }

    #[test]
    fn test_oversized_message_fails() {
        let result = encrypt_oversized_message(2048, &vec![0u8; 200]);
        assert!(result.is_err(), "Encrypting oversized message should fail");
    }

    #[test]
    fn test_max_size_just_fits() {
        let bits = 2048;
        let max = max_plaintext_size(bits);
        let result = encrypt_oversized_message(bits, &vec![0u8; max]);
        assert!(result.is_ok(), "Message at max size should encrypt successfully");
    }

    #[test]
    #[ignore] // Slow test — run with: cargo test --features solution -- --ignored
    fn test_key_generation_timing() {
        let benchmarks = benchmark_key_generation();
        assert_eq!(benchmarks.len(), 4);
        // Larger keys should take longer
        assert!(benchmarks[1].1 >= benchmarks[0].1,
            "2048-bit should take at least as long as 1024-bit");
    }
}
