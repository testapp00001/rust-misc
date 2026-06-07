// ============================================================================
// Problem: Reorder List (LeetCode #143)
// ============================================================================
// Given the head of a singly linked list L0→L1→…→Ln, reorder it to
// L0→Ln→L1→Ln-1→L2→Ln-2→…
//
// Example:
//   Input:  1 -> 2 -> 3 -> 4
//   Output: 1 -> 4 -> 2 -> 3
//
// ============================================================================
// APPROACH: Three Steps (O(n) time, O(1) space)
// ============================================================================
//
// 1. Find the middle of the list (slow/fast pointers).
// 2. Reverse the second half.
// 3. Merge the two halves alternately.
// ============================================================================

use super::list_node::ListNode;

pub fn reorder_list(head: &mut Option<Box<ListNode>>) {
    if head.is_none() || head.as_ref().unwrap().next.is_none() {
        return;
    }

    // Step 1: Find middle
    let mut slow = head.clone();
    let mut fast = head.clone();
    while fast.is_some() && fast.as_ref().unwrap().next.is_some() {
        slow = slow.unwrap().next;
        fast = fast.unwrap().next.unwrap().next;
    }

    // Step 2: Reverse second half
    let mut second = slow.unwrap().next.take();
    let mut prev: Option<Box<ListNode>> = None;
    while let Some(mut node) = second {
        let next = node.next.take();
        node.next = prev;
        prev = Some(node);
        second = next;
    }

    // Step 3: Merge two halves
    let mut first = head.take();
    let mut second = prev;
    let mut dummy = ListNode::new(0);
    let mut tail = &mut dummy;

    while first.is_some() || second.is_some() {
        if let Some(mut node) = first {
            first = node.next.take();
            tail.next = Some(node);
            tail = tail.next.as_mut().unwrap();
        }
        if let Some(mut node) = second {
            second = node.next.take();
            tail.next = Some(node);
            tail = tail.next.as_mut().unwrap();
        }
    }

    *head = dummy.next;
}

// Simpler approach using Vec
pub fn reorder_list_vec(head: &mut Option<Box<ListNode>>) {
    let vals = ListNode::to_vec(head);
    if vals.len() <= 2 {
        return;
    }

    let mut new_vals = Vec::new();
    let n = vals.len();
    for i in 0..n / 2 {
        new_vals.push(vals[i]);
        new_vals.push(vals[n - 1 - i]);
    }
    if n % 2 == 1 {
        new_vals.push(vals[n / 2]);
    }

    *head = ListNode::from_vec(new_vals);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut head = ListNode::from_vec(vec![1, 2, 3, 4]);
        reorder_list_vec(&mut head);
        assert_eq!(ListNode::to_vec(&head), vec![1, 4, 2, 3]);
    }

    #[test]
    fn test_five() {
        let mut head = ListNode::from_vec(vec![1, 2, 3, 4, 5]);
        reorder_list_vec(&mut head);
        assert_eq!(ListNode::to_vec(&head), vec![1, 5, 2, 4, 3]);
    }

    #[test]
    fn test_two() {
        let mut head = ListNode::from_vec(vec![1, 2]);
        reorder_list_vec(&mut head);
        assert_eq!(ListNode::to_vec(&head), vec![1, 2]);
    }

    #[test]
    fn test_single() {
        let mut head = ListNode::from_vec(vec![1]);
        reorder_list_vec(&mut head);
        assert_eq!(ListNode::to_vec(&head), vec![1]);
    }
}
