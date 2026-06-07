# Backtracking

## Overview
Backtracking is a systematic method to iterate through all possible configurations of a search space. It's used for problems like permutations, combinations, and constraint satisfaction.

## Key Concepts

### Backtracking Template
```rust
fn backtrack(state: &mut State, choices: &[Choice], result: &mut Vec<Solution>) {
    if is_solution(state) {
        result.push(construct_solution(state));
        return;
    }

    for choice in choices {
        if is_valid(state, choice) {
            make_choice(state, choice);
            backtrack(state, choices, result);
            undo_choice(state, choice);
        }
    }
}
```

### Rust-Specific Pattern
```rust
fn backtrack(
    nums: &[i32],
    start: usize,
    current: &mut Vec<i32>,  // Mutable reference to current state
    result: &mut Vec<Vec<i32>>,  // Mutable reference to results
) {
    result.push(current.clone());  // Clone to save current state

    for i in start..nums.len() {
        current.push(nums[i]);  // Make choice
        backtrack(nums, i + 1, current, result);  // Recurse
        current.pop();  // Undo choice
    }
}
```

## Common Patterns

### 1. Subsets
```rust
fn subsets(nums: &[i32]) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    backtrack(nums, 0, &mut current, &mut result);
    result
}

fn backtrack(nums: &[i32], start: usize, current: &mut Vec<i32>, result: &mut Vec<Vec<i32>>) {
    result.push(current.clone());

    for i in start..nums.len() {
        current.push(nums[i]);
        backtrack(nums, i + 1, current, result);
        current.pop();
    }
}
```

### 2. Permutations
```rust
fn permute(nums: &[i32]) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut visited = vec![false; nums.len()];
    backtrack(nums, &mut visited, &mut current, &mut result);
    result
}

fn backtrack(
    nums: &[i32],
    visited: &mut [bool],
    current: &mut Vec<i32>,
    result: &mut Vec<Vec<i32>>,
) {
    if current.len() == nums.len() {
        result.push(current.clone());
        return;
    }

    for i in 0..nums.len() {
        if !visited[i] {
            visited[i] = true;
            current.push(nums[i]);
            backtrack(nums, visited, current, result);
            current.pop();
            visited[i] = false;
        }
    }
}
```

### 3. Combination Sum
```rust
fn combination_sum(candidates: &[i32], target: i32) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    backtrack(candidates, target, 0, &mut current, &mut result);
    result
}

fn backtrack(
    candidates: &[i32],
    remaining: i32,
    start: usize,
    current: &mut Vec<i32>,
    result: &mut Vec<Vec<i32>>,
) {
    if remaining == 0 {
        result.push(current.clone());
        return;
    }
    if remaining < 0 {
        return;
    }

    for i in start..candidates.len() {
        current.push(candidates[i]);
        backtrack(candidates, remaining - candidates[i], i, current, result);
        current.pop();
    }
}
```

### 4. N-Queens
```rust
fn solve_n_queens(n: i32) -> Vec<Vec<String>> {
    let mut result = Vec::new();
    let mut board = vec![vec!['.'; n as usize]; n as usize];
    backtrack(&mut board, 0, &mut result);
    result
}

fn backtrack(board: &mut Vec<Vec<char>>, row: usize, result: &mut Vec<Vec<String>>) {
    if row == board.len() {
        result.push(board.iter().map(|r| r.iter().collect()).collect());
        return;
    }

    for col in 0..board.len() {
        if is_safe(board, row, col) {
            board[row][col] = 'Q';
            backtrack(board, row + 1, result);
            board[row][col] = '.';
        }
    }
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Subsets | Medium | Include/exclude each element |
| 2 | Combinations | Medium | Choose k from n |
| 3 | Permutations | Medium | Track visited |
| 4 | Subsets II | Medium | Skip duplicates at same level |
| 5 | Combination Sum | Medium | Allow reuse |
| 6 | Word Search | Medium | DFS + mark visited |
| 7 | Palindrome Partitioning | Medium | Try all prefixes |
| 8 | Letter Combinations | Medium | Map digits to letters |
| 9 | N-Queens | Hard | Check row/col/diagonal |

## Tips for Rust

1. **Pass `current` by reference**: Avoid unnecessary cloning.
2. **Clone when adding to result**: `result.push(current.clone())`.
3. **Use `visited` array**: Track which elements are used.
4. **Sort and skip duplicates**: For problems with duplicate elements.
