// ============================================================================
// Problem: Validate Binary Search Tree (LeetCode #98)
// ============================================================================
// Given the root of a binary tree, determine if it is a valid BST.
//
// A valid BST has:
// - Left subtree values < node's value
// - Right subtree values > node's value
// - Both subtrees are also valid BSTs
//
// ============================================================================
// APPROACH: Recursive with Range (O(n) time, O(h) space)
// ============================================================================
//
// Pass down valid range (min, max) for each node:
// - Root: range is (-∞, ∞)
// - Left child: range is (min, parent.val)
// - Right child: range is (parent.val, max)
//
// Use i64 for bounds to handle i32::MIN and i32::MAX.
// ============================================================================


use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;
type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn is_valid_bst(root: TreeLink) -> bool {
    todo!("Implement is_valid_bst")
}

pub fn is_valid_bst_inorder(root: TreeLink) -> bool {
    todo!("Implement is_valid_bst_inorder")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_valid() {
        let root = TreeNode::from_level_order(&[Some(2), Some(1), Some(3)]);
        assert!(is_valid_bst(root));
    }

    #[test]
    fn test_invalid() {
        let root = TreeNode::from_level_order(&[
            Some(5), Some(1), Some(4), None, None, Some(3), Some(6),
        ]);
        assert!(!is_valid_bst(root));
    }

    #[test]
    fn test_single() {
        let root = TreeNode::from_level_order(&[Some(1)]);
        assert!(is_valid_bst(root));
    }

    #[test]
    fn test_inorder() {
        let root = TreeNode::from_level_order(&[Some(2), Some(1), Some(3)]);
        assert!(is_valid_bst_inorder(root));
    }

    #[test]
    fn test_inorder_invalid() {
        let root = TreeNode::from_level_order(&[
            Some(5), Some(1), Some(4), None, None, Some(3), Some(6),
        ]);
        assert!(!is_valid_bst_inorder(root));
    }


    #[test]
    #[ignore]
    fn bench_performance() {
        // ⏱️  Benchmark test
        // Run: cargo test -p <package> bench_performance -- --ignored --nocapture
        //
        // To use: uncomment and customize the code below with your function
        // and realistic test data.
        //
        // let iterations = 10_000;
        // let input = /* generate your test input here */;
        // let start = std::time::Instant::now();
        // for _ in 0..iterations {
        //     let _ = is_valid_bst(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}