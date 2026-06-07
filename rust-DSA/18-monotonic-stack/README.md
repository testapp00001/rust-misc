# Monotonic Stack

## Overview
A monotonic stack is a stack that maintains elements in sorted order. It's used to efficiently find the next greater/smaller element for each element in an array.

## Key Concepts

### Monotonic Decreasing Stack
```rust
// Find next greater element
fn next_greater(nums: &[i32]) -> Vec<i32> {
    let mut result = vec![-1; nums.len()];
    let mut stack: Vec<usize> = Vec::new();

    for i in 0..nums.len() {
        // Pop elements smaller than current
        while !stack.is_empty() && nums[i] > nums[*stack.last().unwrap()] {
            result[stack.pop().unwrap()] = nums[i];
        }
        stack.push(i);
    }

    result
}
```

### Monotonic Increasing Stack
```rust
// Find next smaller element
fn next_smaller(nums: &[i32]) -> Vec<i32> {
    let mut result = vec![-1; nums.len()];
    let mut stack: Vec<usize> = Vec::new();

    for i in 0..nums.len() {
        // Pop elements larger than current
        while !stack.is_empty() && nums[i] < nums[*stack.last().unwrap()] {
            result[stack.pop().unwrap()] = nums[i];
        }
        stack.push(i);
    }

    result
}
```

## Common Patterns

### 1. Daily Temperatures
```rust
fn daily_temperatures(temps: &[i32]) -> Vec<i32> {
    let mut result = vec![0; temps.len()];
    let mut stack: Vec<usize> = Vec::new();

    for i in 0..temps.len() {
        while !stack.is_empty() && temps[i] > temps[*stack.last().unwrap()] {
            let j = stack.pop().unwrap();
            result[j] = (i - j) as i32;
        }
        stack.push(i);
    }

    result
}
```

### 2. Next Greater Element (Circular)
```rust
fn next_greater_element_circular(nums: &[i32]) -> Vec<i32> {
    let n = nums.len();
    let mut result = vec![-1; n];
    let mut stack: Vec<usize> = Vec::new();

    for i in 0..2 * n {
        while !stack.is_empty() && nums[i % n] > nums[*stack.last().unwrap()] {
            result[stack.pop().unwrap()] = nums[i % n];
        }
        if i < n { stack.push(i); }
    }

    result
}
```

### 3. Largest Rectangle in Histogram
```rust
fn largest_rectangle_area(heights: &[i32]) -> i32 {
    let mut stack: Vec<usize> = Vec::new();
    let mut max_area = 0;
    let n = heights.len();

    for i in 0..=n {
        let h = if i < n { heights[i] } else { 0 };

        while !stack.is_empty() && heights[*stack.last().unwrap()] > h {
            let height = heights[stack.pop().unwrap()];
            let width = if stack.is_empty() { i } else { i - stack.last().unwrap() - 1 };
            max_area = max_area.max(height * width as i32);
        }

        stack.push(i);
    }

    max_area
}
```

### 4. Stock Span Problem
```rust
struct StockSpanner {
    stack: Vec<(i32, i32)>,  // (price, span)
}

impl StockSpanner {
    fn next(&mut self, price: i32) -> i32 {
        let mut span = 1;

        while !self.stack.is_empty() && self.stack.last().unwrap().0 <= price {
            span += self.stack.pop().unwrap().1;
        }

        self.stack.push((price, span));
        span
    }
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Daily Temperatures | Medium | Next greater element |
| 2 | Next Greater Element | Medium | Stack of indices |
| 3 | Online Stock Span | Medium | Stack of (price, span) |
| 4 | Largest Rectangle in Histogram | Hard | Increasing stack |

## Tips for Rust

1. **Store indices, not values**: Indices give you both value and position.
2. **Use `*stack.last().unwrap()`**: To peek at the top without popping.
3. **Process array twice for circular**: Use `i % n`.
4. **Sentinel value**: Add a 0 at the end for histogram problem.
