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
    todo!("Implement reorder_list")
}

pub fn reorder_list_vec(head: &mut Option<Box<ListNode>>) {
    todo!("Implement reorder_list_vec")
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
        //     let _ = reorder_list(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}