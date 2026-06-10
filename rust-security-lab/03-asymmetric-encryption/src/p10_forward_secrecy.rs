//! # Lesson 10: Forward Secrecy — Ephemeral Keys for Session Security
//!
//! ## What is Forward Secrecy?
//!
//! Forward secrecy (also called perfect forward secrecy, PFS) ensures that
//! compromise of long-term keys does not compromise past session keys.
//!
//! **Without forward secrecy:**
//! ```
//! Long-term key compromised → ALL past sessions decryptable
//! ```
//!
//! **With forward secrecy:**
//! ```
//! Long-term key compromised → Only sessions using that key are exposed
//! Past sessions (with deleted ephemeral keys) remain secure
//! ```
//!
//! ## How It Works
//!
//! 1. Each session generates a NEW ephemeral keypair
//! 2. Key exchange uses ephemeral keys (not long-term keys)
//! 3. After the session, ephemeral private keys are DELETED
//! 4. Even if the long-term key is later compromised, past sessions are safe
//!
//! ## Real-World Usage
//!
//! - **TLS 1.3**: Always uses ephemeral ECDHE (mandated by the standard)
//! - **Signal Protocol**: Double Ratchet generates new ephemeral keys per message
//! - **WireGuard**: New ephemeral key per session (1-RTT handshake)
//!
//! ## Why Not Always Use Forward Secrecy?
//!
//! There's no real downside — all modern protocols require it.
//! The only "cost" is generating a new keypair per session, which is
//! negligible with X25519 (~150,000 key generations/second).
//!
//! ## Attack Demo: No Forward Secrecy = Total Compromise
//!
//! If an attacker records encrypted traffic for months, then steals the server's
//! private key, they can decrypt ALL recorded traffic. With forward secrecy,
//! they can only decrypt the current session (if any).

use x25519_dalek::{EphemeralSecret, PublicKey};
use ring::hkdf;
use rand::rngs::OsRng;
use zeroize::Zeroize;

/// Exercise 1: Create an ephemeral keypair for a single session.
///
/// Returns (public_key_bytes, secret_handle).
/// The secret must be zeroized when the session ends.
///
/// Hints:
/// - Use `EphemeralSecret::random_from_rng(OsRng)`
/// - Get public key: `PublicKey::from(&secret)`
/// - Return both
pub fn create_ephemeral_session() -> ([u8; 32], Vec<u8>) {
    todo!("Create an ephemeral X25519 keypair for a session")
}

/// Exercise 2: Perform ephemeral key exchange between two sessions.
///
/// Both parties generate ephemeral keys, perform ECDH, and derive a session key.
/// The ephemeral secrets should be dropped (zeroized) after use.
///
/// Returns the 32-byte session key.
pub fn ephemeral_key_exchange() -> [u8; 32] {
    todo!("Perform ephemeral ECDH and derive session key")
}

/// Exercise 3: Demonstrate that deleting ephemeral keys provides forward secrecy.
///
/// Simulate two sessions with different ephemeral keys.
/// Show that compromising the long-term key doesn't help decrypt past sessions.
///
/// Returns true if forward secrecy property holds.
pub fn demonstrate_forward_secrecy() -> bool {
    todo!("Show that ephemeral keys provide forward secrecy")
}

/// Exercise 4: Implement the Signal Protocol's key ratcheting concept.
///
/// After each message exchange, advance the key:
/// session_key(n+1) = HKDF(session_key(n), "ratchet")
///
/// Returns a chain of 5 derived keys.
pub fn key_ratchet_chain(initial_key: &[u8; 32]) -> Vec<[u8; 32]> {
    todo!("Implement key ratcheting (simplified Signal Protocol)")
}

/// Exercise 5: Demonstrate that each ratchet step produces a different key.
///
/// Returns true if all keys in the chain are unique.
pub fn ratchet_keys_unique() -> bool {
    todo!("Show that ratchet produces unique keys")
}

/// Exercise 6: Compare ephemeral vs static key exchange security properties.
///
/// Returns a summary: (uses_ephemeral, provides_forward_secrecy, key_reuse).
pub fn compare_security_properties() -> (bool, bool, bool) {
    todo!("Compare ephemeral and static key exchange security properties")
}

/// Exercise 7: Implement secure session teardown.
///
/// After a session ends, all ephemeral key material must be zeroized.
/// Demonstrates the full lifecycle: create → use → destroy.
pub fn secure_session_lifecycle(message: &[u8]) -> Vec<u8> {
    todo!("Full session lifecycle: create, use, destroy ephemeral keys")
}

/// Derive a session key from ephemeral ECDH shared secret.
fn derive_session_key(shared_secret: &[u8; 32], context: &[u8]) -> [u8; 32] {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, b"forward-secrecy-session");
    let prk = salt.extract(shared_secret);
    let info = [context];
    let okm = prk.expand(&info, hkdf::HKDF_SHA256).unwrap();
    let mut key = [0u8; 32];
    okm.fill(&mut key).unwrap();
    key
}

/// Ratchet a key forward (one step).
fn ratchet_step(current_key: &[u8; 32], step: usize) -> [u8; 32] {
    let mut label = b"ratchet-step-".to_vec();
    label.extend_from_slice(step.to_string().as_bytes());
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, current_key);
    let prk = salt.extract(b"ratchet");
    let info = [label.as_slice()];
    let okm = prk.expand(&info, hkdf::HKDF_SHA256).unwrap();
    let mut key = [0u8; 32];
    okm.fill(&mut key).unwrap();
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeral_session_creation() {
        let (pub_key, _secret) = create_ephemeral_session();
        assert_eq!(pub_key.len(), 32, "Public key should be 32 bytes");
    }

    #[test]
    fn test_ephemeral_exchange_produces_key() {
        let session_key = ephemeral_key_exchange();
        assert_eq!(session_key.len(), 32, "Session key should be 32 bytes");
    }

    #[test]
    fn test_forward_secrecy_demonstrated() {
        assert!(demonstrate_forward_secrecy(),
            "Ephemeral keys should provide forward secrecy");
    }

    #[test]
    fn test_ratchet_chain_length() {
        let initial = [1u8; 32];
        let chain = key_ratchet_chain(&initial);
        assert_eq!(chain.len(), 5, "Chain should have 5 keys");
    }

    #[test]
    fn test_ratchet_keys_are_unique() {
        assert!(ratchet_keys_unique(), "Ratchet should produce unique keys");
    }

    #[test]
    fn test_ratchet_step_produces_different_key() {
        let key1 = [42u8; 32];
        let key2 = ratchet_step(&key1, 0);
        assert_ne!(key1, key2, "Ratchet step should change the key");
    }

    #[test]
    fn test_ratchet_is_deterministic() {
        let key = [42u8; 32];
        let step1_a = ratchet_step(&key, 0);
        let step1_b = ratchet_step(&key, 0);
        assert_eq!(step1_a, step1_b, "Ratchet should be deterministic");
    }

    #[test]
    fn test_security_properties() {
        let (ephemeral, forward_secrecy, key_reuse) = compare_security_properties();
        // Ephemeral exchange should provide forward secrecy and no key reuse
        assert!(ephemeral, "Should use ephemeral keys");
        assert!(forward_secrecy, "Should provide forward secrecy");
        assert!(!key_reuse, "Should not reuse keys");
    }

    #[test]
    fn test_secure_lifecycle() {
        let message = b"test message";
        let result = secure_session_lifecycle(message);
        assert!(!result.is_empty(), "Lifecycle should produce output");
    }

    #[test]
    fn test_two_sessions_different_keys() {
        let key1 = ephemeral_key_exchange();
        let key2 = ephemeral_key_exchange();
        assert_ne!(key1, key2, "Two sessions should produce different keys");
    }
}
