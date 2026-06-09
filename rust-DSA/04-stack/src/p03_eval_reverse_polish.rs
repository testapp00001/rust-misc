// ============================================================================
// Problem: Evaluate Reverse Polish Notation (LeetCode #150)
// ============================================================================
// Evaluate the value of an arithmetic expression in Reverse Polish Notation.
//
// Valid operators are +, -, *, and /. Each operand may be an integer or
// another expression. Division truncates toward zero.
//
// Example:
//   Input:  ["2","1","+","3","*"]
//   Output: 9  ((2 + 1) * 3 = 9)
//
// ============================================================================
// APPROACH: Stack (O(n) time, O(n) space)
// ============================================================================
//
// 1. Iterate through tokens.
// 2. If it's a number, push onto the stack.
// 3. If it's an operator:
//    - Pop two operands (note: second operand pops first!).
//    - Apply the operator.
//    - Push the result.
// 4. The final result is the only element left on the stack.
//
// Rust-specific tips:
// - `token.parse::<i32>()` returns `Result<i32, _>`.
// - `.unwrap()` is fine here since the problem guarantees valid input.
// ============================================================================



pub fn eval_rpn(tokens: Vec<&str>) -> i32 {
    todo!("Implement eval_rpn")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(eval_rpn(vec!["2", "1", "+", "3", "*"]), 9);
    }

    #[test]
    fn test_complex() {
        assert_eq!(eval_rpn(vec!["4", "13", "5", "/", "+"]), 6);
    }

    #[test]
    fn test_nested() {
        assert_eq!(
            eval_rpn(vec!["10", "6", "9", "3", "+", "-11", "*", "/", "*", "17", "+", "5", "+"]),
            22
        );
    }

    #[test]
    fn test_single() {
        assert_eq!(eval_rpn(vec!["42"]), 42);
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
        //     let _ = eval_rpn(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}