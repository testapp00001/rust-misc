// ============================================================================
// Problem: Lowest Common Ancestor of a Binary Tree (LeetCode #236)
// ============================================================================
// Given a binary tree, find the lowest common ancestor (LCA) of two given
// nodes. The LCA is the deepest node that is an ancestor of both nodes.
//
// Example:
//   Input: root = [3,5,1,6,2,0,8,null,null,7,4], p = 5, q = 1
//   Output: 3
//
// ============================================================================
// APPROACH: Recursive (O(n) time, O(h) space)
// ============================================================================
//
// At each node:
// - If node is None or equals p or q, return the node.
// - Recurse on left and right subtrees.
// - If both return non-None, current node is the LCA.
// - If only one returns non-None, that's the LCA.
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;

type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn lowest_common_ancestor(root: TreeLink, p: i32, q: i32) -> TreeLink {
    fn helper(node: &TreeLink, p: i32, q: i32) -> TreeLink {
        match node {
            None => None,
            Some(n) => {
                let borrowed = n.borrow();
                if borrowed.val == p || borrowed.val == q {
                    return Some(Rc::clone(n));
                }

                let left = helper(&borrowed.left, p, q);
                let right = helper(&borrowed.right, p, q);

                match (left, right) {
                    (Some(_), Some(_)) => Some(Rc::clone(n)),
                    (Some(l), None) => Some(l),
                    (None, Some(r)) => Some(r),
                    (None, None) => None,
                }
            }
        }
    }

    helper(&root, p, q)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_basic() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(5), Some(1), Some(6), Some(2), Some(0), Some(8),
            None, None, Some(7), Some(4),
        ]);
        let lca = lowest_common_ancestor(root, 5, 1);
        assert_eq!(lca.unwrap().borrow().val, 3);
    }

    #[test]
    fn test_ancestor_is_one_node() {
        let root = TreeNode::from_level_order(&[
            Some(3), Some(5), Some(1), Some(6), Some(2), Some(0), Some(8),
            None, None, Some(7), Some(4),
        ]);
        let lca = lowest_common_ancestor(root, 5, 4);
        assert_eq!(lca.unwrap().borrow().val, 5);
    }

    #[test]
    fn test_root() {
        let root = TreeNode::from_level_order(&[Some(1), Some(2), Some(3)]);
        let lca = lowest_common_ancestor(root, 2, 3);
        assert_eq!(lca.unwrap().borrow().val, 1);
    }
}
