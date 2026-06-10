//! # Lesson 06: Signature Malleability Attack
//!
//! ## What is Signature Malleability?
//!
//! ECDSA signatures have a property called malleability: given a valid signature
//! (r, s), the value (r, n - s) is ALSO a valid signature for the same message
//! and public key, where n is the curve order.
//!
//! This means an attacker can take a valid transaction, flip s to n-s, and
//! broadcast a modified version that is still valid.
//!
//! ## Attack Scenario: Transaction Malleability
//!
//! This was a real attack on Bitcoin (pre-SegWit):
//! 1. Alice broadcasts transaction T with signature (r, s)
//! 2. Mallory intercepts T and creates T' with signature (r, n-s)
//! 3. Both T and T' are valid and spend the same coins
//! 4. Alice's wallet sees T was "rejected" (different txid) and might respend
//!
//! ## Defense: Low-S Normalization
//!
//! Require that s <= n/2. If s > n/2, replace it with n - s.
//! This ensures exactly one valid representation for each signature.
//!
//! Both Bitcoin (BIP-62, BIP-66) and Ethereum enforce low-s normalization.
//!
//! ## Why Ed25519 Doesn't Have This Problem
//!
//! Ed25519 uses a different signature construction that does not have this
//! malleability property. Each signature is unique for a given key+message.

/// Curve order for secp256k1 (used in Bitcoin/Ethereum ECDSA)
/// n = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
const SECP256K1_ORDER: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
    0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
    0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
];

/// Half the curve order (n / 2) for low-s check
const SECP256K1_HALF_ORDER: [u8; 32] = [
    0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D,
    0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
];

/// A simplified ECDSA signature with (r, s) components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcdsaSignature {
    pub r: [u8; 32],
    pub s: [u8; 32],
}

impl EcdsaSignature {
    pub fn new(r: [u8; 32], s: [u8; 32]) -> Self {
        Self { r, s }
    }

    /// Serialize to 64 bytes (r || s).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(&self.r);
        bytes.extend_from_slice(&self.s);
        bytes
    }

    /// Deserialize from 64 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 64 {
            return None;
        }
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&bytes[..32]);
        s.copy_from_slice(&bytes[32..]);
        Some(Self { r, s })
    }
}

/// Compare two 256-bit unsigned integers encoded as big-endian bytes.
/// Returns: -1 if a < b, 0 if a == b, 1 if a > b
fn compare_256(a: &[u8; 32], b: &[u8; 32]) -> i8 {
    for i in 0..32 {
        if a[i] < b[i] {
            return -1;
        }
        if a[i] > b[i] {
            return 1;
        }
    }
    0
}

/// Subtract two 256-bit unsigned integers: result = a - b (mod 2^256).
/// Assumes a >= b.
fn sub_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut borrow = 0i32;
    for i in (0..32).rev() {
        let diff = a[i] as i32 - b[i] as i32 - borrow;
        if diff < 0 {
            result[i] = (diff + 256) as u8;
            borrow = 1;
        } else {
            result[i] = diff as u8;
            borrow = 0;
        }
    }
    result
}

/// Exercise 1: Create the malleable counterpart of a signature.
///
/// Given signature (r, s), create (r, n - s) where n is the curve order.
///
/// Hints:
/// - If s <= n/2, the malleable counterpart has s' = n - s
/// - If s > n/2, the malleable counterpart has s' = n - s (which is < n/2)
/// - Compare s with n to determine order, then compute n - s
pub fn create_malleable_signature(sig: &EcdsaSignature) -> EcdsaSignature {
    todo!("Create the malleable counterpart: (r, n-s)")
}

/// Exercise 2: Check if a signature has "low s" (s <= n/2).
///
/// A signature with high s can be malleated to low s.
/// Enforcing low-s prevents this attack.
///
/// Hints:
/// - Compare s with SECP256K1_HALF_ORDER
/// - Return true if s <= half_order
pub fn has_low_s(sig: &EcdsaSignature) -> bool {
    todo!("Check if s <= n/2")
}

/// Exercise 3: Normalize a signature to low-s form.
///
/// If s > n/2, replace s with n - s. This ensures canonical representation.
///
/// Hints:
/// - If has_low_s, return the signature unchanged
/// - Otherwise, compute n - s and use that as the new s
pub fn normalize_to_low_s(sig: &EcdsaSignature) -> EcdsaSignature {
    todo!("Normalize signature to low-s form")
}

/// Exercise 4: Demonstrate the malleability attack.
///
/// Returns (original_sig, malleated_sig) where both are "valid" for the same
/// message. The malleated version has s flipped to n-s.
///
/// In a real blockchain, this would create two different transaction IDs
/// for the same logical transaction.
pub fn demonstrate_malleability_attack(sig: &EcdsaSignature) -> (EcdsaSignature, EcdsaSignature) {
    todo!("Demonstrate ECDSA signature malleability")
}

/// Exercise 5: Validate a signature against malleability rules.
///
/// Reject signatures where s > n/2 (high-s malleability).
/// This is what Bitcoin's BIP-62 and Ethereum require.
///
/// Returns true only if the signature has low-s.
pub fn validate_signature_not_malleable(sig: &EcdsaSignature) -> bool {
    todo!("Validate that a signature is not malleable (low-s)")
}

/// Exercise 6: Demonstrate that normalizing prevents the attack.
///
/// Show that both the original and malleated signature normalize to the
/// same canonical form.
pub fn demonstrate_normalization_defense(
    sig: &EcdsaSignature,
) -> (EcdsaSignature, EcdsaSignature) {
    todo!("Show that normalization makes both forms identical")
}

/// Exercise 7: Compare two signature encodings.
///
/// Check if two signatures are semantically equivalent (same r, and s or n-s).
///
/// Hints:
/// - Check r values are equal
/// - Check if s1 == s2 OR s1 == n - s2
pub fn signatures_equivalent(sig1: &EcdsaSignature, sig2: &EcdsaSignature) -> bool {
    todo!("Check if two signatures are semantically equivalent")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_signature() -> EcdsaSignature {
        // A signature with a "high" s value (s > n/2)
        let r = [0x01u8; 32];
        let s = [
            0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        EcdsaSignature::new(r, s)
    }

    fn make_low_s_signature() -> EcdsaSignature {
        // A signature with a "low" s value (s <= n/2)
        let r = [0x02u8; 32];
        let s = [
            0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        EcdsaSignature::new(r, s)
    }

    #[test]
    fn test_malleable_signature_different() {
        let sig = make_test_signature();
        let malleable = create_malleable_signature(&sig);
        assert_ne!(sig, malleable, "Malleable signature should be different");
        assert_eq!(sig.r, malleable.r, "r should be the same");
        assert_ne!(sig.s, malleable.s, "s should be different");
    }

    #[test]
    fn test_low_s_check() {
        let high_s_sig = make_test_signature();
        let low_s_sig = make_low_s_signature();
        assert!(!has_low_s(&high_s_sig), "High-s should not be low-s");
        assert!(has_low_s(&low_s_sig), "Low-s should be low-s");
    }

    #[test]
    fn test_normalize_high_s() {
        let sig = make_test_signature();
        let normalized = normalize_to_low_s(&sig);
        assert!(has_low_s(&normalized), "Normalized should have low-s");
    }

    #[test]
    fn test_normalize_low_s_unchanged() {
        let sig = make_low_s_signature();
        let normalized = normalize_to_low_s(&sig);
        assert_eq!(sig, normalized, "Already-low-s should be unchanged");
    }

    #[test]
    fn test_malleability_attack_demo() {
        let sig = make_test_signature();
        let (original, malleated) = demonstrate_malleability_attack(&sig);
        assert_ne!(original, malleated);
        assert_eq!(original.r, malleated.r);
    }

    #[test]
    fn test_validate_rejects_high_s() {
        let high_s_sig = make_test_signature();
        let low_s_sig = make_low_s_signature();
        assert!(!validate_signature_not_malleable(&high_s_sig));
        assert!(validate_signature_not_malleable(&low_s_sig));
    }

    #[test]
    fn test_normalization_defense() {
        let sig = make_test_signature();
        let malleable = create_malleable_signature(&sig);
        let norm1 = normalize_to_low_s(&sig);
        let norm2 = normalize_to_low_s(&malleable);
        assert_eq!(norm1, norm2, "Both should normalize to the same form");
    }

    #[test]
    fn test_signatures_equivalent() {
        let sig = make_test_signature();
        let malleable = create_malleable_signature(&sig);
        assert!(signatures_equivalent(&sig, &malleable));
    }
}
