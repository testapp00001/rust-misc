//! Deterministic state machine that applies committed Raft log entries.
//!
//! The state machine maintains a key-value map and processes `Command` entries
//! in log order. Because Raft guarantees the same log order on every node,
//! calling `apply` with the same sequence of commands produces the same state.

use std::collections::HashMap;

use crate::consensus::raft::Command;

/// A simple key-value state machine that applies committed Raft commands.
#[derive(Debug, Clone)]
pub struct StateMachine {
    data: HashMap<String, String>,
}

impl StateMachine {
    /// Create a new, empty state machine.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Apply a single command to the state machine.
    ///
    /// * `Command::Put(k, v)` inserts or overwrites the value for key `k`.
    /// * `Command::Get(k)` is a no-op (reads are served separately).
    /// * `Command::Delete(k)` removes the key if it exists.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Put(key, value) => {
                self.data.insert(key, value);
            }
            Command::Get(_) => {
                // Reads do not mutate state.
            }
            Command::Delete(key) => {
                self.data.remove(&key);
            }
        }
    }

    /// Look up a value by key. Returns `None` if the key does not exist.
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    /// Return a reference to the entire key-value map.
    pub fn get_all(&self) -> &HashMap<String, String> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut sm = StateMachine::new();
        sm.apply(Command::Put("a".into(), "1".into()));
        assert_eq!(sm.get("a"), Some("1".into()));
    }

    #[test]
    fn test_delete() {
        let mut sm = StateMachine::new();
        sm.apply(Command::Put("a".into(), "1".into()));
        sm.apply(Command::Delete("a".into()));
        assert_eq!(sm.get("a"), None);
    }

    #[test]
    fn test_get_is_noop() {
        let mut sm = StateMachine::new();
        sm.apply(Command::Put("a".into(), "1".into()));
        sm.apply(Command::Get("a".into()));
        assert_eq!(sm.get("a"), Some("1".into()));
    }

    #[test]
    fn test_overwrite() {
        let mut sm = StateMachine::new();
        sm.apply(Command::Put("a".into(), "1".into()));
        sm.apply(Command::Put("a".into(), "2".into()));
        assert_eq!(sm.get("a"), Some("2".into()));
    }

    #[test]
    fn test_get_all() {
        let mut sm = StateMachine::new();
        sm.apply(Command::Put("x".into(), "10".into()));
        sm.apply(Command::Put("y".into(), "20".into()));
        let all = sm.get_all();
        assert_eq!(all.len(), 2);
        assert_eq!(all.get("x"), Some(&"10".to_string()));
        assert_eq!(all.get("y"), Some(&"20".to_string()));
    }
}
