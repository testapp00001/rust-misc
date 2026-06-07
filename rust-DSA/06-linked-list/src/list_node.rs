// ============================================================================
// Shared ListNode Definition for Linked List Problems
// ============================================================================
// This module provides a standard ListNode that can be used across all
// linked list problems in this topic.
//
// In Rust, linked lists are trickier than in languages with garbage
// collection because of ownership rules. Key concepts:
//
// 1. Box<T>: A heap-allocated pointer with single ownership.
//    - `Box<ListNode>` owns the node it points to.
//    - When the Box is dropped, the node is deallocated.
//
// 2. Option<T>: Represents nullable values.
//    - `Option<Box<ListNode>>` is either Some(node) or None.
//    - This replaces the null pointer pattern from C/C++.
//
// 3. Ownership transfer:
//    - `node.next.take()` takes ownership and leaves None in its place.
//    - This is how you "unlink" a node from the list.
//
// Common patterns:
// - Use `&mut Option<Box<ListNode>>` to modify the list in place.
// - Use `.as_mut()` to get `Option<&mut Box<ListNode>>`.
// - Use `.take()` to take ownership of a node.

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    pub fn new(val: i32) -> Self {
        ListNode { val, next: None }
    }

    /// Create a linked list from a vector of values.
    /// Example: ListNode::from_vec(vec![1, 2, 3]) creates 1 -> 2 -> 3
    pub fn from_vec(vals: Vec<i32>) -> Option<Box<ListNode>> {
        let mut head: Option<Box<ListNode>> = None;
        for &val in vals.iter().rev() {
            let mut node = ListNode::new(val);
            node.next = head;
            head = Some(Box::new(node));
        }
        head
    }

    /// Convert a linked list to a vector of values.
    pub fn to_vec(mut head: &Option<Box<ListNode>>) -> Vec<i32> {
        let mut result = Vec::new();
        while let Some(node) = head {
            result.push(node.val);
            head = &node.next;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_vec() {
        let head = ListNode::from_vec(vec![1, 2, 3]);
        assert_eq!(ListNode::to_vec(&head), vec![1, 2, 3]);
    }

    #[test]
    fn test_empty() {
        let head = ListNode::from_vec(vec![]);
        assert_eq!(ListNode::to_vec(&head), vec![]);
    }

    #[test]
    fn test_single() {
        let head = ListNode::from_vec(vec![42]);
        assert_eq!(ListNode::to_vec(&head), vec![42]);
    }
}
