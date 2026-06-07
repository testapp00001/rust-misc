// ============================================================================
// Problem: Hand of Straights (LeetCode #846)
// ============================================================================
// Given an array of integers `hand` and an integer `groupSize`, return true
// if the hand can be rearranged into groups of `groupSize` consecutive cards.
//
// Example:
//   hand = [1,2,3,6,2,3,4,7,8], groupSize = 3
//   Output: true  ([[1,2,3],[2,3,4],[6,7,8]])
//
// ============================================================================
// APPROACH: Greedy with HashMap (O(n log n) time, O(n) space)
// ============================================================================
//
// 1. Count card frequencies.
// 2. Sort unique cards.
// 3. For each card (in order), try to form a group starting from it.
// 4. If we can't form a complete group → return false.
// ============================================================================

use std::collections::HashMap;

pub fn is_n_straight_hand(hand: &[i32], group_size: i32) -> bool {
    if hand.len() % group_size as usize != 0 {
        return false;
    }

    let mut count: HashMap<i32, i32> = HashMap::new();
    for &card in hand {
        *count.entry(card).or_insert(0) += 1;
    }

    let mut cards: Vec<i32> = count.keys().copied().collect();
    cards.sort();

    for &card in &cards {
        if let Some(&cnt) = count.get(&card) {
            if cnt > 0 {
                for i in 0..group_size {
                    let next = card + i;
                    match count.get_mut(&next) {
                        Some(c) if *c >= cnt => *c -= cnt,
                        _ => return false,
                    }
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert!(is_n_straight_hand(&[1, 2, 3, 6, 2, 3, 4, 7, 8], 3));
    }

    #[test]
    fn test_impossible() {
        assert!(!is_n_straight_hand(&[1, 2, 3, 4, 5], 4));
    }

    #[test]
    fn test_single_group() {
        assert!(is_n_straight_hand(&[1, 2, 3], 3));
    }
}
