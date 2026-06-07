// ============================================================================
// Union-Find (Disjoint Set Union) Data Structure
// ============================================================================
// Union-Find is a data structure that tracks elements partitioned into
// disjoint (non-overlapping) sets.
//
// Operations:
// - find(x): Find the root/representative of x's set — O(α(n)) amortized
// - union(x, y): Merge the sets containing x and y — O(α(n)) amortized
//
// Optimizations:
// 1. Path compression: Make each node point directly to the root.
// 2. Union by rank: Attach the smaller tree under the larger tree.
//
// α(n) is the inverse Ackermann function, which is effectively O(1).
//
// Use cases:
// - Connected components in a graph
// - Detecting cycles
// - Kruskal's minimum spanning tree
// - Network connectivity
// ============================================================================

pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    count: usize, // Number of components
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
            count: n,
        }
    }

    pub fn find(&mut self, mut x: usize) -> usize {
        // Path compression
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]]; // Path halving
            x = self.parent[x];
        }
        x
    }

    pub fn union(&mut self, x: usize, y: usize) -> bool {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x == root_y {
            return false; // Already in the same set
        }

        // Union by rank
        if self.rank[root_x] < self.rank[root_y] {
            self.parent[root_x] = root_y;
        } else if self.rank[root_x] > self.rank[root_y] {
            self.parent[root_y] = root_x;
        } else {
            self.parent[root_y] = root_x;
            self.rank[root_x] += 1;
        }

        self.count -= 1;
        true
    }

    pub fn connected(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    pub fn component_count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut uf = UnionFind::new(5);
        assert_eq!(uf.component_count(), 5);

        uf.union(0, 1);
        assert_eq!(uf.component_count(), 4);
        assert!(uf.connected(0, 1));
        assert!(!uf.connected(0, 2));

        uf.union(1, 2);
        assert_eq!(uf.component_count(), 3);
        assert!(uf.connected(0, 2));

        uf.union(3, 4);
        assert_eq!(uf.component_count(), 2);

        uf.union(2, 4);
        assert_eq!(uf.component_count(), 1);
        assert!(uf.connected(0, 4));
    }

    #[test]
    fn test_self_union() {
        let mut uf = UnionFind::new(3);
        assert!(!uf.union(0, 0)); // No-op
        assert_eq!(uf.component_count(), 3);
    }

    #[test]
    fn test_duplicate_union() {
        let mut uf = UnionFind::new(3);
        assert!(uf.union(0, 1));
        assert!(!uf.union(0, 1)); // Already connected
        assert_eq!(uf.component_count(), 2);
    }
}
