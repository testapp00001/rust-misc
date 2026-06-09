// ============================================================================
// Problem: Palindrome Partitioning (LeetCode #131)
// ============================================================================
// Given a string `s`, partition `s` such that every substring is a
// palindrome. Return all possible palindrome partitionings.
//
// Example:
//   Input:  "aab"
//   Output: [["a","a","b"],["aa","b"]]
//
// ============================================================================
// APPROACH: Backtracking (O(n * 2^n) time, O(n) space)
// ============================================================================
//
// At each position, try all possible palindrome prefixes:
// 1. If s[start..=end] is a palindrome, add it to current partition.
// 2. Recurse on the remaining string.
// 3. When start reaches the end, we have a valid partition.
// ============================================================================



pub fn partition(s: &str) -> Vec<Vec<String>> {
    todo!("Implement partition")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let result = partition("aab");
        assert_eq!(result.len(), 2);
        assert!(result.contains(&vec!["a".to_string(), "a".to_string(), "b".to_string()]));
        assert!(result.contains(&vec!["aa".to_string(), "b".to_string()]));
    }

    #[test]
    fn test_single() {
        assert_eq!(partition("a"), vec![vec!["a"]]);
    }

    #[test]
    fn test_all_palindrome() {
        let result = partition("aba");
        assert!(result.contains(&vec!["a".to_string(), "b".to_string(), "a".to_string()]));
        assert!(result.contains(&vec!["aba".to_string()]));
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
        //     let _ = partition(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}