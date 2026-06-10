//! # Lesson 04: SPHINCS+ — Hash-Based Digital Signatures
//!
//! ## What is SPHINCS+?
//!
//! SPHINCS+ (SLH-DSA, FIPS 205) is a stateless hash-based digital signature scheme.
//! It was standardized by NIST as a conservative backup to ML-DSA (Dilithium).
//!
//! ## Why Hash-Based Signatures?
//!
//! Hash-based signatures derive their security ONLY from the security of the
//! underlying hash function. No number-theoretic assumptions. If SHA-256 is
//! secure, SPHINCS+ is secure — even against quantum computers (Grover's only
//! provides a quadratic speedup, which is accounted for in parameter selection).
//!
//! ## SPHINCS+ Architecture
//!
//! ```
//! SPHINCS+
//! ├── WOTS+ (Winternitz One-Time Signature)
//! │   └── Signs ONE message with one key
//! ├── Hypertree
//! │   └── Tree of trees (d layers of XMSS trees)
//! ├── FORS (Forest of Random Subsets)
//! │   └── Few-time signature for the message digest
//! └── Address scheme
//!     └── Prevents key reuse across tree positions
//! ```
//!
//! ## Signature Sizes
//!
//! | Parameter Set | Security | Sig Size | pk Size |
//! |---------------|----------|----------|---------|
//! | SPHINCS+-128s | 128-bit | 7,856 B | 32 B |
//! | SPHINCS+-192s | 192-bit | 16,224 B | 48 B |
//! | SPHINCS+-256s | 256-bit | 29,792 B | 64 B |
//!
//! The big downside: signatures are HUGE compared to ECDSA (64 bytes) or
//! ML-DSA (~2,420 bytes). But the security assumptions are minimal.
//!
//! ## Attack Demo: One-Time Signature Key Reuse
//!
//! WOTS+ keys must NEVER be reused. If you sign two different messages with the
//! same WOTS+ key, an attacker can forge signatures for other messages.

use sha2::{Digest, Sha256};
use rand::Rng;

/// A simplified WOTS+ key pair for one-time signing.
#[derive(Debug, Clone)]
pub struct WotsKeyPair {
    pub secret_chains: Vec<Vec<u8>>,
    pub public_chains: Vec<Vec<u8>>,
    pub w: usize,
    pub chain_len: usize,
}

/// A WOTS+ signature: chain values at specific positions.
#[derive(Debug, Clone)]
pub struct WotsSignature {
    pub chain_values: Vec<Vec<u8>>,
}

/// Exercise 1: Compute a Winternitz chain (iterated hash).
///
/// Starting from `seed`, apply SHA-256 `steps` times.
/// chain(seed, 0) = seed
/// chain(seed, n) = SHA256(chain(seed, n-1))
///
/// This is the core building block of WOTS+.
pub fn winternitz_chain(seed: &[u8], steps: usize) -> Vec<u8> {
    todo!("Compute iterated hash chain")
}

/// Exercise 2: Generate a WOTS+ keypair.
///
/// Parameters:
/// - `n`: security parameter (hash output length, e.g., 32 for SHA-256)
/// - `w`: Winternitz parameter (controls speed vs sig size tradeoff)
///
/// Key generation:
/// 1. Compute chain_len = floor(log_w(256)) + 1 (number of chain positions)
/// 2. Total chains needed: n + ceil(n * log2(w) / 8) simplified to n for w=16
/// 3. For each chain i: generate random secret[i], compute public[i] = chain(secret[i], chain_len - 1)
///
/// For simplicity with w=16, chain_len=2, total chains = 2*n.
pub fn wots_keygen(n: usize, w: usize) -> WotsKeyPair {
    todo!("Generate WOTS+ keypair")
}

/// Exercise 3: Sign a message with WOTS+ (simplified).
///
/// To sign a message hash `msg_hash` (n bytes):
/// 1. Convert each byte to a base-w digit (0..w-1)
/// 2. For chain i, signature[i] = chain(secret[i], digit_i)
/// 3. Compute checksum over all digits, sign the checksum chains too
///
/// Return the signature containing chain values.
pub fn wots_sign(kp: &WotsKeyPair, msg_hash: &[u8]) -> WotsSignature {
    todo!("Sign with WOTS+")
}

/// Exercise 4: Verify a WOTS+ signature.
///
/// To verify:
/// 1. Compute the message digits (same as signing)
/// 2. For each chain i, continue the chain from sig.chain_values[i]
///   for (chain_len - 1 - digit_i) more steps
/// 3. The result should equal public_chains[i]
/// 4. Verify checksum chains too
///
/// Return true if all chains verify correctly.
pub fn wots_verify(kp: &WotsKeyPair, msg_hash: &[u8], sig: &WotsSignature) -> bool {
    todo!("Verify WOTS+ signature")
}

/// Exercise 5: Demonstrate WOTS+ key reuse vulnerability.
///
/// If we sign two different messages with the same WOTS+ key, an attacker
/// learns chain values at two different positions for each chain.
/// Given digits d1 and d2 (d1 < d2) for chain i:
/// - From sig1, attacker has chain(secret[i], d1)
/// - From sig2, attacker has chain(secret[i], d2)
/// - Attacker can compute chain(chain(secret[i], d1), d2 - d1) = chain(secret[i], d2)
///   and also chain(chain(secret[i], d1), x) for any x < d2 - d1
///
/// This function returns true if two signatures reveal enough to forge.
/// Simply: return true if the two message hashes differ (key reuse detected).
pub fn detect_key_reuse(hash1: &[u8], hash2: &[u8]) -> bool {
    todo!("Detect WOTS+ key reuse vulnerability")
}

/// Exercise 6: Compute the approximate SPHINCS+ signature size.
///
/// SPHINCS+ signature size ≈ (hypertree_height * wots_sig_size) + fors_sig_size + randomness
///
/// Simplified formula:
/// - wots_sig_size = 2 * n * chain_len (with w=16, chain_len=2)
/// - hypertree_layers = d, each layer has height h/d
/// - fors_trees = k, each with t leaves
/// - sig_size ≈ d * (h/d) * wots_sig_size + k * (n + fors_height) + n
///
/// For the standard SPHINCS+-128s: n=16, h=63, d=7, w=16, k=14, t=12
/// Simplified: sig_size ≈ h * wots_sig_size + k * n + n + k
pub fn sphincs_sig_size_estimate(n: usize, h: usize, d: usize, w: usize, k: usize) -> usize {
    todo!("Estimate SPHINCS+ signature size")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winternitz_chain_zero() {
        let seed = b"test_seed";
        let result = winternitz_chain(seed, 0);
        assert_eq!(result, seed);
    }

    #[test]
    fn test_winternitz_chain_one() {
        let seed = b"test_seed";
        let result = winternitz_chain(seed, 1);
        let expected = Sha256::digest(seed).to_vec();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_winternitz_chain_length() {
        let seed = b"seed";
        let result = winternitz_chain(seed, 5);
        assert_eq!(result.len(), 32); // SHA-256 output
    }

    #[test]
    fn test_wots_keygen_dimensions() {
        let kp = wots_keygen(16, 16);
        assert_eq!(kp.secret_chains.len(), 32); // 2*n for w=16
        assert_eq!(kp.public_chains.len(), 32);
        assert_eq!(kp.w, 16);
    }

    #[test]
    fn test_wots_sign_verify_roundtrip() {
        let kp = wots_keygen(16, 16);
        let msg_hash = Sha256::digest(b"hello world").to_vec();
        let sig = wots_sign(&kp, &msg_hash);
        assert!(wots_verify(&kp, &msg_hash, &sig));
    }

    #[test]
    fn test_wots_verify_wrong_message() {
        let kp = wots_keygen(16, 16);
        let msg_hash = Sha256::digest(b"hello").to_vec();
        let sig = wots_sign(&kp, &msg_hash);
        let wrong_hash = Sha256::digest(b"world").to_vec();
        assert!(!wots_verify(&kp, &wrong_hash, &sig));
    }

    #[test]
    fn test_detect_key_reuse() {
        let hash1 = Sha256::digest(b"msg1").to_vec();
        let hash2 = Sha256::digest(b"msg2").to_vec();
        assert!(detect_key_reuse(&hash1, &hash2));
    }

    #[test]
    fn test_detect_no_reuse() {
        let hash1 = Sha256::digest(b"same").to_vec();
        assert!(!detect_key_reuse(&hash1, &hash1));
    }

    #[test]
    fn test_sphincs_sig_size_estimate() {
        // SPHINCS+-128s: n=16, h=63, d=7, w=16, k=14
        let size = sphincs_sig_size_estimate(16, 63, 7, 16, 14);
        // Should be on the order of thousands of bytes
        assert!(size > 1000);
    }

    #[test]
    fn test_wots_signature_chain_value_lengths() {
        let kp = wots_keygen(16, 16);
        let msg_hash = Sha256::digest(b"test").to_vec();
        let sig = wots_sign(&kp, &msg_hash);
        // All chain values should be 32 bytes (SHA-256)
        for cv in &sig.chain_values {
            assert_eq!(cv.len(), 32);
        }
    }
}
