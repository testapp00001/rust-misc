//! # Lesson 08: Message Padding — Traffic Analysis Resistance
//!
//! ## Why Pad Messages?
//!
//! Even with encryption, an observer can learn information from:
//! - **Message sizes**: "OK" vs "I need to tell you something important" have very different lengths
//! - **Timing patterns**: Regular messages at 9am suggest a work schedule
//! - **Direction patterns**: Alternating messages suggest a conversation
//!
//! Padding messages to fixed or bucketed sizes hides the true message length.
//!
//! ## Padding Strategies
//!
//! 1. **Fixed-size blocks**: Pad all messages to 160 bytes, 512 bytes, etc.
//!    Simple but wastes bandwidth for short messages.
//!
//! 2. **Power-of-2 buckets**: Pad to the next power of 2 (256, 512, 1024, 2048).
//!    Good balance between privacy and efficiency.
//!
//! 3. **MIME-style**: Pad to common content lengths (tweet = 280, paragraph = 1000).
//!
//! ## Attack: Length-Based Traffic Analysis
//!
//! An attacker observing encrypted traffic can distinguish "Yes" (3 bytes + overhead)
//! from a 1000-byte document by the ciphertext length alone. **Defense**: Pad all
//! messages to uniform or bucketed sizes.
//!
//! ## Attack: Compression Oracle
//!
//! If messages are compressed before encryption, the compressed size leaks information
//! about the plaintext content (CRIME attack). **Defense**: Pad AFTER compression,
//! or disable compression for sensitive data.

use serde::{Deserialize, Serialize};

/// Padding strategy determines how messages are padded.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PaddingStrategy {
    /// No padding (insecure — for comparison only).
    None,
    /// Pad to fixed size in bytes.
    Fixed(usize),
    /// Pad to next power of 2 that is >= message length.
    PowerOf2,
    /// Pad to the nearest bucket boundary.
    /// Buckets: [64, 128, 256, 512, 1024, 2048].
    Bucket,
}

/// A padded message with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaddedMessage {
    /// The padded data (plaintext + padding).
    pub data: Vec<u8>,
    /// The original plaintext length (needed for unpadding).
    pub original_length: usize,
    /// The padding strategy used.
    pub strategy: PaddingStrategy,
}

/// Exercise 1: Pad a message to a fixed size.
///
/// If the message is shorter than `size`, append zero bytes.
/// If the message is longer, truncate to `size` (this is a simplified model).
///
/// Hints:
/// - Create a Vec of `size` bytes, initialized to 0
/// - Copy the plaintext into the beginning
/// - Store original_length for unpadding
pub fn pad_fixed(plaintext: &[u8], size: usize) -> PaddedMessage {
    todo!("Pad message to fixed size")
}

/// Exercise 2: Pad a message to the next power of 2.
///
/// Find the smallest power of 2 that is >= plaintext.len() + 4 (for length header).
/// Pad the message to that size.
///
/// Hints:
/// - Start with 64 (minimum block size)
/// - Double until >= plaintext.len() + 4
/// - Prepend the original length as 4 bytes (big-endian)
/// - Pad the rest with zeros
pub fn pad_power_of_2(plaintext: &[u8]) -> PaddedMessage {
    todo!("Pad message to next power of 2")
}

/// Exercise 3: Pad a message to the nearest bucket.
///
/// Buckets: [64, 128, 256, 512, 1024, 2048, 4096]
/// Find the smallest bucket that fits the message + 4-byte length header.
///
/// Hints:
/// - Define the bucket sizes array
/// - Find the first bucket >= plaintext.len() + 4
/// - Pad similarly to power_of_2
pub fn pad_bucket(plaintext: &[u8]) -> PaddedMessage {
    todo!("Pad message to nearest bucket size")
}

/// Exercise 4: Remove padding and recover the original message.
///
/// Extract the original length from the first 4 bytes (big-endian),
/// then return that many bytes from position 4 onward.
///
/// Hints:
/// - Read first 4 bytes as big-endian u32
/// - Extract bytes [4..4+length]
/// - Return the original plaintext
pub fn unpad(msg: &PaddedMessage) -> Vec<u8> {
    todo!("Remove padding and recover original message")
}

/// Exercise 5: Apply a padding strategy to a message.
///
/// Dispatches to the appropriate padding function based on the strategy.
///
/// Hints:
/// - Match on the strategy variant
/// - For None, create a PaddedMessage with no padding (just prepend length)
pub fn apply_padding(plaintext: &[u8], strategy: PaddingStrategy) -> PaddedMessage {
    todo!("Apply the specified padding strategy")
}

/// Exercise 6: Compute the overhead (wasted bytes) for a given strategy and message.
///
/// Returns (padded_size - original_size).
///
/// Hints:
/// - Apply the padding
/// - Return data.len() - original_length
pub fn padding_overhead(plaintext: &[u8], strategy: PaddingStrategy) -> usize {
    todo!("Compute padding overhead")
}

/// Exercise 7: Analyze privacy vs efficiency for different strategies.
///
/// Returns a Vec of (strategy_name, overhead_bytes, max_plaintext_for_bucket).
/// Useful for understanding the tradeoffs.
pub fn analyze_strategies(plaintext_len: usize) -> Vec<(String, usize, usize)> {
    todo!("Analyze padding strategies for a given message length")
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
        let data = vec![0x42u8; 64];
        let msg = pad_fixed(&data, 64);
        assert_eq!(msg.data.len(), 64);
    }

    #[test]
    fn test_pad_power_of_2_minimum() {
        let msg = pad_power_of_2(b"Hi");
        assert_eq!(msg.data.len(), 64, "Minimum block should be 64");
    }

    #[test]
    fn test_pad_power_of_2_sizes() {
        // 30 bytes + 4 header = 34, next power of 2 = 64
        let msg = pad_power_of_2(&vec![0u8; 30]);
        assert_eq!(msg.data.len(), 64);

        // 100 bytes + 4 header = 104, next power of 2 = 128
        let msg = pad_power_of_2(&vec![0u8; 100]);
        assert_eq!(msg.data.len(), 128);
    }

    #[test]
    fn test_pad_bucket_sizes() {
        // Small message fits in 64 bucket
        let msg = pad_bucket(b"Hi");
        assert_eq!(msg.data.len(), 64);

        // Larger message
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
        assert!(overhead > 0, "Should have padding overhead");
        // 64 - 2 (original) - 4 (header) = 58 bytes of actual padding
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
        // Should have entries for each strategy
        assert!(analysis.iter().any(|(name, _, _)| name.contains("PowerOf2")));
    }
}
