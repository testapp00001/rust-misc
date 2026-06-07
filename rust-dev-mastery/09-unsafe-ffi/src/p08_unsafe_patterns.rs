//! # Common Unsafe Patterns
//!
//! This lesson covers the most common patterns you'll encounter when
//! working with unsafe Rust: transmute, unions, ManuallyDrop, and MaybeUninit.
//!
//! Each pattern has legitimate uses but also subtle pitfalls. Understanding
//! when and how to use them correctly is essential for systems programming.

use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ptr;

/// Demonstrates `std::mem::transmute` for type reinterpretation.
/// Transmute reinterprets the bits of one type as another type.
/// Both types must have the same size.
pub fn f32_to_bits(f: f32) -> u32 {
    // SAFETY: f32 and u32 are both 4 bytes. Every bit pattern of f32
    // is a valid u32 and vice versa.
    unsafe { mem::transmute(f) }
}

pub fn bits_to_f32(bits: u32) -> f32 {
    // SAFETY: Same as above - both types are 4 bytes.
    unsafe { mem::transmute(bits) }
}

/// Demonstrates safe enum discriminant extraction via transmute.
/// This is a common pattern for C-compatible enums.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3,
}

impl Color {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Color::Red),
            1 => Some(Color::Green),
            2 => Some(Color::Blue),
            3 => Some(Color::Yellow),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        // SAFETY: Color is #[repr(u8)] with values 0-3, so transmuting
        // back to u8 is always valid.
        unsafe { mem::transmute(self) }
    }
}

/// Demonstrates unions for type punning.
/// A union is like a struct where all fields overlap in memory.
/// Only one field should be active at a time, and reading an inactive
/// field is UB unless both fields have compatible bit patterns.
#[repr(C)]
pub union FloatBits {
    pub f: f32,
    pub bits: u32,
}

impl FloatBits {
    pub fn from_float(f: f32) -> Self {
        FloatBits { f }
    }

    pub fn get_bits(&self) -> u32 {
        // SAFETY: We initialized the union with `f`, but reading `bits`
        // is valid because f32 and u32 have the same size and every
        // bit pattern is valid for both types.
        unsafe { self.bits }
    }

    pub fn get_sign(&self) -> bool {
        (self.get_bits() >> 31) != 0
    }

    pub fn get_exponent(&self) -> u8 {
        ((self.get_bits() >> 23) & 0xFF) as u8
    }

    pub fn get_mantissa(&self) -> u32 {
        self.get_bits() & 0x7FFFFF
    }
}

/// Demonstrates ManuallyDrop for controlling when values are dropped.
/// ManuallyDrop prevents the destructor from running automatically,
/// giving you manual control over cleanup.
pub struct ManualBuffer {
    data: ManuallyDrop<Vec<u8>>,
    header: u64,
}

impl ManualBuffer {
    pub fn new(header: u64) -> Self {
        ManualBuffer {
            data: ManuallyDrop::new(Vec::with_capacity(1024)),
            header,
        }
    }

    pub fn push(&mut self, byte: u8) {
        self.data.push(byte);
    }

    pub fn header(&self) -> u64 {
        self.header
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Consume the buffer and return the data, skipping the header.
    /// ManuallyDrop allows us to move the Vec out without dropping it.
    pub fn into_data(mut self) -> Vec<u8> {
        // SAFETY: We take the Vec out of ManuallyDrop. After this, `self`
        // contains an uninitialized ManuallyDrop, but we won't access it
        // because we're consuming self.
        let data = unsafe { ManuallyDrop::take(&mut self.data) };
        // Prevent self's destructor from running (which would try to drop
        // the now-taken Vec).
        mem::forget(self);
        data
    }
}

/// Demonstrates MaybeUninit for deferred initialization.
/// MaybeUninit<T> tells the compiler "this memory exists but is not yet
/// a valid T." This is essential for:
/// - Allocating arrays of uninitialized memory
/// - Working with C APIs that fill buffers
/// - Performance-critical code that avoids unnecessary zeroing
pub struct UninitArray<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> UninitArray<T, N> {
    pub fn new() -> Self {
        // SAFETY: An array of MaybeUninit does not require initialization.
        // MaybeUninit<T> is valid in any bit state.
        UninitArray {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= N {
            return Err(value);
        }
        self.data[self.len] = MaybeUninit::new(value);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: self.len was > 0, so self.len (after decrement) is a valid
        // initialized index. assume_init_read reads the value without
        // invalidating the MaybeUninit (we'll overwrite it on next push).
        Some(unsafe { self.data[self.len].assume_init_read() })
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: index < len, so this slot was initialized.
            Some(unsafe { self.data[index].assume_init_ref() })
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: Elements 0..len are all initialized.
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T, const N: usize> Drop for UninitArray<T, N> {
    fn drop(&mut self) {
        // Drop all initialized elements
        for i in 0..self.len {
            // SAFETY: Elements 0..len were initialized by push.
            unsafe {
                self.data[i].assume_init_drop();
            }
        }
    }
}

/// Demonstrates `ptr::swap_nonoverlapping` for efficient element swapping.
/// This is useful for sort implementations and data structure manipulation.
pub fn reverse_in_place<T>(slice: &mut [T]) {
    let len = slice.len();
    if len < 2 {
        return;
    }
    let ptr = slice.as_mut_ptr();
    for i in 0..len / 2 {
        // SAFETY: i < len/2, so i and len-1-i are distinct valid indices.
        // The pointers don't overlap because they're at different indices.
        unsafe {
            ptr::swap(ptr.add(i), ptr.add(len - 1 - i));
        }
    }
}

/// Demonstrates `std::mem::swap` and `std::mem::replace` patterns.
/// These are safe but rely on unsafe internally.
pub struct CircularBuffer<T> {
    data: Vec<MaybeUninit<T>>,
    head: usize,
    tail: usize,
    len: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        let mut data = Vec::with_capacity(capacity);
        // Initialize with MaybeUninit values
        for _ in 0..capacity {
            data.push(MaybeUninit::uninit());
        }
        CircularBuffer {
            data,
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= self.data.capacity() {
            return Err(value);
        }
        self.data[self.tail] = MaybeUninit::new(value);
        self.tail = (self.tail + 1) % self.data.capacity();
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: len > 0 means head points to an initialized value.
        let value = unsafe { self.data[self.head].assume_init_read() };
        self.head = (self.head + 1) % self.data.capacity();
        self.len -= 1;
        Some(value)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T> Drop for CircularBuffer<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

/// Demonstrates `std::mem::size_of` and `std::mem::align_of` for
/// computing layout requirements at runtime.
pub fn compute_layout<T>(count: usize) -> (usize, usize, usize) {
    let size = mem::size_of::<T>();
    let align = mem::align_of::<T>();
    let total = (size * count + align - 1) & !(align - 1); // Aligned total
    (size, align, total)
}

/// Demonstrates the "tagged union" pattern using Rust enums.
/// This is how Rust enums work internally -- they're tagged unions
/// where the discriminant tells you which variant is active.
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Str(_) => "str",
        }
    }

    /// Demonstrates that you can safely transmute an enum to its discriminant.
    /// For C-compatible enums only.
    pub fn discriminant(&self) -> u8 {
        // SAFETY: We read the discriminant byte of the enum.
        // For a fieldless enum with #[repr(u8)], this would be the tag.
        // For enums with data, we use mem::discriminant (safe API).
        // This is a simplified example.
        unsafe { *(self as *const Self as *const u8) }
    }
}

/// Demonstrates `mem::zeroed()` for zero-initialization.
/// This is safe for types where all-zeros is a valid representation,
/// but unsound for types like bool or NonNull.
pub fn zeroed_array<T: Copy + Default>(len: usize) -> Vec<T> {
    let mut v = Vec::with_capacity(len);
    // SAFETY: We're using MaybeUninit to avoid the requirement that T: Default.
    // For this helper, we require T: Default so we can initialize properly.
    v.resize_with(len, T::default);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transmute_f32_u32() {
        let pi: f32 = 3.14;
        let bits = f32_to_bits(pi);
        let back = bits_to_f32(bits);
        assert!((back - pi).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transmute_special_values() {
        let zero_bits = f32_to_bits(0.0);
        assert_eq!(zero_bits, 0);

        let neg_zero_bits = f32_to_bits(-0.0);
        assert_eq!(neg_zero_bits, 1 << 31);

        let inf_bits = f32_to_bits(f32::INFINITY);
        assert_eq!(inf_bits, 0x7F800000);
    }

    #[test]
    fn test_color_transmute() {
        assert_eq!(Color::Red.to_u8(), 0);
        assert_eq!(Color::Green.to_u8(), 1);
        assert_eq!(Color::Blue.to_u8(), 2);
        assert_eq!(Color::Yellow.to_u8(), 3);

        assert_eq!(Color::from_u8(0), Some(Color::Red));
        assert_eq!(Color::from_u8(3), Some(Color::Yellow));
        assert_eq!(Color::from_u8(42), None);
    }

    #[test]
    fn test_float_bits_union() {
        let fb = FloatBits::from_float(1.0);
        assert_eq!(fb.get_bits(), 0x3F800000);
        assert!(!fb.get_sign());
        assert_eq!(fb.get_exponent(), 127);
        assert_eq!(fb.get_mantissa(), 0);

        let fb_neg = FloatBits::from_float(-1.0);
        assert!(fb_neg.get_sign());
    }

    #[test]
    fn test_float_bits_special() {
        let fb_zero = FloatBits::from_float(0.0);
        assert_eq!(fb_zero.get_bits(), 0);

        let fb_inf = FloatBits::from_float(f32::INFINITY);
        assert_eq!(fb_inf.get_exponent(), 255);
        assert_eq!(fb_inf.get_mantissa(), 0);
    }

    #[test]
    fn test_manual_buffer() {
        let mut buf = ManualBuffer::new(0xDEADBEEF);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.header(), 0xDEADBEEF);
        assert_eq!(buf.data(), &[1, 2, 3]);

        let data = buf.into_data();
        assert_eq!(data, vec![1, 2, 3]);
    }

    #[test]
    fn test_uninit_array() {
        let mut arr = UninitArray::<i32, 8>::new();
        assert!(arr.is_empty());

        for i in 0..8 {
            arr.push(i * 10).unwrap();
        }
        assert_eq!(arr.len(), 8);
        assert!(arr.push(99).is_err()); // Full

        assert_eq!(arr.as_slice(), &[0, 10, 20, 30, 40, 50, 60, 70]);

        for i in (0..8).rev() {
            assert_eq!(arr.pop(), Some(i * 10));
        }
        assert_eq!(arr.pop(), None);
    }

    #[test]
    fn test_uninit_array_get() {
        let mut arr = UninitArray::<String, 4>::new();
        arr.push("hello".to_string()).unwrap();
        arr.push("world".to_string()).unwrap();

        assert_eq!(arr.get(0), Some(&"hello".to_string()));
        assert_eq!(arr.get(1), Some(&"world".to_string()));
        assert_eq!(arr.get(2), None);
    }

    #[test]
    fn test_reverse_in_place() {
        let mut data = [1, 2, 3, 4, 5];
        reverse_in_place(&mut data);
        assert_eq!(data, [5, 4, 3, 2, 1]);

        let mut single = [42];
        reverse_in_place(&mut single);
        assert_eq!(single, [42]);

        let mut empty: [i32; 0] = [];
        reverse_in_place(&mut empty);
        assert_eq!(empty, []);
    }

    #[test]
    fn test_circular_buffer() {
        let mut cb = CircularBuffer::new(4);
        assert!(cb.is_empty());

        cb.push(1).unwrap();
        cb.push(2).unwrap();
        cb.push(3).unwrap();
        assert_eq!(cb.len(), 3);

        assert_eq!(cb.pop(), Some(1));
        assert_eq!(cb.pop(), Some(2));

        // Can push more after popping
        cb.push(4).unwrap();
        cb.push(5).unwrap();
        cb.push(6).unwrap();
        assert_eq!(cb.len(), 4); // Full

        assert!(cb.push(7).is_err()); // Can't push when full

        assert_eq!(cb.pop(), Some(3));
        assert_eq!(cb.pop(), Some(4));
        assert_eq!(cb.pop(), Some(5));
        assert_eq!(cb.pop(), Some(6));
        assert_eq!(cb.pop(), None);
    }

    #[test]
    fn test_compute_layout() {
        let (size, align, total) = compute_layout::<u64>(10);
        assert_eq!(size, 8);
        assert_eq!(align, 8);
        assert_eq!(total, 80);

        let (size, align, total) = compute_layout::<u8>(3);
        assert_eq!(size, 1);
        assert_eq!(align, 1);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_zeroed_array() {
        let arr = zeroed_array::<u64>(5);
        assert_eq!(arr, vec![0, 0, 0, 0, 0]);
    }
}
