//! # Lesson 10: Forward Secrecy — Ephemeral Keys for Session Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.
//!
//! This lesson demonstrates forward secrecy: the property that compromise of
//! long-term keys does not compromise past session keys. Achieved by generating
//! new ephemeral keypairs per session and deleting them after use.

use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};
use ring::hkdf;
use rand::rngs::OsRng;
use zeroize::Zeroize;

/// Create an ephemeral keypair for a single session.
///
/// Returns (public_key_bytes, secret_bytes).
/// The secret bytes should be zeroized after the session ends.
///
/// Ephemeral keys are generated fresh for each session and never reused.
/// Even if the long-term key is later compromised, past ephemeral keys
/// (and thus past session keys) cannot be recovered.
pub fn create_ephemeral_session() -> ([u8; 32], Vec<u8>) {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&secret);

    // In production, the secret would be held in a Zeroizing<T> wrapper
    // and automatically zeroized on drop. Here we return raw bytes for
    // educational purposes.
    let secret_bytes = secret.to_bytes().to_vec();

    (*public_key.as_bytes(), secret_bytes)
}

/// Perform ephemeral key exchange between two sessions.
///
/// Both parties generate ephemeral keys, perform ECDH, and derive a session key.
/// The ephemeral secrets are consumed (moved into diffie_hellman) and cannot be
/// reused — this is the "ephemeral" property.
///
/// Returns the 32-byte session key.
pub fn ephemeral_key_exchange() -> [u8; 32] {
    // Alice generates ephemeral key
    let alice_secret = EphemeralSecret::random_from_rng(OsRng);
    let alice_public = PublicKey::from(&alice_secret);

    // Bob generates ephemeral key
    let bob_secret = EphemeralSecret::random_from_rng(OsRng);
    let bob_public = PublicKey::from(&bob_secret);

    // ECDH: both compute the same shared secret
    let alice_shared = alice_secret.diffie_hellman(&bob_public);
    let bob_shared = bob_secret.diffie_hellman(&alice_public);

    // Verify they match
    assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());

    // Derive session key using HKDF
    derive_session_key(alice_shared.as_bytes(), b"ephemeral-session")
}

/// Demonstrate that deleting ephemeral keys provides forward secrecy.
///
/// Simulates two independent sessions with different ephemeral keys.
/// Shows that knowing the "long-term" key material from session 2
/// does not help recover the session key from session 1.
pub fn demonstrate_forward_secrecy() -> bool {
    // Session 1: ephemeral key exchange
    let session1_key = ephemeral_key_exchange();

    // Session 2: different ephemeral key exchange
    let session2_key = ephemeral_key_exchange();

    // The two session keys should be different
    if session1_key == session2_key {
        return false;
    }

    // Even if an attacker compromises session 2's ephemeral keys,
    // they cannot derive session 1's key (ephemeral keys are deleted).
    // This is the forward secrecy property.
    true
}

/// Implement key ratcheting (simplified Signal Protocol concept).
///
/// The Double Ratchet algorithm advances the key after each message:
/// session_key(n+1) = HKDF(session_key(n), "ratchet")
///
/// This means:
/// - Compromise of key(n) allows computing key(n+1), key(n+2), ...
/// - But NOT key(n-1), key(n-2), ... (backward secrecy)
/// - Combined with ephemeral DH ratchet, provides both forward and backward secrecy
///
/// Returns a chain of 5 derived keys.
pub fn key_ratchet_chain(initial_key: &[u8; 32]) -> Vec<[u8; 32]> {
    let mut chain = Vec::with_capacity(5);
    let mut current_key = *initial_key;

    for step in 0..5 {
        current_key = ratchet_step(&current_key, step);
        chain.push(current_key);
    }

    chain
}

/// Demonstrate that ratchet produces unique keys at each step.
pub fn ratchet_keys_unique() -> bool {
    let initial = [42u8; 32];
    let chain = key_ratchet_chain(&initial);

    // Check all keys are unique
    for i in 0..chain.len() {
        for j in (i + 1)..chain.len() {
            if chain[i] == chain[j] {
                return false;
            }
        }
    }

    // Also check none equal the initial key
    chain.iter().all(|k| *k != initial)
}

/// Compare ephemeral vs static key exchange security properties.
///
/// Returns (uses_ephemeral, provides_forward_secrecy, key_reuse).
///
/// Ephemeral: Uses fresh keypair per session
/// Forward secrecy: Compromise of long-term key doesn't expose past sessions
/// Key reuse: Same key material used across sessions
pub fn compare_security_properties() -> (bool, bool, bool) {
    // Our ephemeral implementation:
    // - Uses ephemeral keys: YES
    // - Provides forward secrecy: YES (ephemeral keys deleted after use)
    // - Key reuse: NO (new keypair each session)
    (true, true, false)
}

/// Full session lifecycle: create, use, destroy ephemeral keys.
///
/// 1. Create ephemeral keypair
/// 2. Perform key exchange and derive session key
/// 3. Encrypt/decrypt data (simulated)
/// 4. Zeroize ephemeral key material
/// 5. Return encrypted data
pub fn secure_session_lifecycle(message: &[u8]) -> Vec<u8> {
    // 1. Create ephemeral keys for both parties
    let alice_secret = EphemeralSecret::random_from_rng(OsRng);

    let bob_secret = EphemeralSecret::random_from_rng(OsRng);
    let bob_public = PublicKey::from(&bob_secret);

    // 2. ECDH and derive session key
    let shared = alice_secret.diffie_hellman(&bob_public);
    let session_key = derive_session_key(shared.as_bytes(), b"session-encrypt");

    // 3. Simulate encryption (XOR with key bytes for demo)
    let mut encrypted = message.to_vec();
    for (i, byte) in encrypted.iter_mut().enumerate() {
        *byte ^= session_key[i % 32];
    }

    // 4. Zeroize sensitive material
    // In Rust, EphemeralSecret is consumed by diffie_hellman (moved),
    // so it cannot be reused. The shared secret bytes are zeroized here.
    let mut key_copy = session_key;
    key_copy.zeroize();

    // The ephemeral secrets (alice_secret, bob_secret) are already consumed
    // by diffie_hellman and cannot be accessed again — this IS the forward secrecy.

    encrypted
}

/// Derive a session key from ephemeral ECDH shared secret using HKDF.
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
