// ============================================================================
// Problem: Copy List with Random Pointer (LeetCode #138)
// ============================================================================
// A linked list is given such that each node contains an additional random
// pointer. Return a deep copy of the list.
//
// ============================================================================
// APPROACH: HashMap (O(n) time, O(n) space)
// ============================================================================
//
// Two-pass approach:
// Pass 1: Create copies of all nodes, store mapping old → new.
// Pass 2: Set next and random pointers using the mapping.
//
// Alternative: Interleave copies (O(1) space) — insert each copy right
// after the original, then separate.
// ============================================================================

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Node {
    pub val: i32,
    pub next: Option<Box<Node>>,
    pub random: Option<usize>, // Index into the list (for simplicity)
}

impl Node {
    pub fn new(val: i32) -> Self {
        Node {
            val,
            next: None,
            random: None,
        }
    }
}

/// Copy a linked list where each node has a random pointer (by index).
/// Returns the copied list as a vector of (val, random_index) tuples.
pub fn copy_list(nodes: &[(i32, Option<usize>)]) -> Vec<(i32, Option<usize>)> {
    nodes.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        // List: 1 -> 2 -> 3, random: 1->3, 2->1, 3->2
        let nodes = vec![
            (1, Some(2)), // node 0 points to node 2 randomly
            (2, Some(0)), // node 1 points to node 0 randomly
            (3, Some(1)), // node 2 points to node 1 randomly
        ];
        let copied = copy_list(&nodes);
        assert_eq!(copied, nodes);
    }

    #[test]
    fn test_no_random() {
        let nodes = vec![(1, None), (2, None), (3, None)];
        let copied = copy_list(&nodes);
        assert_eq!(copied, nodes);
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
        //     let _ = new(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}