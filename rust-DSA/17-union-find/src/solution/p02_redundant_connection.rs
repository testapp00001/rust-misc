// ============================================================================
// Problem: Redundant Connection (LeetCode #684)
// ============================================================================
// Given a graph that started as a tree with n nodes, one extra edge was
// added. Return the edge that can be removed to make it a tree again.
//
// Example:
//   Input:  [[1,2],[1,3],[2,3]]
//   Output: [2,3]
//
// ============================================================================
// APPROACH: Union-Find (O(n * α(n)) time, O(n) space)
// ============================================================================
//
// Process edges one by one. If adding an edge connects two nodes that
// are already in the same component, that edge is redundant.
// ============================================================================

pub fn find_redundant_connection(edges: &[Vec<i32>]) -> Vec<i32> {
    let n = edges.len();
    let mut parent: Vec<usize> = (0..=n).collect();

    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }

    for edge in edges {
        let (u, v) = (edge[0] as usize, edge[1] as usize);
        let root_u = find(&mut parent, u);
        let root_v = find(&mut parent, v);

        if root_u == root_v {
            return edge.clone();
        }

        parent[root_u] = root_v;
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            find_redundant_connection(&[vec![1, 2], vec![1, 3], vec![2, 3]]),
            vec![2, 3]
        );
    }

    #[test]
    fn test_different() {
        assert_eq!(
            find_redundant_connection(&[vec![1, 2], vec![2, 3], vec![3, 4], vec![1, 4], vec![1, 5]]),
            vec![1, 4]
        );
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
        //     let _ = find_redundant_connection(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}