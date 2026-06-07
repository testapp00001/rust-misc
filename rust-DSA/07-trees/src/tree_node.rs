// ============================================================================
// Shared TreeNode Definition for Tree Problems
// ============================================================================
// This module provides a standard TreeNode that can be used across all
// tree problems in this topic.
//
// In Rust, trees are challenging due to ownership rules. Key concepts:
//
// 1. Rc<T>: Reference-counted pointer for shared ownership.
//    - Multiple references can point to the same node.
//    - `Rc::clone()` increments the reference count (cheap).
//
// 2. RefCell<T>: Interior mutability — allows mutation through shared refs.
//    - `borrow()` gives a shared reference.
//    - `borrow_mut()` gives a mutable reference (panics if already borrowed).
//
// 3. Option<Rc<RefCell<TreeNode>>>: The standard tree node type.
//    - Option: node may or may not exist.
//    - Rc: shared ownership between parent and children.
//    - RefCell: allows modifying children through shared parent reference.
//
// Common patterns:
// - `node.borrow()` to access the value.
// - `node.borrow_mut()` to modify.
// - `Rc::clone(&node)` to create another reference.

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }

    /// Create a tree from a level-order vector where None represents missing nodes.
    /// Example: vec![Some(1), Some(2), Some(3), None, None, Some(4), Some(5)]
    /// Creates:
    ///        1
    ///       / \
    ///      2   3
    ///         / \
    ///        4   5
    pub fn from_level_order(vals: &[Option<i32>]) -> Option<Rc<RefCell<TreeNode>>> {
        if vals.is_empty() || vals[0].is_none() {
            return None;
        }

        let root = Rc::new(RefCell::new(TreeNode::new(vals[0].unwrap())));
        let mut queue = vec![Rc::clone(&root)];
        let mut i = 1;

        while i < vals.len() {
            let node = queue.remove(0);

            // Left child
            if i < vals.len() {
                if let Some(val) = vals[i] {
                    let left = Rc::new(RefCell::new(TreeNode::new(val)));
                    node.borrow_mut().left = Some(Rc::clone(&left));
                    queue.push(left);
                }
                i += 1;
            }

            // Right child
            if i < vals.len() {
                if let Some(val) = vals[i] {
                    let right = Rc::new(RefCell::new(TreeNode::new(val)));
                    node.borrow_mut().right = Some(Rc::clone(&right));
                    queue.push(right);
                }
                i += 1;
            }
        }

        Some(root)
    }

    /// Convert a tree to level-order vector (for testing).
    pub fn to_level_order(root: &Option<Rc<RefCell<TreeNode>>>) -> Vec<Option<i32>> {
        let mut result = Vec::new();
        if let Some(node) = root {
            let mut queue = vec![Rc::clone(node)];
            while !queue.is_empty() {
                let current = queue.remove(0);
                let borrowed = current.borrow();
                result.push(Some(borrowed.val));

                if let Some(ref left) = borrowed.left {
                    queue.push(Rc::clone(left));
                }
                if let Some(ref right) = borrowed.right {
                    queue.push(Rc::clone(right));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_level_order() {
        let vals = vec![Some(1), Some(2), Some(3), None, None, Some(4), Some(5)];
        let root = TreeNode::from_level_order(&vals);
        let result = TreeNode::to_level_order(&root);
        assert_eq!(result, vec![Some(1), Some(2), Some(3), Some(4), Some(5)]);
    }

    #[test]
    fn test_empty() {
        let root = TreeNode::from_level_order(&[]);
        assert_eq!(TreeNode::to_level_order(&root), vec![]);
    }

    #[test]
    fn test_single() {
        let root = TreeNode::from_level_order(&[Some(42)]);
        assert_eq!(TreeNode::to_level_order(&root), vec![Some(42)]);
    }
}
