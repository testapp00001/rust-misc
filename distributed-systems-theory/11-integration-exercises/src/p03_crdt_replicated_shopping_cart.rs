//! # Exercise: CRDT Replicated Shopping Cart
//!
//! ## Theory
//!
//! Conflict-free Replicated Data Types (CRDTs) are data structures that can be
//! replicated across multiple replicas, updated independently and concurrently,
//! and merged automatically without conflicts. The OR-Set (Observed-Remove Set)
//! is a set CRDT that supports add and remove operations.
//!
//! The key insight is that OR-Set uses unique tags for each add operation.
//! When an element is removed, only the tags that have been *observed* at the
//! removing replica are removed. If a concurrent add introduces a new tag that
//! wasn't in the remove set, that add survives the merge.
//!
//! This guarantees that:
//! - Concurrent adds always converge (union of add tags)
//! - Add-then-remove works correctly (remove cancels observed adds)
//! - No coordination is needed between replicas
//!
//! ## Proof / Intuition
//!
//! Consider two replicas. Replica A adds "milk" (tag t1), then Replica B
//! removes "milk" (observing t1). The remove set contains t1. After merge,
//! t1 is in both add_set and remove_set, so "milk" is absent. But if
//! Replica C concurrently adds "milk" with tag t2 (which B never saw),
//! the merged add_set has {t1, t2} and remove_set has {t1}. Since t2 is
//! not in the remove set, "milk" is present with tag t2.
//!
//! This is the fundamental property: removes only affect observed adds.
//!
//! ## Implementation Task
//!
//! 1. Implement the OR-Set with add, remove, contains, and merge operations
//! 2. Build a ShoppingCart on top of the OR-Set
//! 3. Ensure that concurrent adds from different replicas converge
//! 4. Ensure that add-then-remove (sequential) actually removes the element
//! 5. Verify merge correctness with concurrent operations
//!
//! ## Verification
//!
//! - Test concurrent adds converge to contain all added items
//! - Test add then remove results in item being absent
//! - Test merge never loses items that should be present

use std::collections::{HashMap, HashSet};

/// A unique tag generator counter.
static mut TAG_COUNTER: u64 = 0;

fn next_tag() -> u64 {
    unsafe {
        TAG_COUNTER += 1;
        TAG_COUNTER
    }
}

/// Represents an operation on the OR-Set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Add(String),
    Remove(String),
}

/// OR-Set (Observed-Remove Set) CRDT.
///
/// Each add operation is assigned a unique tag. Remove operations record
/// which tags were observed at the time of removal. Merge takes the union
/// of add tags minus any tags that appear in the remove set.
#[derive(Debug, Clone)]
pub struct ORSet {
    /// Elements currently in the set (derived from add_set).
    elements: HashSet<String>,
    /// Map from element -> set of unique add tags.
    add_set: HashMap<String, HashSet<u64>>,
    /// Map from element -> set of tags that were removed.
    remove_set: HashMap<String, HashSet<u64>>,
    /// Unique identifier for this replica's tags.
    unique_id: u64,
}

impl ORSet {
    /// Create a new OR-Set for a given replica.
    pub fn new(unique_id: u64) -> Self {
        Self {
            elements: HashSet::new(),
            add_set: HashMap::new(),
            remove_set: HashMap::new(),
            unique_id,
        }
    }

    /// Add an element to the set. Returns the tag used for this add.
    pub fn add(&mut self, element: &str) -> u64 {
        let tag = next_tag();
        self.elements.insert(element.to_string());
        self.add_set
            .entry(element.to_string())
            .or_insert_with(HashSet::new)
            .insert(tag);
        tag
    }

    /// Remove an element from the set. Only removes tags that are currently
    /// in the add_set (observed-remove semantics).
    pub fn remove(&mut self, element: &str) -> bool {
        if let Some(tags) = self.add_set.get(element) {
            if tags.is_empty() {
                return false;
            }
            // Record which tags we are removing (observed-remove)
            self.remove_set
                .entry(element.to_string())
                .or_insert_with(HashSet::new)
                .extend(tags.iter());
            // Clear the add tags
            self.add_set.insert(element.to_string(), HashSet::new());
            self.elements.remove(element);
            true
        } else {
            false
        }
    }

    /// Check if an element is in the set.
    pub fn contains(&mut self, element: &str) -> bool {
        // An element is present if it has add tags that are NOT in the remove set
        if let Some(add_tags) = self.add_set.get(element) {
            let remove_tags = self.remove_set.get(element);
            let has_active_tags = match remove_tags {
                Some(rt) => add_tags.iter().any(|t| !rt.contains(t)),
                None => !add_tags.is_empty(),
            };
            // Update the elements set to stay consistent
            if has_active_tags {
                self.elements.insert(element.to_string());
            } else {
                self.elements.remove(element);
            }
            has_active_tags
        } else {
            false
        }
    }

    /// Merge another OR-Set into this one using observed-remove semantics.
    /// The merged add_set is the union of both add_sets.
    /// The merged remove_set is the union of both remove_sets.
    /// An element is present if it has add tags not in the remove set.
    pub fn merge(&mut self, other: &ORSet) {
        // Merge add_sets: union of tags
        for (element, other_tags) in &other.add_set {
            let local_tags = self
                .add_set
                .entry(element.clone())
                .or_insert_with(HashSet::new);
            local_tags.extend(other_tags);
        }

        // Merge remove_sets: union of tags
        for (element, other_removed) in &other.remove_set {
            let local_removed = self
                .remove_set
                .entry(element.clone())
                .or_insert_with(HashSet::new);
            local_removed.extend(other_removed);
        }

        // Rebuild elements set from merged state
        self.elements.clear();
        for (element, add_tags) in &self.add_set {
            let remove_tags = self.remove_set.get(element);
            let has_active = match remove_tags {
                Some(rt) => add_tags.iter().any(|t| !rt.contains(t)),
                None => !add_tags.is_empty(),
            };
            if has_active {
                self.elements.insert(element.clone());
            }
        }
    }

    /// Get the number of elements in the set.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

/// A replicated shopping cart using CRDT (OR-Set) for conflict-free merging.
#[derive(Debug, Clone)]
pub struct ShoppingCart {
    /// The OR-Set storing cart items.
    items: ORSet,
    /// Unique identifier for this cart replica.
    id: u64,
}

impl ShoppingCart {
    /// Create a new shopping cart for a given replica.
    pub fn new(id: u64) -> Self {
        Self {
            items: ORSet::new(id),
            id,
        }
    }

    /// Add an item to the cart.
    pub fn add_item(&mut self, item: &str) -> u64 {
        self.items.add(item)
    }

    /// Remove an item from the cart.
    pub fn remove_item(&mut self, item: &str) -> bool {
        self.items.remove(item)
    }

    /// Check if the cart contains an item.
    pub fn has_item(&mut self, item: &str) -> bool {
        self.items.contains(item)
    }

    /// Merge another shopping cart into this one.
    pub fn merge_with(&mut self, other: &ShoppingCart) {
        self.items.merge(&other.items);
    }

    /// Get the number of items in the cart.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Get the replica ID.
    pub fn replica_id(&self) -> u64 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_adds_converge() {
        let mut cart_a = ShoppingCart::new(1);
        let mut cart_b = ShoppingCart::new(2);

        // Replica A adds milk and bread
        cart_a.add_item("milk");
        cart_a.add_item("bread");

        // Replica B adds eggs and milk (concurrent)
        cart_b.add_item("eggs");
        cart_b.add_item("milk");

        // After merge, both replicas should have all items
        cart_a.merge_with(&cart_b);
        cart_b.merge_with(&cart_a);

        assert!(cart_a.has_item("milk"));
        assert!(cart_a.has_item("bread"));
        assert!(cart_a.has_item("eggs"));
        assert_eq!(cart_a.item_count(), 3);

        // Both replicas converge
        assert!(cart_b.has_item("milk"));
        assert!(cart_b.has_item("bread"));
        assert!(cart_b.has_item("eggs"));
        assert_eq!(cart_b.item_count(), 3);
    }

    #[test]
    fn test_add_then_remove() {
        let mut cart = ShoppingCart::new(1);

        cart.add_item("milk");
        assert!(cart.has_item("milk"));

        cart.remove_item("milk");
        assert!(!cart.has_item("milk"));
        assert_eq!(cart.item_count(), 0);
    }

    #[test]
    fn test_concurrent_add_and_remove() {
        let mut cart_a = ShoppingCart::new(1);
        let mut cart_b = ShoppingCart::new(2);

        // Both replicas start with milk
        cart_a.add_item("milk");
        cart_b.add_item("milk");

        // Replica A adds more milk (different tag)
        cart_a.add_item("milk");

        // Replica B removes milk (removes the tag it observed)
        cart_b.remove_item("milk");

        // Merge: cart_a has two tags for milk, cart_b removed one
        cart_a.merge_with(&cart_b);
        cart_b.merge_with(&cart_a);

        // cart_a still has one active tag, so milk should be present
        // (the add on cart_a produced a new tag that cart_b never saw)
        assert!(cart_a.has_item("milk"));
    }

    #[test]
    fn test_merge_never_loses_items() {
        let mut cart_a = ShoppingCart::new(1);
        let mut cart_b = ShoppingCart::new(2);
        let mut cart_c = ShoppingCart::new(3);

        // Each replica adds different items concurrently
        cart_a.add_item("apple");
        cart_b.add_item("banana");
        cart_c.add_item("cherry");

        // Merge all together
        cart_a.merge_with(&cart_b);
        cart_a.merge_with(&cart_c);

        cart_b.merge_with(&cart_a);
        cart_c.merge_with(&cart_a);

        // All items should be present everywhere
        assert!(cart_a.has_item("apple"));
        assert!(cart_a.has_item("banana"));
        assert!(cart_a.has_item("cherry"));
        assert_eq!(cart_a.item_count(), 3);

        assert!(cart_b.has_item("apple"));
        assert!(cart_b.has_item("banana"));
        assert!(cart_b.has_item("cherry"));

        assert!(cart_c.has_item("apple"));
        assert!(cart_c.has_item("banana"));
        assert!(cart_c.has_item("cherry"));
    }

    #[test]
    fn test_multiple_adds_and_removes() {
        let mut cart = ShoppingCart::new(1);

        cart.add_item("a");
        cart.add_item("b");
        cart.add_item("c");
        assert_eq!(cart.item_count(), 3);

        cart.remove_item("b");
        assert_eq!(cart.item_count(), 2);
        assert!(cart.has_item("a"));
        assert!(!cart.has_item("b"));
        assert!(cart.has_item("c"));
    }
}
