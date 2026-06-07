//! # Lesson 3: Property-Based Testing
//!
//! Property-based testing generates random inputs to find edge cases.
//! This lesson covers proptest strategies, prop_assume!, shrinking,
//! and custom generators.

use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Code under test
// ---------------------------------------------------------------------------

/// Sort a vector of integers.
pub fn bubble_sort(data: &mut [i32]) {
    let len = data.len();
    for i in 0..len {
        for j in 0..len - 1 - i {
            if data[j] > data[j + 1] {
                data.swap(j, j + 1);
            }
        }
    }
}

/// Reverse a string, preserving UTF-8 boundaries.
pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

/// Check if a number is prime.
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

/// Compute the greatest common divisor.
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// Normalize a URL path (remove redundant slashes, resolve . and ..).
pub fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    let result = parts.join("/");
    if path.starts_with('/') {
        format!("/{}", result)
    } else {
        result
    }
}

// ---------------------------------------------------------------------------
// Custom strategies
// ---------------------------------------------------------------------------

/// Strategy for generating sorted vectors.
fn sorted_vec_strategy() -> impl Strategy<Value = Vec<i32>> {
    prop::collection::vec(any::<i32>(), 0..100).prop_map(|mut v| {
        v.sort();
        v
    })
}

/// Strategy for generating non-empty ASCII strings.
fn ascii_string_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,100}"
}

/// Strategy for generating positive integers.
fn positive_int_strategy() -> impl Strategy<Value = u64> {
    1u64..10000
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Property: sorting produces a sorted output
    // -----------------------------------------------------------------------

    proptest! {
        #[test]
        fn sort_produces_sorted_output(mut data in prop::collection::vec(any::<i32>(), 0..1000)) {
            bubble_sort(&mut data);
            for i in 1..data.len() {
                prop_assert!(data[i - 1] <= data[i],
                    "not sorted at index {}: {} > {}", i, data[i-1], data[i]);
            }
        }

        #[test]
        fn sort_preserves_length(data in prop::collection::vec(any::<i32>(), 0..1000)) {
            let original_len = data.len();
            let mut sorted = data.clone();
            bubble_sort(&mut sorted);
            prop_assert_eq!(sorted.len(), original_len);
        }

        #[test]
        fn sort_preserves_elements(data in prop::collection::vec(any::<i32>(), 0..100)) {
            let mut sorted = data.clone();
            bubble_sort(&mut sorted);
            let mut original_sorted = data.clone();
            original_sorted.sort();
            prop_assert_eq!(sorted, original_sorted);
        }

        #[test]
        fn sort_idempotent(data in prop::collection::vec(any::<i32>(), 0..100)) {
            let mut sorted1 = data.clone();
            bubble_sort(&mut sorted1);
            let mut sorted2 = sorted1.clone();
            bubble_sort(&mut sorted2);
            prop_assert_eq!(sorted1, sorted2);
        }
    }

    // -----------------------------------------------------------------------
    // Property: reverse(reverse(x)) == x
    // -----------------------------------------------------------------------

    proptest! {
        #[test]
        fn reverse_twice_is_identity(s in ".*") {
            let reversed = reverse_string(&s);
            let double_reversed = reverse_string(&reversed);
            prop_assert_eq!(s, double_reversed);
        }

        #[test]
        fn reverse_preserves_length(s in ".*") {
            prop_assert_eq!(s.chars().count(), reverse_string(&s).chars().count());
        }
    }

    // -----------------------------------------------------------------------
    // Property: prime number properties
    // -----------------------------------------------------------------------

    proptest! {
        #[test]
        fn prime_properties(n in 2u64..100000) {
            if is_prime(n) {
                // A prime has no divisors other than 1 and itself
                for d in 2..n {
                    if d * d > n {
                        break;
                    }
                    prop_assert!(n % d != 0, "{} is divisible by {}", n, d);
                }
            }
        }

        #[test]
        fn composite_not_prime(
            a in 2u64..1000,
            b in 2u64..1000
        ) {
            prop_assume!(a != b || a > 1);
            let product = a * b;
            if product > 1 {
                // A product of two numbers > 1 should not be prime
                prop_assert!(!is_prime(product),
                    "{} * {} = {} should not be prime", a, b, product);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Property: GCD properties
    // -----------------------------------------------------------------------

    proptest! {
        #[test]
        fn gcd_divides_both(a in 1u64..10000, b in 1u64..10000) {
            let g = gcd(a, b);
            prop_assert_eq!(a % g, 0, "gcd({},{})={} does not divide {}", a, b, g, a);
            prop_assert_eq!(b % g, 0, "gcd({},{})={} does not divide {}", a, b, g, b);
        }

        #[test]
        fn gcd_commutative(a in 1u64..10000, b in 1u64..10000) {
            prop_assert_eq!(gcd(a, b), gcd(b, a));
        }

        #[test]
        fn gcd_with_zero(a in 0u64..10000) {
            prop_assert_eq!(gcd(a, 0), a);
            prop_assert_eq!(gcd(0, a), a);
        }
    }

    // -----------------------------------------------------------------------
    // Property: URL path normalization
    // -----------------------------------------------------------------------

    proptest! {
        #[test]
        fn normalize_removes_double_slashes(path in "/[a-z/]{1,50}") {
            let normalized = normalize_path(&path);
            assert!(!normalized.contains("//"));
        }

        #[test]
        fn normalize_preserves_leading_slash(path in "/[a-z/]{1,50}") {
            let normalized = normalize_path(&path);
            assert!(normalized.starts_with('/'), "path '{}' -> '{}'", path, normalized);
        }

        #[test]
        fn normalize_resolves_dotdot(path in "/[a-z/]{1,20}/\\.\\./[a-z/]{0,20}") {
            let normalized = normalize_path(&path);
            assert!(!normalized.contains("/../"), "still contains /../ in '{}'", normalized);
        }
    }

    // -----------------------------------------------------------------------
    // Using custom strategies
    // -----------------------------------------------------------------------

    proptest! {
        #[test]
        fn sorted_input_stays_sorted(data in sorted_vec_strategy()) {
            for i in 1..data.len() {
                assert!(data[i - 1] <= data[i]);
            }
        }

        #[test]
        fn ascii_string_properties(s in ascii_string_strategy()) {
            assert!(!s.is_empty());
            assert!(s.len() <= 100);
            assert!(s.is_ascii());
        }

        #[test]
        fn positive_int_is_positive(n in positive_int_strategy()) {
            assert!(n > 0);
        }
    }

    // -----------------------------------------------------------------------
    // prop_assume! usage
    // -----------------------------------------------------------------------

    proptest! {
        #[test]
        fn division_properties(a in 1i32..1000, b in 1i32..1000) {
            prop_assume!(b != 0);
            let result = a / b;
            // Integer division: result * b <= a
            prop_assert!(result * b <= a);
            // And result * b + remainder == a
            prop_assert_eq!(result * b + a % b, a);
        }
    }

    // -----------------------------------------------------------------------
    // Basic property tests (non-proptest for comparison)
    // -----------------------------------------------------------------------

    #[test]
    fn test_bubble_sort_basic() {
        let mut data = vec![5, 3, 1, 4, 2];
        bubble_sort(&mut data);
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_bubble_sort_empty() {
        let mut data: Vec<i32> = vec![];
        bubble_sort(&mut data);
        assert!(data.is_empty());
    }

    #[test]
    fn test_bubble_sort_single() {
        let mut data = vec![42];
        bubble_sort(&mut data);
        assert_eq!(data, vec![42]);
    }

    #[test]
    fn test_reverse_string_basic() {
        assert_eq!(reverse_string("hello"), "olleh");
        assert_eq!(reverse_string(""), "");
        assert_eq!(reverse_string("a"), "a");
    }

    #[test]
    fn test_is_prime() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(5));
        assert!(!is_prime(6));
        assert!(is_prime(7));
        assert!(!is_prime(9));
        assert!(is_prime(11));
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(7, 13), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(5, 0), 5);
        assert_eq!(gcd(0, 0), 0);
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/a/b/c"), "/a/b/c");
        assert_eq!(normalize_path("/a//b///c"), "/a/b/c");
        assert_eq!(normalize_path("/a/b/../c"), "/a/c");
        assert_eq!(normalize_path("/a/./b"), "/a/b");
    }
}
