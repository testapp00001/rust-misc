//! # Lesson 04: mlock / mprotect — Prevent Secrets from Being Swapped to Disk
//!
//! ## The Problem
//!
//! The operating system manages memory in "pages" (typically 4KB). When physical
//! RAM is full, the OS may swap (page out) inactive pages to disk. If a page
//! containing a secret key is swapped, the secret ends up in the swap file on disk
//! — persisting after the process exits!
//!
//! ```text
//! Process memory:          Swap file on disk:
//! ┌──────────────┐         ┌──────────────┐
//! │ public data  │         │ public data  │ ← swapped
//! │ SECRET KEY   │ ──────→ │ SECRET KEY   │ ← SECRET IS NOW ON DISK!
//! │ more data    │         │ more data    │ ← swapped
//! └──────────────┘         └──────────────┘
//! ```
//!
//! ## The Solution: `mlock`
//!
//! `mlock` tells the OS: "never swap this memory to disk."
//! The page stays in physical RAM until the process exits or calls `munlock`.
//!
//! ```c
//! // C equivalent
//! mlock(addr, len);   // Pin memory in RAM
//! munlock(addr, len); // Allow swapping again
//! ```
//!
//! In Rust, we use `libc::mlock` directly.
//!
//! ## `mprotect` — Change Page Permissions
//!
//! `mprotect` changes memory protection on pages:
//! - `PROT_READ`: read-only
//! - `PROT_WRITE`: write-only
//! - `PROT_EXEC`: executable
//! - `PROT_NONE`: no access (guard pages!)
//!
//! ## Limitations
//!
//! - Requires `CAP_IPC_LOCK` capability or `RLIMIT_MEMLOCK` privilege
//! - Only works on page-aligned memory
//! - Not portable (Unix-specific; Windows uses `VirtualLock`)
//! - Does NOT prevent cold boot attacks (physical RAM access)

/// Exercise 1: Implement `mlock_memory` — pin a memory region using `libc::mlock`.
///
/// Requirements:
/// - Take a `&mut [u8]` slice
/// - Call `libc::mlock` to prevent the OS from swapping it to disk
/// - Return `Ok(())` on success, `Err(String)` on failure
///
/// Hints:
/// - `libc::mlock(addr: *const libc::c_void, len: libc::size_t) -> libc::c_int`
/// - Use `.as_ptr()` and `.len()` on the slice
/// - Returns 0 on success, -1 on failure
/// - On failure, check `std::io::Error::last_os_error()`
pub fn mlock_memory(data: &mut [u8]) -> Result<(), String> {
    todo!("Pin memory in RAM using libc::mlock")
}

/// Exercise 2: Implement `munlock_memory` — unpin a memory region.
///
/// Hints:
/// - Same as mlock but calls `libc::munlock`
/// - This allows the OS to swap the memory again
pub fn munlock_memory(data: &mut [u8]) -> Result<(), String> {
    todo!("Unpin memory using libc::munlock")
}

/// Exercise 3: Implement `SecureBuffer` — a buffer that is mlock'd and zeroize'd.
///
/// Requirements:
/// - Allocate a `Vec<u8>` of the requested size
/// - Immediately `mlock` it to prevent swapping
/// - Implement `Drop` to zeroize and munlock
/// - Provide `as_slice(&self) -> &[u8]` and `as_mut_slice(&mut self) -> &mut [u8]`
///
/// Hints:
/// - Store the `Vec<u8>` and track whether it's locked
/// - In `Drop`: zeroize the data, then munlock
/// - mlock may fail if the process doesn't have privileges — handle gracefully
pub struct SecureBuffer {
    data: Vec<u8>,
    locked: bool,
}

impl SecureBuffer {
    /// Create a new SecureBuffer of `size` bytes, mlock'd in memory.
    pub fn new(size: usize) -> Self {
        todo!("Allocate and mlock a secure buffer")
    }

    pub fn as_slice(&self) -> &[u8] {
        todo!("Return reference to internal buffer")
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        todo!("Return mutable reference to internal buffer")
    }

    pub fn len(&self) -> usize {
        todo!("Return buffer length")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Check if buffer is empty")
    }
}

/// Exercise 4: Implement `page_align_size` — round up a size to the next page boundary.
///
/// Requirements:
/// - Take a byte count
/// - Round up to the next multiple of the system page size
/// - Use `libc::sysconf(libc::_SC_PAGESIZE)` to get the page size
///
/// Hints:
/// - Page size is typically 4096 bytes (4KB)
/// - Formula: `(size + page_size - 1) & !(page_size - 1)`
/// - This ensures the size is a multiple of page_size
pub fn page_align_size(size: usize) -> usize {
    todo!("Round up to next page boundary")
}

/// Exercise 5: Implement `is_mlock_supported` — check if mlock is available.
///
/// Requirements:
/// - Try to mlock a small buffer
/// - If it succeeds, munlock and return true
/// - If it fails (e.g., permission denied), return false
///
/// Hints:
/// - Allocate a small page-aligned buffer
/// - Try mlock
/// - Clean up with munlock
/// - Return whether it succeeded
pub fn is_mlock_supported() -> bool {
    todo!("Check if mlock is available on this system")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlock_success() {
        let mut data = vec![0u8; 4096];
        // mlock may fail without privileges — that's OK for the test
        let _ = mlock_memory(&mut data);
        let _ = munlock_memory(&mut data);
    }

    #[test]
    fn test_secure_buffer_new() {
        let buf = SecureBuffer::new(256);
        assert_eq!(buf.len(), 256);
        assert!(!buf.is_empty());
        assert!(buf.as_slice().iter().all(|&b| b == 0));
    }

    #[test]
    fn test_secure_buffer_write_read() {
        let mut buf = SecureBuffer::new(16);
        let data = b"secret key data!";
        buf.as_mut_slice().copy_from_slice(data);
        assert_eq!(buf.as_slice(), data);
    }

    #[test]
    fn test_secure_buffer_drop_zeroizes() {
        let mut buf = SecureBuffer::new(32);
        // Fill with non-zero data
        for (i, byte) in buf.as_mut_slice().iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        // When buf is dropped, it should zeroize (we trust the Drop impl)
        drop(buf);
        // If we got here without panic, Drop executed
    }

    #[test]
    fn test_page_align_size() {
        let aligned = page_align_size(1);
        assert!(aligned >= 1);
        assert_eq!(aligned % 4096, 0, "Should be page-aligned");
    }

    #[test]
    fn test_page_align_size_already_aligned() {
        let aligned = page_align_size(4096);
        assert_eq!(aligned, 4096);
    }

    #[test]
    fn test_page_align_size_zero() {
        let aligned = page_align_size(0);
        assert_eq!(aligned, 0);
    }

    #[test]
    fn test_mlock_supported_check() {
        // Just verify it runs without panic
        let _supported = is_mlock_supported();
    }
}
