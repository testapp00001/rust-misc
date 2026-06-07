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
    match (p, q) {
        (None, None) => true,
        (Some(p_node), Some(q_node)) => {
            let p = p_node.borrow();
            let q = q_node.borrow();
            p.val == q.val
                && is_same_tree(&p.left, &q.left)
                && is_same_tree(&p.right, &q.right)
        }
        _ => false,
    }
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
}
