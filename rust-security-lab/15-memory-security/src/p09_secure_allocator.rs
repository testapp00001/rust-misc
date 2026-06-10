//! # Lesson 09: Secure Memory Allocator — Alternatives to malloc
//!
//! ## The Problem
//!
//! Standard allocators (glibc malloc, jemalloc, etc.) do NOT zero memory on free.
//! When you `free()` a buffer, the allocator marks it as available but the bytes
//! remain. A subsequent `malloc()` may return the same memory — leaking the secret.
//!
//! ```text
//! malloc(32) → ptr → [DE AD C0 DE ...]
//! free(ptr)  → ptr still points to [DE AD C0 DE ...]
//! malloc(32) → same ptr → [DE AD C0 DE ...]  ← LEAK!
//! ```
//!
//! ## Defense: Secure Allocators
//!
//! Secure allocators zero memory on free:
//! - **`explicit_bzero`** (BSD): Like `bzero` but guaranteed not to be optimized away
//! - **`SecureZeroMemory`** (Windows): Same purpose
//! - **`OPENSSL_cleanse`** (OpenSSL): Zeros memory with compiler fence
//! - **`sodium_malloc`** (libsodium): mlock'd + guarded + zero-on-free
//!
//! In Rust, the `zeroize` crate provides the `Zeroize` trait that achieves this
//! by using a volatile write + compiler fence.
//!
//! ## Architecture: Our SecureAllocator Wrapper
//!
//! ```text
//! ┌───────────────────────────┐
//! │     SecureAllocator       │
//! │  ┌─────────────────────┐  │
//! │  │ 1. alloc(n)         │  │
//! │  │ 2. zeroize on free  │  │
//! │  │ 3. mlock (optional) │  │
//! │  └─────────────────────┘  │
//! └───────────────────────────┘
//! ```

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use zeroize::Zeroize;

/// Exercise 1: Implement `SecureAlloc` — an allocator wrapper that zeroizes on dealloc.
///
/// Requirements:
/// - Wraps the system allocator
/// - On `alloc`: delegates to system allocator, tracks allocation count
/// - On `dealloc`: zeroizes the memory before delegating to system allocator
///
/// Hints:
/// - Implement `unsafe trait GlobalAlloc`
/// - In `dealloc`, create a `&mut [u8]` from the pointer and size, then zeroize it
/// - Use `std::slice::from_raw_parts_mut(ptr, size)` to create the slice
/// - Call `.zeroize()` on the slice
/// - Then call `System.dealloc(ptr, layout)`
pub struct SecureAlloc {
    alloc_count: AtomicUsize,
}

impl SecureAlloc {
    pub const fn new() -> Self {
        Self {
            alloc_count: AtomicUsize::new(0),
        }
    }

    pub fn allocation_count(&self) -> usize {
        self.alloc_count.load(Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for SecureAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!("Delegate to system allocator and increment counter")
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!("Zeroize memory before freeing")
    }
}

/// Exercise 2: Implement `SecureBox` — a Box-like type that zeroizes on drop.
///
/// Requirements:
/// - Stores data in a `Vec<u8>` (heap-allocated)
/// - On drop, zeroizes the data before freeing
/// - `new(data: Vec<u8>)` takes ownership
/// - `get(&self) -> &[u8]` returns reference
///
/// Hints:
/// - Use `Vec<u8>` internally (it already manages heap memory)
/// - Implement `Drop` to call `self.data.zeroize()`
pub struct SecureBox {
    data: Vec<u8>,
}

impl SecureBox {
    pub fn new(data: Vec<u8>) -> Self {
        todo!("Create a SecureBox")
    }

    pub fn get(&self) -> &[u8] {
        todo!("Return reference to data")
    }

    pub fn len(&self) -> usize {
        todo!("Return length")
    }
}

/// Exercise 3: Implement `wipe_and_free` — zeroize a Vec and then drop it.
///
/// Requirements:
/// - Takes ownership of a `Vec<u8>`
/// - Zeroizes all bytes
/// - Drops the Vec (implicit at end of function)
/// - Returns the original length
///
/// Hints:
/// - Call `data.zeroize()` (Vec<u8> implements Zeroize)
/// - Return `data.len()` before drop
pub fn wipe_and_free(mut data: Vec<u8>) -> usize {
    todo!("Zeroize a Vec and return its length")
}

/// Exercise 4: Implement `SecurePool` — a simple pool of pre-zeroized buffers.
///
/// Requirements:
/// - Maintains a pool of `Vec<u8>` buffers, each of a fixed size
/// - `new(buffer_size, count)` creates `count` zeroed buffers
/// - `acquire(&mut self) -> Option<Vec<u8>>` takes a buffer from the pool
/// - `release(&mut self, mut buf: Vec<u8>)` returns a buffer to the pool (zeroing it first)
///
/// Hints:
/// - Store as `Vec<Vec<u8>>`
/// - `acquire`: pop from the pool
/// - `release`: zeroize the buffer, push back to pool
pub struct SecurePool {
    buffers: Vec<Vec<u8>>,
    buffer_size: usize,
}

impl SecurePool {
    pub fn new(buffer_size: usize, count: usize) -> Self {
        todo!("Create a pool of zeroed buffers")
    }

    pub fn acquire(&mut self) -> Option<Vec<u8>> {
        todo!("Take a buffer from the pool")
    }

    pub fn release(&mut self, mut buf: Vec<u8>) {
        todo!("Zeroize and return buffer to pool")
    }

    pub fn available(&self) -> usize {
        todo!("Return number of available buffers")
    }
}

/// Exercise 5: Implement `allocator_stats` — return basic stats about secure allocation.
///
/// Returns a tuple of:
/// - Whether the secure allocator is being used (hint: we can't easily check)
/// - The current allocation count from the global counter
/// - A description string
///
/// For this exercise, just return hardcoded descriptive values.
pub fn allocator_stats() -> (bool, usize, &'static str) {
    todo!("Return allocator statistics")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_box_basic() {
        let sb = SecureBox::new(vec![0xAA; 32]);
        assert_eq!(sb.len(), 32);
        assert_eq!(sb.get(), &[0xAA; 32]);
    }

    #[test]
    fn test_secure_box_drop_zeroizes() {
        let sb = SecureBox::new(vec![0x42; 64]);
        assert!(sb.get().iter().all(|&b| b == 0x42));
        drop(sb);
        // After drop, memory should be zeroed
    }

    #[test]
    fn test_wipe_and_free() {
        let data = vec![0xFF; 128];
        let len = wipe_and_free(data);
        assert_eq!(len, 128);
        // data is dropped here — already zeroed
    }

    #[test]
    fn test_secure_pool_new() {
        let pool = SecurePool::new(64, 4);
        assert_eq!(pool.available(), 4);
    }

    #[test]
    fn test_secure_pool_acquire_release() {
        let mut pool = SecurePool::new(32, 2);
        assert_eq!(pool.available(), 2);

        let buf = pool.acquire();
        assert!(buf.is_some());
        assert_eq!(pool.available(), 1);

        pool.release(buf.unwrap());
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn test_secure_pool_exhaustion() {
        let mut pool = SecurePool::new(16, 1);
        let _buf1 = pool.acquire().unwrap();
        assert!(pool.acquire().is_none());
    }

    #[test]
    fn test_secure_pool_release_zeroizes() {
        let mut pool = SecurePool::new(16, 1);
        let mut buf = pool.acquire().unwrap();
        buf.fill(0x42);
        pool.release(buf);
        // After release, the buffer in the pool should be zeroed
        let buf = pool.acquire().unwrap();
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_allocator_stats() {
        let (_is_secure, count, desc) = allocator_stats();
        let _ = count;
        assert!(!desc.is_empty());
    }
}
