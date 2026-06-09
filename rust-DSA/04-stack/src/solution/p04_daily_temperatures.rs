// ============================================================================
// Problem: Daily Temperatures (LeetCode #739)
// ============================================================================
// Given an array of integers `temperatures` representing daily temperatures,
// return an array `answer` such that `answer[i]` is the number of days you
// have to wait after the ith day to get a warmer temperature.
//
// Example:
//   Input:  [73, 74, 75, 71, 69, 72, 76, 73]
//   Output: [1, 1, 4, 2, 1, 1, 0, 0]
//
// ============================================================================
// APPROACH: Monotonic Stack (O(n) time, O(n) space)
// ============================================================================
//
// Use a stack that stores indices of temperatures in decreasing order:
// 1. For each temperature, pop all stack elements with smaller temperature.
// 2. For each popped element, the answer is (current_index - popped_index).
// 3. Push the current index onto the stack.
//
// The stack maintains a decreasing sequence of temperatures. When we find
// a warmer day, it resolves all the waiting days.
//
// This is the classic "next greater element" pattern.
// ============================================================================

pub fn daily_temperatures(temperatures: Vec<i32>) -> Vec<i32> {
    let n = temperatures.len();
    let mut answer = vec![0; n];
    let mut stack: Vec<usize> = Vec::new(); // Stack of indices

    for i in 0..n {
        while !stack.is_empty() && temperatures[i] > temperatures[*stack.last().unwrap()] {
            let j = stack.pop().unwrap();
            answer[j] = (i - j) as i32;
        }
        stack.push(i);
    }

    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            daily_temperatures(vec![73, 74, 75, 71, 69, 72, 76, 73]),
            vec![1, 1, 4, 2, 1, 1, 0, 0]
        );
    }

    #[test]
    fn test_decreasing() {
        assert_eq!(daily_temperatures(vec![3, 2, 1]), vec![0, 0, 0]);
    }

    #[test]
    fn test_increasing() {
        assert_eq!(daily_temperatures(vec![1, 2, 3]), vec![1, 1, 0]);
    }

    #[test]
    fn test_single() {
        assert_eq!(daily_temperatures(vec![50]), vec![0]);
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
        //     let _ = daily_temperatures(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}