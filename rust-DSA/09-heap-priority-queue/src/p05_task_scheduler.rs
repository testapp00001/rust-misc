// ============================================================================
// Problem: Task Scheduler (LeetCode #621)
// ============================================================================
// Given a char array of tasks and an integer n (cooldown period), return
// the least number of units of time to finish all tasks. Each task takes
// one unit, and the same task must have at least n units between them.
//
// Example:
//   tasks = ["A","A","A","B","B","B"], n = 2
//   Output: 8  (A -> B -> idle -> A -> B -> idle -> A -> B)
//
// ============================================================================
// APPROACH: Greedy + Max Heap (O(m) time, O(1) space — 26 letters)
// ============================================================================
//
// Key insight: The most frequent task determines the minimum time.
//
// Formula: max(tasks.len(), (max_freq - 1) * (n + 1) + count_max_freq)
//
// Where:
// - max_freq: frequency of the most common task
// - count_max_freq: number of tasks with max_freq
// ============================================================================

use std::collections::HashMap;

pub fn least_interval(tasks: Vec<char>, n: i32) -> i32 {
    let mut freq: HashMap<char, i32> = HashMap::new();
    for &task in &tasks {
        *freq.entry(task).or_insert(0) += 1;
    }

    let max_freq = *freq.values().max().unwrap();
    let max_count = freq.values().filter(|&&v| v == max_freq).count() as i32;

    let formula = (max_freq - 1) * (n + 1) + max_count;
    (tasks.len() as i32).max(formula)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let tasks: Vec<char> = vec!['A', 'A', 'A', 'B', 'B', 'B'];
        assert_eq!(least_interval(tasks, 2), 8);
    }

    #[test]
    fn test_no_idle() {
        let tasks: Vec<char> = vec!['A', 'B', 'C', 'A', 'B', 'C'];
        assert_eq!(least_interval(tasks, 1), 6);
    }

    #[test]
    fn test_single_task() {
        let tasks: Vec<char> = vec!['A', 'A', 'A'];
        assert_eq!(least_interval(tasks, 3), 9);
    }
}
