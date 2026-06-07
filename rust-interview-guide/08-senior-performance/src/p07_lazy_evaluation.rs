/// Problem: Lazy Evaluation
///
/// Master lazy evaluation in Rust.
///
/// Key Concepts:
/// - Lazy iterators
/// - Lazy initialization
/// - Lazy computation
/// - On-demand evaluation
/// - Deferred execution

/// Problem 1: Lazy iterator
/// Iterators are lazy by default
pub fn lazy_iterator(data: &[i32]) -> Vec<i32> {
    data.iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * 2)
        .collect()
}

/// Problem 2: Lazy initialization with OnceCell
/// Lazy initialization (simulated)
pub struct LazyInit {
    value: Option<i32>,
}

impl LazyInit {
    pub fn new() -> Self {
        Self { value: None }
    }

    pub fn get_or_init(&mut self, init: impl FnOnce() -> i32) -> i32 {
        if let Some(value) = self.value {
            return value;
        }
        let value = init();
        self.value = Some(value);
        value
    }
}

/// Problem 3: Lazy computation
/// Compute only when needed
pub struct LazyComputation {
    computed: Option<i32>,
    computation: Option<Box<dyn FnOnce() -> i32>>,
}

impl LazyComputation {
    pub fn new(computation: Box<dyn FnOnce() -> i32>) -> Self {
        Self {
            computed: None,
            computation: Some(computation),
        }
    }

    pub fn get(&mut self) -> i32 {
        if let Some(value) = self.computed {
            return value;
        }
        let computation = self.computation.take().unwrap();
        let value = computation();
        self.computed = Some(value);
        value
    }
}

/// Problem 4: Lazy sequence
/// Generate sequence lazily
pub struct LazySequence {
    current: i32,
    step: i32,
}

impl LazySequence {
    pub fn new(start: i32, step: i32) -> Self {
        Self { current: start, step }
    }

    pub fn next(&mut self) -> i32 {
        let value = self.current;
        self.current += self.step;
        value
    }
}

/// Problem 5: Lazy filter
/// Filter lazily
pub fn lazy_filter(data: &[i32], predicate: impl Fn(&i32) -> bool) -> Vec<i32> {
    data.iter()
        .filter(|x| predicate(x))
        .copied()
        .collect()
}

/// Problem 6: Lazy map
/// Map lazily
pub fn lazy_map(data: &[i32], mapper: impl Fn(i32) -> i32) -> Vec<i32> {
    data.iter()
        .map(|&x| mapper(x))
        .collect()
}

/// Problem 7: Lazy fold
/// Fold lazily (not truly lazy, but demonstrates concept)
pub fn lazy_fold(data: &[i32], init: i32, f: impl Fn(i32, i32) -> i32) -> i32 {
    data.iter().fold(init, |acc, &x| f(acc, x))
}

/// Problem 8: Lazy concatenation
/// Concatenate lazily
pub fn lazy_concat(a: &[i32], b: &[i32]) -> Vec<i32> {
    a.iter().chain(b.iter()).copied().collect()
}

/// Problem 9: Lazy flattening
/// Flatten lazily
pub fn lazy_flatten(data: &[Vec<i32>]) -> Vec<i32> {
    data.iter()
        .flat_map(|v| v.iter())
        .copied()
        .collect()
}

/// Problem 10: Lazy take
/// Take n elements lazily
pub fn lazy_take(data: &[i32], n: usize) -> Vec<i32> {
    data.iter().take(n).copied().collect()
}

/// Problem 11: Lazy skip
/// Skip n elements lazily
pub fn lazy_skip(data: &[i32], n: usize) -> Vec<i32> {
    data.iter().skip(n).copied().collect()
}

/// Problem 12: Lazy zip
/// Zip lazily
pub fn lazy_zip(a: &[i32], b: &[i32]) -> Vec<(i32, i32)> {
    a.iter().zip(b.iter()).map(|(&x, &y)| (x, y)).collect()
}

/// Problem 13: Lazy enumerate
/// Enumerate lazily
pub fn lazy_enumerate(data: &[i32]) -> Vec<(usize, i32)> {
    data.iter().enumerate().map(|(i, &x)| (i, x)).collect()
}

/// Problem 14: Lazy chain of operations
/// Chain lazy operations
pub fn lazy_chain(data: &[i32]) -> Vec<i32> {
    data.iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * 2)
        .take(5)
        .collect()
}

/// Problem 15: Lazy with caching
/// Cache lazy computation results
pub struct CachedLazy {
    cache: std::collections::HashMap<i32, i32>,
}

impl CachedLazy {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn compute(&mut self, key: i32, f: impl Fn(i32) -> i32) -> i32 {
        if let Some(&value) = self.cache.get(&key) {
            return value;
        }
        let value = f(key);
        self.cache.insert(key, value);
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_iterator() {
        assert_eq!(lazy_iterator(&[1, -2, 3, -4, 5]), vec![2, 6, 10]);
    }

    #[test]
    fn test_lazy_init() {
        let mut lazy = LazyInit::new();
        assert_eq!(lazy.get_or_init(|| 42), 42);
        assert_eq!(lazy.get_or_init(|| 100), 42); // Returns cached
    }

    #[test]
    fn test_lazy_computation() {
        let mut lazy = LazyComputation::new(Box::new(|| 42));
        assert_eq!(lazy.get(), 42);
        assert_eq!(lazy.get(), 42); // Returns cached
    }

    #[test]
    fn test_lazy_sequence() {
        let mut seq = LazySequence::new(0, 2);
        assert_eq!(seq.next(), 0);
        assert_eq!(seq.next(), 2);
        assert_eq!(seq.next(), 4);
    }

    #[test]
    fn test_lazy_filter() {
        assert_eq!(lazy_filter(&[1, 2, 3, 4, 5], |&x| x % 2 == 0), vec![2, 4]);
    }

    #[test]
    fn test_lazy_map() {
        assert_eq!(lazy_map(&[1, 2, 3], |x| x * 2), vec![2, 4, 6]);
    }

    #[test]
    fn test_lazy_fold() {
        assert_eq!(lazy_fold(&[1, 2, 3, 4, 5], 0, |acc, x| acc + x), 15);
    }

    #[test]
    fn test_lazy_concat() {
        assert_eq!(lazy_concat(&[1, 2], &[3, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_lazy_flatten() {
        let data = vec![vec![1, 2], vec![3, 4], vec![5]];
        assert_eq!(lazy_flatten(&data), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_lazy_take() {
        assert_eq!(lazy_take(&[1, 2, 3, 4, 5], 3), vec![1, 2, 3]);
    }

    #[test]
    fn test_lazy_skip() {
        assert_eq!(lazy_skip(&[1, 2, 3, 4, 5], 2), vec![3, 4, 5]);
    }

    #[test]
    fn test_lazy_zip() {
        assert_eq!(lazy_zip(&[1, 2, 3], &[4, 5, 6]), vec![(1, 4), (2, 5), (3, 6)]);
    }

    #[test]
    fn test_lazy_enumerate() {
        assert_eq!(lazy_enumerate(&[10, 20, 30]), vec![(0, 10), (1, 20), (2, 30)]);
    }

    #[test]
    fn test_lazy_chain() {
        assert_eq!(lazy_chain(&[1, -2, 3, -4, 5, 6, 7]), vec![2, 6, 10, 12, 14]);
    }

    #[test]
    fn test_cached_lazy() {
        let mut lazy = CachedLazy::new();
        assert_eq!(lazy.compute(5, |x| x * 2), 10);
        assert_eq!(lazy.compute(5, |x| x * 2), 10); // Cached
    }
}
