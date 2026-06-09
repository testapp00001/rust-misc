// ============================================================================
// Problem: Group Anagrams (LeetCode #49)
// ============================================================================
// Given an array of strings `strs`, group the anagrams together.
//
// Example:
//   Input:  ["eat","tea","tan","ate","nat","bat"]
//   Output: [["bat"],["nat","tan"],["ate","eat","tea"]]
//
// ============================================================================
// APPROACH: Sorted Key HashMap (O(n * k log k) time, O(n * k) space)
// ============================================================================
//
// Key insight: Two strings are anagrams if they have the same characters
// when sorted. So we can use the sorted version as a HashMap key.
//
// Steps:
// 1. For each string, sort its characters → this is the "key".
// 2. Group strings by their key in a HashMap.
// 3. Return all groups.
//
// n = number of strings, k = max length of a string.
//
// Rust-specific tips:
// - `chars().collect::<Vec<_>>()` to get a sortable char vector.
// - `.entry(key).or_insert_with(Vec::new).push(value)` is the idiomatic
//   way to group into a HashMap.
// ============================================================================

use std::collections::HashMap;

pub fn group_anagrams(strs: Vec<String>) -> Vec<Vec<String>> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();

    for s in strs {
        let mut key: Vec<char> = s.chars().collect();
        key.sort();
        let key: String = key.into_iter().collect();
        groups.entry(key).or_default().push(s);
    }

    groups.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sort_result(mut result: Vec<Vec<String>>) -> Vec<Vec<String>> {
        for group in &mut result {
            group.sort();
        }
        result.sort_by(|a, b| a[0].cmp(&b[0]));
        result
    }

    #[test]
    fn test_basic() {
        let input: Vec<String> = ["eat", "tea", "tan", "ate", "nat", "bat"]
            .iter().map(|s| s.to_string()).collect();
        let mut result = group_anagrams(input);
        // Sort for deterministic comparison
        for group in &mut result {
            group.sort();
        }
        result.sort_by(|a, b| a[0].cmp(&b[0]));

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["ate", "eat", "tea"]);
        assert_eq!(result[1], vec!["bat"]);
        assert_eq!(result[2], vec!["nat", "tan"]);
    }

    #[test]
    fn test_empty() {
        let input: Vec<String> = vec!["".to_string()];
        let result = group_anagrams(input);
        assert_eq!(result, vec![vec![""]]);
    }

    #[test]
    fn test_single() {
        let input: Vec<String> = vec!["a".to_string()];
        let result = group_anagrams(input);
        assert_eq!(result, vec![vec!["a"]]);
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
        //     let _ = group_anagrams(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}