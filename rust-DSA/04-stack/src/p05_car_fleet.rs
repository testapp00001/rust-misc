// ============================================================================
// Problem: Car Fleet (LeetCode #1713, LeetCode #853)
// ============================================================================
// There are `n` cars going to the same destination along a one-lane road.
// The destination is `target` miles away. Each car has a position and speed.
//
// A car can never pass another car, but it can catch up and travel at the
// same speed as the car ahead. A fleet is a group of cars driving at the
// same position and speed (the lead car's speed).
//
// Return the number of car fleets that will arrive at the destination.
//
// Example:
//   target = 12, position = [10,8,0,5,3], speed = [2,4,1,1,3]
//   Output: 3
//
// ============================================================================
// APPROACH: Sort + Greedy (O(n log n) time, O(n) space)
// ============================================================================
//
// 1. Sort cars by position (descending — closest to target first).
// 2. For each car, calculate time to reach target: (target - pos) / speed.
// 3. If a car takes longer than the car ahead, it forms a new fleet.
//    Otherwise, it joins the fleet ahead (and is "absorbed").
//
// Use a stack to track fleet arrival times.
// ============================================================================

pub fn car_fleet(target: i32, position: Vec<i32>, speed: Vec<i32>) -> i32 {
    let mut cars: Vec<(i32, i32)> = position.into_iter().zip(speed).collect();
    cars.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by position descending

    let mut stack: Vec<f64> = Vec::new();

    for (pos, spd) in cars {
        let time = (target - pos) as f64 / spd as f64;
        // If this car takes longer than the fleet ahead, it's a new fleet
        if stack.is_empty() || time > *stack.last().unwrap() {
            stack.push(time);
        }
    }

    stack.len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(car_fleet(12, vec![10, 8, 0, 5, 3], vec![2, 4, 1, 1, 3]), 3);
    }

    #[test]
    fn test_all_same_speed() {
        // All cars have same speed but different positions - they never meet
        assert_eq!(car_fleet(10, vec![0, 4, 2], vec![2, 2, 2]), 3);
    }

    #[test]
    fn test_no_meet() {
        assert_eq!(car_fleet(10, vec![0, 5], vec![1, 2]), 2);
    }

    #[test]
    fn test_single() {
        assert_eq!(car_fleet(10, vec![0], vec![1]), 1);
    }
}
