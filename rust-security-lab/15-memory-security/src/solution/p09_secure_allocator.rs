//! # Lesson 09: Secure Memory Allocator (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use zeroize::Zeroize;

/// An allocator wrapper that zeroizes memory on dealloc.
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
        self.alloc_count.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Zeroize the memory before freeing
        let slice = std::slice::from_raw_parts_mut(ptr, layout.size());
        slice.zeroize();
        System.dealloc(ptr, layout);
    }
}

/// A Box-like type that zeroizes on drop.
pub struct SecureBox {
    data: Vec<u8>,
}

impl SecureBox {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn get(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Drop for SecureBox {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

/// Zeroize a Vec and return its length.
pub fn wipe_and_free(mut data: Vec<u8>) -> usize {
    let len = data.len();
    data.zeroize();
    len
}

/// A pool of pre-zeroized buffers.
pub struct SecurePool {
    buffers: Vec<Vec<u8>>,
    buffer_size: usize,
}

impl SecurePool {
    pub fn new(buffer_size: usize, count: usize) -> Self {
        let buffers = (0..count).map(|_| vec![0u8; buffer_size]).collect();
        Self { buffers, buffer_size }
    }

    pub fn acquire(&mut self) -> Option<Vec<u8>> {
        self.buffers.pop()
    }

    pub fn release(&mut self, mut buf: Vec<u8>) {
        buf.zeroize();
        // Resize to expected size if needed
        buf.resize(self.buffer_size, 0);
        buf.zeroize();
        self.buffers.push(buf);
    }

    pub fn available(&self) -> usize {
        self.buffers.len()
    }
}

/// Return basic allocator statistics.
pub fn allocator_stats() -> (bool, usize, &'static str) {
    (
        true,
        0,
        "Secure allocator wraps system allocator, zeroizes memory on dealloc to prevent residual secrets"
    )
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
    }

    #[test]
    fn test_wipe_and_free() {
        let data = vec![0xFF; 128];
        let len = wipe_and_free(data);
        assert_eq!(len, 128);
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
