//! # Work Stealing with Rayon
//!
//! Work stealing is a scheduling strategy where idle threads "steal" tasks from
//! busy threads' queues. Rayon implements this pattern efficiently for data-parallel
//! and task-parallel workloads in Rust.
//!
//! ## Key Concepts:
//!
//! - **Thread Pool**: Fixed set of worker threads
//! - **Task Queue**: Per-thread deque of tasks
//! - **Work Stealing**: Idle threads take tasks from other threads' queues
//! - **Parallel Iterators**: Automatic parallelization of iterator chains
//! - **Join**: Execute two closures in parallel

use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Work stealing counter that demonstrates atomic operations with rayon.
pub struct ParallelCounter {
    value: AtomicU64,
}

impl ParallelCounter {
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    /// Increment counter from multiple threads using work stealing.
    pub fn parallel_increment(&self, times: usize) {
        (0..times).into_par_iter().for_each(|_| {
            self.value.fetch_add(1, Ordering::Relaxed);
        });
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Parallel search that splits work across threads.
pub fn parallel_find<T: Send + Sync + PartialEq>(items: &[T], target: &T) -> Option<usize> {
    items
        .par_iter()
        .enumerate()
        .find_any(|(_, item)| *item == target)
        .map(|(idx, _)| idx)
}

/// Parallel map that transforms each element using available threads.
pub fn parallel_transform<F, T, R>(items: Vec<T>, f: F) -> Vec<R>
where
    F: Fn(T) -> R + Send + Sync,
    T: Send,
    R: Send,
{
    items.into_par_iter().map(f).collect()
}

/// Parallel filter that keeps elements matching a predicate.
pub fn parallel_filter<F, T>(items: Vec<T>, predicate: F) -> Vec<T>
where
    F: Fn(&T) -> bool + Send + Sync,
    T: Send,
{
    items.into_par_iter().filter(|item| predicate(item)).collect()
}

/// Parallel sort using rayon.
pub fn parallel_sort<T: Ord + Send>(items: &mut [T]) {
    items.par_sort();
}

/// Parallel sort with custom comparator.
pub fn parallel_sort_by<T, F>(items: &mut [T], compare: F)
where
    T: Send,
    F: Fn(&T, &T) -> std::cmp::Ordering + Send + Sync,
{
    items.par_sort_by(compare);
}

/// Parallel chunk processing - split work into chunks and process each chunk.
pub fn parallel_chunk_process<T, R, F>(items: &[T], chunk_size: usize, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&[T]) -> R + Send + Sync,
{
    items.par_chunks(chunk_size).map(f).collect()
}

/// Parallel reduce - combine all elements into a single value.
pub fn parallel_sum(items: &[u64]) -> u64 {
    items.par_iter().sum()
}

/// Parallel min/max finding.
pub fn parallel_min_max(items: &[u64]) -> Option<(u64, u64)> {
    if items.is_empty() {
        return None;
    }
    let min = items.par_iter().copied().reduce(|| u64::MAX, |a, b| a.min(b));
    let max = items.par_iter().copied().reduce(|| u64::MIN, |a, b| a.max(b));
    Some((min, max))
}

/// Parallel fold with initial value.
pub fn parallel_fold<T, R, F, G>(items: &[T], init: R, fold_fn: F, combine_fn: G) -> R
where
    T: Sync,
    R: Send + Sync + Clone,
    F: Fn(R, &T) -> R + Send + Sync,
    G: Fn(R, R) -> R + Send + Sync,
{
    let chunk_size = (items.len() / rayon::current_num_threads().max(1)).max(1);
    let results: Vec<R> = items
        .par_chunks(chunk_size)
        .map(|chunk| chunk.iter().fold(init.clone(), &fold_fn))
        .collect();
    results.into_iter().fold(init, combine_fn)
}

/// Demonstrates rayon's join primitive for task parallelism.
pub fn parallel_fibonacci(n: u32) -> u64 {
    if n <= 20 {
        // Small enough to compute sequentially
        return sequential_fibonacci(n);
    }

    let (a, b) = rayon::join(
        || parallel_fibonacci(n - 1),
        || parallel_fibonacci(n - 2),
    );
    a + b
}

fn sequential_fibonacci(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}

/// Parallel pipeline processing with multiple stages.
pub struct ParallelPipeline<I: Send> {
    items: Vec<I>,
}

impl<I: Send> ParallelPipeline<I> {
    pub fn new(items: Vec<I>) -> Self {
        Self { items }
    }

    /// Apply a transformation stage.
    pub fn map<O: Send, F: Fn(I) -> O + Send + Sync>(self, f: F) -> ParallelPipeline<O> {
        ParallelPipeline {
            items: self.items.into_par_iter().map(f).collect(),
        }
    }

    /// Filter items.
    pub fn filter<F: Fn(&I) -> bool + Send + Sync>(self, predicate: F) -> Self {
        ParallelPipeline {
            items: self
                .items
                .into_par_iter()
                .filter(|item| predicate(item))
                .collect(),
        }
    }

    /// Collect the results.
    pub fn collect(self) -> Vec<I> {
        self.items
    }
}

/// Custom thread pool configuration.
pub fn configure_thread_pool(num_threads: usize) -> Result<(), String> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .map_err(|e| format!("Failed to configure thread pool: {}", e))
}

/// Get the current number of worker threads.
pub fn current_num_threads() -> usize {
    rayon::current_num_threads()
}

/// Parallel string processing example.
pub fn parallel_word_count(texts: &[String]) -> usize {
    texts.par_iter().map(|text| text.split_whitespace().count()).sum()
}

/// Parallel frequency counting.
pub fn parallel_char_frequency(texts: &[String]) -> std::collections::HashMap<char, usize> {
    let maps: Vec<std::collections::HashMap<char, usize>> = texts
        .par_iter()
        .map(|text| {
            let mut map = std::collections::HashMap::new();
            for c in text.chars() {
                *map.entry(c).or_insert(0) += 1;
            }
            map
        })
        .collect();

    // Merge all maps
    let mut result = std::collections::HashMap::new();
    for map in maps {
        for (ch, count) in map {
            *result.entry(ch).or_insert(0) += count;
        }
    }
    result
}

/// Parallel matrix multiplication (demonstrates chunk-based work stealing).
pub fn parallel_matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rows = a.len();
    let cols = b[0].len();
    let inner = b.len();

    let mut result = vec![vec![0.0; cols]; rows];

    result.par_iter_mut().enumerate().for_each(|(i, row)| {
        for j in 0..cols {
            let mut sum = 0.0;
            for k in 0..inner {
                sum += a[i][k] * b[k][j];
            }
            row[j] = sum;
        }
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_counter() {
        let counter = ParallelCounter::new();
        counter.parallel_increment(10_000);
        assert_eq!(counter.get(), 10_000);
    }

    #[test]
    fn test_parallel_find() {
        let items: Vec<i32> = (0..1000).collect();
        assert_eq!(parallel_find(&items, &500), Some(500));
        assert_eq!(parallel_find(&items, &999), Some(999));
        assert_eq!(parallel_find(&items, &1000), None);
    }

    #[test]
    fn test_parallel_transform() {
        let items = vec![1, 2, 3, 4, 5];
        let result = parallel_transform(items, |x| x * 2);
        let mut sorted = result;
        sorted.sort();
        assert_eq!(sorted, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_parallel_filter() {
        let items: Vec<i32> = (0..100).collect();
        let evens = parallel_filter(items, |x| x % 2 == 0);
        assert_eq!(evens.len(), 50);
        assert!(evens.iter().all(|x| x % 2 == 0));
    }

    #[test]
    fn test_parallel_sort() {
        let mut items = vec![5, 3, 1, 4, 2];
        parallel_sort(&mut items);
        assert_eq!(items, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parallel_sort_by() {
        let mut items = vec![5, 3, 1, 4, 2];
        parallel_sort_by(&mut items, |a, b| b.cmp(a)); // Descending
        assert_eq!(items, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_parallel_chunk_process() {
        let items: Vec<i32> = (0..100).collect();
        let sums = parallel_chunk_process(&items, 10, |chunk| chunk.iter().sum::<i32>());
        let total: i32 = sums.iter().sum();
        assert_eq!(total, (0..100).sum::<i32>());
    }

    #[test]
    fn test_parallel_sum() {
        let items: Vec<u64> = (1..=1000).collect();
        assert_eq!(parallel_sum(&items), 500_500);
    }

    #[test]
    fn test_parallel_min_max() {
        let items = vec![5, 3, 1, 4, 2];
        let result = parallel_min_max(&items);
        assert_eq!(result, Some((1, 5)));
    }

    #[test]
    fn test_parallel_min_max_empty() {
        let items: Vec<u64> = vec![];
        assert!(parallel_min_max(&items).is_none());
    }

    #[test]
    fn test_parallel_fold() {
        let items: Vec<i32> = (1..=100).collect();
        let sum = parallel_fold(&items, 0, |acc, x| acc + x, |a, b| a + b);
        assert_eq!(sum, 5050);
    }

    #[test]
    fn test_parallel_fibonacci() {
        assert_eq!(parallel_fibonacci(0), 0);
        assert_eq!(parallel_fibonacci(1), 1);
        assert_eq!(parallel_fibonacci(10), 55);
        assert_eq!(parallel_fibonacci(30), 832040);
    }

    #[test]
    fn test_parallel_pipeline() {
        let result = ParallelPipeline::new(vec![1, 2, 3, 4, 5])
            .map(|x| x * 2)
            .filter(|&x| x > 4)
            .collect();

        let mut sorted = result;
        sorted.sort();
        assert_eq!(sorted, vec![6, 8, 10]);
    }

    #[test]
    fn test_parallel_word_count() {
        let texts = vec![
            "hello world".to_string(),
            "foo bar baz".to_string(),
            "one".to_string(),
        ];
        assert_eq!(parallel_word_count(&texts), 6);
    }

    #[test]
    fn test_parallel_char_frequency() {
        let texts = vec!["hello".to_string(), "world".to_string()];
        let freq = parallel_char_frequency(&texts);
        assert_eq!(freq.get(&'l'), Some(&3)); // l appears 3 times
        assert_eq!(freq.get(&'o'), Some(&2)); // o appears 2 times
    }

    #[test]
    fn test_parallel_matrix_multiply() {
        let a = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let b = vec![vec![5.0, 6.0], vec![7.0, 8.0]];
        let result = parallel_matrix_multiply(&a, &b);

        // [1*5+2*7, 1*6+2*8] = [19, 22]
        // [3*5+4*7, 3*6+4*8] = [43, 50]
        assert!((result[0][0] - 19.0).abs() < f64::EPSILON);
        assert!((result[0][1] - 22.0).abs() < f64::EPSILON);
        assert!((result[1][0] - 43.0).abs() < f64::EPSILON);
        assert!((result[1][1] - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_current_num_threads() {
        let threads = current_num_threads();
        assert!(threads > 0);
    }

    #[test]
    fn test_sequential_fibonacci() {
        assert_eq!(sequential_fibonacci(0), 0);
        assert_eq!(sequential_fibonacci(1), 1);
        assert_eq!(sequential_fibonacci(2), 1);
        assert_eq!(sequential_fibonacci(10), 55);
    }

    #[test]
    fn test_parallel_sort_large() {
        let mut items: Vec<i32> = (0..10000).rev().collect();
        parallel_sort(&mut items);
        assert_eq!(items[0], 0);
        assert_eq!(items[9999], 9999);
    }

    #[test]
    fn test_parallel_transform_preserves_all() {
        let items: Vec<i32> = (0..1000).collect();
        let result = parallel_transform(items, |x| x * x);
        assert_eq!(result.len(), 1000);
    }
}
