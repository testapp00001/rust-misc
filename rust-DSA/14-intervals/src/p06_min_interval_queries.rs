// ============================================================================
// Problem: Minimum Interval to Include Each Query (LeetCode #1851)
// ============================================================================
// Given intervals and queries, for each query find the smallest interval
// that contains it. If none, return -1.
//
// ============================================================================
// APPROACH: Sort + Sweep Line + Min Heap (O((n+q) log(n+q)) time, O(n+q) space)
// ============================================================================
//
// 1. Sort intervals by start, queries by value.
// 2. Use a min heap of (size, end) for active intervals.
// 3. Sweep through queries, adding intervals that start <= query.
// 4. Remove intervals that end < query.
// 5. The smallest active interval is the answer.
// ============================================================================


use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub fn min_interval(intervals: Vec<Vec<i32>>, queries: Vec<i32>) -> Vec<i32> {
    todo!("Implement min_interval")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let intervals = vec![vec![1, 4], vec![2, 4], vec![3, 6], vec![4, 4]];
        let queries = vec![2, 3, 4, 5];
        assert_eq!(min_interval(intervals, queries), vec![3, 3, 1, 4]);
    }

    #[test]
    fn test_no_cover() {
        let intervals = vec![vec![2, 3]];
        let queries = vec![1, 4];
        assert_eq!(min_interval(intervals, queries), vec![-1, -1]);
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
        //     let _ = min_interval(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}