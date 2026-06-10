//! # Lesson 06: Encrypted Container Format (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use serde::{Deserialize, Serialize};

/// Magic bytes identifying our container format
const MAGIC: &[u8; 4] = b"ESCF";

/// Current format version
const VERSION: u8 = 1;

/// Container header (before encryption)
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ContainerHeader {
    pub filename: String,
    pub original_size: u64,
    pub content_type: Option<String>,
    pub created_at: u64,
}

/// Serialize a container header to JSON bytes.
///
/// JSON is used for the header format because it's human-readable,
/// flexible, and doesn't require schema changes for optional fields.
pub fn serialize_header(header: &ContainerHeader) -> Result<Vec<u8>, String> {
    serde_json::to_vec(header).map_err(|e| format!("Header serialization failed: {}", e))
}

/// Deserialize a container header from JSON bytes.
pub fn deserialize_header(bytes: &[u8]) -> Result<ContainerHeader, String> {
    serde_json::from_slice(bytes).map_err(|e| format!("Header deserialization failed: {}", e))
}

/// Pack data into an encrypted container.
///
/// Container format:
/// ```text
/// [MAGIC: 4 bytes] [VERSION: 1 byte] [NONCE: 12 bytes] [ENCRYPTED: header_json || data]
/// ```
///
/// The header and data are concatenated and encrypted as a single AEAD message.
/// This means the auth tag covers both the header and the data — any tampering
/// with either is detected.
pub fn pack_container(
    key: &Key<Aes256Gcm>,
    header: &ContainerHeader,
    data: &[u8],
) -> Result<Vec<u8>, String> {
    let header_json = serialize_header(header)?;

    // Combine header and data into a single plaintext
    // Format: [header_len: 4 bytes] [header_json] [data]
    let header_len = (header_json.len() as u32).to_be_bytes();
    let mut plaintext = Vec::with_capacity(4 + header_json.len() + data.len());
    plaintext.extend_from_slice(&header_len);
    plaintext.extend_from_slice(&header_json);
    plaintext.extend_from_slice(data);

    // Encrypt the combined plaintext
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_slice())
        .map_err(|e| format!("Encryption failed: {:?}", e))?;

    // Build the container
    let mut container = Vec::with_capacity(4 + 1 + 12 + ciphertext.len());
    container.extend_from_slice(MAGIC);
    container.push(VERSION);
    container.extend_from_slice(&nonce);
    container.extend_from_slice(&ciphertext);

    Ok(container)
}

/// Unpack an encrypted container.
///
/// Validates magic bytes and version, then decrypts and splits the
/// header from the data.
pub fn unpack_container(
    key: &Key<Aes256Gcm>,
    container: &[u8],
) -> Result<(ContainerHeader, Vec<u8>), String> {
    // Validate minimum size: magic(4) + version(1) + nonce(12) + tag(16)
    if container.len() < 33 {
        return Err("Container too small".to_string());
    }

    // Validate magic bytes
    if &container[0..4] != MAGIC {
        return Err("Invalid magic bytes".to_string());
    }

    // Validate version
    if container[4] != VERSION {
        return Err(format!("Unsupported version: {}", container[4]));
    }

    // Extract nonce (12 bytes after magic+version)
    let nonce = Nonce::from_slice(&container[5..17]);

    // Decrypt the rest
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher
        .decrypt(nonce, &container[17..])
        .map_err(|e| format!("Decryption failed: {:?}", e))?;

    // Split header and data
    if plaintext.len() < 4 {
        return Err("Decrypted data too short".to_string());
    }

    let header_len = u32::from_be_bytes([plaintext[0], plaintext[1], plaintext[2], plaintext[3]]) as usize;

    if plaintext.len() < 4 + header_len {
        return Err("Header length exceeds data".to_string());
    }

    let header = deserialize_header(&plaintext[4..4 + header_len])?;
    let data = plaintext[4 + header_len..].to_vec();

    Ok((header, data))
}

/// Validate container format without decrypting.
///
/// Checks structural validity:
/// - Minimum size (4 + 1 + 12 + 16 = 33 bytes)
/// - Magic bytes match
/// - Version is supported
pub fn validate_container_format(container: &[u8]) -> Result<(), String> {
    if container.len() < 33 {
        return Err("Container too small (minimum 33 bytes)".to_string());
    }

    if &container[0..4] != MAGIC {
        return Err("Invalid magic bytes".to_string());
    }

    if container[4] != VERSION {
        return Err(format!("Unsupported version: {}", container[4]));
    }

    Ok(())
}

/// Extract just the header metadata (requires decryption).
///
/// The header is encrypted, so we must decrypt to read it.
/// This is useful for listing container contents without extracting
/// the full data.
pub fn get_container_info(
    key: &Key<Aes256Gcm>,
    container: &[u8],
) -> Result<ContainerHeader, String> {
    let (header, _) = unpack_container(key, container)?;
    Ok(header)
}

/// Pack multiple files into a single container.
///
/// Uses JSON serialization for the file list, which is then encrypted
/// as a single blob. Each file entry includes its name and contents.
pub fn pack_multi_container(
    key: &Key<Aes256Gcm>,
    files: &[(String, Vec<u8>)],
) -> Result<Vec<u8>, String> {
    // Serialize files as a JSON array of {name, data} objects
    let file_list: Vec<serde_json::Value> = files
        .iter()
        .map(|(name, data)| {
            serde_json::json!({
                "name": name,
                "data": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data),
            })
        })
        .collect();

    let serialized = serde_json::to_vec(&file_list)
        .map_err(|e| format!("Multi-file serialization failed: {}", e))?;

    // Create a header for the multi-file container
    let header = ContainerHeader {
        filename: "__multi__".to_string(),
        original_size: serialized.len() as u64,
        content_type: Some("application/x-multi-file".to_string()),
        created_at: 0,
    };

    pack_container(key, &header, &serialized)
}

/// Unpack a multi-file container.
///
/// Returns a Vec of (filename, data) pairs.
pub fn unpack_multi_container(
    key: &Key<Aes256Gcm>,
    container: &[u8],
) -> Result<Vec<(String, Vec<u8>)>, String> {
    let (_, data) = unpack_container(key, container)?;

    let file_list: Vec<serde_json::Value> = serde_json::from_slice(&data)
        .map_err(|e| format!("Multi-file deserialization failed: {}", e))?;

    let mut result = Vec::new();
    for entry in &file_list {
        let name = entry["name"]
            .as_str()
            .ok_or("Missing file name")?
            .to_string();
        let data_b64 = entry["data"]
            .as_str()
            .ok_or("Missing file data")?
            .to_string();
        let data = base64::engine::general_purpose::STANDARD
            .decode(&data_b64)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;
        result.push((name, data));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_header() -> ContainerHeader {
        ContainerHeader {
            filename: "secret.txt".to_string(),
            original_size: 1024,
            content_type: Some("text/plain".to_string()),
            created_at: 1700000000,
        }
    }

    #[test]
    fn test_header_serialization() {
        let header = test_header();
        let bytes = serialize_header(&header).unwrap();
        let deserialized = deserialize_header(&bytes).unwrap();
        assert_eq!(header, deserialized);
    }

    #[test]
    fn test_container_roundtrip() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let header = test_header();
        let data = b"This is the secret file content.";

        let container = pack_container(&key, &header, data).unwrap();
        let (dec_header, dec_data) = unpack_container(&key, &container).unwrap();

        assert_eq!(header, dec_header);
        assert_eq!(dec_data, data);
    }

    #[test]
    fn test_container_starts_with_magic() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let container = pack_container(&key, &test_header(), b"test").unwrap();

        assert_eq!(&container[0..4], MAGIC, "Container should start with magic bytes");
        assert_eq!(container[4], VERSION, "Version byte should follow magic");
    }

    #[test]
    fn test_container_wrong_key_fails() {
        let key1 = Aes256Gcm::generate_key(&mut OsRng);
        let key2 = Aes256Gcm::generate_key(&mut OsRng);
        let container = pack_container(&key1, &test_header(), b"data").unwrap();

        let result = unpack_container(&key2, &container);
        assert!(result.is_err(), "Wrong key should fail");
    }

    #[test]
    fn test_validate_container_format() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let container = pack_container(&key, &test_header(), b"data").unwrap();

        assert!(validate_container_format(&container).is_ok());

        // Too short
        assert!(validate_container_format(&[0u8; 10]).is_err());

        // Wrong magic
        let mut bad = container.clone();
        bad[0] = b'X';
        assert!(validate_container_format(&bad).is_err());
    }

    #[test]
    fn test_get_container_info() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let header = test_header();
        let container = pack_container(&key, &header, b"data").unwrap();

        let info = get_container_info(&key, &container).unwrap();
        assert_eq!(info.filename, "secret.txt");
        assert_eq!(info.original_size, 1024);
    }

    #[test]
    fn test_multi_file_container() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let files = vec![
            ("file1.txt".to_string(), b"content1".to_vec()),
            ("file2.txt".to_string(), b"content2".to_vec()),
        ];

        let container = pack_multi_container(&key, &files).unwrap();
        let unpacked = unpack_multi_container(&key, &container).unwrap();

        assert_eq!(unpacked.len(), 2);
        assert_eq!(unpacked[0].0, "file1.txt");
        assert_eq!(unpacked[0].1, b"content1");
        assert_eq!(unpacked[1].0, "file2.txt");
        assert_eq!(unpacked[1].1, b"content2");
    }

    #[test]
    fn test_container_different_each_time() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let header = test_header();
        let data = b"same data";

        let c1 = pack_container(&key, &header, data).unwrap();
        let c2 = pack_container(&key, &header, data).unwrap();

        // Different nonces → different containers
        assert_ne!(c1, c2);
    }
}
