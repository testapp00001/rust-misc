// ============================================================================
// Problem: Subtree of Another Tree (LeetCode #572)
// ============================================================================
// Given the roots of two binary trees `root` and `subRoot`, return true if
// there is a subtree of `root` with the same structure and node values as
// `subRoot`.
//
// ============================================================================
// APPROACH: Recursive (O(m*n) time, O(h) space)
// ============================================================================
//
// For each node in root, check if the subtree rooted at that node is
// the same as subRoot.
// ============================================================================


use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;
type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn is_subtree(root: TreeLink, sub_root: TreeLink) -> bool {
    todo!("Implement is_subtree")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_is_subtree() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(4), Some(5), Some(1), Some(2),
        ]);
        let sub = TreeNode::from_level_order(&[Some(4), Some(1), Some(2)]);
        assert!(is_subtree(root, sub));
    }

    #[test]
    fn test_not_subtree() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(4), Some(5), Some(1), Some(2), None, None, None, None, Some(0),
        ]);
        let sub = TreeNode::from_level_order(&[Some(4), Some(1), Some(2)]);
        assert!(!is_subtree(root, sub));
    }

    #[test]
    fn test_same_tree() {
        let root = TreeNode::from_level_order(&[Some(1), Some(2), Some(3)]);
        let sub = TreeNode::from_level_order(&[Some(1), Some(2), Some(3)]);
        assert!(is_subtree(root, sub));
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
        //     let _ = is_subtree(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}