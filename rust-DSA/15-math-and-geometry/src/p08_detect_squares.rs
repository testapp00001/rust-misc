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
    points: HashMap<(i32, i32), i32>,
}

impl DetectSquares {
    pub fn new() -> Self {
        DetectSquares {
            points: HashMap::new(),
        }
    }

    pub fn add(&mut self, point: Vec<i32>) {
        *self.points.entry((point[0], point[1])).or_insert(0) += 1;
    }

    pub fn count(&self, point: Vec<i32>) -> i32 {
        let (qx, qy) = (point[0], point[1]);
        let mut result = 0;

        for (&(x, y), &cnt) in &self.points {
            if x == qx || y == qy || (x - qx).abs() != (y - qy).abs() {
                continue;
            }
            // Check diagonal points
            let c1 = self.points.get(&(x, qy)).copied().unwrap_or(0);
            let c2 = self.points.get(&(qx, y)).copied().unwrap_or(0);
            result += cnt * c1 * c2;
        }

        result
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
}
