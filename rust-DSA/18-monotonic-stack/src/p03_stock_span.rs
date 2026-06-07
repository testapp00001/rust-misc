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
    stack: Vec<(i32, i32)>, // (price, span)
}

impl StockSpanner {
    pub fn new() -> Self {
        StockSpanner { stack: Vec::new() }
    }

    pub fn next(&mut self, price: i32) -> i32 {
        let mut span = 1;

        while !self.stack.is_empty() && self.stack.last().unwrap().0 <= price {
            span += self.stack.pop().unwrap().1;
        }

        self.stack.push((price, span));
        span
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
}
