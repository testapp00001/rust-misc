//! # Lesson 06: Secure Message Framing
//!
//! ## What is Message Framing?
//!
//! When sending messages over a stream (TCP), the receiver needs to know where
//! one message ends and the next begins. Poor framing leads to injection attacks.
//!
//! ## Framing Strategies
//!
//! 1. **Length prefix**: Each message starts with its length (e.g., 4 bytes).
//!    Simple, efficient, but a corrupted length can cause desync.
//!
//! 2. **Delimiter**: Messages end with a special character (e.g., `\n`).
//!    Simple, but the delimiter must be escaped in the payload.
//!
//! 3. **Fixed size**: All messages are exactly N bytes. No flexibility.
//!
//! ## Attack Scenario: Length Injection
//!
//! If the length field is not validated, an attacker can set a huge length,
//! causing the receiver to buffer excessive data (DoS) or read into the next
//! message's data (message injection).
//!
//! ```text
//! Attacker sends: [length=999999][payload]
//! Receiver allocates 999999 bytes, reads past the message boundary
//! ```
//!
//! ## Attack Scenario: Delimiter Injection
//!
//! If the payload contains the delimiter character and it is not escaped,
//! the receiver splits the message in the wrong place.
//!
//! ## Why This Matters
//!
//! Every network protocol needs framing. Getting it wrong means message
//! confusion, injection, or denial of service.

use serde::{Deserialize, Serialize};

/// A secure message frame using length-prefix encoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    /// Length of the payload (4 bytes, big-endian)
    pub length: u32,
    /// The message payload
    pub payload: Vec<u8>,
    /// CRC-32 checksum of the payload for integrity
    pub checksum: u32,
}

/// Maximum allowed frame size (1 MB) to prevent memory exhaustion.
pub const MAX_FRAME_SIZE: u32 = 1_048_576;

/// Exercise: Encode a message into a length-prefixed frame.
///
/// Format: [4-byte length (big-endian)][payload][4-byte CRC-32]
///
/// Hints:
/// - Convert payload length to 4 big-endian bytes
/// - Compute a simple checksum (XOR of all payload bytes, or use ring)
/// - Concatenate: length_bytes + payload + checksum_bytes
pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    todo!("Implement frame encoding")
}

/// Exercise: Decode a frame from a byte stream.
///
/// Hints:
/// - Read first 4 bytes as big-endian u32 length
/// - Validate length <= MAX_FRAME_SIZE
/// - Read `length` bytes as payload
/// - Read next 4 bytes as checksum
/// - Verify checksum matches the payload
pub fn decode_frame(data: &[u8]) -> Result<Frame, String> {
    todo!("Implement frame decoding")
}

/// Exercise: Decode multiple frames from a continuous byte stream.
///
/// Hints:
/// - Use a cursor/offset to track position in the buffer
/// - Repeatedly call decode_frame starting from the current offset
/// - Advance the offset by the frame's total size after each decode
/// - Stop when there are not enough bytes for a complete frame
pub fn decode_frames(data: &[u8]) -> Result<Vec<Frame>, String> {
    todo!("Implement multi-frame decoding")
}

/// Exercise: Escape delimiter-based framing.
///
/// For delimiter-based framing, escape special characters in the payload.
///
/// Rules:
/// - Escape the delimiter (`\n` -> `\\n`)
/// - Escape the escape character (`\\` -> `\\\\`)
/// - Escape null bytes (`\0` -> `\\0`)
///
/// Hints:
/// - Iterate through bytes, escaping special characters
pub fn escape_payload(payload: &[u8]) -> Vec<u8> {
    todo!("Implement payload escaping")
}

/// Exercise: Unescape a delimiter-escaped payload.
///
/// Hints:
/// - Reverse the escaping: `\\n` -> `\n`, `\\\\` -> `\\`, `\\0` -> `\0`
pub fn unescape_payload(escaped: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Implement payload unescaping")
}

/// Exercise: Compute a simple checksum for a payload.
///
/// Use a CRC-like approach: XOR all bytes, folded into a u32.
///
/// Hints:
/// - XOR all bytes together
/// - For stronger protection, use a rolling hash or ring::digest
pub fn compute_checksum(data: &[u8]) -> u32 {
    todo!("Implement checksum computation")
}

/// Exercise: Validate that a frame is well-formed.
///
/// Checks:
/// - Length field matches actual payload length
/// - Checksum is valid
/// - Length does not exceed MAX_FRAME_SIZE
pub fn validate_frame(frame: &Frame) -> Result<(), String> {
    todo!("Implement frame validation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let payload = b"Hello, world!";
        let encoded = encode_frame(payload);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_decode_empty_payload() {
        let encoded = encode_frame(b"");
        let decoded = decode_frame(&encoded).unwrap();
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn test_decode_large_payload() {
        let payload = vec![0xABu8; 10_000];
        let encoded = encode_frame(&payload);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_decode_rejects_oversized_frame() {
        // Craft a frame claiming to be larger than MAX_FRAME_SIZE
        let mut data = Vec::new();
        data.extend_from_slice(&(MAX_FRAME_SIZE + 1).to_be_bytes());
        data.extend_from_slice(&[0u8; 8]); // dummy payload + checksum
        assert!(decode_frame(&data).is_err());
    }

    #[test]
    fn test_decode_multiple_frames() {
        let f1 = encode_frame(b"message one");
        let f2 = encode_frame(b"message two");
        let f3 = encode_frame(b"message three");

        let mut stream = Vec::new();
        stream.extend_from_slice(&f1);
        stream.extend_from_slice(&f2);
        stream.extend_from_slice(&f3);

        let frames = decode_frames(&stream).unwrap();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].payload, b"message one");
        assert_eq!(frames[1].payload, b"message two");
        assert_eq!(frames[2].payload, b"message three");
    }

    #[test]
    fn test_escape_unescape_roundtrip() {
        let payload = b"line1\nline2\\end\0null";
        let escaped = escape_payload(payload);
        let unescaped = unescape_payload(&escaped).unwrap();
        assert_eq!(unescaped, payload);
    }

    #[test]
    fn test_checksum_deterministic() {
        let data = b"test data";
        let c1 = compute_checksum(data);
        let c2 = compute_checksum(data);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_validate_frame_valid() {
        let frame = Frame {
            length: 5,
            payload: b"hello".to_vec(),
            checksum: compute_checksum(b"hello"),
        };
        assert!(validate_frame(&frame).is_ok());
    }

    #[test]
    fn test_validate_frame_bad_checksum() {
        let frame = Frame {
            length: 5,
            payload: b"hello".to_vec(),
            checksum: 0xDEADBEEF,
        };
        assert!(validate_frame(&frame).is_err());
    }

    #[test]
    fn test_validate_frame_length_mismatch() {
        let frame = Frame {
            length: 999,
            payload: b"hello".to_vec(),
            checksum: compute_checksum(b"hello"),
        };
        assert!(validate_frame(&frame).is_err());
    }
}
