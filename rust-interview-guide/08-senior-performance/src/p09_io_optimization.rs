/// Problem: I/O Optimization
///
/// Master I/O optimization in Rust.
///
/// Key Concepts:
/// - Buffered I/O
/// - Async I/O
/// - Memory-mapped files
/// - I/O batching
/// - Zero-copy I/O

use std::io::{self, Read, Write, BufReader, BufWriter};

/// Problem 1: Buffered reading
/// Use buffered reader
pub fn buffered_read(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut reader = BufReader::new(data);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Problem 2: Buffered writing
/// Use buffered writer
pub fn buffered_write(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    {
        let mut writer = BufWriter::new(&mut buffer);
        writer.write_all(data)?;
        writer.flush()?;
    }
    Ok(buffer)
}

/// Problem 3: Chunked reading
/// Read in chunks
pub fn chunked_read(data: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    data.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect()
}

/// Problem 4: Chunked writing
/// Write in chunks
pub fn chunked_write(chunks: &[Vec<u8>]) -> Vec<u8> {
    let mut result = Vec::new();
    for chunk in chunks {
        result.extend_from_slice(chunk);
    }
    result
}

/// Problem 5: Zero-copy read
/// Simulate zero-copy read
pub fn zero_copy_read(data: &[u8]) -> &[u8] {
    data
}

/// Problem 6: Zero-copy write
/// Simulate zero-copy write
pub fn zero_copy_write(data: &[u8]) -> &[u8] {
    data
}

/// Problem 7: I/O batching
/// Batch I/O operations
pub fn io_batching(data: &[Vec<u8>]) -> Vec<u8> {
    let total_size: usize = data.iter().map(|d| d.len()).sum();
    let mut result = Vec::with_capacity(total_size);
    for chunk in data {
        result.extend_from_slice(chunk);
    }
    result
}

/// Problem 8: Async I/O (simulated)
/// Simulate async I/O
pub async fn async_io_read(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

/// Problem 9: Async I/O write (simulated)
/// Simulate async I/O write
pub async fn async_io_write(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

/// Problem 10: Memory-mapped I/O (simulated)
/// Simulate memory-mapped I/O
pub fn mmap_read(data: &[u8]) -> &[u8] {
    data
}

/// Problem 11: I/O with compression
/// Compress I/O data
pub fn io_compress(data: &[u8]) -> Vec<u8> {
    // Simulate compression
    data.to_vec()
}

/// Problem 12: I/O with decompression
/// Decompress I/O data
pub fn io_decompress(data: &[u8]) -> Vec<u8> {
    // Simulate decompression
    data.to_vec()
}

/// Problem 13: I/O with caching
/// Cache I/O results
pub struct IoCache {
    cache: std::collections::HashMap<String, Vec<u8>>,
}

impl IoCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn read(&self, key: &str) -> Option<&Vec<u8>> {
        self.cache.get(key)
    }

    pub fn write(&mut self, key: &str, data: &[u8]) {
        self.cache.insert(key.to_string(), data.to_vec());
    }
}

/// Problem 14: I/O with pooling
/// Pool I/O buffers
pub struct IoPool {
    buffers: Vec<Vec<u8>>,
}

impl IoPool {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
        }
    }

    pub fn get(&mut self) -> Vec<u8> {
        self.buffers.pop().unwrap_or_else(|| Vec::with_capacity(4096))
    }

    pub fn put(&mut self, mut buffer: Vec<u8>) {
        buffer.clear();
        self.buffers.push(buffer);
    }
}

/// Problem 15: I/O with streaming
/// Stream I/O data
pub fn io_streaming(data: &[u8], chunk_size: usize) -> impl Iterator<Item = &[u8]> {
    data.chunks(chunk_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffered_read() {
        let data = b"Hello, World!";
        let result = buffered_read(data).unwrap();
        assert_eq!(result, b"Hello, World!");
    }

    #[test]
    fn test_buffered_write() {
        let data = b"Hello, World!";
        let result = buffered_write(data).unwrap();
        assert_eq!(result, b"Hello, World!");
    }

    #[test]
    fn test_chunked_read() {
        let data = b"Hello, World!";
        let chunks = chunked_read(data, 5);
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_chunked_write() {
        let chunks = vec![b"Hello".to_vec(), b", ".to_vec(), b"World!".to_vec()];
        let result = chunked_write(&chunks);
        assert_eq!(result, b"Hello, World!");
    }

    #[test]
    fn test_zero_copy_read() {
        let data = b"Hello";
        let result = zero_copy_read(data);
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_zero_copy_write() {
        let data = b"Hello";
        let result = zero_copy_write(data);
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_io_batching() {
        let data = vec![b"Hello".to_vec(), b", ".to_vec(), b"World!".to_vec()];
        let result = io_batching(&data);
        assert_eq!(result, b"Hello, World!");
    }

    #[tokio::test]
    async fn test_async_io_read() {
        let data = b"Hello";
        let result = async_io_read(data).await;
        assert_eq!(result, b"Hello");
    }

    #[tokio::test]
    async fn test_async_io_write() {
        let data = b"Hello";
        let result = async_io_write(data).await;
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_mmap_read() {
        let data = b"Hello";
        let result = mmap_read(data);
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_io_compress() {
        let data = b"Hello";
        let result = io_compress(data);
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_io_decompress() {
        let data = b"Hello";
        let result = io_decompress(data);
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_io_cache() {
        let mut cache = IoCache::new();
        cache.write("key", b"value");
        assert_eq!(cache.read("key"), Some(&b"value".to_vec()));
    }

    #[test]
    fn test_io_pool() {
        let mut pool = IoPool::new();
        let buffer = pool.get();
        assert!(buffer.capacity() >= 4096);
        pool.put(buffer);
        let buffer2 = pool.get();
        assert!(buffer2.capacity() >= 4096);
    }

    #[test]
    fn test_io_streaming() {
        let data = b"Hello, World!";
        let chunks: Vec<&[u8]> = io_streaming(data, 5).collect();
        assert_eq!(chunks.len(), 3);
    }
}
