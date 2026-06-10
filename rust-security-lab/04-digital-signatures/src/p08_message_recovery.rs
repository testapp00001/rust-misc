//! # Lesson 08: RSA Signature with Message Recovery
//!
//! ## Two RSA Signature Modes
//!
//! RSA supports two signature modes:
//!
//! ### Appendix Signatures (RSASSA-PKCS1-v1_5, RSASSA-PSS)
//! - Signature is sent ALONGSIDE the message
//! - Verifier needs both message and signature
//! - More common in practice
//!
//! ### Recovery Signatures (RSA with message recovery)
//! - The message is embedded IN the signature
//! - Verifier extracts the message from the signature
//! - Saves bandwidth: no separate message transmission needed
//! - Used in EMV (chip credit cards), some smart card protocols
//!
## How Recovery Works
//!
//! Signing:   sig = message_part ^ d mod n    (d = private exponent)
//! Recovery:  message_part = sig ^ e mod n    (e = public exponent)
//!
//! The recovered message is then checked against a hash or padding.
//!
## Attack Scenario: Bleichenbacher's Signature Forgery
//!
//! In 2006, Bleichenbacher showed that some RSA implementations incorrectly
//! verify PKCS#1 v1.5 signatures. If the verifier doesn't check padding
//! strictly, an attacker can forge signatures without the private key.
//!
//! The attack works because some implementations only check that the padding
//! starts correctly and don't verify the trailing bytes.
//!
## Defense
//!
//! 1. Use PSS padding (more secure than PKCS#1 v1.5)
//! 2. Verify ALL padding bytes strictly
//! 3. Use constant-time comparison for padding verification

use ring::rsa;
use ring::signature;
use ring::rand::SystemRandom;

/// Exercise 1: Generate an RSA keypair.
///
/// Returns (private_key_der_bytes, public_key_der_bytes).
///
/// Hints:
/// - Use `ring::rand::SystemRandom::new()` for RNG
/// - Use `ring::signature::RsaKeyPair::from_der(&private_key_der)` to parse
/// - Use `ring::signature::RsaKeyPair::public()` to get the public key
/// - Note: ring RSA key generation is not directly supported; use pre-generated keys
///
/// For this exercise, we'll use a simplified approach:
/// generate a keypair structure with known test values.
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    todo!("Generate or load an RSA keypair")
}

/// Exercise 2: Sign a message using RSA PKCS#1 v1.5 (appendix mode).
///
/// The signature is separate from the message. The verifier needs both.
///
/// Hints:
/// - Use `ring::signature::RsaKeyPair::sign(&rng, &padding, message)`
/// - Use `&signature::RSA_PKCS1_SHA256` for padding
/// - Returns the signature bytes
pub fn sign_pkcs1v15(private_key_der: &[u8], message: &[u8]) -> Vec<u8> {
    todo!("Sign with RSA PKCS#1 v1.5")
}

/// Exercise 3: Verify an RSA PKCS#1 v1.5 signature.
///
/// Returns true if the signature is valid for the given message and public key.
///
/// Hints:
/// - Use `ring::signature::verify(&algorithm, public_key, message, signature)`
/// - Use `&signature::RSA_PKCS1_SHA256` for the algorithm
pub fn verify_pkcs1v15(public_key_der: &[u8], message: &[u8], signature_bytes: &[u8]) -> bool {
    todo!("Verify RSA PKCS#1 v1.5 signature")
}

/// Exercise 4: Simulate RSA message recovery.
///
/// In a real recovery scheme, the message is embedded in the signature.
/// We simulate this by:
/// 1. Signing the message
/// 2. Prepending the message to the signature
/// 3. "Recovery" extracts the message from the combined blob
///
/// Returns (recovery_blob, message_length)
pub fn sign_with_recovery(private_key_der: &[u8], message: &[u8]) -> (Vec<u8>, usize) {
    todo!("Simulate RSA signing with message recovery")
}

/// Exercise 5: Recover the message from a recovery signature blob.
///
/// Extracts the message and verifies the signature.
///
/// Hints:
/// - The blob format is: message_len (4 bytes, big-endian) || message || signature
/// - Extract each part
/// - Verify the signature over the message
pub fn recover_message(
    public_key_der: &[u8],
    recovery_blob: &[u8],
    message_len: usize,
) -> Result<Vec<u8>, String> {
    todo!("Recover and verify message from recovery blob")
}

/// Exercise 6: Demonstrate Bleichenbacher's padding oracle attack (simplified).
///
/// Show what happens when padding is not strictly verified.
///
/// This simplified version demonstrates the CONCEPT: if we accept
/// signatures with garbage after the padding, we can forge them.
///
/// Returns true if the "forged" signature would be accepted by a
/// vulnerable verifier (one that doesn't check trailing bytes).
pub fn demonstrate_bleichenbacher(message: &[u8]) -> bool {
    todo!("Demonstrate Bleichenbacher signature forgery concept")
}

/// Exercise 7: Implement strict padding verification.
///
/// Verify that an RSA PKCS#1 v1.5 signature has correct padding:
/// 0x00 0x01 [0xFF...] 0x00 [DigestInfo] [hash]
///
/// Returns true only if padding is strictly correct.
///
/// Hints:
/// - Check byte 0 == 0x00
/// - Check byte 1 == 0x01
/// - Check padding bytes are all 0xFF
/// - Check separator 0x00
/// - Check DigestInfo prefix for SHA-256
/// - Check hash matches
pub fn verify_strict_padding(signature_bytes: &[u8], message: &[u8]) -> bool {
    todo!("Implement strict PKCS#1 v1.5 padding verification")
}

#[cfg(test)]
mod tests {
    use super::*;

    // We use a simplified test approach since ring RSA key generation
    // is not directly exposed. In practice, you'd load pre-generated keys.

    #[test]
    fn test_sign_verify_roundtrip() {
        // This test requires actual RSA keys — skip if not available
        // In a real setup, we'd load test keys from files
        let _message = b"test message for RSA signing";
        // Placeholder: actual implementation would use real keys
    }

    #[test]
    fn test_bleichenbacher_concept() {
        // Demonstrate the concept of the attack
        let message = b"important message";
        let vulnerable = demonstrate_bleichenbacher(message);
        // A vulnerable implementation would accept the forged signature
        // We just verify the function runs without panic
        let _ = vulnerable;
    }

    #[test]
    fn test_strict_padding_valid() {
        // Construct a valid PKCS#1 v1.5 signature block for testing
        // This is a simplified test — real implementation would use actual RSA
        let _message = b"test";
        // Placeholder for strict padding test
    }

    #[test]
    fn test_strict_padding_rejects_garbage() {
        let garbage = vec![0x00u8; 256];
        assert!(!verify_strict_padding(&garbage, b"test"));
    }

    #[test]
    fn test_strict_padding_rejects_wrong_header() {
        let mut sig = vec![0xFFu8; 256];
        sig[0] = 0x01; // Wrong: should be 0x00
        assert!(!verify_strict_padding(&sig, b"test"));
    }

    #[test]
    fn test_message_recovery_concept() {
        // Test the recovery blob format
        let message = b"recovered message";
        let message_len = message.len();
        // Construct a mock recovery blob
        let mut blob = Vec::new();
        blob.extend_from_slice(&(message_len as u32).to_be_bytes());
        blob.extend_from_slice(message);
        blob.extend_from_slice(&[0u8; 256]); // mock signature

        // Recovery should extract the message
        // (verification would fail with mock data, but extraction works)
        assert_eq!(blob.len(), 4 + message_len + 256);
    }

    #[test]
    fn test_padding_format() {
        // Verify we understand PKCS#1 v1.5 format
        // Block type 0x01: 0x00 0x01 [padding] 0x00 [data]
        let block = [
            0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x30, 0x31, 0x30, 0x0D,
        ];
        assert_eq!(block[0], 0x00);
        assert_eq!(block[1], 0x01);
        // Find separator
        let sep_pos = block.iter().position(|&b| b == 0x00 && block[0] == 0x00 && block[1] == 0x01);
        assert!(sep_pos.is_some());
    }
}
