// ============================================================================
// Problem: Reverse Linked List (LeetCode #206)
// ============================================================================
// Given the head of a singly linked list, reverse the list and return the
// reversed list.
//
// Example:
//   Input:  1 -> 2 -> 3 -> 4 -> 5
//   Output: 5 -> 4 -> 3 -> 2 -> 1
//
// ============================================================================
// APPROACH: Iterative (O(n) time, O(1) space)
// ============================================================================
//
// Use three pointers: prev, current, next.
// 1. Save next = current.next.
// 2. Reverse the link: current.next = prev.
// 3. Move prev = current, current = next.
//
// Rust-specific tips:
// - Use `.take()` to take ownership and leave None.
// - `Option::take()` is essential for avoiding double-borrow issues.
// ============================================================================

use super::list_node::ListNode;

pub fn reverse_list(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    let mut prev: Option<Box<ListNode>> = None;
    let mut current = head;

    while let Some(mut node) = current {
        let next = node.next.take();
        node.next = prev;
        prev = Some(node);
        current = next;
    }

    prev
}

// Recursive approach
pub fn reverse_list_recursive(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    fn helper(
        current: Option<Box<ListNode>>,
        prev: Option<Box<ListNode>>,
    ) -> Option<Box<ListNode>> {
        match current {
            None => prev,
            Some(mut node) => {
                let next = node.next.take();
                node.next = prev;
                helper(next, Some(node))
            }
        }
    }

    helper(head, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let head = ListNode::from_vec(vec![1, 2, 3, 4, 5]);
        let reversed = reverse_list(head);
        assert_eq!(ListNode::to_vec(&reversed), vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_two() {
        let head = ListNode::from_vec(vec![1, 2]);
        let reversed = reverse_list(head);
        assert_eq!(ListNode::to_vec(&reversed), vec![2, 1]);
    }

    #[test]
    fn test_empty() {
        let reversed = reverse_list(None);
        assert_eq!(ListNode::to_vec(&reversed), vec![]);
    }

    #[test]
    fn test_recursive() {
        let head = ListNode::from_vec(vec![1, 2, 3, 4, 5]);
        let reversed = reverse_list_recursive(head);
        assert_eq!(ListNode::to_vec(&reversed), vec![5, 4, 3, 2, 1]);
    }


    #[test]
    #[ignore]
    fn bench_performance() {
        // ⏱️  Benchmark test
        // Run: cargo test -p <package> bench_performance -- --ignored --nocapture
        //
        // To use: uncomment and customize the code below with your function
        // and realistic test data.
        //
        // let iterations = 10_000;
        // let input = /* generate your test input here */;
        // let start = std::time::Instant::now();
        // for _ in 0..iterations {
        //     let _ = reverse_list(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}