# Union-Find (Disjoint Set Union)

## Overview
Union-Find is a data structure that tracks elements partitioned into disjoint sets. It supports two operations: find (which set does an element belong to?) and union (merge two sets).

## Key Concepts

### Basic Implementation
```rust
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    count: usize,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
            count: n,
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];  // Path halving
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, x: usize, y: usize) -> bool {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x == root_y { return false; }

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

    fn connected(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }
}
```

### Optimizations
1. **Path compression**: Make each node point directly to the root.
2. **Union by rank**: Attach smaller tree under larger tree.

## Common Patterns

### 1. Number of Connected Components
```rust
fn count_components(n: i32, edges: Vec<Vec<i32>>) -> i32 {
    let mut uf = UnionFind::new(n as usize);

    for edge in &edges {
        uf.union(edge[0] as usize, edge[1] as usize);
    }

    uf.count as i32
}
```

### 2. Redundant Connection
```rust
fn find_redundant_connection(edges: Vec<Vec<i32>>) -> Vec<i32> {
    let n = edges.len();
    let mut uf = UnionFind::new(n + 1);

    for edge in &edges {
        if !uf.union(edge[0] as usize, edge[1] as usize) {
            return edge.clone();
        }
    }

    vec![]
}
```

### 3. Accounts Merge
```rust
fn accounts_merge(accounts: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let n = accounts.len();
    let mut uf = UnionFind::new(n);
    let mut email_to_account: HashMap<String, usize> = HashMap::new();

    for (i, account) in accounts.iter().enumerate() {
        for email in &account[1..] {
            if let Some(&prev) = email_to_account.get(email) {
                uf.union(i, prev);
            } else {
                email_to_account.insert(email.clone(), i);
            }
        }
    }

    // Group emails by root account
    let mut groups: HashMap<usize, BTreeSet<String>> = HashMap::new();
    for (email, &account) in &email_to_account {
        let root = uf.find(account);
        groups.entry(root).or_default().insert(email.clone());
    }

    // Build result
    groups.into_iter().map(|(root, emails)| {
        let mut merged = vec![accounts[root][0].clone()];
        merged.extend(emails);
        merged
    }).collect()
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Accounts Merge | Medium | Union emails by account |
| 2 | Redundant Connection | Medium | First edge that creates cycle |
| 3 | Number of Provinces | Medium | Count connected components |

## Tips for Rust

1. **`find()` with path compression**: Use path halving for efficiency.
2. **`union()` returns bool**: Indicates if a merge actually happened.
3. **`count` field**: Track number of components.
4. **`rank` for union by rank**: Keeps tree balanced.
