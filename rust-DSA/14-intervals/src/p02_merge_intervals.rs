// ============================================================================
// Problem: Merge Intervals (LeetCode #56)
// ============================================================================
// Given an array of intervals, merge all overlapping intervals.
//
// Example:
//   Input:  [[1,3],[2,6],[8,10],[15,18]]
//   Output: [[1,6],[8,10],[15,18]]
//
// ============================================================================
// APPROACH: Sort + Linear Scan (O(n log n) time, O(n) space)
// ============================================================================
//
// 1. Sort intervals by start time.
// 2. For each interval:
//    - If it overlaps with the last merged interval, merge them.
//    - Otherwise, add it as a new interval.
// ============================================================================



pub fn merge(mut intervals: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    todo!("Implement merge")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            merge(vec![vec![1, 3], vec![2, 6], vec![8, 10], vec![15, 18]]),
            vec![vec![1, 6], vec![8, 10], vec![15, 18]]
        );
    }

    #[test]
    fn test_all_overlap() {
        assert_eq!(
            merge(vec![vec![1, 4], vec![4, 5]]),
            vec![vec![1, 5]]
        );
    }

    #[test]
    fn test_no_overlap() {
        assert_eq!(
            merge(vec![vec![1, 2], vec![5, 6]]),
            vec![vec![1, 2], vec![5, 6]]
        );
    }

    #[test]
    fn test_single() {
        assert_eq!(merge(vec![vec![1, 3]]), vec![vec![1, 3]]);
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
        //     let _ = merge(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}