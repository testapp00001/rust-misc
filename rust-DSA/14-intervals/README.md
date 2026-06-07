# Intervals

## Overview
Interval problems involve ranges of values (typically [start, end]). They often require sorting and careful handling of overlaps.

## Key Concepts

### Interval Representation
```rust
struct Interval {
    start: i32,
    end: i32,
}

// Or simply as a Vec<i32> with two elements
let interval = vec![start, end];
```

### Overlap Detection
```rust
fn overlaps(a: &[i32], b: &[i32]) -> bool {
    a[0] < b[1] && b[0] < a[1]
}
```

## Common Patterns

### 1. Merge Intervals
```rust
fn merge(mut intervals: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    if intervals.is_empty() { return vec![]; }

    intervals.sort_by_key(|v| v[0]);
    let mut result = vec![intervals[0].clone()];

    for interval in &intervals[1..] {
        let last = result.last_mut().unwrap();
        if interval[0] <= last[1] {
            last[1] = last[1].max(interval[1]);
        } else {
            result.push(interval.clone());
        }
    }

    result
}
```

### 2. Insert Interval
```rust
fn insert(intervals: Vec<Vec<i32>>, new_interval: Vec<i32>) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    let mut new_interval = new_interval;
    let mut i = 0;

    // Add intervals before new_interval
    while i < intervals.len() && intervals[i][1] < new_interval[0] {
        result.push(intervals[i].clone());
        i += 1;
    }

    // Merge overlapping intervals
    while i < intervals.len() && intervals[i][0] <= new_interval[1] {
        new_interval[0] = new_interval[0].min(intervals[i][0]);
        new_interval[1] = new_interval[1].max(intervals[i][1]);
        i += 1;
    }
    result.push(new_interval);

    // Add remaining intervals
    while i < intervals.len() {
        result.push(intervals[i].clone());
        i += 1;
    }

    result
}
```

### 3. Non-overlapping Intervals (Minimum Removals)
```rust
fn erase_overlap_intervals(mut intervals: Vec<Vec<i32>>) -> i32 {
    if intervals.is_empty() { return 0; }

    intervals.sort_by_key(|v| v[1]);
    let mut count = 0;
    let mut last_end = intervals[0][1];

    for interval in &intervals[1..] {
        if interval[0] < last_end {
            count += 1;
        } else {
            last_end = interval[1];
        }
    }

    count
}
```

### 4. Meeting Rooms
```rust
fn can_attend_meetings(mut intervals: Vec<Vec<i32>>) -> bool {
    intervals.sort_by_key(|v| v[0]);

    for i in 1..intervals.len() {
        if intervals[i][0] < intervals[i-1][1] {
            return false;
        }
    }

    true
}
```

### 5. Meeting Rooms II (Minimum Rooms)
```rust
fn min_meeting_rooms(mut intervals: Vec<Vec<i32>>) -> i32 {
    if intervals.is_empty() { return 0; }

    intervals.sort_by_key(|v| v[0]);
    let mut heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();

    for interval in &intervals {
        if let Some(&Reverse(end)) = heap.peek() {
            if end <= interval[0] {
                heap.pop();
            }
        }
        heap.push(Reverse(interval[1]));
    }

    heap.len() as i32
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Insert Interval | Medium | Three phases |
| 2 | Merge Intervals | Medium | Sort + merge |
| 3 | Non-overlapping Intervals | Medium | Sort by end, count removals |
| 4 | Meeting Rooms | Easy | Check overlaps |
| 5 | Meeting Rooms II | Medium | Min heap for end times |
| 6 | Minimum Interval to Include Queries | Hard | Sweep line + heap |

## Tips for Rust

1. **Sort by start or end**: Depending on the problem.
2. **`BinaryHeap<Reverse<i32>>`**: For tracking minimum end times.
3. **Compare with last merged**: Check if current interval overlaps with the last one in result.
4. **Edge cases**: Empty input, single interval, no overlaps.
