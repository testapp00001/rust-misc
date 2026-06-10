//! # Lesson 09: Hash-Based Signature Concepts — Lamport, Merkle, WOTS+
//!
//! ## The Hash-Based Signature Family
//!
//! Hash-based signatures are the most conservative post-quantum signatures.
//! Their security relies ONLY on the hash function — no algebraic structure,
//! no number theory, no lattices. If SHA-256 is secure, these signatures are secure.
//!
//! ```
//! Evolution:
//! Lamport (1979) → One-time signature, signs ONE bit
//!   ↓
//! WOTS (Winternitz) → Signs ONE message, more efficient
//!   ↓
//! Merkle Trees → Many-time signatures using a tree of one-time keys
//!   ↓
//! XMSS / LMS → Standardized stateful hash-based signatures
//!   ↓
//! SPHINCS+ → Stateless (no need to track which keys were used)
//! ```
//!
//! ## Lamport Signature
//!
//! ```
//! Key Generation:
//!   For each bit position i (0..255):
//!     secret_0[i] = random 256-bit value
//!     secret_1[i] = random 256-bit value
//!     public_0[i] = Hash(secret_0[i])
//!     public_1[i] = Hash(secret_1[i])
//!
//! Signing message hash m (256 bits):
//!   For each bit i of m:
//!     if m[i] == 0: reveal secret_0[i]
//!     if m[i] == 1: reveal secret_1[i]
//!
//! Verification:
//!   For each bit i of m:
//!     Hash(signature[i]) must equal public_{m[i]}[i]
//! ```
//!
//! ## Attack: Lamport Key Reuse
//!
//! Lamport keys can only sign ONE message. If you sign two different messages,
//! an attacker learns secret values for both 0 and 1 positions, enabling forgery
//! of ANY message.

use sha2::{Digest, Sha256};
use rand::Rng;

/// A Lamport keypair.
#[derive(Debug, Clone)]
pub struct LamportKeyPair {
    /// secret_0[i] is the secret for bit i = 0
    pub secret_0: Vec<Vec<u8>>,
    /// secret_1[i] is the secret for bit i = 1
    pub secret_1: Vec<Vec<u8>>,
    /// public_0[i] = Hash(secret_0[i])
    pub public_0: Vec<Vec<u8>>,
    /// public_1[i] = Hash(secret_1[i])
    pub public_1: Vec<Vec<u8>>,
}

/// A Lamport signature: 256 hash values, one per bit of the message hash.
#[derive(Debug, Clone)]
pub struct LamportSignature {
    pub values: Vec<Vec<u8>>,
}

/// A Merkle tree node.
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: Vec<u8>,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

/// Exercise 1: Generate a Lamport keypair.
///
/// For `n` bits (use 256 for SHA-256):
/// 1. Generate 2*n random secret values (32 bytes each)
/// 2. Hash each secret to get the public values
///
/// secret_0[i] and secret_1[i] are for bit position i.
pub fn lamport_keygen(n_bits: usize) -> LamportKeyPair {
    todo!("Generate Lamport keypair")
}

/// Exercise 2: Sign a message hash with Lamport.
///
/// The message hash is a 256-bit (32-byte) value.
/// For each bit position i:
/// - If bit i is 0, include secret_0[i] in the signature
/// - If bit i is 1, include secret_1[i] in the signature
///
/// Return a LamportSignature with 256 values.
pub fn lamport_sign(kp: &LamportKeyPair, msg_hash: &[u8]) -> LamportSignature {
    todo!("Sign message hash with Lamport")
}

/// Exercise 3: Verify a Lamport signature.
///
/// For each bit position i:
/// - Hash signature.values[i]
/// - If bit i is 0, compare with public_0[i]
/// - If bit i is 1, compare with public_1[i]
///
/// Return true if all comparisons match.
pub fn lamport_verify(
    kp: &LamportKeyPair,
    msg_hash: &[u8],
    sig: &LamportSignature,
) -> bool {
    todo!("Verify Lamport signature")
}

/// Exercise 4: Demonstrate Lamport forgery after key reuse.
///
/// Given two signatures for different messages signed with the same key,
/// compute a forged signature for a target message.
///
/// For each bit position i:
/// - If sig1 and sig2 reveal different values, the attacker has both secret_0[i]
///   and secret_1[i]
/// - Use the appropriate secret to forge the target bit
///
/// Return Some(forged_signature) if forgery is possible, None if the two
/// messages have the same hash (no forgery needed).
pub fn lamport_forge_from_reuse(
    msg1_hash: &[u8],
    sig1: &LamportSignature,
    msg2_hash: &[u8],
    sig2: &LamportSignature,
    target_hash: &[u8],
) -> Option<LamportSignature> {
    todo!("Forge Lamport signature from key reuse")
}

/// Exercise 5: Compute a Merkle tree root from leaf hashes.
///
/// A Merkle tree combines pairs of hashes up to a single root:
/// ```
///         root
///        /    \
///      h01    h23
///     /  \   /  \
///   h0  h1 h2  h3
/// ```
///
/// h01 = Hash(h0 || h1), h23 = Hash(h2 || h3)
/// root = Hash(h01 || h23)
///
/// If the number of leaves is odd, duplicate the last leaf.
/// Return the root hash.
pub fn merkle_root(leaves: &[Vec<u8>]) -> Vec<u8> {
    todo!("Compute Merkle tree root hash")
}

/// Exercise 6: Compute the size overhead of Lamport vs ECDSA.
///
/// ECDSA signature (P-256): 64 bytes
/// Lamport signature (SHA-256): 256 * 32 = 8192 bytes
/// Public key: 2 * 256 * 32 = 16384 bytes
///
/// Return (lamport_sig_bytes, lamport_pk_bytes, ecdsa_sig_bytes).
pub fn lamport_size_overhead() -> (usize, usize, usize) {
    todo!("Compute Lamport signature size overhead")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(data: &[u8]) -> Vec<u8> {
        Sha256::digest(data).to_vec()
    }

    #[test]
    fn test_lamport_keygen_dimensions() {
        let kp = lamport_keygen(256);
        assert_eq!(kp.secret_0.len(), 256);
        assert_eq!(kp.secret_1.len(), 256);
        assert_eq!(kp.public_0.len(), 256);
        assert_eq!(kp.public_1.len(), 256);
        // Each value should be 32 bytes (SHA-256)
        assert_eq!(kp.secret_0[0].len(), 32);
        assert_eq!(kp.public_0[0].len(), 32);
    }

    #[test]
    fn test_lamport_keygen_random() {
        let kp = lamport_keygen(256);
        // Secrets should be different from each other
        assert_ne!(kp.secret_0[0], kp.secret_1[0]);
    }

    #[test]
    fn test_lamport_sign_verify_roundtrip() {
        let kp = lamport_keygen(256);
        let msg_hash = hash(b"hello world");
        let sig = lamport_sign(&kp, &msg_hash);
        assert!(lamport_verify(&kp, &msg_hash, &sig));
    }

    #[test]
    fn test_lamport_verify_wrong_message() {
        let kp = lamport_keygen(256);
        let msg_hash = hash(b"hello");
        let sig = lamport_sign(&kp, &msg_hash);
        let wrong_hash = hash(b"world");
        assert!(!lamport_verify(&kp, &wrong_hash, &sig));
    }

    #[test]
    fn test_lamport_signature_size() {
        let kp = lamport_keygen(256);
        let msg_hash = hash(b"test");
        let sig = lamport_sign(&kp, &msg_hash);
        assert_eq!(sig.values.len(), 256);
        for v in &sig.values {
            assert_eq!(v.len(), 32);
        }
    }

    #[test]
    fn test_lamport_forge_from_reuse() {
        let kp = lamport_keygen(256);
        let h1 = hash(b"message1");
        let h2 = hash(b"message2");
        let target = hash(b"target");

        let sig1 = lamport_sign(&kp, &h1);
        let sig2 = lamport_sign(&kp, &h2);

        let forged = lamport_forge_from_reuse(&h1, &sig1, &h2, &sig2, &target);
        assert!(forged.is_some());
        // Forged signature should verify against the target hash
        assert!(lamport_verify(&kp, &target, &forged.unwrap()));
    }

    #[test]
    fn test_lamport_no_forge_same_message() {
        let kp = lamport_keygen(256);
        let h1 = hash(b"same");
        let sig1 = lamport_sign(&kp, &h1);
        let sig2 = lamport_sign(&kp, &h1);
        let forged = lamport_forge_from_reuse(&h1, &sig1, &h1, &sig2, &h1);
        assert!(forged.is_none());
    }

    #[test]
    fn test_merkle_root_two_leaves() {
        let h0 = hash(b"leaf0");
        let h1 = hash(b"leaf1");
        let root = merkle_root(&[h0.clone(), h1.clone()]);
        let expected = hash(&[h0, h1].concat());
        assert_eq!(root, expected);
    }

    #[test]
    fn test_merkle_root_four_leaves() {
        let leaves: Vec<Vec<u8>> = (0..4).map(|i| hash(&[i])).collect();
        let root = merkle_root(&leaves);
        assert_eq!(root.len(), 32);
    }

    #[test]
    fn test_merkle_root_single_leaf() {
        let leaf = hash(b"only");
        let root = merkle_root(&[leaf.clone()]);
        assert_eq!(root, leaf);
    }

    #[test]
    fn test_lamport_size_overhead() {
        let (sig, pk, ecdsa) = lamport_size_overhead();
        assert_eq!(sig, 256 * 32);       // 8192 bytes
        assert_eq!(pk, 2 * 256 * 32);    // 16384 bytes
        assert_eq!(ecdsa, 64);            // 64 bytes
    }
}
