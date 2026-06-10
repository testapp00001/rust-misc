//! # Lesson 09: Hash-Based Signature Concepts — Lamport, Merkle, WOTS+ (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};
use rand::Rng;

/// A Lamport keypair.
#[derive(Debug, Clone)]
pub struct LamportKeyPair {
    pub secret_0: Vec<Vec<u8>>,
    pub secret_1: Vec<Vec<u8>>,
    pub public_0: Vec<Vec<u8>>,
    pub public_1: Vec<Vec<u8>>,
}

/// A Lamport signature.
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

/// Generate a Lamport keypair for n_bits.
pub fn lamport_keygen(n_bits: usize) -> LamportKeyPair {
    let mut rng = rand::thread_rng();

    let secret_0: Vec<Vec<u8>> = (0..n_bits).map(|_| (0..32u8).map(|_| rng.gen()).collect()).collect();
    let secret_1: Vec<Vec<u8>> = (0..n_bits).map(|_| (0..32u8).map(|_| rng.gen()).collect()).collect();

    let public_0: Vec<Vec<u8>> = secret_0.iter().map(|s| Sha256::digest(s).to_vec()).collect();
    let public_1: Vec<Vec<u8>> = secret_1.iter().map(|s| Sha256::digest(s).to_vec()).collect();

    LamportKeyPair { secret_0, secret_1, public_0, public_1 }
}

/// Get the i-th bit of a byte slice (big-endian bit ordering).
fn get_bit(data: &[u8], bit_index: usize) -> u8 {
    let byte_index = bit_index / 8;
    let bit_pos = 7 - (bit_index % 8); // MSB first
    if byte_index < data.len() {
        (data[byte_index] >> bit_pos) & 1
    } else {
        0
    }
}

/// Sign a message hash with Lamport.
pub fn lamport_sign(kp: &LamportKeyPair, msg_hash: &[u8]) -> LamportSignature {
    let n_bits = kp.secret_0.len();
    let values: Vec<Vec<u8>> = (0..n_bits)
        .map(|i| {
            if get_bit(msg_hash, i) == 0 {
                kp.secret_0[i].clone()
            } else {
                kp.secret_1[i].clone()
            }
        })
        .collect();

    LamportSignature { values }
}

/// Verify a Lamport signature.
pub fn lamport_verify(
    kp: &LamportKeyPair,
    msg_hash: &[u8],
    sig: &LamportSignature,
) -> bool {
    if sig.values.len() != kp.public_0.len() {
        return false;
    }

    for i in 0..sig.values.len() {
        let computed = Sha256::digest(&sig.values[i]).to_vec();
        let expected = if get_bit(msg_hash, i) == 0 {
            &kp.public_0[i]
        } else {
            &kp.public_1[i]
        };
        if computed != *expected {
            return false;
        }
    }

    true
}

/// Forge a Lamport signature from key reuse.
///
/// When two different messages are signed with the same key, the attacker
/// learns both secret values for positions where the message bits differ.
pub fn lamport_forge_from_reuse(
    msg1_hash: &[u8],
    sig1: &LamportSignature,
    msg2_hash: &[u8],
    sig2: &LamportSignature,
    target_hash: &[u8],
) -> Option<LamportSignature> {
    // Can't forge if messages are identical
    if msg1_hash == msg2_hash {
        return None;
    }

    let n_bits = sig1.values.len();
    let mut forged_values = Vec::with_capacity(n_bits);

    for i in 0..n_bits {
        let bit1 = get_bit(msg1_hash, i);
        let bit2 = get_bit(msg2_hash, i);
        let target_bit = get_bit(target_hash, i);

        if bit1 == bit2 {
            // Both signatures reveal the same secret for this position
            if target_bit == bit1 {
                // Target wants the same bit — we have it
                forged_values.push(sig1.values[i].clone());
            } else {
                // Target wants the opposite bit — we don't have it
                return None;
            }
        } else {
            // Messages differ at this position — we have both secrets
            if target_bit == bit1 {
                forged_values.push(sig1.values[i].clone());
            } else {
                forged_values.push(sig2.values[i].clone());
            }
        }
    }

    Some(LamportSignature { values: forged_values })
}

/// Compute Merkle tree root from leaf hashes.
pub fn merkle_root(leaves: &[Vec<u8>]) -> Vec<u8> {
    if leaves.is_empty() {
        return Sha256::digest(b"").to_vec();
    }

    if leaves.len() == 1 {
        return leaves[0].clone();
    }

    // Duplicate last leaf if odd number
    let mut current_level: Vec<Vec<u8>> = leaves.to_vec();
    if current_level.len() % 2 != 0 {
        let last = current_level.last().unwrap().clone();
        current_level.push(last);
    }

    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        for pair in current_level.chunks(2) {
            let mut combined = Vec::new();
            combined.extend_from_slice(&pair[0]);
            combined.extend_from_slice(&pair[1]);
            next_level.push(Sha256::digest(&combined).to_vec());
        }
        current_level = next_level;
    }

    current_level.into_iter().next().unwrap()
}

/// Compute Lamport size overhead: (sig_bytes, pk_bytes, ecdsa_sig_bytes).
pub fn lamport_size_overhead() -> (usize, usize, usize) {
    let sig_bytes = 256 * 32;      // 256 values * 32 bytes each
    let pk_bytes = 2 * 256 * 32;   // 2 sets of 256 values * 32 bytes
    let ecdsa_sig_bytes = 64;       // P-256 ECDSA signature
    (sig_bytes, pk_bytes, ecdsa_sig_bytes)
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
        assert_eq!(kp.secret_0[0].len(), 32);
        assert_eq!(kp.public_0[0].len(), 32);
    }

    #[test]
    fn test_lamport_keygen_random() {
        let kp = lamport_keygen(256);
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
        assert_eq!(sig, 256 * 32);
        assert_eq!(pk, 2 * 256 * 32);
        assert_eq!(ecdsa, 64);
    }
}
