//! # Exercise: OR-Set (Observed-Remove Set)
//!
//! ## Theory
//!
//! An OR-Set (Observed-Remove Set) extends the G-Set to support removal.
//! Each add operation generates a unique tag for the element. Remove deletes
//! all currently observed tags for that element.
//!
//! The key insight is handling concurrent add and remove:
//! - If replica A adds "x" and replica B removes "x" concurrently,
//!   the add wins (the tag from A was not in B's observed set).
//! - This provides "add-wins" semantics.
//!
//! ## Proof / Intuition
//!
//! Each element is stored with a set of unique tags. When merging:
//! - Union all tags for each element
//! - Remove tags that appear in the removed set
//! - An element exists if it has any tags not in the removed set
//!
//! Concurrent add/remove: The add generates a new tag not known to the
//! remover, so the tag survives the merge. The remove only removes tags
//! it observed (and their causal descendants).
//!
//! ## Implementation Task
//!
//! Implement `ORSet<T>` with:
//! - `add(value)` - add with unique tag
//! - `remove(value)` - remove all observed tags
//! - `contains(value)` - check if value has surviving tags
//! - `merge(other)` - merge tags and removals
//!
//! ## Verification
//!
//! Verify add/remove work, concurrent add-wins, merge properties.
//!
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global unique tag counter.
static TAG_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A unique tag for an element addition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqueTag(u64);

impl UniqueTag {
    /// Generate a new unique tag.
    pub fn new() -> Self {
        Self(TAG_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// An Observed-Remove Set CRDT.
///
/// Supports add and remove operations with add-wins semantics for
/// concurrent add/remove conflicts.
#[derive(Debug, Clone)]
pub struct ORSet<T: Eq + Hash + Clone> {
    /// Element -> set of tags that were added.
    elements: HashMap<T, HashSet<UniqueTag>>,
    /// Tags that have been removed.
    removed: HashSet<UniqueTag>,
}

impl<T: Eq + Hash + Clone> ORSet<T> {
    /// Create a new empty OR-Set.
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            removed: HashSet::new(),
        }
    }

    /// Add an element with a new unique tag.
    pub fn add(&mut self, value: T) {
        let tag = UniqueTag::new();
        self.elements.entry(value).or_default().insert(tag);
    }

    /// Remove all observed tags for the given element.
    pub fn remove(&mut self, value: &T) {
        if let Some(tags) = self.elements.remove(value) {
            self.removed.extend(tags);
        }
    }

    /// Check if an element is in the set (has surviving tags).
    pub fn contains(&self, value: &T) -> bool {
        self.elements
            .get(value)
            .map(|tags| tags.iter().any(|t| !self.removed.contains(t)))
            .unwrap_or(false)
    }

    /// Merge with another OR-Set.
    pub fn merge(&mut self, other: &ORSet<T>) {
        // Merge element tags
        for (value, tags) in &other.elements {
            let entry = self.elements.entry(value.clone()).or_default();
            entry.extend(tags);
        }

        // Merge removed tags
        self.removed.extend(&other.removed);
    }

    /// Get the number of elements with surviving tags.
    pub fn len(&self) -> usize {
        self.elements
            .values()
            .filter(|tags| tags.iter().any(|t| !self.removed.contains(t)))
            .count()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all elements currently in the set.
    pub fn elements(&self) -> Vec<&T> {
        self.elements
            .iter()
            .filter(|(_, tags)| tags.iter().any(|t| !self.removed.contains(t)))
            .map(|(value, _)| value)
            .collect()
    }
}

impl<T: Eq + Hash + Clone> Default for ORSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + Clone> PartialEq for ORSet<T> {
    fn eq(&self, other: &Self) -> bool {
        // Two OR-Sets are equal if they contain the same elements
        let self_elements: std::collections::HashSet<&T> = self.elements().into_iter().collect();
        let other_elements: std::collections::HashSet<&T> = other.elements().into_iter().collect();
        self_elements == other_elements
    }
}

impl<T: Eq + Hash + Clone> Eq for ORSet<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove() {
        let mut set = ORSet::new();
        set.add("a");
        set.add("b");
        assert!(set.contains(&"a"));
        assert!(set.contains(&"b"));

        set.remove(&"a");
        assert!(!set.contains(&"a"));
        assert!(set.contains(&"b"));
    }

    #[test]
    fn test_concurrent_add_wins() {
        // Simulate two replicas adding the same element concurrently
        let mut replica_a = ORSet::new();
        let mut replica_b = ORSet::new();

        // Both add "x" independently (different tags)
        replica_a.add("x");
        replica_b.add("x");

        // Replica B removes "x" (but doesn't know about A's tag)
        replica_b.remove(&"x");

        // After merge, "x" should survive because A's tag was not observed by B
        replica_a.merge(&replica_b);
        assert!(
            replica_a.contains(&"x"),
            "Concurrent add-wins: element should survive"
        );
    }

    #[test]
    fn test_merge_is_idempotent() {
        let mut set = ORSet::new();
        set.add("a");
        set.add("b");
        set.remove(&"a");

        let original = set.clone();
        set.merge(&original.clone());

        assert_eq!(set.len(), original.len(), "Merge with self must be idempotent");
    }

    #[test]
    fn test_merge_is_commutative() {
        let mut a = ORSet::new();
        a.add("x");
        a.add("y");

        let mut b = ORSet::new();
        b.add("y");
        b.add("z");

        let a_clone = a.clone();
        let b_clone = b.clone();

        a.merge(&b_clone);
        let ab_elements = {
            let mut v = a.elements().into_iter().cloned().collect::<Vec<_>>();
            v.sort();
            v
        };

        b.merge(&a_clone);
        let ba_elements = {
            let mut v = b.elements().into_iter().cloned().collect::<Vec<_>>();
            v.sort();
            v
        };

        assert_eq!(ab_elements, ba_elements, "Merge must be commutative");
    }

    #[test]
    fn test_remove_after_merge() {
        let mut a = ORSet::new();
        let mut b = ORSet::new();

        a.add("a");
        b.add("b");

        a.merge(&b);
        assert!(a.contains(&"a"));
        assert!(a.contains(&"b"));

        a.remove(&"b");
        assert!(a.contains(&"a"));
        assert!(!a.contains(&"b"));
    }
}
