/// Problem: SIMD
///
/// Master SIMD in Rust.
///
/// Key Concepts:
/// - SIMD types
/// - SIMD operations
/// - Vectorization
/// - Platform-specific SIMD
/// - Auto-vectorization

/// Problem 1: Basic SIMD (simulated)
/// Simulate SIMD operations
pub fn simd_add(a: &[i32], b: &[i32]) -> Vec<i32> {
    a.iter().zip(b.iter()).map(|(&x, &y)| x + y).collect()
}

/// Problem 2: SIMD multiply
/// Simulate SIMD multiply
pub fn simd_multiply(a: &[i32], b: &[i32]) -> Vec<i32> {
    a.iter().zip(b.iter()).map(|(&x, &y)| x * y).collect()
}

/// Problem 3: SIMD dot product
/// Simulate SIMD dot product
pub fn simd_dot_product(a: &[i32], b: &[i32]) -> i32 {
    a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum()
}

/// Problem 4: SIMD sum
/// Simulate SIMD sum
pub fn simd_sum(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Problem 5: SIMD min/max
/// Simulate SIMD min/max
pub fn simd_min_max(data: &[i32]) -> (i32, i32) {
    let min = *data.iter().min().unwrap();
    let max = *data.iter().max().unwrap();
    (min, max)
}

/// Problem 6: SIMD comparison
/// Simulate SIMD comparison
pub fn simd_compare(a: &[i32], b: &[i32]) -> Vec<bool> {
    a.iter().zip(b.iter()).map(|(&x, &y)| x > y).collect()
}

/// Problem 7: SIMD absolute value
/// Simulate SIMD absolute value
pub fn simd_abs(data: &[i32]) -> Vec<i32> {
    data.iter().map(|&x| x.abs()).collect()
}

/// Problem 8: SIMD square root (simulated)
/// Simulate SIMD square root
pub fn simd_sqrt(data: &[f64]) -> Vec<f64> {
    data.iter().map(|&x| x.sqrt()).collect()
}

/// Problem 9: SIMD shuffle (simulated)
/// Simulate SIMD shuffle
pub fn simd_shuffle(a: &[i32], b: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    result.extend_from_slice(a);
    result.extend_from_slice(b);
    result
}

/// Problem 10: SIMD blend (simulated)
/// Simulate SIMD blend
pub fn simd_blend(a: &[i32], b: &[i32], mask: &[bool]) -> Vec<i32> {
    a.iter()
        .zip(b.iter())
        .zip(mask.iter())
        .map(|((&x, &y), &m)| if m { x } else { y })
        .collect()
}

/// Problem 11: SIMD horizontal sum
/// Simulate SIMD horizontal sum
pub fn simd_horizontal_sum(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Problem 12: SIMD prefix sum
/// Simulate SIMD prefix sum
pub fn simd_prefix_sum(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(data.len());
    let mut sum = 0;
    for &x in data {
        sum += x;
        result.push(sum);
    }
    result
}

/// Problem 13: SIMD convolution (simulated)
/// Simulate SIMD convolution
pub fn simd_convolution(data: &[f64], kernel: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        let mut sum = 0.0;
        for j in 0..kernel.len() {
            if i + j < data.len() {
                sum += data[i + j] * kernel[j];
            }
        }
        result.push(sum);
    }
    result
}

/// Problem 14: SIMD matrix multiply (simulated)
/// Simulate SIMD matrix multiply
pub fn simd_matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut result = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

/// Problem 15: SIMD with auto-vectorization
/// Let compiler auto-vectorize
pub fn auto_vectorize(data: &[i32]) -> i32 {
    data.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_add() {
        assert_eq!(simd_add(&[1, 2, 3], &[4, 5, 6]), vec![5, 7, 9]);
    }

    #[test]
    fn test_simd_multiply() {
        assert_eq!(simd_multiply(&[1, 2, 3], &[4, 5, 6]), vec![4, 10, 18]);
    }

    #[test]
    fn test_simd_dot_product() {
        assert_eq!(simd_dot_product(&[1, 2, 3], &[4, 5, 6]), 32);
    }

    #[test]
    fn test_simd_sum() {
        assert_eq!(simd_sum(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_simd_min_max() {
        assert_eq!(simd_min_max(&[3, 1, 4, 1, 5, 9]), (1, 9));
    }

    #[test]
    fn test_simd_compare() {
        assert_eq!(simd_compare(&[1, 2, 3], &[3, 2, 1]), vec![false, false, true]);
    }

    #[test]
    fn test_simd_abs() {
        assert_eq!(simd_abs(&[-1, 2, -3, 4, -5]), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_simd_sqrt() {
        let result = simd_sqrt(&[4.0, 9.0, 16.0]);
        assert!((result[0] - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_simd_shuffle() {
        assert_eq!(simd_shuffle(&[1, 2], &[3, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_simd_blend() {
        assert_eq!(
            simd_blend(&[1, 2, 3], &[4, 5, 6], &[true, false, true]),
            vec![1, 5, 3]
        );
    }

    #[test]
    fn test_simd_horizontal_sum() {
        assert_eq!(simd_horizontal_sum(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_simd_prefix_sum() {
        assert_eq!(simd_prefix_sum(&[1, 2, 3, 4, 5]), vec![1, 3, 6, 10, 15]);
    }

    #[test]
    fn test_simd_convolution() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let kernel = vec![1.0, 0.5];
        let result = simd_convolution(&data, &kernel);
        assert!(result.len() > 0);
    }

    #[test]
    fn test_simd_matrix_multiply() {
        let a = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let b = vec![vec![5.0, 6.0], vec![7.0, 8.0]];
        let result = simd_matrix_multiply(&a, &b);
        assert_eq!(result[0][0], 19.0);
    }

    #[test]
    fn test_auto_vectorize() {
        assert_eq!(auto_vectorize(&[1, 2, 3, 4, 5]), 15);
    }
}
