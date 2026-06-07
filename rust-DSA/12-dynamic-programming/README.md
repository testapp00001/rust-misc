# Dynamic Programming

## Overview
Dynamic Programming (DP) solves problems by breaking them into overlapping subproblems. It stores solutions to subproblems to avoid redundant computation.

## Key Concepts

### Top-Down (Memoization)
```rust
fn fib(n: i32, memo: &mut HashMap<i32, i32>) -> i32 {
    if n <= 1 { return n; }
    if let Some(&result) = memo.get(&n) { return result; }

    let result = fib(n - 1, memo) + fib(n - 2, memo);
    memo.insert(n, result);
    result
}
```

### Bottom-Up (Tabulation)
```rust
fn fib(n: i32) -> i32 {
    if n <= 1 { return n; }

    let mut dp = vec![0; (n + 1) as usize];
    dp[1] = 1;

    for i in 2..=n as usize {
        dp[i] = dp[i - 1] + dp[i - 2];
    }

    dp[n as usize]
}
```

### Space Optimization
```rust
fn fib(n: i32) -> i32 {
    if n <= 1 { return n; }

    let mut prev2 = 0;
    let mut prev1 = 1;

    for _ in 2..=n {
        let current = prev1 + prev2;
        prev2 = prev1;
        prev1 = current;
    }

    prev1
}
```

## Common Patterns

### 1. 1D DP (Climbing Stairs)
```rust
fn climb_stairs(n: i32) -> i32 {
    if n <= 2 { return n; }

    let mut prev2 = 1;
    let mut prev1 = 2;

    for _ in 3..=n {
        let current = prev1 + prev2;
        prev2 = prev1;
        prev1 = current;
    }

    prev1
}
```

### 2. 1D DP with Decision (House Robber)
```rust
fn rob(nums: &[i32]) -> i32 {
    let mut prev2 = 0;
    let mut prev1 = 0;

    for &num in nums {
        let current = prev1.max(num + prev2);
        prev2 = prev1;
        prev1 = current;
    }

    prev1
}
```

### 3. 2D DP (Longest Common Subsequence)
```rust
fn lcs(text1: &str, text2: &str) -> i32 {
    let s1: Vec<char> = text1.chars().collect();
    let s2: Vec<char> = text2.chars().collect();
    let m = s1.len();
    let n = s2.len();

    let mut prev = vec![0; n + 1];
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        for j in 1..=n {
            if s1[i-1] == s2[j-1] {
                curr[j] = prev[j-1] + 1;
            } else {
                curr[j] = prev[j].max(curr[j-1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.fill(0);
    }

    prev[n]
}
```

### 4. Knapsack (0/1)
```rust
fn knapsack(weights: &[i32], values: &[i32], capacity: i32) -> i32 {
    let n = weights.len();
    let mut dp = vec![0; (capacity + 1) as usize];

    for i in 0..n {
        for w in (weights[i]..=capacity).rev() {
            dp[w as usize] = dp[w as usize].max(dp[(w - weights[i]) as usize] + values[i]);
        }
    }

    dp[capacity as usize]
}
```

### 5. Unbounded Knapsack (Coin Change)
```rust
fn coin_change(coins: &[i32], amount: i32) -> i32 {
    let mut dp = vec![i32::MAX; (amount + 1) as usize];
    dp[0] = 0;

    for i in 1..=amount as usize {
        for &coin in coins {
            let coin = coin as usize;
            if coin <= i && dp[i - coin] != i32::MAX {
                dp[i] = dp[i].min(dp[i - coin] + 1);
            }
        }
    }

    if dp[amount as usize] == i32::MAX { -1 } else { dp[amount as usize] }
}
```

### 6. Interval DP (Longest Palindromic Substring)
```rust
fn longest_palindrome(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut start = 0;
    let mut max_len = 1;

    for i in 0..chars.len() {
        let len1 = expand(&chars, i as i32, i as i32);
        let len2 = expand(&chars, i as i32, i as i32 + 1);
        let len = len1.max(len2);

        if len > max_len {
            max_len = len;
            start = i - (len - 1) / 2;
        }
    }

    chars[start..start + max_len].iter().collect()
}

fn expand(chars: &[char], mut left: i32, mut right: i32) -> usize {
    while left >= 0 && (right as usize) < chars.len() && chars[left as usize] == chars[right as usize] {
        left -= 1;
        right += 1;
    }
    (right - left - 1) as usize
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Climbing Stairs | Easy | Fibonacci pattern |
| 2 | House Robber | Medium | Max of include/exclude |
| 3 | House Robber II | Medium | Circular → two linear |
| 4 | Coin Change | Medium | Unbounded knapsack |
| 5 | Longest Increasing Subsequence | Medium | O(n²) DP or O(n log n) binary search |
| 6 | Longest Common Subsequence | Medium | 2D DP |
| 7 | Edit Distance | Medium | 2D DP with three operations |
| 8 | Partition Equal Subset Sum | Medium | 0/1 knapsack |
| 9 | Unique Paths | Medium | Grid DP |
| 10 | Word Break | Medium | Check all splits |
| 11 | Longest Palindromic Substring | Medium | Expand around center |
| 12 | Decode Ways | Medium | Count ways with constraints |

## Tips for Rust

1. **Use `Vec` for DP table**: Index by subproblem parameters.
2. **Space optimization**: Only keep previous row/column.
3. **`i32::MAX` for infinity**: Use as sentinel for impossible states.
4. **`.min()` and `.max()`**: For finding optimal values.
