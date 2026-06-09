// ============================================================================
// Problem: Plus One (LeetCode #66)
// ============================================================================
// Given a large integer represented as an array of digits, increment it by
// one and return the resulting array.
//
// Example:
//   [1,2,3] → [1,2,4]
//   [9,9,9] → [1,0,0,0]
//
// ============================================================================
// APPROACH: Right to Left with Carry (O(n) time, O(1) space)
// ============================================================================
//
// Process digits from right to left:
// 1. Add 1 to the last digit.
// 2. If it becomes 10, set to 0 and carry 1.
// 3. Continue until no carry.
// 4. If carry remains, prepend 1.
// ============================================================================

pub fn plus_one(mut digits: Vec<i32>) -> Vec<i32> {
    let n = digits.len();

    for i in (0..n).rev() {
        if digits[i] < 9 {
            digits[i] += 1;
            return digits;
        }
        digits[i] = 0;
    }

    // All digits were 9
    let mut result = vec![0; n + 1];
    result[0] = 1;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(plus_one(vec![1, 2, 3]), vec![1, 2, 4]);
    }

    #[test]
    fn test_carry() {
        assert_eq!(plus_one(vec![9, 9, 9]), vec![1, 0, 0, 0]);
    }

    #[test]
    fn test_single() {
        assert_eq!(plus_one(vec![0]), vec![1]);
    }

    #[test]
    fn test_no_carry() {
        assert_eq!(plus_one(vec![1, 2, 9]), vec![1, 3, 0]);
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
        //     let _ = plus_one(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}