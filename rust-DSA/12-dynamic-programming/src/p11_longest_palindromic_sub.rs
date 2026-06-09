// ============================================================================
// Problem: Longest Palindromic Substring (LeetCode #5)
// ============================================================================
// Given a string `s`, return the longest palindromic substring.
//
// Example:
//   Input:  "babad"
//   Output: "bab" or "aba"
//
// ============================================================================
// APPROACH: Expand Around Center (O(n²) time, O(1) space)
// ============================================================================
//
// For each character (and each pair of characters), expand outward:
// 1. Odd-length palindromes: center at i.
// 2. Even-length palindromes: center between i and i+1.
//
// Track the maximum palindrome found.
//
// Alternative: Manacher's algorithm (O(n) time).
// ============================================================================



pub fn longest_palindrome(s: &str) -> String {
    todo!("Implement longest_palindrome")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let result = longest_palindrome("babad");
        assert!(result == "bab" || result == "aba");
    }

    #[test]
    fn test_even() {
        assert_eq!(longest_palindrome("cbbd"), "bb");
    }

    #[test]
    fn test_single() {
        assert_eq!(longest_palindrome("a"), "a");
    }

    #[test]
    fn test_all_same() {
        assert_eq!(longest_palindrome("aaaa"), "aaaa");
    }

    #[test]
    fn test_empty() {
        assert_eq!(longest_palindrome(""), "");
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
        //     let _ = longest_palindrome(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}