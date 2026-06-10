//! # Lesson 05: Replay Attack and Defense
//!
//! ## What is a Replay Attack?
//!
//! An attacker records a valid message and re-transmits it later. The message
//! is authentic (properly signed/encrypted) and arrives at the right server,
//! but it is being played at the wrong time or too many times.
//!
//! ```text
//! Legitimate:  Client --[Transfer $100]--> Server  (OK)
//! Attacker:    Attacker --[Transfer $100]--> Server  (REPLAY!)
//! Attacker:    Attacker --[Transfer $100]--> Server  (REPLAY!)
//! Attacker:    Attacker --[Transfer $100]--> Server  (REPLAY!)
//! ```
//!
//! ## Defenses
//!
//! 1. **Nonce (Number used ONCE)**: Each message includes a unique ID.
//!    The server tracks seen nonces and rejects duplicates.
//!
//! 2. **Timestamp**: Messages include a timestamp. The server rejects messages
//!    outside a time window (e.g., 5 minutes). Limits replay window.
//!
//! 3. **Sequence numbers**: Messages have incrementing sequence numbers.
//!    The server rejects out-of-order or duplicate numbers.
//!
//! 4. **Combined**: Nonce + timestamp + sequence provides the strongest defense.
//!
//! ## Attack Scenario: TLS 1.3 0-RTT Replay
//!
//! TLS 1.3 supports 0-RTT (zero round-trip time) resumption, where the client
//! can send data in the first message. This data has NO replay protection.
//! An attacker can replay the 0-RTT data. Servers must design 0-RTT handlers
//! to be idempotent (safe to replay).
//!
//! ## Why This Matters
//!
//! Any protocol over TLS can still be vulnerable to replay attacks at the
//! application layer. TLS protects the transport, but your application must
//! protect its own messages.

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// A message with anti-replay protection.
#[derive(Debug, Clone)]
pub struct ProtectedMessage {
    /// Unique nonce for this message
    pub nonce: [u8; 16],
    /// Unix timestamp when the message was created
    pub timestamp: u64,
    /// Monotonically increasing sequence number
    pub sequence: u64,
    /// The actual message payload
    pub payload: Vec<u8>,
    /// HMAC over all fields (nonce + timestamp + sequence + payload)
    pub mac: Vec<u8>,
}

/// Exercise: Generate a unique nonce.
///
/// A nonce must be unique for each message. Use random bytes.
///
/// Hints:
/// - Use `ring::rand::SystemRandom` and `ring::rand::generate`
/// - The nonce should be 16 bytes
pub fn generate_nonce() -> [u8; 16] {
    todo!("Implement nonce generation")
}

/// Exercise: Get the current Unix timestamp in seconds.
///
/// Hints:
/// - Use `SystemTime::now().duration_since(UNIX_EPOCH)`
pub fn current_timestamp() -> u64 {
    todo!("Implement timestamp retrieval")
}

/// Exercise: Create a protected message with nonce, timestamp, and sequence number.
///
/// Hints:
/// - Generate a nonce
/// - Get the current timestamp
/// - Compute an HMAC over nonce + timestamp + sequence + payload
/// - Use `ring::hmac` with the provided key
pub fn create_protected_message(
    payload: &[u8],
    sequence: u64,
    hmac_key: &[u8],
) -> ProtectedMessage {
    todo!("Implement protected message creation")
}

/// Exercise: Verify a protected message's integrity and freshness.
///
/// Checks:
/// 1. HMAC is valid (integrity)
/// 2. Timestamp is within the allowed window (freshness)
/// 3. Sequence number has not been seen before (replay protection)
/// 4. Nonce has not been seen before (replay protection)
///
/// Hints:
/// - Recompute the HMAC and compare with the message's MAC
/// - Check timestamp against current time with a tolerance window
/// - Check nonce and sequence against the seen sets
/// - On success, insert nonce and sequence into the seen sets
pub fn verify_protected_message(
    msg: &ProtectedMessage,
    hmac_key: &[u8],
    tolerance_secs: u64,
    seen_nonces: &mut HashSet<[u8; 16]>,
    seen_sequences: &mut HashSet<u64>,
) -> Result<(), String> {
    todo!("Implement message verification with replay detection")
}

/// A replay-aware message processor.
///
/// Tracks seen nonces and sequences to detect replays.
pub struct ReplayGuard {
    hmac_key: Vec<u8>,
    tolerance_secs: u64,
    seen_nonces: HashSet<[u8; 16]>,
    seen_sequences: HashSet<u64>,
    next_expected_sequence: u64,
}

impl ReplayGuard {
    /// Create a new ReplayGuard.
    ///
    /// Exercise: Initialize with the given HMAC key and tolerance window.
    pub fn new(hmac_key: Vec<u8>, tolerance_secs: u64) -> Self {
        todo!("Create a new ReplayGuard")
    }

    /// Process a message, rejecting replays.
    ///
    /// Exercise: Verify the message and reject if:
    /// - HMAC is invalid
    /// - Timestamp is outside tolerance
    /// - Nonce was already seen
    /// - Sequence number was already seen
    pub fn process(&mut self, msg: &ProtectedMessage) -> Result<Vec<u8>, String> {
        todo!("Implement replay-safe message processing")
    }
}

/// Exercise: Demonstrate a replay attack.
///
/// Create a valid message, then show that replaying it is detected.
///
/// Hints:
/// - Create a protected message
/// - Process it successfully once
/// - Try to process the same message again
/// - The second attempt should fail
pub fn demonstrate_replay_attack(
    guard: &mut ReplayGuard,
    payload: &[u8],
    sequence: u64,
) -> Result<(Vec<u8>, String), String> {
    todo!("Implement replay attack demonstration")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Vec<u8> {
        vec![0x42u8; 32]
    }

    #[test]
    fn test_generate_nonce_unique() {
        let n1 = generate_nonce();
        let n2 = generate_nonce();
        assert_ne!(n1, n2, "Nonces must be unique");
    }

    #[test]
    fn test_current_timestamp_reasonable() {
        let ts = current_timestamp();
        // Should be after 2020 and before 2100
        assert!(ts > 1_577_836_800);
        assert!(ts < 4_102_444_800);
    }

    #[test]
    fn test_protected_message_roundtrip() {
        let key = test_key();
        let msg = create_protected_message(b"hello", 1, &key);
        let mut nonces = HashSet::new();
        let mut seqs = HashSet::new();
        assert!(verify_protected_message(&msg, &key, 300, &mut nonces, &mut seqs).is_ok());
    }

    #[test]
    fn test_replay_detected() {
        let key = test_key();
        let msg = create_protected_message(b"transfer $100", 1, &key);
        let mut guard = ReplayGuard::new(key, 300);

        // First time: success
        let result1 = guard.process(&msg);
        assert!(result1.is_ok());

        // Replay: should fail
        let result2 = guard.process(&msg);
        assert!(result2.is_err(), "Replay should be detected");
    }

    #[test]
    fn test_tampered_message_rejected() {
        let key = test_key();
        let mut msg = create_protected_message(b"hello", 1, &key);
        // Tamper with the payload
        msg.payload[0] = b'X';
        let mut nonces = HashSet::new();
        let mut seqs = HashSet::new();
        assert!(verify_protected_message(&msg, &key, 300, &mut nonces, &mut seqs).is_err());
    }

    #[test]
    fn test_sequence_ordering() {
        let key = test_key();
        let mut guard = ReplayGuard::new(key.clone(), 300);

        // Process messages in order
        for seq in 1..=5 {
            let msg = create_protected_message(b"data", seq, &key);
            assert!(guard.process(&msg).is_ok(), "Sequence {} should succeed", seq);
        }

        // Replay sequence 3
        let msg = create_protected_message(b"data", 3, &key);
        assert!(guard.process(&msg).is_err(), "Replayed sequence should fail");
    }

    #[test]
    fn test_wrong_hmac_key_rejected() {
        let key = test_key();
        let wrong_key = vec![0x99u8; 32];
        let msg = create_protected_message(b"hello", 1, &key);
        let mut nonces = HashSet::new();
        let mut seqs = HashSet::new();
        assert!(verify_protected_message(&msg, &wrong_key, 300, &mut nonces, &mut seqs).is_err());
    }

    #[test]
    fn test_demonstrate_replay_attack() {
        let key = test_key();
        let mut guard = ReplayGuard::new(key, 300);
        let (payload, _error) = demonstrate_replay_attack(&mut guard, b"transfer $100", 1).unwrap();
        assert_eq!(payload, b"transfer $100");
    }

    #[test]
    fn test_different_nonces_different_messages() {
        let key = test_key();
        let msg1 = create_protected_message(b"hello", 1, &key);
        let msg2 = create_protected_message(b"hello", 1, &key);
        assert_ne!(msg1.nonce, msg2.nonce, "Same input should produce different nonces");
    }
}
