//! # Binary Formats
//!
//! Binary serialization formats offer better performance and smaller size
//! compared to text formats like JSON. This lesson covers patterns for
//! binary serialization using serde.
//!
//! Key concepts:
//! - Compact encoding reduces wire size
//! - Zero-copy deserialization avoids allocations
//! - Self-describing vs schema-based formats
//! - Endianness considerations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Demonstrates a compact binary representation for network messages.
/// Fixed-size fields come first, variable-size fields use length prefixes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkPacket {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
    pub checksum: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PacketHeader {
    pub version: u8,
    pub packet_type: PacketType,
    pub sequence: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PacketType {
    Data,
    Ack,
    Nack,
    Heartbeat,
}

/// Demonstrates a simple binary encoding scheme.
/// This simulates what a binary format like bincode does internally.
pub fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    while value >= 0x80 {
        result.push((value as u8) | 0x80);
        value >>= 7;
    }
    result.push(value as u8);
    result
}

pub fn decode_varint(bytes: &[u8]) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some((result, i + 1));
        }
        shift += 7;
        if shift >= 64 {
            return None; // Too many bytes
        }
    }
    None // Incomplete
}

/// Demonstrates a simple binary protocol for structured data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BinaryRecord {
    pub id: u64,
    pub name: String,
    pub values: Vec<f64>,
    pub tags: HashMap<String, String>,
}

/// Simulates encoding a BinaryRecord to a compact byte representation.
pub fn encode_record(record: &BinaryRecord) -> Vec<u8> {
    let mut buf = Vec::new();

    // ID (8 bytes, little-endian)
    buf.extend_from_slice(&record.id.to_le_bytes());

    // Name length + name bytes
    let name_bytes = record.name.as_bytes();
    buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(name_bytes);

    // Values count + values
    buf.extend_from_slice(&(record.values.len() as u32).to_le_bytes());
    for val in &record.values {
        buf.extend_from_slice(&val.to_le_bytes());
    }

    // Tags count + key-value pairs
    buf.extend_from_slice(&(record.tags.len() as u32).to_le_bytes());
    for (key, val) in &record.tags {
        let key_bytes = key.as_bytes();
        let val_bytes = val.as_bytes();
        buf.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(key_bytes);
        buf.extend_from_slice(&(val_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(val_bytes);
    }

    buf
}

pub fn decode_record(bytes: &[u8]) -> Option<BinaryRecord> {
    let mut offset = 0;

    // ID
    if bytes.len() < offset + 8 {
        return None;
    }
    let id = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
    offset += 8;

    // Name
    if bytes.len() < offset + 4 {
        return None;
    }
    let name_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    if bytes.len() < offset + name_len {
        return None;
    }
    let name = std::str::from_utf8(&bytes[offset..offset + name_len]).ok()?.to_string();
    offset += name_len;

    // Values
    if bytes.len() < offset + 4 {
        return None;
    }
    let values_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    let mut values = Vec::with_capacity(values_count);
    for _ in 0..values_count {
        if bytes.len() < offset + 8 {
            return None;
        }
        let val = f64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        values.push(val);
        offset += 8;
    }

    // Tags
    if bytes.len() < offset + 4 {
        return None;
    }
    let tags_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    let mut tags = HashMap::new();
    for _ in 0..tags_count {
        // Key
        if bytes.len() < offset + 4 {
            return None;
        }
        let key_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;
        if bytes.len() < offset + key_len {
            return None;
        }
        let key = std::str::from_utf8(&bytes[offset..offset + key_len]).ok()?.to_string();
        offset += key_len;

        // Value
        if bytes.len() < offset + 4 {
            return None;
        }
        let val_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;
        if bytes.len() < offset + val_len {
            return None;
        }
        let val = std::str::from_utf8(&bytes[offset..offset + val_len]).ok()?.to_string();
        offset += val_len;

        tags.insert(key, val);
    }

    Some(BinaryRecord { id, name, values, tags })
}

/// Demonstrates compact enum encoding using a single byte discriminant.
#[derive(Debug, Clone, PartialEq)]
pub enum CompactCommand {
    Ping,
    Reset,
    Set { key: String, value: i64 },
    Get { key: String },
    Delete { key: String },
}

pub fn encode_command(cmd: &CompactCommand) -> Vec<u8> {
    let mut buf = Vec::new();
    match cmd {
        CompactCommand::Ping => buf.push(0),
        CompactCommand::Reset => buf.push(1),
        CompactCommand::Set { key, value } => {
            buf.push(2);
            let key_bytes = key.as_bytes();
            buf.extend_from_slice(&(key_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(key_bytes);
            buf.extend_from_slice(&value.to_le_bytes());
        }
        CompactCommand::Get { key } => {
            buf.push(3);
            let key_bytes = key.as_bytes();
            buf.extend_from_slice(&(key_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(key_bytes);
        }
        CompactCommand::Delete { key } => {
            buf.push(4);
            let key_bytes = key.as_bytes();
            buf.extend_from_slice(&(key_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(key_bytes);
        }
    }
    buf
}

pub fn decode_command(bytes: &[u8]) -> Option<CompactCommand> {
    if bytes.is_empty() {
        return None;
    }
    let mut offset = 1;
    match bytes[0] {
        0 => Some(CompactCommand::Ping),
        1 => Some(CompactCommand::Reset),
        2 => {
            if bytes.len() < offset + 2 {
                return None;
            }
            let key_len = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?) as usize;
            offset += 2;
            if bytes.len() < offset + key_len + 8 {
                return None;
            }
            let key = std::str::from_utf8(&bytes[offset..offset + key_len]).ok()?.to_string();
            offset += key_len;
            let value = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
            Some(CompactCommand::Set { key, value })
        }
        3 => {
            if bytes.len() < offset + 2 {
                return None;
            }
            let key_len = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?) as usize;
            offset += 2;
            if bytes.len() < offset + key_len {
                return None;
            }
            let key = std::str::from_utf8(&bytes[offset..offset + key_len]).ok()?.to_string();
            Some(CompactCommand::Get { key })
        }
        4 => {
            if bytes.len() < offset + 2 {
                return None;
            }
            let key_len = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?) as usize;
            offset += 2;
            if bytes.len() < offset + key_len {
                return None;
            }
            let key = std::str::from_utf8(&bytes[offset..offset + key_len]).ok()?.to_string();
            Some(CompactCommand::Delete { key })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_small() {
        let encoded = encode_varint(42);
        assert_eq!(encoded.len(), 1);
        let (decoded, consumed) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, 42);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_varint_large() {
        let encoded = encode_varint(300);
        assert_eq!(encoded.len(), 2);
        let (decoded, _) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, 300);
    }

    #[test]
    fn test_varint_max() {
        let encoded = encode_varint(u64::MAX);
        let (decoded, _) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, u64::MAX);
    }

    #[test]
    fn test_binary_record_roundtrip() {
        let record = BinaryRecord {
            id: 42,
            name: "test".to_string(),
            values: vec![1.0, 2.5, 3.14],
            tags: {
                let mut map = HashMap::new();
                map.insert("env".to_string(), "prod".to_string());
                map
            },
        };
        let encoded = encode_record(&record);
        let decoded = decode_record(&encoded).unwrap();
        assert_eq!(record, decoded);
    }

    #[test]
    fn test_binary_record_empty() {
        let record = BinaryRecord {
            id: 0,
            name: String::new(),
            values: vec![],
            tags: HashMap::new(),
        };
        let encoded = encode_record(&record);
        let decoded = decode_record(&encoded).unwrap();
        assert_eq!(record, decoded);
    }

    #[test]
    fn test_command_ping() {
        let cmd = CompactCommand::Ping;
        let encoded = encode_command(&cmd);
        assert_eq!(encoded.len(), 1);
        let decoded = decode_command(&encoded).unwrap();
        assert_eq!(decoded, cmd);
    }

    #[test]
    fn test_command_set() {
        let cmd = CompactCommand::Set {
            key: "hello".to_string(),
            value: 42,
        };
        let encoded = encode_command(&cmd);
        let decoded = decode_command(&encoded).unwrap();
        assert_eq!(decoded, cmd);
    }

    #[test]
    fn test_command_get() {
        let cmd = CompactCommand::Get {
            key: "my_key".to_string(),
        };
        let encoded = encode_command(&cmd);
        let decoded = decode_command(&encoded).unwrap();
        assert_eq!(decoded, cmd);
    }

    #[test]
    fn test_command_delete() {
        let cmd = CompactCommand::Delete {
            key: "old_key".to_string(),
        };
        let encoded = encode_command(&cmd);
        let decoded = decode_command(&encoded).unwrap();
        assert_eq!(decoded, cmd);
    }

    #[test]
    fn test_decode_invalid() {
        assert!(decode_record(&[]).is_none());
        assert!(decode_command(&[]).is_none());
        assert!(decode_command(&[99]).is_none());
    }
}
