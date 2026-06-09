// ============================================================================
// Problem: Gas Station (LeetCode #134)
// ============================================================================
// There are n gas stations in a circle. Given arrays `gas` and `cost`,
// return the starting station index to travel around the circle once,
// or -1 if impossible.
//
// Example:
//   gas  = [1,2,3,4,5]
//   cost = [3,4,5,1,2]
//   Output: 3
//
// ============================================================================
// APPROACH: Greedy (O(n) time, O(1) space)
// ============================================================================
//
// Key insight: If the total gas >= total cost, there's always a solution.
//
// Algorithm:
// 1. Track current tank and total surplus.
// 2. If tank goes negative at station i, reset start to i+1.
// 3. If total surplus >= 0, return start.
// ============================================================================

pub fn can_complete_circuit(gas: &[i32], cost: &[i32]) -> i32 {
    let mut total_surplus = 0;
    let mut current_surplus = 0;
    let mut start = 0;

    for i in 0..gas.len() {
        let surplus = gas[i] - cost[i];
        total_surplus += surplus;
        current_surplus += surplus;

        if current_surplus < 0 {
            start = i + 1;
            current_surplus = 0;
        }
    }

    if total_surplus >= 0 { start as i32 } else { -1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            can_complete_circuit(&[1, 2, 3, 4, 5], &[3, 4, 5, 1, 2]),
            3
        );
    }

    #[test]
    fn test_impossible() {
        assert_eq!(
            can_complete_circuit(&[2, 3, 4], &[3, 4, 3]),
            -1
        );
    }

    #[test]
    fn test_single() {
        assert_eq!(can_complete_circuit(&[5], &[4]), 0);
    }


    #[test]
    #[ignore]
    fn bench_performance() {
        // ⏱️  Benchmark test
        // Run: cargo test -p <package> bench_performance -- --ignored --nocapture
        //
        // To use: uncomment and customize the code below with your function
        // and realistic test data.
        //
        // let iterations = 10_000;
        // let input = /* generate your test input here */;
        // let start = std::time::Instant::now();
        // for _ in 0..iterations {
        //     let _ = can_complete_circuit(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}