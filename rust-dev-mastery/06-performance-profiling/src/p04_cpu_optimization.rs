//! # CPU Optimization
//!
//! Understanding CPU internals helps write faster code. This module covers
//! branch prediction, cache lines, data layout, loop optimization, and
//! inlining — all of which can have dramatic performance impacts.
//!
//! ## Key Concepts
//! - **Branch prediction**: CPUs speculatively execute branches; predictable branches are fast
//! - **Cache lines**: Data is loaded in 64-byte chunks; spatial locality matters
//! - **Data layout**: Struct field ordering and alignment affect cache performance
//! - **Loop optimization**: Unrolling, vectorization, and reducing loop-carried dependencies

/// Demonstrates branch prediction-friendly code.
/// Sorting data before processing branches makes them predictable.
pub fn sum_if_sorted(data: &[i64], threshold: i64) -> i64 {
    let mut sum = 0i64;
    for &val in data {
        if val > threshold {
            sum += val;
        }
    }
    sum
}

pub fn sum_if_unsorted(data: &[i64], threshold: i64) -> i64 {
    // Same function, but callers pass unsorted data
    sum_if_sorted(data, threshold)
}

/// Demonstrates the impact of data layout on cache performance.
/// AoS (Array of Structs) vs SoA (Struct of Arrays).

// AoS: Each element is a full struct, spread across cache lines
#[derive(Clone, Copy)]
pub struct ParticleAoS {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub mass: f64,
    pub charge: f64,
    pub spin: f64,
}

// SoA: Each field in its own contiguous array
pub struct ParticleSoA {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub mass: Vec<f64>,
    pub charge: Vec<f64>,
    pub spin: Vec<f64>,
}

impl ParticleSoA {
    pub fn new(capacity: usize) -> Self {
        ParticleSoA {
            x: Vec::with_capacity(capacity),
            y: Vec::with_capacity(capacity),
            z: Vec::with_capacity(capacity),
            mass: Vec::with_capacity(capacity),
            charge: Vec::with_capacity(capacity),
            spin: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, p: &ParticleAoS) {
        self.x.push(p.x);
        self.y.push(p.y);
        self.z.push(p.z);
        self.mass.push(p.mass);
        self.charge.push(p.charge);
        self.spin.push(p.spin);
    }

    pub fn len(&self) -> usize {
        self.x.len()
    }
}

/// Sum only the x-coordinates: SoA is much faster because all x values
/// are contiguous in memory (cache-friendly).
pub fn sum_x_aos(particles: &[ParticleAoS]) -> f64 {
    particles.iter().map(|p| p.x).sum()
}

pub fn sum_x_soa(particles: &ParticleSoA) -> f64 {
    particles.x.iter().sum()
}

/// Demonstrates loop optimization techniques.
pub fn manual_loop_sum(data: &[f64]) -> f64 {
    let mut sum = 0.0;
    for i in 0..data.len() {
        sum += data[i];
    }
    sum
}

/// Iterator-based sum (compiler can often optimize this better).
pub fn iterator_sum(data: &[f64]) -> f64 {
    data.iter().sum()
}

/// Chunked processing for better cache utilization.
pub fn chunked_sum(data: &[f64], chunk_size: usize) -> f64 {
    let mut sum = 0.0;
    for chunk in data.chunks(chunk_size) {
        for &val in chunk {
            sum += val;
        }
    }
    sum
}

/// Loop unrolling: process multiple elements per iteration.
/// Reduces loop overhead and enables better instruction pipelining.
pub fn unrolled_sum(data: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut i = 0;

    // Process 4 elements at a time
    while i + 4 <= data.len() {
        sum += data[i];
        sum += data[i + 1];
        sum += data[i + 2];
        sum += data[i + 3];
        i += 4;
    }

    // Handle remaining elements
    while i < data.len() {
        sum += data[i];
        i += 1;
    }

    sum
}

/// Demonstrates reducing loop-carried dependencies.
/// Multiple accumulators allow the CPU to execute additions in parallel.
pub fn multi_accumulator_sum(data: &[f64]) -> f64 {
    let mut a = 0.0;
    let mut b = 0.0;
    let mut c = 0.0;
    let mut d = 0.0;

    let mut i = 0;
    while i + 4 <= data.len() {
        a += data[i];
        b += data[i + 1];
        c += data[i + 2];
        d += data[i + 3];
        i += 4;
    }

    let mut sum = a + b + c + d;
    while i < data.len() {
        sum += data[i];
        i += 1;
    }
    sum
}

/// Demonstrates struct packing and alignment.
/// Grouping hot fields together improves cache utilization.
#[repr(C)]
pub struct PackedRecord {
    // Hot fields (accessed frequently)
    pub id: u32,
    pub value: f64,
    // Warm fields
    pub name: String,
    pub tags: Vec<String>,
    // Cold fields (rarely accessed)
    pub created_at: u64,
    pub metadata: Vec<u8>,
}

/// A benchmarkable matrix multiplication using naive triple-loop.
pub fn matmul_naive(a: &[Vec<f64>], b: &[Vec<f64>], c: &mut [Vec<f64>]) {
    let n = a.len();
    for i in 0..n {
        for j in 0..n {
            c[i][j] = 0.0;
            for k in 0..n {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
}

/// Cache-blocked matrix multiplication.
/// Processes sub-matrices that fit in L1 cache.
pub fn matmul_blocked(a: &[Vec<f64>], b: &[Vec<f64>], c: &mut [Vec<f64>], block_size: usize) {
    let n = a.len();

    // Zero out C
    for i in 0..n {
        for j in 0..n {
            c[i][j] = 0.0;
        }
    }

    for ii in (0..n).step_by(block_size) {
        for jj in (0..n).step_by(block_size) {
            for kk in (0..n).step_by(block_size) {
                let i_end = (ii + block_size).min(n);
                let j_end = (jj + block_size).min(n);
                let k_end = (kk + block_size).min(n);

                for i in ii..i_end {
                    for j in jj..j_end {
                        let mut sum = c[i][j];
                        for k in kk..k_end {
                            sum += a[i][k] * b[k][j];
                        }
                        c[i][j] = sum;
                    }
                }
            }
        }
    }
}

/// Demonstrates `#[inline]` hints for the compiler.
/// The compiler usually inlines small functions automatically, but
/// explicit hints help in hot paths.

#[inline(always)]
pub fn fast_square(x: f64) -> f64 {
    x * x
}

#[inline(never)]
pub fn never_inline_example() -> i32 {
    // This function is never inlined — useful for profiling to ensure
    // it appears as a distinct frame in flamegraphs.
    42
}

/// Demonstrates using `std::hint::black_box` to prevent compiler optimizations
/// from eliminating code in benchmarks.
pub fn benchmark_target(data: &[f64]) -> f64 {
    let sum: f64 = data.iter().sum();
    std::hint::black_box(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_if() {
        let data = vec![1, 5, 3, 8, 2, 9, 4, 7, 6, 0];
        let result = sum_if_sorted(&data, 5);
        assert_eq!(result, 8 + 9 + 7 + 6); // Values > 5
    }

    #[test]
    fn test_aos_vs_soa_sum() {
        let particles_aos: Vec<ParticleAoS> = (0..1000)
            .map(|i| ParticleAoS {
                x: i as f64,
                y: 0.0,
                z: 0.0,
                mass: 1.0,
                charge: 0.0,
                spin: 0.0,
            })
            .collect();

        let mut soa = ParticleSoA::new(1000);
        for p in &particles_aos {
            soa.push(p);
        }

        let sum_aos = sum_x_aos(&particles_aos);
        let sum_soa = sum_x_soa(&soa);

        assert!((sum_aos - sum_soa).abs() < 0.001);
    }

    #[test]
    fn test_sum_methods_agree() {
        let data: Vec<f64> = (0..1000).map(|i| i as f64).collect();

        let sum1 = manual_loop_sum(&data);
        let sum2 = iterator_sum(&data);
        let sum3 = chunked_sum(&data, 64);
        let sum4 = unrolled_sum(&data);
        let sum5 = multi_accumulator_sum(&data);

        assert!((sum1 - sum2).abs() < 0.001);
        assert!((sum1 - sum3).abs() < 0.001);
        assert!((sum1 - sum4).abs() < 0.001);
        assert!((sum1 - sum5).abs() < 0.001);
    }

    #[test]
    fn test_unrolled_sum_partial() {
        // Test with non-multiple-of-4 length
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let expected: f64 = data.iter().sum();
        let result = unrolled_sum(&data);
        assert!((result - expected).abs() < 0.001);
    }

    #[test]
    fn test_multi_accumulator_sum() {
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let expected: f64 = data.iter().sum();
        let result = multi_accumulator_sum(&data);
        assert!((result - expected).abs() < 0.001);
    }

    #[test]
    fn test_matmul_naive() {
        let n = 4;
        let a: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect();
        let b: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| (i * n + j) as f64).collect()).collect();
        let mut c = vec![vec![0.0; n]; n];

        matmul_naive(&a, &b, &mut c);

        // Identity * B = B
        for i in 0..n {
            for j in 0..n {
                assert!((c[i][j] - b[i][j]).abs() < 0.001);
            }
        }
    }

    #[test]
    fn test_matmul_blocked() {
        let n = 8;
        let a: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect();
        let b: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| (i * n + j) as f64).collect()).collect();
        let mut c = vec![vec![0.0; n]; n];

        matmul_blocked(&a, &b, &mut c, 4);

        for i in 0..n {
            for j in 0..n {
                assert!((c[i][j] - b[i][j]).abs() < 0.001);
            }
        }
    }

    #[test]
    fn test_fast_square() {
        assert!((fast_square(3.0) - 9.0).abs() < 0.001);
        assert!((fast_square(-2.0) - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_never_inline_example() {
        assert_eq!(never_inline_example(), 42);
    }

    #[test]
    fn test_benchmark_target() {
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let result = benchmark_target(&data);
        // black_box prevents optimization, but result should still be correct
        let expected: f64 = data.iter().sum();
        assert!((result - expected).abs() < 0.001);
    }

    #[test]
    fn test_struct_size() {
        // Verify that our structs have reasonable sizes
        let aos_size = std::mem::size_of::<ParticleAoS>();
        assert!(aos_size > 0);
        // AoS should be 6 * 8 = 48 bytes
        assert_eq!(aos_size, 48);
    }
}
