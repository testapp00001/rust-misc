// ============================================================================
// Problem: Combinations (LeetCode #77)
// ============================================================================
// Given two integers `n` and `k`, return all possible combinations of k
// numbers chosen from 1 to n.
//
// Example:
//   n = 4, k = 2
//   Output: [[1,2],[1,3],[1,4],[2,3],[2,4],[3,4]]
//
// ============================================================================
// APPROACH: Backtracking (O(C(n,k) * k) time, O(k) space)
// ============================================================================
//
// Similar to subsets, but only collect combinations of size k.
// Use `start` to ensure combinations are in increasing order.
// ============================================================================

pub fn combine(n: i32, k: i32) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    backtrack(1, n, k, &mut current, &mut result);
    result
}

fn backtrack(start: i32, n: i32, k: i32, current: &mut Vec<i32>, result: &mut Vec<Vec<i32>>) {
    if current.len() == k as usize {
        result.push(current.clone());
        return;
    }

    for i in start..=n {
        current.push(i);
        backtrack(i + 1, n, k, current, result);
        current.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut v: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
        v.sort();
        v
    }

    #[test]
    fn test_basic() {
        let result = sorted(combine(4, 2));
        let expected = sorted(vec![
            vec![1, 2], vec![1, 3], vec![1, 4],
            vec![2, 3], vec![2, 4], vec![3, 4],
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_k_equals_n() {
        assert_eq!(combine(3, 3), vec![vec![1, 2, 3]]);
    }

    #[test]
    fn test_k_equals_one() {
        let result = sorted(combine(3, 1));
        assert_eq!(result, sorted(vec![vec![1], vec![2], vec![3]]));
    }
}
