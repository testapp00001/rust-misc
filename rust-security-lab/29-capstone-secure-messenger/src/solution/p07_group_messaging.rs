//! # Lesson 07: Group Messaging — Sender Keys (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}, Nonce, Key};
use hkdf::Hkdf;
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A sender key used by one group member to encrypt messages.
#[derive(Clone, Debug)]
pub struct SenderKey {
    pub secret: [u8; 32],
    pub counter: u64,
}

/// A group session managing sender keys for all members.
#[derive(Clone)]
pub struct GroupSession {
    pub group_id: Vec<u8>,
    pub sender_keys: HashMap<Vec<u8>, SenderKey>,
    /// Old sender keys for decrypting messages from before key rotation.
    pub old_sender_keys: HashMap<Vec<u8>, Vec<SenderKey>>,
}

/// An encrypted group message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub group_id: Vec<u8>,
    pub sender_id: Vec<u8>,
    pub counter: u64,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

impl SenderKey {
    /// Generate a new random sender key.
    pub fn generate() -> Self {
        Self {
            secret: rand::random::<[u8; 32]>(),
            counter: 0,
        }
    }

    /// Derive a message key from the sender key and counter.
    pub fn derive_message_key(&self) -> [u8; 32] {
        let mut ikm = Vec::with_capacity(40);
        ikm.extend_from_slice(&self.secret);
        ikm.extend_from_slice(&self.counter.to_le_bytes());

        let hk = Hkdf::<Sha256>::new(None, &ikm);
        let mut message_key = [0u8; 32];
        hk.expand(b"group_message_key", &mut message_key)
            .expect("HKDF expand failed");
        message_key
    }

    /// Advance the sender key counter.
    pub fn advance(&mut self) {
        self.counter += 1;
    }
}

impl GroupSession {
    /// Create a new group session.
    pub fn new() -> Self {
        Self {
            group_id: rand::random::<[u8; 16]>().to_vec(),
            sender_keys: HashMap::new(),
            old_sender_keys: HashMap::new(),
        }
    }

    /// Add a member to the group with their sender key.
    pub fn add_member(&mut self, member_id: Vec<u8>, sender_key: SenderKey) {
        self.sender_keys.insert(member_id, sender_key);
    }

    /// Encrypt a message for the group.
    pub fn encrypt(&mut self, sender_id: &[u8], plaintext: &[u8]) -> Result<GroupMessage, String> {
        let key = self.sender_keys.get_mut(sender_id)
            .ok_or("Unknown sender")?;

        let msg_key = key.derive_message_key();
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&msg_key));

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[..8].copy_from_slice(&key.counter.to_le_bytes());

        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce_bytes), plaintext)
            .map_err(|e| format!("Encryption failed: {:?}", e))?;

        let msg = GroupMessage {
            group_id: self.group_id.clone(),
            sender_id: sender_id.to_vec(),
            counter: key.counter,
            nonce: nonce_bytes.to_vec(),
            ciphertext,
        };

        key.advance();
        Ok(msg)
    }

    /// Decrypt a group message.
    pub fn decrypt(&self, msg: &GroupMessage) -> Result<Vec<u8>, String> {
        // Try current key first
        if let Some(key) = self.sender_keys.get(&msg.sender_id) {
            if let Ok(plaintext) = self.try_decrypt_with_key(msg, key) {
                return Ok(plaintext);
            }
        }

        // Try old keys (for messages from before key rotation)
        if let Some(old_keys) = self.old_sender_keys.get(&msg.sender_id) {
            for old_key in old_keys {
                if let Ok(plaintext) = self.try_decrypt_with_key(msg, old_key) {
                    return Ok(plaintext);
                }
            }
        }

        Err("Decryption failed: no matching key found".to_string())
    }

    fn try_decrypt_with_key(&self, msg: &GroupMessage, key: &SenderKey) -> Result<Vec<u8>, aes_gcm::Error> {
        let temp_key = SenderKey {
            secret: key.secret,
            counter: msg.counter,
        };
        let msg_key = temp_key.derive_message_key();
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&msg_key));
        let nonce = Nonce::from_slice(&msg.nonce);
        cipher.decrypt(nonce, msg.ciphertext.as_ref())
    }

    /// Rotate a member's sender key.
    pub fn rotate_key(&mut self, member_id: &[u8]) -> SenderKey {
        // Save old key for decrypting past messages
        if let Some(old_key) = self.sender_keys.get(member_id) {
            self.old_sender_keys
                .entry(member_id.to_vec())
                .or_insert_with(Vec::new)
                .push(old_key.clone());
        }
        let new_key = SenderKey::generate();
        self.sender_keys.insert(member_id.to_vec(), new_key.clone());
        new_key
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
        assert_ne!(mk0, mk1);
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

        let msg1 = session.encrypt(&alice_id, b"before rotation").unwrap();
        let _new_key = session.rotate_key(&alice_id);
        let msg2 = session.encrypt(&alice_id, b"after rotation").unwrap();

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
