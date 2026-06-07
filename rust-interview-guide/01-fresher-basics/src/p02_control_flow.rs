/// Problem: Control Flow
///
/// Master Rust's control flow: if, match, loops, and iterators.
///
/// Key Concepts:
/// - if as expression
/// - match with patterns
/// - loop, while, for
/// - break and continue
/// - Loop labels

/// Problem 1: if as expression
/// Use if to return a value
pub fn absolute_value(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

/// Problem 2: match with ranges
/// Match a score to a grade
pub fn score_to_grade(score: u32) -> char {
    match score {
        90..=100 => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        60..=69 => 'D',
        _ => 'F',
    }
}

/// Problem 3: match with Option
/// Unwrap an Option with a default
pub fn unwrap_or_default(value: Option<i32>) -> i32 {
    match value {
        Some(x) => x,
        None => 0,
    }
}

/// Problem 4: match with multiple patterns
/// Check if a character is a vowel
pub fn is_vowel(c: char) -> bool {
    match c {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        'A' | 'E' | 'I' | 'O' | 'U' => true,
        _ => false,
    }
}

/// Problem 5: match with guards
/// Match with additional conditions
pub fn classify_number(x: i32) -> &'static str {
    match x {
        x if x > 0 => "positive",
        x if x < 0 => "negative",
        _ => "zero",
    }
}

/// Problem 6: loop with break value
/// Use loop to find the first perfect square
pub fn find_first_perfect_square(start: i32) -> i32 {
    let mut n = start;
    loop {
        let sqrt = (n as f64).sqrt() as i32;
        if sqrt * sqrt == n {
            break n;
        }
        n += 1;
    }
}

/// Problem 7: while loop
/// Sum digits of a number
pub fn sum_digits(mut n: u32) -> u32 {
    let mut sum = 0;
    while n > 0 {
        sum += n % 10;
        n /= 10;
    }
    sum
}

/// Problem 8: for loop with range
/// Calculate factorial
pub fn factorial(n: u32) -> u64 {
    let mut result: u64 = 1;
    for i in 1..=n as u64 {
        result *= i;
    }
    result
}

/// Problem 9: for loop with iterator
/// Collect even numbers
pub fn collect_evens(limit: u32) -> Vec<u32> {
    (0..limit).filter(|x| x % 2 == 0).collect()
}

/// Problem 10: Nested loops with labels
/// Find two numbers that sum to target
pub fn find_pair_sum(arr: &[i32], target: i32) -> Option<(usize, usize)> {
    for i in 0..arr.len() {
        for j in (i + 1)..arr.len() {
            if arr[i] + arr[j] == target {
                return Some((i, j));
            }
        }
    }
    None
}

/// Problem 11: loop with continue
/// Sum only positive numbers
pub fn sum_positives(arr: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in arr {
        if x <= 0 {
            continue;
        }
        sum += x;
    }
    sum
}

/// Problem 12: while let
/// Pop from a vector until empty
pub fn pop_all(mut v: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    while let Some(x) = v.pop() {
        result.push(x);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absolute_value() {
        assert_eq!(absolute_value(5), 5);
        assert_eq!(absolute_value(-5), 5);
        assert_eq!(absolute_value(0), 0);
    }

    #[test]
    fn test_score_to_grade() {
        assert_eq!(score_to_grade(95), 'A');
        assert_eq!(score_to_grade(85), 'B');
        assert_eq!(score_to_grade(75), 'C');
        assert_eq!(score_to_grade(65), 'D');
        assert_eq!(score_to_grade(55), 'F');
    }

    #[test]
    fn test_unwrap_or_default() {
        assert_eq!(unwrap_or_default(Some(42)), 42);
        assert_eq!(unwrap_or_default(None), 0);
    }

    #[test]
    fn test_is_vowel() {
        assert!(is_vowel('a'));
        assert!(is_vowel('E'));
        assert!(!is_vowel('b'));
    }

    #[test]
    fn test_classify_number() {
        assert_eq!(classify_number(5), "positive");
        assert_eq!(classify_number(-5), "negative");
        assert_eq!(classify_number(0), "zero");
    }

    #[test]
    fn test_find_first_perfect_square() {
        assert_eq!(find_first_perfect_square(1), 1);
        assert_eq!(find_first_perfect_square(2), 4);
        assert_eq!(find_first_perfect_square(5), 9);
    }

    #[test]
    fn test_sum_digits() {
        assert_eq!(sum_digits(123), 6);
        assert_eq!(sum_digits(999), 27);
        assert_eq!(sum_digits(0), 0);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
    }

    #[test]
    fn test_collect_evens() {
        assert_eq!(collect_evens(10), vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_find_pair_sum() {
        let arr = vec![1, 2, 3, 4, 5];
        let result = find_pair_sum(&arr, 6);
        assert!(result.is_some()); // Could be (0,4) or (1,3)
        assert_eq!(find_pair_sum(&arr, 10), None);
    }

    #[test]
    fn test_sum_positives() {
        assert_eq!(sum_positives(&[1, -2, 3, -4, 5]), 9);
    }

    #[test]
    fn test_pop_all() {
        let v = vec![1, 2, 3];
        assert_eq!(pop_all(v), vec![3, 2, 1]);
    }
}
