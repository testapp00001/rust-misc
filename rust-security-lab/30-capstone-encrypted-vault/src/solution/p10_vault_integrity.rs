//! # Lesson 10: Vault Integrity and Version Control (Reference Solution)
//!
//! See the exercise file for full documentation on the complete vault with integrity.

use argon2::{Argon2, Algorithm, Params, Version};
use hkdf::Hkdf;
use sha2::Sha256;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;

/// Magic bytes identifying the vault file format.
pub const VAULT_MAGIC: &[u8; 20] = b"ENCRYPTED_VAULT_V1\0\0";

/// A complete vault file with integrity protection.
#[derive(Debug, Clone)]
pub struct VaultFile {
    pub magic: [u8; 20],
    pub version: u64,
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub encrypted_data: Vec<u8>,
    pub hmac: Vec<u8>,
}

/// Audit log entry for vault modifications.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub version: u64,
    pub timestamp: String,
    pub action: String,
}

fn derive_enc_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let params = Params::new(4096, 3, 1, None)
        .map_err(|e| format!("Invalid params: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut master = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut master)
        .map_err(|e| format!("Argon2id failed: {}", e))?;
    let hk = Hkdf::<Sha256>::new(Some(salt), &master);
    let mut enc_key = [0u8; 32];
    hk.expand(b"vault-encryption", &mut enc_key)
        .map_err(|e| format!("HKDF failed: {}", e))?;
    Ok(enc_key)
}

fn derive_hmac_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let params = Params::new(4096, 3, 1, None)
        .map_err(|e| format!("Invalid params: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut master = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut master)
        .map_err(|e| format!("Argon2id failed: {}", e))?;
    let hk = Hkdf::<Sha256>::new(Some(salt), &master);
    let mut hmac_key = [0u8; 32];
    hk.expand(b"vault-hmac", &mut hmac_key)
        .map_err(|e| format!("HKDF failed: {}", e))?;
    Ok(hmac_key)
}

fn compute_hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let hmac_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    let tag = ring::hmac::sign(&hmac_key, data);
    Ok(tag.as_ref().to_vec())
}

fn verify_hmac_sha256(key: &[u8], data: &[u8], expected: &[u8]) -> Result<bool, String> {
    let hmac_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    match ring::hmac::verify(&hmac_key, data, expected) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn build_hmac_input(vault_file: &VaultFile) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&vault_file.magic);
    data.extend_from_slice(&vault_file.version.to_be_bytes());
    data.extend_from_slice(&(vault_file.salt.len() as u32).to_be_bytes());
    data.extend_from_slice(&vault_file.salt);
    data.extend_from_slice(&(vault_file.nonce.len() as u32).to_be_bytes());
    data.extend_from_slice(&vault_file.nonce);
    data.extend_from_slice(&(vault_file.encrypted_data.len() as u32).to_be_bytes());
    data.extend_from_slice(&vault_file.encrypted_data);
    data
}

/// Create a new vault file from plaintext data.
pub fn create_vault_file(
    plaintext: &[u8],
    passphrase: &str,
) -> Result<VaultFile, String> {
    let mut salt = vec![0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let enc_key = derive_enc_key(passphrase, &salt)?;
    let hmac_key = derive_hmac_key(passphrase, &salt)?;

    let cipher = Aes256Gcm::new((&enc_key).into());
    let mut nonce_bytes = vec![0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let encrypted_data = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut vault_file = VaultFile {
        magic: *VAULT_MAGIC,
        version: 1,
        salt,
        nonce: nonce_bytes,
        encrypted_data,
        hmac: Vec::new(),
    };

    let hmac_input = build_hmac_input(&vault_file);
    vault_file.hmac = compute_hmac_sha256(&hmac_key, &hmac_input)?;

    Ok(vault_file)
}

/// Open and verify a vault file.
pub fn open_vault_file(
    vault_file: &VaultFile,
    passphrase: &str,
) -> Result<(Vec<u8>, u64), String> {
    // Verify HMAC first
    let hmac_key = derive_hmac_key(passphrase, &vault_file.salt)?;
    let hmac_input = build_hmac_input(vault_file);
    verify_hmac_sha256(&hmac_key, &hmac_input, &vault_file.hmac)?;

    // Derive encryption key and decrypt
    let enc_key = derive_enc_key(passphrase, &vault_file.salt)?;
    let cipher = Aes256Gcm::new((&enc_key).into());
    let nonce = Nonce::from_slice(&vault_file.nonce);
    let plaintext = cipher
        .decrypt(nonce, vault_file.encrypted_data.as_ref())
        .map_err(|_| "Decryption failed -- wrong passphrase or corrupted data".to_string())?;

    Ok((plaintext, vault_file.version))
}

/// Save/update a vault file (incrementing version).
pub fn save_vault_file(
    vault_file: &mut VaultFile,
    plaintext: &[u8],
    passphrase: &str,
) -> Result<(), String> {
    // Verify current integrity first
    let hmac_key = derive_hmac_key(passphrase, &vault_file.salt)?;
    let hmac_input = build_hmac_input(vault_file);
    verify_hmac_sha256(&hmac_key, &hmac_input, &vault_file.hmac)?;

    // Increment version
    vault_file.version += 1;

    // Re-encrypt with new nonce
    let enc_key = derive_enc_key(passphrase, &vault_file.salt)?;
    let cipher = Aes256Gcm::new((&enc_key).into());
    rand::rngs::OsRng.fill_bytes(&mut vault_file.nonce);
    let nonce = Nonce::from_slice(&vault_file.nonce);
    vault_file.encrypted_data = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Recompute HMAC
    let hmac_input = build_hmac_input(vault_file);
    vault_file.hmac = compute_hmac_sha256(&hmac_key, &hmac_input)?;

    Ok(())
}

/// Serialize a VaultFile to bytes for storage.
pub fn serialize_vault_file(vault_file: &VaultFile) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&vault_file.magic);
    data.extend_from_slice(&vault_file.version.to_be_bytes());
    data.extend_from_slice(&(vault_file.salt.len() as u32).to_be_bytes());
    data.extend_from_slice(&vault_file.salt);
    data.extend_from_slice(&(vault_file.nonce.len() as u32).to_be_bytes());
    data.extend_from_slice(&vault_file.nonce);
    data.extend_from_slice(&(vault_file.encrypted_data.len() as u32).to_be_bytes());
    data.extend_from_slice(&vault_file.encrypted_data);
    data.extend_from_slice(&vault_file.hmac);
    data
}

/// Deserialize a VaultFile from bytes.
pub fn deserialize_vault_file(data: &[u8]) -> Result<VaultFile, String> {
    if data.len() < 20 {
        return Err("Data too short".to_string());
    }

    // Magic
    let magic: [u8; 20] = data[..20].try_into().unwrap();
    if &magic != VAULT_MAGIC {
        return Err("Invalid vault magic bytes".to_string());
    }

    let mut pos = 20;

    // Version (8 bytes, big-endian u64)
    if data.len() < pos + 8 {
        return Err("Data too short for version".to_string());
    }
    let version = u64::from_be_bytes(data[pos..pos + 8].try_into().unwrap());
    pos += 8;

    // Salt
    if data.len() < pos + 4 {
        return Err("Data too short for salt length".to_string());
    }
    let salt_len = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if data.len() < pos + salt_len {
        return Err("Data too short for salt".to_string());
    }
    let salt = data[pos..pos + salt_len].to_vec();
    pos += salt_len;

    // Nonce
    if data.len() < pos + 4 {
        return Err("Data too short for nonce length".to_string());
    }
    let nonce_len = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if data.len() < pos + nonce_len {
        return Err("Data too short for nonce".to_string());
    }
    let nonce = data[pos..pos + nonce_len].to_vec();
    pos += nonce_len;

    // Encrypted data
    if data.len() < pos + 4 {
        return Err("Data too short for data length".to_string());
    }
    let data_len = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if data.len() < pos + data_len {
        return Err("Data too short for encrypted data".to_string());
    }
    let encrypted_data = data[pos..pos + data_len].to_vec();
    pos += data_len;

    // HMAC (remaining bytes, should be 32)
    if data.len() < pos + 32 {
        return Err("Data too short for HMAC".to_string());
    }
    let hmac = data[pos..pos + 32].to_vec();

    Ok(VaultFile {
        magic,
        version,
        salt,
        nonce,
        encrypted_data,
        hmac,
    })
}

/// Verify only the integrity (HMAC) of a vault file without decrypting.
pub fn verify_vault_integrity(
    vault_file: &VaultFile,
    passphrase: &str,
) -> Result<bool, String> {
    let hmac_key = derive_hmac_key(passphrase, &vault_file.salt)?;
    let hmac_input = build_hmac_input(vault_file);
    verify_hmac_sha256(&hmac_key, &hmac_input, &vault_file.hmac)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_plaintext() -> Vec<u8> {
        b"{\"version\":1,\"entries\":[{\"name\":\"GitHub\",\"username\":\"user\",\"password\":\"s3cret\"}]}".to_vec()
    }

    #[test]
    fn test_create_and_open_vault() {
        let plaintext = test_plaintext();
        let passphrase = "correcthorsebatterystaple";
        let vault = create_vault_file(&plaintext, passphrase).unwrap();
        let (decrypted, version) = open_vault_file(&vault, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
        assert_eq!(version, 1);
    }

    #[test]
    fn test_wrong_passphrase() {
        let vault = create_vault_file(b"secret", "correct").unwrap();
        assert!(open_vault_file(&vault, "wrong").is_err());
    }

    #[test]
    fn test_save_increments_version() {
        let mut vault = create_vault_file(b"data", "pass").unwrap();
        assert_eq!(vault.version, 1);
        save_vault_file(&mut vault, b"data", "pass").unwrap();
        assert_eq!(vault.version, 2);
        save_vault_file(&mut vault, b"data", "pass").unwrap();
        assert_eq!(vault.version, 3);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let vault = create_vault_file(&test_plaintext(), "passphrase").unwrap();
        let bytes = serialize_vault_file(&vault);
        let restored = deserialize_vault_file(&bytes).unwrap();
        assert_eq!(restored.version, vault.version);
        assert_eq!(restored.salt, vault.salt);
        assert_eq!(restored.encrypted_data, vault.encrypted_data);
    }

    #[test]
    fn test_serialize_deserialize_decrypt() {
        let plaintext = test_plaintext();
        let passphrase = "mypassword";
        let vault = create_vault_file(&plaintext, passphrase).unwrap();
        let bytes = serialize_vault_file(&vault);
        let restored = deserialize_vault_file(&bytes).unwrap();
        let (decrypted, _) = open_vault_file(&restored, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_integrity_tamper_detected() {
        let mut vault = create_vault_file(b"secret", "pass").unwrap();
        if !vault.encrypted_data.is_empty() {
            vault.encrypted_data[0] ^= 0xFF;
        }
        assert!(!verify_vault_integrity(&vault, "pass").unwrap());
    }

    #[test]
    fn test_verify_integrity_valid() {
        let vault = create_vault_file(b"data", "passphrase").unwrap();
        assert!(verify_vault_integrity(&vault, "passphrase").unwrap());
    }

    #[test]
    fn test_deserialize_invalid_magic() {
        let data = vec![0u8; 100];
        assert!(deserialize_vault_file(&data).is_err());
    }

    #[test]
    fn test_save_preserves_data() {
        let plaintext = test_plaintext();
        let passphrase = "test123";
        let mut vault = create_vault_file(&plaintext, passphrase).unwrap();
        save_vault_file(&mut vault, &plaintext, passphrase).unwrap();
        let (decrypted, version) = open_vault_file(&vault, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
        assert_eq!(version, 2);
    }
}
