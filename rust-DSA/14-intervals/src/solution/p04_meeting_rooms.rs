// ============================================================================
// Problem: Meeting Rooms (LeetCode #252)
// ============================================================================
// Given an array of meeting time intervals, determine if a person could
// attend all meetings.
//
// Example:
//   Input:  [[0,30],[5,10],[15,20]]
//   Output: false  (meetings overlap)
//
// ============================================================================
// APPROACH: Sort + Check Adjacent (O(n log n) time, O(1) space)
// ============================================================================
//
// Sort by start time. Check if any two adjacent meetings overlap.
// ============================================================================

pub fn can_attend_meetings(mut intervals: Vec<Vec<i32>>) -> bool {
    intervals.sort_by_key(|v| v[0]);

    for i in 1..intervals.len() {
        if intervals[i][0] < intervals[i - 1][1] {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cannot_attend() {
        assert!(!can_attend_meetings(vec![vec![0, 30], vec![5, 10], vec![15, 20]]));
    }

    #[test]
    fn test_can_attend() {
        assert!(can_attend_meetings(vec![vec![0, 5], vec![10, 15]]));
    }

    #[test]
    fn test_single() {
        assert!(can_attend_meetings(vec![vec![0, 5]]));
    }

    #[test]
    fn test_empty() {
        assert!(can_attend_meetings(vec![]));
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
        //     let _ = can_attend_meetings(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}