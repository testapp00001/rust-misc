//! # Lesson 10: Traffic Analysis Resistance (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedTrafficMessage {
    pub padded_payload: Vec<u8>,
    pub original_length: u32,
    pub is_dummy: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaddingBucket {
    Small,
    Medium,
    Large,
    XLarge,
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

pub fn pad_payload(payload: &[u8]) -> (Vec<u8>, PaddingBucket) {
    let bucket = PaddingBucket::select(payload.len());
    let target_size = bucket.size();

    // Reserve 4 bytes at the end for the original length
    let content_capacity = target_size - 4;
    let mut padded = Vec::with_capacity(target_size);

    // Copy payload
    padded.extend_from_slice(payload);

    // Pad with random bytes to fill to content_capacity
    if payload.len() < content_capacity {
        let rng = SystemRandom::new();
        let mut padding = vec![0u8; content_capacity - payload.len()];
        rng.fill(&mut padding).unwrap();
        padded.extend_from_slice(&padding);
    }

    // Append original length as big-endian u32
    padded.extend_from_slice(&(payload.len() as u32).to_be_bytes());

    (padded, bucket)
}

pub fn strip_padding(padded: &[u8]) -> Result<Vec<u8>, String> {
    if padded.len() < 4 {
        return Err("Padded payload too short".to_string());
    }

    let len_offset = padded.len() - 4;
    let original_length = u32::from_be_bytes([
        padded[len_offset],
        padded[len_offset + 1],
        padded[len_offset + 2],
        padded[len_offset + 3],
    ]) as usize;

    if original_length > padded.len() - 4 {
        return Err(format!(
            "Original length {} exceeds available data",
            original_length
        ));
    }

    Ok(padded[..original_length].to_vec())
}

pub fn generate_dummy_traffic(bucket: PaddingBucket) -> ProtectedTrafficMessage {
    let rng = SystemRandom::new();
    let size = bucket.size();
    let content_size = size - 4;

    let mut padded = vec![0u8; content_size];
    rng.fill(&mut padded).unwrap();

    // Use a random "original length" within the bucket to look realistic
    let fake_length = (content_size / 2) as u32;
    padded.extend_from_slice(&fake_length.to_be_bytes());

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    ProtectedTrafficMessage {
        padded_payload: padded,
        original_length: fake_length,
        is_dummy: true,
        timestamp: now,
    }
}

pub fn mix_traffic(
    real_messages: Vec<ProtectedTrafficMessage>,
    cover_ratio: usize,
) -> Vec<ProtectedTrafficMessage> {
    let mut mixed = Vec::new();

    for msg in real_messages {
        mixed.push(msg);
        // Add `cover_ratio` dummy messages after each real message
        let bucket = PaddingBucket::select(mixed.last().unwrap().padded_payload.len());
        for _ in 0..cover_ratio {
            mixed.push(generate_dummy_traffic(bucket));
        }
    }

    mixed
}

pub fn constant_rate_schedule(
    real_messages: Vec<(u64, ProtectedTrafficMessage)>,
    interval_ms: u64,
    total_slots: usize,
) -> Vec<ProtectedTrafficMessage> {
    let mut schedule = Vec::with_capacity(total_slots);

    // Build a lookup of real messages by slot
    let mut real_by_slot: std::collections::HashMap<usize, ProtectedTrafficMessage> =
        std::collections::HashMap::new();
    for (timestamp, msg) in real_messages {
        let slot = (timestamp / interval_ms) as usize;
        real_by_slot.insert(slot, msg);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for slot in 0..total_slots {
        if let Some(msg) = real_by_slot.remove(&slot) {
            schedule.push(msg);
        } else {
            // No real message for this slot — send dummy
            let mut dummy = generate_dummy_traffic(PaddingBucket::Small);
            dummy.timestamp = now + (slot as u64 * interval_ms / 1000);
            schedule.push(dummy);
        }
    }

    schedule
}

#[derive(Debug, Clone)]
pub struct TrafficAnalysis {
    pub total_messages: usize,
    pub dummy_count: usize,
    pub real_count: usize,
    pub size_distribution: Vec<(PaddingBucket, usize)>,
    pub avg_interval_ms: f64,
    pub burst_detected: bool,
}

pub fn analyze_traffic(messages: &[ProtectedTrafficMessage]) -> TrafficAnalysis {
    let total_messages = messages.len();
    let dummy_count = messages.iter().filter(|m| m.is_dummy).count();
    let real_count = total_messages - dummy_count;

    // Size distribution
    let mut small = 0;
    let mut medium = 0;
    let mut large = 0;
    let mut xlarge = 0;
    for msg in messages {
        let len = msg.padded_payload.len();
        if len <= 256 {
            small += 1;
        } else if len <= 1024 {
            medium += 1;
        } else if len <= 4096 {
            large += 1;
        } else {
            xlarge += 1;
        }
    }
    let size_distribution = vec![
        (PaddingBucket::Small, small),
        (PaddingBucket::Medium, medium),
        (PaddingBucket::Large, large),
        (PaddingBucket::XLarge, xlarge),
    ];

    // Timing analysis
    let avg_interval_ms = if messages.len() > 1 {
        let mut intervals = Vec::new();
        for i in 1..messages.len() {
            let diff = messages[i]
                .timestamp
                .saturating_sub(messages[i - 1].timestamp);
            intervals.push(diff as f64 * 1000.0);
        }
        intervals.iter().sum::<f64>() / intervals.len() as f64
    } else {
        0.0
    };

    // Burst detection: if any 1-second window has > 5 messages
    let mut burst_detected = false;
    for i in 0..messages.len() {
        let window_end = messages[i].timestamp + 1;
        let count = messages[i..]
            .iter()
            .take_while(|m| m.timestamp < window_end)
            .count();
        if count > 5 {
            burst_detected = true;
            break;
        }
    }

    TrafficAnalysis {
        total_messages,
        dummy_count,
        real_count,
        size_distribution,
        avg_interval_ms,
        burst_detected,
    }
}

pub fn measure_protection_effectiveness(messages: &[ProtectedTrafficMessage]) -> f64 {
    if messages.is_empty() {
        return 0.0;
    }

    let analysis = analyze_traffic(messages);
    let mut score = 0.0;

    // Dummy ratio: higher is better (max 0.4)
    let dummy_ratio = analysis.dummy_count as f64 / analysis.total_messages as f64;
    score += dummy_ratio * 0.4;

    // Size diversity: more bucket types used is better (max 0.3)
    let used_buckets = analysis
        .size_distribution
        .iter()
        .filter(|(_, count)| *count > 0)
        .count();
    score += (used_buckets as f64 / 4.0) * 0.3;

    // Timing regularity: lower burst score is better (max 0.3)
    if !analysis.burst_detected {
        score += 0.3;
    }

    score.min(1.0)
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
        assert_eq!(mixed.len(), 9);
        let dummy_count = mixed.iter().filter(|m| m.is_dummy).count();
        assert_eq!(dummy_count, 6);
    }

    #[test]
    fn test_constant_rate_fills_gaps() {
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
        let mut messages = Vec::new();
        for i in 0..100 {
            messages.push(ProtectedTrafficMessage {
                padded_payload: vec![0u8; if i % 3 == 0 { 256 } else { 1024 }],
                original_length: 10,
                is_dummy: i % 2 == 0,
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
