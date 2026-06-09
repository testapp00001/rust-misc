// ============================================================================
// Problem: Meeting Rooms II (LeetCode #253)
// ============================================================================
// Given an array of meeting time intervals, return the minimum number of
// conference rooms required.
//
// Example:
//   Input:  [[0,30],[5,10],[15,20]]
//   Output: 2
//
// ============================================================================
// APPROACH: Sort + Min Heap (O(n log n) time, O(n) space)
// ============================================================================
//
// 1. Sort meetings by start time.
// 2. Use a min heap to track end times of ongoing meetings.
// 3. For each meeting:
//    - If the earliest ending meeting has ended → reuse that room.
//    - Otherwise → allocate a new room.
// 4. The heap size is the number of rooms needed.
//
// Alternative: Chronological ordering (two sorted arrays).
// ============================================================================


use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub fn min_meeting_rooms(mut intervals: Vec<Vec<i32>>) -> i32 {
    todo!("Implement min_meeting_rooms")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            min_meeting_rooms(vec![vec![0, 30], vec![5, 10], vec![15, 20]]),
            2
        );
    }

    #[test]
    fn test_one_room() {
        assert_eq!(
            min_meeting_rooms(vec![vec![0, 5], vec![10, 15]]),
            1
        );
    }

    #[test]
    fn test_all_overlap() {
        assert_eq!(
            min_meeting_rooms(vec![vec![0, 10], vec![5, 15], vec![10, 20]]),
            2
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(min_meeting_rooms(vec![]), 0);
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
        //     let _ = min_meeting_rooms(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}