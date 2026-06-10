//! # Lesson 06: Encrypted Container Format
//!
//! ## What is an Encrypted Container?
//!
//! An encrypted container is a file format that bundles encrypted data with
//! metadata needed for decryption. Think of it as a "vault" that stores
//! your encrypted files along with the information needed to unlock them.
//!
//! ## Container Format Design
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │  Magic Bytes (4)  │  Version (1)  │  Header Len (4) │
//! ├─────────────────────────────────────────────────────┤
//! │  Nonce (12 bytes)                                    │
//! ├─────────────────────────────────────────────────────┤
//! │  Encrypted Header (JSON metadata)                    │
//! ├─────────────────────────────────────────────────────┤
//! │  Encrypted Data (file contents)                      │
//! ├─────────────────────────────────────────────────────┤
//! │  Auth Tag (16 bytes)                                 │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Properties
//!
//! - **Magic bytes**: Identify the file format (not secret, but authenticated)
//! - **Version**: Allows format evolution without breaking compatibility
//! - **Header**: Contains metadata (filename, size, timestamps) — encrypted
//! - **Data**: The actual file contents — encrypted
//! - **Auth tag**: Single tag covers both header and data (AEAD)
//!
//! ## Attack Scenario: Container Manipulation
//!
//! Without authenticated encryption:
//! 1. Attacker swaps two containers (replay attack)
//! 2. Attacker modifies the header to change the filename
//! 3. Attacker truncates the container to corrupt data
//!
//! With AEAD: All of these are detected because the auth tag covers
//! the entire container contents.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    Aes256Gcm, Key, Nonce,
};
use serde::{Deserialize, Serialize};

/// Magic bytes identifying our container format
const MAGIC: &[u8; 4] = b"ESCF"; // Encrypted Secure Container Format

/// Current format version
const VERSION: u8 = 1;

/// Container header (before encryption)
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ContainerHeader {
    /// Original filename
    pub filename: String,
    /// Original file size in bytes
    pub original_size: u64,
    /// MIME type (optional)
    pub content_type: Option<String>,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
}

/// Exercise 1: Serialize a container header to JSON bytes.
///
/// The header is serialized to JSON before encryption. This allows
/// flexible metadata without changing the binary format.
///
/// # Hints
/// - Use `serde_json::to_vec(&header)` to serialize to JSON bytes
pub fn serialize_header(header: &ContainerHeader) -> Result<Vec<u8>, String> {
    todo!("Serialize container header to JSON bytes")
}

/// Exercise 2: Deserialize a container header from JSON bytes.
///
/// # Hints
/// - Use `serde_json::from_slice(&bytes)` to deserialize
pub fn deserialize_header(bytes: &[u8]) -> Result<ContainerHeader, String> {
    todo!("Deserialize container header from JSON bytes")
}

/// Exercise 3: Pack data into an encrypted container.
///
/// Creates a container with the format:
/// `[MAGIC (4)] [VERSION (1)] [NONCE (12)] [ENCRYPTED HEADER + DATA]`
///
/// The header and data are encrypted together as a single AEAD message.
///
/// # Arguments
/// * `key` - AES-256 encryption key
/// * `header` - Container metadata
/// * `data` - File contents to encrypt
///
/// # Hints
/// - Serialize header to JSON
/// - Combine header and data: `let plaintext = [header_json, data].concat()`
/// - Encrypt the combined plaintext with AEAD
/// - Prepend MAGIC + VERSION + NONCE to the ciphertext
pub fn pack_container(
    key: &Key<Aes256Gcm>,
    header: &ContainerHeader,
    data: &[u8],
) -> Result<Vec<u8>, String> {
    todo!("Pack header and data into an encrypted container")
}

/// Exercise 4: Unpack an encrypted container.
///
/// Extracts the header and data from an encrypted container.
///
/// # Arguments
/// * `key` - AES-256 decryption key
/// * `container` - The encrypted container bytes
///
/// # Returns
/// A tuple of (ContainerHeader, data_bytes)
///
/// # Hints
/// - Verify magic bytes
/// - Verify version
/// - Extract nonce (12 bytes after magic+version)
/// - Decrypt the remainder
/// - Split decrypted data into header JSON and file data
pub fn unpack_container(
    key: &Key<Aes256Gcm>,
    container: &[u8],
) -> Result<(ContainerHeader, Vec<u8>), String> {
    todo!("Unpack and decrypt an encrypted container")
}

/// Exercise 5: Validate container format without decrypting.
///
/// Check that the container has valid magic bytes, version, and minimum size.
///
/// # Hints
/// - Minimum size: 4 (magic) + 1 (version) + 12 (nonce) + 16 (tag) = 33 bytes
/// - Check magic bytes == "ESCF"
/// - Check version == 1
pub fn validate_container_format(container: &[u8]) -> Result<(), String> {
    todo!("Validate container format without decrypting")
}

/// Exercise 6: Extract just the header metadata without decrypting the data.
///
/// This is useful for listing container contents without needing the key.
/// However, the header IS encrypted, so this requires the key.
///
/// # Hints
/// - Decrypt the container, but only parse the header
/// - You still need to decrypt to get the header
pub fn get_container_info(
    key: &Key<Aes256Gcm>,
    container: &[u8],
) -> Result<ContainerHeader, String> {
    todo!("Extract container metadata (requires decryption)")
}

/// Exercise 7: Pack multiple files into a single container.
///
/// Store multiple files by concatenating their data with a length prefix,
/// and including filenames in the header.
///
/// # Hints
/// - Serialize each file's data as: `[name_len (4 bytes)] [name] [data_len (8 bytes)] [data]`
/// - Or use JSON serialization for the file list
pub fn pack_multi_container(
    key: &Key<Aes256Gcm>,
    files: &[(String, Vec<u8>)],
) -> Result<Vec<u8>, String> {
    todo!("Pack multiple files into a single encrypted container")
}

/// Exercise 8: Unpack a multi-file container.
///
/// # Hints
/// - Deserialize the file list from the decrypted data
/// - Return Vec of (filename, data) pairs
pub fn unpack_multi_container(
    key: &Key<Aes256Gcm>,
    container: &[u8],
) -> Result<Vec<(String, Vec<u8>)>, String> {
    todo!("Unpack a multi-file container")
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
