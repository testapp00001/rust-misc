// ============================================================================
// Problem: Partition Labels (LeetCode #763)
// ============================================================================
// Given a string `s`, partition it into as many parts as possible so that
// each letter appears in at most one part. Return the sizes.
//
// Example:
//   Input:  "ababcbacadefegdehijhklij"
//   Output: [9,7,8]
//
// ============================================================================
// APPROACH: Greedy (O(n) time, O(1) space)
// ============================================================================
//
// 1. Record the last occurrence of each character.
// 2. Iterate through the string, tracking the end of the current partition.
// 3. When we reach the end, we've found a partition.
// ============================================================================



pub fn partition_labels(s: &str) -> Vec<i32> {
    todo!("Implement partition_labels")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            partition_labels("ababcbacadefegdehijhklij"),
            vec![9, 7, 8]
        );
    }

    #[test]
    fn test_single() {
        assert_eq!(partition_labels("abc"), vec![1, 1, 1]);
    }

    #[test]
    fn test_all_same() {
        assert_eq!(partition_labels("aaaa"), vec![4]);
    }

    #[test]
    fn test_two_partitions() {
        assert_eq!(partition_labels("abac"), vec![3, 1]);
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
        //     let _ = partition_labels(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}