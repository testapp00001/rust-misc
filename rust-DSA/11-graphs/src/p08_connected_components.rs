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
    let n = n as usize;
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank = vec![0; n];
    let mut count = n as i32;

    for edge in &edges {
        let (a, b) = (edge[0] as usize, edge[1] as usize);
        let root_a = find(&mut parent, a);
        let root_b = find(&mut parent, b);
        if root_a != root_b {
            // Union by rank
            if rank[root_a] < rank[root_b] {
                parent[root_a] = root_b;
            } else if rank[root_a] > rank[root_b] {
                parent[root_b] = root_a;
            } else {
                parent[root_b] = root_a;
                rank[root_a] += 1;
            }
            count -= 1;
        }
    }

    count
}

fn find(parent: &mut [usize], mut node: usize) -> usize {
    while parent[node] != node {
        parent[node] = parent[parent[node]]; // Path compression
        node = parent[node];
    }
    node
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
}
