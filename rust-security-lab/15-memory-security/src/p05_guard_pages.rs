//! # Lesson 05: Guard Pages — Detecting Buffer Overflows
//!
//! ## The Problem
//!
//! Buffer overflows write past the end of a buffer, corrupting adjacent memory.
//! This can overwrite return addresses, function pointers, or other security-critical data.
//!
//! ```text
//! Stack layout (growing down):
//! ┌──────────────────┐
//! │ return address   │ ← attacker overwrites this
//! ├──────────────────┤
//! │ saved frame ptr  │
//! ├──────────────────┤
//! │ buffer[64 bytes] │ ← attacker writes past this
//! └──────────────────┘
//! ```
//!
//! ## Defense: Guard Pages
//!
//! A guard page is a memory page set to `PROT_NONE` (no access). Any read, write,
//! or execute triggers a segfault (SIGSEGV), immediately catching the overflow.
//!
//! ```text
//! ┌──────────────────┐
//! │ buffer[64 bytes] │ ← normal read/write
//! ├──────────────────┤
//! │ GUARD PAGE       │ ← PROT_NONE, any access → SIGSEGV
//! └──────────────────┘
//! ```
//!
//! ## Stack Canaries
//!
//! A "canary" is a random value placed between local variables and the return address.
//! Before returning, the function checks if the canary was modified. If so → abort.
//!
//! ```text
//! ┌──────────────────┐
//! │ return address   │
//! ├──────────────────┤
//! │ canary (random)  │ ← checked before return
//! ├──────────────────┤
//! │ buffer[64]       │
//! └──────────────────┘
//! ```
//!
//! Rust uses stack canaries in debug mode. The LLVM backend inserts them automatically.

use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::ptr;

/// Exercise 1: Implement `Canary` — a simple stack canary for buffer overflow detection.
///
/// Requirements:
/// - Store a random canary value alongside the buffer
/// - On creation, generate a random 8-byte canary
/// - Provide `check(&self) -> bool` that verifies the canary is intact
/// - Provide `buffer(&self) -> &[u8]` and `buffer_mut(&mut self) -> &mut [u8]`
///
/// Hints:
/// - Use `rand::random::<[u8; 8]>()` for the canary
/// - The canary is stored AFTER the buffer in memory
/// - `check()` compares current canary bytes against stored original
pub struct Canary {
    buffer: Vec<u8>,
    original_canary: [u8; 8],
    canary: [u8; 8],
}

impl Canary {
    pub fn new(size: usize) -> Self {
        todo!("Create a canary-protected buffer")
    }

    pub fn buffer(&self) -> &[u8] {
        todo!("Return reference to the protected buffer")
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        todo!("Return mutable reference to the protected buffer")
    }

    /// Check if the canary is intact (no overflow detected).
    pub fn check(&self) -> bool {
        todo!("Verify the canary hasn't been modified")
    }
}

/// Exercise 2: Implement `GuardedBuffer` — a buffer with a guard page.
///
/// Requirements:
/// - Allocate memory using `libc::mmap` with a guard page after the buffer
/// - The guard page should be `PROT_NONE` (no access)
/// - Any overflow into the guard page will cause SIGSEGV
///
/// This is a conceptual exercise — on Linux you'd use mmap with PROT_NONE.
/// For portability, we simulate with a check on access bounds.
///
/// Hints:
/// - Use `alloc_zeroed` for the buffer portion
/// - Store the buffer size
/// - Bounds-check all access in `get` and `set`
pub struct GuardedBuffer {
    ptr: *mut u8,
    size: usize,
}

impl GuardedBuffer {
    pub fn new(size: usize) -> Self {
        todo!("Allocate a guarded buffer with page protection")
    }

    /// Safely read a byte at `index`. Returns None if out of bounds.
    pub fn get(&self, index: usize) -> Option<u8> {
        todo!("Bounds-checked read")
    }

    /// Safely write a byte at `index`. Returns false if out of bounds.
    pub fn set(&mut self, index: usize, value: u8) -> bool {
        todo!("Bounds-checked write")
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for GuardedBuffer {
    fn drop(&mut self) {
        // Deallocate the buffer
        if self.size > 0 {
            unsafe {
                let layout = Layout::from_size_align(self.size, 4096).unwrap();
                dealloc(self.ptr, layout);
            }
        }
    }
}

/// Exercise 3: Implement `detect_overflow` — check if a write would overflow.
///
/// Given a buffer size and a write offset + length, return true if the write
/// would overflow the buffer.
///
/// Hints:
/// - Check if `offset + length > buffer_size`
/// - Watch for integer overflow in the addition
pub fn detect_overflow(buffer_size: usize, offset: usize, length: usize) -> bool {
    todo!("Check if a write would overflow the buffer")
}

/// Exercise 4: Implement `SafeMemcpy` — copy with overflow detection.
///
/// Copy `n` bytes from `src` to `dst`, but only if it won't overflow.
/// Returns Ok(bytes_copied) or Err(message).
///
/// Hints:
/// - Check `n <= dst.len()` and `n <= src.len()`
/// - Use `dst[..n].copy_from_slice(&src[..n])`
pub fn safe_memcpy(dst: &mut [u8], src: &[u8], n: usize) -> Result<usize, String> {
    todo!("Copy with overflow detection")
}

/// Exercise 5: Implement `StackCanary` — a runtime stack canary check.
///
/// Create a canary value at the start of a function, check it at the end.
/// If the canary was modified, return false (overflow detected).
///
/// Hints:
/// - Generate a random 8-byte canary at the start
/// - Store it on the stack
/// - After "doing work" (the closure), check if it's still intact
/// - Return true if canary is intact
pub fn with_stack_canary<F: FnOnce(&mut [u8])>(size: usize, f: F) -> bool {
    todo!("Execute f with a stack canary, return true if canary is intact")
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
        // Corrupt the canary directly (simulating an overflow)
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
            // Normal operation — no overflow
            buf.fill(0x42);
        });
        assert!(intact, "Canary should be intact after normal operation");
    }
}
