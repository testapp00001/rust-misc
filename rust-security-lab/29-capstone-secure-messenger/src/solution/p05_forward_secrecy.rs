//! # Lesson 05: Forward Secrecy — Double Ratchet Concept (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}, Nonce, Key};
use zeroize::Zeroize;

type HmacSha256 = Hmac<Sha256>;

/// A KDF chain that ratchets forward to produce message keys.
#[derive(Clone)]
pub struct KdfChain {
    chain_key: [u8; 32],
}

impl KdfChain {
    /// Create a new KDF chain from a root key.
    pub fn new(root_key: &[u8; 32]) -> Self {
        Self {
            chain_key: *root_key,
        }
    }

    /// Advance the chain and return the message key.
    ///
    /// message_key = HMAC-SHA256(chain_key, 0x01)
    /// chain_key   = HMAC-SHA256(chain_key, 0x02)
    pub fn advance(&mut self) -> [u8; 32] {
        // Derive message key
        let mut mac_mk = <HmacSha256 as Mac>::new_from_slice(&self.chain_key)
            .expect("HMAC key creation failed");
        mac_mk.update(&[0x01]);
        let message_key: [u8; 32] = mac_mk.finalize().into_bytes().into();

        // Derive new chain key
        let mut mac_ck = <HmacSha256 as Mac>::new_from_slice(&self.chain_key)
            .expect("HMAC key creation failed");
        mac_ck.update(&[0x02]);
        self.chain_key = mac_ck.finalize().into_bytes().into();

        message_key
    }

    /// Skip ahead N steps, caching skipped message keys.
    pub fn skip_ahead(&mut self, n: usize) -> Vec<[u8; 32]> {
        (0..n).map(|_| self.advance()).collect()
    }

    /// Get the current chain key without advancing.
    pub fn current_chain_key(&self) -> [u8; 32] {
        self.chain_key
    }
}

impl Zeroize for KdfChain {
    fn zeroize(&mut self) {
        self.chain_key.zeroize();
    }
}

impl Drop for KdfChain {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// A simplified Double Ratchet session.
pub struct DoubleRatchet {
    pub sending_chain: KdfChain,
    pub receiving_chain: KdfChain,
    pub send_count: u64,
    pub recv_count: u64,
    pub skipped_keys: Vec<(u64, [u8; 32])>,
    /// Whether this instance is the initiator (sends on chain A, receives on chain B).
    is_initiator: bool,
}

impl DoubleRatchet {
    /// Create a new Double Ratchet session.
    ///
    /// The `initiator` flag determines which chain is used for sending vs receiving.
    /// Initiator: send_chain = root_key XOR 0xAA, recv_chain = root_key XOR 0xBB
    /// Responder: send_chain = root_key XOR 0xBB, recv_chain = root_key XOR 0xAA
    pub fn new(root_key: &[u8; 32], initiator: bool) -> Self {
        let mut chain_a = *root_key;
        for b in chain_a.iter_mut() {
            *b ^= 0xAA;
        }
        let mut chain_b = *root_key;
        for b in chain_b.iter_mut() {
            *b ^= 0xBB;
        }
        let (send_chain, recv_chain) = if initiator {
            (KdfChain::new(&chain_a), KdfChain::new(&chain_b))
        } else {
            (KdfChain::new(&chain_b), KdfChain::new(&chain_a))
        };
        Self {
            sending_chain: send_chain,
            receiving_chain: recv_chain,
            send_count: 0,
            recv_count: 0,
            skipped_keys: Vec::new(),
            is_initiator: initiator,
        }
    }

    /// Encrypt a message using the sending chain.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        let msg_key = self.sending_chain.advance();
        let cipher = message_key_to_cipher(&msg_key);
        let nonce = nonce_from_counter(self.send_count);
        self.send_count += 1;
        cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .expect("encryption failed")
    }

    /// Decrypt a message using the receiving chain.
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        let msg_key = self.receiving_chain.advance();
        let cipher = message_key_to_cipher(&msg_key);
        let nonce = nonce_from_counter(self.recv_count);
        self.recv_count += 1;
        cipher
            .decrypt(Nonce::from_slice(&nonce), ciphertext)
            .map_err(|e| format!("Decryption failed: {:?}", e))
    }

    /// Perform a DH ratchet step.
    ///
    /// Resets both chains from a new shared secret, preserving the
    /// initiator/responder chain ordering.
    pub fn ratchet(&mut self, new_shared_secret: &[u8; 32]) {
        let mut chain_a = *new_shared_secret;
        for b in chain_a.iter_mut() {
            *b ^= 0xAA;
        }
        let mut chain_b = *new_shared_secret;
        for b in chain_b.iter_mut() {
            *b ^= 0xBB;
        }

        if self.is_initiator {
            self.sending_chain = KdfChain::new(&chain_a);
            self.receiving_chain = KdfChain::new(&chain_b);
        } else {
            self.sending_chain = KdfChain::new(&chain_b);
            self.receiving_chain = KdfChain::new(&chain_a);
        }

        self.send_count = 0;
        self.recv_count = 0;
    }
}

/// Derive a message key for AES-256-GCM from a KDF output.
pub fn message_key_to_cipher(msg_key: &[u8; 32]) -> Aes256Gcm {
    let key = Key::<Aes256Gcm>::from_slice(msg_key);
    Aes256Gcm::new(key)
}

fn nonce_from_counter(counter: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&counter.to_le_bytes());
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdf_chain_advance() {
        let mut chain = KdfChain::new(&[1u8; 32]);
        let mk1 = chain.advance();
        let mk2 = chain.advance();
        assert_ne!(mk1, mk2);
    }

    #[test]
    fn test_kdf_chain_deterministic() {
        let mut chain1 = KdfChain::new(&[42u8; 32]);
        let mut chain2 = KdfChain::new(&[42u8; 32]);
        let mk1 = chain1.advance();
        let mk2 = chain2.advance();
        assert_eq!(mk1, mk2);
    }

    #[test]
    fn test_kdf_chain_skip_ahead() {
        let mut chain = KdfChain::new(&[5u8; 32]);
        let keys = chain.skip_ahead(5);
        assert_eq!(keys.len(), 5);
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j]);
            }
        }
    }

    #[test]
    fn test_kdf_chain_forward_secrecy() {
        let mut chain = KdfChain::new(&[7u8; 32]);
        let mk1 = chain.advance();
        let mk2 = chain.advance();
        assert_ne!(mk1, mk2);
    }

    #[test]
    fn test_double_ratchet_encrypt_decrypt() {
        let root = [0xABu8; 32];
        let mut alice = DoubleRatchet::new(&root, true);
        let mut bob = DoubleRatchet::new(&root, false);

        let plaintext = b"Hello from Alice!";
        let ciphertext = alice.encrypt(plaintext);
        let decrypted = bob.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_double_ratchet_multiple_messages() {
        let root = [0xCDu8; 32];
        let mut alice = DoubleRatchet::new(&root, true);
        let mut bob = DoubleRatchet::new(&root, false);

        for i in 0..5 {
            let msg = format!("Message {}", i);
            let ct = alice.encrypt(msg.as_bytes());
            let pt = bob.decrypt(&ct).unwrap();
            assert_eq!(pt, msg.as_bytes());
        }
    }

    #[test]
    fn test_double_ratchet_ratchet_step() {
        let root = [0xEFu8; 32];
        let mut alice = DoubleRatchet::new(&root, true);
        let mut bob = DoubleRatchet::new(&root, false);

        let new_secret = [0x12u8; 32];
        alice.ratchet(&new_secret);
        bob.ratchet(&new_secret);

        let ct2 = alice.encrypt(b"after ratchet");
        let pt2 = bob.decrypt(&ct2).unwrap();
        assert_eq!(pt2, b"after ratchet");
    }

    #[test]
    fn test_message_key_to_cipher() {
        let mk = [0x42u8; 32];
        let cipher = message_key_to_cipher(&mk);
        let nonce = Nonce::from_slice(&[0u8; 12]);
        let ct = cipher.encrypt(nonce, b"test".as_ref());
        assert!(ct.is_ok());
    }
}
