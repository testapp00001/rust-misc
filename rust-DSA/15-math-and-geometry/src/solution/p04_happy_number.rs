// ============================================================================
// Problem: Happy Number (LeetCode #202)
// ============================================================================
// A happy number eventually reaches 1 when replaced by the sum of squares
// of its digits. Return true if n is a happy number.
//
// Example:
//   19 → 1² + 9² = 82 → 8² + 2² = 68 → 6² + 8² = 100 → 1² + 0² + 0² = 1
//   Output: true
//
// ============================================================================
// APPROACH: Floyd's Cycle Detection (O(log n) time, O(1) space)
// ============================================================================
//
// Use slow and fast pointers to detect cycles:
// - If we reach 1 → happy.
// - If slow == fast (cycle) → not happy.
// ============================================================================

pub fn is_happy(n: i32) -> bool {
    let mut slow = n;
    let mut fast = n;

    loop {
        slow = digit_square_sum(slow);
        fast = digit_square_sum(digit_square_sum(fast));

        if fast == 1 {
            return true;
        }
        if slow == fast {
            return false;
        }
    }
}

fn digit_square_sum(mut n: i32) -> i32 {
    let mut sum = 0;
    while n > 0 {
        let digit = n % 10;
        sum += digit * digit;
        n /= 10;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy() {
        assert!(is_happy(19));
    }

    #[test]
    fn test_not_happy() {
        assert!(!is_happy(2));
    }

    #[test]
    fn test_one() {
        assert!(is_happy(1));
    }

    #[test]
    fn test_large() {
        assert!(is_happy(100));
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
        //     let _ = is_happy(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}