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
    let mut stack: Vec<i32> = Vec::new();

    for token in tokens {
        match token {
            "+" | "-" | "*" | "/" => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let result = match token {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => a / b,
                    _ => unreachable!(),
                };
                stack.push(result);
            }
            _ => {
                stack.push(token.parse().unwrap());
            }
        }
    }

    stack.pop().unwrap()
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
}
