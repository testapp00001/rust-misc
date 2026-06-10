//! # Lesson 10: Traffic Analysis Resistance
//!
//! ## What is Traffic Analysis?
//!
//! Even with encryption, an observer can learn a lot from traffic patterns:
//! - **Message size**: A 1-byte message is "yes/no", a 1000-byte message is a document
//! - **Timing**: Messages at regular intervals suggest a heartbeat; bursts suggest activity
//! - **Direction**: Who is sending to whom, and when
//! - **Volume**: High traffic to a specific server suggests a specific service
//!
//! ## Attack Scenario: Encrypted Traffic Analysis
//!
//! A journalist uses an encrypted messaging app. An observer cannot read the
//! messages, but can see:
//! - The journalist contacts a specific whistleblower's IP every Tuesday at 2am
//! - Messages are ~500 bytes (short texts)
//! - After contact, the journalist's newspaper publishes stories
//!
//! The encryption is broken without breaking the cipher.
//!
//! ## Defenses
//!
//! 1. **Padding**: Pad all messages to fixed sizes (e.g., 1KB, 4KB, 16KB)
//!    so an observer cannot distinguish message types by size.
//!
//! 2. **Constant-rate traffic**: Send data at a fixed rate, regardless of
//!    whether you have real data. Fill idle time with dummy traffic.
//!
//! 3. **Dummy traffic**: Send random encrypted messages to hide real traffic
//!    patterns among cover traffic.
//!
//! 4. **Mix networks**: Route messages through multiple relays, mixing
//!    traffic from different users.
//!
//! ## Why This Matters
//!
//! Encryption protects content. Traffic analysis protection protects metadata.
//! Metadata can be more revealing than content — who you talk to, when, and
//! how often tells a story even without reading the messages.

use ring::digest;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A message with traffic analysis protection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedTrafficMessage {
    /// Padded payload (always a fixed size bucket)
    pub padded_payload: Vec<u8>,
    /// Original payload length (encrypted inside)
    pub original_length: u32,
    /// Whether this is real or dummy traffic
    pub is_dummy: bool,
    /// Timestamp (for constant-rate enforcement)
    pub timestamp: u64,
}

/// Supported padding size buckets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaddingBucket {
    Small,  // 256 bytes
    Medium, // 1024 bytes
    Large,  // 4096 bytes
    XLarge, // 16384 bytes
}

impl PaddingBucket {
    pub fn size(&self) -> usize {
        match self {
            PaddingBucket::Small => 256,
            PaddingBucket::Medium => 1024,
            PaddingBucket::Large => 4096,
            PaddingBucket::XLarge => 16384,
        }
    }

    /// Select the smallest bucket that fits the payload.
    pub fn select(payload_len: usize) -> Self {
        if payload_len <= 256 {
            PaddingBucket::Small
        } else if payload_len <= 1024 {
            PaddingBucket::Medium
        } else if payload_len <= 4096 {
            PaddingBucket::Large
        } else {
            PaddingBucket::XLarge
        }
    }
}

/// Exercise: Pad a payload to the next size bucket.
///
/// All messages should be one of the standard sizes so an observer
/// cannot distinguish message types by length.
///
/// Hints:
/// - Select the appropriate padding bucket
/// - Pad with random bytes to reach the bucket size
/// - Store the original length so the receiver can strip padding
pub fn pad_payload(payload: &[u8]) -> (Vec<u8>, PaddingBucket) {
    todo!("Implement payload padding")
}

/// Exercise: Strip padding from a padded payload.
///
/// Hints:
/// - The last 4 bytes store the original length (big-endian u32)
/// - Extract the original payload using this length
pub fn strip_padding(padded: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Implement padding removal")
}

/// Exercise: Generate dummy traffic (cover traffic).
///
/// Dummy messages are indistinguishable from real messages.
/// They have the same size distribution and timing.
///
/// Hints:
/// - Generate random bytes of a random size bucket
/// - Set is_dummy = true
/// - Use ring::rand for the random bytes
pub fn generate_dummy_traffic(bucket: PaddingBucket) -> ProtectedTrafficMessage {
    todo!("Implement dummy traffic generation")
}

/// Exercise: Mix real and dummy traffic to hide real message patterns.
///
/// For every real message, generate `cover_ratio` dummy messages.
///
/// Hints:
/// - Iterate through real messages
/// - After each real message, add `cover_ratio` dummy messages
/// - Shuffle or interleave them
pub fn mix_traffic(
    real_messages: Vec<ProtectedTrafficMessage>,
    cover_ratio: usize,
) -> Vec<ProtectedTrafficMessage> {
    todo!("Implement traffic mixing")
}

/// Exercise: Enforce constant-rate traffic.
///
/// Messages are sent at fixed intervals regardless of actual activity.
/// If there is no real message, send a dummy.
///
/// Hints:
/// - For each time slot, check if there is a real message to send
/// - If not, generate a dummy message
/// - Return the list of messages for each time slot
pub fn constant_rate_schedule(
    real_messages: Vec<(u64, ProtectedTrafficMessage)>,
    interval_ms: u64,
    total_slots: usize,
) -> Vec<ProtectedTrafficMessage> {
    todo!("Implement constant-rate scheduling")
}

/// Exercise: Analyze traffic to detect patterns (attack tool).
///
/// This simulates what an attacker does: look at message sizes and timing
/// to infer information about the communication.
///
/// Returns a summary of detected patterns.
///
/// Hints:
/// - Group messages by size bucket
/// - Calculate inter-message timing statistics
/// - Identify bursts (many messages in a short time)
pub fn analyze_traffic(messages: &[ProtectedTrafficMessage]) -> TrafficAnalysis {
    todo!("Implement traffic analysis")
}

/// Results of traffic analysis.
#[derive(Debug, Clone)]
pub struct TrafficAnalysis {
    pub total_messages: usize,
    pub dummy_count: usize,
    pub real_count: usize,
    pub size_distribution: Vec<(PaddingBucket, usize)>,
    pub avg_interval_ms: f64,
    pub burst_detected: bool,
}

/// Exercise: Measure the effectiveness of traffic analysis protection.
///
/// Compare traffic patterns with and without protection.
/// Returns a score from 0.0 (no protection) to 1.0 (perfect protection).
///
/// Hints:
/// - Perfect protection: all buckets equally used, constant intervals, high dummy ratio
/// - Poor protection: all messages same size, bursty, no dummies
pub fn measure_protection_effectiveness(messages: &[ProtectedTrafficMessage]) -> f64 {
    todo!("Implement protection measurement")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_roundtrip() {
        let payload = b"short message";
        let (padded, bucket) = pad_payload(payload);
        assert_eq!(padded.len(), bucket.size());
        let stripped = strip_padding(&padded).unwrap();
        assert_eq!(stripped, payload);
    }

    #[test]
    fn test_padding_all_sizes_bucket() {
        let small = pad_payload(b"hi");
        assert_eq!(small.1, PaddingBucket::Small);

        let medium = pad_payload(&vec![0u8; 300]);
        assert_eq!(medium.1, PaddingBucket::Medium);

        let large = pad_payload(&vec![0u8; 2000]);
        assert_eq!(large.1, PaddingBucket::Large);
    }

    #[test]
    fn test_dummy_traffic_correct_size() {
        for bucket in [PaddingBucket::Small, PaddingBucket::Medium, PaddingBucket::Large] {
            let dummy = generate_dummy_traffic(bucket);
            assert!(dummy.is_dummy);
            assert_eq!(dummy.padded_payload.len(), bucket.size());
        }
    }

    #[test]
    fn test_mix_traffic_ratio() {
        let real: Vec<ProtectedTrafficMessage> = (0..3)
            .map(|_| ProtectedTrafficMessage {
                padded_payload: vec![0u8; 256],
                original_length: 10,
                is_dummy: false,
                timestamp: 0,
            })
            .collect();

        let mixed = mix_traffic(real, 2);
        // 3 real + 3*2 dummy = 9 total
        assert_eq!(mixed.len(), 9);
        let dummy_count = mixed.iter().filter(|m| m.is_dummy).count();
        assert_eq!(dummy_count, 6);
    }

    #[test]
    fn test_constant_rate_fills_gaps() {
        // Send 1 message, but schedule 5 slots -> should get 4 dummies
        let msg = ProtectedTrafficMessage {
            padded_payload: vec![0u8; 256],
            original_length: 10,
            is_dummy: false,
            timestamp: 0,
        };
        let real = vec![(0, msg)];
        let schedule = constant_rate_schedule(real, 100, 5);
        assert_eq!(schedule.len(), 5);
        let dummy_count = schedule.iter().filter(|m| m.is_dummy).count();
        assert_eq!(dummy_count, 4);
    }

    #[test]
    fn test_traffic_analysis_counts() {
        let messages = vec![
            ProtectedTrafficMessage {
                padded_payload: vec![0u8; 256],
                original_length: 10,
                is_dummy: false,
                timestamp: 0,
            },
            ProtectedTrafficMessage {
                padded_payload: vec![0u8; 256],
                original_length: 5,
                is_dummy: true,
                timestamp: 100,
            },
        ];
        let analysis = analyze_traffic(&messages);
        assert_eq!(analysis.total_messages, 2);
        assert_eq!(analysis.real_count, 1);
        assert_eq!(analysis.dummy_count, 1);
    }

    #[test]
    fn test_protection_effectiveness_high() {
        // Well-protected traffic: mix of sizes, many dummies
        let mut messages = Vec::new();
        for i in 0..100 {
            messages.push(ProtectedTrafficMessage {
                padded_payload: vec![0u8; if i % 3 == 0 { 256 } else { 1024 }],
                original_length: 10,
                is_dummy: i % 2 == 0, // 50% dummy
                timestamp: i * 100,
            });
        }
        let score = measure_protection_effectiveness(&messages);
        assert!(score > 0.3, "Protection score should be reasonable: {}", score);
    }

    #[test]
    fn test_padding_preserves_original_data() {
        let payload = vec![0xABu8; 500];
        let (padded, _) = pad_payload(&payload);
        let stripped = strip_padding(&padded).unwrap();
        assert_eq!(stripped, payload);
    }
}
