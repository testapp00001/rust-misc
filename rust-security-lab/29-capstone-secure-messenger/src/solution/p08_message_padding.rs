//! # Lesson 08: Message Padding — Traffic Analysis Resistance (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

/// Padding strategy determines how messages are padded.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PaddingStrategy {
    None,
    Fixed(usize),
    PowerOf2,
    Bucket,
}

/// A padded message with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaddedMessage {
    pub data: Vec<u8>,
    pub original_length: usize,
    pub strategy: PaddingStrategy,
}

const BUCKETS: &[usize] = &[64, 128, 256, 512, 1024, 2048, 4096];

/// Pad a message to a fixed size.
pub fn pad_fixed(plaintext: &[u8], size: usize) -> PaddedMessage {
    let header_size = 4; // 4 bytes for length
    let mut data = vec![0u8; size];
    let len = plaintext.len().min(size - header_size);
    data[..4].copy_from_slice(&(len as u32).to_be_bytes());
    data[4..4 + len].copy_from_slice(&plaintext[..len]);
    PaddedMessage {
        data,
        original_length: plaintext.len(),
        strategy: PaddingStrategy::Fixed(size),
    }
}

/// Pad a message to the next power of 2.
pub fn pad_power_of_2(plaintext: &[u8]) -> PaddedMessage {
    let needed = plaintext.len() + 4; // 4 bytes for length header
    let mut size: usize = 64; // minimum block
    while size < needed {
        size *= 2;
    }

    let mut data = vec![0u8; size];
    data[..4].copy_from_slice(&(plaintext.len() as u32).to_be_bytes());
    data[4..4 + plaintext.len()].copy_from_slice(plaintext);

    PaddedMessage {
        data,
        original_length: plaintext.len(),
        strategy: PaddingStrategy::PowerOf2,
    }
}

/// Pad a message to the nearest bucket.
pub fn pad_bucket(plaintext: &[u8]) -> PaddedMessage {
    let needed = plaintext.len() + 4;
    let size = BUCKETS.iter()
        .find(|&&b| b >= needed)
        .copied()
        .unwrap_or(4096);

    let mut data = vec![0u8; size];
    data[..4].copy_from_slice(&(plaintext.len() as u32).to_be_bytes());
    data[4..4 + plaintext.len()].copy_from_slice(plaintext);

    PaddedMessage {
        data,
        original_length: plaintext.len(),
        strategy: PaddingStrategy::Bucket,
    }
}

/// Remove padding and recover the original message.
pub fn unpad(msg: &PaddedMessage) -> Vec<u8> {
    if msg.data.len() < 4 {
        return vec![];
    }
    let len = u32::from_be_bytes([msg.data[0], msg.data[1], msg.data[2], msg.data[3]]) as usize;
    if len + 4 > msg.data.len() {
        return vec![];
    }
    msg.data[4..4 + len].to_vec()
}

/// Apply a padding strategy to a message.
pub fn apply_padding(plaintext: &[u8], strategy: PaddingStrategy) -> PaddedMessage {
    match strategy {
        PaddingStrategy::None => {
            let mut data = Vec::with_capacity(4 + plaintext.len());
            data.extend_from_slice(&(plaintext.len() as u32).to_be_bytes());
            data.extend_from_slice(plaintext);
            PaddedMessage {
                data,
                original_length: plaintext.len(),
                strategy: PaddingStrategy::None,
            }
        }
        PaddingStrategy::Fixed(size) => pad_fixed(plaintext, size),
        PaddingStrategy::PowerOf2 => pad_power_of_2(plaintext),
        PaddingStrategy::Bucket => pad_bucket(plaintext),
    }
}

/// Compute the overhead for a given strategy and message.
pub fn padding_overhead(plaintext: &[u8], strategy: PaddingStrategy) -> usize {
    let padded = apply_padding(plaintext, strategy);
    padded.data.len() - plaintext.len()
}

/// Analyze privacy vs efficiency for different strategies.
pub fn analyze_strategies(plaintext_len: usize) -> Vec<(String, usize, usize)> {
    let strategies = vec![
        ("None".to_string(), PaddingStrategy::None),
        ("Fixed-512".to_string(), PaddingStrategy::Fixed(512)),
        ("PowerOf2".to_string(), PaddingStrategy::PowerOf2),
        ("Bucket".to_string(), PaddingStrategy::Bucket),
    ];

    strategies.into_iter().map(|(name, strat)| {
        let dummy = vec![0u8; plaintext_len];
        let overhead = padding_overhead(&dummy, strat);
        let max_plaintext = match strat {
            PaddingStrategy::Fixed(s) => s - 4,
            PaddingStrategy::PowerOf2 => {
                let needed = plaintext_len + 4;
                let mut size: usize = 64;
                while size < needed { size *= 2; }
                size - 4
            }
            PaddingStrategy::Bucket => {
                let needed = plaintext_len + 4;
                BUCKETS.iter().find(|&&b| b >= needed).copied().unwrap_or(4096) - 4
            }
            PaddingStrategy::None => plaintext_len,
        };
        (name, overhead, max_plaintext)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_fixed_shorter() {
        let msg = pad_fixed(b"Hello", 64);
        assert_eq!(msg.data.len(), 64);
        assert_eq!(msg.original_length, 5);
    }

    #[test]
    fn test_pad_fixed_exact() {
        let data = vec![0x42u8; 60]; // 60 + 4 header = 64
        let msg = pad_fixed(&data, 64);
        assert_eq!(msg.data.len(), 64);
    }

    #[test]
    fn test_pad_power_of_2_minimum() {
        let msg = pad_power_of_2(b"Hi");
        assert_eq!(msg.data.len(), 64);
    }

    #[test]
    fn test_pad_power_of_2_sizes() {
        let msg = pad_power_of_2(&vec![0u8; 30]);
        assert_eq!(msg.data.len(), 64);

        let msg = pad_power_of_2(&vec![0u8; 100]);
        assert_eq!(msg.data.len(), 128);
    }

    #[test]
    fn test_pad_bucket_sizes() {
        let msg = pad_bucket(b"Hi");
        assert_eq!(msg.data.len(), 64);

        let msg = pad_bucket(&vec![0u8; 200]);
        assert_eq!(msg.data.len(), 256);
    }

    #[test]
    fn test_unpad_roundtrip() {
        let plaintext = b"Hello, secure world!";
        let padded = pad_power_of_2(plaintext);
        let recovered = unpad(&padded);
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn test_unpad_bucket_roundtrip() {
        let plaintext = vec![0xABu8; 500];
        let padded = pad_bucket(&plaintext);
        let recovered = unpad(&padded);
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn test_padding_overhead() {
        let overhead = padding_overhead(b"Hi", PaddingStrategy::PowerOf2);
        assert!(overhead > 0);
        assert_eq!(overhead, 64 - 2);
    }

    #[test]
    fn test_apply_padding_dispatches() {
        let plaintext = b"Test message";
        let msg = apply_padding(plaintext, PaddingStrategy::PowerOf2);
        let recovered = unpad(&msg);
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn test_analyze_strategies() {
        let analysis = analyze_strategies(100);
        assert!(!analysis.is_empty());
        assert!(analysis.iter().any(|(name, _, _)| name.contains("PowerOf2")));
    }
}
