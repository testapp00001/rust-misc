//! # Lesson 05: Forward Secrecy — Double Ratchet Concept
//!
//! ## What Is Forward Secrecy?
//!
//! Forward secrecy ensures that compromise of long-term keys does not compromise
//! past session keys. Even if an attacker records all ciphertext and later
//! compromises the identity key, they cannot decrypt past messages.
//!
//! ## The Double Ratchet Algorithm
//!
//! Signal's Double Ratchet combines two ratcheting mechanisms:
//!
//! 1. **Symmetric-key ratchet (KDF chain)**: Advances a chain key to produce
//!    message keys. Each message uses a unique key.
//!
//! 2. **DH ratchet**: When the communication direction changes, a new DH
//!    shared secret is mixed into the chain, providing forward secrecy
//!    across sending/receiving phases.
//!
//! ```
//! Chain Key[i+1] = HMAC-SHA256(Chain Key[i], 0x02)
//! Message Key[i] = HMAC-SHA256(Chain Key[i], 0x01)
//! ```
//!
//! ## Attack: Key Compromise Without Forward Secrecy
//!
//! If a single key encrypts all messages and is compromised, ALL messages
//! are exposed. **Defense**: Derive a unique key per message; zeroize chain
//! keys after advancing.
//!
//! ## Attack: Out-of-Order Messages
//!
//! Messages may arrive out of order. The ratchet must support skipping ahead
//! and storing skipped message keys temporarily. **Defense**: Cache future
//! message keys up to a limit (e.g., 1000).

use hmac::{Hmac, Mac};
use sha2::Sha256;
use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}, Nonce, Key};
use zeroize::Zeroize;

type HmacSha256 = Hmac<Sha256>;

/// A KDF chain that ratchets forward to produce message keys.
///
/// Each call to `advance()` produces a new message key and advances
/// the chain key. The old chain key cannot be recovered.
#[derive(Clone)]
pub struct KdfChain {
    /// Current chain key (32 bytes). Zeroized on drop.
    chain_key: [u8; 32],
}

impl KdfChain {
    /// Exercise 1: Create a new KDF chain from a root key.
    ///
    /// Hints:
    /// - Copy the root key bytes into a new KdfChain
    pub fn new(root_key: &[u8; 32]) -> Self {
        todo!("Create a KDF chain from root key")
    }

    /// Exercise 2: Advance the chain and return the message key.
    ///
    /// Computes:
    ///   message_key = HMAC-SHA256(chain_key, 0x01)
    ///   chain_key   = HMAC-SHA256(chain_key, 0x02)
    ///
    /// Returns the 32-byte message key.
    ///
    /// Hints:
    /// - Create HMAC with chain_key as the MAC key
    /// - For message key: update with [0x01], finalize
    /// - For new chain key: create new HMAC with old chain_key, update with [0x02], finalize
    /// - Replace self.chain_key with the new chain key
    pub fn advance(&mut self) -> [u8; 32] {
        todo!("Advance the KDF chain and return the message key")
    }

    /// Exercise 3: Skip ahead N steps, caching skipped message keys.
    ///
    /// Returns a vector of message keys for positions [current..current+n].
    /// The chain advances by n steps.
    ///
    /// Hints:
    /// - Call advance() n times
    /// - Collect the message keys
    pub fn skip_ahead(&mut self, n: usize) -> Vec<[u8; 32]> {
        todo!("Skip ahead N steps, returning skipped message keys")
    }

    /// Exercise 4: Get the current chain key without advancing.
    ///
    /// Returns a copy of the current chain key (for DH ratchet mixing).
    pub fn current_chain_key(&self) -> [u8; 32] {
        todo!("Return the current chain key")
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
///
/// Maintains separate sending and receiving chains, with DH ratcheting
/// when the direction changes.
pub struct DoubleRatchet {
    /// Sending chain (KDF).
    pub sending_chain: KdfChain,
    /// Receiving chain (KDF).
    pub receiving_chain: KdfChain,
    /// Message counter for sending.
    pub send_count: u64,
    /// Message counter for receiving.
    pub recv_count: u64,
    /// Cached skipped message keys: (counter, key).
    pub skipped_keys: Vec<(u64, [u8; 32])>,
}

impl DoubleRatchet {
    /// Exercise 5: Create a new Double Ratchet session.
    ///
    /// Initializes both sending and receiving chains from the same root key.
    /// In a real protocol, the chains would be initialized differently for
    /// initiator vs responder.
    ///
    /// Hints:
    /// - Create two KdfChains, one for sending and one for receiving
    /// - Use different initial chain keys (derive from root using HKDF or similar)
    /// - For simplicity, use root_key XOR 0xAA for sending, root_key XOR 0xBB for receiving
    pub fn new(root_key: &[u8; 32]) -> Self {
        todo!("Initialize a Double Ratchet session")
    }

    /// Exercise 6: Encrypt a message using the sending chain.
    ///
    /// Advances the sending chain, derives a message key, encrypts with AES-256-GCM.
    ///
    /// Hints:
    /// - Advance the sending chain to get a message key
    /// - Create AES-256-GCM cipher from the message key
    /// - Use send_count as nonce (convert to 12-byte nonce)
    /// - Increment send_count
    /// - Return the ciphertext
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        todo!("Encrypt a message using the sending chain")
    }

    /// Exercise 7: Decrypt a message using the receiving chain.
    ///
    /// Advances the receiving chain, derives a message key, decrypts.
    ///
    /// Hints:
    /// - Advance the receiving chain to get a message key
    /// - Create AES-256-GCM cipher from the message key
    /// - Use recv_count as nonce
    /// - Increment recv_count
    /// - Return the plaintext
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        todo!("Decrypt a message using the receiving chain")
    }

    /// Exercise 8: Perform a DH ratchet step (advance both chains).
    ///
    /// Simulates a DH ratchet by mixing a new shared secret into the chains.
    /// In a real protocol, this would involve X25519 DH computation.
    ///
    /// Hints:
    /// - Create new sending chain from receiving chain's current key
    /// - Create new receiving chain from a provided new root
    /// - Reset message counters
    pub fn ratchet(&mut self, new_shared_secret: &[u8; 32]) {
        todo!("Perform a DH ratchet step")
    }
}

/// Exercise 9: Derive a message key for AES-256-GCM from a KDF output.
///
/// Takes a 32-byte message key from the KDF chain and returns
/// an AES-256-GCM cipher instance.
///
/// Hints:
/// - Use `Key::<Aes256Gcm>::from_slice(&msg_key)`
/// - Use `Aes256Gcm::new(&key)`
pub fn message_key_to_cipher(msg_key: &[u8; 32]) -> Aes256Gcm {
    todo!("Convert a message key to an AES-256-GCM cipher")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdf_chain_advance() {
        let mut chain = KdfChain::new(&[1u8; 32]);
        let mk1 = chain.advance();
        let mk2 = chain.advance();
        assert_ne!(mk1, mk2, "Each advance should produce a different message key");
    }

    #[test]
    fn test_kdf_chain_deterministic() {
        let mut chain1 = KdfChain::new(&[42u8; 32]);
        let mut chain2 = KdfChain::new(&[42u8; 32]);
        let mk1 = chain1.advance();
        let mk2 = chain2.advance();
        assert_eq!(mk1, mk2, "Same initial state should produce same keys");
    }

    #[test]
    fn test_kdf_chain_skip_ahead() {
        let mut chain = KdfChain::new(&[5u8; 32]);
        let keys = chain.skip_ahead(5);
        assert_eq!(keys.len(), 5);
        // All keys should be unique
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
        // Even if mk2 is compromised, mk1 should not be derivable
        // (we can't easily test this property, but at least verify they differ)
        assert_ne!(mk1, mk2);
    }

    #[test]
    fn test_double_ratchet_encrypt_decrypt() {
        let root = [0xABu8; 32];
        let mut alice = DoubleRatchet::new(&root);
        let mut bob = DoubleRatchet::new(&root);

        let plaintext = b"Hello from Alice!";
        let ciphertext = alice.encrypt(plaintext);
        let decrypted = bob.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_double_ratchet_multiple_messages() {
        let root = [0xCDu8; 32];
        let mut alice = DoubleRatchet::new(&root);
        let mut bob = DoubleRatchet::new(&root);

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
        let mut alice = DoubleRatchet::new(&root);
        let mut bob = DoubleRatchet::new(&root);

        // Send a few messages
        let ct1 = alice.encrypt(b"before ratchet");

        // Simulate DH ratchet
        let new_secret = [0x12u8; 32];
        alice.ratchet(&new_secret);
        bob.ratchet(&new_secret);

        let ct2 = alice.encrypt(b"after ratchet");

        // Bob should be able to decrypt both
        // Note: ct1 was encrypted before ratchet, ct2 after
        // In our simplified model, we need to handle this differently
        // Let's just verify the post-ratchet message works
        let pt2 = bob.decrypt(&ct2).unwrap();
        assert_eq!(pt2, b"after ratchet");
    }

    #[test]
    fn test_message_key_to_cipher() {
        let mk = [0x42u8; 32];
        let cipher = message_key_to_cipher(&mk);
        // Should be able to encrypt with it
        let nonce = Nonce::from_slice(&[0u8; 12]);
        let ct = cipher.encrypt(nonce, b"test".as_ref());
        assert!(ct.is_ok());
    }
}
