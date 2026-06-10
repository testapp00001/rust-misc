//! # Lesson 06: Key Verification — Safety Numbers (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

/// A pair of identity keys for safety number computation.
#[derive(Clone, Debug)]
pub struct IdentityKeyPair {
    pub key_bytes: [u8; 32],
    pub display_name: String,
}

/// Compute the safety number bytes from two identity key pairs.
///
/// Keys are sorted to ensure both parties compute the same value.
pub fn compute_safety_number_bytes(
    alice: &IdentityKeyPair,
    bob: &IdentityKeyPair,
) -> [u8; 32] {
    // Hash each key
    let hash_a = Sha256::digest(&alice.key_bytes);
    let hash_b = Sha256::digest(&bob.key_bytes);

    // Sort by key bytes for deterministic ordering
    let (hash_first, hash_second, name_first, name_second) = if alice.key_bytes <= bob.key_bytes {
        (&hash_a, &hash_b, &alice.display_name, &bob.display_name)
    } else {
        (&hash_b, &hash_a, &bob.display_name, &alice.display_name)
    };

    // Hash concatenated display names
    let mut name_input = Vec::new();
    name_input.extend_from_slice(name_first.as_bytes());
    name_input.extend_from_slice(name_second.as_bytes());
    let hash_names = Sha256::digest(&name_input);

    // Final hash
    let mut hasher = Sha256::new();
    hasher.update(hash_first);
    hasher.update(hash_second);
    hasher.update(hash_names);
    hasher.finalize().into()
}

/// Format safety number bytes as a human-readable string.
///
/// 60 decimal digits, displayed in groups of 5, two lines of 6 groups.
pub fn format_safety_number(bytes: &[u8; 32]) -> String {
    // Convert bytes to decimal digits directly using repeated division
    // We need 60 decimal digits from 32 bytes (256 bits of entropy)
    let mut digits = Vec::with_capacity(60);
    let mut num: Vec<u8> = bytes.to_vec();

    // Convert big-endian bytes to decimal by repeated division by 10
    for _ in 0..60 {
        let mut remainder: u16 = 0;
        for byte in num.iter_mut() {
            let value = remainder * 256 + *byte as u16;
            *byte = (value / 10) as u8;
            remainder = value % 10;
        }
        digits.push((b'0' + remainder as u8) as char);
    }

    digits.reverse();
    let s: String = digits.into_iter().collect();

    // Split into groups of 5
    let groups: Vec<&str> = s.as_bytes()
        .chunks(5)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect();

    format!("{} {}",
        groups[0..6].join(" "),
        groups[6..12].join(" ")
    )
}

/// Verify that two safety numbers match using constant-time comparison.
pub fn verify_safety_numbers(expected: &str, provided: &str) -> bool {
    ring::constant_time::verify_slices_are_equal(
        expected.as_bytes(),
        provided.as_bytes(),
    ).is_ok()
}

/// Generate a QR code payload for safety number verification.
pub fn generate_qr_payload(alice: &IdentityKeyPair, bob: &IdentityKeyPair) -> String {
    let (key_first, key_second) = if alice.key_bytes <= bob.key_bytes {
        (&alice.key_bytes, &bob.key_bytes)
    } else {
        (&bob.key_bytes, &alice.key_bytes)
    };

    let mut payload = Vec::with_capacity(65);
    payload.push(0x01); // version
    payload.extend_from_slice(key_first);
    payload.extend_from_slice(key_second);

    general_purpose::STANDARD.encode(&payload)
}

/// Parse and verify a QR code payload.
pub fn parse_qr_payload(payload: &str) -> Option<([u8; 32], [u8; 32])> {
    let decoded = general_purpose::STANDARD.decode(payload).ok()?;

    if decoded.len() != 65 || decoded[0] != 0x01 {
        return None;
    }

    let mut key_a = [0u8; 32];
    let mut key_b = [0u8; 32];
    key_a.copy_from_slice(&decoded[1..33]);
    key_b.copy_from_slice(&decoded[33..65]);

    Some((key_a, key_b))
}

/// Compute safety number from raw key bytes.
pub fn safety_number_from_keys(
    key_a: &[u8; 32],
    name_a: &str,
    key_b: &[u8; 32],
    name_b: &str,
) -> [u8; 32] {
    let pair_a = IdentityKeyPair {
        key_bytes: *key_a,
        display_name: name_a.to_string(),
    };
    let pair_b = IdentityKeyPair {
        key_bytes: *key_b,
        display_name: name_b.to_string(),
    };
    compute_safety_number_bytes(&pair_a, &pair_b)
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
        assert_eq!(sn1, sn2);
    }

    #[test]
    fn test_safety_number_symmetric() {
        let alice = make_pair(0xAA, "Alice");
        let bob = make_pair(0xBB, "Bob");
        let sn_ab = compute_safety_number_bytes(&alice, &bob);
        let sn_ba = compute_safety_number_bytes(&bob, &alice);
        assert_eq!(sn_ab, sn_ba);
    }

    #[test]
    fn test_safety_number_different_keys() {
        let alice = make_pair(0xAA, "Alice");
        let bob = make_pair(0xBB, "Bob");
        let carol = make_pair(0xCC, "Carol");
        let sn_ab = compute_safety_number_bytes(&alice, &bob);
        let sn_ac = compute_safety_number_bytes(&alice, &carol);
        assert_ne!(sn_ab, sn_ac);
    }

    #[test]
    fn test_format_safety_number() {
        let sn = [0x12u8; 32];
        let formatted = format_safety_number(&sn);
        let lines: Vec<&str> = formatted.split('\n').collect();
        // May be one line with space separator or two lines
        let all_text = formatted.replace('\n', " ");
        let groups: Vec<&str> = all_text.split(' ').collect();
        assert_eq!(groups.len(), 12);
        for group in &groups {
            assert_eq!(group.len(), 5);
            assert!(group.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_verify_safety_numbers_match() {
        let formatted = "12345 67890 12345 67890 12345 67890 12345 67890 12345 67890 12345 67890";
        assert!(verify_safety_numbers(formatted, formatted));
    }

    #[test]
    fn test_verify_safety_numbers_mismatch() {
        let a = "12345 67890 12345 67890 12345 67890 12345 67890 12345 67890 12345 67890";
        let b = "99999 67890 12345 67890 12345 67890 12345 67890 12345 67890 12345 67890";
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
        assert!(
            (k1 == alice.key_bytes && k2 == bob.key_bytes) ||
            (k1 == bob.key_bytes && k2 == alice.key_bytes)
        );
    }

    #[test]
    fn test_safety_number_from_keys() {
        let sn = safety_number_from_keys(&[0xAA; 32], "Alice", &[0xBB; 32], "Bob");
        assert_eq!(sn.len(), 32);
        let sn2 = safety_number_from_keys(&[0xAA; 32], "Alice", &[0xBB; 32], "Bob");
        assert_eq!(sn, sn2);
    }
}
