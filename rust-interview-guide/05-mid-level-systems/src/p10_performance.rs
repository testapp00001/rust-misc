/// Problem: Performance
///
/// Master performance optimization in Rust.
///
/// Key Concepts:
/// - Profiling
/// - Memory optimization
/// - Algorithm optimization
/// - Caching
/// - Benchmarking

use std::collections::HashMap;

/// Problem 1: Basic profiling
/// Measure execution time
pub fn measure_time<F, T>(f: F) -> (T, std::time::Duration)
where
    F: FnOnce() -> T,
{
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Problem 2: Memory optimization
/// Use efficient data structures
pub fn efficient_vec() -> Vec<i32> {
    Vec::with_capacity(1000) // Pre-allocate
}

/// Problem 3: Algorithm optimization
/// Use efficient algorithm
pub fn sum_efficient(data: &[i32]) -> i32 {
    data.iter().sum() // Use iterator
}

/// Problem 4: Caching
/// Implement simple cache
pub struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
}

/// Problem 5: Benchmarking
/// Benchmark function
pub fn benchmark<F>(name: &str, iterations: u32, f: F)
where
    F: Fn(),
{
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f();
    }
    let duration = start.elapsed();
    println!("{}: {:?} for {} iterations", name, duration, iterations);
}

/// Problem 6: Lazy evaluation
/// Use lazy evaluation
pub fn lazy_sum(data: &[i32]) -> i32 {
    data.iter().filter(|&&x| x > 0).sum()
}

/// Problem 7: Avoid cloning
/// Minimize cloning
pub fn avoid_clone(data: &[String]) -> Vec<&str> {
    data.iter().map(|s| s.as_str()).collect()
}

/// Problem 8: Use references
/// Prefer references over owned values
pub fn use_references(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Problem 9: Parallel processing (simulated)
/// Simulate parallel processing
pub fn parallel_sum(data: &[i32]) -> i32 {
    // In real implementation, you'd use rayon or similar
    data.iter().sum()
}

/// Problem 10: Memory pooling (simulated)
/// Simulate memory pooling
pub struct Pool {
    objects: Vec<Vec<u8>>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn get(&mut self) -> Vec<u8> {
        self.objects.pop().unwrap_or_else(|| Vec::with_capacity(1024))
    }

    pub fn put(&mut self, obj: Vec<u8>) {
        self.objects.push(obj);
    }
}

/// Problem 11: Zero-copy parsing
/// Parse without copying
pub fn parse_without_copy(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

/// Problem 12: Stack allocation
/// Prefer stack allocation
pub fn stack_allocation() -> [i32; 100] {
    [0; 100]
}

/// Problem 13: Avoid allocations
/// Minimize allocations
pub fn avoid_allocations(data: &mut Vec<i32>) {
    data.clear(); // Reuse allocation
}

/// Problem 14: Use iterators efficiently
/// Efficient iterator usage
pub fn efficient_iterator(data: &[i32]) -> Vec<i32> {
    data.iter().filter(|&&x| x > 0).copied().collect()
}

/// Problem 15: Profile-guided optimization (simulated)
/// Simulate PGO
pub fn pgo_optimized(data: &[i32]) -> i32 {
    // In real implementation, you'd use profile data
    data.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_time() {
        let (result, duration) = measure_time(|| 42);
        assert_eq!(result, 42);
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_efficient_vec() {
        let v = efficient_vec();
        assert!(v.capacity() >= 1000);
    }

    #[test]
    fn test_sum_efficient() {
        assert_eq!(sum_efficient(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_cache() {
        let mut cache = Cache::new();
        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_benchmark() {
        benchmark("test", 1000, || {
            let _ = 1 + 1;
        });
    }

    #[test]
    fn test_lazy_sum() {
        assert_eq!(lazy_sum(&[1, -2, 3, -4, 5]), 9);
    }

    #[test]
    fn test_avoid_clone() {
        let data = vec!["hello".to_string(), "world".to_string()];
        let refs = avoid_clone(&data);
        assert_eq!(refs, vec!["hello", "world"]);
    }

    #[test]
    fn test_use_references() {
        assert_eq!(use_references(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_parallel_sum() {
        assert_eq!(parallel_sum(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_pool() {
        let mut pool = Pool::new();
        let obj = pool.get();
        assert!(obj.capacity() >= 1024);
        pool.put(obj);
        let obj2 = pool.get();
        assert!(obj2.capacity() >= 1024);
    }

    #[test]
    fn test_parse_without_copy() {
        let result = parse_without_copy("hello world");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_stack_allocation() {
        let arr = stack_allocation();
        assert_eq!(arr.len(), 100);
    }

    #[test]
    fn test_avoid_allocations() {
        let mut v = vec![1, 2, 3];
        avoid_allocations(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_efficient_iterator() {
        let result = efficient_iterator(&[1, -2, 3, -4, 5]);
        assert_eq!(result, vec![1, 3, 5]);
    }

    #[test]
    fn test_pgo_optimized() {
        assert_eq!(pgo_optimized(&[1, 2, 3, 4, 5]), 15);
    }
}
