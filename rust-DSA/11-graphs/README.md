# Graphs

## Overview
Graphs are non-linear data structures consisting of nodes (vertices) and edges. They can be directed or undirected, weighted or unweighted.

## Key Concepts

### Graph Representations

#### Adjacency List
```rust
let mut graph: Vec<Vec<usize>> = vec![vec![]; n];
for edge in &edges {
    graph[edge[0]].push(edge[1]);
    graph[edge[1]].push(edge[0]); // For undirected
}
```

#### Adjacency Matrix
```rust
let mut matrix = vec![vec![0; n]; n];
for edge in &edges {
    matrix[edge[0]][edge[1]] = 1;
    matrix[edge[1]][edge[0]] = 1; // For undirected
}
```

## Traversal Algorithms

### 1. DFS (Recursive)
```rust
fn dfs(graph: &[Vec<usize>], node: usize, visited: &mut [bool]) {
    visited[node] = true;
    println!("{}", node);

    for &neighbor in &graph[node] {
        if !visited[neighbor] {
            dfs(graph, neighbor, visited);
        }
    }
}
```

### 2. DFS (Iterative)
```rust
fn dfs_iterative(graph: &[Vec<usize>], start: usize) {
    let mut visited = vec![false; graph.len()];
    let mut stack = vec![start];

    while let Some(node) = stack.pop() {
        if visited[node] { continue; }
        visited[node] = true;
        println!("{}", node);

        for &neighbor in graph[node].iter().rev() {
            if !visited[neighbor] {
                stack.push(neighbor);
            }
        }
    }
}
```

### 3. BFS
```rust
fn bfs(graph: &[Vec<usize>], start: usize) {
    let mut visited = vec![false; graph.len()];
    let mut queue = VecDeque::new();
    queue.push_back(start);
    visited[start] = true;

    while let Some(node) = queue.pop_front() {
        println!("{}", node);

        for &neighbor in &graph[node] {
            if !visited[neighbor] {
                visited[neighbor] = true;
                queue.push_back(neighbor);
            }
        }
    }
}
```

## Common Patterns

### 1. Number of Islands (DFS Flood Fill)
```rust
fn num_islands(grid: &mut [Vec<char>]) -> i32 {
    let mut count = 0;
    for r in 0..grid.len() {
        for c in 0..grid[0].len() {
            if grid[r][c] == '1' {
                count += 1;
                dfs(grid, r, c);
            }
        }
    }
    count
}

fn dfs(grid: &mut [Vec<char>], r: usize, c: usize) {
    if r >= grid.len() || c >= grid[0].len() || grid[r][c] != '1' {
        return;
    }
    grid[r][c] = '0';
    let directions = [(0,1),(1,0),(0,-1),(-1,0)];
    for (dr, dc) in directions {
        let nr = r as i32 + dr;
        let nc = c as i32 + dc;
        if nr >= 0 && nc >= 0 {
            dfs(grid, nr as usize, nc as usize);
        }
    }
}
```

### 2. Topological Sort (Kahn's Algorithm)
```rust
fn topological_sort(graph: &[Vec<usize>], in_degree: &[i32]) -> Vec<usize> {
    let mut queue: VecDeque<usize> = VecDeque::new();
    for i in 0..in_degree.len() {
        if in_degree[i] == 0 {
            queue.push_back(i);
        }
    }

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &neighbor in &graph[node] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    order
}
```

### 3. Cycle Detection (DFS)
```rust
fn has_cycle(graph: &[Vec<usize>], node: usize, visited: &mut [i32]) -> bool {
    if visited[node] == 1 { return true; }  // Visiting
    if visited[node] == 2 { return false; } // Visited

    visited[node] = 1;
    for &neighbor in &graph[node] {
        if has_cycle(graph, neighbor, visited) {
            return true;
        }
    }
    visited[node] = 2;
    false
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Number of Islands | Medium | DFS flood fill |
| 2 | Clone Graph | Medium | DFS + HashMap |
| 3 | Max Area of Island | Medium | DFS count area |
| 4 | Pacific Atlantic Water Flow | Medium | Multi-source BFS |
| 5 | Course Schedule | Medium | Cycle detection |
| 6 | Course Schedule II | Medium | Topological sort |
| 7 | Graph Valid Tree | Medium | n-1 edges + connected |
| 8 | Connected Components | Medium | Union-Find or DFS |
| 9 | Rotting Oranges | Medium | Multi-source BFS |

## Tips for Rust

1. **`VecDeque` for BFS**: Use `push_back()` and `pop_front()`.
2. **`Vec` for DFS stack**: Use `push()` and `pop()`.
3. **Visited array**: `vec![false; n]` for simple visited tracking.
4. **State enum**: Use `0=unvisited, 1=visiting, 2=visited` for cycle detection.
