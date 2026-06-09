// ============================================================================
// Problem: Merge Two Sorted Lists (LeetCode #21)
// ============================================================================
// You are given the heads of two sorted linked lists. Merge the two lists
// into one sorted list by splicing together the nodes.
//
// Example:
//   Input:  1 -> 2 -> 4,  1 -> 3 -> 4
//   Output: 1 -> 1 -> 2 -> 3 -> 4 -> 4
//
// ============================================================================
// APPROACH: Iterative Merge (O(n+m) time, O(1) space)
// ============================================================================
//
// Use a dummy head to simplify edge cases:
// 1. Compare the current nodes of both lists.
// 2. Attach the smaller node to the result.
// 3. Advance the pointer of the list we took from.
// 4. Attach the remaining nodes.
//
// Rust-specific tips:
// - Use a mutable reference to the "tail" of the result list.
// - `.as_mut()` to get a mutable reference to the inner value.
// ============================================================================


use super::list_node::ListNode;

pub fn merge_two_lists( list1: Option<Box<ListNode>>, list2: Option<Box<ListNode>>, ) -> Option<Box<ListNode>> {
    todo!("Implement merge_two_lists")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let l1 = ListNode::from_vec(vec![1, 2, 4]);
        let l2 = ListNode::from_vec(vec![1, 3, 4]);
        let merged = merge_two_lists(l1, l2);
        assert_eq!(ListNode::to_vec(&merged), vec![1, 1, 2, 3, 4, 4]);
    }

    #[test]
    fn test_empty() {
        let l1 = ListNode::from_vec(vec![]);
        let l2 = ListNode::from_vec(vec![]);
        let merged = merge_two_lists(l1, l2);
        assert_eq!(ListNode::to_vec(&merged), vec![]);
    }

    #[test]
    fn test_one_empty() {
        let l1 = ListNode::from_vec(vec![1, 2, 3]);
        let l2 = ListNode::from_vec(vec![]);
        let merged = merge_two_lists(l1, l2);
        assert_eq!(ListNode::to_vec(&merged), vec![1, 2, 3]);
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
        //     let _ = merge_two_lists(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}