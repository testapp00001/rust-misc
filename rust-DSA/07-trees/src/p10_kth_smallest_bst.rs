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
    let mut result = -1;
    let mut count = 0;
    inorder(&root, k, &mut count, &mut result);
    result
}

fn inorder(node: &TreeLink, k: i32, count: &mut i32, result: &mut i32) {
    if let Some(n) = node {
        let borrowed = n.borrow();
        inorder(&borrowed.left, k, count, result);
        *count += 1;
        if *count == k {
            *result = borrowed.val;
            return;
        }
        inorder(&borrowed.right, k, count, result);
    }
}

// Alternative: Collect all values
pub fn kth_smallest_collect(root: TreeLink, k: i32) -> i32 {
    let mut vals = Vec::new();
    collect(&root, &mut vals);
    vals[(k - 1) as usize]
}

fn collect(node: &TreeLink, vals: &mut Vec<i32>) {
    if let Some(n) = node {
        let borrowed = n.borrow();
        collect(&borrowed.left, vals);
        vals.push(borrowed.val);
        collect(&borrowed.right, vals);
    }
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
}
