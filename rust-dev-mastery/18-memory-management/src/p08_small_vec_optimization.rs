//! # Small Vec Optimization
//!
//! SmallVec and similar types store a small number of elements inline (on the
//! stack) and only allocate on the heap when the inline capacity is exceeded.
//! This eliminates heap allocation for the common case of small collections.
//!
//! ## Benefits:
//!
//! - **No heap allocation** for small collections
//! - **Cache-friendly**: Data is on the stack
//! - **Fewer allocations**: Reduces allocator pressure
//! - **Predictable performance**: Common case is always fast
//!
//! ## When to Use:
//!
//! - Known maximum size that's usually small
//! - Many small collections (e.g., per-node child lists)
//! - Hot paths where allocation overhead matters

/// A simple SmallVec implementation that stores N elements inline.
/// Falls back to Vec when the inline capacity is exceeded.
pub struct SmallVec<T, const N: usize> {
    inline: [std::mem::MaybeUninit<T>; N],
    len: usize,
    heap: Option<Vec<T>>,
}

impl<T, const N: usize> SmallVec<T, N> {
    pub fn new() -> Self {
        Self {
            // Safety: MaybeUninit doesn't need initialization
            inline: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
            heap: None,
        }
    }

    pub fn push(&mut self, value: T) {
        if let Some(ref mut heap) = self.heap {
            heap.push(value);
        } else if self.len < N {
            self.inline[self.len] = std::mem::MaybeUninit::new(value);
            self.len += 1;
        } else {
            // Overflow to heap
            let mut heap = Vec::with_capacity(N * 2);
            for i in 0..N {
                unsafe {
                    heap.push(self.inline[i].assume_init_read());
                }
            }
            heap.push(value);
            self.heap = Some(heap);
            // Mark inline as consumed
            self.len = N;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(ref mut heap) = self.heap {
            heap.pop()
        } else if self.len > 0 {
            self.len -= 1;
            unsafe { Some(self.inline[self.len].assume_init_read()) }
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        if let Some(ref heap) = self.heap {
            heap.len()
        } else {
            self.len
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_inline(&self) -> bool {
        self.heap.is_none()
    }

    pub fn capacity(&self) -> usize {
        if let Some(ref heap) = self.heap {
            heap.capacity()
        } else {
            N
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if let Some(ref heap) = self.heap {
            heap.get(index)
        } else if index < self.len {
            unsafe { Some(self.inline[index].assume_init_ref()) }
        } else {
            None
        }
    }

    pub fn iter(&self) -> SmallVecIter<'_, T, N> {
        SmallVecIter {
            vec: self,
            index: 0,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        if let Some(ref heap) = self.heap {
            heap.as_slice()
        } else {
            unsafe {
                std::slice::from_raw_parts(
                    self.inline.as_ptr() as *const T,
                    self.len,
                )
            }
        }
    }
}

impl<T, const N: usize> Drop for SmallVec<T, N> {
    fn drop(&mut self) {
        if self.heap.is_none() {
            for i in 0..self.len {
                unsafe {
                    self.inline[i].assume_init_drop();
                }
            }
        }
    }
}

pub struct SmallVecIter<'a, T, const N: usize> {
    vec: &'a SmallVec<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for SmallVecIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.vec.get(self.index);
        self.index += 1;
        item
    }
}

/// Compact string representation for short strings.
/// Uses inline storage for strings up to N bytes.
pub struct CompactString<const N: usize> {
    inline: [u8; N],
    len: u8,
    heap: Option<String>,
}

impl<const N: usize> CompactString<N> {
    pub fn new(s: &str) -> Self {
        if s.len() <= N && s.len() <= 255 {
            let mut inline = [0u8; N];
            inline[..s.len()].copy_from_slice(s.as_bytes());
            Self {
                inline,
                len: s.len() as u8,
                heap: None,
            }
        } else {
            Self {
                inline: [0u8; N],
                len: 0,
                heap: Some(s.to_string()),
            }
        }
    }

    pub fn as_str(&self) -> &str {
        if let Some(ref heap) = self.heap {
            heap.as_str()
        } else {
            unsafe { std::str::from_utf8_unchecked(&self.inline[..self.len as usize]) }
        }
    }

    pub fn len(&self) -> usize {
        if let Some(ref heap) = self.heap {
            heap.len()
        } else {
            self.len as usize
        }
    }

    pub fn is_inline(&self) -> bool {
        self.heap.is_none()
    }
}

/// Bit-packed boolean vector for space-efficient boolean storage.
pub struct BitVec {
    data: Vec<u64>,
    len: usize,
}

impl BitVec {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(bits: usize) -> Self {
        Self {
            data: Vec::with_capacity((bits + 63) / 64),
            len: 0,
        }
    }

    pub fn push(&mut self, bit: bool) {
        let word_index = self.len / 64;
        let bit_index = self.len % 64;

        while self.data.len() <= word_index {
            self.data.push(0);
        }

        if bit {
            self.data[word_index] |= 1u64 << bit_index;
        }

        self.len += 1;
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.len {
            return None;
        }
        let word_index = index / 64;
        let bit_index = index % 64;
        Some(self.data[word_index] & (1u64 << bit_index) != 0)
    }

    pub fn set(&mut self, index: usize, value: bool) -> Result<(), ()> {
        if index >= self.len {
            return Err(());
        }
        let word_index = index / 64;
        let bit_index = index % 64;

        if value {
            self.data[word_index] |= 1u64 << bit_index;
        } else {
            self.data[word_index] &= !(1u64 << bit_index);
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn count_ones(&self) -> usize {
        self.data.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn memory_bytes(&self) -> usize {
        self.data.len() * 8
    }
}

/// Compact enum representation using repr(u8).
/// Minimizes the size of enum values.
#[repr(u8)]
pub enum CompactVariant {
    Empty,
    Small(u8),
    Medium(u16),
    // Note: With niche optimization, Option<CompactVariant> may be same size
}

/// Packed struct with explicit alignment.
#[repr(C, packed)]
pub struct PackedStruct {
    pub a: u8,
    pub b: u32,
    pub c: u8,
}

/// Interleaved vs. struct-of-arrays for cache optimization.
pub struct ArrayOfStructs {
    pub items: Vec<(f64, f64, f64)>,
}

impl ArrayOfStructs {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, x: f64, y: f64, z: f64) {
        self.items.push((x, y, z));
    }

    pub fn sum_x(&self) -> f64 {
        self.items.iter().map(|(x, _, _)| x).sum()
    }
}

pub struct StructOfArrays {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
}

impl StructOfArrays {
    pub fn new() -> Self {
        Self {
            x: Vec::new(),
            y: Vec::new(),
            z: Vec::new(),
        }
    }

    pub fn push(&mut self, x: f64, y: f64, z: f64) {
        self.x.push(x);
        self.y.push(y);
        self.z.push(z);
    }

    pub fn sum_x(&self) -> f64 {
        self.x.iter().sum()
    }

    pub fn len(&self) -> usize {
        self.x.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_vec_inline() {
        let mut sv = SmallVec::<i32, 4>::new();
        sv.push(1);
        sv.push(2);
        sv.push(3);

        assert_eq!(sv.len(), 3);
        assert!(sv.is_inline());
        assert_eq!(sv.get(0), Some(&1));
        assert_eq!(sv.get(2), Some(&3));
    }

    #[test]
    fn test_small_vec_overflow() {
        let mut sv = SmallVec::<i32, 2>::new();
        sv.push(1);
        sv.push(2);
        assert!(sv.is_inline());

        sv.push(3); // Triggers heap allocation
        assert!(!sv.is_inline());
        assert_eq!(sv.len(), 3);
        assert_eq!(sv.get(0), Some(&1));
        assert_eq!(sv.get(2), Some(&3));
    }

    #[test]
    fn test_small_vec_pop() {
        let mut sv = SmallVec::<i32, 4>::new();
        sv.push(10);
        sv.push(20);

        assert_eq!(sv.pop(), Some(20));
        assert_eq!(sv.pop(), Some(10));
        assert_eq!(sv.pop(), None);
    }

    #[test]
    fn test_small_vec_iter() {
        let mut sv = SmallVec::<i32, 4>::new();
        sv.push(1);
        sv.push(2);
        sv.push(3);

        let collected: Vec<&i32> = sv.iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }

    #[test]
    fn test_small_vec_as_slice() {
        let mut sv = SmallVec::<i32, 4>::new();
        sv.push(10);
        sv.push(20);
        assert_eq!(sv.as_slice(), &[10, 20]);
    }

    #[test]
    fn test_compact_string_inline() {
        let cs = CompactString::<16>::new("hello");
        assert_eq!(cs.as_str(), "hello");
        assert!(cs.is_inline());
        assert_eq!(cs.len(), 5);
    }

    #[test]
    fn test_compact_string_heap() {
        let long = "this is definitely longer than 16 bytes for sure";
        let cs = CompactString::<16>::new(long);
        assert_eq!(cs.as_str(), long);
        assert!(!cs.is_inline());
    }

    #[test]
    fn test_bit_vec_basic() {
        let mut bv = BitVec::new();
        bv.push(true);
        bv.push(false);
        bv.push(true);
        bv.push(true);

        assert_eq!(bv.len(), 4);
        assert_eq!(bv.get(0), Some(true));
        assert_eq!(bv.get(1), Some(false));
        assert_eq!(bv.get(2), Some(true));
        assert_eq!(bv.count_ones(), 3);
    }

    #[test]
    fn test_bit_vec_set() {
        let mut bv = BitVec::new();
        bv.push(false);
        bv.push(false);

        bv.set(0, true).unwrap();
        assert_eq!(bv.get(0), Some(true));

        bv.set(1, true).unwrap();
        assert_eq!(bv.count_ones(), 2);
    }

    #[test]
    fn test_bit_vec_out_of_bounds() {
        let bv = BitVec::new();
        assert_eq!(bv.get(0), None);
    }

    #[test]
    fn test_bit_vec_memory() {
        let mut bv = BitVec::new();
        for _ in 0..64 {
            bv.push(true);
        }
        assert_eq!(bv.memory_bytes(), 8); // 64 bits = 8 bytes
    }

    #[test]
    fn test_bit_vec_large() {
        let mut bv = BitVec::new();
        for i in 0..1000 {
            bv.push(i % 2 == 0);
        }
        assert_eq!(bv.len(), 1000);
        assert_eq!(bv.count_ones(), 500);
    }

    #[test]
    fn test_array_of_structs() {
        let mut aos = ArrayOfStructs::new();
        aos.push(1.0, 2.0, 3.0);
        aos.push(4.0, 5.0, 6.0);
        assert!((aos.sum_x() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_struct_of_arrays() {
        let mut soa = StructOfArrays::new();
        soa.push(1.0, 2.0, 3.0);
        soa.push(4.0, 5.0, 6.0);
        assert!((soa.sum_x() - 5.0).abs() < f64::EPSILON);
        assert_eq!(soa.len(), 2);
    }

    #[test]
    fn test_small_vec_capacity() {
        let sv = SmallVec::<i32, 8>::new();
        assert_eq!(sv.capacity(), 8);
    }

    #[test]
    fn test_small_vec_empty() {
        let mut sv = SmallVec::<i32, 4>::new();
        assert!(sv.is_empty());
        assert_eq!(sv.len(), 0);
        assert_eq!(sv.pop(), None);
    }

    #[test]
    fn test_compact_string_empty() {
        let cs = CompactString::<8>::new("");
        assert_eq!(cs.as_str(), "");
        assert_eq!(cs.len(), 0);
        assert!(cs.is_inline());
    }
}
