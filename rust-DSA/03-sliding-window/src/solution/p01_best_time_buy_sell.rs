// ============================================================================
// Problem: Best Time to Buy and Sell Stock (LeetCode #121)
// ============================================================================
// You are given an array `prices` where prices[i] is the price of a stock
// on the ith day. You want to maximize your profit by choosing a single day
// to buy and a single day to sell.
//
// Return the maximum profit. If you cannot achieve any profit, return 0.
//
// Example:
//   Input:  prices = [7,1,5,3,6,4]
//   Output: 5  (buy at 1, sell at 6)
//
// ============================================================================
// APPROACH: Track Minimum So Far (O(n) time, O(1) space)
// ============================================================================
//
// Single pass approach:
// 1. Track the minimum price seen so far.
// 2. For each price, calculate profit = price - min_so_far.
// 3. Track the maximum profit.
//
// This works because we're looking for the best day to sell, given the
// best (lowest) day to buy before it.
// ============================================================================

pub fn max_profit(prices: Vec<i32>) -> i32 {
    let mut min_price = i32::MAX;
    let mut max_profit = 0;

    for price in prices {
        min_price = min_price.min(price);
        max_profit = max_profit.max(price - min_price);
    }

    max_profit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(max_profit(vec![7, 1, 5, 3, 6, 4]), 5);
    }

    #[test]
    fn test_decreasing() {
        assert_eq!(max_profit(vec![7, 6, 4, 3, 1]), 0);
    }

    #[test]
    fn test_single() {
        assert_eq!(max_profit(vec![5]), 0);
    }

    #[test]
    fn test_two_days() {
        assert_eq!(max_profit(vec![1, 5]), 4);
    }

    #[test]
    fn test_increasing() {
        assert_eq!(max_profit(vec![1, 2, 3, 4, 5]), 4);
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
        //     let _ = max_profit(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}