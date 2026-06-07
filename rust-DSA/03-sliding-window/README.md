# Sliding Window

## Overview
Sliding Window is a technique for problems involving contiguous subarrays or substrings. Instead of recalculating for each window position, we slide the window and update incrementally.

## Key Concepts

### When to Use Sliding Window
1. **Contiguous subarray/substring**: Find max/min/sum/count of contiguous elements.
2. **Fixed window size**: Process every window of size k.
3. **Variable window size**: Find longest/shortest window satisfying a condition.

### Types of Sliding Window

#### 1. Fixed Size Window
```rust
fn fixed_window(nums: &[i32], k: usize) -> i32 {
    let mut window_sum: i32 = nums[..k].iter().sum();
    let mut max_sum = window_sum;

    for i in k..nums.len() {
        window_sum += nums[i] - nums[i - k];
        max_sum = max_sum.max(window_sum);
    }

    max_sum
}
```

#### 2. Variable Size Window (Shrinkable)
```rust
fn variable_window(s: &str) -> i32 {
    let chars: Vec<char> = s.chars().collect();
    let mut left = 0;
    let mut max_len = 0;
    let mut seen = HashSet::new();

    for right in 0..chars.len() {
        while seen.contains(&chars[right]) {
            seen.remove(&chars[left]);
            left += 1;
        }
        seen.insert(chars[right]);
        max_len = max_len.max(right - left + 1);
    }

    max_len as i32
}
```

#### 3. Variable Size Window (Non-shrinkable)
```rust
fn non_shrinkable_window(s: &str, k: i32) -> i32 {
    let chars: Vec<char> = s.chars().collect();
    let mut left = 0;
    let mut max_freq = 0;
    let mut freq = [0usize; 26];

    for right in 0..chars.len() {
        let idx = (chars[right] as u8 - b'a') as usize;
        freq[idx] += 1;
        max_freq = max_freq.max(freq[idx]);

        // Don't shrink, just slide
        if (right - left + 1) - max_freq > k as usize {
            let left_idx = (chars[left] as u8 - b'a') as usize;
            freq[left_idx] -= 1;
            left += 1;
        }
    }

    (chars.len() - left) as i32
}
```

## Common Patterns

### 1. Longest Substring Without Repeating Characters
```rust
fn length_of_longest_substring(s: &str) -> i32 {
    let chars: Vec<char> = s.chars().collect();
    let mut seen = HashSet::new();
    let mut left = 0;
    let mut max_len = 0;

    for right in 0..chars.len() {
        while seen.contains(&chars[right]) {
            seen.remove(&chars[left]);
            left += 1;
        }
        seen.insert(chars[right]);
        max_len = max_len.max(right - left + 1);
    }

    max_len as i32
}
```

### 2. Minimum Window Substring
```rust
fn min_window(s: &str, t: &str) -> String {
    // Use HashMap to track required characters
    // Expand right to include more characters
    // Shrink left when window is valid
    // Track minimum valid window
}
```

### 3. Sliding Window Maximum (Monotonic Deque)
```rust
fn max_sliding_window(nums: Vec<i32>, k: usize) -> Vec<i32> {
    let mut deque: VecDeque<usize> = VecDeque::new();
    let mut result = Vec::new();

    for i in 0..nums.len() {
        // Remove indices outside window
        while !deque.is_empty() && deque.front().unwrap() + k <= i {
            deque.pop_front();
        }
        // Remove smaller elements from back
        while !deque.is_empty() && nums[*deque.back().unwrap()] <= nums[i] {
            deque.pop_back();
        }
        deque.push_back(i);

        if i >= k - 1 {
            result.push(nums[*deque.front().unwrap()]);
        }
    }

    result
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Best Time to Buy and Sell Stock | Easy | Track minimum so far |
| 2 | Longest Substring Without Repeating | Medium | HashSet for unique chars |
| 3 | Longest Repeating Character Replacement | Medium | Window size - max_freq <= k |
| 4 | Minimum Window Substring | Hard | HashMap + expand/shrink |
| 5 | Sliding Window Maximum | Hard | Monotonic deque |

## Tips for Rust

1. **`VecDeque` for deque**: Use `pop_front()` and `pop_back()`.
2. **`HashSet` for unique elements**: Track what's in the window.
3. **`HashMap` for frequency**: Count occurrences in the window.
4. **Avoid `.clone()`**: Use references when possible.
