//! # Exercise: Shopping Cart with OR-Set
//!
//! ## Theory
//!
//! The shopping cart is the classic example of CRDTs in distributed systems.
//! First described in the Riak documentation, it demonstrates why naive
//! approaches fail and how OR-Sets solve the problem.
//!
//! The scenario: Two users (or the same user on two devices) add items to a
//! cart concurrently. With a naive approach, one add might be lost during
//! merge. With an OR-Set, concurrent adds are preserved (add-wins semantics).
//!
//! ## Proof / Intuition
//!
//! The OR-Set's add-wins property ensures that when two replicas concurrently
//! add the same item, the merged cart contains that item. This is exactly
//! what users expect: adding an item twice (from different devices) should
//! result in the item being in the cart, not lost.
//!
//! ## Implementation Task
//!
//! Implement `ShoppingCart` using `ORSet<Item>` and simulate the classic
//! concurrent add scenario.
//!
//! ## Verification
//!
//! Verify no items are lost in concurrent adds and remove works correctly.
//!
use crate::p04_or_set::ORSet;
use std::fmt;

/// An item in the shopping cart.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub name: String,
    pub price_cents: u64,
}

impl Item {
    pub fn new(name: &str, price_cents: u64) -> Self {
        Self {
            name: name.to_string(),
            price_cents,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (${:.2})", self.name, self.price_cents as f64 / 100.0)
    }
}

/// A shopping cart CRDT using OR-Set for items.
#[derive(Debug, Clone)]
pub struct ShoppingCart {
    items: ORSet<Item>,
}

impl ShoppingCart {
    /// Create a new empty shopping cart.
    pub fn new() -> Self {
        Self {
            items: ORSet::new(),
        }
    }

    /// Add an item to the cart.
    pub fn add_item(&mut self, item: Item) {
        self.items.add(item);
    }

    /// Remove an item from the cart.
    pub fn remove_item(&mut self, item: &Item) {
        self.items.remove(item);
    }

    /// Check if an item is in the cart.
    pub fn has_item(&self, item: &Item) -> bool {
        self.items.contains(item)
    }

    /// Get the total number of unique items.
    pub fn num_items(&self) -> usize {
        self.items.len()
    }

    /// Get all items.
    pub fn get_items(&self) -> Vec<&Item> {
        self.items.elements()
    }

    /// Calculate total price.
    pub fn total_price(&self) -> u64 {
        self.items
            .elements()
            .iter()
            .map(|item| item.price_cents)
            .sum()
    }

    /// Merge with another cart (for syncing across devices).
    pub fn merge(&mut self, other: &ShoppingCart) {
        self.items.merge(&other.items);
    }
}

impl Default for ShoppingCart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_check_item() {
        let mut cart = ShoppingCart::new();
        let laptop = Item::new("Laptop", 99999);
        let mouse = Item::new("Mouse", 2999);

        cart.add_item(laptop.clone());
        cart.add_item(mouse.clone());

        assert!(cart.has_item(&laptop));
        assert!(cart.has_item(&mouse));
        assert_eq!(cart.num_items(), 2);
        assert_eq!(cart.total_price(), 99999 + 2999);
    }

    #[test]
    fn test_remove_item() {
        let mut cart = ShoppingCart::new();
        let laptop = Item::new("Laptop", 99999);
        let mouse = Item::new("Mouse", 2999);

        cart.add_item(laptop.clone());
        cart.add_item(mouse.clone());
        cart.remove_item(&laptop);

        assert!(!cart.has_item(&laptop));
        assert!(cart.has_item(&mouse));
        assert_eq!(cart.num_items(), 1);
    }

    #[test]
    fn test_concurrent_add_no_items_lost() {
        // Classic Riak scenario: two users add items concurrently
        let mut cart_a = ShoppingCart::new();
        let mut cart_b = ShoppingCart::new();

        let laptop = Item::new("Laptop", 99999);
        let mouse = Item::new("Mouse", 2999);
        let keyboard = Item::new("Keyboard", 7999);

        // User A adds laptop and mouse
        cart_a.add_item(laptop.clone());
        cart_a.add_item(mouse.clone());

        // User B (same user, different device) adds mouse and keyboard
        cart_b.add_item(mouse.clone());
        cart_b.add_item(keyboard.clone());

        // Merge: should have all three items
        cart_a.merge(&cart_b);

        assert!(
            cart_a.has_item(&laptop),
            "Laptop should be in merged cart"
        );
        assert!(
            cart_a.has_item(&mouse),
            "Mouse should be in merged cart (add-wins)"
        );
        assert!(
            cart_a.has_item(&keyboard),
            "Keyboard should be in merged cart"
        );
        assert_eq!(cart_a.num_items(), 3);
    }

    #[test]
    fn test_merge_commutative() {
        let mut cart_a = ShoppingCart::new();
        let mut cart_b = ShoppingCart::new();

        let item_a = Item::new("A", 100);
        let item_b = Item::new("B", 200);

        cart_a.add_item(item_a.clone());
        cart_b.add_item(item_b.clone());

        let mut ab = cart_a.clone();
        ab.merge(&cart_b);

        let mut ba = cart_b.clone();
        ba.merge(&cart_a);

        assert_eq!(ab.num_items(), ba.num_items(), "Merge must be commutative");
    }
}
