//! # Exercise: LWW-Register (Last-Writer-Wins Register)
//!
//! ## Theory
//!
//! A LWW-Register (Last-Writer-Wins Register) stores a single value and
//! resolves conflicts by choosing the value with the highest timestamp.
//!
//! Each write includes a timestamp. When merging, the register with the
//! higher timestamp wins. This is simple but loses concurrent writes --
//! only the "last" writer's value survives.
//!
//! ## Proof / Intuition
//!
//! Total order on timestamps ensures a deterministic winner. The merge
//! function is:
//! - Commutative: max(a, b) = max(b, a)
//! - Associative: max(max(a, b), c) = max(a, max(b, c))
//! - Idempotent: max(a, a) = a
//!
//! ## Implementation Task
//!
//! Implement `LWWRegister<T>` with:
//! - `set(value, timestamp)` - write a new value
//! - `get()` - read the current value
//! - `merge(other)` - keep higher timestamp
//!
//! ## Verification
//!
//! Verify higher timestamp wins, merge is commutative, concurrent writes
//! resolved by timestamp.
//!
/// A Last-Writer-Wins Register CRDT.
///
/// Stores a value with a timestamp. On merge, the higher timestamp wins.
#[derive(Debug, Clone)]
pub struct LWWRegister<T: Clone> {
    value: T,
    timestamp: u64,
}

impl<T: Clone> LWWRegister<T> {
    /// Create a new LWW-Register with an initial value.
    pub fn new(value: T, timestamp: u64) -> Self {
        Self { value, timestamp }
    }

    /// Set a new value with a given timestamp.
    /// Only updates if the new timestamp is higher.
    pub fn set(&mut self, value: T, timestamp: u64) {
        if timestamp >= self.timestamp {
            self.value = value;
            self.timestamp = timestamp;
        }
    }

    /// Get the current value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Get the current timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Merge with another register. Higher timestamp wins.
    pub fn merge(&mut self, other: &LWWRegister<T>) {
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_higher_timestamp_wins() {
        let mut reg = LWWRegister::new("old_value", 100);
        reg.set("new_value", 200);

        assert_eq!(reg.get(), &"new_value");
        assert_eq!(reg.timestamp(), 200);
    }

    #[test]
    fn test_lower_timestamp_does_not_override() {
        let mut reg = LWWRegister::new("current", 200);
        reg.set("old", 100);

        assert_eq!(reg.get(), &"current");
        assert_eq!(reg.timestamp(), 200);
    }

    #[test]
    fn test_merge_is_commutative() {
        let a = LWWRegister::new("value_a", 100);
        let b = LWWRegister::new("value_b", 200);

        let mut a_clone = a.clone();
        let mut b_clone = b.clone();

        a_clone.merge(&b);
        b_clone.merge(&a);

        assert_eq!(a_clone.get(), b_clone.get(), "Merge must be commutative");
        assert_eq!(a_clone.timestamp(), b_clone.timestamp());
    }

    #[test]
    fn test_merge_higher_timestamp_wins() {
        let mut reg1 = LWWRegister::new("first", 100);
        let reg2 = LWWRegister::new("second", 50);

        reg1.merge(&reg2);
        assert_eq!(reg1.get(), &"first", "Higher timestamp should win");

        let mut reg3 = LWWRegister::new("third", 150);
        reg3.merge(&reg1);
        assert_eq!(reg3.get(), &"third", "Higher timestamp (150) should win over 100");
        reg3.merge(&LWWRegister::new("fourth", 200));
        assert_eq!(reg3.get(), &"fourth", "New highest timestamp wins");
    }

    #[test]
    fn test_concurrent_writes_resolved_by_timestamp() {
        let mut reg_a = LWWRegister::new("a_value", 150);
        let reg_b = LWWRegister::new("b_value", 100);

        // Simulate concurrent writes arriving at different replicas
        reg_a.merge(&reg_b);
        assert_eq!(reg_a.get(), &"a_value", "Higher timestamp (150) should win over 100");

        // Now merge the other direction
        let mut reg_b2 = LWWRegister::new("b_value", 100);
        reg_b2.merge(&LWWRegister::new("a_value", 150));
        assert_eq!(reg_b2.get(), &"a_value", "Both replicas converge to same value");
    }
}
