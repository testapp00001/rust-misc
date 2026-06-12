//! Shard rebalancing logic.
//!
//! When shards are added or removed, keys must be migrated between shards to maintain
//! the consistent hash ring's invariants. The `Rebalancer` compares an old ring
//! configuration with a new one and computes the minimal set of key migrations needed.

use super::consistent_hash::ConsistentHashRing;

/// Computes and tracks key migrations between two ring configurations.
#[derive(Debug)]
pub struct Rebalancer {
    /// The ring before the topology change.
    old_ring: ConsistentHashRing,
    /// The ring after the topology change.
    new_ring: ConsistentHashRing,
    /// Migrations as `(from_shard, to_shard, key)` triples.
    migrations: Vec<(usize, usize, String)>,
}

impl Rebalancer {
    /// Create a new rebalancer from the old and new ring configurations.
    ///
    /// # Arguments
    ///
    /// * `old_ring` - The ring before the change.
    /// * `new_ring` - The ring after the change.
    pub fn new(old_ring: ConsistentHashRing, new_ring: ConsistentHashRing) -> Self {
        Self {
            old_ring,
            new_ring,
            migrations: Vec::new(),
        }
    }

    /// Determine which keys need to be migrated.
    ///
    /// For each key, the old and new ring are consulted. If they disagree on the
    /// owning shard, a migration record is created.
    ///
    /// # Arguments
    ///
    /// * `keys` - The set of keys to evaluate.
    pub fn compute_migrations(&mut self, keys: &[String]) {
        self.migrations.clear();
        for key in keys {
            let old_shard = self.old_ring.get_shard(key);
            let new_shard = self.new_ring.get_shard(key);
            if let (Some(from), Some(to)) = (old_shard, new_shard) {
                if from != to {
                    self.migrations.push((from, to, key.clone()));
                }
            }
        }
    }

    /// Return the list of migrations computed by `compute_migrations`.
    pub fn execute_migrations(&self) -> Vec<(usize, usize, String)> {
        self.migrations.clone()
    }

    /// Return the number of migrations that need to occur.
    pub fn migration_count(&self) -> usize {
        self.migrations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_migrations_when_rings_match() {
        let mut old = ConsistentHashRing::new(10);
        old.add_shard(0);
        old.add_shard(1);
        let new = old.clone();

        let mut rebalancer = Rebalancer::new(old, new);
        let keys: Vec<String> = (0..100).map(|i| format!("key-{}", i)).collect();
        rebalancer.compute_migrations(&keys);
        assert_eq!(rebalancer.migration_count(), 0);
    }

    #[test]
    fn test_migrations_on_add_shard() {
        let mut old = ConsistentHashRing::new(150);
        old.add_shard(0);
        old.add_shard(1);

        let mut new = ConsistentHashRing::new(150);
        new.add_shard(0);
        new.add_shard(1);
        new.add_shard(2);

        let mut rebalancer = Rebalancer::new(old, new);
        let keys: Vec<String> = (0..1000).map(|i| format!("key-{}", i)).collect();
        rebalancer.compute_migrations(&keys);

        // Some keys should have moved to the new shard.
        assert!(rebalancer.migration_count() > 0);
        // But not all keys.
        assert!(rebalancer.migration_count() < 1000);
    }

    #[test]
    fn test_execute_migrations_returns_list() {
        let mut old = ConsistentHashRing::new(10);
        old.add_shard(0);

        let mut new = ConsistentHashRing::new(10);
        new.add_shard(0);
        new.add_shard(1);

        let mut rebalancer = Rebalancer::new(old, new);
        let keys = vec!["my-key".to_string()];
        rebalancer.compute_migrations(&keys);

        let migrations = rebalancer.execute_migrations();
        // The migration list should be valid triples.
        for (from, to, _key) in &migrations {
            assert_ne!(from, to);
        }
    }
}
