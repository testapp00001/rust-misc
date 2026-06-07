/// Problem: Parallel Computation
///
/// Master parallel computation patterns.
///
/// Key Concepts:
/// - Parallel iteration
/// - Parallel reduction
/// - Parallel map
/// - Work distribution
/// - Result collection

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

/// Problem 1: Parallel sum
/// Sum numbers in parallel
pub fn parallel_sum(data: &[i32]) -> i32 {
    let chunk_size = (data.len() + 3) / 4; // 4 threads
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.iter().sum::<i32>()
        });
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// Problem 2: Parallel map
/// Apply function to each element in parallel
pub fn parallel_map(data: &[i32], f: fn(i32) -> i32) -> Vec<i32> {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.into_iter().map(f).collect::<Vec<i32>>()
        });
        handles.push(handle);
    }

    handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
}

/// Problem 3: Parallel filter
/// Filter elements in parallel
pub fn parallel_filter(data: &[i32], predicate: fn(&i32) -> bool) -> Vec<i32> {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.into_iter().filter(|x| predicate(x)).collect::<Vec<i32>>()
        });
        handles.push(handle);
    }

    handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
}

/// Problem 4: Parallel max
/// Find maximum in parallel
pub fn parallel_max(data: &[i32]) -> Option<i32> {
    if data.is_empty() {
        return None;
    }

    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.into_iter().max()
        });
        handles.push(handle);
    }

    handles.into_iter()
        .filter_map(|h| h.join().unwrap())
        .max()
}

/// Problem 5: Parallel min
/// Find minimum in parallel
pub fn parallel_min(data: &[i32]) -> Option<i32> {
    if data.is_empty() {
        return None;
    }

    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.into_iter().min()
        });
        handles.push(handle);
    }

    handles.into_iter()
        .filter_map(|h| h.join().unwrap())
        .min()
}

/// Problem 6: Parallel count
/// Count elements matching predicate in parallel
pub fn parallel_count(data: &[i32], predicate: fn(&i32) -> bool) -> usize {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.iter().filter(|x| predicate(x)).count()
        });
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// Problem 7: Parallel sort
/// Sort chunks in parallel then merge
pub fn parallel_sort(data: &[i32]) -> Vec<i32> {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let mut chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.sort();
            chunk
        });
        handles.push(handle);
    }

    let sorted_chunks: Vec<Vec<i32>> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    // Merge sorted chunks
    let mut result = Vec::new();
    for chunk in sorted_chunks {
        result.extend(chunk);
    }
    result.sort();
    result
}

/// Problem 8: Parallel matrix multiply
/// Multiply matrices in parallel
pub fn parallel_matrix_multiply(a: &[Vec<i32>], b: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let n = a.len();
    let mut result = vec![vec![0; n]; n];
    let result = Arc::new(Mutex::new(result));
    let mut handles = vec![];

    for i in 0..n {
        for j in 0..n {
            let a = a.to_vec();
            let b = b.to_vec();
            let result = Arc::clone(&result);
            let handle = thread::spawn(move || {
                let mut sum = 0;
                for k in 0..n {
                    sum += a[i][k] * b[k][j];
                }
                let mut result = result.lock().unwrap();
                result[i][j] = sum;
            });
            handles.push(handle);
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(result).unwrap().into_inner().unwrap()
}

/// Problem 9: Parallel word count
/// Count words in parallel
pub fn parallel_word_count(text: &str) -> usize {
    let words: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
    let chunk_size = (words.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in words.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.len()
        });
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// Problem 10: Parallel reduction
/// Reduce elements in parallel
pub fn parallel_reduce(data: &[i32], init: i32, f: fn(i32, i32) -> i32) -> i32 {
    let chunk_size = (data.len() + 3) / 4;
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.into_iter().fold(init, f)
        });
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).fold(init, f)
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
    fn test_parallel_max() {
        let data = vec![3, 1, 4, 1, 5, 9, 2, 6];
        assert_eq!(parallel_max(&data), Some(9));
    }

    #[test]
    fn test_parallel_min() {
        let data = vec![3, 1, 4, 1, 5, 9, 2, 6];
        assert_eq!(parallel_min(&data), Some(1));
    }

    #[test]
    fn test_parallel_count() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(parallel_count(&data, |x| x % 2 == 0), 5);
    }

    #[test]
    fn test_parallel_sort() {
        let data = vec![3, 1, 4, 1, 5, 9, 2, 6];
        assert_eq!(parallel_sort(&data), vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_parallel_matrix_multiply() {
        let a = vec![vec![1, 2], vec![3, 4]];
        let b = vec![vec![5, 6], vec![7, 8]];
        let result = parallel_matrix_multiply(&a, &b);
        assert_eq!(result, vec![vec![19, 22], vec![43, 50]]);
    }

    #[test]
    fn test_parallel_word_count() {
        let text = "hello world foo bar baz";
        assert_eq!(parallel_word_count(text), 5);
    }

    #[test]
    fn test_parallel_reduce() {
        let data = vec![1, 2, 3, 4, 5];
        let result = parallel_reduce(&data, 0, |acc, x| acc + x);
        assert_eq!(result, 15);
    }
}
