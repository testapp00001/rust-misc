// ============================================================================
// Problem: Contains Duplicate (LeetCode #217)
// ============================================================================
// Given an integer array `nums`, return `true` if any value appears at least
// twice, and `false` if every element is distinct.
//
// Example:
//   Input:  [1, 2, 3, 1]
//   Output: true
//
//   Input:  [1, 2, 3, 4]
//   Output: false
//
// ============================================================================
// APPROACH: HashSet (O(n) time, O(n) space)
// ============================================================================
//
// Use a HashSet to track seen elements:
// - Iterate through the array.
// - If the current element is already in the set → return true.
// - Otherwise, insert it.
// - If we finish the loop → return false.
//
// Alternative: Sort first and check adjacent elements — O(n log n) time, O(1) space.
//
// Rust-specific tips:
// - `HashSet::insert()` returns `false` if the element already exists!
//   This is more idiomatic than checking `.contains()` first.
// ============================================================================


use std::collections::HashSet;

pub fn contains_duplicate(nums: Vec<i32>) -> bool {
    let mut distinct_set: HashSet<i32> = HashSet::new();

    for num in nums {
        if !distinct_set.insert(num) {
            return true;
        }
    }
    false
}

pub fn contains_duplicate_sort(mut nums: Vec<i32>) -> bool {
    nums.sort_unstable();

    println!("{:#?}", nums);
    for i in nums.windows(2) {
        if &i[0] == &i[1] {
            return true;
        }
    }
    false
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_duplicate() {
        assert!(contains_duplicate(vec![1, 2, 3, 1]));
    }

    #[test]
    fn test_no_duplicate() {
        assert!(!contains_duplicate(vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_empty() {
        assert!(!contains_duplicate(vec![]));
    }

    #[test]
    fn test_single_element() {
        assert!(!contains_duplicate(vec![1]));
    }

    #[test]
    fn test_all_same() {
        assert!(contains_duplicate(vec![1, 1, 1, 1]));
    }

    #[test]
    fn test_sort_approach() {
        assert!(contains_duplicate_sort(vec![1, 2, 3, 1]));
        assert!(!contains_duplicate_sort(vec![1, 2, 3, 4]));
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
        //     let _ = contains_duplicate(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}