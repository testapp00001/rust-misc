// ============================================================================
// Problem: Merge Triplets to Form Target Triplet (LeetCode #1899)
// ============================================================================
// Given a list of triplets and a target triplet, return true if you can
// form the target by merging triplets (taking max of each position).
//
// ============================================================================
// APPROACH: Greedy (O(n) time, O(1) space)
// ============================================================================
//
// A triplet is useful if all its values are <= target values.
// Merge all useful triplets and check if we get the target.
// ============================================================================

pub fn merge_triplets(triplets: &[Vec<i32>], target: Vec<i32>) -> bool {
    let mut result = [0, 0, 0];

    for triplet in triplets {
        // Skip triplets that exceed target in any position
        if triplet[0] > target[0] || triplet[1] > target[1] || triplet[2] > target[2] {
            continue;
        }
        // Merge (take max)
        result[0] = result[0].max(triplet[0]);
        result[1] = result[1].max(triplet[1]);
        result[2] = result[2].max(triplet[2]);
    }

    result[0] == target[0] && result[1] == target[1] && result[2] == target[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_merge() {
        assert!(merge_triplets(
            &[vec![2, 5, 3], vec![1, 8, 4], vec![1, 7, 5]],
            vec![2, 7, 5]
        ));
    }

    #[test]
    fn test_cannot_merge() {
        assert!(!merge_triplets(
            &[vec![3, 4, 5], vec![4, 5, 6]],
            vec![3, 2, 5]
        ));
    }

    #[test]
    fn test_single() {
        assert!(merge_triplets(&[vec![1, 2, 3]], vec![1, 2, 3]));
    }
}
