/// Problem: Iterator Pattern
///
/// Master the iterator pattern in Rust.
///
/// Key Concepts:
/// - Iterator trait
/// - Custom iterators
/// - Iterator adaptors
/// - Lazy evaluation
/// - Composition

/// Problem 1: Basic iterator
/// Create basic iterator
pub struct Counter {
    count: u32,
    max: u32,
}

impl Counter {
    pub fn new(max: u32) -> Self {
        Self { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max {
            let value = self.count;
            self.count += 1;
            Some(value)
        } else {
            None
        }
    }
}

/// Problem 2: Iterator with filter
/// Create filtering iterator
pub struct FilterIterator<I, F> {
    iter: I,
    predicate: F,
}

impl<I, F> FilterIterator<I, F>
where
    I: Iterator,
    F: Fn(&I::Item) -> bool,
{
    pub fn new(iter: I, predicate: F) -> Self {
        Self { iter, predicate }
    }
}

impl<I, F> Iterator for FilterIterator<I, F>
where
    I: Iterator,
    F: Fn(&I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(item) => {
                    if (self.predicate)(&item) {
                        return Some(item);
                    }
                }
                None => return None,
            }
        }
    }
}

/// Problem 3: Iterator with map
/// Create mapping iterator
pub struct MapIterator<I, F> {
    iter: I,
    mapper: F,
}

impl<I, F, T> MapIterator<I, F>
where
    I: Iterator,
    F: Fn(I::Item) -> T,
{
    pub fn new(iter: I, mapper: F) -> Self {
        Self { iter, mapper }
    }
}

impl<I, F, T> Iterator for MapIterator<I, F>
where
    I: Iterator,
    F: Fn(I::Item) -> T,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| (self.mapper)(item))
    }
}

/// Problem 4: Iterator with take
/// Create take iterator
pub struct TakeIterator<I> {
    iter: I,
    remaining: usize,
}

impl<I> TakeIterator<I> {
    pub fn new(iter: I, n: usize) -> Self {
        Self { iter, remaining: n }
    }
}

impl<I: Iterator> Iterator for TakeIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining > 0 {
            self.remaining -= 1;
            self.iter.next()
        } else {
            None
        }
    }
}

/// Problem 5: Iterator with skip
/// Create skip iterator
pub struct SkipIterator<I> {
    iter: I,
    skipped: bool,
    n: usize,
}

impl<I> SkipIterator<I> {
    pub fn new(iter: I, n: usize) -> Self {
        Self {
            iter,
            skipped: false,
            n,
        }
    }
}

impl<I: Iterator> Iterator for SkipIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.skipped {
            for _ in 0..self.n {
                self.iter.next();
            }
            self.skipped = true;
        }
        self.iter.next()
    }
}

/// Problem 6: Iterator with chain
/// Chain two iterators
pub struct ChainIterator<I1, I2> {
    first: I1,
    second: I2,
}

impl<I1, I2> ChainIterator<I1, I2> {
    pub fn new(first: I1, second: I2) -> Self {
        Self { first, second }
    }
}

impl<I1, I2> Iterator for ChainIterator<I1, I2>
where
    I1: Iterator,
    I2: Iterator<Item = I1::Item>,
{
    type Item = I1::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.next().or_else(|| self.second.next())
    }
}

/// Problem 7: Iterator with zip
/// Zip two iterators
pub struct ZipIterator<I1, I2> {
    first: I1,
    second: I2,
}

impl<I1, I2> ZipIterator<I1, I2> {
    pub fn new(first: I1, second: I2) -> Self {
        Self { first, second }
    }
}

impl<I1, I2> Iterator for ZipIterator<I1, I2>
where
    I1: Iterator,
    I2: Iterator,
{
    type Item = (I1::Item, I2::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.first.next(), self.second.next()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

/// Problem 8: Iterator with enumerate
/// Enumerate iterator
pub struct EnumerateIterator<I> {
    iter: I,
    index: usize,
}

impl<I> EnumerateIterator<I> {
    pub fn new(iter: I) -> Self {
        Self { iter, index: 0 }
    }
}

impl<I: Iterator> Iterator for EnumerateIterator<I> {
    type Item = (usize, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            let index = self.index;
            self.index += 1;
            (index, item)
        })
    }
}

/// Problem 9: Iterator with flatten
/// Flatten nested iterators
pub struct FlattenIterator<I, T> {
    iter: I,
    current: Option<T>,
}

impl<I, T> FlattenIterator<I, T>
where
    I: Iterator<Item = T>,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            current: None,
        }
    }
}

impl<I, T> Iterator for FlattenIterator<I, T>
where
    I: Iterator<Item = T>,
    T: Iterator,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut current) = self.current {
                if let Some(item) = current.next() {
                    return Some(item);
                }
            }
            match self.iter.next() {
                Some(iter) => self.current = Some(iter),
                None => return None,
            }
        }
    }
}

/// Problem 10: Iterator with peek
/// Peek at next element
pub struct PeekIterator<I: Iterator> {
    iter: I,
    peeked: Option<Option<I::Item>>,
}

impl<I: Iterator> PeekIterator<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Option<&I::Item> {
        if self.peeked.is_none() {
            self.peeked = Some(self.iter.next());
        }
        self.peeked.as_ref().unwrap().as_ref()
    }
}

impl<I: Iterator> Iterator for PeekIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(peeked) = self.peeked.take() {
            peeked
        } else {
            self.iter.next()
        }
    }
}

/// Problem 11: Iterator with windows
/// Sliding windows
pub struct WindowsIterator<I: Iterator> {
    iter: I,
    window: Vec<I::Item>,
    size: usize,
}

impl<I: Iterator> WindowsIterator<I>
where
    I::Item: Clone,
{
    pub fn new(mut iter: I, size: usize) -> Self {
        let mut window = Vec::new();
        for _ in 0..size {
            if let Some(item) = iter.next() {
                window.push(item);
            }
        }
        Self { iter, window, size }
    }
}

impl<I: Iterator> Iterator for WindowsIterator<I>
where
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.window.len() < self.size {
            return None;
        }
        let result = self.window.clone();
        if let Some(item) = self.iter.next() {
            self.window.remove(0);
            self.window.push(item);
        } else {
            self.window.remove(0);
        }
        Some(result)
    }
}

/// Problem 12: Iterator with chunks
/// Chunk elements
pub struct ChunksIterator<I: Iterator> {
    iter: I,
    size: usize,
}

impl<I: Iterator> ChunksIterator<I> {
    pub fn new(iter: I, size: usize) -> Self {
        Self { iter, size }
    }
}

impl<I: Iterator> Iterator for ChunksIterator<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::new();
        for _ in 0..self.size {
            match self.iter.next() {
                Some(item) => chunk.push(item),
                None => break,
            }
        }
        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    }
}

/// Problem 13: Iterator with unique
/// Unique elements
pub struct UniqueIterator<I: Iterator> {
    iter: I,
    seen: std::collections::HashSet<u64>,
}

impl<I: Iterator> UniqueIterator<I>
where
    I::Item: std::hash::Hash + Eq,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            seen: std::collections::HashSet::new(),
        }
    }
}

impl<I: Iterator> Iterator for UniqueIterator<I>
where
    I::Item: std::hash::Hash + Eq + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(item) => {
                    let hash = {
                        use std::hash::{Hash, Hasher};
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        item.hash(&mut hasher);
                        hasher.finish()
                    };
                    if self.seen.insert(hash) {
                        return Some(item);
                    }
                }
                None => return None,
            }
        }
    }
}

/// Problem 14: Iterator with intersperse
/// Intersperse elements
pub struct IntersperseIterator<I: Iterator> {
    iter: I,
    separator: I::Item,
    needs_sep: bool,
}

impl<I: Iterator> IntersperseIterator<I>
where
    I::Item: Clone,
{
    pub fn new(iter: I, separator: I::Item) -> Self {
        Self {
            iter,
            separator,
            needs_sep: false,
        }
    }
}

impl<I: Iterator> Iterator for IntersperseIterator<I>
where
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.needs_sep {
            self.needs_sep = false;
            Some(self.separator.clone())
        } else {
            self.needs_sep = true;
            self.iter.next()
        }
    }
}

/// Problem 15: Iterator with step_by
/// Step by n elements
pub struct StepByIterator<I: Iterator> {
    iter: I,
    step: usize,
    first: bool,
}

impl<I: Iterator> StepByIterator<I> {
    pub fn new(iter: I, step: usize) -> Self {
        Self {
            iter,
            step,
            first: true,
        }
    }
}

impl<I: Iterator> Iterator for StepByIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            return self.iter.next();
        }
        for _ in 0..self.step - 1 {
            self.iter.next();
        }
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_iterator() {
        let counter = Counter::new(5);
        let result: Vec<u32> = counter.collect();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_filter_iterator() {
        let iter = FilterIterator::new(0..10, |&x| x % 2 == 0);
        let result: Vec<i32> = iter.collect();
        assert_eq!(result, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_map_iterator() {
        let iter = MapIterator::new(0..5, |x| x * 2);
        let result: Vec<i32> = iter.collect();
        assert_eq!(result, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_take_iterator() {
        let iter = TakeIterator::new(0..10, 3);
        let result: Vec<i32> = iter.collect();
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_skip_iterator() {
        let iter = SkipIterator::new(0..5, 2);
        let result: Vec<i32> = iter.collect();
        assert_eq!(result, vec![2, 3, 4]);
    }

    #[test]
    fn test_chain_iterator() {
        let iter = ChainIterator::new(0..3, 3..6);
        let result: Vec<i32> = iter.collect();
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_zip_iterator() {
        let iter = ZipIterator::new(0..3, 10..13);
        let result: Vec<(i32, i32)> = iter.collect();
        assert_eq!(result, vec![(0, 10), (1, 11), (2, 12)]);
    }

    #[test]
    fn test_enumerate_iterator() {
        let iter = EnumerateIterator::new(vec!["a", "b", "c"].into_iter());
        let result: Vec<(usize, &str)> = iter.collect();
        assert_eq!(result, vec![(0, "a"), (1, "b"), (2, "c")]);
    }

    #[test]
    fn test_flatten_iterator() {
        let data = vec![vec![1, 2], vec![3, 4], vec![5]];
        let iter = FlattenIterator::new(data.into_iter().map(|v| v.into_iter()));
        let result: Vec<i32> = iter.collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_peek_iterator() {
        let mut iter = PeekIterator::new(0..3);
        assert_eq!(iter.peek(), Some(&0));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.peek(), Some(&1));
    }

    #[test]
    fn test_windows_iterator() {
        let iter = WindowsIterator::new(0..5, 3);
        let result: Vec<Vec<i32>> = iter.collect();
        assert_eq!(result, vec![vec![0, 1, 2], vec![1, 2, 3], vec![2, 3, 4]]);
    }

    #[test]
    fn test_chunks_iterator() {
        let iter = ChunksIterator::new(0..7, 3);
        let result: Vec<Vec<i32>> = iter.collect();
        assert_eq!(result, vec![vec![0, 1, 2], vec![3, 4, 5], vec![6]]);
    }

    #[test]
    fn test_step_by_iterator() {
        let iter = StepByIterator::new(0..10, 2);
        let result: Vec<i32> = iter.collect();
        assert_eq!(result, vec![0, 2, 4, 6, 8]);
    }
}
