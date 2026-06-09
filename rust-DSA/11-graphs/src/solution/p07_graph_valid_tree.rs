// ============================================================================
// Problem: Graph Valid Tree (LeetCode #261)
// ============================================================================
// Given n nodes and a list of edges, determine if they form a valid tree.
//
// A valid tree has:
// 1. No cycles.
// 2. All nodes are connected (single connected component).
//
// ============================================================================
// APPROACH: Union-Find or DFS (O(V+E) time, O(V+E) space)
// ============================================================================
//
// Conditions for a valid tree:
// 1. Exactly n-1 edges.
// 2. All nodes are connected.
//
// Use DFS to check connectivity and cycle detection.
// ============================================================================

pub fn valid_tree(n: i32, edges: Vec<Vec<i32>>) -> bool {
    let n = n as usize;

    // A tree must have exactly n-1 edges
    if edges.len() != n - 1 {
        return false;
    }

    // Build adjacency list
    let mut graph: Vec<Vec<usize>> = vec![vec![]; n];
    for edge in &edges {
        graph[edge[0] as usize].push(edge[1] as usize);
        graph[edge[1] as usize].push(edge[0] as usize);
    }

    // Check connectivity using DFS
    let mut visited = vec![false; n];
    dfs(&graph, 0, &mut visited);

    visited.iter().all(|&v| v)
}

fn dfs(graph: &[Vec<usize>], node: usize, visited: &mut [bool]) {
    if visited[node] {
        return;
    }
    visited[node] = true;
    for &neighbor in &graph[node] {
        dfs(graph, neighbor, visited);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid() {
        assert!(valid_tree(5, vec![vec![0, 1], vec![0, 2], vec![0, 3], vec![1, 4]]));
    }

    #[test]
    fn test_cycle() {
        assert!(!valid_tree(5, vec![vec![0, 1], vec![1, 2], vec![2, 3], vec![1, 3], vec![1, 4]]));
    }

    #[test]
    fn test_disconnected() {
        assert!(!valid_tree(4, vec![vec![0, 1], vec![2, 3]]));
    }

    #[test]
    fn test_single() {
        assert!(valid_tree(1, vec![]));
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
        //     let _ = valid_tree(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}