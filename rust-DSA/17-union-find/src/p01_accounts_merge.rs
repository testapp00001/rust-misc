// ============================================================================
// Problem: Accounts Merge (LeetCode #721)
// ============================================================================
// Given a list of accounts where each element is a name followed by emails,
// merge accounts that have common emails.
//
// Example:
//   Input:  [["John","john@mail.com","john_new@mail.com"],
//            ["John","john@mail.com","johnsmith@mail.com"],
//            ["Mary","mary@mail.com"]]
//   Output: [["John","john@mail.com","john_new@mail.com","johnsmith@mail.com"],
//            ["Mary","mary@mail.com"]]
//
// ============================================================================
// APPROACH: Union-Find on Emails (O(n * α(n)) time, O(n) space)
// ============================================================================
//
// 1. Map each email to an account index.
// 2. If an email is seen before, union the current account with the previous.
// 3. Group emails by their root account.
// ============================================================================


use std::collections::{BTreeMap, BTreeSet};

pub fn accounts_merge(accounts: Vec<Vec<String>>) -> Vec<Vec<String>> {
    todo!("Implement accounts_merge")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_string_vec(accounts: &[&[&str]]) -> Vec<Vec<String>> {
        accounts
            .iter()
            .map(|a| a.iter().map(|s| s.to_string()).collect())
            .collect()
    }

    #[test]
    fn test_basic() {
        let accounts = to_string_vec(&[
            &["John", "john@mail.com", "john_new@mail.com"],
            &["John", "john@mail.com", "johnsmith@mail.com"],
            &["Mary", "mary@mail.com"],
        ]);
        let result = accounts_merge(accounts);
        assert_eq!(result.len(), 2);
        // John's emails should be merged
        let john = result.iter().find(|a| a[0] == "John").unwrap();
        assert!(john.contains(&"john@mail.com".to_string()));
        assert!(john.contains(&"john_new@mail.com".to_string()));
        assert!(john.contains(&"johnsmith@mail.com".to_string()));
    }

    #[test]
    fn test_no_merge() {
        let accounts = to_string_vec(&[
            &["John", "john@mail.com"],
            &["Mary", "mary@mail.com"],
        ]);
        let result = accounts_merge(accounts);
        assert_eq!(result.len(), 2);
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
        //     let _ = accounts_merge(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}