// ============================================================================
// Problem: Combination Sum (LeetCode #39)
// ============================================================================
// Given an array of distinct integers `candidates` and a target, return all
// unique combinations where the chosen numbers sum to target. The same
// number may be chosen an unlimited number of times.
//
// Example:
//   candidates = [2,3,6,7], target = 7
//   Output: [[2,2,3],[7]]
//
// ============================================================================
// APPROACH: Backtracking (O(n^(t/m) * t) time, O(t/m) space)
// ============================================================================
//
// At each step:
// 1. If target == 0: found a valid combination.
// 2. If target < 0: invalid path, backtrack.
// 3. Try each candidate from `start` onwards (to avoid duplicates).
// 4. Allow reusing the same candidate (pass `i` not `i+1`).
// ============================================================================

pub fn combination_sum(candidates: &[i32], target: i32) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    backtrack(candidates, target, 0, &mut current, &mut result);
    result
}

fn backtrack(
    candidates: &[i32],
    remaining: i32,
    start: usize,
    current: &mut Vec<i32>,
    result: &mut Vec<Vec<i32>>,
) {
    if remaining == 0 {
        result.push(current.clone());
        return;
    }
    if remaining < 0 {
        return;
    }

    for i in start..candidates.len() {
        current.push(candidates[i]);
        backtrack(candidates, remaining - candidates[i], i, current, result);
        current.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut v: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
        for inner in &mut v {
            inner.sort();
        }
        v.sort();
        v
    }

    #[test]
    fn test_basic() {
        let result = sorted(combination_sum(&[2, 3, 6, 7], 7));
        assert_eq!(result, sorted(vec![vec![2, 2, 3], vec![7]]));
    }

    #[test]
    fn test_multiple() {
        let result = sorted(combination_sum(&[2, 3, 5], 8));
        let expected = sorted(vec![
            vec![2, 2, 2, 2], vec![2, 3, 3], vec![3, 5],
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_solution() {
        assert_eq!(combination_sum(&[2], 3), Vec::<Vec<i32>>::new());
    }
}
