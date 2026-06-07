//! # Test Organization
//!
//! Good test organization makes tests maintainable and discoverable.
//! This lesson covers patterns for organizing tests in Rust projects.

/// Demonstrates organizing tests by functionality.
pub mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn multiply(a: i32, b: i32) -> i32 {
        a * b
    }

    pub fn factorial(n: u64) -> u64 {
        match n {
            0 | 1 => 1,
            _ => n * factorial(n - 1),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_add_positive() {
            assert_eq!(add(2, 3), 5);
        }

        #[test]
        fn test_add_negative() {
            assert_eq!(add(-1, -2), -3);
        }

        #[test]
        fn test_multiply() {
            assert_eq!(multiply(3, 4), 12);
        }

        #[test]
        fn test_factorial() {
            assert_eq!(factorial(0), 1);
            assert_eq!(factorial(1), 1);
            assert_eq!(factorial(5), 120);
            assert_eq!(factorial(10), 3628800);
        }
    }
}

/// Demonstrates organizing tests by feature.
pub mod string_utils {
    pub fn reverse(s: &str) -> String {
        s.chars().rev().collect()
    }

    pub fn is_palindrome(s: &str) -> bool {
        let cleaned: String = s
            .chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| c.to_lowercase().next().unwrap())
            .collect();
        cleaned == reverse(&cleaned)
    }

    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                let upper: String = first.to_uppercase().collect();
                upper + chars.as_str()
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_reverse() {
            assert_eq!(reverse("hello"), "olleh");
            assert_eq!(reverse(""), "");
            assert_eq!(reverse("a"), "a");
        }

        #[test]
        fn test_is_palindrome() {
            assert!(is_palindrome("racecar"));
            assert!(is_palindrome("A man a plan a canal Panama"));
            assert!(!is_palindrome("hello"));
        }

        #[test]
        fn test_capitalize() {
            assert_eq!(capitalize("hello"), "Hello");
            assert_eq!(capitalize(""), "");
            assert_eq!(capitalize("HELLO"), "HELLO");
        }
    }
}

/// Demonstrates test helpers module.
#[cfg(test)]
pub mod test_helpers {
    use std::collections::HashMap;

    pub fn create_test_map() -> HashMap<String, i32> {
        let mut map = HashMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        map.insert("c".to_string(), 3);
        map
    }

    pub fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!(
            (a - b).abs() < epsilon,
            "assertion failed: {a} is not approximately equal to {b} (epsilon = {epsilon})"
        );
    }
}

/// Demonstrates a reusable test trait.
pub trait Testable {
    fn is_valid(&self) -> bool;
    fn validate(&self) -> Result<(), String>;
}

#[derive(Debug)]
pub struct Email(pub String);

impl Testable for Email {
    fn is_valid(&self) -> bool {
        self.0.contains('@') && self.0.contains('.')
    }

    fn validate(&self) -> Result<(), String> {
        if !self.0.contains('@') {
            return Err("missing @".to_string());
        }
        if !self.0.contains('.') {
            return Err("missing domain".to_string());
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Url(pub String);

impl Testable for Url {
    fn is_valid(&self) -> bool {
        self.0.starts_with("http://") || self.0.starts_with("https://")
    }

    fn validate(&self) -> Result<(), String> {
        if !self.is_valid() {
            return Err("must start with http:// or https://".to_string());
        }
        Ok(())
    }
}

/// Demonstrates test naming conventions.
pub struct Calculator;

impl Calculator {
    pub fn evaluate(expr: &str) -> Result<f64, String> {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() != 3 {
            return Err("expected 'a op b'".to_string());
        }

        let a: f64 = parts[0].parse().map_err(|_| "invalid number")?;
        let b: f64 = parts[2].parse().map_err(|_| "invalid number")?;

        match parts[1] {
            "+" => Ok(a + b),
            "-" => Ok(a - b),
            "*" => Ok(a * b),
            "/" => {
                if b == 0.0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(a / b)
                }
            }
            op => Err(format!("unknown operator: {op}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helpers::*;

    // Test naming convention: test_<what>_<condition>_<expected>
    #[test]
    fn test_calculator_addition_returns_sum() {
        assert_eq!(Calculator::evaluate("2 + 3").unwrap(), 5.0);
    }

    #[test]
    fn test_calculator_subtraction_returns_difference() {
        assert_eq!(Calculator::evaluate("10 - 4").unwrap(), 6.0);
    }

    #[test]
    fn test_calculator_multiplication_returns_product() {
        assert_eq!(Calculator::evaluate("3 * 4").unwrap(), 12.0);
    }

    #[test]
    fn test_calculator_division_returns_quotient() {
        assert_eq!(Calculator::evaluate("10 / 2").unwrap(), 5.0);
    }

    #[test]
    fn test_calculator_division_by_zero_returns_error() {
        assert!(Calculator::evaluate("1 / 0").is_err());
    }

    #[test]
    fn test_calculator_invalid_expression_returns_error() {
        assert!(Calculator::evaluate("invalid").is_err());
    }

    #[test]
    fn test_email_valid() {
        assert!(Email("user@example.com".to_string()).is_valid());
    }

    #[test]
    fn test_email_invalid_no_at() {
        assert!(!Email("user.example.com".to_string()).is_valid());
    }

    #[test]
    fn test_url_valid() {
        assert!(Url("https://example.com".to_string()).is_valid());
    }

    #[test]
    fn test_url_invalid() {
        assert!(!Url("ftp://example.com".to_string()).is_valid());
    }

    #[test]
    fn test_approx_eq() {
        assert_approx_eq(1.0, 1.00001, 0.001);
    }

    #[test]
    fn test_create_test_map() {
        let map = create_test_map();
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("a"), Some(&1));
    }
}
