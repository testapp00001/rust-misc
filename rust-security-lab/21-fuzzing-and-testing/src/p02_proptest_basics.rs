//! # Lesson 02: Property-Based Testing with proptest
//!
//! ## What is Property-Based Testing?
//!
//! Instead of testing specific examples, you define *properties* that must hold for ALL
//! valid inputs. The framework generates hundreds of random inputs and checks the property.
//!
//! ```
//! Traditional:  assert_eq!(add(2, 3), 5)
//! Property:     for all (a, b): add(a, b) == add(b, a)   // commutativity
//! ```
//!
//! ## proptest Strategies
//!
//! A *strategy* defines how to generate random values:
//! - `any::<u8>()` — any u8 value (0..=255)
//! - `"[a-z]{1,10}"` — regex-based string generation
//! - `prop::collection::vec(any::<u8>(), 0..100)` — vector of 0-100 bytes
//! - `prop_oneof!` — pick from multiple strategies
//!
//! ## Shrinking
//!
//! When a property fails, proptest automatically *shrinks* the input to the smallest
//! failing case. This makes debugging much easier than raw fuzzing.
//!
//! ```
//! Failing input:  [47, 203, 12, 0, 89, 255, 1, 3]
//! Shrunk input:   [0, 1]
//! ```
//!
//! ## Security Perspective
//!
//! ### Attack: Boundary Value Exploitation
//! Attackers target edge cases: empty strings, max-length buffers, zero-length arrays.
//! Property-based testing naturally explores these boundaries.
//!
//! ### Defense: Test Invariants, Not Examples
//! Security-critical properties:
//! - `encrypt` never panics on any input
//! - `decrypt(encrypt(x)) == x` for all x
//! - Hash output is always 32 bytes
//! - Encoding is always valid UTF-8

use proptest::prelude::*;

/// A function that adds two numbers but has a subtle bug.
///
/// The bug: when both inputs are 0, it returns 1 instead of 0.
///
/// Fix this function so it correctly adds all inputs.
pub fn buggy_add(a: u32, b: u32) -> u32 {
    if a == 0 && b == 0 {
        1 // BUG: should be 0
    } else {
        a.wrapping_add(b)
    }
}

/// Implement a circular buffer that stores up to `capacity` elements.
///
/// When full, pushing a new element overwrites the oldest.
///
/// Hints:
/// - Use a Vec<T> as backing store
/// - Track `head` index (where next push goes)
/// - Track `len` (how many elements are currently stored)
/// - `push`: write at head, advance head, cap len at capacity
/// - `pop_oldest`: read from (head - len), decrease len
/// - `to_vec`: return elements in insertion order (oldest first)
pub struct CircularBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    len: usize,
}

impl<T: Clone + std::fmt::Debug> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        todo!("Initialize circular buffer with given capacity")
    }

    pub fn push(&mut self, item: T) {
        todo!("Push item, overwriting oldest if full")
    }

    pub fn pop_oldest(&mut self) -> Option<T> {
        todo!("Pop the oldest element")
    }

    pub fn to_vec(&self) -> Vec<T> {
        todo!("Return elements in insertion order (oldest first)")
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// A simple linear congruential generator for deterministic pseudo-random numbers.
///
/// Formula: next = (a * current + c) % m
///
/// Implement with the following constants:
/// - a = 1664525
/// - c = 1013904223
/// - m = 2^32
///
/// The generator should produce the same sequence given the same seed.
pub struct LcgRng {
    state: u64,
}

impl LcgRng {
    pub fn new(seed: u64) -> Self {
        todo!("Initialize LCG with seed")
    }

    pub fn next_u32(&mut self) -> u32 {
        todo!("Generate next pseudo-random u32")
    }

    pub fn next_range(&mut self, min: u32, max: u32) -> u32 {
        todo!("Generate u32 in [min, max)")
    }
}

/// Validate that a string contains only allowed characters.
///
/// Returns true if all characters are in: a-z, A-Z, 0-9, '-', '_', '.'
///
/// This is a "username-safe" character validator.
pub fn is_safe_username(s: &str) -> bool {
    todo!("Validate username characters")
}

/// Normalize a path to prevent directory traversal attacks.
///
/// Rules:
/// - Remove "." components
/// - Process ".." by removing the parent component
/// - Remove leading "/" characters
/// - Remove consecutive "/" characters
/// - Return Err if ".." would escape the root
///
/// Examples:
/// - "a/b/c" -> "a/b/c"
/// - "a/../b" -> "b"
/// - "../a" -> Err
/// - "a//b" -> "a/b"
pub fn normalize_path(path: &str) -> Result<String, &'static str> {
    todo!("Implement path normalization")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- buggy_add tests ---

    #[test]
    fn test_add_basic() {
        assert_eq!(buggy_add(2, 3), 5);
        assert_eq!(buggy_add(0, 5), 5);
        assert_eq!(buggy_add(5, 0), 5);
        assert_eq!(buggy_add(0, 0), 0);
    }

    proptest! {
        #[test]
        fn test_add_commutative(a in any::<u32>(), b in any::<u32>()) {
            // Property: a + b == b + a (commutativity)
            prop_assert_eq!(buggy_add(a, b), buggy_add(b, a));
        }

        #[test]
        fn test_add_identity(a in any::<u32>()) {
            // Property: a + 0 == a (identity)
            prop_assert_eq!(buggy_add(a, 0), a);
        }

        #[test]
        fn test_add_associative(a in any::<u16>(), b in any::<u16>(), c in any::<u16>()) {
            // Property: (a + b) + c == a + (b + c) (associativity)
            let a = a as u32;
            let b = b as u32;
            let c = c as u32;
            prop_assert_eq!(
                buggy_add(buggy_add(a, b), c),
                buggy_add(a, buggy_add(b, c))
            );
        }
    }

    // --- CircularBuffer tests ---

    #[test]
    fn test_circular_buffer_basic() {
        let mut buf = CircularBuffer::new(3);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.to_vec(), vec![1, 2, 3]);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_circular_buffer_overwrite() {
        let mut buf = CircularBuffer::new(2);
        buf.push(1);
        buf.push(2);
        buf.push(3); // overwrites 1
        assert_eq!(buf.to_vec(), vec![2, 3]);
    }

    proptest! {
        #[test]
        fn test_circular_buffer_len_never_exceeds_capacity(
            ops in prop::collection::vec(any::<bool>(), 0..50)
        ) {
            let capacity = 5usize;
            let mut buf = CircularBuffer::new(capacity);
            for (i, push) in ops.iter().enumerate() {
                if *push {
                    buf.push(i);
                } else {
                    buf.pop_oldest();
                }
                prop_assert!(buf.len() <= capacity);
            }
        }
    }

    // --- LcgRng tests ---

    #[test]
    fn test_lcg_deterministic() {
        let mut rng1 = LcgRng::new(42);
        let mut rng2 = LcgRng::new(42);
        for _ in 0..100 {
            assert_eq!(rng1.next_u32(), rng2.next_u32());
        }
    }

    #[test]
    fn test_lcg_range() {
        let mut rng = LcgRng::new(123);
        for _ in 0..1000 {
            let v = rng.next_range(10, 20);
            assert!(v >= 10 && v < 20, "Value {} not in [10, 20)", v);
        }
    }

    // --- is_safe_username tests ---

    #[test]
    fn test_safe_username_valid() {
        assert!(is_safe_username("alice"));
        assert!(is_safe_username("Bob-123"));
        assert!(is_safe_username("user.name_42"));
    }

    #[test]
    fn test_safe_username_invalid() {
        assert!(!is_safe_username("")); // empty
        assert!(!is_safe_username("user name")); // space
        assert!(!is_safe_username("user@host")); // @
        assert!(!is_safe_username("../etc/passwd")); // traversal
    }

    proptest! {
        #[test]
        fn test_safe_username_no_special_chars(s in "[a-zA-Z0-9._-]{1,50}") {
            // Any string matching the safe pattern should be accepted
            prop_assert!(is_safe_username(&s));
        }
    }

    // --- normalize_path tests ---

    #[test]
    fn test_normalize_path_basic() {
        assert_eq!(normalize_path("a/b/c").unwrap(), "a/b/c");
    }

    #[test]
    fn test_normalize_path_traversal_blocked() {
        assert!(normalize_path("../etc/passwd").is_err());
        assert!(normalize_path("a/../../b").is_err());
    }

    #[test]
    fn test_normalize_path_dotdot_ok() {
        assert_eq!(normalize_path("a/b/../c").unwrap(), "a/c");
    }

    #[test]
    fn test_normalize_path_double_slash() {
        assert_eq!(normalize_path("a//b").unwrap(), "a/b");
    }

    proptest! {
        #[test]
        fn test_normalize_never_panics(s in ".*") {
            // Path normalization must never panic on arbitrary input
            let _ = normalize_path(&s);
        }

        #[test]
        fn test_normalize_no_double_slash(s in "[a-z/]{1,30}") {
            if let Ok(result) = normalize_path(&s) {
                prop_assert!(!result.contains("//"), "Double slash in: {}", result);
            }
        }
    }
}
