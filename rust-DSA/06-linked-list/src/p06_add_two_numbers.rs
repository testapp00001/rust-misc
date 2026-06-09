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

pub fn add_two_numbers( l1: Option<Box<ListNode>>, l2: Option<Box<ListNode>>, ) -> Option<Box<ListNode>> {
    todo!("Implement add_two_numbers")
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
        //     let _ = add_two_numbers(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}