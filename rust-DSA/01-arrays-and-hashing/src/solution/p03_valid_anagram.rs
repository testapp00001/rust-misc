// ============================================================================
// Problem: Valid Anagram (LeetCode #242)
// ============================================================================
// Given two strings `s` and `t`, return `true` if `t` is an anagram of `s`.
// An anagram is a word formed by rearranging the letters of another word.
//
// Example:
//   s = "anagram", t = "nagaram" → true
//   s = "rat",     t = "car"     → false
//
// ============================================================================
// APPROACH 1: Character Frequency Count (O(n) time, O(1) space)
// ============================================================================
//
// Since we're dealing with lowercase English letters (26 chars), we can use
// a fixed-size array of 26 counters:
// 1. For each char in `s`, increment its counter.
// 2. For each char in `t`, decrement its counter.
// 3. If all counters are zero → anagram.
//
// This is O(1) space because the array size is fixed (26).
//
// APPROACH 2: Sort both strings and compare — O(n log n).
//
// Rust-specific tips:
// - `b'a'` gives the byte value of 'a' (97). Use this to map chars to indices.
// - `as_bytes()` converts a string slice to a byte slice.
// ============================================================================

pub fn is_anagram(s: &str, t: &str) -> bool {
    if s.len() != t.len() {
        return false;
    }

    let mut counts = [0i32; 26];

    for (sc, tc) in s.bytes().zip(t.bytes()) {
        counts[(sc - b'a') as usize] += 1;
        counts[(tc - b'a') as usize] -= 1;
    }

    counts.iter().all(|&c| c == 0)
}

// Alternative: HashMap approach (works with Unicode)
pub fn is_anagram_unicode(s: &str, t: &str) -> bool {
    use std::collections::HashMap;
    let mut map: HashMap<char, i32> = HashMap::new();

    for c in s.chars() {
        *map.entry(c).or_insert(0) += 1;
    }
    for c in t.chars() {
        *map.entry(c).or_insert(0) -= 1;
    }

    map.values().all(|&v| v == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_anagram() {
        assert!(is_anagram("anagram", "nagaram"));
    }

    #[test]
    fn test_not_anagram() {
        assert!(!is_anagram("rat", "car"));
    }

    #[test]
    fn test_different_length() {
        assert!(!is_anagram("ab", "a"));
    }

    #[test]
    fn test_empty_strings() {
        assert!(is_anagram("", ""));
    }

    #[test]
    fn test_single_char() {
        assert!(is_anagram("a", "a"));
        assert!(!is_anagram("a", "b"));
    }

    #[test]
    fn test_unicode_approach() {
        assert!(is_anagram_unicode("anagram", "nagaram"));
        assert!(!is_anagram_unicode("rat", "car"));
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
        //     let _ = is_anagram(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}