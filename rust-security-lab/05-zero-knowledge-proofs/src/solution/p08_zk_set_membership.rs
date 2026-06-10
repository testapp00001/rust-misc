//! # Lesson 08: ZK Set Membership — Merkle Tree + ZK Proof (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};

/// Compute SHA-256 hash.
pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Build a Merkle tree from leaf values.
pub fn build_merkle_tree(leaves: &[&[u8]]) -> Vec<Vec<Vec<u8>>> {
    if leaves.is_empty() {
        return vec![];
    }

    let mut tree: Vec<Vec<Vec<u8>>> = Vec::new();

    // Level 0: leaf hashes
    let mut current_level: Vec<Vec<u8>> = leaves.iter().map(|l| sha256(l)).collect();
    tree.push(current_level.clone());

    // Build up the tree
    while current_level.len() > 1 {
        // If odd number, duplicate last element
        if current_level.len() % 2 != 0 {
            current_level.push(current_level.last().unwrap().clone());
        }

        let mut next_level = Vec::new();
        for pair in current_level.chunks(2) {
            let mut combined = pair[0].clone();
            combined.extend_from_slice(&pair[1]);
            next_level.push(sha256(&combined));
        }

        tree.push(next_level.clone());
        current_level = next_level;
    }

    tree
}

/// Get the Merkle root from a tree.
pub fn merkle_root(tree: &[Vec<Vec<u8>>]) -> Vec<u8> {
    tree.last().unwrap()[0].clone()
}

/// Generate a Merkle proof for a leaf at given index.
pub fn generate_merkle_proof(tree: &[Vec<Vec<u8>>], leaf_index: usize) -> Vec<(Vec<u8>, bool)> {
    let mut proof = Vec::new();
    let mut index = leaf_index;

    for level in tree.iter().take(tree.len() - 1) {
        let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
        let is_right = sibling_index > index;

        if sibling_index < level.len() {
            proof.push((level[sibling_index].clone(), is_right));
        } else {
            // Sibling is the duplicated node
            proof.push((level[index].clone(), is_right));
        }

        index /= 2;
    }

    proof
}

/// Verify a Merkle proof.
pub fn verify_merkle_proof(
    leaf_hash: &[u8],
    proof: &[(Vec<u8>, bool)],
    expected_root: &[u8],
) -> bool {
    let mut current = leaf_hash.to_vec();

    for (sibling, is_right) in proof {
        let combined = if *is_right {
            let mut c = current.clone();
            c.extend_from_slice(sibling);
            c
        } else {
            let mut c = sibling.clone();
            c.extend_from_slice(&current);
            c
        };
        current = sha256(&combined);
    }

    current == expected_root
}

/// Prove set membership (non-ZK — leaks position).
pub fn prove_membership(
    value: &[u8],
    tree: &[Vec<Vec<u8>>],
    leaves: &[&[u8]],
) -> (Vec<u8>, Vec<(Vec<u8>, bool)>, usize) {
    let leaf_hash = sha256(value);

    // Find the leaf index
    let leaf_hashes: Vec<Vec<u8>> = leaves.iter().map(|l| sha256(l)).collect();
    let leaf_index = leaf_hashes.iter().position(|h| *h == leaf_hash).unwrap_or(leaves.len());

    let proof = if leaf_index < tree[0].len() {
        generate_merkle_proof(tree, leaf_index)
    } else {
        Vec::new()
    };

    (leaf_hash, proof, leaf_index)
}

/// Verify set membership proof.
pub fn verify_membership(
    leaf_hash: &[u8],
    proof: &[(Vec<u8>, bool)],
    root: &[u8],
) -> bool {
    verify_merkle_proof(leaf_hash, proof, root)
}

/// Demonstrate position leakage in non-ZK proof.
pub fn demonstrate_position_leakage(
    value: &[u8],
    tree: &[Vec<Vec<u8>>],
    leaves: &[&[u8]],
) -> usize {
    let (_, _, idx) = prove_membership(value, tree, leaves);
    idx
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
        assert_eq!(tree.last().unwrap().len(), 1);
    }

    #[test]
    fn test_merkle_root_non_empty() {
        let leaves = test_leaves();
        let tree = build_merkle_tree(&leaves);
        let root = merkle_root(&tree);
        assert_eq!(root.len(), 32);
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
