// ============================================================================
// Problem: Binary Tree Level Order Traversal (LeetCode #102)
// ============================================================================
// Given the root of a binary tree, return the level order traversal of its
// nodes' values (from left to right, level by level).
//
// Example:
//   Input:  [3, 9, 20, null, null, 15, 7]
//   Output: [[3], [9, 20], [15, 7]]
//
// ============================================================================
// APPROACH: BFS with Queue (O(n) time, O(n) space)
// ============================================================================
//
// Use a queue to process nodes level by level:
// 1. Start with the root in the queue.
// 2. For each level, process all nodes currently in the queue.
// 3. Add their children to the queue for the next level.
// ============================================================================


use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use super::tree_node::TreeNode;
type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn level_order(root: TreeLink) -> Vec<Vec<i32>> {
    todo!("Implement level_order")
}

pub fn level_order_dfs(root: TreeLink) -> Vec<Vec<i32>> {
    todo!("Implement level_order_dfs")
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
        assert_eq!(level_order(root), vec![vec![3], vec![9, 20], vec![15, 7]]);
    }

    #[test]
    fn test_single() {
        let root = TreeNode::from_level_order(&[Some(1)]);
        assert_eq!(level_order(root), vec![vec![1]]);
    }

    #[test]
    fn test_empty() {
        let root: TreeLink = None;
        assert_eq!(level_order(root), Vec::<Vec<i32>>::new());
    }

    #[test]
    fn test_dfs_approach() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(9), Some(20), None, None, Some(15), Some(7),
        ]);
        assert_eq!(level_order_dfs(root), vec![vec![3], vec![9, 20], vec![15, 7]]);
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
        //     let _ = level_order(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}