// ============================================================================
// Problem: Valid Parenthesis String (LeetCode #678)
// ============================================================================
// Given a string `s` containing '(', ')', and '*', return true if the
// string is valid. '*' can be '(', ')', or empty.
//
// Example:
//   "()"   → true
//   "(*)"  → true
//   "(*))" → true
//
// ============================================================================
// APPROACH: Greedy with Range Tracking (O(n) time, O(1) space)
// ============================================================================
//
// Track the range of possible open parenthesis counts:
// - lo: minimum possible open count
// - hi: maximum possible open count
//
// For each char:
//   '(': lo++, hi++
//   ')': lo--, hi--
//   '*': lo--, hi++  (wildcard can be either)
//
// lo = max(lo, 0) — can't have negative open count.
// If hi < 0 → invalid.
// At the end, lo must be 0.
// ============================================================================

pub fn check_valid_string(s: &str) -> bool {
    let mut lo = 0;
    let mut hi = 0;

    for c in s.chars() {
        match c {
            '(' => {
                lo += 1;
                hi += 1;
            }
            ')' => {
                lo -= 1;
                hi -= 1;
            }
            '*' => {
                lo -= 1;
                hi += 1;
            }
            _ => {}
        }

        lo = lo.max(0);
        if hi < 0 {
            return false;
        }
    }

    lo == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert!(check_valid_string("()"));
    }

    #[test]
    fn test_wildcard() {
        assert!(check_valid_string("(*)"));
    }

    #[test]
    fn test_complex() {
        assert!(check_valid_string("(*))"));
    }

    #[test]
    fn test_invalid() {
        assert!(!check_valid_string("((*"));
    }

    #[test]
    fn test_empty() {
        assert!(check_valid_string(""));
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
        //     let _ = check_valid_string(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}