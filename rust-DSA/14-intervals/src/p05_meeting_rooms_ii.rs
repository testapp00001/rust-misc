// ============================================================================
// Problem: Meeting Rooms II (LeetCode #253)
// ============================================================================
// Given an array of meeting time intervals, return the minimum number of
// conference rooms required.
//
// Example:
//   Input:  [[0,30],[5,10],[15,20]]
//   Output: 2
//
// ============================================================================
// APPROACH: Sort + Min Heap (O(n log n) time, O(n) space)
// ============================================================================
//
// 1. Sort meetings by start time.
// 2. Use a min heap to track end times of ongoing meetings.
// 3. For each meeting:
//    - If the earliest ending meeting has ended → reuse that room.
//    - Otherwise → allocate a new room.
// 4. The heap size is the number of rooms needed.
//
// Alternative: Chronological ordering (two sorted arrays).
// ============================================================================

use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub fn min_meeting_rooms(mut intervals: Vec<Vec<i32>>) -> i32 {
    if intervals.is_empty() {
        return 0;
    }

    intervals.sort_by_key(|v| v[0]);
    let mut heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();

    for interval in &intervals {
        // If the earliest ending meeting has ended, reuse that room
        if let Some(&Reverse(end)) = heap.peek() {
            if end <= interval[0] {
                heap.pop();
            }
        }
        heap.push(Reverse(interval[1]));
    }

    heap.len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            min_meeting_rooms(vec![vec![0, 30], vec![5, 10], vec![15, 20]]),
            2
        );
    }

    #[test]
    fn test_one_room() {
        assert_eq!(
            min_meeting_rooms(vec![vec![0, 5], vec![10, 15]]),
            1
        );
    }

    #[test]
    fn test_all_overlap() {
        assert_eq!(
            min_meeting_rooms(vec![vec![0, 10], vec![5, 15], vec![10, 20]]),
            2
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(min_meeting_rooms(vec![]), 0);
    }
}
