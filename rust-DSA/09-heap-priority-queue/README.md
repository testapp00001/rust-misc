# Heap / Priority Queue

## Overview
A Heap is a complete binary tree that satisfies the heap property. In Rust, `BinaryHeap` is a max-heap by default.

## Key Concepts

### BinaryHeap in Rust
```rust
use std::collections::BinaryHeap;

let mut heap = BinaryHeap::new();

// Push
heap.push(3);
heap.push(1);
heap.push(4);

// Pop (returns largest)
let max = heap.pop(); // Some(4)

// Peek
let max = heap.peek(); // Some(&4)
```

### Min-Heap (using Reverse)
```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

let mut min_heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();
min_heap.push(Reverse(3));
min_heap.push(Reverse(1));
min_heap.push(Reverse(4));

let min = min_heap.pop(); // Some(Reverse(1))
```

## Common Patterns

### 1. Kth Largest Element
```rust
fn find_kth_largest(nums: Vec<i32>, k: i32) -> i32 {
    let mut heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();

    for num in nums {
        heap.push(Reverse(num));
        if heap.len() > k as usize {
            heap.pop();
        }
    }

    heap.peek().unwrap().0
}
```

### 2. Merge K Sorted Lists
```rust
fn merge_k_lists(lists: Vec<Vec<i32>>) -> Vec<i32> {
    let mut heap: BinaryHeap<Reverse<(i32, usize, usize)>> = BinaryHeap::new();
    // (value, list_index, element_index)

    for (i, list) in lists.iter().enumerate() {
        if !list.is_empty() {
            heap.push(Reverse((list[0], i, 0)));
        }
    }

    let mut result = Vec::new();
    while let Some(Reverse((val, list_idx, elem_idx))) = heap.pop() {
        result.push(val);
        if elem_idx + 1 < lists[list_idx].len() {
            heap.push(Reverse((lists[list_idx][elem_idx + 1], list_idx, elem_idx + 1)));
        }
    }

    result
}
```

### 3. Task Scheduler
```rust
fn least_interval(tasks: Vec<char>, n: i32) -> i32 {
    let mut freq: HashMap<char, i32> = HashMap::new();
    for &task in &tasks {
        *freq.entry(task).or_insert(0) += 1;
    }

    let max_freq = *freq.values().max().unwrap();
    let max_count = freq.values().filter(|&&v| v == max_freq).count() as i32;

    let formula = (max_freq - 1) * (n + 1) + max_count;
    tasks.len() as i32.max(formula)
}
```

### 4. Median Finder (Two Heaps)
```rust
struct MedianFinder {
    small: BinaryHeap<i32>,           // Max-heap for smaller half
    large: BinaryHeap<Reverse<i32>>,  // Min-heap for larger half
}

impl MedianFinder {
    fn add_num(&mut self, num: i32) {
        self.small.push(num);
        self.large.push(Reverse(self.small.pop().unwrap()));

        if self.large.len() > self.small.len() {
            let Reverse(val) = self.large.pop().unwrap();
            self.small.push(val);
        }
    }

    fn find_median(&self) -> f64 {
        if self.small.len() > self.large.len() {
            *self.small.peek().unwrap() as f64
        } else {
            (*self.small.peek().unwrap() + self.large.peek().unwrap().0) as f64 / 2.0
        }
    }
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Kth Largest Element | Medium | Min-heap of size k |
| 2 | Last Stone Weight | Easy | Max-heap |
| 3 | K Closest Points | Medium | Max-heap of size k |
| 4 | Kth Largest in Stream | Medium | Min-heap of size k |
| 5 | Task Scheduler | Medium | Formula or heap |
| 6 | Design Twitter | Medium | Merge with heap |

## Tips for Rust

1. **`BinaryHeap` is max-heap**: Use `Reverse<T>` for min-heap.
2. **`peek()` returns `Option<&T>`**: Check before accessing.
3. **Custom ordering**: Implement `Ord`/`PartialOrd` for custom types.
4. **`BinaryHeap::from()`**: Create from iterator.
