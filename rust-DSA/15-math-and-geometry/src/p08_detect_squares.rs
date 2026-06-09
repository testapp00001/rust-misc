// ============================================================================
// Problem: Detect Squares (LeetCode #2013)
// ============================================================================
// You are given a stream of points. Implement:
// - add(point): Add a point.
// - count(point): Count squares that can be formed with point as one corner.
//
// ============================================================================
// APPROACH: HashMap (O(1) add, O(n) count)
// ============================================================================
//
// Store points in a HashMap<(x,y), count>.
// For count(point):
//   For each other point with the same y-coordinate:
//     Check if the two diagonal points exist.
// ============================================================================


use std::collections::HashMap;

pub struct DetectSquares {
    // TODO: Define fields
}

impl DetectSquares {
    pub fn new() -> Self {
        todo!("Implement new")
    }

    pub fn add(&mut self, point: Vec<i32>) {
        todo!("Implement add")
    }

    pub fn count(&self, point: Vec<i32>) -> i32 {
        todo!("Implement count")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut ds = DetectSquares::new();
        ds.add(vec![3, 10]);
        ds.add(vec![11, 2]);
        ds.add(vec![3, 2]);
        assert_eq!(ds.count(vec![11, 10]), 1);
        assert_eq!(ds.count(vec![14, 8]), 0);
        ds.add(vec![11, 2]);
        assert_eq!(ds.count(vec![11, 10]), 2);
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
        //     let _ = new(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}