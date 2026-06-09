// ============================================================================
// Problem: Letter Combinations of a Phone Number (LeetCode #17)
// ============================================================================
// Given a string of digits from 2-9, return all possible letter combinations
// that the number could represent (like on a phone keypad).
//
// Example:
//   Input:  "23"
//   Output: ["ad","ae","af","bd","be","bf","cd","ce","cf"]
//
// ============================================================================
// APPROACH: Backtracking (O(4^n * n) time, O(n) space)
// ============================================================================
//
// Map each digit to its letters, then generate all combinations.
// ============================================================================

const PHONE: [&str; 10] = ["", "", "abc", "def", "ghi", "jkl", "mno", "pqrs", "tuv", "wxyz"];

pub fn letter_combinations(digits: &str) -> Vec<String> {
    if digits.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();
    let mut current = String::new();
    let digits: Vec<char> = digits.chars().collect();
    backtrack(&digits, 0, &mut current, &mut result);
    result
}

fn backtrack(digits: &[char], idx: usize, current: &mut String, result: &mut Vec<String>) {
    if idx == digits.len() {
        result.push(current.clone());
        return;
    }

    let digit = digits[idx] as usize - '0' as usize;
    for c in PHONE[digit].chars() {
        current.push(c);
        backtrack(digits, idx + 1, current, result);
        current.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut v: Vec<String>) -> Vec<String> {
        v.sort();
        v
    }

    #[test]
    fn test_basic() {
        let result = sorted(letter_combinations("23"));
        let expected = sorted(vec![
            "ad".to_string(), "ae".to_string(), "af".to_string(),
            "bd".to_string(), "be".to_string(), "bf".to_string(),
            "cd".to_string(), "ce".to_string(), "cf".to_string(),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty() {
        assert_eq!(letter_combinations(""), Vec::<String>::new());
    }

    #[test]
    fn test_single() {
        let result = sorted(letter_combinations("2"));
        assert_eq!(result, sorted(vec!["a".to_string(), "b".to_string(), "c".to_string()]));
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
        //     let _ = letter_combinations(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}