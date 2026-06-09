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
    // TODO: Define fields
}

impl TimeMap {
    pub fn new() -> Self {
        todo!("Implement new")
    }

    pub fn set(&mut self, key: String, value: String, timestamp: i32) {
        todo!("Implement set")
    }

    pub fn get(&self, key: String, timestamp: i32) -> String {
        todo!("Implement get")
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