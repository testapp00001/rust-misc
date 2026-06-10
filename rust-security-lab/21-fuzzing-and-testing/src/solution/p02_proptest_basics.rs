//! # Lesson 02: Property-Based Testing with proptest (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use proptest::prelude::*;

/// A function that adds two numbers -- fixed version.
pub fn buggy_add(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}

/// A circular buffer that stores up to `capacity` elements.
///
/// When full, pushing a new element overwrites the oldest.
pub struct CircularBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    len: usize,
}

impl<T: Clone + std::fmt::Debug> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }
        Self {
            buffer,
            capacity,
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    pub fn pop_oldest(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let oldest_idx = (self.head + self.capacity - self.len) % self.capacity;
        self.len -= 1;
        self.buffer[oldest_idx].take()
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len);
        let start = (self.head + self.capacity - self.len) % self.capacity;
        for i in 0..self.len {
            let idx = (start + i) % self.capacity;
            if let Some(ref item) = self.buffer[idx] {
                result.push(item.clone());
            }
        }
        result
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// A simple linear congruential generator for deterministic pseudo-random numbers.
pub struct LcgRng {
    state: u64,
}

impl LcgRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        // LCG constants from Numerical Recipes
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state >> 16) as u32
    }

    pub fn next_range(&mut self, min: u32, max: u32) -> u32 {
        let range = max - min;
        min + self.next_u32() % range
    }
}

/// Validate that a string contains only allowed characters.
///
/// Allowed: a-z, A-Z, 0-9, '-', '_', '.'
pub fn is_safe_username(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Normalize a path to prevent directory traversal attacks.
pub fn normalize_path(path: &str) -> Result<String, &'static str> {
    let components: Vec<&str> = path.split('/').collect();
    let mut result: Vec<&str> = Vec::new();

    for component in components {
        match component {
            "" | "." => continue,
            ".." => {
                if result.pop().is_none() {
                    return Err("directory traversal escapes root");
                }
            }
            _ => result.push(component),
        }
    }

    Ok(result.join("/"))
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
            prop_assert_eq!(buggy_add(a, b), buggy_add(b, a));
        }

        #[test]
        fn test_add_identity(a in any::<u32>()) {
            prop_assert_eq!(buggy_add(a, 0), a);
        }

        #[test]
        fn test_add_associative(a in any::<u16>(), b in any::<u16>(), c in any::<u16>()) {
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
        assert!(!is_safe_username(""));
        assert!(!is_safe_username("user name"));
        assert!(!is_safe_username("user@host"));
        assert!(!is_safe_username("../etc/passwd"));
    }

    proptest! {
        #[test]
        fn test_safe_username_no_special_chars(s in "[a-zA-Z0-9._-]{1,50}") {
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
