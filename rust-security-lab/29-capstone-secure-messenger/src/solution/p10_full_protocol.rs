//! # Lesson 10: Full Protocol — Complete Secure Messenger (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ed25519_dalek::SigningKey as Ed25519SigningKey;
use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}, Nonce, Key};
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

// ─── Data Types ─────────────────────────────────────────────

/// A user's identity.
#[derive(Clone)]
pub struct UserIdentity {
    pub signing_key: Ed25519SigningKey,
}

/// A session between two users.
#[derive(Clone)]
pub struct Session {
    pub send_chain: [u8; 32],
    pub recv_chain: [u8; 32],
    pub send_counter: u64,
    pub recv_counter: u64,
}

/// A wire message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
    pub sender_id: Vec<u8>,
    pub recipient_id: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub version: u32,
}

/// A complete secure messenger.
pub struct SecureMessenger {
    pub identity: UserIdentity,
    pub sessions: HashMap<Vec<u8>, Session>,
    pub sent_messages: Vec<WireMessage>,
    pub nonce_counter: u64,
}

// ─── Implementations ────────────────────────────────────────

impl UserIdentity {
    /// Create a new user identity.
    pub fn new() -> Self {
        Self {
            signing_key: Ed25519SigningKey::generate(&mut OsRng),
        }
    }

    /// Get the public key bytes.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }
}

impl Session {
    /// Create a new session from a shared secret.
    ///
    /// The `initiator` flag determines which derived chain is used for
    /// sending vs receiving. Alice (initiator) and Bob (responder) must
    /// use opposite values so Alice's send_chain matches Bob's recv_chain.
    pub fn from_shared_secret(shared_secret: &[u8; 32], initiator: bool) -> Self {
        let hk = Hkdf::<Sha256>::new(None, shared_secret);

        let mut chain_a = [0u8; 32];
        let mut chain_b = [0u8; 32];
        hk.expand(b"chain_a", &mut chain_a).expect("HKDF expand failed");
        hk.expand(b"chain_b", &mut chain_b).expect("HKDF expand failed");

        let (send_chain, recv_chain) = if initiator {
            (chain_a, chain_b)
        } else {
            (chain_b, chain_a)
        };

        Self {
            send_chain,
            recv_chain,
            send_counter: 0,
            recv_counter: 0,
        }
    }

    /// Derive a message key from a chain key.
    ///
    /// message_key = HMAC(chain_key, 0x01)
    /// new_chain   = HMAC(chain_key, 0x02)
    pub fn step_chain(chain_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
        let mut mac_mk = <HmacSha256 as Mac>::new_from_slice(chain_key).expect("HMAC failed");
        mac_mk.update(&[0x01]);
        let message_key: [u8; 32] = mac_mk.finalize().into_bytes().into();

        let mut mac_ck = <HmacSha256 as Mac>::new_from_slice(chain_key).expect("HMAC failed");
        mac_ck.update(&[0x02]);
        let new_chain: [u8; 32] = mac_ck.finalize().into_bytes().into();

        (message_key, new_chain)
    }

    /// Encrypt a message using the sending chain.
    pub fn encrypt_message(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let (msg_key, new_chain) = Self::step_chain(&self.send_chain);
        self.send_chain = new_chain;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&msg_key));
        let nonce = nonce_from_counter(self.send_counter);
        self.send_counter += 1;

        cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|e| format!("Encryption failed: {:?}", e))
    }

    /// Decrypt a message using the receiving chain.
    pub fn decrypt_message(&mut self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
        let (msg_key, new_chain) = Self::step_chain(&self.recv_chain);
        self.recv_chain = new_chain;
        self.recv_counter += 1;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&msg_key));
        cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| format!("Decryption failed: {:?}", e))
    }
}

impl SecureMessenger {
    /// Create a new secure messenger.
    pub fn new() -> Self {
        Self {
            identity: UserIdentity::new(),
            sessions: HashMap::new(),
            sent_messages: Vec::new(),
            nonce_counter: 0,
        }
    }

    /// Establish a session with a peer.
    ///
    /// The `initiator` flag determines chain ordering.
    pub fn establish_session(&mut self, peer_id: Vec<u8>, shared_secret: [u8; 32], initiator: bool) {
        let session = Session::from_shared_secret(&shared_secret, initiator);
        self.sessions.insert(peer_id, session);
    }

    /// Send an encrypted message to a peer.
    pub fn send_message(&mut self, peer_id: &[u8], plaintext: &[u8]) -> Result<WireMessage, String> {
        let session = self.sessions.get_mut(peer_id)
            .ok_or("No session with peer")?;

        // Pad to 64-byte block
        let padded = pad_to_block(plaintext, 64);

        // Encrypt
        let ciphertext = session.encrypt_message(&padded)?;

        // Generate nonce for the wire message
        let nonce = nonce_from_counter(self.nonce_counter);
        self.nonce_counter += 1;

        let msg = WireMessage {
            sender_id: self.identity.public_key_bytes().to_vec(),
            recipient_id: peer_id.to_vec(),
            nonce: nonce.to_vec(),
            ciphertext,
            version: 1,
        };

        self.sent_messages.push(msg.clone());
        Ok(msg)
    }

    /// Receive and decrypt a message.
    pub fn receive_message(&mut self, msg: &WireMessage) -> Result<Vec<u8>, String> {
        let session = self.sessions.get_mut(&msg.sender_id)
            .ok_or("No session with sender")?;

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&msg.nonce);

        let padded = session.decrypt_message(&msg.ciphertext, &nonce)?;
        Ok(unpad_block(&padded))
    }

    /// Compute safety number for a peer.
    pub fn safety_number(&self, peer_id: &[u8]) -> [u8; 32] {
        let own_key = self.identity.public_key_bytes();

        // Sort keys for deterministic ordering
        let (first, second) = if own_key[..] <= *peer_id {
            (&own_key[..], peer_id)
        } else {
            (peer_id, &own_key[..])
        };

        let mut hasher = Sha256::new();
        hasher.update(first);
        hasher.update(second);
        hasher.finalize().into()
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
        let mut alice = Session::from_shared_secret(&shared, true);
        let mut bob = Session::from_shared_secret(&shared, false);

        let plaintext = b"Hello, Bob!";
        let ciphertext = alice.encrypt_message(plaintext).unwrap();
        let nonce = nonce_from_counter(0);
        let decrypted = bob.decrypt_message(&ciphertext, &nonce).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_session_forward_secrecy() {
        let shared = [0xABu8; 32];
        let mut alice = Session::from_shared_secret(&shared, true);
        let bob = Session::from_shared_secret(&shared, false);

        let ct1 = alice.encrypt_message(b"msg1").unwrap();
        let ct2 = alice.encrypt_message(b"msg2").unwrap();
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_secure_messenger_full_flow() {
        let mut alice = SecureMessenger::new();
        let mut bob = SecureMessenger::new();

        let shared_secret = [0xCDu8; 32];
        let bob_id = bob.identity.public_key_bytes().to_vec();

        alice.establish_session(bob_id.clone(), shared_secret, true);
        bob.establish_session(alice.identity.public_key_bytes().to_vec(), shared_secret, false);

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

        alice.establish_session(bob_id.clone(), shared, true);
        bob.establish_session(alice_id, shared, false);

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
        assert_eq!(sn_ab, sn_ba);
    }

    #[test]
    fn test_safety_number_unique() {
        let alice = SecureMessenger::new();
        let bob = SecureMessenger::new();
        let carol = SecureMessenger::new();

        let sn_ab = alice.safety_number(&bob.identity.public_key_bytes());
        let sn_ac = alice.safety_number(&carol.identity.public_key_bytes());
        assert_ne!(sn_ab, sn_ac);
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
        alice.establish_session(bob_id.clone(), shared, true);

        let _ = alice.send_message(&bob_id, b"msg1").unwrap();
        let _ = alice.send_message(&bob_id, b"msg2").unwrap();
        assert_eq!(alice.sent_messages.len(), 2);
    }
}
