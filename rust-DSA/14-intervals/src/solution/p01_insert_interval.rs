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
    let mut result = Vec::new();
    let mut new_interval = new_interval;
    let mut i = 0;
    let n = intervals.len();

    // Add intervals before new_interval
    while i < n && intervals[i][1] < new_interval[0] {
        result.push(intervals[i].clone());
        i += 1;
    }

    // Merge overlapping intervals
    while i < n && intervals[i][0] <= new_interval[1] {
        new_interval[0] = new_interval[0].min(intervals[i][0]);
        new_interval[1] = new_interval[1].max(intervals[i][1]);
        i += 1;
    }
    result.push(new_interval);

    // Add remaining intervals
    while i < n {
        result.push(intervals[i].clone());
        i += 1;
    }

    result
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