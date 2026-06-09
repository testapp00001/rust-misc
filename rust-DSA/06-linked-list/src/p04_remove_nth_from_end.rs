// ============================================================================
// Problem: Remove Nth Node From End of List (LeetCode #19)
// ============================================================================
// Given the head of a linked list, remove the nth node from the end and
// return the head.
//
// Example:
//   Input:  1 -> 2 -> 3 -> 4 -> 5, n = 2
//   Output: 1 -> 2 -> 3 -> 5
//
// ============================================================================
// APPROACH: Two Pointers (O(L) time, O(1) space)
// ============================================================================
//
// 1. Move a "fast" pointer n steps ahead.
// 2. Move "slow" and "fast" together until fast reaches the end.
// 3. Slow is now at the node before the one to remove.
//
// Use a dummy head to handle the case where we remove the first node.
// ============================================================================


use super::list_node::ListNode;

pub fn remove_nth_from_end(head: Option<Box<ListNode>>, n: i32) -> Option<Box<ListNode>> {
    todo!("Implement remove_nth_from_end")
}

pub fn remove_nth_from_end_simple(head: Option<Box<ListNode>>, n: i32) -> Option<Box<ListNode>> {
    todo!("Implement remove_nth_from_end_simple")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let head = ListNode::from_vec(vec![1, 2, 3, 4, 5]);
        let result = remove_nth_from_end_simple(head, 2);
        assert_eq!(ListNode::to_vec(&result), vec![1, 2, 3, 5]);
    }

    #[test]
    fn test_remove_first() {
        let head = ListNode::from_vec(vec![1, 2]);
        let result = remove_nth_from_end_simple(head, 2);
        assert_eq!(ListNode::to_vec(&result), vec![2]);
    }

    #[test]
    fn test_single() {
        let head = ListNode::from_vec(vec![1]);
        let result = remove_nth_from_end_simple(head, 1);
        assert_eq!(ListNode::to_vec(&result), vec![]);
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
        //     let _ = remove_nth_from_end(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}