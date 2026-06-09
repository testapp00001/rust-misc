// ============================================================================
// Problem: Time Based Key-Value Store (LeetCode #981)
// ============================================================================
// Design a time-based key-value store that supports:
// - set(key, value, timestamp): stores the key with the value at the time.
// - get(key, timestamp): returns the value such that set was called with
//   the largest timestamp_prev <= timestamp.
//
// Example:
//   set("foo", "bar", 1)
//   get("foo", 1)     → "bar"
//   get("foo", 3)     → "bar" (no update between timestamp 1 and 3)
//   set("foo", "bar2", 4)
//   get("foo", 4)     → "bar2"
//   get("foo", 5)     → "bar2"
//
// ============================================================================
// APPROACH: HashMap + Binary Search (O(log n) per get, O(1) per set)
// ============================================================================
//
// Use a HashMap<String, Vec<(timestamp, value)>> to store entries.
// For get(), binary search on the timestamp vector.
// ============================================================================

use std::collections::HashMap;

pub struct TimeMap {
    store: HashMap<String, Vec<(i32, String)>>,
}

impl TimeMap {
    pub fn new() -> Self {
        TimeMap {
            store: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String, timestamp: i32) {
        self.store
            .entry(key)
            .or_default()
            .push((timestamp, value));
    }

    pub fn get(&self, key: String, timestamp: i32) -> String {
        if let Some(entries) = self.store.get(&key) {
            // Binary search for the largest timestamp <= given timestamp
            let mut left = 0;
            let mut right = entries.len();
            let mut result = String::new();

            while left < right {
                let mid = left + (right - left) / 2;
                if entries[mid].0 <= timestamp {
                    result = entries[mid].1.clone();
                    left = mid + 1;
                } else {
                    right = mid;
                }
            }

            result
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut tm = TimeMap::new();
        tm.set("foo".to_string(), "bar".to_string(), 1);
        assert_eq!(tm.get("foo".to_string(), 1), "bar");
        assert_eq!(tm.get("foo".to_string(), 3), "bar");
        tm.set("foo".to_string(), "bar2".to_string(), 4);
        assert_eq!(tm.get("foo".to_string(), 4), "bar2");
        assert_eq!(tm.get("foo".to_string(), 5), "bar2");
    }

    #[test]
    fn test_missing_key() {
        let tm = TimeMap::new();
        assert_eq!(tm.get("missing".to_string(), 1), "");
    }

    #[test]
    fn test_before_first() {
        let mut tm = TimeMap::new();
        tm.set("key".to_string(), "val".to_string(), 5);
        assert_eq!(tm.get("key".to_string(), 1), "");
    }


    #[test]
    #[ignore]
    fn bench_performance() {
        // ⏱️  Benchmark test
        // Run: cargo test -p <package> bench_performance -- --ignored --nocapture
        //
        // To use: uncomment and customize the code below with your function
        // and realistic test data.
        //
        // let iterations = 10_000;
        // let input = /* generate your test input here */;
        // let start = std::time::Instant::now();
        // for _ in 0..iterations {
        //     let _ = new(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}