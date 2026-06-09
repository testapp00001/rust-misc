// ============================================================================
// Problem: Online Stock Span (LeetCode #901)
// ============================================================================
// Design an algorithm that collects daily stock prices and returns the
// span of the stock's price for the current day.
//
// The span is the maximum number of consecutive days (including today)
// where the price was <= today's price.
//
// Example:
//   next(100) → 1
//   next(80)  → 1
//   next(60)  → 1
//   next(70)  → 2
//   next(60)  → 1
//   next(75)  → 4
//   next(85)  → 6
//
// ============================================================================
// APPROACH: Monotonic Stack (O(1) amortized per next, O(n) space)
// ============================================================================
//
// Use a stack of (price, span) pairs in decreasing order of price:
// 1. For each new price, pop all elements with smaller or equal price.
// 2. Sum up their spans.
// 3. Push (price, total_span).
// ============================================================================


pub struct StockSpanner {
    // TODO: Define fields
}

impl StockSpanner {
    pub fn new() -> Self {
        todo!("Implement new")
    }

    pub fn next(&mut self, price: i32) -> i32 {
        todo!("Implement next")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut ss = StockSpanner::new();
        assert_eq!(ss.next(100), 1);
        assert_eq!(ss.next(80), 1);
        assert_eq!(ss.next(60), 1);
        assert_eq!(ss.next(70), 2);
        assert_eq!(ss.next(60), 1);
        assert_eq!(ss.next(75), 4);
        assert_eq!(ss.next(85), 6);
    }

    #[test]
    fn test_increasing() {
        let mut ss = StockSpanner::new();
        assert_eq!(ss.next(10), 1);
        assert_eq!(ss.next(20), 2);
        assert_eq!(ss.next(30), 3);
    }

    #[test]
    fn test_decreasing() {
        let mut ss = StockSpanner::new();
        assert_eq!(ss.next(30), 1);
        assert_eq!(ss.next(20), 1);
        assert_eq!(ss.next(10), 1);
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
        //     let _ = new(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}