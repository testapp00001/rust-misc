// ============================================================================
// Problem: Same Tree (LeetCode #100)
// ============================================================================
// Given the roots of two binary trees, check if they are the same.
// Two trees are the same if they have the same structure and node values.
//
// ============================================================================
// APPROACH: Recursive (O(n) time, O(h) space)
// ============================================================================
//
// Compare nodes recursively:
// - Both None → same.
// - One None → different.
// - Both Some → values equal AND left subtrees same AND right subtrees same.
// ============================================================================


use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;
type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn is_same_tree(p: &TreeLink, q: &TreeLink) -> bool {
    todo!("Implement is_same_tree")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_same() {
        let p = TreeNode::from_level_order(&[Some(1), Some(2), Some(3)]);
        let q = TreeNode::from_level_order(&[Some(1), Some(2), Some(3)]);
        assert!(is_same_tree(&p, &q));
    }

    #[test]
    fn test_different() {
        let p = TreeNode::from_level_order(&[Some(1), Some(2)]);
        let q = TreeNode::from_level_order(&[Some(1), None, Some(2)]);
        assert!(!is_same_tree(&p, &q));
    }

    #[test]
    fn test_both_empty() {
        let p: TreeLink = None;
        let q: TreeLink = None;
        assert!(is_same_tree(&p, &q));
    }

    #[test]
    fn test_one_empty() {
        let p = TreeNode::from_level_order(&[Some(1)]);
        let q: TreeLink = None;
        assert!(!is_same_tree(&p, &q));
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
        //     let _ = is_same_tree(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}