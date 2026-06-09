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
    use std::collections::HashSet;

pub fn has_cycle(head: &Option<Box<ListNode>>) -> bool {
    todo!("Implement has_cycle")
}

pub fn has_cycle_hashset(head: &Option<Box<ListNode>>) -> bool {
    todo!("Implement has_cycle_hashset")
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
        //     let _ = has_cycle(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}