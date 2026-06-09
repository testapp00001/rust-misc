// ============================================================================
// Problem: Number of Provinces (LeetCode #547)
// ============================================================================
// Given an n x n matrix isConnected where isConnected[i][j] = 1 means
// city i and city j are directly connected, return the number of provinces.
//
// Example:
//   Input:  [[1,1,0],[1,1,0],[0,0,1]]
//   Output: 2
//
// ============================================================================
// APPROACH: Union-Find (O(n² * α(n)) time, O(n) space)
// ============================================================================
//
// Use Union-Find to count connected components.
// For each pair (i, j) where isConnected[i][j] == 1, union them.
// ============================================================================

pub fn find_circle_num(is_connected: &[Vec<i32>]) -> i32 {
    let n = is_connected.len();
    let mut parent: Vec<usize> = (0..n).collect();
    let mut count = n as i32;

    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }

    for i in 0..n {
        for j in i + 1..n {
            if is_connected[i][j] == 1 {
                let root_i = find(&mut parent, i);
                let root_j = find(&mut parent, j);
                if root_i != root_j {
                    parent[root_i] = root_j;
                    count -= 1;
                }
            }
        }
    }

    count
}

// Alternative: DFS approach
pub fn find_circle_num_dfs(is_connected: &[Vec<i32>]) -> i32 {
    let n = is_connected.len();
    let mut visited = vec![false; n];
    let mut count = 0;

    for i in 0..n {
        if !visited[i] {
            count += 1;
            dfs(is_connected, i, &mut visited);
        }
    }

    count
}

fn dfs(is_connected: &[Vec<i32>], city: usize, visited: &mut [bool]) {
    visited[city] = true;
    for j in 0..is_connected.len() {
        if is_connected[city][j] == 1 && !visited[j] {
            dfs(is_connected, j, visited);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let isConnected = vec![vec![1, 1, 0], vec![1, 1, 0], vec![0, 0, 1]];
        assert_eq!(find_circle_num(&isConnected), 2);
    }

    #[test]
    fn test_three_provinces() {
        let isConnected = vec![vec![1, 0, 0], vec![0, 1, 0], vec![0, 0, 1]];
        assert_eq!(find_circle_num(&isConnected), 3);
    }

    #[test]
    fn test_one_province() {
        let isConnected = vec![vec![1, 1, 1], vec![1, 1, 1], vec![1, 1, 1]];
        assert_eq!(find_circle_num(&isConnected), 1);
    }

    #[test]
    fn test_dfs_approach() {
        let isConnected = vec![vec![1, 1, 0], vec![1, 1, 0], vec![0, 0, 1]];
        assert_eq!(find_circle_num_dfs(&isConnected), 2);
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
        //     let _ = find_circle_num(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}