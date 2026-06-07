/// Problem: Variables and Types
///
/// Understand Rust's variable system, type inference, and basic types.
///
/// Key Concepts:
/// - Immutability by default
/// - Mutability with `mut`
/// - Shadowing
/// - Type inference vs explicit types
/// - Scalar and compound types

/// Problem 1: Fix the code to make it compile
/// The variable should be mutable so we can change its value
pub fn fix_mutability() -> i32 {
    let mut x = 5;
    x = 10;
    x
}

/// Problem 2: Use shadowing to change the type
/// Start with a string, convert to number, then add 10
pub fn shadowing_example() -> i32 {
    let x = "42";
    let x: i32 = x.parse().unwrap();
    x + 10
}

/// Problem 3: Type inference
/// The compiler can infer the type from context
pub fn type_inference() -> Vec<i32> {
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    v
}

/// Problem 4: Explicit type annotation
/// Sometimes you need to help the compiler
pub fn explicit_type() -> Vec<f64> {
    let v: Vec<f64> = Vec::new();
    v
}

/// Problem 5: Constants
/// Constants must have explicit type and are always immutable
pub const MAX_SIZE: usize = 100;

pub fn use_constant() -> usize {
    MAX_SIZE * 2
}

/// Problem 6: Tuple destructuring
/// Extract values from a tuple
pub fn tuple_destructuring() -> (i32, f64, bool) {
    let t = (42, 3.14, true);
    let (a, b, c) = t;
    (a, b, c)
}

/// Problem 7: Array basics
/// Create and access arrays
pub fn array_basics() -> [i32; 5] {
    let arr = [1, 2, 3, 4, 5];
    arr
}

/// Problem 8: String vs &str
/// Understand the difference between owned and borrowed strings
pub fn string_types() -> (String, &'static str) {
    let owned = String::from("hello");
    let borrowed = "world";
    (owned, borrowed)
}

/// Problem 9: Type casting
/// Convert between numeric types
pub fn type_casting() -> f64 {
    let x: i32 = 42;
    x as f64
}

/// Problem 10: Overflow handling
/// Handle integer overflow safely
pub fn safe_add(a: i32, b: i32) -> Option<i32> {
    a.checked_add(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_mutability() {
        assert_eq!(fix_mutability(), 10);
    }

    #[test]
    fn test_shadowing_example() {
        assert_eq!(shadowing_example(), 52);
    }

    #[test]
    fn test_type_inference() {
        let v = type_inference();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_explicit_type() {
        let v = explicit_type();
        assert!(v.is_empty());
    }

    #[test]
    fn test_use_constant() {
        assert_eq!(use_constant(), 200);
    }

    #[test]
    fn test_tuple_destructuring() {
        let (a, b, c) = tuple_destructuring();
        assert_eq!(a, 42);
        assert!((b - 3.14).abs() < f64::EPSILON);
        assert!(c);
    }

    #[test]
    fn test_array_basics() {
        let arr = array_basics();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[4], 5);
    }

    #[test]
    fn test_string_types() {
        let (owned, borrowed) = string_types();
        assert_eq!(owned, "hello");
        assert_eq!(borrowed, "world");
    }

    #[test]
    fn test_type_casting() {
        let result = type_casting();
        assert!((result - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_safe_add() {
        assert_eq!(safe_add(100, 200), Some(300));
        assert_eq!(safe_add(i32::MAX, 1), None);
    }
}
