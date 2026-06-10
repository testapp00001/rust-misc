//! # Lesson 08: ZK Set Membership — Merkle Tree + ZK Proof
//!
//! ## What is ZK Set Membership?
//!
//! Prove that a value is a member of a set, without revealing WHICH element it is.
//!
//! Classic use case: "I have a valid credential in this revocation list" without revealing
//! which credential is yours.
//!
//! ## How It Works
//!
//! 1. **Merkle Tree**: Build a tree of all set elements. The root commits to the entire set.
//! 2. **Membership Proof**: Show the Merkle path from your element to the root.
//! 3. **Zero-Knowledge**: Hide which leaf you're proving by randomizing the path.
//!
//! ## Merkle Tree Review
//!
//! ```text
//!        Root
//!       /    \
//!    H(AB)  H(CD)
//!    / \    / \
//!   A   B  C   D
//! ```
//!
//! To prove B is in the set: show {A, H(CD)} — the sibling hashes along the path.
//! Verifier recomputes root and checks it matches.
//!
//! ## ATTACK: Revealing the Path Leaks Position
//!
//! A standard Merkle proof reveals WHICH leaf you're proving (the path index).
//! In a ZK version, we must hide the path index.
//!
//! ## ZK Approach (Simplified)
//!
//! For teaching, we implement a simplified version where:
//! - We build a Merkle tree
//! - We prove membership by revealing the path
//! - We demonstrate how the path index leaks information
//! - We show the concept of hiding the index (in practice, use ZK-SNARKs)

use sha2::{Digest, Sha256};

/// Exercise 1: Compute SHA-256 hash.
///
/// Hints:
/// - Use Sha256 hasher
pub fn sha256(data: &[u8]) -> Vec<u8> {
    todo!("Compute SHA-256 hash")
}

/// Exercise 2: Build a Merkle tree from a list of leaf values.
///
/// Returns the tree as a vector of vectors (levels), where:
/// - tree[0] = leaf hashes
/// - tree[1] = parent hashes
/// - tree[last] = [root]
///
/// Hints:
/// - Hash each leaf value to get leaf hashes
/// - Pair up adjacent hashes and hash their concatenation
/// - If odd number of nodes, duplicate the last one
/// - Repeat until one node remains (the root)
pub fn build_merkle_tree(leaves: &[&[u8]]) -> Vec<Vec<Vec<u8>>> {
    todo!("Build a Merkle tree from leaf values")
}

/// Exercise 3: Get the Merkle root from a tree.
///
/// Hints:
/// - Root is tree.last().unwrap()[0]
pub fn merkle_root(tree: &[Vec<Vec<u8>>]) -> Vec<u8> {
    todo!("Get Merkle root from tree")
}

/// Exercise 4: Generate a Merkle proof (authentication path) for a leaf at given index.
///
/// Returns a vector of (sibling_hash, is_right_sibling) tuples.
/// - is_right_sibling = true means the sibling is on the right
///
/// Hints:
/// - For each level, get the sibling of the current node
/// - If index is even, sibling is index+1 (right); if odd, sibling is index-1 (left)
/// - Move up: index = index / 2
pub fn generate_merkle_proof(tree: &[Vec<Vec<u8>>], leaf_index: usize) -> Vec<(Vec<u8>, bool)> {
    todo!("Generate Merkle proof for a leaf")
}

/// Exercise 5: Verify a Merkle proof.
///
/// Starting from the leaf hash, apply the proof path to recompute the root.
///
/// Hints:
/// - Start with leaf_hash
/// - For each (sibling, is_right) in proof:
///   - If is_right: hash(current || sibling)
///   - If !is_right: hash(sibling || current)
/// - Final result should equal the root
pub fn verify_merkle_proof(
    leaf_hash: &[u8],
    proof: &[(Vec<u8>, bool)],
    expected_root: &[u8],
) -> bool {
    todo!("Verify a Merkle proof")
}

/// Exercise 6: Prove set membership (non-ZK).
///
/// Given a value and the tree, produce a proof that the value is in the set.
/// This is NOT zero-knowledge — the proof reveals the position.
///
/// Returns (leaf_hash, proof, leaf_index).
///
/// Hints:
/// - Find the leaf index by hashing each leaf and comparing
/// - Generate Merkle proof for that index
pub fn prove_membership(value: &[u8], tree: &[Vec<Vec<u8>>], leaves: &[&[u8]]) -> (Vec<u8>, Vec<(Vec<u8>, bool)>, usize) {
    todo!("Prove set membership (non-ZK)")
}

/// Exercise 7: Verify set membership proof.
///
/// Hints:
/// - Verify the Merkle proof against the root
pub fn verify_membership(
    leaf_hash: &[u8],
    proof: &[(Vec<u8>, bool)],
    root: &[u8],
) -> bool {
    todo!("Verify set membership proof")
}

/// Exercise 8: Demonstrate information leakage in non-ZK proof.
///
/// The leaf_index reveals WHERE in the set the value is.
/// If the set is ordered, this leaks information about the value.
///
/// Returns the leaked index.
///
/// Hints:
/// - Just return the leaf_index from prove_membership
pub fn demonstrate_position_leakage(
    value: &[u8],
    tree: &[Vec<Vec<u8>>],
    leaves: &[&[u8]],
) -> usize {
    todo!("Show that non-ZK proof leaks position")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_leaves() -> Vec<&'static [u8]> {
        vec![b"alice", b"bob", b"charlie", b"dave"]
    }

    #[test]
    fn test_merkle_tree_construction() {
        let leaves = test_leaves();
        let tree = build_merkle_tree(&leaves);
        assert!(!tree.is_empty());
        // Root should be a single hash
        assert_eq!(tree.last().unwrap().len(), 1);
    }

    #[test]
    fn test_merkle_root_non_empty() {
        let leaves = test_leaves();
        let tree = build_merkle_tree(&leaves);
        let root = merkle_root(&tree);
        assert_eq!(root.len(), 32); // SHA-256 output
    }

    #[test]
    fn test_merkle_proof_valid() {
        let leaves = test_leaves();
        let tree = build_merkle_tree(&leaves);
        let root = merkle_root(&tree);
        let leaf_hash = sha256(b"bob");
        let proof = generate_merkle_proof(&tree, 1);
        assert!(verify_merkle_proof(&leaf_hash, &proof, &root));
    }

    #[test]
    fn test_merkle_proof_wrong_leaf() {
        let leaves = test_leaves();
        let tree = build_merkle_tree(&leaves);
        let root = merkle_root(&tree);
        let wrong_hash = sha256(b"eve");
        let proof = generate_merkle_proof(&tree, 1);
        assert!(!verify_merkle_proof(&wrong_hash, &proof, &root));
    }

    #[test]
    fn test_set_membership_proof() {
        let leaves = test_leaves();
        let tree = build_merkle_tree(&leaves);
        let root = merkle_root(&tree);
        let (leaf_hash, proof, _idx) = prove_membership(b"charlie", &tree, &leaves);
        assert!(verify_membership(&leaf_hash, &proof, &root));
    }

    #[test]
    fn test_set_membership_non_member() {
        let leaves = test_leaves();
        let tree = build_merkle_tree(&leaves);
        let root = merkle_root(&tree);
        let (leaf_hash, proof, _idx) = prove_membership(b"eve", &tree, &leaves);
        // Eve is not in the set, proof should fail
        assert!(!verify_membership(&leaf_hash, &proof, &root));
    }

    #[test]
    fn test_position_leakage() {
        let leaves = test_leaves();
        let tree = build_merkle_tree(&leaves);
        let leaked = demonstrate_position_leakage(b"bob", &tree, &leaves);
        assert_eq!(leaked, 1, "Bob is at index 1 — position is leaked!");
    }

    #[test]
    fn test_all_members_verify() {
        let leaves = test_leaves();
        let tree = build_merkle_tree(&leaves);
        let root = merkle_root(&tree);
        for (i, leaf) in leaves.iter().enumerate() {
            let (leaf_hash, proof, idx) = prove_membership(leaf, &tree, &leaves);
            assert_eq!(idx, i);
            assert!(verify_membership(&leaf_hash, &proof, &root));
        }
    }
}
