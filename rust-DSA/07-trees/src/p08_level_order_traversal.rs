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
    let mut result = Vec::new();
    if root.is_none() {
        return result;
    }

    let mut queue = VecDeque::new();
    queue.push_back(root.unwrap());

    while !queue.is_empty() {
        let level_size = queue.len();
        let mut level = Vec::new();

        for _ in 0..level_size {
            let node = queue.pop_front().unwrap();
            let borrowed = node.borrow();
            level.push(borrowed.val);

            if let Some(ref left) = borrowed.left {
                queue.push_back(Rc::clone(left));
            }
            if let Some(ref right) = borrowed.right {
                queue.push_back(Rc::clone(right));
            }
        }

        result.push(level);
    }

    result
}

// Alternative: Recursive DFS approach
pub fn level_order_dfs(root: TreeLink) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    dfs(&root, 0, &mut result);
    result
}

fn dfs(node: &TreeLink, depth: usize, result: &mut Vec<Vec<i32>>) {
    if let Some(n) = node {
        let borrowed = n.borrow();
        if depth == result.len() {
            result.push(Vec::new());
        }
        result[depth].push(borrowed.val);
        dfs(&borrowed.left, depth + 1, result);
        dfs(&borrowed.right, depth + 1, result);
    }
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
}
