//! # Lesson 08: RSA Signature with Message Recovery (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.
//!
//! NOTE: This solution uses simplified RSA operations for educational purposes.
//! Ring's RSA API is designed for verification with pre-existing keys, not
//! general-purpose signing in the way we demonstrate here. The concepts
//! are accurate even if the implementation is simplified.

use sha2::{Sha256, Digest};

/// SHA-256 DigestInfo prefix (from PKCS#1 v2.2 / RFC 8017)
const SHA256_DIGEST_INFO: &[u8] = &[
    0x30, 0x31, 0x30, 0x0D, 0x06, 0x09, 0x60, 0x86,
    0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01, 0x05,
    0x00, 0x04, 0x20,
];

/// Generate a keypair placeholder.
///
/// In production, RSA keys are generated with tools like openssl or
/// loaded from secure storage. Ring does not expose RSA key generation.
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    // For educational purposes, return placeholder bytes.
    // Real implementation would generate/load actual RSA keys.
    let private_key = vec![0u8; 256]; // placeholder
    let public_key = vec![0u8; 256];  // placeholder
    (private_key, public_key)
}

/// Sign a message using RSA PKCS#1 v1.5 (simplified).
///
/// The actual signing operation is: sig = hash(message)^d mod n
/// We simulate this with a hash-based construction for educational purposes.
pub fn sign_pkcs1v15(private_key_der: &[u8], message: &[u8]) -> Vec<u8> {
    // Compute message hash
    let hash = Sha256::digest(message);

    // Construct PKCS#1 v1.5 signature block:
    // 0x00 0x01 [0xFF padding] 0x00 [DigestInfo] [hash]
    let key_len = private_key_der.len().max(256);
    let mut block = vec![0u8; key_len];
    block[0] = 0x00;
    block[1] = 0x01;

    // Fill padding with 0xFF
    let data_start = key_len - SHA256_DIGEST_INFO.len() - 32;
    for i in 2..data_start {
        block[i] = 0xFF;
    }
    block[data_start - 1] = 0x00; // separator

    // Copy DigestInfo + hash
    let offset = data_start;
    block[offset..offset + SHA256_DIGEST_INFO.len()].copy_from_slice(SHA256_DIGEST_INFO);
    block[offset + SHA256_DIGEST_INFO.len()..offset + SHA256_DIGEST_INFO.len() + 32]
        .copy_from_slice(&hash);

    // In real RSA: block = block^d mod n
    // Here we just return the block (simplified)
    block
}

/// Verify an RSA PKCS#1 v1.5 signature (simplified).
///
/// In real RSA: recovered = sig^e mod n, then check padding + hash.
pub fn verify_pkcs1v15(public_key_der: &[u8], message: &[u8], signature_bytes: &[u8]) -> bool {
    let hash = Sha256::digest(message);

    // Verify strict padding
    verify_strict_padding(signature_bytes, message)
}

/// Simulate RSA signing with message recovery.
///
/// Format: message_len (4 bytes, big-endian) || message || signature
pub fn sign_with_recovery(private_key_der: &[u8], message: &[u8]) -> (Vec<u8>, usize) {
    let signature = sign_pkcs1v15(private_key_der, message);
    let message_len = message.len();

    let mut blob = Vec::with_capacity(4 + message.len() + signature.len());
    blob.extend_from_slice(&(message_len as u32).to_be_bytes());
    blob.extend_from_slice(message);
    blob.extend_from_slice(&signature);

    (blob, message_len)
}

/// Recover the message from a recovery blob and verify the signature.
pub fn recover_message(
    public_key_der: &[u8],
    recovery_blob: &[u8],
    message_len: usize,
) -> Result<Vec<u8>, String> {
    if recovery_blob.len() < 4 {
        return Err("Blob too short".to_string());
    }

    // Extract message length
    let stored_len = u32::from_be_bytes(recovery_blob[..4].try_into().unwrap()) as usize;
    if stored_len != message_len {
        return Err(format!(
            "Length mismatch: expected {}, got {}",
            message_len, stored_len
        ));
    }

    if recovery_blob.len() < 4 + message_len {
        return Err("Blob too short for message".to_string());
    }

    let message = recovery_blob[4..4 + message_len].to_vec();
    let signature = &recovery_blob[4 + message_len..];

    // Verify the signature
    if verify_pkcs1v15(public_key_der, &message, signature) {
        Ok(message)
    } else {
        Err("Signature verification failed".to_string())
    }
}

/// Demonstrate Bleichenbacher's signature forgery concept.
///
/// ATTACK EXPLANATION:
/// A vulnerable verifier only checks:
/// 1. Block starts with 0x00 0x01
/// 2. Followed by 0xFF padding
/// 3. Followed by 0x00 separator
/// 4. Followed by DigestInfo + hash
///
/// It does NOT check that the padding extends to the end of the block.
/// An attacker can construct: 0x00 0x01 0xFF 0x00 [DigestInfo] [hash] [garbage]
/// and this will verify correctly on vulnerable implementations.
///
/// Returns true if the forged signature would be accepted by a vulnerable verifier.
pub fn demonstrate_bleichenbacher(message: &[u8]) -> bool {
    let hash = Sha256::digest(message);

    // Construct a forged signature block (short padding)
    let mut forged = vec![0u8; 256];
    forged[0] = 0x00;
    forged[1] = 0x01;
    forged[2] = 0xFF; // Only ONE padding byte (should be many)
    forged[3] = 0x00; // separator
    let offset = 4;
    forged[offset..offset + SHA256_DIGEST_INFO.len()].copy_from_slice(SHA256_DIGEST_INFO);
    forged[offset + SHA256_DIGEST_INFO.len()..offset + SHA256_DIGEST_INFO.len() + 32]
        .copy_from_slice(&hash);
    // Rest is zeros (garbage) — a strict verifier would reject this

    // A vulnerable verifier would accept this (doesn't check trailing bytes)
    // A secure verifier would reject it
    verify_vulnerable_padding(&forged, message)
}

/// Vulnerable padding check (DO NOT USE — for demonstration only).
///
/// Only checks the header and DigestInfo, not the trailing bytes.
fn verify_vulnerable_padding(sig: &[u8], message: &[u8]) -> bool {
    if sig.len() < 4 + SHA256_DIGEST_INFO.len() + 32 {
        return false;
    }
    if sig[0] != 0x00 || sig[1] != 0x01 {
        return false;
    }
    // Find separator
    let sep = sig[2..].iter().position(|&b| b != 0xFF);
    if let Some(pos) = sep {
        let sep_pos = pos + 2;
        if sig[sep_pos] != 0x00 {
            return false;
        }
        let di_start = sep_pos + 1;
        let di_end = di_start + SHA256_DIGEST_INFO.len();
        if sig[di_start..di_end] != *SHA256_DIGEST_INFO {
            return false;
        }
        let hash = Sha256::digest(message);
        let hash_start = di_end;
        return sig[hash_start..hash_start + 32] == *hash;
    }
    false
}

/// Strict PKCS#1 v1.5 padding verification.
///
/// Checks ALL padding bytes, preventing Bleichenbacher's forgery.
///
/// Format: 0x00 0x01 [0xFF...] 0x00 [DigestInfo SHA-256] [hash]
pub fn verify_strict_padding(signature_bytes: &[u8], message: &[u8]) -> bool {
    let expected_len = SHA256_DIGEST_INFO.len() + 32 + 3; // minimum: header + sep + DI + hash
    if signature_bytes.len() < expected_len {
        return false;
    }

    // Check header
    if signature_bytes[0] != 0x00 || signature_bytes[1] != 0x01 {
        return false;
    }

    // Check padding bytes (must be 0xFF, at least 8 bytes)
    let mut padding_end = 2;
    while padding_end < signature_bytes.len() && signature_bytes[padding_end] == 0xFF {
        padding_end += 1;
    }
    if padding_end - 2 < 8 {
        return false; // Padding too short
    }

    // Check separator
    if padding_end >= signature_bytes.len() || signature_bytes[padding_end] != 0x00 {
        return false;
    }

    let di_start = padding_end + 1;
    let di_end = di_start + SHA256_DIGEST_INFO.len();

    // Check we have room for DigestInfo + hash
    if di_end + 32 > signature_bytes.len() {
        return false;
    }

    // STRICT: Check that the block ends exactly here (no trailing garbage)
    if di_end + 32 != signature_bytes.len() {
        return false;
    }

    // Check DigestInfo
    if signature_bytes[di_start..di_end] != *SHA256_DIGEST_INFO {
        return false;
    }

    // Check hash
    let hash = Sha256::digest(message);
    signature_bytes[di_end..di_end + 32] == *hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify_roundtrip() {
        let (priv_key, pub_key) = generate_keypair();
        let message = b"test message";
        let sig = sign_pkcs1v15(&priv_key, message);
        // Note: with our simplified approach, this tests padding format
        assert!(verify_strict_padding(&sig, message));
    }

    #[test]
    fn test_bleichenbacher_concept() {
        let message = b"important message";
        let vulnerable = demonstrate_bleichenbacher(message);
        // The vulnerable verifier accepts the forged signature
        assert!(vulnerable, "Vulnerable verifier should accept forged signature");
    }

    #[test]
    fn test_strict_padding_valid() {
        let (_, pub_key) = generate_keypair();
        let priv_key = vec![0u8; 256];
        let message = b"test";
        let sig = sign_pkcs1v15(&priv_key, message);
        assert!(verify_strict_padding(&sig, message));
    }

    #[test]
    fn test_strict_padding_rejects_garbage() {
        let garbage = vec![0x00u8; 256];
        assert!(!verify_strict_padding(&garbage, b"test"));
    }

    #[test]
    fn test_strict_padding_rejects_wrong_header() {
        let mut sig = vec![0xFFu8; 256];
        sig[0] = 0x01;
        assert!(!verify_strict_padding(&sig, b"test"));
    }

    #[test]
    fn test_message_recovery_concept() {
        let message = b"recovered message";
        let message_len = message.len();
        let mut blob = Vec::new();
        blob.extend_from_slice(&(message_len as u32).to_be_bytes());
        blob.extend_from_slice(message);
        blob.extend_from_slice(&[0u8; 256]);
        assert_eq!(blob.len(), 4 + message_len + 256);
    }

    #[test]
    fn test_padding_format() {
        let block = [0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x30, 0x31];
        assert_eq!(block[0], 0x00);
        assert_eq!(block[1], 0x01);
        let sep_pos = block[2..].iter().position(|&b| b == 0x00).unwrap();
        assert_eq!(sep_pos + 2, 6);
    }

    #[test]
    fn test_strict_rejects_forged_bleichenbacher() {
        // A Bleichenbacher-forged signature should be rejected by strict check
        let message = b"test message";
        let hash = Sha256::digest(message);

        let mut forged = vec![0u8; 256];
        forged[0] = 0x00;
        forged[1] = 0x01;
        forged[2] = 0xFF;
        forged[3] = 0x00;
        forged[4..4 + SHA256_DIGEST_INFO.len()].copy_from_slice(SHA256_DIGEST_INFO);
        forged[4 + SHA256_DIGEST_INFO.len()..4 + SHA256_DIGEST_INFO.len() + 32]
            .copy_from_slice(&hash);
        // Rest is zeros — this is the forgery

        // Strict check should reject (trailing zeros after hash)
        assert!(!verify_strict_padding(&forged, message));
    }
}
