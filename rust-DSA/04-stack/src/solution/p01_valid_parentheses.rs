// ============================================================================
// Problem: Valid Parentheses (LeetCode #20)
// ============================================================================
// Given a string `s` containing just the characters '(', ')', '{', '}',
// '[' and ']', determine if the input string is valid.
//
// A string is valid if:
// 1. Open brackets must be closed by the same type of brackets.
// 2. Open brackets must be closed in the correct order.
//
// Example:
//   Input:  "()"     → true
//   Input:  "()[]{}" → true
//   Input:  "(]"     → false
//
// ============================================================================
// APPROACH: Stack (O(n) time, O(n) space)
// ============================================================================
//
// 1. Iterate through each character.
// 2. If it's an opening bracket, push it onto the stack.
// 3. If it's a closing bracket:
//    - If stack is empty → invalid.
//    - If top of stack matches → pop.
//    - If top doesn't match → invalid.
// 4. At the end, stack must be empty.
//
// Rust-specific tips:
// - Use `Vec<char>` as a stack (push/pop).
// - `.last()` gives `Option<&char>` — use `.unwrap_or(&'\0')` for safety.
// ============================================================================

pub fn is_valid(s: &str) -> bool {
    let mut stack: Vec<char> = Vec::new();

    for c in s.chars() {
        match c {
            '(' | '{' | '[' => stack.push(c),
            ')' => {
                if stack.pop() != Some('(') {
                    return false;
                }
            }
            '}' => {
                if stack.pop() != Some('{') {
                    return false;
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return false;
                }
            }
            _ => return false, // Invalid character
        }
    }

    stack.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pair() {
        assert!(is_valid("()"));
    }

    #[test]
    fn test_multiple_types() {
        assert!(is_valid("()[]{}"));
    }

    #[test]
    fn test_mismatched() {
        assert!(!is_valid("(]"));
    }

    #[test]
    fn test_nested() {
        assert!(is_valid("{[()]}"));
    }

    #[test]
    fn test_empty() {
        assert!(is_valid(""));
    }

    #[test]
    fn test_closing_only() {
        assert!(!is_valid(")"));
    }

    #[test]
    fn test_opening_only() {
        assert!(!is_valid("("));
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
        //     let _ = is_valid(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}