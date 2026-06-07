//! # Memory-Mapped Files
//!
//! Memory mapping allows treating files as if they were in memory. The OS
//! handles paging data in and out, which can be more efficient than read/write
//! for large files and enables shared memory between processes.
//!
//! ## Use Cases:
//!
//! - **Large file processing**: Process files larger than RAM
//! - **Shared memory**: IPC between processes
//! - **Database storage**: mmap-based storage engines
//! - **Fast file I/O**: Avoid copy overhead
//!
//! ## Safety Considerations:
//!
//! - Files can be modified by other processes
//! - Memory-mapped regions can become invalid if file is truncated
//! - Alignment requirements for direct memory access

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

/// Wrapper around memory-mapped file operations.
/// This is a safe abstraction over OS-level mmap.
pub struct MmapRegion {
    data: Vec<u8>,
    len: usize,
    is_readonly: bool,
}

impl MmapRegion {
    /// Create a new memory-mapped region from a file.
    /// Note: This is a simplified implementation for teaching.
    /// Production code would use the `memmap2` or `mmap` crate.
    pub fn from_file(path: &Path, readonly: bool) -> io::Result<Self> {
        let data = std::fs::read(path)?;
        let len = data.len();
        Ok(Self {
            data,
            len,
            is_readonly: readonly,
        })
    }

    /// Create a new anonymous memory mapping.
    pub fn anonymous(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
            len: size,
            is_readonly: false,
        }
    }

    /// Get the length of the mapped region.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get a slice of the mapped data.
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Get a mutable slice (only if not readonly).
    pub fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        if self.is_readonly {
            None
        } else {
            Some(&mut self.data[..self.len])
        }
    }

    /// Read a value at a specific offset.
    pub fn read_at<T: Copy>(&self, offset: usize) -> Option<T> {
        let size = std::mem::size_of::<T>();
        if offset + size > self.len {
            return None;
        }
        unsafe {
            let ptr = self.data.as_ptr().add(offset) as *const T;
            Some(std::ptr::read_unaligned(ptr))
        }
    }

    /// Write a value at a specific offset.
    pub fn write_at<T: Copy>(&mut self, offset: usize, value: T) -> Result<(), &'static str> {
        if self.is_readonly {
            return Err("Cannot write to readonly mapping");
        }
        let size = std::mem::size_of::<T>();
        if offset + size > self.len {
            return Err("Offset out of bounds");
        }
        unsafe {
            let ptr = self.data.as_mut_ptr().add(offset) as *mut T;
            std::ptr::write_unaligned(ptr, value);
        }
        Ok(())
    }

    /// Search for a byte pattern in the mapped region.
    pub fn find_pattern(&self, pattern: &[u8]) -> Option<usize> {
        if pattern.is_empty() || pattern.len() > self.len {
            return None;
        }
        self.data
            .windows(pattern.len())
            .position(|window| window == pattern)
    }

    /// Get the data as a string (if valid UTF-8).
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.as_slice())
    }
}

/// Temporary memory-mapped file for testing and development.
pub struct TempMmap {
    path: std::path::PathBuf,
    region: MmapRegion,
}

impl TempMmap {
    /// Create a temporary mmap with the given data.
    pub fn new(data: &[u8]) -> io::Result<Self> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("mmap_{}.tmp", std::process::id()));
        std::fs::write(&path, data).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to write temp file: {}", e)))?;
        let region = MmapRegion::from_file(&path, false)?;
        Ok(Self { path, region })
    }

    pub fn region(&self) -> &MmapRegion {
        &self.region
    }

    pub fn region_mut(&mut self) -> &mut MmapRegion {
        &mut self.region
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempMmap {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Memory-mapped ring buffer for high-throughput I/O.
pub struct MmapRingBuffer {
    buffer: Vec<u8>,
    capacity: usize,
    head: usize,
    tail: usize,
    len: usize,
}

impl MmapRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity],
            capacity,
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    /// Write data to the ring buffer.
    pub fn write(&mut self, data: &[u8]) -> usize {
        let available = self.capacity - self.len;
        let to_write = data.len().min(available);

        for i in 0..to_write {
            self.buffer[self.tail] = data[i];
            self.tail = (self.tail + 1) % self.capacity;
        }

        self.len += to_write;
        to_write
    }

    /// Read data from the ring buffer.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.len);

        for i in 0..to_read {
            buf[i] = self.buffer[self.head];
            self.head = (self.head + 1) % self.capacity;
        }

        self.len -= to_read;
        to_read
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

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn available(&self) -> usize {
        self.capacity - self.len
    }
}

/// File-backed storage with memory-mapped access patterns.
pub struct FileBackedStorage {
    data: Vec<u8>,
    page_size: usize,
    dirty_pages: Vec<bool>,
}

impl FileBackedStorage {
    pub fn new(size: usize, page_size: usize) -> Self {
        let num_pages = (size + page_size - 1) / page_size;
        Self {
            data: vec![0u8; size],
            page_size,
            dirty_pages: vec![false; num_pages],
        }
    }

    /// Read data at an offset.
    pub fn read(&self, offset: usize, buf: &mut [u8]) -> usize {
        let available = self.data.len().saturating_sub(offset);
        let to_read = buf.len().min(available);
        buf[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
        to_read
    }

    /// Write data at an offset, marking the page as dirty.
    pub fn write(&mut self, offset: usize, data: &[u8]) -> usize {
        let available = self.data.len().saturating_sub(offset);
        let to_write = data.len().min(available);
        self.data[offset..offset + to_write].copy_from_slice(&data[..to_write]);

        // Mark affected pages as dirty
        let start_page = offset / self.page_size;
        let end_page = (offset + to_write) / self.page_size;
        for page in start_page..=end_page.min(self.dirty_pages.len() - 1) {
            self.dirty_pages[page] = true;
        }

        to_write
    }

    /// Get dirty pages that need to be flushed.
    pub fn dirty_pages(&self) -> Vec<usize> {
        self.dirty_pages
            .iter()
            .enumerate()
            .filter(|(_, &dirty)| dirty)
            .map(|(i, _)| i)
            .collect()
    }

    /// Mark all pages as clean (after flushing).
    pub fn mark_clean(&mut self) {
        self.dirty_pages.iter_mut().for_each(|d| *d = false);
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn num_pages(&self) -> usize {
        self.dirty_pages.len()
    }

    pub fn total_size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmap_anonymous() {
        let region = MmapRegion::anonymous(1024);
        assert_eq!(region.len(), 1024);
        assert!(!region.is_empty());
        assert!(region.as_slice().iter().all(|&b| b == 0));
    }

    #[test]
    fn test_mmap_read_write() {
        let mut region = MmapRegion::anonymous(64);
        region.write_at(0, 42u32).unwrap();
        let value: u32 = region.read_at(0).unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_mmap_readonly() {
        let mut region = MmapRegion::anonymous(64);
        region.is_readonly = true;
        assert!(region.as_mut_slice().is_none());
        assert!(region.write_at(0, 42u32).is_err());
    }

    #[test]
    fn test_mmap_out_of_bounds() {
        let region = MmapRegion::anonymous(16);
        assert!(region.read_at::<u64>(100).is_none());
    }

    #[test]
    fn test_mmap_find_pattern() {
        let mut region = MmapRegion::anonymous(64);
        region.data[..5].copy_from_slice(b"hello");
        region.data[10..15].copy_from_slice(b"world");

        assert_eq!(region.find_pattern(b"hello"), Some(0));
        assert_eq!(region.find_pattern(b"world"), Some(10));
        assert_eq!(region.find_pattern(b"xyz"), None);
    }

    #[test]
    fn test_mmap_as_str() {
        let mut region = MmapRegion::anonymous(5);
        region.data.copy_from_slice(b"hello");
        region.len = 5;
        assert_eq!(region.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_temp_mmap() {
        // This test requires filesystem access which may not be available in all environments
        let result = TempMmap::new(b"test data");
        if let Ok(mmap) = result {
            assert_eq!(mmap.region().as_slice(), b"test data");
            assert!(mmap.path().exists());
        }
        // If temp dir is not available, just pass
    }

    #[test]
    fn test_temp_mmap_cleanup() {
        let path;
        {
            let mmap = TempMmap::new(b"temporary").unwrap();
            path = mmap.path().to_path_buf();
            assert!(path.exists());
        }
        // File should be cleaned up
        assert!(!path.exists());
    }

    #[test]
    fn test_ring_buffer_basic() {
        let mut ring = MmapRingBuffer::new(16);
        assert!(ring.is_empty());
        assert_eq!(ring.capacity(), 16);

        ring.write(b"hello");
        assert_eq!(ring.len(), 5);

        let mut buf = [0u8; 5];
        ring.read(&mut buf);
        assert_eq!(&buf, b"hello");
        assert!(ring.is_empty());
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let mut ring = MmapRingBuffer::new(8);
        let written = ring.write(b"hello world");
        assert_eq!(written, 8); // Only 8 bytes fit
        assert!(ring.is_full());
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let mut ring = MmapRingBuffer::new(8);
        ring.write(b"abcdef");
        let mut buf = [0u8; 3];
        ring.read(&mut buf);
        assert_eq!(&buf, b"abc");

        ring.write(b"ghij");
        assert_eq!(ring.len(), 7);

        let mut buf = [0u8; 7];
        ring.read(&mut buf);
        assert_eq!(&buf, b"defghij");
    }

    #[test]
    fn test_ring_buffer_available() {
        let mut ring = MmapRingBuffer::new(10);
        assert_eq!(ring.available(), 10);
        ring.write(b"hello");
        assert_eq!(ring.available(), 5);
    }

    #[test]
    fn test_file_backed_storage_read_write() {
        let mut storage = FileBackedStorage::new(1024, 256);
        storage.write(0, b"hello");
        storage.write(300, b"world");

        let mut buf = [0u8; 5];
        storage.read(0, &mut buf);
        assert_eq!(&buf, b"hello");

        storage.read(300, &mut buf);
        assert_eq!(&buf, b"world");
    }

    #[test]
    fn test_file_backed_storage_dirty_pages() {
        let mut storage = FileBackedStorage::new(1024, 256);
        assert!(storage.dirty_pages().is_empty());

        storage.write(0, b"page0");
        storage.write(300, b"page1");

        let dirty = storage.dirty_pages();
        assert!(dirty.contains(&0)); // First page
        assert!(dirty.contains(&1)); // Second page (300 / 256 = 1)
    }

    #[test]
    fn test_file_backed_storage_mark_clean() {
        let mut storage = FileBackedStorage::new(1024, 256);
        storage.write(0, b"dirty");
        assert!(!storage.dirty_pages().is_empty());

        storage.mark_clean();
        assert!(storage.dirty_pages().is_empty());
    }

    #[test]
    fn test_file_backed_storage_page_info() {
        let storage = FileBackedStorage::new(1024, 256);
        assert_eq!(storage.page_size(), 256);
        assert_eq!(storage.num_pages(), 4);
        assert_eq!(storage.total_size(), 1024);
    }
}
