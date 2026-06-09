// ============================================================================
// Problem: Diameter of Binary Tree (LeetCode #543)
// ============================================================================
// Given the root of a binary tree, return the length of the diameter.
// The diameter is the length of the longest path between any two nodes
// (may or may not pass through the root).
//
// Example:
//   Input:      1
//             /   \
//            2     3
//           / \
//          4   5
//   Output: 3  (path: 4 -> 2 -> 1 -> 3 or 5 -> 2 -> 1 -> 3)
//
// ============================================================================
// APPROACH: Recursive DFS (O(n) time, O(h) space)
// ============================================================================
//
// At each node, the diameter passing through it is:
//   left_depth + right_depth
//
// We track the maximum diameter seen so far and return the depth.
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;

type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn diameter_of_binary_tree(root: TreeLink) -> i32 {
    let mut max_diameter = 0;
    depth(&root, &mut max_diameter);
    max_diameter
}

fn depth(node: &TreeLink, max_diameter: &mut i32) -> i32 {
    match node {
        None => 0,
        Some(n) => {
            let borrowed = n.borrow();
            let left = depth(&borrowed.left, max_diameter);
            let right = depth(&borrowed.right, max_diameter);
            *max_diameter = (*max_diameter).max(left + right);
            1 + left.max(right)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_basic() {
        let root = TreeNode::from_level_order(&[
            Some(1), Some(2), Some(3), Some(4), Some(5),
        ]);
        assert_eq!(diameter_of_binary_tree(root), 3);
    }

    #[test]
    fn test_single() {
        let root = TreeNode::from_level_order(&[Some(1)]);
        assert_eq!(diameter_of_binary_tree(root), 0);
    }

    #[test]
    fn test_line() {
        let root = TreeNode::from_level_order(&[Some(1), Some(2), None, Some(3)]);
        assert_eq!(diameter_of_binary_tree(root), 2);
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
        //     let _ = diameter_of_binary_tree(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}