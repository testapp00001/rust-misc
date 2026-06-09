// ============================================================================
// Problem: Non-overlapping Intervals (LeetCode #435)
// ============================================================================
// Given an array of intervals, return the minimum number of intervals to
// remove to make the rest non-overlapping.
//
// Example:
//   Input:  [[1,2],[2,3],[3,4],[1,3]]
//   Output: 1  (remove [1,3])
//
// ============================================================================
// APPROACH: Greedy (O(n log n) time, O(1) space)
// ============================================================================
//
// Sort by end time. Greedily keep intervals that end earliest:
// 1. Sort intervals by end time.
// 2. Keep the first interval.
// 3. For each subsequent interval:
//    - If it starts after the last kept interval ends → keep it.
//    - Otherwise → remove it (count++).
//
// This maximizes the number of non-overlapping intervals.
// ============================================================================



pub fn erase_overlap_intervals(mut intervals: Vec<Vec<i32>>) -> i32 {
    todo!("Implement erase_overlap_intervals")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            erase_overlap_intervals(vec![vec![1, 2], vec![2, 3], vec![3, 4], vec![1, 3]]),
            1
        );
    }

    #[test]
    fn test_all_overlap() {
        assert_eq!(
            erase_overlap_intervals(vec![vec![1, 2], vec![1, 2], vec![1, 2]]),
            2
        );
    }

    #[test]
    fn test_no_overlap() {
        assert_eq!(
            erase_overlap_intervals(vec![vec![1, 2], vec![2, 3]]),
            0
        );
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
        //     let _ = erase_overlap_intervals(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}