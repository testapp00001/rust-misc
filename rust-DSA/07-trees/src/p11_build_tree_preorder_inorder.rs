// ============================================================================
// Problem: Construct Binary Tree from Preorder and Inorder (LeetCode #105)
// ============================================================================
// Given two integer arrays `preorder` and `inorder`, construct the binary tree.
//
// Example:
//   preorder = [3,9,20,15,7], inorder = [9,3,15,20,7]
//   Output: [3,9,20,null,null,15,7]
//
// ============================================================================
// APPROACH: Recursive with HashMap (O(n) time, O(n) space)
// ============================================================================
//
// Key insight:
// - preorder[0] is always the root.
// - In inorder, everything left of root is the left subtree,
//   everything right is the right subtree.
//
// Use a HashMap to quickly find the index of each value in inorder.
// ============================================================================

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::tree_node::TreeNode;

type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub fn build_tree(preorder: &[i32], inorder: &[i32]) -> TreeLink {
    if preorder.is_empty() {
        return None;
    }

    let mut inorder_map: HashMap<i32, usize> = HashMap::new();
    for (i, &val) in inorder.iter().enumerate() {
        inorder_map.insert(val, i);
    }

    build(preorder, 0, preorder.len(), inorder, 0, inorder.len(), &inorder_map)
}

fn build(
    preorder: &[i32],
    pre_start: usize,
    pre_end: usize,
    inorder: &[i32],
    in_start: usize,
    in_end: usize,
    inorder_map: &HashMap<i32, usize>,
) -> TreeLink {
    if pre_start >= pre_end || in_start >= in_end {
        return None;
    }

    let root_val = preorder[pre_start];
    let root_idx = inorder_map[&root_val];
    let left_size = root_idx - in_start;

    let root = Rc::new(RefCell::new(TreeNode::new(root_val)));
    root.borrow_mut().left = build(
        preorder,
        pre_start + 1,
        pre_start + 1 + left_size,
        inorder,
        in_start,
        root_idx,
        inorder_map,
    );
    root.borrow_mut().right = build(
        preorder,
        pre_start + 1 + left_size,
        pre_end,
        inorder,
        root_idx + 1,
        in_end,
        inorder_map,
    );

    Some(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_basic() {
        let root = build_tree(&[3, 9, 20, 15, 7], &[9, 3, 15, 20, 7]);
        let result = TreeNode::to_level_order(&root);
        assert_eq!(result, vec![Some(3), Some(9), Some(20), Some(15), Some(7)]);
    }

    #[test]
    fn test_single() {
        let root = build_tree(&[1], &[1]);
        assert_eq!(TreeNode::to_level_order(&root), vec![Some(1)]);
    }

    #[test]
    fn test_empty() {
        let root = build_tree(&[], &[]);
        assert_eq!(TreeNode::to_level_order(&root), vec![]);
    }

    #[test]
    fn test_left_skewed() {
        let root = build_tree(&[1, 2, 3], &[3, 2, 1]);
        let result = TreeNode::to_level_order(&root);
        assert_eq!(result, vec![Some(1), Some(2), Some(3)]);
    }
}
