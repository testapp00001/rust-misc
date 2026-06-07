//! # Property-Based Testing
//!
//! Property-based testing generates random inputs and verifies that certain
//! properties hold for all inputs. This finds edge cases that example-based
//! tests miss.
//!
//! Key concepts:
//! - Properties: invariants that should always hold
//! - Strategies: how to generate random inputs
//! - Shrinking: finding minimal failing cases
//! - Proptest and quickcheck patterns

use std::collections::HashMap;

/// Demonstrates property-based testing patterns.
/// In production code, you'd use `proptest` or `quickcheck` crate.
/// Here we implement the core concepts manually.

/// A trait for generating random values (simplified proptest Strategy).
pub trait Strategy {
    type Value;
    fn generate(&self, seed: u64) -> Self::Value;
}

/// A simple seeded RNG for deterministic testing.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        SimpleRng { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn next_range(&mut self, min: u64, max: u64) -> u64 {
        min + self.next_u64() % (max - min + 1)
    }
}

/// Property: sort should produce a sorted output.
pub fn is_sorted(data: &[i32]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}

/// Property: sort should be idempotent.
pub fn sort_idempotent(data: &[i32]) -> bool {
    let mut sorted1 = data.to_vec();
    sorted1.sort();
    let mut sorted2 = sorted1.clone();
    sorted2.sort();
    sorted1 == sorted2
}

/// Property: sort should preserve length.
pub fn sort_preserves_length(data: &[i32]) -> bool {
    let mut sorted = data.to_vec();
    sorted.sort();
    sorted.len() == data.len()
}

/// Property: sort should preserve elements (permutation).
pub fn sort_preserves_elements(data: &[i32]) -> bool {
    let mut sorted = data.to_vec();
    sorted.sort();
    let mut original_sorted = data.to_vec();
    original_sorted.sort();
    sorted == original_sorted
}

/// A simple sorting implementation for testing.
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

/// Property: reverse should be its own inverse.
pub fn reverse_involution(data: &[i32]) -> bool {
    let mut reversed = data.to_vec();
    reversed.reverse();
    reversed.reverse();
    reversed == data
}

/// Property: map then collect should preserve length.
pub fn map_preserves_length<F>(data: &[i32], f: F) -> bool
where
    F: Fn(i32) -> i32,
{
    let mapped: Vec<i32> = data.iter().copied().map(f).collect();
    mapped.len() == data.len()
}

/// Property: filter should not increase length.
pub fn filter_decreases_length<F>(data: &[i32], predicate: F) -> bool
where
    F: Fn(&i32) -> bool,
{
    let filtered: Vec<&i32> = data.iter().filter(|x| predicate(x)).collect();
    filtered.len() <= data.len()
}

/// Property: HashMap insert then get should return the inserted value.
pub fn hashmap_insert_get(key: &str, value: i32) -> bool {
    let mut map = HashMap::new();
    map.insert(key.to_string(), value);
    map.get(key) == Some(&value)
}

/// Property: Vec push then pop should return the pushed value.
pub fn vec_push_pop(values: &[i32]) -> bool {
    let mut stack = Vec::new();
    for v in values {
        stack.push(*v);
    }
    for v in values.iter().rev() {
        if stack.pop() != Some(*v) {
            return false;
        }
    }
    stack.is_empty()
}

/// Property: string operations should compose correctly.
pub fn string_reverse_reverse(s: &str) -> bool {
    let reversed: String = s.chars().rev().collect();
    let double_reversed: String = reversed.chars().rev().collect();
    s == double_reversed
}

/// Property: addition should be commutative.
pub fn addition_commutative(a: i32, b: i32) -> bool {
    a.wrapping_add(b) == b.wrapping_add(a)
}

/// Property: addition should be associative.
pub fn addition_associative(a: i32, b: i32, c: i32) -> bool {
    a.wrapping_add(b).wrapping_add(c) == a.wrapping_add(b.wrapping_add(c))
}

/// Demonstrates a property test runner that generates random inputs.
pub fn run_property_test<F, G>(name: &str, num_tests: usize, generator: G, property: F)
where
    F: Fn(&[i32]) -> bool,
    G: Fn(u64) -> Vec<i32>,
{
    for seed in 0..num_tests as u64 {
        let input = generator(seed);
        assert!(
            property(&input),
            "Property '{name}' failed for seed {seed}: {input:?}"
        );
    }
}

/// A generator for random integer vectors.
pub fn generate_vec(seed: u64) -> Vec<i32> {
    let mut rng = SimpleRng::new(seed);
    let len = rng.next_range(0, 20) as usize;
    (0..len)
        .map(|_| rng.next_range(0, 1000) as i32)
        .collect()
}

/// Demonstrates shrinking: finding the minimal failing case.
pub fn shrink_vec(input: &[i32]) -> Vec<Vec<i32>> {
    let mut candidates = Vec::new();

    // Try empty
    candidates.push(vec![]);

    // Try single elements
    for &elem in input {
        candidates.push(vec![elem]);
    }

    // Try removing each element
    for i in 0..input.len() {
        let mut shrunk = input.to_vec();
        shrunk.remove(i);
        candidates.push(shrunk);
    }

    // Try halving
    if input.len() > 1 {
        candidates.push(input[..input.len() / 2].to_vec());
        candidates.push(input[input.len() / 2..].to_vec());
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_properties() {
        let test_cases = vec![
            vec![],
            vec![1],
            vec![3, 1, 2],
            vec![5, 4, 3, 2, 1],
            vec![1, 2, 3, 4, 5],
            vec![3, 3, 3, 1, 1],
        ];

        for data in &test_cases {
            assert!(is_sorted(&{
                let mut v = data.clone();
                v.sort();
                v
            }));
            assert!(sort_idempotent(data));
            assert!(sort_preserves_length(data));
            assert!(sort_preserves_elements(data));
        }
    }

    #[test]
    fn test_bubble_sort_correctness() {
        let mut data = vec![5, 3, 1, 4, 2];
        bubble_sort(&mut data);
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
        assert!(is_sorted(&data));
    }

    #[test]
    fn test_reverse_involution() {
        let test_cases = vec![
            vec![],
            vec![1],
            vec![1, 2, 3],
            vec![5, 4, 3, 2, 1],
        ];
        for data in &test_cases {
            assert!(reverse_involution(data));
        }
    }

    #[test]
    fn test_map_preserves_length() {
        let data = vec![1, 2, 3, 4, 5];
        assert!(map_preserves_length(&data, |x| x * 2));
        assert!(map_preserves_length(&data, |x| x + 1));
        assert!(map_preserves_length(&data, |_| 0));
    }

    #[test]
    fn test_filter_decreases_length() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert!(filter_decreases_length(&data, |x| x % 2 == 0));
        assert!(filter_decreases_length(&data, |x| *x > 100));
        assert!(filter_decreases_length(&data, |_| true));
    }

    #[test]
    fn test_hashmap_insert_get() {
        assert!(hashmap_insert_get("key", 42));
        assert!(hashmap_insert_get("", 0));
        assert!(hashmap_insert_get("long_key_name", -1));
    }

    #[test]
    fn test_vec_push_pop() {
        assert!(vec_push_pop(&[]));
        assert!(vec_push_pop(&[1]));
        assert!(vec_push_pop(&[1, 2, 3]));
    }

    #[test]
    fn test_string_reverse() {
        assert!(string_reverse_reverse(""));
        assert!(string_reverse_reverse("a"));
        assert!(string_reverse_reverse("hello"));
        assert!(string_reverse_reverse("racecar"));
    }

    #[test]
    fn test_addition_properties() {
        assert!(addition_commutative(1, 2));
        assert!(addition_commutative(0, 0));
        assert!(addition_commutative(-5, 10));

        assert!(addition_associative(1, 2, 3));
        assert!(addition_associative(0, 0, 0));
    }

    #[test]
    fn test_property_with_generated_inputs() {
        run_property_test("sort_idempotent", 100, generate_vec, |data| {
            sort_idempotent(data)
        });
    }

    #[test]
    fn test_property_preserves_length() {
        run_property_test("sort_preserves_length", 100, generate_vec, |data| {
            sort_preserves_length(data)
        });
    }

    #[test]
    fn test_shrink() {
        let input = vec![5, 3, 1, 4, 2];
        let candidates = shrink_vec(&input);
        assert!(!candidates.is_empty());
        assert!(candidates.contains(&vec![]));
        assert!(candidates.contains(&vec![5]));
        assert!(candidates.contains(&vec![3, 1, 4, 2]));
    }
}
