# Binary Search

## Overview
Binary Search is an efficient algorithm for finding an element in a sorted collection. It reduces the search space by half at each step.

## Key Concepts

### Standard Binary Search
```rust
fn binary_search(nums: &[i32], target: i32) -> i32 {
    let mut left = 0;
    let mut right = nums.len();

    while left < right {
        let mid = left + (right - left) / 2;
        match nums[mid].cmp(&target) {
            Ordering::Equal => return mid as i32,
            Ordering::Less => left = mid + 1,
            Ordering::Greater => right = mid,
        }
    }

    -1
}
```

### Binary Search on Answer
```rust
fn binary_search_answer(nums: &[i32], condition: impl Fn(i32) -> bool) -> i32 {
    let mut left = min_possible;
    let mut right = max_possible;

    while left < right {
        let mid = left + (right - left) / 2;
        if condition(mid) {
            right = mid;
        } else {
            left = mid + 1;
        }
    }

    left
}
```

## Common Patterns

### 1. Find Exact Match
```rust
fn find(nums: &[i32], target: i32) -> Option<usize> {
    let mut left = 0;
    let mut right = nums.len();

    while left < right {
        let mid = left + (right - left) / 2;
        match nums[mid].cmp(&target) {
            Ordering::Equal => return Some(mid),
            Ordering::Less => left = mid + 1,
            Ordering::Greater => right = mid,
        }
    }

    None
}
```

### 2. Find First/Last Position
```rust
fn find_first(nums: &[i32], target: i32) -> Option<usize> {
    let mut left = 0;
    let mut right = nums.len();

    while left < right {
        let mid = left + (right - left) / 2;
        if nums[mid] >= target {
            right = mid;
        } else {
            left = mid + 1;
        }
    }

    if left < nums.len() && nums[left] == target {
        Some(left)
    } else {
        None
    }
}
```

### 3. Binary Search on Rotated Array
```rust
fn search_rotated(nums: &[i32], target: i32) -> i32 {
    let mut left = 0;
    let mut right = nums.len();

    while left < right {
        let mid = left + (right - left) / 2;

        if nums[mid] == target {
            return mid as i32;
        }

        // Left half is sorted
        if nums[left] <= nums[mid] {
            if nums[left] <= target && target < nums[mid] {
                right = mid;
            } else {
                left = mid + 1;
            }
        }
        // Right half is sorted
        else {
            if nums[mid] < target && target <= nums[right - 1] {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
    }

    -1
}
```

### 4. Binary Search on Answer (Koko Eating Bananas)
```rust
fn min_eating_speed(piles: &[i32], h: i32) -> i32 {
    let mut left = 1;
    let mut right = *piles.iter().max().unwrap();

    while left < right {
        let mid = left + (right - left) / 2;
        let hours: i32 = piles.iter().map(|&p| (p + mid - 1) / mid).sum();

        if hours <= h {
            right = mid;
        } else {
            left = mid + 1;
        }
    }

    left
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Binary Search | Easy | Classic implementation |
| 2 | Search a 2D Matrix | Medium | Treat as 1D array |
| 3 | Koko Eating Bananas | Medium | Binary search on answer |
| 4 | Find Minimum in Rotated Array | Medium | Compare with right |
| 5 | Search in Rotated Array | Medium | Identify sorted half |
| 6 | Time Based Key-Value Store | Medium | Binary search on timestamps |

## Tips for Rust

1. **Avoid overflow**: Use `left + (right - left) / 2` instead of `(left + right) / 2`.
2. **Use `cmp()` method**: Returns `Ordering` enum for clean matching.
3. **Exclusive upper bound**: Use `right = nums.len()` (not `nums.len() - 1`).
4. **`binary_search()` method**: Built-in method on slices for simple cases.
