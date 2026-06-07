//! # SIMD Intrinsics
//!
//! SIMD (Single Instruction, Multiple Data) processes multiple data elements
//! in a single CPU instruction. Rust supports SIMD through `std::arch` for
//! platform-specific intrinsics and auto-vectorization from the compiler.
//!
//! ## Key Concepts
//! - **std::arch**: Platform-specific SIMD intrinsics (x86_64, aarch64)
//! - **Auto-vectorization**: The compiler converts simple loops to SIMD automatically
//! - **Portable SIMD**: Using safe abstractions that work across platforms
//! - **Alignment**: SIMD often requires aligned memory for best performance

/// Adds two slices element-wise using a scalar loop.
/// The compiler may auto-vectorize this.
pub fn add_scalar(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
}

/// Multiplies two slices element-wise.
pub fn mul_scalar(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    for i in 0..a.len() {
        result[i] = a[i] * b[i];
    }
}

/// Dot product of two slices.
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let mut sum = 0.0f32;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

/// Horizontal sum of a slice.
pub fn hsum_scalar(data: &[f32]) -> f32 {
    data.iter().sum()
}

/// Fused multiply-add: result[i] = a[i] * b[i] + c[i]
pub fn fma_scalar(a: &[f32], b: &[f32], c: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), c.len());
    assert_eq!(a.len(), result.len());

    for i in 0..a.len() {
        result[i] = a[i] * b[i] + c[i];
    }
}

/// Min of two slices element-wise.
pub fn min_scalar(a: &[f32], b: &[f32], result: &mut [f32]) {
    for i in 0..a.len().min(b.len()) {
        result[i] = a[i].min(b[i]);
    }
}

/// Computes the L2 norm (Euclidean distance) of a vector.
pub fn l2_norm_scalar(data: &[f32]) -> f32 {
    let sum_sq: f32 = data.iter().map(|x| x * x).sum();
    sum_sq.sqrt()
}

/// Normalizes a vector to unit length.
pub fn normalize_scalar(data: &mut [f32]) {
    let norm = l2_norm_scalar(data);
    if norm > 0.0 {
        for val in data.iter_mut() {
            *val /= norm;
        }
    }
}

/// SIMD-friendly chunked processing.
/// Processes data in chunks that align with SIMD register width.
pub fn add_chunked_4(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    let len = a.len();
    let chunks = len / 4;
    let remainder = len % 4;

    // Process 4 elements at a time (SSE register width)
    for i in 0..chunks {
        let base = i * 4;
        result[base] = a[base] + b[base];
        result[base + 1] = a[base + 1] + b[base + 1];
        result[base + 2] = a[base + 2] + b[base + 2];
        result[base + 3] = a[base + 3] + b[base + 3];
    }

    // Handle remainder
    for i in (chunks * 4)..len {
        result[i] = a[i] + b[i];
    }
}

/// Processes 8 elements at a time (AVX register width).
pub fn add_chunked_8(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    let len = a.len();
    let chunks = len / 8;

    for i in 0..chunks {
        let base = i * 8;
        result[base] = a[base] + b[base];
        result[base + 1] = a[base + 1] + b[base + 1];
        result[base + 2] = a[base + 2] + b[base + 2];
        result[base + 3] = a[base + 3] + b[base + 3];
        result[base + 4] = a[base + 4] + b[base + 4];
        result[base + 5] = a[base + 5] + b[base + 5];
        result[base + 6] = a[base + 6] + b[base + 6];
        result[base + 7] = a[base + 7] + b[base + 7];
    }

    for i in (chunks * 8)..len {
        result[i] = a[i] + b[i];
    }
}

/// Auto-vectorization friendly: simple, predictable loop patterns.
/// The compiler can vectorize this because there are no data dependencies
/// between iterations.
pub fn square_in_place(data: &mut [f32]) {
    for val in data.iter_mut() {
        *val *= *val;
    }
}

/// Example showing why some loops DON'T auto-vectorize.
/// This has a loop-carried dependency (prefix sum) that prevents vectorization.
pub fn prefix_sum(data: &[f32], result: &mut [f32]) {
    if data.is_empty() {
        return;
    }
    result[0] = data[0];
    for i in 1..data.len() {
        result[i] = result[i - 1] + data[i]; // Depends on previous result
    }
}

/// SIMD-friendly absolute value.
pub fn abs_in_place(data: &mut [f32]) {
    for val in data.iter_mut() {
        *val = val.abs();
    }
}

/// Threshold operation: clamp values to a minimum.
pub fn threshold(data: &mut [f32], min_value: f32) {
    for val in data.iter_mut() {
        if *val < min_value {
            *val = min_value;
        }
    }
}

/// Compute the maximum element in a slice.
pub fn max_element(data: &[f32]) -> f32 {
    data.iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max)
}

/// SIMD-friendly conditional select: result[i] = if mask[i] { a[i] } else { b[i] }
pub fn conditional_select(mask: &[bool], a: &[f32], b: &[f32], result: &mut [f32]) {
    for i in 0..result.len().min(mask.len()).min(a.len()).min(b.len()) {
        result[i] = if mask[i] { a[i] } else { b[i] };
    }
}

/// Interleaves two arrays: [a0, b0, a1, b1, ...]
pub fn interleave(a: &[f32], b: &[f32], result: &mut [f32]) {
    for i in 0..a.len().min(b.len()) {
        result[i * 2] = a[i];
        result[i * 2 + 1] = b[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data(n: usize) -> (Vec<f32>, Vec<f32>) {
        let a: Vec<f32> = (0..n).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..n).map(|i| (i as f32) * 0.5).collect();
        (a, b)
    }

    #[test]
    fn test_add_scalar() {
        let (a, b) = test_data(16);
        let mut result = vec![0.0f32; 16];
        add_scalar(&a, &b, &mut result);

        for i in 0..16 {
            assert!((result[i] - (a[i] + b[i])).abs() < 0.001);
        }
    }

    #[test]
    fn test_mul_scalar() {
        let (a, b) = test_data(16);
        let mut result = vec![0.0f32; 16];
        mul_scalar(&a, &b, &mut result);

        for i in 0..16 {
            assert!((result[i] - (a[i] * b[i])).abs() < 0.001);
        }
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let result = dot_product_scalar(&a, &b);
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert!((result - 32.0).abs() < 0.001);
    }

    #[test]
    fn test_fma() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let c = vec![10.0, 20.0, 30.0];
        let mut result = vec![0.0; 3];
        fma_scalar(&a, &b, &c, &mut result);

        assert!((result[0] - 14.0).abs() < 0.001); // 1*4 + 10
        assert!((result[1] - 30.0).abs() < 0.001); // 2*5 + 20
        assert!((result[2] - 48.0).abs() < 0.001); // 3*6 + 30
    }

    #[test]
    fn test_add_chunked_4() {
        let (a, b) = test_data(16);
        let mut result_scalar = vec![0.0f32; 16];
        let mut result_chunked = vec![0.0f32; 16];

        add_scalar(&a, &b, &mut result_scalar);
        add_chunked_4(&a, &b, &mut result_chunked);

        for i in 0..16 {
            assert!((result_scalar[i] - result_chunked[i]).abs() < 0.001);
        }
    }

    #[test]
    fn test_add_chunked_8() {
        let (a, b) = test_data(32);
        let mut result_scalar = vec![0.0f32; 32];
        let mut result_chunked = vec![0.0f32; 32];

        add_scalar(&a, &b, &mut result_scalar);
        add_chunked_8(&a, &b, &mut result_chunked);

        for i in 0..32 {
            assert!((result_scalar[i] - result_chunked[i]).abs() < 0.001);
        }
    }

    #[test]
    fn test_add_non_aligned_length() {
        let (a, b) = test_data(13);
        let mut result = vec![0.0f32; 13];
        add_chunked_8(&a, &b, &mut result);

        for i in 0..13 {
            assert!((result[i] - (a[i] + b[i])).abs() < 0.001);
        }
    }

    #[test]
    fn test_square_in_place() {
        let mut data = vec![1.0, 2.0, 3.0, 4.0];
        square_in_place(&mut data);

        assert!((data[0] - 1.0).abs() < 0.001);
        assert!((data[1] - 4.0).abs() < 0.001);
        assert!((data[2] - 9.0).abs() < 0.001);
        assert!((data[3] - 16.0).abs() < 0.001);
    }

    #[test]
    fn test_prefix_sum() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let mut result = vec![0.0; 4];
        prefix_sum(&data, &mut result);

        assert!((result[0] - 1.0).abs() < 0.001);
        assert!((result[1] - 3.0).abs() < 0.001);
        assert!((result[2] - 6.0).abs() < 0.001);
        assert!((result[3] - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_abs_in_place() {
        let mut data = vec![-1.0, 2.0, -3.0, 4.0, -5.0];
        abs_in_place(&mut data);

        for &v in &data {
            assert!(v >= 0.0);
        }
        assert!((data[0] - 1.0).abs() < 0.001);
        assert!((data[2] - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_threshold() {
        let mut data = vec![0.1, 0.5, 0.3, 0.8, 0.2];
        threshold(&mut data, 0.4);

        assert!((data[0] - 0.4).abs() < 0.001);
        assert!((data[1] - 0.5).abs() < 0.001);
        assert!((data[2] - 0.4).abs() < 0.001);
        assert!((data[3] - 0.8).abs() < 0.001);
        assert!((data[4] - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_max_element() {
        let data = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0];
        assert!((max_element(&data) - 9.0).abs() < 0.001);
    }

    #[test]
    fn test_l2_norm() {
        let data = vec![3.0, 4.0];
        let norm = l2_norm_scalar(&data);
        assert!((norm - 5.0).abs() < 0.001); // 3-4-5 triangle
    }

    #[test]
    fn test_normalize() {
        let mut data = vec![3.0, 4.0];
        normalize_scalar(&mut data);
        assert!((l2_norm_scalar(&data) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_conditional_select() {
        let mask = vec![true, false, true, false];
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![10.0, 20.0, 30.0, 40.0];
        let mut result = vec![0.0; 4];
        conditional_select(&mask, &a, &b, &mut result);

        assert!((result[0] - 1.0).abs() < 0.001);
        assert!((result[1] - 20.0).abs() < 0.001);
        assert!((result[2] - 3.0).abs() < 0.001);
        assert!((result[3] - 40.0).abs() < 0.001);
    }

    #[test]
    fn test_interleave() {
        let a = vec![1.0, 3.0, 5.0];
        let b = vec![2.0, 4.0, 6.0];
        let mut result = vec![0.0; 6];
        interleave(&a, &b, &mut result);

        assert_eq!(result, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }
}
