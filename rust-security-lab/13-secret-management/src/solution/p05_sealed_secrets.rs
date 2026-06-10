//! # Lesson 05: Sealed Secrets (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedSecret {
    pub name: String,
    pub ciphertext: String,
    pub nonce: String,
    pub algorithm: String,
}

#[derive(Debug, Clone)]
pub struct SecretInfo {
    pub name: String,
    pub algorithm: String,
    pub ciphertext_len: usize,
}

pub fn seal_secret(name: &str, plaintext: &[u8], key: &[u8; 32]) -> Result<SealedSecret, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(SealedSecret {
        name: name.to_string(),
        ciphertext: BASE64.encode(&ciphertext),
        nonce: BASE64.encode(&nonce_bytes),
        algorithm: "AES-256-GCM".to_string(),
    })
}

pub fn unseal_secret(sealed: &SealedSecret, key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let ciphertext = BASE64
        .decode(&sealed.ciphertext)
        .map_err(|e| format!("Failed to decode ciphertext: {}", e))?;
    let nonce_bytes = BASE64
        .decode(&sealed.nonce)
        .map_err(|e| format!("Failed to decode nonce: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn seal_many(secrets: &[(&str, &[u8])], key: &[u8; 32]) -> Result<Vec<SealedSecret>, String> {
    secrets
        .iter()
        .map(|(name, value)| seal_secret(name, value, key))
        .collect()
}

pub fn get_secret_info(sealed: &SealedSecret) -> Result<SecretInfo, String> {
    let ciphertext_len = BASE64
        .decode(&sealed.ciphertext)
        .map_err(|e| format!("Failed to decode: {}", e))?
        .len();

    Ok(SecretInfo {
        name: sealed.name.clone(),
        algorithm: sealed.algorithm.clone(),
        ciphertext_len,
    })
}

pub fn sealed_to_json(sealed: &SealedSecret) -> Result<String, String> {
    serde_json::to_string_pretty(sealed).map_err(|e| format!("Serialization failed: {}", e))
}

pub fn sealed_from_json(json: &str) -> Result<SealedSecret, String> {
    serde_json::from_str(json).map_err(|e| format!("Deserialization failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ]
    }

    #[test]
    fn test_seal_and_unseal_roundtrip() {
        let key = test_key();
        let plaintext = b"super_secret_password";
        let sealed = seal_secret("db_password", plaintext, &key).unwrap();
        let decrypted = unseal_secret(&sealed, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_seal_different_nonces() {
        let key = test_key();
        let s1 = seal_secret("key", b"secret1", &key).unwrap();
        let s2 = seal_secret("key", b"secret1", &key).unwrap();
        assert_ne!(s1.nonce, s2.nonce);
        assert_ne!(s1.ciphertext, s2.ciphertext);
    }

    #[test]
    fn test_unseal_wrong_key() {
        let key1 = test_key();
        let mut key2 = test_key();
        key2[0] ^= 0xff;

        let sealed = seal_secret("key", b"secret", &key1).unwrap();
        let result = unseal_secret(&sealed, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_key_random() {
        let k1 = generate_key();
        let k2 = generate_key();
        assert_ne!(k1, k2, "Generated keys should be random");
        assert_eq!(k1.len(), 32);
    }

    #[test]
    fn test_seal_many() {
        let key = test_key();
        let secrets = vec![
            ("db_pass", b"password123" as &[u8]),
            ("api_key", b"sk-abcdef123456" as &[u8]),
        ];
        let sealed = seal_many(&secrets, &key).unwrap();
        assert_eq!(sealed.len(), 2);
        assert_eq!(sealed[0].name, "db_pass");
        assert_eq!(sealed[1].name, "api_key");
    }

    #[test]
    fn test_sealed_to_json_and_back() {
        let key = test_key();
        let sealed = seal_secret("test", b"value", &key).unwrap();
        let json = sealed_to_json(&sealed).unwrap();
        let parsed = sealed_from_json(&json).unwrap();

        assert_eq!(parsed.name, sealed.name);
        assert_eq!(parsed.ciphertext, sealed.ciphertext);
        assert_eq!(parsed.nonce, sealed.nonce);

        let decrypted = unseal_secret(&parsed, &key).unwrap();
        assert_eq!(decrypted, b"value");
    }

    #[test]
    fn test_get_secret_info() {
        let key = test_key();
        let sealed = seal_secret("my_secret", b"hello world", &key).unwrap();
        let info = get_secret_info(&sealed).unwrap();
        assert_eq!(info.name, "my_secret");
        assert_eq!(info.algorithm, "AES-256-GCM");
        assert!(info.ciphertext_len > 0);
    }
}
