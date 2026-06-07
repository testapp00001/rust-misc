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

pub fn max_depth(root: TreeLink) -> i32 {
    match root {
        None => 0,
        Some(node) => {
            let borrowed = node.borrow();
            1 + max_depth(borrowed.left.clone()).max(max_depth(borrowed.right.clone()))
        }
    }
}

// Iterative BFS approach
pub fn max_depth_bfs(root: TreeLink) -> i32 {
    use std::collections::VecDeque;

    if root.is_none() {
        return 0;
    }

    let mut queue = VecDeque::new();
    queue.push_back(root.unwrap());
    let mut depth = 0;

    while !queue.is_empty() {
        let level_size = queue.len();
        for _ in 0..level_size {
            let node = queue.pop_front().unwrap();
            let borrowed = node.borrow();
            if let Some(ref left) = borrowed.left {
                queue.push_back(Rc::clone(left));
            }
            if let Some(ref right) = borrowed.right {
                queue.push_back(Rc::clone(right));
            }
        }
        depth += 1;
    }

    depth
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
}
