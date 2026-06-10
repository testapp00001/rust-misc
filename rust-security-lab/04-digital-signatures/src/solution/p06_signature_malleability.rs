//! # Lesson 06: Signature Malleability Attack (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Curve order for secp256k1
const SECP256K1_ORDER: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
    0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
    0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
];

const SECP256K1_HALF_ORDER: [u8; 32] = [
    0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D,
    0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcdsaSignature {
    pub r: [u8; 32],
    pub s: [u8; 32],
}

impl EcdsaSignature {
    pub fn new(r: [u8; 32], s: [u8; 32]) -> Self {
        Self { r, s }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(&self.r);
        bytes.extend_from_slice(&self.s);
        bytes
    }

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

/// Compare two 256-bit big-endian unsigned integers.
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

/// Subtract two 256-bit unsigned integers: result = a - b.
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

/// Create the malleable counterpart: (r, n - s).
///
/// ATTACK EXPLANATION:
/// In ECDSA, both (r, s) and (r, n-s) verify against the same message and key.
/// An attacker can flip this bit to create a different but equally valid signature.
/// This changes the signature's byte representation, which changes any hash
/// that includes the signature (e.g., a transaction ID).
pub fn create_malleable_signature(sig: &EcdsaSignature) -> EcdsaSignature {
    let s_flipped = sub_256(&SECP256K1_ORDER, &sig.s);
    EcdsaSignature::new(sig.r, s_flipped)
}

/// Check if s <= n/2 (low-s form).
///
/// This is the canonical form. Bitcoin and Ethereum reject high-s signatures.
pub fn has_low_s(sig: &EcdsaSignature) -> bool {
    compare_256(&sig.s, &SECP256K1_HALF_ORDER) <= 0
}

/// Normalize a signature to low-s form.
///
/// If s > n/2, replace with n - s. This ensures exactly one valid encoding.
pub fn normalize_to_low_s(sig: &EcdsaSignature) -> EcdsaSignature {
    if has_low_s(sig) {
        sig.clone()
    } else {
        EcdsaSignature::new(sig.r, sub_256(&SECP256K1_ORDER, &sig.s))
    }
}

/// Demonstrate the malleability attack.
///
/// Returns (original, malleated) — both valid for the same message.
pub fn demonstrate_malleability_attack(sig: &EcdsaSignature) -> (EcdsaSignature, EcdsaSignature) {
    let malleated = create_malleable_signature(sig);
    (sig.clone(), malleated)
}

/// Validate that a signature is not malleable (has low-s).
///
/// This is the defense: reject any signature with high-s.
/// Bitcoin enforced this starting with BIP-62.
pub fn validate_signature_not_malleable(sig: &EcdsaSignature) -> bool {
    has_low_s(sig)
}

/// Demonstrate that normalization makes both forms identical.
pub fn demonstrate_normalization_defense(
    sig: &EcdsaSignature,
) -> (EcdsaSignature, EcdsaSignature) {
    let malleated = create_malleable_signature(sig);
    let norm1 = normalize_to_low_s(sig);
    let norm2 = normalize_to_low_s(&malleated);
    (norm1, norm2)
}

/// Check if two signatures are semantically equivalent.
///
/// Two signatures are equivalent if r values match and s values are
/// either equal or complementary (s1 == n - s2).
pub fn signatures_equivalent(sig1: &EcdsaSignature, sig2: &EcdsaSignature) -> bool {
    if sig1.r != sig2.r {
        return false;
    }
    if sig1.s == sig2.s {
        return true;
    }
    // Check if s1 == n - s2
    let s2_flipped = sub_256(&SECP256K1_ORDER, &sig2.s);
    sig1.s == s2_flipped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_signature() -> EcdsaSignature {
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
        assert_ne!(sig, malleable);
        assert_eq!(sig.r, malleable.r);
        assert_ne!(sig.s, malleable.s);
    }

    #[test]
    fn test_low_s_check() {
        let high_s_sig = make_test_signature();
        let low_s_sig = make_low_s_signature();
        assert!(!has_low_s(&high_s_sig));
        assert!(has_low_s(&low_s_sig));
    }

    #[test]
    fn test_normalize_high_s() {
        let sig = make_test_signature();
        let normalized = normalize_to_low_s(&sig);
        assert!(has_low_s(&normalized));
    }

    #[test]
    fn test_normalize_low_s_unchanged() {
        let sig = make_low_s_signature();
        let normalized = normalize_to_low_s(&sig);
        assert_eq!(sig, normalized);
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
        let (norm1, norm2) = demonstrate_normalization_defense(&sig);
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_signatures_equivalent() {
        let sig = make_test_signature();
        let malleable = create_malleable_signature(&sig);
        assert!(signatures_equivalent(&sig, &malleable));
    }
}
