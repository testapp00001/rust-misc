//! # Lesson 10: Full Protocol — Complete Secure Messenger
//!
//! ## Putting It All Together
//!
//! This lesson integrates all previous lessons into a complete secure messaging
//! protocol. A `SecureMessenger` handles:
//!
//! 1. **Identity management**: Generate and store identity keys
//! 2. **Key exchange**: Establish sessions using X3DH
//! 3. **Message encryption/decryption**: AES-256-GCM with proper nonce management
//! 4. **Forward secrecy**: Key rotation via KDF chains
//! 5. **Key verification**: Safety number computation
//! 6. **Message padding**: Traffic analysis resistance
//! 7. **Key backup**: Encrypted backup and restore
//!
//! ## Protocol State Machine
//!
//! ```
//! [No Session] → [Key Exchange] → [Active Session] → [Message Exchange]
//!      ↑                                                    ↓
//!      └──────────── [Session Reset / Key Rotation] ←───────┘
//! ```
//!
//! ## Attack: Protocol Downgrade
//!
//! An attacker might strip newer protocol features to force weaker encryption.
//! **Defense**: Negotiate protocol version explicitly; reject downgrades.
//!
//! ## Attack: Metadata Leakage
//!
//! Even with encrypted content, metadata (who talks to whom, when, how much)
//! reveals social graphs. **Defense**: Padding, cover traffic, onion routing.

use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}, Nonce, Key};
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::Zeroize;

type HmacSha256 = Hmac<Sha256>;

// ─── Data Types ─────────────────────────────────────────────

/// A user's identity (long-term signing key).
#[derive(Clone)]
pub struct UserIdentity {
    pub signing_key: Ed25519SigningKey,
}

/// A session between two users.
#[derive(Clone)]
pub struct Session {
    /// Sending KDF chain key.
    pub send_chain: [u8; 32],
    /// Receiving KDF chain key.
    pub recv_chain: [u8; 32],
    /// Send message counter.
    pub send_counter: u64,
    /// Receive message counter.
    pub recv_counter: u64,
}

/// A wire message containing encrypted content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
    /// Sender's identity (public key bytes).
    pub sender_id: Vec<u8>,
    /// Recipient's identity (public key bytes).
    pub recipient_id: Vec<u8>,
    /// Nonce (12 bytes).
    pub nonce: Vec<u8>,
    /// Padded + encrypted ciphertext.
    pub ciphertext: Vec<u8>,
    /// Protocol version.
    pub version: u32,
}

/// A complete secure messenger instance.
pub struct SecureMessenger {
    /// This user's identity.
    pub identity: UserIdentity,
    /// Active sessions: peer_id -> session.
    pub sessions: HashMap<Vec<u8>, Session>,
    /// Message history (for testing).
    pub sent_messages: Vec<WireMessage>,
    /// Message counter for nonce generation.
    pub nonce_counter: u64,
}

// ─── Exercise Stubs ─────────────────────────────────────────

impl UserIdentity {
    /// Exercise 1: Create a new user identity.
    ///
    /// Hints:
    /// - Generate Ed25519 keypair with OsRng
    pub fn new() -> Self {
        todo!("Create a new user identity with Ed25519 keypair")
    }

    /// Get the public key bytes (32 bytes).
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }
}

impl Session {
    /// Exercise 2: Create a new session from a shared secret.
    ///
    /// Derives separate sending and receiving chain keys from the shared secret.
    ///
    /// Hints:
    /// - Use HKDF with info="send_chain" and "recv_chain"
    /// - Initialize counters to 0
    pub fn from_shared_secret(shared_secret: &[u8; 32]) -> Self {
        todo!("Create a session from a shared secret")
    }

    /// Exercise 3: Derive a message key from a chain key.
    ///
    /// message_key = HMAC-SHA256(chain_key, 0x01)
    /// new_chain   = HMAC-SHA256(chain_key, 0x02)
    ///
    /// Returns (message_key, new_chain_key).
    pub fn step_chain(chain_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
        todo!("Step a KDF chain: derive message key and advance chain")
    }

    /// Exercise 4: Encrypt a message using the sending chain.
    ///
    /// 1. Step the sending chain to get a message key
    /// 2. Create AES-256-GCM cipher from the message key
    /// 3. Generate nonce from send_counter
    /// 4. Encrypt the plaintext
    /// 5. Update the sending chain key and increment counter
    pub fn encrypt_message(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        todo!("Encrypt a message using the sending chain")
    }

    /// Exercise 5: Decrypt a message using the receiving chain.
    ///
    /// 1. Step the receiving chain to get a message key
    /// 2. Create AES-256-GCM cipher from the message key
    /// 3. Decrypt using the nonce from the message
    /// 4. Update the receiving chain key and increment counter
    pub fn decrypt_message(&mut self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
        todo!("Decrypt a message using the receiving chain")
    }
}

impl SecureMessenger {
    /// Exercise 6: Create a new secure messenger.
    ///
    /// Hints:
    /// - Generate a new UserIdentity
    /// - Initialize empty sessions and history
    pub fn new() -> Self {
        todo!("Create a new SecureMessenger")
    }

    /// Exercise 7: Establish a session with a peer using a shared secret.
    ///
    /// In a real protocol, this would involve X3DH. Here we accept
    /// a pre-computed shared secret for simplicity.
    ///
    /// Hints:
    /// - Create a Session from the shared secret
    /// - Store it in the sessions map keyed by peer_id
    pub fn establish_session(&mut self, peer_id: Vec<u8>, shared_secret: [u8; 32]) {
        todo!("Establish a session with a peer")
    }

    /// Exercise 8: Send an encrypted message to a peer.
    ///
    /// 1. Look up the session for the peer
    /// 2. Pad the message (pad to next 64-byte block)
    /// 3. Encrypt with the session's sending chain
    /// 4. Create a WireMessage
    /// 5. Store in sent_messages
    pub fn send_message(&mut self, peer_id: &[u8], plaintext: &[u8]) -> Result<WireMessage, String> {
        todo!("Send an encrypted message to a peer")
    }

    /// Exercise 9: Receive and decrypt a message.
    ///
    /// 1. Look up the session by sender_id
    /// 2. Decrypt the ciphertext
    /// 3. Remove padding
    /// 4. Return the plaintext
    pub fn receive_message(&mut self, msg: &WireMessage) -> Result<Vec<u8>, String> {
        todo!("Receive and decrypt a message")
    }

    /// Exercise 10: Compute safety numbers for a peer.
    ///
    /// Computes SHA-256 of both identity keys (sorted) for verification.
    ///
    /// Hints:
    /// - Get own public key bytes
    /// - Get peer's public key bytes from peer_id
    /// - Sort them and hash together
    pub fn safety_number(&self, peer_id: &[u8]) -> [u8; 32] {
        todo!("Compute safety number for a peer")
    }
}

// ─── Helpers ────────────────────────────────────────────────

fn pad_to_block(data: &[u8], block_size: usize) -> Vec<u8> {
    let total = ((data.len() + 4) / block_size + 1) * block_size;
    let mut padded = Vec::with_capacity(total);
    padded.extend_from_slice(&(data.len() as u32).to_be_bytes());
    padded.extend_from_slice(data);
    padded.resize(total, 0);
    padded
}

fn unpad_block(data: &[u8]) -> Vec<u8> {
    if data.len() < 4 {
        return vec![];
    }
    let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if len + 4 > data.len() {
        return vec![];
    }
    data[4..4 + len].to_vec()
}

fn nonce_from_counter(counter: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&counter.to_le_bytes());
    nonce
}

// ─── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_identity() {
        let user = UserIdentity::new();
        let pk = user.public_key_bytes();
        assert_eq!(pk.len(), 32);
    }

    #[test]
    fn test_session_encrypt_decrypt() {
        let shared = [0x42u8; 32];
        let mut alice = Session::from_shared_secret(&shared);
        let mut bob = Session::from_shared_secret(&shared);

        let plaintext = b"Hello, Bob!";
        let ciphertext = alice.encrypt_message(plaintext).unwrap();
        let nonce = nonce_from_counter(0);
        let decrypted = bob.decrypt_message(&ciphertext, &nonce).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_session_forward_secrecy() {
        let shared = [0xABu8; 32];
        let mut alice = Session::from_shared_secret(&shared);
        let mut bob = Session::from_shared_secret(&shared);

        let ct1 = alice.encrypt_message(b"msg1").unwrap();
        let ct2 = alice.encrypt_message(b"msg2").unwrap();

        // Different messages should have different keys
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_secure_messenger_full_flow() {
        let mut alice = SecureMessenger::new();
        let mut bob = SecureMessenger::new();

        let shared_secret = [0xCDu8; 32];
        let bob_id = bob.identity.public_key_bytes().to_vec();

        alice.establish_session(bob_id.clone(), shared_secret);
        bob.establish_session(alice.identity.public_key_bytes().to_vec(), shared_secret);

        let wire = alice.send_message(&bob_id, b"Secret message!").unwrap();
        let received = bob.receive_message(&wire).unwrap();
        assert_eq!(received, b"Secret message!");
    }

    #[test]
    fn test_secure_messenger_multiple_messages() {
        let mut alice = SecureMessenger::new();
        let mut bob = SecureMessenger::new();

        let shared = [0xEFu8; 32];
        let bob_id = bob.identity.public_key_bytes().to_vec();
        let alice_id = alice.identity.public_key_bytes().to_vec();

        alice.establish_session(bob_id.clone(), shared);
        bob.establish_session(alice_id, shared);

        for i in 0..5 {
            let msg = format!("Message {}", i);
            let wire = alice.send_message(&bob_id, msg.as_bytes()).unwrap();
            let received = bob.receive_message(&wire).unwrap();
            assert_eq!(received, msg.as_bytes());
        }
    }

    #[test]
    fn test_safety_number_symmetric() {
        let mut alice = SecureMessenger::new();
        let bob = SecureMessenger::new();

        let bob_id = bob.identity.public_key_bytes().to_vec();
        let alice_id = alice.identity.public_key_bytes().to_vec();

        let sn_ab = alice.safety_number(&bob_id);
        let sn_ba = bob.safety_number(&alice_id);
        assert_eq!(sn_ab, sn_ba, "Safety numbers should be symmetric");
    }

    #[test]
    fn test_safety_number_unique() {
        let alice = SecureMessenger::new();
        let bob = SecureMessenger::new();
        let carol = SecureMessenger::new();

        let sn_ab = alice.safety_number(&bob.identity.public_key_bytes());
        let sn_ac = alice.safety_number(&carol.identity.public_key_bytes());
        assert_ne!(sn_ab, sn_ac, "Different peers should have different safety numbers");
    }

    #[test]
    fn test_pad_unpad_roundtrip() {
        let data = b"Hello, world!";
        let padded = pad_to_block(data, 64);
        let unpadded = unpad_block(&padded);
        assert_eq!(unpadded, data);
    }

    #[test]
    fn test_message_counter_increments() {
        let mut alice = SecureMessenger::new();
        let bob = SecureMessenger::new();
        let shared = [0x11u8; 32];
        let bob_id = bob.identity.public_key_bytes().to_vec();
        alice.establish_session(bob_id.clone(), shared);

        let _ = alice.send_message(&bob_id, b"msg1").unwrap();
        let _ = alice.send_message(&bob_id, b"msg2").unwrap();
        assert_eq!(alice.sent_messages.len(), 2);
    }
}
