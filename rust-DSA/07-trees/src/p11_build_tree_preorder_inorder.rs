// ============================================================================
// Problem: Construct Binary Tree from Preorder and Inorder (LeetCode #105)
// ============================================================================
// Given two integer arrays `preorder` and `inorder`, construct the binary tree.
//
// Example:
//   preorder = [3,9,20,15,7], inorder = [9,3,15,20,7]
//   Output: [3,9,20,null,null,15,7]
//
// ============================================================================
// APPROACH: Recursive with HashMap (O(n) time, O(n) space)
// ============================================================================
//
// Key insight:
// - preorder[0] is always the root.
// - In inorder, everything left of root is the left subtree,
//   everything right is the right subtree.
//
// Use a HashMap to quickly find the index of each value in inorder.
// ============================================================================


use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::tree_node::TreeNode;
type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn build_tree(preorder: &[i32], inorder: &[i32]) -> TreeLink {
    todo!("Implement build_tree")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_basic() {
        let root = build_tree(&[3, 9, 20, 15, 7], &[9, 3, 15, 20, 7]);
        let result = TreeNode::to_level_order(&root);
        assert_eq!(result, vec![Some(3), Some(9), Some(20), Some(15), Some(7)]);
    }

    #[test]
    fn test_single() {
        let root = build_tree(&[1], &[1]);
        assert_eq!(TreeNode::to_level_order(&root), vec![Some(1)]);
    }

    #[test]
    fn test_empty() {
        let root = build_tree(&[], &[]);
        assert_eq!(TreeNode::to_level_order(&root), vec![]);
    }

    #[test]
    fn test_left_skewed() {
        let root = build_tree(&[1, 2, 3], &[3, 2, 1]);
        let result = TreeNode::to_level_order(&root);
        assert_eq!(result, vec![Some(1), Some(2), Some(3)]);
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
        //     let _ = build_tree(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}