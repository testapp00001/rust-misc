// ============================================================================
// Problem: Number of Connected Components (LeetCode #323)
// ============================================================================
// Given n nodes and a list of edges, return the number of connected
// components in the graph.
//
// ============================================================================
// APPROACH: Union-Find (O(V+E) time, O(V) space)
// ============================================================================
//
// Use Union-Find to count connected components:
// 1. Start with n components (each node is its own component).
// 2. For each edge, union the two nodes.
// 3. If they were in different components, decrement the count.
// ============================================================================



pub fn count_components(n: i32, edges: Vec<Vec<i32>>) -> i32 {
    todo!("Implement count_components")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(count_components(5, vec![vec![0, 1], vec![1, 2], vec![3, 4]]), 2);
    }

    #[test]
    fn test_single() {
        assert_eq!(count_components(3, vec![vec![0, 1], vec![0, 2]]), 1);
    }

    #[test]
    fn test_disconnected() {
        assert_eq!(count_components(4, vec![]), 4);
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
        //     let _ = count_components(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}