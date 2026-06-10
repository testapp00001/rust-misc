//! # Lesson 03: Signature Verification
//!
//! ## Why Verification is Hard
//!
//! The most common security mistake with digital signatures is not verifying
//! them correctly. Common failures:
//!
//! 1. **Not verifying at all**: Accepting any input as "signed"
//! 2. **Verifying against the wrong key**: Using a hardcoded key instead of the sender's
//! 3. **Accepting errors as valid**: Treating verification failures as success
//! 4. **Not checking the full message**: Verifying a hash but accepting a different message
//! 5. **Confusion attacks**: Attacker provides their own public key
//!
//! ## Attack Scenario: Verification Bypass
//!
//! A server checks `if verify(key, msg, sig)` but the `verify` function returns
//! `Ok(())` on success and `Err(...)` on failure. A developer writes:
//!
//! ```ignore
//! if let Err(_) = verify(key, msg, sig) { return Ok(()); }
//! ```
//!
//! This accepts INVALID signatures! The logic is inverted.
//!
//! ## Defense
//!
//! 1. Always check the boolean result explicitly
//! 2. Use type-safe APIs that force you to handle verification
//! 3. Never proceed with unverified data
//! 4. Test with known-good and known-bad signatures

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// A simple message structure with signature metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignedMessage {
    pub sender: String,
    pub payload: Vec<u8>,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Exercise 1: Generate a keypair for use in the verification exercises.
///
/// Hints:
/// - Use `SigningKey::generate(&mut OsRng)`
/// - Return (signing_key_bytes, verifying_key_bytes)
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    todo!("Generate Ed25519 keypair")
}

/// Exercise 2: Create a SignedMessage.
///
/// Sign the payload and package it into a SignedMessage struct.
///
/// Hints:
/// - Sign the payload bytes
/// - Include the sender name, payload, public key, and signature
pub fn create_signed_message(
    signing_key_bytes: &[u8],
    sender: &str,
    payload: &[u8],
) -> SignedMessage {
    todo!("Create a SignedMessage with proper signature")
}

/// Exercise 3: Verify a SignedMessage — CORRECTLY.
///
/// This is the most important function in this lesson.
///
/// Verification steps (all must pass):
/// 1. Verify the signature over the payload using the embedded public key
/// 2. Return true ONLY if verification succeeds
///
/// Common mistakes to avoid:
/// - Returning true on error
/// - Not checking the result
/// - Verifying the wrong data (e.g., signing key bytes instead of payload)
pub fn verify_signed_message(msg: &SignedMessage) -> bool {
    todo!("Verify a SignedMessage correctly")
}

/// Exercise 4: Verify a SignedMessage against an EXPECTED sender.
///
/// This prevents an attacker from substituting their own signed message.
/// The attack: Mallory creates a valid signed message claiming to be Alice.
/// Defense: Check that the public key matches the expected sender's key.
///
/// Hints:
/// - First verify the signature (as before)
/// - Then check that the public key matches expected_pubkey
pub fn verify_signed_message_from(
    msg: &SignedMessage,
    expected_pubkey: &[u8],
) -> bool {
    todo!("Verify signature AND that it came from the expected sender")
}

/// Exercise 5: Buggy verification (for attack demonstration).
///
/// This function intentionally contains a logic bug that allows
/// invalid signatures to pass. Find and explain the bug.
///
/// The "bug" is: we return true when the signature DOESN'T match,
/// and false when it does. This simulates a common developer mistake
/// of inverting the verification logic.
///
/// Implement this function with the bug intentional, so tests can
/// demonstrate the vulnerability.
pub fn buggy_verify(_msg: &SignedMessage) -> bool {
    todo!("Implement a buggy verification (intentionally wrong) for demonstration")
}

/// Exercise 6: Verify a batch of messages, returning indices of invalid ones.
///
/// Given a vector of SignedMessages, verify each one and return the
/// indices of messages that fail verification.
///
/// Hints:
/// - Iterate with `.enumerate()`
/// - Verify each message
/// - Collect indices where verification fails
pub fn find_invalid_messages(messages: &[SignedMessage]) -> Vec<usize> {
    todo!("Find all messages with invalid signatures")
}

/// Exercise 7: Create a message chain where each message references the previous.
///
/// This prevents reordering attacks. Each message includes the hash of the
/// previous message's signature, creating an append-only log.
///
/// Returns a vector of SignedMessages where:
/// - messages[0] signs (payload || [0u8; 32])
/// - messages[i] signs (payload || hash(messages[i-1].signature))
///
/// Hints:
/// - Use `sha2::Sha256` and `sha2::Digest` for hashing
/// - For the first message, use 32 zero bytes as the "previous hash"
pub fn create_message_chain(
    signing_key_bytes: &[u8],
    payloads: &[&[u8]],
) -> Vec<SignedMessage> {
    todo!("Create a chain of signed messages")
}

/// Exercise 8: Verify a message chain.
///
/// Check that:
/// 1. Each message has a valid signature
/// 2. Each message correctly references the previous one
///
/// Returns true only if the entire chain is valid.
pub fn verify_message_chain(messages: &[SignedMessage]) -> bool {
    todo!("Verify the integrity of a message chain")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify() {
        let (priv_key, _) = generate_keypair();
        let msg = create_signed_message(&priv_key, "alice", b"hello");
        assert!(verify_signed_message(&msg));
    }

    #[test]
    fn test_tampered_payload() {
        let (priv_key, _) = generate_keypair();
        let mut msg = create_signed_message(&priv_key, "alice", b"hello");
        msg.payload = b"evil".to_vec();
        assert!(!verify_signed_message(&msg));
    }

    #[test]
    fn test_tampered_sender() {
        let (priv_key, _) = generate_keypair();
        let mut msg = create_signed_message(&priv_key, "alice", b"hello");
        msg.sender = "mallory".to_string();
        // Signature is still valid over the payload — but sender changed
        // This is why verify_signed_message_from exists
        assert!(verify_signed_message(&msg)); // sig is still valid
        assert!(!verify_signed_message_from(&msg, &msg.public_key)); // different check
    }

    #[test]
    fn test_verify_from_expected_sender() {
        let (priv_key, pub_key) = generate_keypair();
        let msg = create_signed_message(&priv_key, "alice", b"hello");
        assert!(verify_signed_message_from(&msg, &pub_key));
    }

    #[test]
    fn test_verify_from_wrong_sender() {
        let (priv_key, _) = generate_keypair();
        let (_, wrong_pub) = generate_keypair();
        let msg = create_signed_message(&priv_key, "alice", b"hello");
        assert!(!verify_signed_message_from(&msg, &wrong_pub));
    }

    #[test]
    fn test_find_invalid_messages() {
        let (priv_key, _) = generate_keypair();
        let mut messages = Vec::new();
        for i in 0..5 {
            messages.push(create_signed_message(
                &priv_key,
                "alice",
                format!("msg {}", i).as_bytes(),
            ));
        }
        // Tamper with messages 1 and 3
        messages[1].payload = b"tampered".to_vec();
        messages[3].signature = vec![0u8; 64];
        let invalid = find_invalid_messages(&messages);
        assert_eq!(invalid, vec![1, 3]);
    }

    #[test]
    fn test_message_chain_valid() {
        let (priv_key, _) = generate_keypair();
        let payloads: Vec<&[u8]> = vec![b"first", b"second", b"third"];
        let chain = create_message_chain(&priv_key, &payloads);
        assert_eq!(chain.len(), 3);
        assert!(verify_message_chain(&chain));
    }

    #[test]
    fn test_message_chain_tampered() {
        let (priv_key, _) = generate_keypair();
        let payloads: Vec<&[u8]> = vec![b"first", b"second", b"third"];
        let mut chain = create_message_chain(&priv_key, &payloads);
        // Tamper with the middle message
        chain[1].payload = b"EVIL".to_vec();
        assert!(!verify_message_chain(&chain));
    }
}
