//! # Parallel Iterators with Rayon
//!
//! Rayon's parallel iterators provide a drop-in replacement for standard iterators
//! that automatically parallelize computation across available CPU cores. This
//! module covers the parallel iterator API, custom parallel operations, and
//! performance considerations.
//!
//! ## Key Traits:
//!
//! - `ParallelIterator`: Base trait for parallel iteration
//! - `IndexedParallelIterator`: For random-access collections
//! - `IntoParallelIterator`: Convert collections into parallel iterators
//!
//! ## Common Operations:
//!
//! | Operation | Sequential | Parallel |
//! |-----------|-----------|----------|
//! | map | `.map(f)` | `.par_iter().map(f)` |
//! | filter | `.filter(f)` | `.par_iter().filter(f)` |
//! | reduce | `.fold(init, f)` | `.par_iter().fold(init, f).reduce(f)` |
//! | for_each | `.for_each(f)` | `.par_iter().for_each(f)` |
//! | collect | `.collect()` | `.par_iter().collect()` |

use rayon::prelude::*;
use std::collections::HashMap;

/// Parallel processing statistics.
#[derive(Debug)]
pub struct ParallelStats {
    pub items_processed: usize,
    pub threads_used: usize,
    pub results: Vec<f64>,
}

/// Parallel map with statistics collection.
pub fn parallel_map_with_stats<F>(items: &[f64], f: F) -> ParallelStats
where
    F: Fn(f64) -> f64 + Send + Sync,
{
    let results: Vec<f64> = items.par_iter().map(|&x| f(x)).collect();
    ParallelStats {
        items_processed: items.len(),
        threads_used: rayon::current_num_threads(),
        results,
    }
}

/// Parallel group-by operation.
pub fn parallel_group_by<T, K, F>(items: &[T], key_fn: F) -> HashMap<K, Vec<&T>>
where
    T: Sync,
    K: Eq + std::hash::Hash + Send,
    F: Fn(&T) -> K + Send + Sync,
{
    let pairs: Vec<(K, &T)> = items.par_iter().map(|item| (key_fn(item), item)).collect();

    let mut groups: HashMap<K, Vec<&T>> = HashMap::new();
    for (key, value) in pairs {
        groups.entry(key).or_default().push(value);
    }
    groups
}

/// Parallel unique/distinct operation.
pub fn parallel_unique<T: Eq + std::hash::Hash + Clone + Send + Sync>(items: &[T]) -> Vec<T> {
    let set: std::collections::HashSet<&T> = items.par_iter().collect();
    set.into_iter().cloned().collect()
}

/// Parallel flatmap operation.
pub fn parallel_flatmap<T, R, F, I>(items: &[T], f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> I + Send + Sync,
    I: IntoIterator<Item = R>,
{
    // Collect results from each item in parallel, then flatten
    let nested: Vec<Vec<R>> = items.par_iter().map(|item| f(item).into_iter().collect()).collect();
    nested.into_iter().flatten().collect()
}

/// Parallel zip operation combining two slices.
pub fn parallel_zip_with<A, B, R, F>(a: &[A], b: &[B], f: F) -> Vec<R>
where
    A: Sync,
    B: Sync,
    R: Send,
    F: Fn(&A, &B) -> R + Send + Sync,
{
    a.par_iter()
        .zip(b.par_iter())
        .map(|(x, y)| f(x, y))
        .collect()
}

/// Parallel window processing.
pub fn parallel_windows<T, R, F>(items: &[T], window_size: usize, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&[T]) -> R + Send + Sync,
{
    items
        .par_windows(window_size)
        .map(f)
        .collect()
}

/// Parallel chunk processing with overlap.
pub fn parallel_chunks_overlapping<T, R, F>(
    items: &[T],
    chunk_size: usize,
    overlap: usize,
    f: F,
) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&[T]) -> R + Send + Sync,
{
    let step = chunk_size.saturating_sub(overlap);
    if step == 0 {
        return vec![];
    }

    let indices: Vec<usize> = (0..items.len()).step_by(step).collect();
    indices
        .par_iter()
        .map(|&start| {
            let end = (start + chunk_size).min(items.len());
            f(&items[start..end])
        })
        .collect()
}

/// Parallel search with early termination using find_any.
pub fn parallel_find_first<P, T>(items: &[T], predicate: P) -> Option<usize>
where
    T: Sync,
    P: Fn(&T) -> bool + Send + Sync,
{
    items
        .par_iter()
        .position_any(|item| predicate(item))
}

/// Parallel any/all checks.
pub fn parallel_any<P, T>(items: &[T], predicate: P) -> bool
where
    T: Sync,
    P: Fn(&T) -> bool + Send + Sync,
{
    items.par_iter().any(|item| predicate(item))
}

pub fn parallel_all<P, T>(items: &[T], predicate: P) -> bool
where
    T: Sync,
    P: Fn(&T) -> bool + Send + Sync,
{
    items.par_iter().all(|item| predicate(item))
}

/// Parallel counting with predicate.
pub fn parallel_count<P, T>(items: &[T], predicate: P) -> usize
where
    T: Sync,
    P: Fn(&T) -> bool + Send + Sync,
{
    items.par_iter().filter(|item| predicate(item)).count()
}

/// Parallel histogram computation.
pub fn parallel_histogram(items: &[f64], num_bins: usize, min: f64, max: f64) -> Vec<usize> {
    let bin_width = (max - min) / num_bins as f64;
    let local_histograms: Vec<Vec<usize>> = items
        .par_chunks(items.len() / rayon::current_num_threads().max(1))
        .map(|chunk| {
            let mut hist = vec![0usize; num_bins];
            for &val in chunk {
                if val >= min && val < max {
                    let bin = ((val - min) / bin_width) as usize;
                    let bin = bin.min(num_bins - 1);
                    hist[bin] += 1;
                }
            }
            hist
        })
        .collect();

    // Merge histograms
    let mut result = vec![0usize; num_bins];
    for hist in local_histograms {
        for (i, count) in hist.into_iter().enumerate() {
            result[i] += count;
        }
    }
    result
}

/// Parallel top-N elements.
pub fn parallel_top_n<T: Ord + Send + Clone>(items: &[T], n: usize) -> Vec<T> {
    let mut sorted: Vec<T> = items.to_vec();
    sorted.par_sort();
    sorted.into_iter().rev().take(n).collect()
}

/// Parallel string processing example.
pub fn parallel_string_transform(texts: &[String], transform: fn(&str) -> String) -> Vec<String> {
    texts.par_iter().map(|s| transform(s)).collect()
}

/// Parallel aggregate computation with multiple metrics.
#[derive(Debug, Clone)]
pub struct AggregateResult {
    pub sum: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

pub fn parallel_aggregate(items: &[f64]) -> AggregateResult {
    if items.is_empty() {
        return AggregateResult {
            sum: 0.0,
            count: 0,
            min: 0.0,
            max: 0.0,
            mean: 0.0,
        };
    }

    let chunk_size = items.len() / rayon::current_num_threads().max(1);
    let chunk_results: Vec<(f64, usize, f64, f64)> = items
        .par_chunks(chunk_size.max(1))
        .map(|chunk| {
            let sum: f64 = chunk.iter().sum();
            let count = chunk.len();
            let min = chunk.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = chunk.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (sum, count, min, max)
        })
        .collect();

    let total_sum: f64 = chunk_results.iter().map(|r| r.0).sum();
    let total_count: usize = chunk_results.iter().map(|r| r.1).sum();
    let min = chunk_results
        .iter()
        .map(|r| r.2)
        .fold(f64::INFINITY, f64::min);
    let max = chunk_results
        .iter()
        .map(|r| r.3)
        .fold(f64::NEG_INFINITY, f64::max);

    AggregateResult {
        sum: total_sum,
        count: total_count,
        min,
        max,
        mean: total_sum / total_count as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_map_with_stats() {
        let items = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = parallel_map_with_stats(&items, |x| x * 2.0);
        assert_eq!(stats.items_processed, 5);
        assert!(stats.threads_used > 0);
        let mut sorted = stats.results;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(sorted, vec![2.0, 4.0, 6.0, 8.0, 10.0]);
    }

    #[test]
    fn test_parallel_group_by() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let groups = parallel_group_by(&items, |x| if x % 2 == 0 { "even" } else { "odd" });
        assert_eq!(groups.get("even").unwrap().len(), 3);
        assert_eq!(groups.get("odd").unwrap().len(), 3);
    }

    #[test]
    fn test_parallel_unique() {
        let items = vec![1, 2, 3, 2, 1, 4, 5, 4];
        let unique = parallel_unique(&items);
        let mut sorted = unique;
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parallel_flatmap() {
        let items = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        let result = parallel_flatmap(&items, |v| v.clone());
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn test_parallel_zip_with() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let result = parallel_zip_with(&a, &b, |x, y| x + y);
        assert_eq!(result, vec![5, 7, 9]);
    }

    #[test]
    fn test_parallel_windows() {
        let items = vec![1, 2, 3, 4, 5];
        let sums = parallel_windows(&items, 3, |w| w.iter().sum::<i32>());
        assert_eq!(sums, vec![6, 9, 12]);
    }

    #[test]
    fn test_parallel_chunks_overlapping() {
        let items: Vec<i32> = (0..10).collect();
        let results = parallel_chunks_overlapping(&items, 4, 1, |chunk| chunk.to_vec());
        assert!(!results.is_empty());
    }

    #[test]
    fn test_parallel_find_first() {
        let items: Vec<i32> = (0..1000).collect();
        let result = parallel_find_first(&items, |&x| x == 500);
        assert!(result.is_some());
    }

    #[test]
    fn test_parallel_any() {
        let items = vec![1, 2, 3, 4, 5];
        assert!(parallel_any(&items, |&x| x > 3));
        assert!(!parallel_any(&items, |&x| x > 10));
    }

    #[test]
    fn test_parallel_all() {
        let items = vec![2, 4, 6, 8, 10];
        assert!(parallel_all(&items, |x: &i32| x % 2 == 0));
        assert!(!parallel_all(&items, |x: &i32| *x > 5));
    }

    #[test]
    fn test_parallel_count() {
        let items: Vec<i32> = (0..100).collect();
        let count = parallel_count(&items, |x| x % 2 == 0);
        assert_eq!(count, 50);
    }

    #[test]
    fn test_parallel_histogram() {
        let items: Vec<f64> = (0..100).map(|x| x as f64).collect();
        let hist = parallel_histogram(&items, 10, 0.0, 100.0);
        assert_eq!(hist.len(), 10);
        let total: usize = hist.iter().sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn test_parallel_top_n() {
        let items = vec![5, 3, 1, 4, 2, 8, 7, 6];
        let top3 = parallel_top_n(&items, 3);
        assert_eq!(top3, vec![8, 7, 6]);
    }

    #[test]
    fn test_parallel_string_transform() {
        let texts = vec!["hello".to_string(), "world".to_string()];
        let result = parallel_string_transform(&texts, |s| s.to_uppercase());
        assert!(result.contains(&"HELLO".to_string()));
        assert!(result.contains(&"WORLD".to_string()));
    }

    #[test]
    fn test_parallel_aggregate() {
        let items = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let agg = parallel_aggregate(&items);
        assert!((agg.sum - 15.0).abs() < f64::EPSILON);
        assert_eq!(agg.count, 5);
        assert!((agg.min - 1.0).abs() < f64::EPSILON);
        assert!((agg.max - 5.0).abs() < f64::EPSILON);
        assert!((agg.mean - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parallel_aggregate_empty() {
        let items: Vec<f64> = vec![];
        let agg = parallel_aggregate(&items);
        assert_eq!(agg.count, 0);
    }

    #[test]
    fn test_parallel_group_by_empty() {
        let items: Vec<i32> = vec![];
        let groups = parallel_group_by(&items, |x| x % 2);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_parallel_unique_empty() {
        let items: Vec<i32> = vec![];
        let unique = parallel_unique(&items);
        assert!(unique.is_empty());
    }

    #[test]
    fn test_parallel_find_first_not_found() {
        let items: Vec<i32> = (0..100).collect();
        let result = parallel_find_first(&items, |&x| x > 200);
        assert!(result.is_none());
    }

    #[test]
    fn test_parallel_count_empty() {
        let items: Vec<i32> = vec![];
        assert_eq!(parallel_count(&items, |_: &i32| true), 0);
    }
}
