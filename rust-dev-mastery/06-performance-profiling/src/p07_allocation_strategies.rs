//! # Allocation Strategies
//!
//! Heap allocations are expensive. This module covers strategies for minimizing
//! allocations, choosing the right data types, and using copy-on-write and
//! small-vector optimizations.
//!
//! ## Key Concepts
//! - **Pre-allocation**: `Vec::with_capacity`, `String::with_capacity`
//! - **String vs &str**: Borrowed strings avoid allocation
//! - **Cow**: Clone-on-write for deferred allocation
//! - **SmallVec**: Inline small vectors to avoid heap allocation
//! - **Avoiding temporary allocations**: Reusing buffers, passing &mut Vec

use std::borrow::Cow;

/// Demonstrates the cost difference between pre-allocated and growing Vecs.
pub fn build_vec_growing(n: usize) -> Vec<usize> {
    let mut v = Vec::new();
    for i in 0..n {
        v.push(i);
    }
    v
}

pub fn build_vec_preallocated(n: usize) -> Vec<usize> {
    let mut v = Vec::with_capacity(n);
    for i in 0..n {
        v.push(i);
    }
    v
}

/// Demonstrates reusing a buffer across iterations.
/// Instead of allocating a new Vec each time, clear and reuse.
pub fn process_batches_reuse_buffer(data: &[&[u8]], buffer: &mut Vec<u8>) -> Vec<Vec<u8>> {
    let mut results = Vec::with_capacity(data.len());

    for batch in data {
        buffer.clear();
        buffer.extend_from_slice(batch);
        // Process in-place
        buffer.iter_mut().for_each(|b| *b = b.wrapping_add(1));
        results.push(buffer.clone());
    }

    results
}

pub fn process_batches_new_buffer(data: &[&[u8]]) -> Vec<Vec<u8>> {
    data.iter()
        .map(|batch| {
            let mut buf = Vec::with_capacity(batch.len());
            buf.extend_from_slice(batch);
            buf.iter_mut().for_each(|b| *b = b.wrapping_add(1));
            buf
        })
        .collect()
}

/// Demonstrates Cow<str> for strings that are usually borrowed but
/// occasionally need to be owned (e.g., when transformation is needed).
pub fn normalize_name(name: &str) -> Cow<'_, str> {
    if name.chars().all(|c| c.is_lowercase() || c == '_') {
        // Already normalized, no allocation needed
        Cow::Borrowed(name)
    } else {
        // Need to transform — allocate a new String
        Cow::Owned(name.to_lowercase().replace(' ', "_"))
    }
}

/// Demonstrates Cow<[T]> for slices that may or may not need modification.
pub fn ensure_prefix<'a>(data: &'a [u8], prefix: &[u8]) -> Cow<'a, [u8]> {
    if data.starts_with(prefix) {
        Cow::Borrowed(data)
    } else {
        let mut owned = Vec::with_capacity(prefix.len() + data.len());
        owned.extend_from_slice(prefix);
        owned.extend_from_slice(data);
        Cow::Owned(owned)
    }
}

/// A simple inline vector that stores up to N elements on the stack.
/// Falls back to heap allocation for larger collections.
/// This is the concept behind SmallVec.
pub enum InlineVec<T, const N: usize> {
    Inline { data: [std::mem::MaybeUninit<T>; N], len: usize },
    Heap(Vec<T>),
}

impl<T, const N: usize> InlineVec<T, N> {
    pub fn new() -> Self {
        InlineVec::Inline {
            data: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        match self {
            InlineVec::Inline { data, len } => {
                if *len < N {
                    data[*len] = std::mem::MaybeUninit::new(value);
                    *len += 1;
                } else {
                    // Promote to heap
                    let mut v = Vec::with_capacity(N + 1);
                    for i in 0..N {
                        v.push(unsafe { data[i].assume_init_read() });
                    }
                    v.push(value);
                    *self = InlineVec::Heap(v);
                }
            }
            InlineVec::Heap(v) => v.push(value),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            InlineVec::Inline { len, .. } => *len,
            InlineVec::Heap(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_inline(&self) -> bool {
        matches!(self, InlineVec::Inline { .. })
    }

    pub fn iter(&self) -> InlineVecIter<'_, T, N> {
        InlineVecIter { vec: self, index: 0 }
    }
}

pub struct InlineVecIter<'a, T, const N: usize> {
    vec: &'a InlineVec<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for InlineVecIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.vec {
            InlineVec::Inline { data, len } => {
                if self.index < *len {
                    let item = unsafe { data[self.index].assume_init_ref() };
                    self.index += 1;
                    Some(item)
                } else {
                    None
                }
            }
            InlineVec::Heap(v) => {
                if self.index < v.len() {
                    let item = &v[self.index];
                    self.index += 1;
                    Some(item)
                } else {
                    None
                }
            }
        }
    }
}

impl<T, const N: usize> Drop for InlineVec<T, N> {
    fn drop(&mut self) {
        if let InlineVec::Inline { data, len } = self {
            for i in 0..*len {
                unsafe {
                    data[i].assume_init_drop();
                }
            }
        }
    }
}

/// A string builder that reuses a buffer for building strings.
/// Avoids repeated allocations when building many strings.
pub struct StringBuilder {
    buffer: String,
}

impl StringBuilder {
    pub fn new() -> Self {
        StringBuilder {
            buffer: String::with_capacity(256),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        StringBuilder {
            buffer: String::with_capacity(capacity),
        }
    }

    /// Builds a string using the internal buffer, returning the result.
    /// The buffer is retained for reuse.
    pub fn build<F>(&mut self, f: F) -> String
    where
        F: FnOnce(&mut String),
    {
        self.buffer.clear();
        f(&mut self.buffer);
        self.buffer.clone()
    }

    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
}

/// Demonstrates avoiding String allocations with &str parameters.
pub fn greet(name: &str) -> String {
    // Accepts &str, not String — caller decides whether to allocate
    format!("Hello, {name}!")
}

pub fn process_names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| greet(name)).collect()
}

/// Demonstrates pre-allocating a HashMap.
pub fn build_map_preallocated<I, K, V>(iter: I) -> std::collections::HashMap<K, V>
where
    I: Iterator<Item = (K, V)>,
    K: Eq + std::hash::Hash,
{
    let hint = iter.size_hint();
    let capacity = hint.1.unwrap_or(hint.0);
    let mut map = std::collections::HashMap::with_capacity(capacity);
    for (k, v) in iter {
        map.insert(k, v);
    }
    map
}

/// A pool of reusable String objects to avoid allocation churn.
pub struct StringPool {
    available: Vec<String>,
    in_use: usize,
}

impl StringPool {
    pub fn new(initial_size: usize) -> Self {
        StringPool {
            available: (0..initial_size).map(|_| String::with_capacity(128)).collect(),
            in_use: 0,
        }
    }

    pub fn acquire(&mut self) -> String {
        self.in_use += 1;
        self.available.pop().unwrap_or_else(|| String::with_capacity(128))
    }

    pub fn release(&mut self, mut s: String) {
        s.clear();
        self.available.push(s);
        self.in_use -= 1;
    }

    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    pub fn in_use_count(&self) -> usize {
        self.in_use
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_construction() {
        let v1 = build_vec_growing(1000);
        let v2 = build_vec_preallocated(1000);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_buffer_reuse() {
        let data: Vec<&[u8]> = vec![b"hello", b"world", b"test"];
        let mut buffer = Vec::new();
        let results = process_batches_reuse_buffer(&data, &mut buffer);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], b"ifmmp"); // hello + 1
    }

    #[test]
    fn test_normalize_name_borrowed() {
        let result = normalize_name("already_normalized");
        match result {
            Cow::Borrowed(s) => assert_eq!(s, "already_normalized"),
            Cow::Owned(_) => panic!("Expected Borrowed"),
        }
    }

    #[test]
    fn test_normalize_name_owned() {
        let result = normalize_name("Needs Normalization");
        match result {
            Cow::Owned(s) => assert_eq!(s, "needs_normalization"),
            Cow::Borrowed(_) => panic!("Expected Owned"),
        }
    }

    #[test]
    fn test_ensure_prefix_with_prefix() {
        let data = b"prefix_data";
        let result = ensure_prefix(data, b"prefix_");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(&*result, b"prefix_data");
    }

    #[test]
    fn test_ensure_prefix_without_prefix() {
        let data = b"data";
        let result = ensure_prefix(data, b"prefix_");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(&*result, b"prefix_data");
    }

    #[test]
    fn test_inline_vec_basic() {
        let mut v = InlineVec::<i32, 4>::new();
        v.push(1);
        v.push(2);
        v.push(3);

        assert!(v.is_inline());
        assert_eq!(v.len(), 3);

        let items: Vec<&i32> = v.iter().collect();
        assert_eq!(items, vec![&1, &2, &3]);
    }

    #[test]
    fn test_inline_vec_spill() {
        let mut v = InlineVec::<i32, 2>::new();
        v.push(1);
        v.push(2);
        assert!(v.is_inline());

        v.push(3); // Should spill to heap
        assert!(!v.is_inline());
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn test_inline_vec_empty() {
        let v = InlineVec::<i32, 4>::new();
        assert!(v.is_empty());
        assert_eq!(v.len(), 0);
        assert!(v.iter().next().is_none());
    }

    #[test]
    fn test_string_builder() {
        let mut sb = StringBuilder::new();

        let result = sb.build(|buf| {
            buf.push_str("Hello, ");
            buf.push_str("World!");
        });
        assert_eq!(result, "Hello, World!");

        // Reuse for another string
        let result2 = sb.build(|buf| {
            buf.push_str("Second string");
        });
        assert_eq!(result2, "Second string");
    }

    #[test]
    fn test_greet() {
        assert_eq!(greet("Alice"), "Hello, Alice!");
    }

    #[test]
    fn test_process_names() {
        let names = vec!["Alice", "Bob", "Charlie"];
        let greetings = process_names(&names);
        assert_eq!(greetings.len(), 3);
        assert_eq!(greetings[0], "Hello, Alice!");
    }

    #[test]
    fn test_build_map_preallocated() {
        let data = vec![(1, "a"), (2, "b"), (3, "c")];
        let map = build_map_preallocated(data.into_iter());
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&1), Some(&"a"));
    }

    #[test]
    fn test_string_pool() {
        let mut pool = StringPool::new(3);
        assert_eq!(pool.available_count(), 3);

        let s1 = pool.acquire();
        let s2 = pool.acquire();
        assert_eq!(pool.available_count(), 1);
        assert_eq!(pool.in_use_count(), 2);

        pool.release(s1);
        assert_eq!(pool.available_count(), 2);
        assert_eq!(pool.in_use_count(), 1);

        pool.release(s2);
        assert_eq!(pool.available_count(), 3);
        assert_eq!(pool.in_use_count(), 0);
    }

    #[test]
    fn test_string_pool_exhaustion() {
        let mut pool = StringPool::new(1);
        let _s1 = pool.acquire();
        // Pool is empty, should create new String
        let s2 = pool.acquire();
        assert_eq!(pool.in_use_count(), 2);
        drop(s2);
    }
}
