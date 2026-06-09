// ============================================================================
// Problem: Kth Smallest Element in a BST (LeetCode #230)
// ============================================================================
// Given the root of a BST and an integer `k`, return the kth smallest
// value (1-indexed) of all values in the tree.
//
// ============================================================================
// APPROACH: In-order Traversal (O(n) time, O(h) space)
// ============================================================================
//
// In-order traversal of a BST gives sorted order.
// Collect elements until we reach the kth one.
// ============================================================================


use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;
type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn kth_smallest(root: TreeLink, k: i32) -> i32 {
    todo!("Implement kth_smallest")
}

pub fn kth_smallest_collect(root: TreeLink, k: i32) -> i32 {
    todo!("Implement kth_smallest_collect")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_basic() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(1), Some(4), None, Some(2),
        ]);
        assert_eq!(kth_smallest(root, 1), 1);
    }

    #[test]
    fn test_second() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(1), Some(4), None, Some(2),
        ]);
        assert_eq!(kth_smallest(root, 2), 2);
    }

    #[test]
    fn test_collect() {
        let root = TreeNode::from_level_order(&[
            Some(5), Some(3), Some(6), Some(2), Some(4), None, None, Some(1),
        ]);
        assert_eq!(kth_smallest_collect(root, 3), 3);
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
        //     let _ = kth_smallest(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}