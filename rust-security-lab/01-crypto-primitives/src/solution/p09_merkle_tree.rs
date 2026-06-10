//! # Lesson 09: Merkle Trees (Reference Solution)

use ring::digest;

/// A node in the Merkle tree.
#[derive(Debug, Clone, PartialEq)]
pub enum MerkleNode {
    Leaf(Vec<u8>),
    Internal {
        hash: Vec<u8>,
        left: Box<MerkleNode>,
        right: Box<MerkleNode>,
    },
}

/// A Merkle proof (authentication path) for verifying a leaf.
#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub siblings: Vec<Vec<u8>>,
}

/// Compute SHA-256 hash of data.
fn hash_data(data: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, data).as_ref().to_vec()
}

/// Compute parent hash from two child hashes: H(left || right).
fn hash_children(left: &[u8], right: &[u8]) -> Vec<u8> {
    let combined = [left, right].concat();
    hash_data(&combined)
}

/// Build a Merkle tree from data blocks.
///
/// Algorithm:
/// 1. Hash each data block to create leaf nodes
/// 2. If odd number of leaves, duplicate the last one
/// 3. Pair adjacent nodes and hash them to create parent nodes
/// 4. Repeat until we have a single root
pub fn build_merkle_tree(data_blocks: &[&[u8]]) -> MerkleNode {
    if data_blocks.is_empty() {
        // Empty tree: root is hash of empty data
        return MerkleNode::Leaf(hash_data(b""));
    }

    // Create leaf nodes
    let mut nodes: Vec<MerkleNode> = data_blocks
        .iter()
        .map(|block| MerkleNode::Leaf(hash_data(block)))
        .collect();

    // If odd number, duplicate the last leaf
    if nodes.len() % 2 != 0 {
        let last = nodes.last().unwrap().clone();
        nodes.push(last);
    }

    // Build tree bottom-up
    while nodes.len() > 1 {
        let mut next_level = Vec::new();
        for pair in nodes.chunks(2) {
            let left = &pair[0];
            let right = &pair[1];
            let left_hash = root_hash(left).to_vec();
            let right_hash = root_hash(right).to_vec();
            let parent_hash = hash_children(&left_hash, &right_hash);
            next_level.push(MerkleNode::Internal {
                hash: parent_hash,
                left: Box::new(left.clone()),
                right: Box::new(right.clone()),
            });
        }
        nodes = next_level;
        // If odd at this level, duplicate last
        if nodes.len() > 1 && nodes.len() % 2 != 0 {
            let last = nodes.last().unwrap().clone();
            nodes.push(last);
        }
    }

    nodes.into_iter().next().unwrap()
}

/// Get the root hash of a Merkle tree node.
pub fn root_hash(tree: &MerkleNode) -> &[u8] {
    match tree {
        MerkleNode::Leaf(hash) => hash,
        MerkleNode::Internal { hash, .. } => hash,
    }
}

/// Generate a Merkle proof (authentication path) for a leaf at the given index.
///
/// The proof contains the sibling hash at each level from leaf to root.
/// To verify, start with the leaf hash, combine with each sibling, and
/// check that the result equals the root.
pub fn generate_proof(tree: &MerkleNode, leaf_index: usize) -> Option<MerkleProof> {
    let leaves = get_leaf_hashes(tree);
    if leaf_index >= leaves.len() {
        return None;
    }

    let mut siblings = Vec::new();
    let mut index = leaf_index;
    let mut nodes = leaves;

    // Build proof bottom-up
    while nodes.len() > 1 {
        // If odd number, duplicate last
        if nodes.len() % 2 != 0 {
            let last = nodes.last().unwrap().clone();
            nodes.push(last);
        }

        // Find sibling
        let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
        if sibling_index < nodes.len() {
            siblings.push(nodes[sibling_index].clone());
        }

        // Compute parent level
        let mut parents = Vec::new();
        for pair in nodes.chunks(2) {
            parents.push(hash_children(&pair[0], &pair[1]));
        }

        index /= 2;
        nodes = parents;
    }

    Some(MerkleProof {
        leaf_index,
        siblings,
    })
}

/// Verify a Merkle proof.
///
/// Start with the leaf hash. For each sibling in the proof:
/// - If current index is even, combine as H(current || sibling)
/// - If current index is odd, combine as H(sibling || current)
/// The final result should equal the root hash.
pub fn verify_proof(leaf_hash: &[u8], proof: &MerkleProof, root: &[u8]) -> bool {
    let mut current = leaf_hash.to_vec();
    let mut index = proof.leaf_index;

    for sibling in &proof.siblings {
        current = if index % 2 == 0 {
            hash_children(&current, sibling)
        } else {
            hash_children(sibling, &current)
        };
        index /= 2;
    }

    current == root
}

/// Collect all leaf hashes from the tree (in order).
pub fn get_leaf_hashes(tree: &MerkleNode) -> Vec<Vec<u8>> {
    match tree {
        MerkleNode::Leaf(hash) => vec![hash.clone()],
        MerkleNode::Internal { left, right, .. } => {
            let mut leaves = get_leaf_hashes(left);
            leaves.extend(get_leaf_hashes(right));
            leaves
        }
    }
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
        assert_eq!(root.len(), 32);

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
        assert_ne!(root_hash(&tree1), root_hash(&tree2));
    }

    #[test]
    fn test_generate_and_verify_proof() {
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let tree = build_merkle_tree(&blocks);
        let root = root_hash(&tree).to_vec();

        // Proof for leaf 0
        let proof = generate_proof(&tree, 0).unwrap();
        let leaf_hash = hash_data(b"a");
        assert!(verify_proof(&leaf_hash, &proof, &root));

        // Proof for leaf 2
        let proof = generate_proof(&tree, 2).unwrap();
        let leaf_hash = hash_data(b"c");
        assert!(verify_proof(&leaf_hash, &proof, &root));
    }

    #[test]
    fn test_tampered_leaf_fails_proof() {
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let tree = build_merkle_tree(&blocks);
        let root = root_hash(&tree).to_vec();
        let proof = generate_proof(&tree, 0).unwrap();
        let tampered_hash = hash_data(b"EVIL");
        assert!(!verify_proof(&tampered_hash, &proof, &root));
    }

    #[test]
    fn test_empty_tree() {
        let tree = build_merkle_tree(&[]);
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
        let blocks: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d", b"e", b"f", b"g", b"h"];
        let tree = build_merkle_tree(&blocks);
        let proof = generate_proof(&tree, 0).unwrap();
        assert_eq!(proof.siblings.len(), 3);
    }
}
