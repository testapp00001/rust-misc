//! # Lesson 07: Group Messaging — Sender Keys
//!
//! ## The Group Messaging Problem
//!
//! In a 1:1 conversation, each message is encrypted with a shared secret between
//! two parties. In a group of N members, naive approaches require:
//! - N*(N-1)/2 pairwise sessions, or
//! - Encrypting each message N-1 times (once per recipient)
//!
//! **Sender keys** solve this: each sender has a key that all group members share.
//! A message is encrypted once and sent to everyone.
//!
//! ## Sender Key Protocol
//!
//! ```
//! 1. Group creator generates a random group_id
//! 2. Each member generates a sender_key (random 32-byte secret)
//! 3. When a member joins, they receive all existing sender keys
//!    (encrypted pairwise using existing sessions)
//! 4. To send: encrypt with sender_key + counter
//! 5. To receive: look up the sender's key, derive message key, decrypt
//! ```
//!
//! ## Message Key Derivation
//!
//! ```
//! message_key = HKDF(sender_key || counter_bytes || "group_msg")
//! ciphertext = AES-256-GCM(message_key, plaintext)
//! ```
//!
//! ## Attack: Sender Key Compromise
//!
//! If a sender's key is compromised, all past messages from that sender
//! in the group are exposed. **Defense**: Rotate sender keys periodically;
//! use per-message ratcheting within the sender key chain.
//!
//! ## Attack: Key Replacement
//!
//! An attacker replaces a sender key for a group member. All future
//! messages to that member would be encrypted with the attacker's key.
//! **Defense**: Authenticate sender key distribution with pairwise sessions.

use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}, Nonce, Key};
use hkdf::Hkdf;
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A sender key used by one group member to encrypt messages.
#[derive(Clone, Debug)]
pub struct SenderKey {
    /// The base secret (32 bytes).
    pub secret: [u8; 32],
    /// Current message counter (incremented per message).
    pub counter: u64,
}

/// A group session managing sender keys for all members.
#[derive(Clone)]
pub struct GroupSession {
    /// Unique group identifier.
    pub group_id: Vec<u8>,
    /// Map from member_id to their sender key.
    pub sender_keys: HashMap<Vec<u8>, SenderKey>,
}

/// An encrypted group message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    /// The group this message belongs to.
    pub group_id: Vec<u8>,
    /// The sender's member ID.
    pub sender_id: Vec<u8>,
    /// The counter value used for this message.
    pub counter: u64,
    /// The nonce (12 bytes).
    pub nonce: Vec<u8>,
    /// The ciphertext (includes auth tag).
    pub ciphertext: Vec<u8>,
}

impl SenderKey {
    /// Exercise 1: Generate a new random sender key.
    ///
    /// Hints:
    /// - Use `rand::random::<[u8; 32]>()` for the secret
    /// - Start counter at 0
    pub fn generate() -> Self {
        todo!("Generate a new random sender key")
    }

    /// Exercise 2: Derive a message key from the sender key and counter.
    ///
    /// Uses HKDF-SHA256 with:
    /// - IKM: sender_secret || counter_bytes
    /// - Info: "group_message_key"
    ///
    /// Returns a 32-byte message key.
    ///
    /// Hints:
    /// - Concatenate secret (32 bytes) + counter.to_le_bytes() (8 bytes)
    /// - Feed into HKDF with info="group_message_key"
    /// - Extract 32 bytes
    pub fn derive_message_key(&self) -> [u8; 32] {
        todo!("Derive message key from sender key and counter")
    }

    /// Exercise 3: Advance the sender key counter.
    ///
    /// Increments the counter by 1. The counter ensures each message
    /// uses a unique key even with the same sender secret.
    pub fn advance(&mut self) {
        todo!("Increment the sender key counter")
    }
}

impl GroupSession {
    /// Exercise 4: Create a new group session.
    ///
    /// Hints:
    /// - Generate a random group_id (16 bytes)
    /// - Initialize empty sender_keys map
    pub fn new() -> Self {
        todo!("Create a new group session")
    }

    /// Exercise 5: Add a member to the group with their sender key.
    ///
    /// Hints:
    /// - Insert (member_id, sender_key) into the HashMap
    pub fn add_member(&mut self, member_id: Vec<u8>, sender_key: SenderKey) {
        todo!("Add a member with their sender key")
    }

    /// Exercise 6: Encrypt a message for the group.
    ///
    /// Looks up the sender's key, derives the message key, encrypts,
    /// and advances the counter.
    ///
    /// Hints:
    /// - Get sender's key from the map
    /// - Derive message key
    /// - Create AES-256-GCM cipher from message key
    /// - Use counter as nonce basis
    /// - Advance the counter
    pub fn encrypt(&mut self, sender_id: &[u8], plaintext: &[u8]) -> Result<GroupMessage, String> {
        todo!("Encrypt a message for the group")
    }

    /// Exercise 7: Decrypt a group message.
    ///
    /// Looks up the sender's key by sender_id, derives the message key,
    /// and decrypts.
    ///
    /// Hints:
    /// - Get sender's key from the map
    /// - Note: we use the counter from the message, not the current counter
    /// - Derive message key using the message's counter
    /// - Decrypt with AES-256-GCM
    pub fn decrypt(&self, msg: &GroupMessage) -> Result<Vec<u8>, String> {
        todo!("Decrypt a group message")
    }

    /// Exercise 8: Rotate a member's sender key (post-compromise recovery).
    ///
    /// Generates a new sender key for the member and distributes it.
    /// Returns the new key so it can be sent to other members.
    ///
    /// Hints:
    /// - Generate a new SenderKey
    /// - Replace the old key in the map
    /// - Return the new key
    pub fn rotate_key(&mut self, member_id: &[u8]) -> SenderKey {
        todo!("Rotate a member's sender key")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_key_generate() {
        let key = SenderKey::generate();
        assert_eq!(key.counter, 0);
        assert_ne!(key.secret, [0u8; 32]);
    }

    #[test]
    fn test_sender_key_derive_unique() {
        let mut key = SenderKey::generate();
        let mk0 = key.derive_message_key();
        key.advance();
        let mk1 = key.derive_message_key();
        assert_ne!(mk0, mk1, "Different counters should produce different message keys");
    }

    #[test]
    fn test_sender_key_derive_deterministic() {
        let key = SenderKey { secret: [42u8; 32], counter: 5 };
        let mk1 = key.derive_message_key();
        let mk2 = key.derive_message_key();
        assert_eq!(mk1, mk2);
    }

    #[test]
    fn test_group_session_create() {
        let session = GroupSession::new();
        assert!(!session.group_id.is_empty());
        assert!(session.sender_keys.is_empty());
    }

    #[test]
    fn test_group_encrypt_decrypt_single_member() {
        let mut session = GroupSession::new();
        let alice_id = b"alice".to_vec();
        let alice_key = SenderKey::generate();
        session.add_member(alice_id.clone(), alice_key);

        let plaintext = b"Hello group!";
        let msg = session.encrypt(&alice_id, plaintext).unwrap();
        let decrypted = session.decrypt(&msg).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_group_multiple_members() {
        let mut session = GroupSession::new();
        let alice_id = b"alice".to_vec();
        let bob_id = b"bob".to_vec();

        session.add_member(alice_id.clone(), SenderKey::generate());
        session.add_member(bob_id.clone(), SenderKey::generate());

        let msg_alice = session.encrypt(&alice_id, b"From Alice").unwrap();
        let msg_bob = session.encrypt(&bob_id, b"From Bob").unwrap();

        let pt_alice = session.decrypt(&msg_alice).unwrap();
        let pt_bob = session.decrypt(&msg_bob).unwrap();

        assert_eq!(pt_alice, b"From Alice");
        assert_eq!(pt_bob, b"From Bob");
    }

    #[test]
    fn test_group_counter_increments() {
        let mut session = GroupSession::new();
        let alice_id = b"alice".to_vec();
        session.add_member(alice_id.clone(), SenderKey::generate());

        let msg1 = session.encrypt(&alice_id, b"msg1").unwrap();
        let msg2 = session.encrypt(&alice_id, b"msg2").unwrap();
        assert_eq!(msg1.counter, 0);
        assert_eq!(msg2.counter, 1);
    }

    #[test]
    fn test_group_rotate_key() {
        let mut session = GroupSession::new();
        let alice_id = b"alice".to_vec();
        session.add_member(alice_id.clone(), SenderKey::generate());

        // Encrypt with old key
        let msg1 = session.encrypt(&alice_id, b"before rotation").unwrap();

        // Rotate
        let new_key = session.rotate_key(&alice_id);

        // Encrypt with new key
        let msg2 = session.encrypt(&alice_id, b"after rotation").unwrap();

        // Both should decrypt
        assert_eq!(session.decrypt(&msg1).unwrap(), b"before rotation");
        assert_eq!(session.decrypt(&msg2).unwrap(), b"after rotation");
    }

    #[test]
    fn test_group_unknown_sender() {
        let session = GroupSession::new();
        let msg = GroupMessage {
            group_id: vec![],
            sender_id: b"unknown".to_vec(),
            counter: 0,
            nonce: vec![0u8; 12],
            ciphertext: vec![],
        };
        assert!(session.decrypt(&msg).is_err());
    }
}
