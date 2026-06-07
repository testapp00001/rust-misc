//! # Cache Optimization
//!
//! Modern CPUs spend more time waiting for data from memory than executing
//! instructions. Understanding cache hierarchies and designing cache-friendly
//! data structures is one of the most impactful performance optimizations.
//!
//! ## Key Concepts
//! - **Cache lines**: 64-byte blocks loaded from memory
//! - **Spatial locality**: Accessing nearby memory addresses is fast
//! - **Temporal locality**: Reusing recently accessed data is fast
//! - **False sharing**: Different threads on different cache lines sharing data
//! - **Prefetching**: Hinting the CPU about future memory accesses

use std::collections::HashMap;

/// A cache-friendly flat array map that stores keys and values in parallel arrays.
/// Better cache performance than HashMap for small to medium datasets with
/// sequential access patterns.
pub struct FlatMap<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K: Eq, V> FlatMap<K, V> {
    pub fn new() -> Self {
        FlatMap {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(idx) = self.keys.iter().position(|k| k == &key) {
            self.values[idx] = value;
        } else {
            self.keys.push(key);
            self.values.push(value);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.keys
            .iter()
            .position(|k| k == key)
            .map(|idx| &self.values[idx])
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Iterates over key-value pairs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys.iter().zip(self.values.iter())
    }
}

/// A cache-optimized bitset that packs 64 bits per u64.
/// Much more cache-friendly than a Vec<bool> for large sets.
pub struct Bitset {
    words: Vec<u64>,
    len: usize,
}

impl Bitset {
    pub fn new(len: usize) -> Self {
        Bitset {
            words: vec![0u64; (len + 63) / 64],
            len,
        }
    }

    pub fn set(&mut self, index: usize) {
        debug_assert!(index < self.len);
        self.words[index / 64] |= 1u64 << (index % 64);
    }

    pub fn clear(&mut self, index: usize) {
        debug_assert!(index < self.len);
        self.words[index / 64] &= !(1u64 << (index % 64));
    }

    pub fn get(&self, index: usize) -> bool {
        debug_assert!(index < self.len);
        (self.words[index / 64] >> (index % 64)) & 1 != 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Counts set bits using Brian Kernighan's algorithm.
    pub fn count_ones(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    /// Iterates over set bit indices.
    pub fn iter_set(&self) -> impl Iterator<Item = usize> + '_ {
        let len = self.len;
        self.words.iter().enumerate().flat_map(move |(word_idx, &word)| {
            (0..64).filter_map(move |bit_idx| {
                let index = word_idx * 64 + bit_idx;
                if index < len && (word >> bit_idx) & 1 != 0 {
                    Some(index)
                } else {
                    None
                }
            })
        })
    }
}

/// A struct-of-arrays (SoA) container for better cache utilization.
/// When you only need to access one field, SoA keeps that field's data
/// contiguous in cache.
pub struct SoA {
    pub ids: Vec<u64>,
    pub names: Vec<String>,
    pub scores: Vec<f64>,
    pub active: Vec<bool>,
}

impl SoA {
    pub fn new() -> Self {
        SoA {
            ids: Vec::new(),
            names: Vec::new(),
            scores: Vec::new(),
            active: Vec::new(),
        }
    }

    pub fn push(&mut self, id: u64, name: String, score: f64, active: bool) {
        self.ids.push(id);
        self.names.push(name);
        self.scores.push(score);
        self.active.push(active);
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Only touches the scores array — cache-friendly!
    pub fn average_score(&self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }
        self.scores.iter().sum::<f64>() / self.scores.len() as f64
    }

    /// Only touches ids and scores — two contiguous arrays.
    pub fn top_scores(&self, n: usize) -> Vec<(u64, f64)> {
        let mut pairs: Vec<(u64, f64)> = self
            .ids
            .iter()
            .zip(self.scores.iter())
            .map(|(&id, &score)| (id, score))
            .collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        pairs.into_iter().take(n).collect()
    }
}

/// An array-of-structs (AoS) version for comparison.
#[derive(Clone)]
pub struct Record {
    pub id: u64,
    pub name: String,
    pub score: f64,
    pub active: bool,
}

pub struct AoS {
    records: Vec<Record>,
}

impl AoS {
    pub fn new() -> Self {
        AoS { records: Vec::new() }
    }

    pub fn push(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Touches ALL fields of each record — more cache misses than SoA version.
    pub fn average_score(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.records.iter().map(|r| r.score).sum::<f64>() / self.records.len() as f64
    }
}

/// A cache-friendly circular buffer that avoids shifting elements.
pub struct CircularBuffer<T> {
    data: Vec<Option<T>>,
    head: usize,
    tail: usize,
    len: usize,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let mut data = Vec::with_capacity(capacity);
        data.resize_with(capacity, || None);
        CircularBuffer {
            data,
            head: 0,
            tail: 0,
            len: 0,
            capacity,
        }
    }

    pub fn push(&mut self, value: T) -> Option<T> {
        let old = self.data[self.tail].take();
        self.data[self.tail] = Some(value);
        self.tail = (self.tail + 1) % self.capacity;

        if self.len == self.capacity {
            self.head = (self.head + 1) % self.capacity;
            old
        } else {
            self.len += 1;
            None
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = self.data[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.len -= 1;
        value
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }
}

/// A blocked matrix that stores data in cache-line-sized tiles.
/// Better for matrix operations that access sub-matrices.
pub struct BlockedMatrix {
    data: Vec<f64>,
    rows: usize,
    cols: usize,
    block_size: usize,
}

impl BlockedMatrix {
    pub fn new(rows: usize, cols: usize, block_size: usize) -> Self {
        BlockedMatrix {
            data: vec![0.0; rows * cols],
            rows,
            cols,
            block_size,
        }
    }

    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        let idx = self.block_index(row, col);
        self.data[idx] = value;
    }

    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[self.block_index(row, col)]
    }

    fn block_index(&self, row: usize, col: usize) -> usize {
        let block_row = row / self.block_size;
        let block_col = col / self.block_size;
        let local_row = row % self.block_size;
        let local_col = col % self.block_size;

        let blocks_per_row = (self.cols + self.block_size - 1) / self.block_size;
        let block_idx = block_row * blocks_per_row + block_col;
        let local_idx = local_row * self.block_size + local_col;

        block_idx * self.block_size * self.block_size + local_idx
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
}

/// A sorted container that uses binary search for lookups.
/// Much more cache-friendly than a tree-based structure for read-heavy workloads.
pub struct SortedVec<T: Ord> {
    data: Vec<T>,
}

impl<T: Ord> SortedVec<T> {
    pub fn new() -> Self {
        SortedVec { data: Vec::new() }
    }

    pub fn insert(&mut self, value: T) {
        let pos = self.data.binary_search(&value).unwrap_or_else(|e| e);
        self.data.insert(pos, value);
    }

    pub fn contains(&self, value: &T) -> bool {
        self.data.binary_search(value).is_ok()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }
}

/// Demonstrates false sharing avoidance by padding cache lines.
#[repr(C)]
pub struct PaddedCounter {
    pub value: u64,
    _padding: [u8; 56], // Pad to 64 bytes (cache line size)
}

impl PaddedCounter {
    pub fn new() -> Self {
        PaddedCounter {
            value: 0,
            _padding: [0; 56],
        }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_map_basic() {
        let mut map = FlatMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        assert_eq!(map.get(&"a"), Some(&1));
        assert_eq!(map.get(&"b"), Some(&2));
        assert_eq!(map.get(&"d"), None);
    }

    #[test]
    fn test_flat_map_update() {
        let mut map = FlatMap::new();
        map.insert("key", 1);
        map.insert("key", 2);

        assert_eq!(map.get(&"key"), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_flat_map_iter() {
        let mut map = FlatMap::new();
        map.insert(1, "a");
        map.insert(2, "b");

        let pairs: Vec<_> = map.iter().collect();
        assert_eq!(pairs, vec![(&1, &"a"), (&2, &"b")]);
    }

    #[test]
    fn test_bitset_basic() {
        let mut bits = Bitset::new(128);
        bits.set(0);
        bits.set(63);
        bits.set(64);
        bits.set(127);

        assert!(bits.get(0));
        assert!(bits.get(63));
        assert!(bits.get(64));
        assert!(bits.get(127));
        assert!(!bits.get(1));
        assert_eq!(bits.count_ones(), 4);
    }

    #[test]
    fn test_bitset_iter_set() {
        let mut bits = Bitset::new(100);
        bits.set(5);
        bits.set(10);
        bits.set(99);

        let set: Vec<usize> = bits.iter_set().collect();
        assert_eq!(set, vec![5, 10, 99]);
    }

    #[test]
    fn test_bitset_clear() {
        let mut bits = Bitset::new(64);
        bits.set(32);
        assert!(bits.get(32));
        bits.clear(32);
        assert!(!bits.get(32));
    }

    #[test]
    fn test_soa_basic() {
        let mut soa = SoA::new();
        soa.push(1, "alice".into(), 95.0, true);
        soa.push(2, "bob".into(), 87.5, true);
        soa.push(3, "charlie".into(), 92.0, false);

        assert_eq!(soa.len(), 3);
        assert!((soa.average_score() - 91.5).abs() < 0.01);
    }

    #[test]
    fn test_soa_top_scores() {
        let mut soa = SoA::new();
        soa.push(1, "a".into(), 90.0, true);
        soa.push(2, "b".into(), 95.0, true);
        soa.push(3, "c".into(), 85.0, true);
        soa.push(4, "d".into(), 99.0, true);

        let top = soa.top_scores(2);
        assert_eq!(top[0].0, 4); // id=4, score=99
        assert_eq!(top[1].0, 2); // id=2, score=95
    }

    #[test]
    fn test_aos_average() {
        let mut aos = AoS::new();
        aos.push(Record { id: 1, name: "a".into(), score: 80.0, active: true });
        aos.push(Record { id: 2, name: "b".into(), score: 90.0, active: true });

        assert!((aos.average_score() - 85.0).abs() < 0.01);
    }

    #[test]
    fn test_circular_buffer() {
        let mut buf = CircularBuffer::new(3);

        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.len(), 3);
        assert!(buf.is_full());

        // Overwrites oldest
        let old = buf.push(4);
        assert_eq!(old, Some(1));
        assert_eq!(buf.pop(), Some(2));
        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
        assert!(buf.is_empty());
    }

    #[test]
    fn test_circular_buffer_pop_empty() {
        let mut buf: CircularBuffer<i32> = CircularBuffer::new(5);
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_blocked_matrix() {
        let mut mat = BlockedMatrix::new(8, 8, 4);

        mat.set(0, 0, 1.0);
        mat.set(7, 7, 2.0);
        mat.set(3, 4, 3.0);

        assert!((mat.get(0, 0) - 1.0).abs() < 0.001);
        assert!((mat.get(7, 7) - 2.0).abs() < 0.001);
        assert!((mat.get(3, 4) - 3.0).abs() < 0.001);
        assert!((mat.get(1, 1) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_sorted_vec() {
        let mut sv = SortedVec::new();
        sv.insert(5);
        sv.insert(3);
        sv.insert(1);
        sv.insert(4);
        sv.insert(2);

        assert!(sv.contains(&3));
        assert!(!sv.contains(&6));

        let items: Vec<&i32> = sv.iter().collect();
        assert_eq!(items, vec![&1, &2, &3, &4, &5]);
    }

    #[test]
    fn test_sorted_vec_duplicate() {
        let mut sv = SortedVec::new();
        sv.insert(1);
        sv.insert(1);
        sv.insert(1);

        assert_eq!(sv.len(), 3);
    }

    #[test]
    fn test_padded_counter() {
        let mut counters: Vec<PaddedCounter> = (0..4).map(|_| PaddedCounter::new()).collect();

        for c in &mut counters {
            for _ in 0..100 {
                c.increment();
            }
        }

        for c in &counters {
            assert_eq!(c.value, 100);
        }
    }

    #[test]
    fn test_padded_counter_size() {
        // Should be 64 bytes (one cache line)
        assert_eq!(std::mem::size_of::<PaddedCounter>(), 64);
    }
}
