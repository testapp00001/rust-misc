//! # Exercise: G-Set (Grow-only Set)
//!
//! ## Theory
//!
//! A G-Set (Grow-only Set) is a CRDT that supports only adding elements.
//! Merging two G-Sets is simply taking the union. Elements can never be removed.
//!
//! Properties:
//! - Union is commutative: A U B = B U A
//! - Union is associative: (A U B) U C = A U (B U C)
//! - Union is idempotent: A U A = A
//!
//! ## Proof / Intuition
//!
//! Set union satisfies all three semilattice properties. Since elements can
//! only be added (never removed), there are no conflict scenarios. The
//! grow-only restriction ensures monotonic growth.
//!
//! ## Implementation Task
//!
//! Implement `GSet<T>` with:
//! - `add(value)` - add an element
//! - `contains(value)` - check membership
//! - `merge(other)` - union of both sets
//!
//! ## Verification
//!
//! Verify add works, merge is union, merge is idempotent.
//!
use std::collections::HashSet;
use std::hash::Hash;

/// A Grow-only Set CRDT.
///
/// Elements can only be added, never removed. Merging is set union.
#[derive(Debug, Clone)]
pub struct GSet<T: Eq + Hash + Clone> {
    elements: HashSet<T>,
}

impl<T: Eq + Hash + Clone> GSet<T> {
    /// Create a new empty G-Set.
    pub fn new() -> Self {
        Self {
            elements: HashSet::new(),
        }
    }

    /// Create a G-Set from an initial set of elements.
    pub fn from_elements(elements: Vec<T>) -> Self {
        Self {
            elements: elements.into_iter().collect(),
        }
    }

    /// Add an element to the set.
    pub fn add(&mut self, value: T) {
        self.elements.insert(value);
    }

    /// Check if an element is in the set.
    pub fn contains(&self, value: &T) -> bool {
        self.elements.contains(value)
    }

    /// Merge with another G-Set (set union).
    pub fn merge(&mut self, other: &GSet<T>) {
        for element in &other.elements {
            self.elements.insert(element.clone());
        }
    }

    /// Get the number of elements.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get all elements as a vector.
    pub fn elements(&self) -> Vec<&T> {
        self.elements.iter().collect()
    }
}

impl<T: Eq + Hash + Clone> Default for GSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + Clone> PartialEq for GSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl<T: Eq + Hash + Clone> Eq for GSet<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_contains() {
        let mut set = GSet::new();
        set.add(1);
        set.add(2);
        set.add(3);

        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert!(!set.contains(&4));
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_merge_is_union() {
        let mut a = GSet::from_elements(vec![1, 2, 3]);
        let b = GSet::from_elements(vec![3, 4, 5]);

        a.merge(&b);

        assert!(a.contains(&1));
        assert!(a.contains(&2));
        assert!(a.contains(&3));
        assert!(a.contains(&4));
        assert!(a.contains(&5));
        assert_eq!(a.len(), 5);
    }

    #[test]
    fn test_merge_is_idempotent() {
        let mut a = GSet::from_elements(vec![1, 2, 3]);
        let original = a.clone();

        a.merge(&original.clone());

        assert_eq!(a, original, "Merge with self must be idempotent");
    }

    #[test]
    fn test_merge_is_commutative() {
        let mut a = GSet::from_elements(vec![1, 2]);
        let mut b = GSet::from_elements(vec![2, 3]);

        let a_clone = a.clone();
        let b_clone = b.clone();

        a.merge(&b_clone);
        let mut ab_elements: Vec<&i32> = a.elements();
        ab_elements.sort();

        b.merge(&a_clone);
        let mut ba_elements: Vec<&i32> = b.elements();
        ba_elements.sort();

        assert_eq!(ab_elements, ba_elements, "Merge must be commutative");
    }

    #[test]
    fn test_string_elements() {
        let mut set = GSet::new();
        set.add("hello".to_string());
        set.add("world".to_string());

        assert!(set.contains(&"hello".to_string()));
        assert!(set.contains(&"world".to_string()));
        assert!(!set.contains(&"foo".to_string()));
    }

    #[test]
    fn test_empty_set_merge() {
        let mut a = GSet::from_elements(vec![1, 2]);
        let b: GSet<i32> = GSet::new();

        a.merge(&b);
        assert_eq!(a.len(), 2);
    }
}
