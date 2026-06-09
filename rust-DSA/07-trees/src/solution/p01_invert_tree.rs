// ============================================================================
// Problem: Invert Binary Tree (LeetCode #226)
// ============================================================================
// Given the root of a binary tree, invert the tree and return its root.
//
// Inverting means swapping left and right children of every node.
//
// Example:
//   Input:      4            Output:     4
//             /   \                    /   \
//            2     7                  7     2
//           / \   / \                / \   / \
//          1   3 6   9              9   6 3   1
//
// ============================================================================
// APPROACH: Recursive (O(n) time, O(h) space for call stack)
// ============================================================================
//
// For each node:
// 1. Recursively invert the left subtree.
// 2. Recursively invert the right subtree.
// 3. Swap left and right children.
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;

type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn invert_tree(root: TreeLink) -> TreeLink {
    root.map(|node| {
        let mut borrowed = node.borrow_mut();
        let left = borrowed.left.take();
        let right = borrowed.right.take();
        borrowed.left = invert_tree(right);
        borrowed.right = invert_tree(left);
        Rc::clone(&node)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    fn to_level(root: &TreeLink) -> Vec<Option<i32>> {
        TreeNode::to_level_order(root)
    }

    #[test]
    fn test_basic() {
        let root = TreeNode::from_level_order(&[
            Some(4), Some(2), Some(7), Some(1), Some(3), Some(6), Some(9),
        ]);
        let inverted = invert_tree(root);
        assert_eq!(to_level(&inverted), vec![Some(4), Some(7), Some(2), Some(9), Some(6), Some(3), Some(1)]);
    }

    #[test]
    fn test_simple() {
        let root = TreeNode::from_level_order(&[Some(2), Some(1), Some(3)]);
        let inverted = invert_tree(root);
        assert_eq!(to_level(&inverted), vec![Some(2), Some(3), Some(1)]);
    }

    #[test]
    fn test_empty() {
        let root: TreeLink = None;
        let inverted = invert_tree(root);
        assert_eq!(to_level(&inverted), vec![]);
    }

    #[test]
    fn test_single() {
        let root = TreeNode::from_level_order(&[Some(1)]);
        let inverted = invert_tree(root);
        assert_eq!(to_level(&inverted), vec![Some(1)]);
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
        //     let _ = invert_tree(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}