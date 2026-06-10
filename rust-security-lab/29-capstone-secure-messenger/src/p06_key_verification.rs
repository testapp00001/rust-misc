//! # Lesson 06: Key Verification — Safety Numbers
//!
//! ## What Are Safety Numbers?
//!
//! Safety numbers are a human-readable representation of both parties' identity keys.
//! If Alice and Bob compare safety numbers (out of band) and they match, they can be
//! confident that no man-in-the-middle is present.
//!
//! ## How Safety Numbers Are Computed (Signal)
//!
//! ```
//! safety_number = Truncate(
//!     SHA-256(
//!         SHA-256(identity_key_A) ||
//!         SHA-256(identity_key_B) ||
//!         SHA-256(display_name_A || display_name_B)
//!     ),
//!     60 decimal digits
//! )
//!
//! Display as 12 groups of 5 digits:
//! 12345 67890 12345 67890 12345 67890
//! 12345 67890 12345 67890 12345 67890
//! ```
//!
//! ## Attack: MITM During Key Exchange
//!
//! Without out-of-band verification, an attacker who compromises the server
//! can substitute their own keys during key exchange. Safety numbers let users
//! detect this by comparing the expected fingerprint.
//!
//! ## Attack: QR Code Swap
//!
//! An attacker replaces the QR code with one containing their key.
//! **Defense**: Users should verify the safety number text matches,
//! not just scan the QR code.

use ed25519_dalek::VerifyingKey;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

/// A pair of identity keys for safety number computation.
#[derive(Clone, Debug)]
pub struct IdentityKeyPair {
    /// The Ed25519 verifying key bytes (32 bytes).
    pub key_bytes: [u8; 32],
    /// The user's display name.
    pub display_name: String,
}

/// Exercise 1: Compute the safety number bytes from two identity key pairs.
///
/// The safety number is computed as:
/// SHA-256(
///     SHA-256(identity_key_A_bytes) ||
///     SHA-256(identity_key_B_bytes) ||
///     SHA-256(display_name_A || display_name_B)
/// )
///
/// The keys are sorted to ensure both parties compute the same value.
///
/// Hints:
/// - Hash each key separately with SHA-256
/// - Concatenate display names in sorted order (by key bytes)
/// - Hash the concatenated names
/// - Concatenate the three hashes and hash again
/// - Return the final 32-byte hash
pub fn compute_safety_number_bytes(
    alice: &IdentityKeyPair,
    bob: &IdentityKeyPair,
) -> [u8; 32] {
    todo!("Compute safety number hash from two identity key pairs")
}

/// Exercise 2: Format safety number bytes as a human-readable string.
///
/// Format: 60 decimal digits, displayed in groups of 5, two lines of 6 groups.
///
/// Example output:
/// "12345 67890 12345 67890 12345 67890\n12345 67890 12345 67890 12345 67890"
///
/// Hints:
/// - Convert the 32-byte hash to a big integer
/// - Take the last 60 decimal digits (or pad with leading zeros)
/// - Split into groups of 5, separated by spaces
/// - Split into two lines of 6 groups each
pub fn format_safety_number(bytes: &[u8; 32]) -> String {
    todo!("Format safety number as human-readable string")
}

/// Exercise 3: Verify that two safety numbers match.
///
/// Compare two safety number strings character by character using
/// constant-time comparison to prevent timing attacks.
///
/// Hints:
/// - Use `ring::constant_time::verify_slices_are_equal`
/// - Convert both strings to bytes and compare
pub fn verify_safety_numbers(expected: &str, provided: &str) -> bool {
    todo!("Verify safety numbers match using constant-time comparison")
}

/// Exercise 4: Generate a QR code payload for safety number verification.
///
/// Returns a base64-encoded string containing:
/// - version byte (0x01)
/// - 32 bytes of alice's key
/// - 32 bytes of bob's key (sorted)
///
/// Hints:
/// - Sort the key bytes to ensure deterministic ordering
/// - Concatenate version + sorted keys
/// - Base64 encode the result
pub fn generate_qr_payload(alice: &IdentityKeyPair, bob: &IdentityKeyPair) -> String {
    todo!("Generate a QR code payload for safety number verification")
}

/// Exercise 5: Parse and verify a QR code payload.
///
/// Extracts the two key pairs from a base64-encoded QR payload.
/// Returns (key_a, key_b) in sorted order, or None if the payload is invalid.
///
/// Hints:
/// - Base64 decode the payload
/// - Check version byte is 0x01
/// - Check total length is 65 bytes (1 + 32 + 32)
/// - Extract and return the two 32-byte keys
pub fn parse_qr_payload(payload: &str) -> Option<([u8; 32], [u8; 32])> {
    todo!("Parse QR code payload and extract keys")
}

/// Exercise 6: Compute safety number from raw key bytes (not IdentityKeyPair).
///
/// Convenience function that creates temporary IdentityKeyPairs and computes
/// the safety number.
pub fn safety_number_from_keys(
    key_a: &[u8; 32],
    name_a: &str,
    key_b: &[u8; 32],
    name_b: &str,
) -> [u8; 32] {
    todo!("Compute safety number from raw key bytes and names")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pair(key_byte: u8, name: &str) -> IdentityKeyPair {
        IdentityKeyPair {
            key_bytes: [key_byte; 32],
            display_name: name.to_string(),
        }
    }

    #[test]
    fn test_safety_number_deterministic() {
        let alice = make_pair(0xAA, "Alice");
        let bob = make_pair(0xBB, "Bob");
        let sn1 = compute_safety_number_bytes(&alice, &bob);
        let sn2 = compute_safety_number_bytes(&alice, &bob);
        assert_eq!(sn1, sn2, "Safety number should be deterministic");
    }

    #[test]
    fn test_safety_number_symmetric() {
        let alice = make_pair(0xAA, "Alice");
        let bob = make_pair(0xBB, "Bob");
        let sn_ab = compute_safety_number_bytes(&alice, &bob);
        let sn_ba = compute_safety_number_bytes(&bob, &alice);
        assert_eq!(sn_ab, sn_ba, "Safety number should be the same regardless of order");
    }

    #[test]
    fn test_safety_number_different_keys() {
        let alice = make_pair(0xAA, "Alice");
        let bob = make_pair(0xBB, "Bob");
        let carol = make_pair(0xCC, "Carol");
        let sn_ab = compute_safety_number_bytes(&alice, &bob);
        let sn_ac = compute_safety_number_bytes(&alice, &carol);
        assert_ne!(sn_ab, sn_ac, "Different key pairs should produce different safety numbers");
    }

    #[test]
    fn test_format_safety_number() {
        let sn = [0x12u8; 32];
        let formatted = format_safety_number(&sn);
        // Should be two lines, each with 6 groups of 5 digits
        let lines: Vec<&str> = formatted.split('\n').collect();
        assert_eq!(lines.len(), 2, "Should have two lines");
        for line in &lines {
            let groups: Vec<&str> = line.split(' ').collect();
            assert_eq!(groups.len(), 6, "Each line should have 6 groups");
            for group in &groups {
                assert_eq!(group.len(), 5, "Each group should be 5 digits");
                // Verify all characters are digits
                assert!(group.chars().all(|c| c.is_ascii_digit()));
            }
        }
    }

    #[test]
    fn test_verify_safety_numbers_match() {
        let formatted = "12345 67890 12345 67890 12345 67890\n12345 67890 12345 67890 12345 67890";
        assert!(verify_safety_numbers(formatted, formatted));
    }

    #[test]
    fn test_verify_safety_numbers_mismatch() {
        let a = "12345 67890 12345 67890 12345 67890\n12345 67890 12345 67890 12345 67890";
        let b = "99999 67890 12345 67890 12345 67890\n12345 67890 12345 67890 12345 67890";
        assert!(!verify_safety_numbers(a, b));
    }

    #[test]
    fn test_qr_payload_roundtrip() {
        let alice = make_pair(0xAA, "Alice");
        let bob = make_pair(0xBB, "Bob");
        let payload = generate_qr_payload(&alice, &bob);
        let parsed = parse_qr_payload(&payload);
        assert!(parsed.is_some());
        let (k1, k2) = parsed.unwrap();
        // Keys should be the same pair (sorted)
        assert!(
            (k1 == alice.key_bytes && k2 == bob.key_bytes) ||
            (k1 == bob.key_bytes && k2 == alice.key_bytes)
        );
    }

    #[test]
    fn test_safety_number_from_keys() {
        let sn = safety_number_from_keys(&[0xAA; 32], "Alice", &[0xBB; 32], "Bob");
        assert_eq!(sn.len(), 32);
        // Should be deterministic
        let sn2 = safety_number_from_keys(&[0xAA; 32], "Alice", &[0xBB; 32], "Bob");
        assert_eq!(sn, sn2);
    }
}
