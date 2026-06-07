# Greedy

## Overview
Greedy algorithms make locally optimal choices at each step, hoping to find a global optimum. They work when the problem has the "greedy choice property" and "optimal substructure."

## Key Concepts

### When to Use Greedy
1. **Optimization problems**: Finding maximum/minimum.
2. **Interval scheduling**: Activity selection, meeting rooms.
3. **Huffman coding**: Data compression.
4. **Minimum spanning tree**: Kruskal's, Prim's.

### Greedy vs Dynamic Programming
- **Greedy**: Makes the best local choice without reconsidering.
- **DP**: Considers all possibilities and chooses the best.

## Common Patterns

### 1. Maximum Subarray (Kadane's Algorithm)
```rust
fn max_sub_array(nums: &[i32]) -> i32 {
    let mut current_sum = nums[0];
    let mut max_sum = nums[0];

    for &num in &nums[1..] {
        current_sum = num.max(current_sum + num);
        max_sum = max_sum.max(current_sum);
    }

    max_sum
}
```

### 2. Jump Game
```rust
fn can_jump(nums: &[i32]) -> bool {
    let mut farthest = 0;

    for (i, &num) in nums.iter().enumerate() {
        if i > farthest { return false; }
        farthest = farthest.max(i + num as usize);
    }

    true
}
```

### 3. Interval Scheduling (Activity Selection)
```rust
fn max_activities(mut intervals: Vec<(i32, i32)>) -> usize {
    intervals.sort_by_key(|&(_, end)| end);

    let mut count = 0;
    let mut last_end = i32::MIN;

    for (start, end) in intervals {
        if start >= last_end {
            count += 1;
            last_end = end;
        }
    }

    count
}
```

### 4. Jump Game II (Minimum Jumps)
```rust
fn jump(nums: &[i32]) -> i32 {
    let mut jumps = 0;
    let mut current_end = 0;
    let mut farthest = 0;

    for i in 0..nums.len() - 1 {
        farthest = farthest.max(i + nums[i] as usize);
        if i == current_end {
            jumps += 1;
            current_end = farthest;
        }
    }

    jumps
}
```

### 5. Gas Station
```rust
fn can_complete_circuit(gas: &[i32], cost: &[i32]) -> i32 {
    let mut total_surplus = 0;
    let mut current_surplus = 0;
    let mut start = 0;

    for i in 0..gas.len() {
        let surplus = gas[i] - cost[i];
        total_surplus += surplus;
        current_surplus += surplus;

        if current_surplus < 0 {
            start = i + 1;
            current_surplus = 0;
        }
    }

    if total_surplus >= 0 { start as i32 } else { -1 }
}
```

### 6. Partition Labels
```rust
fn partition_labels(s: &str) -> Vec<i32> {
    let chars: Vec<char> = s.chars().collect();
    let mut last = [0usize; 26];

    for (i, &c) in chars.iter().enumerate() {
        last[(c as u8 - b'a') as usize] = i;
    }

    let mut result = Vec::new();
    let mut start = 0;
    let mut end = 0;

    for (i, &c) in chars.iter().enumerate() {
        end = end.max(last[(c as u8 - b'a') as usize]);
        if i == end {
            result.push((end - start + 1) as i32);
            start = i + 1;
        }
    }

    result
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Maximum Subarray | Medium | Kadane's algorithm |
| 2 | Jump Game | Medium | Track farthest reachable |
| 3 | Jump Game II | Medium | BFS-like level tracking |
| 4 | Gas Station | Medium | Track surplus |
| 5 | Hand of Straights | Medium | Sort + greedy |
| 6 | Merge Triplets | Medium | Filter + merge |
| 7 | Partition Labels | Medium | Track last occurrence |
| 8 | Valid Parenthesis String | Medium | Track min/max open count |

## Tips for Rust

1. **Sort first**: Many greedy problems require sorting.
2. **Track state**: Use variables to track the current best.
3. **Prove correctness**: Greedy works only when locally optimal = globally optimal.
4. **Consider edge cases**: Empty input, single element, all same.
