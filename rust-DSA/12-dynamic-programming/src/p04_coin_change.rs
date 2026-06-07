// ============================================================================
// Problem: Coin Change (LeetCode #322)
// ============================================================================
// You are given coins of different denominations and a total amount. Return
// the fewest number of coins needed to make up that amount. Return -1 if
// it cannot be made.
//
// Example:
//   coins = [1, 2, 5], amount = 11
//   Output: 3  (5 + 5 + 1)
//
// ============================================================================
// APPROACH: Dynamic Programming (O(amount * coins) time, O(amount) space)
// ============================================================================
//
// dp[i] = minimum coins to make amount i
// dp[0] = 0
// dp[i] = min(dp[i - coin] + 1) for each coin where i - coin >= 0
// ============================================================================

pub fn coin_change(coins: &[i32], amount: i32) -> i32 {
    let amount = amount as usize;
    let mut dp = vec![i32::MAX; amount + 1];
    dp[0] = 0;

    for i in 1..=amount {
        for &coin in coins {
            let coin = coin as usize;
            if coin <= i && dp[i - coin] != i32::MAX {
                dp[i] = dp[i].min(dp[i - coin] + 1);
            }
        }
    }

    if dp[amount] == i32::MAX { -1 } else { dp[amount] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(coin_change(&[1, 2, 5], 11), 3);
    }

    #[test]
    fn test_impossible() {
        assert_eq!(coin_change(&[2], 3), -1);
    }

    #[test]
    fn test_zero() {
        assert_eq!(coin_change(&[1], 0), 0);
    }

    #[test]
    fn test_single_coin() {
        assert_eq!(coin_change(&[1], 2), 2);
    }

    #[test]
    fn test_large() {
        assert_eq!(coin_change(&[1, 5, 10, 25], 30), 2);
    }
}
