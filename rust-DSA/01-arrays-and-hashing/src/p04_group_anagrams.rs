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
    let mut map: HashMap<[u8; 26], Vec<String>> = HashMap::with_capacity(strs.len());

    for s in strs {
        let mut count = [0u8; 26];

        for byte in s.as_bytes() {
            count[(byte - b'a') as usize] += 1;
        }

        map.entry(count).or_default().push(s);
    }
    map.into_values().collect()
}
pub fn group_anagrams_sort(strs: Vec<String>) -> Vec<Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::with_capacity(strs.len());

    for s in strs {
        let mut bytes = s.clone().into_bytes();
        bytes.sort_unstable();
        let key = String::from_utf8(bytes).unwrap();

        map.entry(key).or_default().push(s);
    }
    map.into_values().collect()
}

pub fn group_anagrams_solution(strs: Vec<String>) -> Vec<Vec<String>> {
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
            .iter()
            .map(|s| s.to_string())
            .collect();
        let result = sort_result(group_anagrams(input));

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
    fn bench_performance() {
        // Generate test input: 5,000 strings of varying lengths
        let mut input = Vec::with_capacity(5000);
        let mut seed: u32 = 12345;
        let mut next_rand = || {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            seed
        };

        for _ in 0..5000 {
            let len = 5 + (next_rand() % 10) as usize;
            let mut s = String::with_capacity(len);
            for _ in 0..len {
                let char_code = b'a' + (next_rand() % 26) as u8;
                s.push(char_code as char);
            }
            input.push(s);
        }

        let iterations = 100;

        // Benchmark group_anagrams (frequency counting)
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = group_anagrams(input.clone());
        }
        let elapsed_counting = start.elapsed();

        // Benchmark group_anagrams_sort (character sorting)
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = group_anagrams_sort(input.clone());
        }
        let elapsed_sorting = start.elapsed();
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = group_anagrams_solution(input.clone());
        }
        let elapsed_solution = start.elapsed();

        println!(
            "\n  ⏱️  Benchmark Results ({} iterations, {} strings/iter):",
            iterations,
            input.len()
        );
        println!(
            "     group_anagrams (counting):     {:?} total ({:?}/call)",
            elapsed_counting,
            elapsed_counting / iterations
        );
        println!(
            "     group_anagrams_sort (sorting): {:?} total ({:?}/call)",
            elapsed_sorting,
            elapsed_sorting / iterations
        );
        println!(
            "     group_anagrams_solution (solution): {:?} total ({:?}/call)",
            elapsed_solution,
            elapsed_solution / iterations
        );
    }
}
