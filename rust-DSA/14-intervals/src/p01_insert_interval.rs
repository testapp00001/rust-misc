// ============================================================================
// Problem: Insert Interval (LeetCode #57)
// ============================================================================
// Given a sorted list of non-overlapping intervals and a new interval,
// insert the new interval and merge if necessary.
//
// Example:
//   intervals = [[1,3],[6,9]], newInterval = [2,5]
//   Output: [[1,5],[6,9]]
//
// ============================================================================
// APPROACH: Linear Scan (O(n) time, O(n) space)
// ============================================================================
//
// 1. Add all intervals that end before newInterval starts.
// 2. Merge all overlapping intervals with newInterval.
// 3. Add remaining intervals.
// ============================================================================



pub fn insert(intervals: Vec<Vec<i32>>, new_interval: Vec<i32>) -> Vec<Vec<i32>> {
    todo!("Implement insert")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            insert(vec![vec![1, 3], vec![6, 9]], vec![2, 5]),
            vec![vec![1, 5], vec![6, 9]]
        );
    }

    #[test]
    fn test_merge_multiple() {
        assert_eq!(
            insert(
                vec![vec![1, 2], vec![3, 5], vec![6, 7], vec![8, 10], vec![12, 16]],
                vec![4, 8]
            ),
            vec![vec![1, 2], vec![3, 10], vec![12, 16]]
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(insert(vec![], vec![5, 7]), vec![vec![5, 7]]);
    }

    #[test]
    fn test_no_overlap() {
        assert_eq!(
            insert(vec![vec![1, 3], vec![6, 9]], vec![4, 5]),
            vec![vec![1, 3], vec![4, 5], vec![6, 9]]
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
        //     let _ = insert(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}