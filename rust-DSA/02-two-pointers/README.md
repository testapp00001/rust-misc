# Two Pointers

## Overview
Two Pointers is a technique where two pointers traverse an array or string, typically from opposite ends or moving in the same direction at different speeds.

## Key Concepts

### When to Use Two Pointers
1. **Sorted arrays**: Finding pairs that sum to a target.
2. **Palindromes**: Checking if a string reads the same forwards and backwards.
3. **Container problems**: Finding maximum area between two lines.
4. **Merging**: Combining two sorted arrays.

### Types of Two Pointer Patterns

#### 1. Opposite Ends (Converging)
```rust
let mut left = 0;
let mut right = nums.len() - 1;

while left < right {
    let sum = nums[left] + nums[right];
    if sum == target {
        return vec![left, right];
    } else if sum < target {
        left += 1;
    } else {
        right -= 1;
    }
}
```

#### 2. Same Direction (Fast/Slow)
```rust
let mut slow = 0;
for fast in 0..nums.len() {
    if condition(nums[fast]) {
        nums.swap(slow, fast);
        slow += 1;
    }
}
```

#### 3. Skip Non-Alphanumeric (for palindromes)
```rust
while left < right && !chars[left].is_alphanumeric() {
    left += 1;
}
while left < right && !chars[right].is_alphanumeric() {
    right -= 1;
}
```

## Common Patterns

### 1. Two Sum on Sorted Array
```rust
fn two_sum_sorted(nums: &[i32], target: i32) -> Vec<i32> {
    let mut left = 0;
    let mut right = nums.len() - 1;

    while left < right {
        match (nums[left] + nums[right]).cmp(&target) {
            Ordering::Less => left += 1,
            Ordering::Greater => right -= 1,
            Ordering::Equal => return vec![left as i32, right as i32],
        }
    }
    vec![]
}
```

### 2. Three Sum (Sort + Two Pointers)
```rust
fn three_sum(mut nums: Vec<i32>) -> Vec<Vec<i32>> {
    nums.sort();
    let mut result = Vec::new();

    for i in 0..nums.len() {
        if i > 0 && nums[i] == nums[i - 1] { continue; } // Skip duplicates

        let (mut left, mut right) = (i + 1, nums.len() - 1);
        while left < right {
            let sum = nums[i] + nums[left] + nums[right];
            match sum.cmp(&0) {
                Ordering::Less => left += 1,
                Ordering::Greater => right -= 1,
                Ordering::Equal => {
                    result.push(vec![nums[i], nums[left], nums[right]]);
                    while left < right && nums[left] == nums[left + 1] { left += 1; }
                    while left < right && nums[right] == nums[right - 1] { right -= 1; }
                    left += 1;
                    right -= 1;
                }
            }
        }
    }
    result
}
```

### 3. Container With Most Water
```rust
fn max_area(height: Vec<i32>) -> i32 {
    let (mut left, mut right) = (0, height.len() - 1);
    let mut max_area = 0;

    while left < right {
        let h = height[left].min(height[right]);
        let w = (right - left) as i32;
        max_area = max_area.max(h * w);

        if height[left] < height[right] {
            left += 1;
        } else {
            right -= 1;
        }
    }
    max_area
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Valid Palindrome | Easy | Skip non-alphanumeric chars |
| 2 | Two Sum II | Medium | Sorted array advantage |
| 3 | 3Sum | Medium | Sort + skip duplicates |
| 4 | Container With Most Water | Medium | Move shorter pointer |
| 5 | Trapping Rain Water | Hard | Track left/right max |

## Tips for Rust

1. **Use `match` with `cmp()`**: Clean way to handle three-way comparison.
2. **`saturating_sub()`**: Safe subtraction that clamps to 0.
3. **`char::is_alphanumeric()`**: Built-in method for checking valid characters.
4. **Bounds checking**: Always ensure `left < right` before accessing elements.
