// ============================================================================
// Problem: Maximum Depth of Binary Tree (LeetCode #104)
// ============================================================================
// Given the root of a binary tree, return its maximum depth.
// The maximum depth is the number of nodes along the longest path from
// the root node down to the farthest leaf node.
//
// Example:
//   Input:  [3, 9, 20, null, null, 15, 7]
//   Output: 3
//
// ============================================================================
// APPROACH: Recursive DFS (O(n) time, O(h) space)
// ============================================================================
//
// Base case: if node is None, depth is 0.
// Recursive case: depth = 1 + max(depth(left), depth(right))
// ============================================================================


use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;
type TreeLink = Option<Rc<RefCell<TreeNode>>>;
    use std::collections::VecDeque;

pub fn max_depth(root: TreeLink) -> i32 {
    todo!("Implement max_depth")
}

pub fn max_depth_bfs(root: TreeLink) -> i32 {
    todo!("Implement max_depth_bfs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_basic() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(9), Some(20), None, None, Some(15), Some(7),
        ]);
        assert_eq!(max_depth(root), 3);
    }

    #[test]
    fn test_single() {
        let root = TreeNode::from_level_order(&[Some(1)]);
        assert_eq!(max_depth(root), 1);
    }

    #[test]
    fn test_empty() {
        let root: TreeLink = None;
        assert_eq!(max_depth(root), 0);
    }

    #[test]
    fn test_bfs() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(9), Some(20), None, None, Some(15), Some(7),
        ]);
        assert_eq!(max_depth_bfs(root), 3);
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
        //     let _ = max_depth(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}