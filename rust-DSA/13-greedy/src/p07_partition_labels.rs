// ============================================================================
// Problem: Partition Labels (LeetCode #763)
// ============================================================================
// Given a string `s`, partition it into as many parts as possible so that
// each letter appears in at most one part. Return the sizes.
//
// Example:
//   Input:  "ababcbacadefegdehijhklij"
//   Output: [9,7,8]
//
// ============================================================================
// APPROACH: Greedy (O(n) time, O(1) space)
// ============================================================================
//
// 1. Record the last occurrence of each character.
// 2. Iterate through the string, tracking the end of the current partition.
// 3. When we reach the end, we've found a partition.
// ============================================================================

pub fn partition_labels(s: &str) -> Vec<i32> {
    let chars: Vec<char> = s.chars().collect();
    let mut last = [0usize; 26];

    // Record last occurrence of each character
    for (i, &c) in chars.iter().enumerate() {
        last[(c as u8 - b'a') as usize] = i;
    }

    let mut result = Vec::new();
    let mut start = 0;
    let mut end = 0;

    for (i, &c) in chars.iter().enumerate() {
        end = end.max(last[(c as u8 - b'a') as usize]);
        if i == end {
            result.push((end - start + 1) as i32);
            start = i + 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            partition_labels("ababcbacadefegdehijhklij"),
            vec![9, 7, 8]
        );
    }

    #[test]
    fn test_single() {
        assert_eq!(partition_labels("abc"), vec![1, 1, 1]);
    }

    #[test]
    fn test_all_same() {
        assert_eq!(partition_labels("aaaa"), vec![4]);
    }

    #[test]
    fn test_two_partitions() {
        assert_eq!(partition_labels("abac"), vec![3, 1]);
    }
}
