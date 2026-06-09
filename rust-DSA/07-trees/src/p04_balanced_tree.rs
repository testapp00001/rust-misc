// ============================================================================
// Problem: Balanced Binary Tree (LeetCode #110)
// ============================================================================
// Given a binary tree, determine if it is height-balanced.
// A height-balanced binary tree is one where the depth of the two subtrees
// of every node never differs by more than 1.
//
// ============================================================================
// APPROACH: Recursive (O(n) time, O(h) space)
// ============================================================================
//
// Return -1 if unbalanced, otherwise return the height.
// At each node, check if left and right subtrees are balanced and
// their heights differ by at most 1.
// ============================================================================


use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;
type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn is_balanced(root: TreeLink) -> bool {
    todo!("Implement is_balanced")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_balanced() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(9), Some(20), None, None, Some(15), Some(7),
        ]);
        assert!(is_balanced(root));
    }

    #[test]
    fn test_unbalanced() {
        let root = TreeNode::from_level_order(&[
            Some(1), Some(2), Some(2), Some(3), Some(3), None, None, Some(4), Some(4),
        ]);
        assert!(!is_balanced(root));
    }

    #[test]
    fn test_empty() {
        let root: TreeLink = None;
        assert!(is_balanced(root));
    }

    #[test]
    fn test_single() {
        let root = TreeNode::from_level_order(&[Some(1)]);
        assert!(is_balanced(root));
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
        //     let _ = is_balanced(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}