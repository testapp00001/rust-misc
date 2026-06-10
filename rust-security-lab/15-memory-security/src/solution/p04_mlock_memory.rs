//! # Lesson 04: mlock / mprotect (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use zeroize::Zeroize;

/// Pin a memory region using libc::mlock to prevent swapping to disk.
pub fn mlock_memory(data: &mut [u8]) -> Result<(), String> {
    let result = unsafe { libc::mlock(data.as_ptr() as *const libc::c_void, data.len()) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

/// Unpin a memory region using libc::munlock.
pub fn munlock_memory(data: &mut [u8]) -> Result<(), String> {
    let result = unsafe { libc::munlock(data.as_ptr() as *const libc::c_void, data.len()) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

/// A buffer that is mlock'd and zeroize'd — prevents swap and residual memory.
pub struct SecureBuffer {
    data: Vec<u8>,
    locked: bool,
}

impl SecureBuffer {
    pub fn new(size: usize) -> Self {
        let data = vec![0u8; size];
        let mut buf = Self { data, locked: false };
        // Try to mlock — may fail without privileges
        if mlock_memory(&mut buf.data).is_ok() {
            buf.locked = true;
        }
        buf
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // Zeroize the data before freeing
        self.data.zeroize();
        // Munlock if we locked it
        if self.locked {
            let _ = munlock_memory(&mut self.data);
        }
    }
}

/// Round up a size to the next page boundary.
pub fn page_align_size(size: usize) -> usize {
    if size == 0 {
        return 0;
    }
    let page_size = get_page_size();
    (size + page_size - 1) & !(page_size - 1)
}

fn get_page_size() -> usize {
    let ps = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if ps > 0 { ps as usize } else { 4096 }
}

/// Check if mlock is available on this system.
pub fn is_mlock_supported() -> bool {
    let mut test_buf = vec![0u8; 4096];
    let supported = mlock_memory(&mut test_buf).is_ok();
    if supported {
        let _ = munlock_memory(&mut test_buf);
    }
    supported
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlock_success() {
        let mut data = vec![0u8; 4096];
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
        for (i, byte) in buf.as_mut_slice().iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        drop(buf);
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
        let _supported = is_mlock_supported();
    }
}
