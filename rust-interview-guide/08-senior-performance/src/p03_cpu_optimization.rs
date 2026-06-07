/// Problem: CPU Optimization
///
/// Master CPU optimization in Rust.
///
/// Key Concepts:
/// - Algorithm optimization
/// - Loop optimization
/// - Branch prediction
/// - Inlining
/// - Vectorization

/// Problem 1: Algorithm optimization
/// Use efficient algorithm
pub fn efficient_sum(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Problem 2: Loop optimization
/// Optimize loops
pub fn optimized_loop(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        sum += x;
    }
    sum
}

/// Problem 3: Branch prediction
/// Help branch predictor
pub fn branch_prediction(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        if x > 0 {
            sum += x;
        }
    }
    sum
}

/// Problem 4: Inlining
/// Use inline for hot functions
#[inline(always)]
pub fn inline_function(x: i32) -> i32 {
    x * 2
}

/// Problem 5: Avoid bounds checking
/// Use unchecked access
pub fn unchecked_access(data: &[i32]) -> i32 {
    unsafe { *data.get_unchecked(0) }
}

/// Problem 6: Use iterators
/// Use zero-cost iterators
pub fn iterator_sum(data: &[i32]) -> i32 {
    data.iter().copied().sum()
}

/// Problem 7: Avoid allocations in hot path
/// Reuse allocations
pub fn reuse_allocation(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(data.len());
    for &x in data {
        result.push(x * 2);
    }
    result
}

/// Problem 8: Use SIMD (simulated)
/// Simulate SIMD operations
pub fn simd_add(a: &[i32], b: &[i32]) -> Vec<i32> {
    a.iter().zip(b.iter()).map(|(&x, &y)| x + y).collect()
}

/// Problem 9: Cache-friendly access
/// Access data sequentially
pub fn cache_friendly(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        sum += x;
    }
    sum
}

/// Problem 10: Avoid function call overhead
/// Inline hot functions
#[inline]
pub fn hot_function(x: i32) -> i32 {
    x + 1
}

/// Problem 11: Use lookup tables
/// Use lookup tables for repeated calculations
pub fn lookup_table() -> [i32; 256] {
    let mut table = [0; 256];
    for i in 0..256 {
        table[i] = i as i32 * 2;
    }
    table
}

/// Problem 12: Avoid division
/// Use multiplication instead
pub fn avoid_division(x: f64) -> f64 {
    x * 0.5 // Instead of x / 2.0
}

/// Problem 13: Use bit operations
/// Use bit operations for efficiency
pub fn bit_operations(x: i32) -> i32 {
    x << 1 // Multiply by 2
}

/// Problem 14: Avoid unnecessary comparisons
/// Minimize comparisons
pub fn minimize_comparisons(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

/// Problem 15: Use const evaluation
/// Use const for compile-time calculation
pub const fn const_factorial(n: u64) -> u64 {
    match n {
        0 => 1,
        _ => n * const_factorial(n - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_efficient_sum() {
        assert_eq!(efficient_sum(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_optimized_loop() {
        assert_eq!(optimized_loop(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_branch_prediction() {
        assert_eq!(branch_prediction(&[1, -2, 3, -4, 5]), 9);
    }

    #[test]
    fn test_inline_function() {
        assert_eq!(inline_function(21), 42);
    }

    #[test]
    fn test_unchecked_access() {
        assert_eq!(unchecked_access(&[42, 2, 3]), 42);
    }

    #[test]
    fn test_iterator_sum() {
        assert_eq!(iterator_sum(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_reuse_allocation() {
        assert_eq!(reuse_allocation(&[1, 2, 3]), vec![2, 4, 6]);
    }

    #[test]
    fn test_simd_add() {
        assert_eq!(simd_add(&[1, 2, 3], &[4, 5, 6]), vec![5, 7, 9]);
    }

    #[test]
    fn test_cache_friendly() {
        assert_eq!(cache_friendly(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_hot_function() {
        assert_eq!(hot_function(41), 42);
    }

    #[test]
    fn test_lookup_table() {
        let table = lookup_table();
        assert_eq!(table[21], 42);
    }

    #[test]
    fn test_avoid_division() {
        assert!((avoid_division(84.0) - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bit_operations() {
        assert_eq!(bit_operations(21), 42);
    }

    #[test]
    fn test_minimize_comparisons() {
        assert_eq!(minimize_comparisons(42, 21), 42);
    }

    #[test]
    fn test_const_factorial() {
        assert_eq!(const_factorial(5), 120);
    }
}
