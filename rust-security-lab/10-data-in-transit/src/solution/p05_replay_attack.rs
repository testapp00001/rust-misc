//! # Lesson 05: Replay Attack and Defense (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ProtectedMessage {
    pub nonce: [u8; 16],
    pub timestamp: u64,
    pub sequence: u64,
    pub payload: Vec<u8>,
    pub mac: Vec<u8>,
}

pub fn generate_nonce() -> [u8; 16] {
    let rng = SystemRandom::new();
    let mut nonce = [0u8; 16];
    rng.fill(&mut nonce).unwrap();
    nonce
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn create_protected_message(
    payload: &[u8],
    sequence: u64,
    hmac_key: &[u8],
) -> ProtectedMessage {
    let nonce = generate_nonce();
    let timestamp = current_timestamp();

    // Compute HMAC over nonce + timestamp + sequence + payload
    let key = hmac::Key::new(hmac::HMAC_SHA256, hmac_key);
    let mut data = Vec::new();
    data.extend_from_slice(&nonce);
    data.extend_from_slice(&timestamp.to_be_bytes());
    data.extend_from_slice(&sequence.to_be_bytes());
    data.extend_from_slice(payload);
    let mac = hmac::sign(&key, &data).as_ref().to_vec();

    ProtectedMessage {
        nonce,
        timestamp,
        sequence,
        payload: payload.to_vec(),
        mac,
    }
}

pub fn verify_protected_message(
    msg: &ProtectedMessage,
    hmac_key: &[u8],
    tolerance_secs: u64,
    seen_nonces: &mut HashSet<[u8; 16]>,
    seen_sequences: &mut HashSet<u64>,
) -> Result<(), String> {
    // 1. Verify HMAC
    let key = hmac::Key::new(hmac::HMAC_SHA256, hmac_key);
    let mut data = Vec::new();
    data.extend_from_slice(&msg.nonce);
    data.extend_from_slice(&msg.timestamp.to_be_bytes());
    data.extend_from_slice(&msg.sequence.to_be_bytes());
    data.extend_from_slice(&msg.payload);
    let computed_mac = hmac::sign(&key, &data);

    ring::constant_time::verify_slices_are_equal(computed_mac.as_ref(), &msg.mac)
        .map_err(|_| "HMAC verification failed".to_string())?;

    // 2. Check timestamp freshness
    let now = current_timestamp();
    if msg.timestamp > now + tolerance_secs {
        return Err("Message timestamp is in the future".to_string());
    }
    if now > msg.timestamp + tolerance_secs {
        return Err("Message timestamp is too old".to_string());
    }

    // 3. Check nonce uniqueness
    if seen_nonces.contains(&msg.nonce) {
        return Err("Nonce already seen (replay detected)".to_string());
    }

    // 4. Check sequence uniqueness
    if seen_sequences.contains(&msg.sequence) {
        return Err("Sequence number already seen (replay detected)".to_string());
    }

    // Record nonce and sequence
    seen_nonces.insert(msg.nonce);
    seen_sequences.insert(msg.sequence);

    Ok(())
}

pub struct ReplayGuard {
    hmac_key: Vec<u8>,
    tolerance_secs: u64,
    seen_nonces: HashSet<[u8; 16]>,
    seen_sequences: HashSet<u64>,
    next_expected_sequence: u64,
}

impl ReplayGuard {
    pub fn new(hmac_key: Vec<u8>, tolerance_secs: u64) -> Self {
        ReplayGuard {
            hmac_key,
            tolerance_secs,
            seen_nonces: HashSet::new(),
            seen_sequences: HashSet::new(),
            next_expected_sequence: 1,
        }
    }

    pub fn process(&mut self, msg: &ProtectedMessage) -> Result<Vec<u8>, String> {
        verify_protected_message(
            msg,
            &self.hmac_key,
            self.tolerance_secs,
            &mut self.seen_nonces,
            &mut self.seen_sequences,
        )?;
        Ok(msg.payload.clone())
    }
}

pub fn demonstrate_replay_attack(
    guard: &mut ReplayGuard,
    payload: &[u8],
    sequence: u64,
) -> Result<(Vec<u8>, String), String> {
    let msg = create_protected_message(payload, sequence, &guard.hmac_key.clone());

    // First processing succeeds
    let result = guard.process(&msg)?;

    // Replay attempt fails
    let replay_err = guard
        .process(&msg)
        .expect_err("Replay should fail");

    Ok((result, replay_err))
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

        let result1 = guard.process(&msg);
        assert!(result1.is_ok());

        let result2 = guard.process(&msg);
        assert!(result2.is_err(), "Replay should be detected");
    }

    #[test]
    fn test_tampered_message_rejected() {
        let key = test_key();
        let mut msg = create_protected_message(b"hello", 1, &key);
        msg.payload[0] = b'X';
        let mut nonces = HashSet::new();
        let mut seqs = HashSet::new();
        assert!(verify_protected_message(&msg, &key, 300, &mut nonces, &mut seqs).is_err());
    }

    #[test]
    fn test_sequence_ordering() {
        let key = test_key();
        let mut guard = ReplayGuard::new(key.clone(), 300);

        for seq in 1..=5 {
            let msg = create_protected_message(b"data", seq, &key);
            assert!(guard.process(&msg).is_ok(), "Sequence {} should succeed", seq);
        }

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
