// ============================================================================
// Problem: Linked List Cycle (LeetCode #141)
// ============================================================================
// Given `head`, determine if the linked list has a cycle.
//
// A cycle exists if some node can be reached again by continuously following
// the next pointer.
//
// ============================================================================
// APPROACH: Floyd's Tortoise and Hare (O(n) time, O(1) space)
// ============================================================================
//
// Use two pointers moving at different speeds:
// 1. Slow pointer moves one step at a time.
// 2. Fast pointer moves two steps at a time.
// 3. If there's a cycle, they will eventually meet.
// 4. If fast reaches the end (None), there's no cycle.
//
// Why it works: If there's a cycle, the fast pointer will "lap" the slow
// pointer inside the cycle.
// ============================================================================

use super::list_node::ListNode;

pub fn has_cycle(head: &Option<Box<ListNode>>) -> bool {
    let mut slow = head.as_ref();
    let mut fast = head.as_ref();

    while let (Some(s), Some(f)) = (slow, fast) {
        // Move slow one step
        slow = s.next.as_ref();
        // Move fast two steps
        fast = f.next.as_ref().and_then(|n| n.next.as_ref());

        // Check if they meet
        if let (Some(s), Some(f)) = (slow, fast) {
            if std::ptr::eq(s, f) {
                return true;
            }
        }
    }

    false
}

// Alternative: HashSet approach (O(n) space)
pub fn has_cycle_hashset(head: &Option<Box<ListNode>>) -> bool {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut current = head.as_ref();

    while let Some(node) = current {
        let ptr = node.as_ref() as *const ListNode;
        if !seen.insert(ptr) {
            return true;
        }
        current = node.next.as_ref();
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycle() {
        let head = ListNode::from_vec(vec![1, 2, 3, 4]);
        assert!(!has_cycle(&head));
    }

    #[test]
    fn test_empty() {
        let head: Option<Box<ListNode>> = None;
        assert!(!has_cycle(&head));
    }

    #[test]
    fn test_single_no_cycle() {
        let head = ListNode::from_vec(vec![1]);
        assert!(!has_cycle(&head));
    }

    #[test]
    fn test_hashset_approach() {
        let head = ListNode::from_vec(vec![1, 2, 3]);
        assert!(!has_cycle_hashset(&head));
    }
}
