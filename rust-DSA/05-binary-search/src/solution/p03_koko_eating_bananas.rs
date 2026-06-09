// ============================================================================
// Problem: Koko Eating Bananas (LeetCode #875)
// ============================================================================
// Koko loves to eat bananas. There are `n` piles of bananas, and the ith
// pile has `piles[i]` bananas. She can decide her eating speed `k`
// (bananas per hour). Each hour, she chooses a pile and eats k bananas.
// If the pile has less than k, she eats the whole pile.
//
// Return the minimum integer `k` such that she can eat all bananas within
// `h` hours.
//
// Example:
//   piles = [3,6,7,11], h = 8 → Output: 4
//
// ============================================================================
// APPROACH: Binary Search on Answer (O(n * log(max_pile)) time, O(1) space)
// ============================================================================
//
// The answer is between 1 and max(piles). Binary search for the minimum k:
// 1. For each candidate k, calculate total hours = sum(ceil(pile/k)).
// 2. If total hours <= h → k works, try smaller (right = mid).
// 3. If total hours > h → k too small, try larger (left = mid + 1).
//
// This is "binary search on the answer" — a common pattern!
// ============================================================================

pub fn min_eating_speed(piles: &[i32], h: i32) -> i32 {
    let mut left = 1;
    let mut right = *piles.iter().max().unwrap();

    while left < right {
        let mid = left + (right - left) / 2;
        let hours: i32 = piles.iter().map(|&p| (p + mid - 1) / mid).sum();

        if hours <= h {
            right = mid;
        } else {
            left = mid + 1;
        }
    }

    left
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(min_eating_speed(&[3, 6, 7, 11], 8), 4);
    }

    #[test]
    fn test_exact() {
        assert_eq!(min_eating_speed(&[30, 11, 23, 4, 20], 5), 30);
    }

    #[test]
    fn test_many_hours() {
        assert_eq!(min_eating_speed(&[30, 11, 23, 4, 20], 6), 23);
    }

    #[test]
    fn test_single_pile() {
        assert_eq!(min_eating_speed(&[100], 10), 10);
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
        //     let _ = min_eating_speed(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}