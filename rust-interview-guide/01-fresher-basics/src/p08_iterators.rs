/// Problem: Iterators
///
/// Master Rust's iterator system.
///
/// Key Concepts:
/// - Iterator trait
/// - Iterator adaptors (map, filter, etc.)
/// - Consumer adaptors (collect, sum, etc.)
/// - Creating custom iterators
/// - Lazy evaluation

/// Problem 1: Basic iteration
/// Sum all elements using an iterator
pub fn sum_iter(arr: &[i32]) -> i32 {
    arr.iter().sum()
}

/// Problem 2: Iterator with map
/// Double all elements
pub fn double_all(arr: &[i32]) -> Vec<i32> {
    arr.iter().map(|&x| x * 2).collect()
}

/// Problem 3: Iterator with filter
/// Keep only even numbers
pub fn keep_evens(arr: &[i32]) -> Vec<i32> {
    arr.iter().filter(|&&x| x % 2 == 0).copied().collect()
}

/// Problem 4: Iterator with map and filter
/// Double even numbers only
pub fn double_evens(arr: &[i32]) -> Vec<i32> {
    arr.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * 2)
        .collect()
}

/// Problem 5: Iterator with enumerate
/// Get index and value pairs
pub fn with_indices(arr: &[i32]) -> Vec<(usize, i32)> {
    arr.iter().enumerate().map(|(i, &x)| (i, x)).collect()
}

/// Problem 6: Iterator with zip
/// Combine two slices element-wise
pub fn zip_sum(a: &[i32], b: &[i32]) -> Vec<i32> {
    a.iter().zip(b.iter()).map(|(&x, &y)| x + y).collect()
}

/// Problem 7: Iterator with fold
/// Calculate product of all elements
pub fn product(arr: &[i32]) -> i64 {
    arr.iter().fold(1i64, |acc, &x| acc * x as i64)
}

/// Problem 8: Iterator with any and all
/// Check conditions
pub fn has_negative(arr: &[i32]) -> bool {
    arr.iter().any(|&x| x < 0)
}

pub fn all_positive(arr: &[i32]) -> bool {
    arr.iter().all(|&x| x > 0)
}

/// Problem 9: Iterator with find
/// Find the first negative number
pub fn find_first_negative(arr: &[i32]) -> Option<i32> {
    arr.iter().find(|&&x| x < 0).copied()
}

/// Problem 10: Iterator with position
/// Find the index of the first negative number
pub fn position_of_first_negative(arr: &[i32]) -> Option<usize> {
    arr.iter().position(|&x| x < 0)
}

/// Problem 11: Iterator with take and skip
/// Get the first 3 elements
pub fn first_three(arr: &[i32]) -> Vec<i32> {
    arr.iter().take(3).copied().collect()
}

/// Get elements after the first 3
pub fn skip_three(arr: &[i32]) -> Vec<i32> {
    arr.iter().skip(3).copied().collect()
}

/// Problem 12: Iterator with chain
/// Chain two iterators
pub fn chain_arrays(a: &[i32], b: &[i32]) -> Vec<i32> {
    a.iter().chain(b.iter()).copied().collect()
}

/// Problem 13: Iterator with flat_map
/// Flatten nested structures
pub fn flatten_nested(arr: &[Vec<i32>]) -> Vec<i32> {
    arr.iter().flat_map(|v| v.iter()).copied().collect()
}

/// Problem 14: Iterator with scan
/// Running sum
pub fn running_sum(arr: &[i32]) -> Vec<i32> {
    arr.iter()
        .scan(0, |state, &x| {
            *state += x;
            Some(*state)
        })
        .collect()
}

/// Problem 15: Custom iterator
/// Create a Fibonacci iterator
pub struct Fibonacci {
    a: u64,
    b: u64,
}

impl Fibonacci {
    pub fn new() -> Self {
        Self { a: 0, b: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.a;
        let new_b = self.a + self.b;
        self.a = self.b;
        self.b = new_b;
        Some(result)
    }
}

/// Problem 16: Iterator with windows
/// Get sliding windows of size 3
pub fn windows_of_three(arr: &[i32]) -> Vec<Vec<i32>> {
    arr.windows(3)
        .map(|w| w.to_vec())
        .collect()
}

/// Problem 17: Iterator with chunks
/// Split into chunks of size 2
pub fn chunks_of_two(arr: &[i32]) -> Vec<Vec<i32>> {
    arr.chunks(2)
        .map(|c| c.to_vec())
        .collect()
}

/// Problem 18: Iterator with peekable
/// Look ahead without consuming
pub fn has_consecutive_duplicates(arr: &[i32]) -> bool {
    let mut iter = arr.iter().peekable();
    while let Some(&x) = iter.next() {
        if let Some(&&next) = iter.peek() {
            if x == next {
                return true;
            }
        }
    }
    false
}

/// Problem 19: Iterator with unzip
/// Unzip a vector of tuples
pub fn unzip(pairs: &[(i32, i32)]) -> (Vec<i32>, Vec<i32>) {
    pairs.iter().cloned().unzip()
}

/// Problem 20: Iterator with collect into HashMap
/// Collect into a HashMap
pub fn to_hashmap(pairs: &[(String, i32)]) -> std::collections::HashMap<String, i32> {
    pairs.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_iter() {
        assert_eq!(sum_iter(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_double_all() {
        assert_eq!(double_all(&[1, 2, 3]), vec![2, 4, 6]);
    }

    #[test]
    fn test_keep_evens() {
        assert_eq!(keep_evens(&[1, 2, 3, 4, 5]), vec![2, 4]);
    }

    #[test]
    fn test_double_evens() {
        assert_eq!(double_evens(&[1, 2, 3, 4, 5]), vec![4, 8]);
    }

    #[test]
    fn test_with_indices() {
        assert_eq!(with_indices(&[10, 20, 30]), vec![(0, 10), (1, 20), (2, 30)]);
    }

    #[test]
    fn test_zip_sum() {
        assert_eq!(zip_sum(&[1, 2, 3], &[10, 20, 30]), vec![11, 22, 33]);
    }

    #[test]
    fn test_product() {
        assert_eq!(product(&[1, 2, 3, 4]), 24);
    }

    #[test]
    fn test_has_negative() {
        assert!(has_negative(&[1, -2, 3]));
        assert!(!has_negative(&[1, 2, 3]));
    }

    #[test]
    fn test_all_positive() {
        assert!(all_positive(&[1, 2, 3]));
        assert!(!all_positive(&[1, -2, 3]));
    }

    #[test]
    fn test_find_first_negative() {
        assert_eq!(find_first_negative(&[1, -2, 3]), Some(-2));
        assert_eq!(find_first_negative(&[1, 2, 3]), None);
    }

    #[test]
    fn test_position_of_first_negative() {
        assert_eq!(position_of_first_negative(&[1, -2, 3]), Some(1));
        assert_eq!(position_of_first_negative(&[1, 2, 3]), None);
    }

    #[test]
    fn test_first_three() {
        assert_eq!(first_three(&[1, 2, 3, 4, 5]), vec![1, 2, 3]);
    }

    #[test]
    fn test_skip_three() {
        assert_eq!(skip_three(&[1, 2, 3, 4, 5]), vec![4, 5]);
    }

    #[test]
    fn test_chain_arrays() {
        assert_eq!(chain_arrays(&[1, 2], &[3, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_flatten_nested() {
        let nested = vec![vec![1, 2], vec![3, 4], vec![5]];
        assert_eq!(flatten_nested(&nested), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_running_sum() {
        assert_eq!(running_sum(&[1, 2, 3, 4, 5]), vec![1, 3, 6, 10, 15]);
    }

    #[test]
    fn test_fibonacci() {
        let fib: Vec<u64> = Fibonacci::new().take(10).collect();
        assert_eq!(fib, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn test_windows_of_three() {
        assert_eq!(
            windows_of_three(&[1, 2, 3, 4, 5]),
            vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]
        );
    }

    #[test]
    fn test_chunks_of_two() {
        assert_eq!(
            chunks_of_two(&[1, 2, 3, 4, 5]),
            vec![vec![1, 2], vec![3, 4], vec![5]]
        );
    }

    #[test]
    fn test_has_consecutive_duplicates() {
        assert!(has_consecutive_duplicates(&[1, 2, 2, 3]));
        assert!(!has_consecutive_duplicates(&[1, 2, 3]));
    }

    #[test]
    fn test_unzip() {
        let pairs = vec![(1, 10), (2, 20), (3, 30)];
        let (a, b) = unzip(&pairs);
        assert_eq!(a, vec![1, 2, 3]);
        assert_eq!(b, vec![10, 20, 30]);
    }

    #[test]
    fn test_to_hashmap() {
        let pairs = vec![
            ("one".to_string(), 1),
            ("two".to_string(), 2),
        ];
        let map = to_hashmap(&pairs);
        assert_eq!(map.get("one"), Some(&1));
    }
}
