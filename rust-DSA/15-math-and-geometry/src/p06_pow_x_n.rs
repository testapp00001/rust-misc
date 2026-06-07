// ============================================================================
// Problem: Pow(x, n) (LeetCode #50)
// ============================================================================
// Implement pow(x, n), which calculates x raised to the power n.
//
// Example:
//   pow(2.0, 10) = 1024.0
//   pow(2.0, -2) = 0.25
//
// ============================================================================
// APPROACH: Fast Exponentiation (O(log n) time, O(1) space)
// ============================================================================
//
// Use binary exponentiation:
// - If n is even: x^n = (x²)^(n/2)
// - If n is odd:  x^n = x * (x²)^((n-1)/2)
//
// Handle negative exponents: x^(-n) = (1/x)^n
// ============================================================================

pub fn my_pow(x: f64, n: i32) -> f64 {
    let mut x = x;
    let mut n = n as i64;

    if n < 0 {
        x = 1.0 / x;
        n = -n;
    }

    let mut result = 1.0;
    while n > 0 {
        if n % 2 == 1 {
            result *= x;
        }
        x *= x;
        n /= 2;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn test_basic() {
        assert!(approx_eq(my_pow(2.0, 10), 1024.0));
    }

    #[test]
    fn test_negative() {
        assert!(approx_eq(my_pow(2.0, -2), 0.25));
    }

    #[test]
    fn test_zero() {
        assert!(approx_eq(my_pow(2.0, 0), 1.0));
    }

    #[test]
    fn test_one() {
        assert!(approx_eq(my_pow(2.0, 1), 2.0));
    }

    #[test]
    fn test_fractional() {
        assert!(approx_eq(my_pow(0.5, 2), 0.25));
    }
}
