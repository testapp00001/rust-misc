// ============================================================================
// Problem: Add Two Numbers (LeetCode #2)
// ============================================================================
// You are given two non-empty linked lists representing two non-negative
// integers. The digits are stored in reverse order. Add the two numbers
// and return the sum as a linked list.
//
// Example:
//   Input:  (2 -> 4 -> 3) + (5 -> 6 -> 4)
//   Output: 7 -> 0 -> 8  (342 + 465 = 807)
//
// ============================================================================
// APPROACH: Grade-School Addition (O(max(m,n)) time, O(max(m,n)) space)
// ============================================================================
//
// Simulate digit-by-digit addition with carry:
// 1. Add corresponding digits plus carry.
// 2. New digit = sum % 10, new carry = sum / 10.
// 3. Continue until both lists are exhausted and carry is 0.
// ============================================================================

use super::list_node::ListNode;

pub fn add_two_numbers(
    l1: Option<Box<ListNode>>,
    l2: Option<Box<ListNode>>,
) -> Option<Box<ListNode>> {
    let mut dummy = ListNode::new(0);
    let mut tail = &mut dummy;
    let mut p1 = l1;
    let mut p2 = l2;
    let mut carry = 0;

    while p1.is_some() || p2.is_some() || carry > 0 {
        let sum = carry
            + p1.as_ref().map_or(0, |n| n.val)
            + p2.as_ref().map_or(0, |n| n.val);

        carry = sum / 10;
        tail.next = Some(Box::new(ListNode::new(sum % 10)));
        tail = tail.next.as_mut().unwrap();

        p1 = p1.and_then(|n| n.next);
        p2 = p2.and_then(|n| n.next);
    }

    dummy.next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let l1 = ListNode::from_vec(vec![2, 4, 3]);
        let l2 = ListNode::from_vec(vec![5, 6, 4]);
        let result = add_two_numbers(l1, l2);
        assert_eq!(ListNode::to_vec(&result), vec![7, 0, 8]);
    }

    #[test]
    fn test_carry() {
        let l1 = ListNode::from_vec(vec![9, 9, 9]);
        let l2 = ListNode::from_vec(vec![1]);
        let result = add_two_numbers(l1, l2);
        assert_eq!(ListNode::to_vec(&result), vec![0, 0, 0, 1]);
    }

    #[test]
    fn test_different_lengths() {
        let l1 = ListNode::from_vec(vec![1, 8]);
        let l2 = ListNode::from_vec(vec![0]);
        let result = add_two_numbers(l1, l2);
        assert_eq!(ListNode::to_vec(&result), vec![1, 8]);
    }
}
