//! # Lesson 06: Secure Message Framing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub length: u32,
    pub payload: Vec<u8>,
    pub checksum: u32,
}

pub const MAX_FRAME_SIZE: u32 = 1_048_576;

pub fn compute_checksum(data: &[u8]) -> u32 {
    // Simple rolling XOR-based checksum folded into u32
    let mut checksum: u32 = 0;
    for (i, &byte) in data.iter().enumerate() {
        checksum ^= (byte as u32).wrapping_shl((i % 4) as u32 * 8);
    }
    checksum
}

pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let length = payload.len() as u32;
    let checksum = compute_checksum(payload);

    let mut frame = Vec::with_capacity(4 + payload.len() + 4);
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(payload);
    frame.extend_from_slice(&checksum.to_be_bytes());
    frame
}

pub fn decode_frame(data: &[u8]) -> Result<Frame, String> {
    if data.len() < 8 {
        return Err("Not enough data for a frame header + checksum".to_string());
    }

    let length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

    if length > MAX_FRAME_SIZE {
        return Err(format!(
            "Frame length {} exceeds maximum {}",
            length, MAX_FRAME_SIZE
        ));
    }

    let total_size = 4 + length as usize + 4;
    if data.len() < total_size {
        return Err(format!(
            "Need {} bytes but only {} available",
            total_size,
            data.len()
        ));
    }

    let payload = data[4..4 + length as usize].to_vec();
    let checksum = u32::from_be_bytes([
        data[4 + length as usize],
        data[5 + length as usize],
        data[6 + length as usize],
        data[7 + length as usize],
    ]);

    let expected_checksum = compute_checksum(&payload);
    if checksum != expected_checksum {
        return Err(format!(
            "Checksum mismatch: expected {}, got {}",
            expected_checksum, checksum
        ));
    }

    Ok(Frame {
        length,
        payload,
        checksum,
    })
}

pub fn decode_frames(data: &[u8]) -> Result<Vec<Frame>, String> {
    let mut frames = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        if data.len() - offset < 8 {
            break; // Not enough data for another frame
        }

        let length = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;

        let frame_size = 4 + length + 4;
        if data.len() - offset < frame_size {
            break; // Incomplete frame
        }

        let frame = decode_frame(&data[offset..offset + frame_size])?;
        frames.push(frame);
        offset += frame_size;
    }

    Ok(frames)
}

pub fn escape_payload(payload: &[u8]) -> Vec<u8> {
    let mut escaped = Vec::with_capacity(payload.len());
    for &byte in payload {
        match byte {
            b'\n' => {
                escaped.push(b'\\');
                escaped.push(b'n');
            }
            b'\\' => {
                escaped.push(b'\\');
                escaped.push(b'\\');
            }
            b'\0' => {
                escaped.push(b'\\');
                escaped.push(b'0');
            }
            _ => escaped.push(byte),
        }
    }
    escaped
}

pub fn unescape_payload(escaped: &[u8]) -> Result<Vec<u8>, String> {
    let mut unescaped = Vec::with_capacity(escaped.len());
    let mut i = 0;
    while i < escaped.len() {
        if escaped[i] == b'\\' {
            if i + 1 >= escaped.len() {
                return Err("Trailing backslash".to_string());
            }
            match escaped[i + 1] {
                b'n' => {
                    unescaped.push(b'\n');
                    i += 2;
                }
                b'\\' => {
                    unescaped.push(b'\\');
                    i += 2;
                }
                b'0' => {
                    unescaped.push(b'\0');
                    i += 2;
                }
                _ => {
                    return Err(format!("Unknown escape sequence: \\{}", escaped[i + 1] as char));
                }
            }
        } else {
            unescaped.push(escaped[i]);
            i += 1;
        }
    }
    Ok(unescaped)
}

pub fn validate_frame(frame: &Frame) -> Result<(), String> {
    if frame.payload.len() != frame.length as usize {
        return Err(format!(
            "Length mismatch: field says {} but payload is {} bytes",
            frame.length,
            frame.payload.len()
        ));
    }
    if frame.length > MAX_FRAME_SIZE {
        return Err(format!(
            "Frame length {} exceeds maximum {}",
            frame.length, MAX_FRAME_SIZE
        ));
    }
    let expected = compute_checksum(&frame.payload);
    if frame.checksum != expected {
        return Err(format!(
            "Checksum mismatch: expected {}, got {}",
            expected, frame.checksum
        ));
    }
    Ok(())
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
        let mut data = Vec::new();
        data.extend_from_slice(&(MAX_FRAME_SIZE + 1).to_be_bytes());
        data.extend_from_slice(&[0u8; 8]);
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
