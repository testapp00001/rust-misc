// ============================================================================
// Problem: Longest Repeating Character Replacement (LeetCode #424)
// ============================================================================
// Given a string `s` and an integer `k`, you can choose any character of
// the string and change it to any other uppercase English character. You
// can perform this operation at most `k` times.
//
// Return the length of the longest substring containing the same letter.
//
// Example:
//   Input:  s = "ABAB", k = 2
//   Output: 4  (replace the two 'A's with 'B's or vice versa)
//
// ============================================================================
// APPROACH: Sliding Window + Frequency Count (O(n) time, O(1) space)
// ============================================================================
//
// Key insight: In a valid window, the number of characters we need to
// replace is: window_size - max_frequency. This must be <= k.
//
// Steps:
// 1. Expand the window by moving right.
// 2. Track character frequencies in the window.
// 3. If (window_size - max_freq) > k, shrink from left.
// 4. Track the maximum valid window size.
//
// Optimization: We never shrink the window — we only slide it right.
// The max_freq might be stale, but the window size only grows when we
// find a truly better max_freq.
// ============================================================================

pub fn character_replacement(s: &str, k: i32) -> i32 {
    let chars: Vec<char> = s.chars().collect();
    let mut freq = [0usize; 26];
    let mut left = 0;
    let mut max_freq = 0;
    let mut max_len = 0;

    for right in 0..chars.len() {
        let idx = (chars[right] as u8 - b'A') as usize;
        freq[idx] += 1;
        max_freq = max_freq.max(freq[idx]);

        // Shrink window if too many replacements needed
        while (right - left + 1) - max_freq > k as usize {
            let left_idx = (chars[left] as u8 - b'A') as usize;
            freq[left_idx] -= 1;
            left += 1;
        }

        max_len = max_len.max(right - left + 1);
    }

    max_len as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(character_replacement("ABAB", 2), 4);
    }

    #[test]
    fn test_partial() {
        assert_eq!(character_replacement("AABABBA", 1), 4);
    }

    #[test]
    fn test_single_char() {
        assert_eq!(character_replacement("AAAA", 2), 4);
    }

    #[test]
    fn test_k_zero() {
        assert_eq!(character_replacement("ABCD", 0), 1);
    }

    #[test]
    fn test_empty() {
        assert_eq!(character_replacement("", 2), 0);
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
        //     let _ = character_replacement(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}