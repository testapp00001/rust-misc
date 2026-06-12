//! Sparse offset index for fast message lookups within segments.
//!
//! The index maps message offsets to their byte positions within a segment
//! file, enabling O(log N) lookups via binary search on a `BTreeMap`. It
//! also supports range queries for batch reads.
//!
//! In production, the index is persisted alongside the segment file and
//! rebuilt on startup from the segment's sparse index entries.

use std::collections::BTreeMap;

/// An index entry mapping a message offset to its byte position within a
/// segment file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexEntry {
    /// Byte position of the message (start of the length-prefixed record).
    pub byte_position: u64,
    /// Serialized size of the record in bytes (including length prefix).
    pub size: u32,
}

/// A sparse offset index for a single segment.
///
/// Entries are inserted as messages are appended. The index is "sparse" in
/// that not every offset needs an entry, but in practice we index every
/// message for O(log N) random access.
pub struct OffsetIndex {
    /// B-tree of offset -> index entry, enabling O(log N) lookups and
    /// efficient range queries.
    entries: BTreeMap<u64, IndexEntry>,
}

impl OffsetIndex {
    /// Create an empty index.
    pub fn new() -> Self {
        OffsetIndex {
            entries: BTreeMap::new(),
        }
    }

    /// Create an index with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut entries = BTreeMap::new();
        // BTreeMap doesn't have with_capacity; we pre-fill then clear to
        // hint the allocator.
        for i in 0..capacity as u64 {
            entries.insert(i, IndexEntry { byte_position: 0, size: 0 });
        }
        entries.clear();
        OffsetIndex { entries }
    }

    /// Insert or update an index entry for the given offset.
    pub fn add(&mut self, offset: u64, byte_position: u64, size: u32) {
        self.entries.insert(offset, IndexEntry { byte_position, size });
    }

    /// Look up the index entry for an exact offset.
    ///
    /// Returns `(byte_position, size)` if found, or `None` if the offset
    /// is not in the index.
    pub fn lookup(&self, offset: u64) -> Option<(u64, u32)> {
        self.entries
            .get(&offset)
            .map(|e| (e.byte_position, e.size))
    }

    /// Find the byte position for an offset by falling back to the nearest
    /// preceding entry.
    ///
    /// This is the primary lookup method for reading: if we don't have an
    /// exact entry for the requested offset, we find the closest one before
    /// it and scan forward from that position.
    pub fn lookup_fuzzy(&self, offset: u64) -> Option<(u64, u64, u32)> {
        // Find the entry with the largest offset <= the requested offset.
        // Returns (index_offset, byte_position, size).
        self.entries
            .range(..=offset)
            .next_back()
            .map(|(&idx_offset, entry)| (idx_offset, entry.byte_position, entry.size))
    }

    /// Return the entry with the largest offset <= `offset`.
    ///
    /// This is the floor operation: find the greatest key that is <= offset.
    pub fn floor(&self, offset: u64) -> Option<u64> {
        self.entries.range(..=offset).next_back().map(|(&k, _)| k)
    }

    /// Return the entry with the smallest offset >= `offset`.
    ///
    /// This is the ceiling operation: find the smallest key that is >= offset.
    pub fn ceiling(&self, offset: u64) -> Option<u64> {
        self.entries.range(offset..).next().map(|(&k, _)| k)
    }

    /// Remove all entries with offsets in the range [from, to).
    pub fn remove_range(&mut self, from: u64, to: u64) {
        let keys: Vec<u64> = self.entries
            .range(from..to)
            .map(|(&k, _)| k)
            .collect();
        for key in keys {
            self.entries.remove(&key);
        }
    }

    /// Remove all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of entries in the index.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The smallest offset in the index.
    pub fn min_offset(&self) -> Option<u64> {
        self.entries.keys().next().copied()
    }

    /// The largest offset in the index.
    pub fn max_offset(&self) -> Option<u64> {
        self.entries.keys().next_back().copied()
    }

    /// Return all offsets in the index, in sorted order.
    pub fn offsets(&self) -> Vec<u64> {
        self.entries.keys().copied().collect()
    }

    /// Iterate over all entries in offset order.
    pub fn iter(&self) -> impl Iterator<Item = (u64, &IndexEntry)> {
        self.entries.iter().map(|(&k, v)| (k, v))
    }

    /// Return a range of index entries in [from, to].
    pub fn range(&self, from: u64, to: u64) -> Vec<(u64, IndexEntry)> {
        self.entries
            .range(from..=to)
            .map(|(&k, v)| (k, *v))
            .collect()
    }
}

impl Default for OffsetIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// A multi-segment index that manages offset lookups across all segments
/// of a partition.
pub struct PartitionIndex {
    /// Per-segment indexes, keyed by segment base offset.
    segment_indexes: BTreeMap<u64, OffsetIndex>,
}

impl PartitionIndex {
    /// Create an empty partition index.
    pub fn new() -> Self {
        PartitionIndex {
            segment_indexes: BTreeMap::new(),
        }
    }

    /// Register a new segment's index.
    pub fn add_segment(&mut self, base_offset: u64, index: OffsetIndex) {
        self.segment_indexes.insert(base_offset, index);
    }

    /// Look up the byte position for an offset across all segments.
    ///
    /// Returns `(segment_base_offset, byte_position, size)` or `None`.
    pub fn lookup(&self, offset: u64) -> Option<(u64, u64, u32)> {
        // Find the segment whose base offset is <= the target offset.
        // The segment with the largest base_offset <= offset is the one
        // that contains it.
        let (&base_offset, segment_index) = self
            .segment_indexes
            .range(..=offset)
            .next_back()?;

        let (_, byte_position, size) = segment_index.lookup_fuzzy(offset)?;
        Some((base_offset, byte_position, size))
    }

    /// Number of segments indexed.
    pub fn segment_count(&self) -> usize {
        self.segment_indexes.len()
    }

    /// Total number of index entries across all segments.
    pub fn total_entries(&self) -> usize {
        self.segment_indexes.values().map(|idx| idx.len()).sum()
    }
}

impl Default for PartitionIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- OffsetIndex tests ---

    #[test]
    fn basic_insert_and_lookup() {
        let mut idx = OffsetIndex::new();
        idx.add(0, 0, 128);
        idx.add(1, 128, 64);
        idx.add(2, 192, 96);

        let (pos, size) = idx.lookup(1).unwrap();
        assert_eq!(pos, 128);
        assert_eq!(size, 64);
    }

    #[test]
    fn lookup_missing_offset() {
        let mut idx = OffsetIndex::new();
        idx.add(0, 0, 100);
        assert!(idx.lookup(5).is_none());
    }

    #[test]
    fn fuzzy_lookup_exact_match() {
        let mut idx = OffsetIndex::new();
        idx.add(0, 0, 100);
        idx.add(10, 100, 200);

        let (offset, pos, size) = idx.lookup_fuzzy(10).unwrap();
        assert_eq!(offset, 10);
        assert_eq!(pos, 100);
        assert_eq!(size, 200);
    }

    #[test]
    fn fuzzy_lookup_falls_back() {
        let mut idx = OffsetIndex::new();
        idx.add(0, 0, 100);
        idx.add(10, 100, 200);

        // Offset 15 is not in the index, but offset 10 is the closest
        // preceding entry.
        let (offset, pos, size) = idx.lookup_fuzzy(15).unwrap();
        assert_eq!(offset, 10);
        assert_eq!(pos, 100);
        assert_eq!(size, 200);
    }

    #[test]
    fn floor_and_ceiling() {
        let mut idx = OffsetIndex::new();
        idx.add(5, 0, 10);
        idx.add(10, 10, 20);
        idx.add(15, 30, 30);
        idx.add(20, 60, 40);

        assert_eq!(idx.floor(12), Some(10));
        assert_eq!(idx.floor(5), Some(5));
        assert_eq!(idx.floor(3), None);

        assert_eq!(idx.ceiling(12), Some(15));
        assert_eq!(idx.ceiling(15), Some(15));
        assert_eq!(idx.ceiling(25), None);
    }

    #[test]
    fn min_max_offset() {
        let mut idx = OffsetIndex::new();
        assert!(idx.min_offset().is_none());
        assert!(idx.max_offset().is_none());

        idx.add(5, 0, 10);
        idx.add(10, 10, 20);
        idx.add(3, 0, 10);

        assert_eq!(idx.min_offset(), Some(3));
        assert_eq!(idx.max_offset(), Some(10));
    }

    #[test]
    fn remove_range() {
        let mut idx = OffsetIndex::new();
        for i in 0..20 {
            idx.add(i, i * 100, 50);
        }
        assert_eq!(idx.len(), 20);

        idx.remove_range(5, 15);
        assert_eq!(idx.len(), 10);
        assert!(idx.lookup(4).is_some());
        assert!(idx.lookup(5).is_none());
        assert!(idx.lookup(14).is_none());
        assert!(idx.lookup(15).is_some());
    }

    #[test]
    fn range_query() {
        let mut idx = OffsetIndex::new();
        for i in 0..10 {
            idx.add(i, i * 100, 50);
        }

        let entries = idx.range(3, 7);
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].0, 3);
        assert_eq!(entries[4].0, 7);
    }

    #[test]
    fn offsets_sorted() {
        let mut idx = OffsetIndex::new();
        idx.add(20, 0, 10);
        idx.add(5, 0, 10);
        idx.add(100, 0, 10);
        idx.add(1, 0, 10);

        let offsets = idx.offsets();
        assert_eq!(offsets, vec![1, 5, 20, 100]);
    }

    #[test]
    fn empty_index() {
        let idx = OffsetIndex::new();
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
        assert!(idx.lookup(0).is_none());
        assert!(idx.floor(0).is_none());
        assert!(idx.ceiling(0).is_none());
    }

    #[test]
    fn default_is_empty() {
        let idx = OffsetIndex::default();
        assert!(idx.is_empty());
    }

    #[test]
    fn clear_empties_index() {
        let mut idx = OffsetIndex::new();
        idx.add(0, 0, 10);
        idx.add(1, 10, 20);
        idx.clear();
        assert!(idx.is_empty());
    }

    #[test]
    fn iter_yields_sorted_entries() {
        let mut idx = OffsetIndex::new();
        idx.add(3, 30, 3);
        idx.add(1, 10, 1);
        idx.add(2, 20, 2);

        let collected: Vec<_> = idx.iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0].0, 1);
        assert_eq!(collected[1].0, 2);
        assert_eq!(collected[2].0, 3);
    }

    #[test]
    fn large_index_performance() {
        let mut idx = OffsetIndex::with_capacity(100_000);
        for i in 0..100_000u64 {
            idx.add(i, i * 200, 200);
        }
        assert_eq!(idx.len(), 100_000);

        // Lookup should be O(log N)
        let (_, pos, size) = idx.lookup_fuzzy(99_999).unwrap();
        assert_eq!(pos, 99_999 * 200);
        assert_eq!(size, 200);

        // Floor/ceiling
        assert_eq!(idx.floor(50_000), Some(50_000));
        assert_eq!(idx.ceiling(50_000), Some(50_000));
    }

    // --- PartitionIndex tests ---

    #[test]
    fn partition_index_cross_segment_lookup() {
        let mut pidx = PartitionIndex::new();

        let mut seg0 = OffsetIndex::new();
        seg0.add(0, 0, 100);
        seg0.add(1, 100, 100);
        pidx.add_segment(0, seg0);

        let mut seg1 = OffsetIndex::new();
        seg1.add(2, 0, 100);
        seg1.add(3, 100, 100);
        pidx.add_segment(2, seg1);

        // Lookup in first segment
        let (base, pos, size) = pidx.lookup(1).unwrap();
        assert_eq!(base, 0);
        assert_eq!(pos, 100);
        assert_eq!(size, 100);

        // Lookup in second segment
        let (base, pos, size) = pidx.lookup(3).unwrap();
        assert_eq!(base, 2);
        assert_eq!(pos, 100);
        assert_eq!(size, 100);
    }

    #[test]
    fn partition_index_counts() {
        let mut pidx = PartitionIndex::new();
        assert_eq!(pidx.segment_count(), 0);
        assert_eq!(pidx.total_entries(), 0);

        let mut idx1 = OffsetIndex::new();
        for i in 0..10 {
            idx1.add(i, i * 100, 50);
        }
        pidx.add_segment(0, idx1);

        let mut idx2 = OffsetIndex::new();
        for i in 10..20 {
            idx2.add(i, (i - 10) * 100, 50);
        }
        pidx.add_segment(10, idx2);

        assert_eq!(pidx.segment_count(), 2);
        assert_eq!(pidx.total_entries(), 20);
    }

    #[test]
    fn partition_index_lookup_before_first_segment() {
        let mut pidx = PartitionIndex::new();
        let mut idx = OffsetIndex::new();
        idx.add(10, 0, 50);
        pidx.add_segment(10, idx);

        // Offset 5 is before any segment's base offset
        assert!(pidx.lookup(5).is_none());
    }
}
