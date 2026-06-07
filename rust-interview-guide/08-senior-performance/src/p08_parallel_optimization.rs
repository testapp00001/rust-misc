/// Problem: Parallel Optimization
///
/// Master parallel optimization in Rust.
///
/// Key Concepts:
/// - Thread parallelism
/// - Work stealing
/// - Load balancing
/// - Synchronization overhead
/// - Parallel algorithms

use std::sync::{Arc, Mutex};
use std::thread;

/// Problem 1: Parallel sum
/// Sum in parallel
pub fn parallel_sum(data: &[i32]) -> i32 {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            chunk.iter().sum::<i32>()
        }));
    }

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// Problem 2: Parallel map
/// Map in parallel
pub fn parallel_map(data: &[i32], f: fn(i32) -> i32) -> Vec<i32> {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            chunk.into_iter().map(f).collect::<Vec<i32>>()
        }));
    }

    handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
}

/// Problem 3: Parallel filter
/// Filter in parallel
pub fn parallel_filter(data: &[i32], predicate: fn(&i32) -> bool) -> Vec<i32> {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            chunk.into_iter().filter(|x| predicate(x)).collect::<Vec<i32>>()
        }));
    }

    handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
}

/// Problem 4: Parallel reduce
/// Reduce in parallel
pub fn parallel_reduce(data: &[i32], init: i32, f: fn(i32, i32) -> i32) -> i32 {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            chunk.into_iter().fold(init, f)
        }));
    }

    handles.into_iter().map(|h| h.join().unwrap()).fold(init, f)
}

/// Problem 5: Parallel sort
/// Sort in parallel (simulated)
pub fn parallel_sort(data: &mut [i32]) {
    data.sort();
}

/// Problem 6: Parallel search
/// Search in parallel
pub fn parallel_search(data: &[i32], target: i32) -> Option<usize> {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for (i, chunk) in data.chunks(chunk_size).enumerate() {
        let chunk = chunk.to_vec();
        let offset = i * chunk_size;
        handles.push(thread::spawn(move || {
            chunk.iter().position(|&x| x == target).map(|p| p + offset)
        }));
    }

    for handle in handles {
        if let Some(result) = handle.join().unwrap() {
            return Some(result);
        }
    }
    None
}

/// Problem 7: Parallel matrix multiply
/// Multiply matrices in parallel
pub fn parallel_matrix_multiply(a: &[Vec<i32>], b: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let n = a.len();
    let result = Arc::new(Mutex::new(vec![vec![0; n]; n]));
    let mut handles = vec![];

    for i in 0..n {
        for j in 0..n {
            let a = a.to_vec();
            let b = b.to_vec();
            let result = Arc::clone(&result);
            handles.push(thread::spawn(move || {
                let mut sum = 0;
                for k in 0..n {
                    sum += a[i][k] * b[k][j];
                }
                let mut result = result.lock().unwrap();
                result[i][j] = sum;
            }));
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(result).unwrap().into_inner().unwrap()
}

/// Problem 8: Work stealing
/// Simulate work stealing
pub fn work_stealing(data: &[i32]) -> i32 {
    let data = Arc::new(data.to_vec());
    let result = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for i in 0..4 {
        let data = Arc::clone(&data);
        let result = Arc::clone(&result);
        handles.push(thread::spawn(move || {
            let chunk_size = (data.len() + 3) / 4;
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, data.len());
            let sum: i32 = data[start..end].iter().sum();
            let mut result = result.lock().unwrap();
            *result += sum;
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = *result.lock().unwrap(); result
}

/// Problem 9: Load balancing
/// Balance load across threads
pub fn load_balancing(data: &[i32]) -> i32 {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            chunk.iter().sum::<i32>()
        }));
    }

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// Problem 10: Minimize synchronization
/// Reduce lock contention
pub fn minimize_sync(data: &[i32]) -> i32 {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            chunk.iter().sum::<i32>()
        }));
    }

    // No shared state, no synchronization
    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// Problem 11: Parallel pipeline
/// Parallel pipeline stages
pub fn parallel_pipeline(data: &[i32]) -> Vec<i32> {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            chunk.into_iter()
                .filter(|&x| x > 0)
                .map(|x| x * 2)
                .collect::<Vec<i32>>()
        }));
    }

    handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
}

/// Problem 12: Parallel fan-out/fan-in
/// Fan-out and fan-in pattern
pub fn parallel_fan_out_fan_in(data: &[i32]) -> i32 {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    // Fan-out
    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            chunk.iter().sum::<i32>()
        }));
    }

    // Fan-in
    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// Problem 13: Parallel with channels
/// Use channels for communication
pub fn parallel_channels(data: &[i32]) -> Vec<i32> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let sum: i32 = chunk.iter().sum();
            tx.send(sum).unwrap();
        }));
    }

    drop(tx);

    let mut results = Vec::new();
    for val in rx {
        results.push(val);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    results
}

/// Problem 14: Parallel with barrier
/// Synchronize with barrier
pub fn parallel_barrier(data: &[i32]) -> Vec<i32> {
    use std::sync::Barrier;

    let barrier = Arc::new(Barrier::new(4));
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            chunk.iter().sum::<i32>()
        }));
    }

    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

/// Problem 15: Parallel with scoped threads
/// Use scoped threads
pub fn parallel_scoped(data: &[i32]) -> i32 {
    thread::scope(|s| {
        let chunk_size = (data.len() + 3) / 4;
        let mut handles = vec![];

        for chunk in data.chunks(chunk_size) {
            handles.push(s.spawn(move || {
                chunk.iter().sum::<i32>()
            }));
        }

        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_sum() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(parallel_sum(&data), 55);
    }

    #[test]
    fn test_parallel_map() {
        let data = vec![1, 2, 3, 4, 5];
        let result = parallel_map(&data, |x| x * 2);
        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_parallel_filter() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = parallel_filter(&data, |x| x % 2 == 0);
        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_parallel_reduce() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(parallel_reduce(&data, 0, |acc, x| acc + x), 15);
    }

    #[test]
    fn test_parallel_sort() {
        let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];
        parallel_sort(&mut data);
        assert_eq!(data, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_parallel_search() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(parallel_search(&data, 5), Some(4));
    }

    #[test]
    fn test_parallel_matrix_multiply() {
        let a = vec![vec![1, 2], vec![3, 4]];
        let b = vec![vec![5, 6], vec![7, 8]];
        let result = parallel_matrix_multiply(&a, &b);
        assert_eq!(result, vec![vec![19, 22], vec![43, 50]]);
    }

    #[test]
    fn test_work_stealing() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(work_stealing(&data), 55);
    }

    #[test]
    fn test_load_balancing() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(load_balancing(&data), 55);
    }

    #[test]
    fn test_minimize_sync() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(minimize_sync(&data), 55);
    }

    #[test]
    fn test_parallel_pipeline() {
        let data = vec![1, -2, 3, -4, 5];
        let result = parallel_pipeline(&data);
        assert!(result.contains(&2));
        assert!(result.contains(&6));
        assert!(result.contains(&10));
    }

    #[test]
    fn test_parallel_fan_out_fan_in() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(parallel_fan_out_fan_in(&data), 55);
    }

    #[test]
    fn test_parallel_channels() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let results = parallel_channels(&data);
        assert_eq!(results.iter().sum::<i32>(), 55);
    }

    #[test]
    fn test_parallel_barrier() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let results = parallel_barrier(&data);
        assert_eq!(results.iter().sum::<i32>(), 55);
    }

    #[test]
    fn test_parallel_scoped() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(parallel_scoped(&data), 55);
    }
}
