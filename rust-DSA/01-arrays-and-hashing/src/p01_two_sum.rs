// ============================================================================
// Problem: Two Sum (LeetCode #1)
// ============================================================================
// Given an array of integers `nums` and an integer `target`, return the
// indices of the two numbers such that they add up to `target`.
//
// You may assume that each input would have exactly one solution, and you may
// not use the same element twice.
//
// Example:
//   Input:  nums = [2, 7, 11, 15], target = 9
//   Output: [0, 1]  (because nums[0] + nums[1] = 2 + 7 = 9)
//
// ============================================================================
// APPROACH: Hash Map (O(n) time, O(n) space)
// ============================================================================
//
// Brute force would be O(n²) — check every pair.
//
// The optimal approach uses a HashMap:
// 1. Iterate through the array.
// 2. For each number, calculate its complement: `complement = target - num`.
// 3. Check if the complement exists in the HashMap.
//    - If yes: return [index_of_complement, current_index].
//    - If no:  insert {num: current_index} into the HashMap.
//
// Why this works: We're essentially asking "have I seen the number I need
// before?" The HashMap gives O(1) lookups.
//
// Rust-specific tips:
// - Use `std::collections::HashMap` for the hash map.
// - `.entry()` API is great for inserting-or-updating.
// - `enumerate()` gives you (index, &value) pairs.
// ============================================================================

use std::collections::HashMap;

pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
    let mut seen: HashMap<i32, usize> = HashMap::new();

    for (i, &v) in nums.iter().enumerate() {
        let subtract = target - v;
        if let Some(&j) = seen.get(&subtract) {
           return vec![j as i32,i as i32];
        }
        seen.insert(v, i);
    }
    vec![]
}
















// pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
//     let mut seen: HashMap<i32, usize> = HashMap::new();

//     for (i, &num) in nums.iter().enumerate() {
//         let complement = target - num;
//         if let Some(&j) = seen.get(&complement) {
//             return vec![j as i32, i as i32];
//         }
//         seen.insert(num, i);
//     }

//     vec![] // No solution found (problem guarantees one exists)
// }

// ============================================================================
// Tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        assert_eq!(two_sum(vec![2, 7, 11, 15], 9), vec![0, 1]);
    }

    #[test]
    fn test_middle_elements() {
        assert_eq!(two_sum(vec![3, 2, 4], 6), vec![1, 2]);
    }

    #[test]
    fn test_same_elements() {
        assert_eq!(two_sum(vec![3, 3], 6), vec![0, 1]);
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(two_sum(vec![-1, -2, -3, -4, -5], -8), vec![2, 4]);
    }

    #[test]
    fn test_mixed_numbers() {
        assert_eq!(two_sum(vec![0, 4, 3, 0], 0), vec![0, 3]);
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
        //     let _ = two_sum(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}