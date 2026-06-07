// ============================================================================
// Problem: Course Schedule (LeetCode #207)
// ============================================================================
// There are `numCourses` courses. Some have prerequisites. Return true if
// you can finish all courses (no cycle in the dependency graph).
//
// Example:
//   numCourses = 2, prerequisites = [[1,0]]
//   Output: true  (take course 0, then course 1)
//
// ============================================================================
// APPROACH: Topological Sort / Cycle Detection (O(V+E) time, O(V+E) space)
// ============================================================================
//
// Use DFS to detect cycles in the directed graph:
// 1. Build adjacency list from prerequisites.
// 2. For each unvisited node, run DFS.
// 3. If we visit a node that's currently in the recursion stack → cycle.
// 4. If no cycle found → all courses can be finished.
//
// Alternative: Kahn's algorithm (BFS-based topological sort).
// ============================================================================

use std::collections::VecDeque;

// DFS approach
pub fn can_finish(num_courses: i32, prerequisites: Vec<Vec<i32>>) -> bool {
    let n = num_courses as usize;
    let mut graph: Vec<Vec<usize>> = vec![vec![]; n];
    for prereq in &prerequisites {
        graph[prereq[0] as usize].push(prereq[1] as usize);
    }

    let mut visited = vec![0; n]; // 0=unvisited, 1=visiting, 2=visited

    for i in 0..n {
        if has_cycle(&graph, i, &mut visited) {
            return false;
        }
    }

    true
}

fn has_cycle(graph: &[Vec<usize>], node: usize, visited: &mut [i32]) -> bool {
    if visited[node] == 1 {
        return true; // Cycle detected
    }
    if visited[node] == 2 {
        return false; // Already processed
    }

    visited[node] = 1; // Mark as visiting

    for &neighbor in &graph[node] {
        if has_cycle(graph, neighbor, visited) {
            return true;
        }
    }

    visited[node] = 2; // Mark as visited
    false
}

// BFS (Kahn's algorithm) approach
pub fn can_finish_bfs(num_courses: i32, prerequisites: Vec<Vec<i32>>) -> bool {
    let n = num_courses as usize;
    let mut graph: Vec<Vec<usize>> = vec![vec![]; n];
    let mut in_degree = vec![0; n];

    for prereq in &prerequisites {
        graph[prereq[1] as usize].push(prereq[0] as usize);
        in_degree[prereq[0] as usize] += 1;
    }

    let mut queue: VecDeque<usize> = VecDeque::new();
    for i in 0..n {
        if in_degree[i] == 0 {
            queue.push_back(i);
        }
    }

    let mut count = 0;
    while let Some(node) = queue.pop_front() {
        count += 1;
        for &neighbor in &graph[node] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    count == n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_finish() {
        assert!(can_finish(2, vec![vec![1, 0]]));
    }

    #[test]
    fn test_cannot_finish() {
        assert!(!can_finish(2, vec![vec![1, 0], vec![0, 1]]));
    }

    #[test]
    fn test_no_prereqs() {
        assert!(can_finish(3, vec![]));
    }

    #[test]
    fn test_bfs_approach() {
        assert!(can_finish_bfs(2, vec![vec![1, 0]]));
        assert!(!can_finish_bfs(2, vec![vec![1, 0], vec![0, 1]]));
    }
}
