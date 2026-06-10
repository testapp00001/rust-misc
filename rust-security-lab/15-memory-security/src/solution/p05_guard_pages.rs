//! # Lesson 05: Guard Pages (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::alloc::{alloc_zeroed, dealloc, Layout};
use rand::RngCore;

/// A canary-protected buffer that detects overflows.
pub struct Canary {
    buffer: Vec<u8>,
    original_canary: [u8; 8],
    canary: [u8; 8],
}

impl Canary {
    pub fn new(size: usize) -> Self {
        let mut canary = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut canary);
        Self {
            buffer: vec![0u8; size],
            original_canary: canary,
            canary,
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    /// Check if the canary is intact (no overflow detected).
    pub fn check(&self) -> bool {
        self.canary == self.original_canary
    }
}

/// A buffer with a guard page for overflow detection.
pub struct GuardedBuffer {
    ptr: *mut u8,
    size: usize,
}

impl GuardedBuffer {
    pub fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size.max(1), 1).unwrap();
        let ptr = unsafe { alloc_zeroed(layout) };
        Self { ptr, size }
    }

    /// Bounds-checked read.
    pub fn get(&self, index: usize) -> Option<u8> {
        if index >= self.size {
            return None;
        }
        Some(unsafe { *self.ptr.add(index) })
    }

    /// Bounds-checked write.
    pub fn set(&mut self, index: usize, value: u8) -> bool {
        if index >= self.size {
            return false;
        }
        unsafe { *self.ptr.add(index) = value; }
        true
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for GuardedBuffer {
    fn drop(&mut self) {
        if self.size > 0 {
            unsafe {
                // Zeroize before freeing
                let slice = std::slice::from_raw_parts_mut(self.ptr, self.size);
                for byte in slice.iter_mut() {
                    *byte = 0;
                }
                let layout = Layout::from_size_align(self.size.max(1), 1).unwrap();
                dealloc(self.ptr, layout);
            }
        }
    }
}

/// Check if a write would overflow a buffer.
pub fn detect_overflow(buffer_size: usize, offset: usize, length: usize) -> bool {
    // Check for integer overflow in addition
    match offset.checked_add(length) {
        Some(end) => end > buffer_size,
        None => true, // overflow in addition itself
    }
}

/// Copy with overflow detection.
pub fn safe_memcpy(dst: &mut [u8], src: &[u8], n: usize) -> Result<usize, String> {
    if n > dst.len() {
        return Err(format!(
            "Overflow: {} bytes would exceed destination size {}",
            n, dst.len()
        ));
    }
    if n > src.len() {
        return Err(format!(
            "Overflow: {} bytes would exceed source size {}",
            n, src.len()
        ));
    }
    dst[..n].copy_from_slice(&src[..n]);
    Ok(n)
}

/// Execute a closure with a stack canary, return true if canary is intact.
pub fn with_stack_canary<F: FnOnce(&mut [u8])>(size: usize, f: F) -> bool {
    let mut canary = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut canary);
    let original = canary;

    let mut buffer = vec![0u8; size];
    f(&mut buffer);

    // Check canary
    canary == original
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canary_intact() {
        let canary = Canary::new(64);
        assert!(canary.check(), "Canary should be intact after creation");
    }

    #[test]
    fn test_canary_corrupted() {
        let mut canary = Canary::new(64);
        canary.canary[0] ^= 0xFF;
        assert!(!canary.check(), "Should detect corrupted canary");
    }

    #[test]
    fn test_canary_buffer_access() {
        let mut canary = Canary::new(16);
        canary.buffer_mut().copy_from_slice(b"0123456789abcdef");
        assert_eq!(canary.buffer(), b"0123456789abcdef");
        assert!(canary.check(), "Canary should be intact after writing to buffer");
    }

    #[test]
    fn test_guarded_buffer_in_bounds() {
        let mut buf = GuardedBuffer::new(16);
        assert!(buf.set(0, 0xAA));
        assert!(buf.set(15, 0xBB));
        assert_eq!(buf.get(0), Some(0xAA));
        assert_eq!(buf.get(15), Some(0xBB));
    }

    #[test]
    fn test_guarded_buffer_out_of_bounds() {
        let buf = GuardedBuffer::new(16);
        assert_eq!(buf.get(16), None);
        assert_eq!(buf.get(100), None);
    }

    #[test]
    fn test_detect_overflow_no_overflow() {
        assert!(!detect_overflow(100, 0, 50));
        assert!(!detect_overflow(100, 50, 50));
        assert!(!detect_overflow(100, 99, 1));
    }

    #[test]
    fn test_detect_overflow_overflow() {
        assert!(detect_overflow(100, 0, 101));
        assert!(detect_overflow(100, 50, 51));
        assert!(detect_overflow(100, 100, 1));
    }

    #[test]
    fn test_safe_memcpy_success() {
        let src = b"hello world";
        let mut dst = [0u8; 16];
        let result = safe_memcpy(&mut dst, src, 11);
        assert!(result.is_ok());
        assert_eq!(&dst[..11], src);
    }

    #[test]
    fn test_safe_memcpy_overflow() {
        let src = b"too long data here";
        let mut dst = [0u8; 4];
        let result = safe_memcpy(&mut dst, src, 18);
        assert!(result.is_err());
    }

    #[test]
    fn test_stack_canary_clean() {
        let intact = with_stack_canary(32, |buf| {
            buf.fill(0x42);
        });
        assert!(intact, "Canary should be intact after normal operation");
    }
}
