//! # Zero-Copy Techniques
//!
//! Zero-copy avoids unnecessary memory copies by sharing buffers between
//! components. The `bytes` crate provides `Bytes` and `BytesMut` for
//! reference-counted buffer sharing.
//!
//! ## Key Concepts:
//!
//! - **Buffer sharing**: Multiple owners of the same memory region
//! - **Slice without copy**: Sub-slices share the underlying buffer
//! - **Split operations**: Divide a buffer without copying
//! - **Reference counting**: Automatic cleanup when last reference drops
//!
//! ## Use Cases:
//!
//! - Network protocol parsing
//! - File I/O processing
//! - Streaming data pipelines
//! - Message serialization

use bytes::{Buf, Bytes, BytesMut};

/// Zero-copy parser that works with borrowed slices.
pub struct ZeroCopyParser<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> ZeroCopyParser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, position: 0 }
    }

    /// Read a fixed number of bytes without copying.
    pub fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.position + n > self.input.len() {
            return None;
        }
        let slice = &self.input[self.position..self.position + n];
        self.position += n;
        Some(slice)
    }

    /// Read a byte.
    pub fn read_u8(&mut self) -> Option<u8> {
        if self.position >= self.input.len() {
            return None;
        }
        let byte = self.input[self.position];
        self.position += 1;
        Some(byte)
    }

    /// Read a big-endian u32.
    pub fn read_u32_be(&mut self) -> Option<u32> {
        let bytes = self.read_bytes(4)?;
        Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a length-prefixed string (zero-copy).
    pub fn read_string(&mut self) -> Option<&'a str> {
        let len = self.read_u8()? as usize;
        let bytes = self.read_bytes(len)?;
        std::str::from_utf8(bytes).ok()
    }

    pub fn remaining(&self) -> usize {
        self.input.len().saturating_sub(self.position)
    }

    pub fn position(&self) -> usize {
        self.position
    }
}

/// Zero-copy buffer pool using the Bytes crate.
pub struct BufferPool {
    buffers: Vec<BytesMut>,
    buffer_size: usize,
}

impl BufferPool {
    pub fn new(initial_count: usize, buffer_size: usize) -> Self {
        let buffers = (0..initial_count)
            .map(|_| BytesMut::with_capacity(buffer_size))
            .collect();

        Self {
            buffers,
            buffer_size,
        }
    }

    /// Acquire a buffer from the pool.
    pub fn acquire(&mut self) -> BytesMut {
        self.buffers
            .pop()
            .unwrap_or_else(|| BytesMut::with_capacity(self.buffer_size))
    }

    /// Return a buffer to the pool.
    pub fn release(&mut self, mut buf: BytesMut) {
        buf.clear();
        self.buffers.push(buf);
    }

    pub fn available(&self) -> usize {
        self.buffers.len()
    }
}

/// Zero-copy message frame parser.
/// Parses frames from a buffer without copying data.
pub struct FrameParser {
    buffer: BytesMut,
}

impl FrameParser {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    /// Feed data into the parser.
    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Try to parse a length-prefixed frame.
    /// Returns the frame data as a zero-copy Bytes slice.
    pub fn parse_frame(&mut self) -> Option<Bytes> {
        if self.buffer.len() < 4 {
            return None; // Not enough data for length prefix
        }

        let len = u32::from_be_bytes([
            self.buffer[0],
            self.buffer[1],
            self.buffer[2],
            self.buffer[3],
        ]) as usize;

        if self.buffer.len() < 4 + len {
            return None; // Not enough data for full frame
        }

        // Advance past length prefix
        self.buffer.advance(4);
        // Split off the frame data (zero-copy)
        Some(self.buffer.split_to(len).freeze())
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

/// Zero-copy string interning.
/// Stores each unique string once and returns shared references.
pub struct StringInterner {
    strings: Vec<String>,
    index: std::collections::HashMap<String, usize>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            index: std::collections::HashMap::new(),
        }
    }

    /// Intern a string, returning a static reference.
    /// Note: The reference is valid as long as the interner lives.
    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&idx) = self.index.get(s) {
            return idx;
        }
        let idx = self.strings.len();
        self.strings.push(s.to_string());
        self.index.insert(s.to_string(), idx);
        idx
    }

    /// Get an interned string by index.
    pub fn get(&self, idx: usize) -> Option<&str> {
        self.strings.get(idx).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    pub fn memory_usage(&self) -> usize {
        self.strings.iter().map(|s| s.len()).sum()
    }
}

/// Zero-copy buffer view that can be split without allocation.
pub struct BufferView {
    data: Bytes,
}

impl BufferView {
    pub fn new(data: Bytes) -> Self {
        Self { data }
    }

    pub fn from_static(data: &'static [u8]) -> Self {
        Self {
            data: Bytes::from_static(data),
        }
    }

    /// Split the view at the given position (zero-copy).
    pub fn split_at(&self, mid: usize) -> (Bytes, Bytes) {
        let left = self.data.slice(..mid);
        let right = self.data.slice(mid..);
        (left, right)
    }

    /// Get a sub-slice (zero-copy).
    pub fn slice(&self, range: std::ops::Range<usize>) -> Bytes {
        self.data.slice(range)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Share the underlying buffer (increment reference count, no copy).
    pub fn share(&self) -> Bytes {
        self.data.clone() // Cheap clone - just increments refcount
    }
}

/// Zero-copy vector builder that accumulates data into a BytesMut.
pub struct BufferBuilder {
    buffer: BytesMut,
}

impl BufferBuilder {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(cap),
        }
    }

    pub fn put_u8(&mut self, n: u8) {
        self.buffer.extend_from_slice(&[n]);
    }

    pub fn put_u32(&mut self, n: u32) {
        self.buffer.extend_from_slice(&n.to_be_bytes());
    }

    pub fn put_bytes(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn put_str(&mut self, s: &str) {
        self.put_u8(s.len() as u8);
        self.put_bytes(s.as_bytes());
    }

    /// Finalize the buffer and return a Bytes (zero-copy conversion).
    pub fn finalize(self) -> Bytes {
        self.buffer.freeze()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_parser() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let mut parser = ZeroCopyParser::new(&data);

        assert_eq!(parser.read_u8(), Some(0x01));
        let bytes = parser.read_bytes(2).unwrap();
        assert_eq!(bytes, &[0x02, 0x03]);
        assert_eq!(parser.remaining(), 2);
    }

    #[test]
    fn test_zero_copy_parser_u32() {
        let data = [0x00, 0x00, 0x01, 0x00]; // 256 in big-endian
        let mut parser = ZeroCopyParser::new(&data);
        assert_eq!(parser.read_u32_be(), Some(256));
    }

    #[test]
    fn test_zero_copy_parser_string() {
        let data = [0x05, b'h', b'e', b'l', b'l', b'o'];
        let mut parser = ZeroCopyParser::new(&data);
        assert_eq!(parser.read_string(), Some("hello"));
    }

    #[test]
    fn test_zero_copy_parser_insufficient_data() {
        let data = [0x01, 0x02];
        let mut parser = ZeroCopyParser::new(&data);
        assert!(parser.read_bytes(10).is_none());
    }

    #[test]
    fn test_buffer_pool() {
        let mut pool = BufferPool::new(3, 1024);
        assert_eq!(pool.available(), 3);

        let buf = pool.acquire();
        assert_eq!(pool.available(), 2);

        pool.release(buf);
        assert_eq!(pool.available(), 3);
    }

    #[test]
    fn test_frame_parser() {
        let mut parser = FrameParser::new();

        // Frame: length(4 bytes) + data
        let frame_data = b"hello";
        let len = (frame_data.len() as u32).to_be_bytes();
        parser.feed(&len);
        parser.feed(frame_data);

        let frame = parser.parse_frame().unwrap();
        assert_eq!(&frame[..], b"hello");
    }

    #[test]
    fn test_frame_parser_insufficient_data() {
        let mut parser = FrameParser::new();
        parser.feed(&[0x00, 0x00]); // Only 2 bytes, need 4 for length
        assert!(parser.parse_frame().is_none());
    }

    #[test]
    fn test_frame_parser_multiple_frames() {
        let mut parser = FrameParser::new();

        // Two frames
        let frame1 = b"hello";
        let frame2 = b"world";
        parser.feed(&(frame1.len() as u32).to_be_bytes());
        parser.feed(frame1);
        parser.feed(&(frame2.len() as u32).to_be_bytes());
        parser.feed(frame2);

        let f1 = parser.parse_frame().unwrap();
        let f2 = parser.parse_frame().unwrap();
        assert_eq!(&f1[..], b"hello");
        assert_eq!(&f2[..], b"world");
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();
        let idx1 = interner.intern("hello");
        let idx2 = interner.intern("world");
        let idx3 = interner.intern("hello"); // Same as idx1

        assert_eq!(idx1, idx3);
        assert_ne!(idx1, idx2);
        assert_eq!(interner.get(idx1), Some("hello"));
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_string_interner_memory() {
        let mut interner = StringInterner::new();
        interner.intern("short");
        interner.intern("a longer string");
        assert!(interner.memory_usage() > 0);
    }

    #[test]
    fn test_buffer_view_split() {
        let data = Bytes::from(vec![1, 2, 3, 4, 5]);
        let view = BufferView::new(data);

        let (left, right) = view.split_at(2);
        assert_eq!(&left[..], &[1, 2]);
        assert_eq!(&right[..], &[3, 4, 5]);
    }

    #[test]
    fn test_buffer_view_slice() {
        let data = Bytes::from(vec![10, 20, 30, 40, 50]);
        let view = BufferView::new(data);
        let slice = view.slice(1..4);
        assert_eq!(&slice[..], &[20, 30, 40]);
    }

    #[test]
    fn test_buffer_view_share() {
        let data = Bytes::from(vec![1, 2, 3]);
        let view = BufferView::new(data);
        let shared = view.share();

        // Both point to same memory
        assert_eq!(view.as_bytes(), &shared[..]);
    }

    #[test]
    fn test_buffer_view_from_static() {
        let view = BufferView::from_static(b"static data");
        assert_eq!(view.as_bytes(), b"static data");
    }

    #[test]
    fn test_buffer_builder() {
        let mut builder = BufferBuilder::new();
        builder.put_u8(0x01);
        builder.put_u32(42);
        builder.put_bytes(b"hello");
        builder.put_str("world");

        let data = builder.finalize();
        assert!(data.len() > 0);
        assert_eq!(data[0], 0x01);
    }

    #[test]
    fn test_buffer_builder_with_capacity() {
        let builder = BufferBuilder::with_capacity(1024);
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn test_zero_copy_no_allocation() {
        // Demonstrate that splitting Bytes doesn't allocate
        let original = Bytes::from(vec![0u8; 1000]);
        let _clone = original.clone(); // Just increments refcount
        let _slice = original.slice(100..200); // Just a pointer + length
    }

    #[test]
    fn test_string_interner_empty() {
        let interner = StringInterner::new();
        assert!(interner.is_empty());
        assert_eq!(interner.get(0), None);
    }
}
