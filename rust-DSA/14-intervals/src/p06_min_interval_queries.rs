// ============================================================================
// Problem: Minimum Interval to Include Each Query (LeetCode #1851)
// ============================================================================
// Given intervals and queries, for each query find the smallest interval
// that contains it. If none, return -1.
//
// ============================================================================
// APPROACH: Sort + Sweep Line + Min Heap (O((n+q) log(n+q)) time, O(n+q) space)
// ============================================================================
//
// 1. Sort intervals by start, queries by value.
// 2. Use a min heap of (size, end) for active intervals.
// 3. Sweep through queries, adding intervals that start <= query.
// 4. Remove intervals that end < query.
// 5. The smallest active interval is the answer.
// ============================================================================

use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub fn min_interval(intervals: Vec<Vec<i32>>, queries: Vec<i32>) -> Vec<i32> {
    let mut intervals = intervals;
    intervals.sort_by_key(|v| v[0]);

    let mut queries: Vec<(i32, usize)> = queries
        .into_iter()
        .enumerate()
        .map(|(i, q)| (q, i))
        .collect();
    queries.sort_by_key(|&(q, _)| q);

    let mut result = vec![-1; queries.len()];
    let mut heap: BinaryHeap<Reverse<(i32, i32)>> = BinaryHeap::new(); // (size, end)
    let mut i = 0;

    for (query, idx) in queries {
        // Add intervals that start <= query
        while i < intervals.len() && intervals[i][0] <= query {
            let size = intervals[i][1] - intervals[i][0] + 1;
            heap.push(Reverse((size, intervals[i][1])));
            i += 1;
        }

        // Remove intervals that end < query
        while let Some(&Reverse((_, end))) = heap.peek() {
            if end < query {
                heap.pop();
            } else {
                break;
            }
        }

        // The smallest active interval
        if let Some(&Reverse((size, _))) = heap.peek() {
            result[idx] = size;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let intervals = vec![vec![1, 4], vec![2, 4], vec![3, 6], vec![4, 4]];
        let queries = vec![2, 3, 4, 5];
        assert_eq!(min_interval(intervals, queries), vec![3, 3, 1, 4]);
    }

    #[test]
    fn test_no_cover() {
        let intervals = vec![vec![2, 3]];
        let queries = vec![1, 4];
        assert_eq!(min_interval(intervals, queries), vec![-1, -1]);
    }
}
