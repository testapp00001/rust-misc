// ============================================================================
// Problem: Implement Trie (LeetCode #208)
// ============================================================================
// Implement a trie with insert, search, and startsWith operations.
//
// This is already implemented in the `trie` module. This file re-exports
// it for the problem interface.
//
// See trie.rs for the full implementation with explanations.
// ============================================================================

pub use super::trie::Trie;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_operations() {
        let mut trie = Trie::new();
        trie.insert("apple");
        assert!(trie.search("apple"));
        assert!(!trie.search("app"));
        assert!(trie.starts_with("app"));
        trie.insert("app");
        assert!(trie.search("app"));
    }

    #[test]
    fn test_edge_cases() {
        let mut trie = Trie::new();
        trie.insert("a");
        assert!(trie.search("a"));
        assert!(!trie.search("b"));
        assert!(trie.starts_with("a"));
        assert!(!trie.starts_with("b"));
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
        //     let _ = solution(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}