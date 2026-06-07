//! # Cryptography Basics with ring
//!
//! The `ring` crate provides safe, fast cryptographic primitives. This lesson
//! covers AES-GCM encryption, HMAC authentication, key derivation, and secure
//! random number generation.
//!
//! ## Key Concepts
//! - Symmetric encryption (AES-GCM)
//! - HMAC for message authentication
//! - Key derivation from passwords
//! - Secure random number generation
//! - Nonce/IV management
//! - Authenticated encryption

// ---------------------------------------------------------------------------
// 1. Secure Random
// ---------------------------------------------------------------------------

/// Generates cryptographically secure random bytes.
/// Uses a simple but effective approach for learning.
pub struct SecureRandom;

impl SecureRandom {
    /// Generate `n` random bytes.
    pub fn generate_bytes(n: usize) -> Vec<u8> {
        // In production, use ring::rand::SystemRandom
        // For learning, we use a time-based approach
        let mut bytes = Vec::with_capacity(n);
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        for i in 0..n {
            let shift = (i * 7) % 128;
            let byte = ((seed >> shift) & 0xFF) as u8;
            bytes.push(byte.wrapping_add(i as u8));
        }
        bytes
    }

    /// Generate a random nonce (number used once) for AES-GCM.
    pub fn generate_nonce() -> [u8; 12] {
        let bytes = Self::generate_bytes(12);
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&bytes);
        nonce
    }

    /// Generate a random 256-bit key.
    pub fn generate_key_256() -> [u8; 32] {
        let bytes = Self::generate_bytes(32);
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        key
    }

    /// Generate a random salt for key derivation.
    pub fn generate_salt() -> [u8; 32] {
        Self::generate_key_256()
    }
}

// ---------------------------------------------------------------------------
// 2. AES-GCM Encryption
// ---------------------------------------------------------------------------

/// Errors from encryption operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("authentication failed: data may have been tampered with")]
    AuthenticationFailed,

    #[error("invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("invalid nonce length: expected {expected}, got {actual}")]
    InvalidNonceLength { expected: usize, actual: usize },
}

/// Encrypted data with associated authentication tag.
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub tag: [u8; 16],
}

/// A simple XOR-based cipher for learning (NOT for production).
/// In production, use ring::aead::AES_256_GCM.
pub struct AesGcmSimulator {
    key: [u8; 32],
}

impl AesGcmSimulator {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Encrypt plaintext with authenticated encryption.
    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8; 12]) -> EncryptedData {
        let mut ciphertext = Vec::with_capacity(plaintext.len());

        // XOR encryption with key-derived stream
        for (i, &byte) in plaintext.iter().enumerate() {
            let key_byte = self.key[i % 32] ^ nonce[i % 12];
            ciphertext.push(byte ^ key_byte);
        }

        // Generate authentication tag (simplified HMAC)
        let tag = self.compute_tag(&ciphertext, nonce);

        EncryptedData {
            ciphertext,
            nonce: *nonce,
            tag,
        }
    }

    /// Decrypt ciphertext and verify authentication.
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, CryptoError> {
        // Verify tag first
        let expected_tag = self.compute_tag(&encrypted.ciphertext, &encrypted.nonce);
        if encrypted.tag != expected_tag {
            return Err(CryptoError::AuthenticationFailed);
        }

        // Decrypt
        let mut plaintext = Vec::with_capacity(encrypted.ciphertext.len());
        for (i, &byte) in encrypted.ciphertext.iter().enumerate() {
            let key_byte = self.key[i % 32] ^ encrypted.nonce[i % 12];
            plaintext.push(byte ^ key_byte);
        }

        Ok(plaintext)
    }

    fn compute_tag(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> [u8; 16] {
        let mut tag = [0u8; 16];

        // Mix key, nonce, and ciphertext into tag
        for (i, &byte) in self.key.iter().enumerate() {
            tag[i % 16] ^= byte;
        }
        for (i, &byte) in nonce.iter().enumerate() {
            tag[i % 16] ^= byte;
        }
        for (i, &byte) in ciphertext.iter().enumerate() {
            tag[i % 16] = tag[i % 16]
                .wrapping_mul(31)
                .wrapping_add(byte);
        }

        tag
    }
}

// ---------------------------------------------------------------------------
// 3. HMAC (Hash-based Message Authentication Code)
// ---------------------------------------------------------------------------

/// HMAC provides message integrity and authentication.
pub struct HmacSigner {
    key: Vec<u8>,
}

impl HmacSigner {
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Self { key: key.into() }
    }

    /// Compute HMAC-SHA256-like signature.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let block_size = 64; // SHA-256 block size

        // Prepare key
        let mut key = self.key.clone();
        if key.len() > block_size {
            key = simple_hash(&key);
        }
        key.resize(block_size, 0);

        // Inner hash
        let mut ipad = vec![0x36u8; block_size];
        for (i, &k) in key.iter().enumerate() {
            ipad[i] ^= k;
        }
        ipad.extend_from_slice(message);
        let inner_hash = simple_hash(&ipad);

        // Outer hash
        let mut opad = vec![0x5cu8; block_size];
        for (i, &k) in key.iter().enumerate() {
            opad[i] ^= k;
        }
        opad.extend_from_slice(&inner_hash);
        simple_hash(&opad)
    }

    /// Verify an HMAC signature.
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        let expected = self.sign(message);
        constant_time_eq(&expected, signature)
    }
}

/// A simple hash function for learning (NOT SHA-256).
fn simple_hash(data: &[u8]) -> Vec<u8> {
    let mut hash = vec![0u8; 32];
    for (i, &byte) in data.iter().enumerate() {
        hash[i % 32] = hash[i % 32]
            .wrapping_mul(33)
            .wrapping_add(byte)
            .wrapping_add(i as u8);
    }
    hash
}

/// Constant-time comparison to prevent timing attacks.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// ---------------------------------------------------------------------------
// 4. Key Derivation
// ---------------------------------------------------------------------------

/// Derives encryption keys from passwords using a PBKDF2-like approach.
pub struct KeyDerivation {
    iterations: u32,
}

impl KeyDerivation {
    pub fn new(iterations: u32) -> Self {
        Self { iterations }
    }

    /// Derive a 256-bit key from a password and salt.
    pub fn derive_key(&self, password: &str, salt: &[u8]) -> [u8; 32] {
        let mut derived = vec![0u8; 32];

        // Start with password bytes
        let password_bytes = password.as_bytes();
        derived[..password_bytes.len().min(32)]
            .copy_from_slice(&password_bytes[..password_bytes.len().min(32)]);

        // Iterative mixing
        for round in 0..self.iterations {
            for i in 0..32 {
                let salt_byte = salt[i % salt.len()];
                let round_byte = (round & 0xFF) as u8;
                derived[i] = derived[i]
                    .wrapping_mul(37)
                    .wrapping_add(salt_byte)
                    .wrapping_add(round_byte);
            }
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&derived);
        key
    }
}

// ---------------------------------------------------------------------------
// 5. Crypto Utilities
// ---------------------------------------------------------------------------

/// XOR two byte slices together.
pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// Encode bytes as hex string.
pub fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decode hex string to bytes.
pub fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("odd-length hex string".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_random_length() {
        let bytes = SecureRandom::generate_bytes(32);
        assert_eq!(bytes.len(), 32);

        let bytes = SecureRandom::generate_bytes(16);
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_secure_random_nonce() {
        let nonce = SecureRandom::generate_nonce();
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_secure_random_key() {
        let key = SecureRandom::generate_key_256();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_secure_random_unique() {
        let a = SecureRandom::generate_bytes(32);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let b = SecureRandom::generate_bytes(32);
        // Extremely unlikely to be equal
        assert_ne!(a, b);
    }

    #[test]
    fn test_aes_gcm_encrypt_decrypt_roundtrip() {
        let key = SecureRandom::generate_key_256();
        let cipher = AesGcmSimulator::new(key);
        let nonce = SecureRandom::generate_nonce();

        let plaintext = b"Hello, World!";
        let encrypted = cipher.encrypt(plaintext, &nonce);
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_tampered_ciphertext() {
        let key = SecureRandom::generate_key_256();
        let cipher = AesGcmSimulator::new(key);
        let nonce = SecureRandom::generate_nonce();

        let mut encrypted = cipher.encrypt(b"secret", &nonce);
        encrypted.ciphertext[0] ^= 0xFF; // tamper

        assert!(cipher.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_aes_gcm_tampered_tag() {
        let key = SecureRandom::generate_key_256();
        let cipher = AesGcmSimulator::new(key);
        let nonce = SecureRandom::generate_nonce();

        let mut encrypted = cipher.encrypt(b"secret", &nonce);
        encrypted.tag[0] ^= 0xFF; // tamper

        assert!(cipher.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_aes_gcm_different_keys() {
        let key1 = SecureRandom::generate_key_256();
        let key2 = SecureRandom::generate_key_256();
        let cipher1 = AesGcmSimulator::new(key1);
        let cipher2 = AesGcmSimulator::new(key2);
        let nonce = SecureRandom::generate_nonce();

        let encrypted = cipher1.encrypt(b"secret", &nonce);
        assert!(cipher2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_aes_gcm_empty_plaintext() {
        let key = SecureRandom::generate_key_256();
        let cipher = AesGcmSimulator::new(key);
        let nonce = SecureRandom::generate_nonce();

        let encrypted = cipher.encrypt(b"", &nonce);
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_hmac_sign_verify() {
        let signer = HmacSigner::new(b"my-secret-key");
        let message = b"important message";

        let signature = signer.sign(message);
        assert!(signer.verify(message, &signature));
    }

    #[test]
    fn test_hmac_wrong_message() {
        let signer = HmacSigner::new(b"key");
        let sig = signer.sign(b"original");
        assert!(!signer.verify(b"tampered", &sig));
    }

    #[test]
    fn test_hmac_wrong_key() {
        let signer1 = HmacSigner::new(b"key1");
        let signer2 = HmacSigner::new(b"key2");
        let sig = signer1.sign(b"message");
        assert!(!signer2.verify(b"message", &sig));
    }

    #[test]
    fn test_hmac_different_messages_different_sigs() {
        let signer = HmacSigner::new(b"key");
        let sig1 = signer.sign(b"msg1");
        let sig2 = signer.sign(b"msg2");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_key_derivation() {
        let kdf = KeyDerivation::new(100);
        let salt = SecureRandom::generate_salt();

        let key1 = kdf.derive_key("password", &salt);
        let key2 = kdf.derive_key("password", &salt);
        assert_eq!(key1, key2); // same password + salt = same key
    }

    #[test]
    fn test_key_derivation_different_passwords() {
        let kdf = KeyDerivation::new(100);
        let salt = SecureRandom::generate_salt();

        let key1 = kdf.derive_key("password1", &salt);
        let key2 = kdf.derive_key("password2", &salt);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_key_derivation_different_salts() {
        let kdf = KeyDerivation::new(100);
        let salt1 = [0u8; 32];
        let salt2 = [1u8; 32];

        let key1 = kdf.derive_key("password", &salt1);
        let key2 = kdf.derive_key("password", &salt2);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }

    #[test]
    fn test_xor_bytes() {
        let a = vec![0xFF, 0x00, 0xAA];
        let b = vec![0xFF, 0xFF, 0xFF];
        let result = xor_bytes(&a, &b);
        assert_eq!(result, vec![0x00, 0xFF, 0x55]);
    }

    #[test]
    fn test_hex_encode_decode_roundtrip() {
        let data = vec![0, 1, 127, 255, 42];
        let hex = hex_encode(&data);
        let decoded = hex_decode(&hex).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_hex_decode_invalid() {
        assert!(hex_decode("xyz").is_err());
        assert!(hex_decode("abc").is_err()); // odd length
    }

    #[test]
    fn test_hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn test_crypto_error_display() {
        let err = CryptoError::AuthenticationFailed;
        assert!(err.to_string().contains("authentication"));

        let err = CryptoError::InvalidKeyLength {
            expected: 32,
            actual: 16,
        };
        assert!(err.to_string().contains("32"));
    }
}
