// ============================================================================
// Problem: Valid Palindrome (LeetCode #125)
// ============================================================================
// A phrase is a palindrome if, after converting all uppercase letters to
// lowercase and removing all non-alphanumeric characters, it reads the same
// forward and backward.
//
// Example:
//   "A man, a plan, a canal: Panama" → true
//   "race a car" → false
//
// ============================================================================
// APPROACH: Two Pointers (O(n) time, O(1) space)
// ============================================================================
//
// Use two pointers from both ends:
// 1. Left pointer starts at the beginning, right at the end.
// 2. Skip non-alphanumeric characters.
// 3. Compare characters (case-insensitive).
// 4. If they match, move both pointers inward.
// 5. If they don't match, return false.
//
// Rust-specific tips:
// - `char::is_alphanumeric()` checks if a character is a letter or digit.
// - `char::to_ascii_lowercase()` converts to lowercase (fast, ASCII-only).
// - `.chars()` gives an iterator over characters.
// ============================================================================

pub fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut left = 0;
    let mut right = chars.len();

    while left < right {
        // Move left pointer to next alphanumeric char
        while left < right && !chars[left].is_alphanumeric() {
            left += 1;
        }
        // Move right pointer to previous alphanumeric char
        while left < right && !chars[right - 1].is_alphanumeric() {
            right -= 1;
        }

        if left < right {
            if chars[left].to_ascii_lowercase() != chars[right - 1].to_ascii_lowercase() {
                return false;
            }
            left += 1;
            right -= 1;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palindrome_with_special_chars() {
        assert!(is_palindrome("A man, a plan, a canal: Panama"));
    }

    #[test]
    fn test_not_palindrome() {
        assert!(!is_palindrome("race a car"));
    }

    #[test]
    fn test_empty_string() {
        assert!(is_palindrome(""));
    }

    #[test]
    fn test_only_special_chars() {
        assert!(is_palindrome(".,;:!@#$%"));
    }

    #[test]
    fn test_single_char() {
        assert!(is_palindrome("a"));
    }

    #[test]
    fn test_numeric_palindrome() {
        assert!(is_palindrome("12321"));
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
        //     let _ = is_palindrome(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}