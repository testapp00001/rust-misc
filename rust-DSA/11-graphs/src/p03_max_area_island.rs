// ============================================================================
// Problem: Max Area of Island (LeetCode #695)
// ============================================================================
// Given an m x n binary matrix grid, return the maximum area of an island.
// An island is a group of 1s connected horizontally or vertically.
//
// ============================================================================
// APPROACH: DFS (O(m*n) time, O(m*n) space)
// ============================================================================
//
// Similar to Number of Islands, but track the area of each island.
// ============================================================================

pub fn max_area_of_island(grid: &mut [Vec<i32>]) -> i32 {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut max_area = 0;

    for r in 0..rows {
        for c in 0..cols {
            if grid[r][c] == 1 {
                let area = dfs(grid, r, c);
                max_area = max_area.max(area);
            }
        }
    }

    max_area
}

fn dfs(grid: &mut [Vec<i32>], r: usize, c: usize) -> i32 {
    if r >= grid.len() || c >= grid[0].len() || grid[r][c] != 1 {
        return 0;
    }

    grid[r][c] = 0;
    let mut area = 1;

    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    for (dr, dc) in directions {
        let nr = r as i32 + dr;
        let nc = c as i32 + dc;
        if nr >= 0 && nc >= 0 {
            area += dfs(grid, nr as usize, nc as usize);
        }
    }

    area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut grid = vec![
            vec![0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
            vec![0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0],
            vec![0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0],
        ];
        assert_eq!(max_area_of_island(&mut grid), 6);
    }

    #[test]
    fn test_no_island() {
        let mut grid = vec![vec![0, 0, 0, 0]];
        assert_eq!(max_area_of_island(&mut grid), 0);
    }

    #[test]
    fn test_all_land() {
        let mut grid = vec![vec![1, 1], vec![1, 1]];
        assert_eq!(max_area_of_island(&mut grid), 4);
    }
}
