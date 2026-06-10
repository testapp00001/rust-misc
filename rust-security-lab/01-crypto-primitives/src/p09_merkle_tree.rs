//! # Lesson 09: Merkle Trees
//!
//! ## What is a Merkle Tree?
//!
//! A Merkle tree is a binary tree of hashes where:
//! - Each **leaf** is the hash of a data block
//! - Each **internal node** is the hash of its two children's hashes concatenated
//! - The **root** is a single hash that commits to ALL the data
//!
//! ```text
//!              Root = H(H(AB) || H(CD))
//!             /                        \
//!       H(AB) = H(H(A)||H(B))    H(CD) = H(H(C)||H(D))
//!       /          \               /           \
//!    H(A)         H(B)         H(C)          H(D)
//!     |            |            |              |
//!   Data A      Data B       Data C         Data D
//! ```
//!
//! ## Why Merkle Trees?
//!
//! **Efficient verification**: To prove that Data B is in the tree, you only need:
//! - H(A), H(CD) (the "authentication path" or "proof")
//! - H(B) = hash of the data being verified
//!
//! You DON'T need all the data — just O(log n) hashes.
//!
//! ## Applications
//!
//! - **Git**: Every commit includes a Merkle root of all files
//! - **Bitcoin/Ethereum**: Transaction verification in blocks
//! - **IPFS**: Content-addressed storage
//! - **Certificate Transparency**: Verifying certificate inclusion
//! - **Distributed systems**: Verifying data consistency across nodes
//!
//! ## Key Properties
//!
//! 1. **Tamper evidence**: Changing any data block changes the root
//! 2. **Efficient proofs**: O(log n) proof size, O(log n) verification time
//! 3. **Incremental updates**: Only rehash the path from changed leaf to root
//!
//! ## Attack Scenario
//!
//! An attacker tries to swap Data B with malicious data.
//! The Merkle root changes, and the authentication path proof fails to verify
//! against the known-good root.

use ring::digest;

/// A node in the Merkle tree.
#[derive(Debug, Clone, PartialEq)]
pub enum MerkleNode {
    /// Leaf node: hash of a data block
    Leaf(Vec<u8>),
    /// Internal node: hash of left and right children
    Internal {
        hash: Vec<u8>,
        left: Box<MerkleNode>,
        right: Box<MerkleNode>,
    },
}

/// A Merkle proof (authentication path) for verifying a leaf.
#[derive(Debug, Clone)]
pub struct MerkleProof {
    /// The index of the leaf (0-based, left to right)
    pub leaf_index: usize,
    /// Sibling hashes along the path from leaf to root
    /// proof[0] = sibling at leaf level, proof[1] = sibling at next level, etc.
    pub siblings: Vec<Vec<u8>>,
}

/// Exercise 1: Compute SHA-256 hash of data.
///
/// Hints:
/// - Use `ring::digest::digest(&digest::SHA256, data)`
/// - Return the hash bytes as Vec<u8>
fn hash_data(data: &[u8]) -> Vec<u8> {
    todo!("Compute SHA-256 hash of data")
}

/// Exercise 2: Compute the hash of two child hashes concatenated.
///
/// This is how internal Merkle nodes are computed:
/// `parent_hash = SHA256(left_hash || right_hash)`
///
/// Hints:
/// - Concatenate left and right hash bytes
/// - Hash the concatenation
fn hash_children(left: &[u8], right: &[u8]) -> Vec<u8> {
    todo!("Compute parent hash from two children")
}

/// Exercise 3: Build a Merkle tree from data blocks.
///
/// Returns the root node of the tree.
///
/// Rules:
/// - If there's only one block, return a Leaf node
/// - If the number of blocks is odd, duplicate the last block
/// - Split blocks into left and right halves, recurse
///
/// Hints:
/// - Use `hash_data()` for leaf nodes
/// - Use `hash_children()` for internal nodes
/// - Recursively build left and right subtrees
pub fn build_merkle_tree(data_blocks: &[&[u8]]) -> MerkleNode {
    todo!("Build a Merkle tree from data blocks")
}

/// Exercise 4: Get the root hash of a Merkle tree.
///
/// Hints:
/// - For a Leaf node, return its hash
/// - For an Internal node, return its hash field
pub fn root_hash(tree: &MerkleNode) -> &[u8] {
    todo!("Get the root hash of a Merkle tree")
}

/// Exercise 5: Generate a Merkle proof for a leaf at the given index.
///
/// Returns the authentication path (sibling hashes) from leaf to root.
///
/// Hints:
/// - Walk down the tree to find the leaf at the given index
/// - At each level, record the sibling hash
/// - For a tree of height h, the proof has h sibling hashes
pub fn generate_proof(tree: &MerkleNode, leaf_index: usize) -> Option<MerkleProof> {
    todo!("Generate a Merkle proof for a leaf")
}

/// Exercise 6: Verify a Merkle proof.
///
/// Given a leaf hash, a proof, and the expected root hash,
/// verify that the leaf is indeed part of the tree.
///
/// Hints:
/// - Start with the leaf hash
/// - At each level, combine with the sibling hash
/// - The direction (left/right) depends on whether the current index is even/odd
/// - After processing all siblings, the result should equal the root hash
pub fn verify_proof(leaf_hash: &[u8], proof: &MerkleProof, root: &[u8]) -> bool {
    todo!("Verify a Merkle proof against the root hash")
}

/// Exercise 7: Get all leaf hashes from the tree (in order).
///
/// Hints:
/// - For a Leaf node, return a vec containing its hash
/// - For an Internal node, recursively get leaves from left and right
pub fn get_leaf_hashes(tree: &MerkleNode) -> Vec<Vec<u8>> {
    todo!("Collect all leaf hashes from the tree")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tree_single_block() {
        let tree = build_merkle_tree(&[b"hello"]);
        let root = root_hash(&tree);
        let expected = hash_data(b"hello");
        assert_eq!(root, &expected);
    }

    #[test]
    fn test_build_tree_two_blocks() {
        let tree = build_merkle_tree(&[b"a", b"b"]);
        let root = root_hash(&tree);
        assert_eq!(root.len(), 32, "Root should be a 32-byte SHA-256 hash");

        // Root should be hash(hash("a") || hash("b"))
        let ha = hash_data(b"a");
        let hb = hash_data(b"b");
        let expected = hash_children(&ha, &hb);
        assert_eq!(root, &expected);
    }

    #[test]
    fn test_build_tree_four_blocks() {
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let tree = build_merkle_tree(&blocks);
        let root = root_hash(&tree);
        assert_eq!(root.len(), 32);
    }

    #[test]
    fn test_odd_number_of_blocks() {
        // With 3 blocks, the 3rd should be duplicated to make 4
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c"];
        let tree = build_merkle_tree(&blocks);
        let root = root_hash(&tree);
        assert_eq!(root.len(), 32);
    }

    #[test]
    fn test_tampered_data_changes_root() {
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let tree1 = build_merkle_tree(&blocks);

        let tampered: Vec<&[u8]> = vec![b"a", b"b", b"X", b"d"];
        let tree2 = build_merkle_tree(&tampered);

        assert_ne!(
            root_hash(&tree1),
            root_hash(&tree2),
            "Tampering should change the root"
        );
    }

    #[test]
    fn test_generate_and_verify_proof() {
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let tree = build_merkle_tree(&blocks);
        let root = root_hash(&tree).to_vec();

        // Generate proof for leaf 0 ("a")
        let proof = generate_proof(&tree, 0).expect("Proof should exist for leaf 0");
        let leaf_hash = hash_data(b"a");
        assert!(verify_proof(&leaf_hash, &proof, &root));

        // Generate proof for leaf 2 ("c")
        let proof = generate_proof(&tree, 2).expect("Proof should exist for leaf 2");
        let leaf_hash = hash_data(b"c");
        assert!(verify_proof(&leaf_hash, &proof, &root));
    }

    #[test]
    fn test_tampered_leaf_fails_proof() {
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let tree = build_merkle_tree(&blocks);
        let root = root_hash(&tree).to_vec();

        // Get proof for leaf 0
        let proof = generate_proof(&tree, 0).unwrap();

        // Verify with tampered leaf hash
        let tampered_hash = hash_data(b"EVIL");
        assert!(
            !verify_proof(&tampered_hash, &proof, &root),
            "Tampered leaf should fail verification"
        );
    }

    #[test]
    fn test_empty_tree() {
        let tree = build_merkle_tree(&[]);
        // Empty tree should still produce a root (hash of empty)
        let root = root_hash(&tree);
        assert_eq!(root.len(), 32);
    }

    #[test]
    fn test_get_leaf_hashes() {
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let tree = build_merkle_tree(&blocks);
        let leaves = get_leaf_hashes(&tree);
        assert_eq!(leaves.len(), 4);
        assert_eq!(leaves[0], hash_data(b"a"));
        assert_eq!(leaves[1], hash_data(b"b"));
        assert_eq!(leaves[2], hash_data(b"c"));
        assert_eq!(leaves[3], hash_data(b"d"));
    }

    #[test]
    fn test_proof_size_is_logarithmic() {
        // For 8 blocks, proof should have 3 siblings (log2(8) = 3)
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d", b"e", b"f", b"g", b"h"];
        let tree = build_merkle_tree(&blocks);
        let proof = generate_proof(&tree, 0).unwrap();
        assert_eq!(proof.siblings.len(), 3, "8-block tree should have 3-level proof");
    }
}
