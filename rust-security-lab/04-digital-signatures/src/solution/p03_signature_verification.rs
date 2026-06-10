//! # Lesson 03: Signature Verification (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// A simple message structure with signature metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignedMessage {
    pub sender: String,
    pub payload: Vec<u8>,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Generate an Ed25519 keypair.
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    (
        signing_key.to_bytes().to_vec(),
        verifying_key.to_bytes().to_vec(),
    )
}

/// Create a SignedMessage with a valid signature.
pub fn create_signed_message(
    signing_key_bytes: &[u8],
    sender: &str,
    payload: &[u8],
) -> SignedMessage {
    let key_bytes: [u8; 32] = signing_key_bytes.try_into().expect("key must be 32 bytes");
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = VerifyingKey::from(&signing_key);

    let signature = signing_key.sign(payload);

    SignedMessage {
        sender: sender.to_string(),
        payload: payload.to_vec(),
        public_key: verifying_key.to_bytes().to_vec(),
        signature: signature.to_bytes().to_vec(),
    }
}

/// Verify a SignedMessage — CORRECTLY.
///
/// The verification is straightforward:
/// 1. Extract the public key and signature from the message
/// 2. Verify the signature over the payload
/// 3. Return the boolean result directly
///
/// NEVER invert the logic. NEVER return true on error.
pub fn verify_signed_message(msg: &SignedMessage) -> bool {
    let key_bytes: [u8; 32] = match msg.public_key.clone().try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let sig_bytes: [u8; 64] = match msg.signature.clone().try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    verifying_key.verify(&msg.payload, &signature).is_ok()
}

/// Verify a SignedMessage against an expected sender's public key.
///
/// Defense against substitution attacks: even if the attacker creates a
/// perfectly valid signed message, it won't match Alice's public key.
pub fn verify_signed_message_from(
    msg: &SignedMessage,
    expected_pubkey: &[u8],
) -> bool {
    if msg.public_key != expected_pubkey {
        return false;
    }
    verify_signed_message(msg)
}

/// Buggy verification — intentionally inverted logic.
///
/// This simulates the common mistake:
///   if verify fails → return true (WRONG!)
///   if verify succeeds → return false (WRONG!)
///
/// In real code, this bug would accept forged messages and reject valid ones.
pub fn buggy_verify(msg: &SignedMessage) -> bool {
    // BUG: inverted logic — returns true when signature is INVALID
    let key_bytes: [u8; 32] = match msg.public_key.clone().try_into() {
        Ok(b) => b,
        Err(_) => return true, // BUG: error treated as valid
    };
    let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
        Ok(k) => k,
        Err(_) => return true, // BUG: error treated as valid
    };
    let sig_bytes: [u8; 64] = match msg.signature.clone().try_into() {
        Ok(b) => b,
        Err(_) => return true, // BUG: error treated as valid
    };
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    // BUG: returns true when verify FAILS
    !verifying_key.verify(&msg.payload, &signature).is_ok()
}

/// Find messages with invalid signatures in a batch.
///
/// Returns indices of messages where verification fails.
pub fn find_invalid_messages(messages: &[SignedMessage]) -> Vec<usize> {
    messages
        .iter()
        .enumerate()
        .filter(|(_, msg)| !verify_signed_message(msg))
        .map(|(i, _)| i)
        .collect()
}

/// Create a message chain where each message references the previous.
///
/// This creates a hash chain that prevents reordering.
/// The signed data is: payload || previous_hash
/// where previous_hash is SHA-256 of the previous signature (or zeros for the first).
pub fn create_message_chain(
    signing_key_bytes: &[u8],
    payloads: &[&[u8]],
) -> Vec<SignedMessage> {
    let key_bytes: [u8; 32] = signing_key_bytes.try_into().expect("key must be 32 bytes");
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = VerifyingKey::from(&signing_key);

    let mut chain = Vec::new();
    let mut prev_hash = [0u8; 32];

    for payload in payloads {
        let mut data_to_sign = Vec::new();
        data_to_sign.extend_from_slice(payload);
        data_to_sign.extend_from_slice(&prev_hash);

        let signature = signing_key.sign(&data_to_sign);

        let sig_bytes = signature.to_bytes();
        let hash = Sha256::digest(&sig_bytes);
        prev_hash.copy_from_slice(&hash);

        chain.push(SignedMessage {
            sender: "chain".to_string(),
            payload: payload.to_vec(),
            public_key: verifying_key.to_bytes().to_vec(),
            signature: sig_bytes.to_vec(),
        });
    }

    chain
}

/// Verify a message chain.
///
/// Checks that:
/// 1. Each message has a valid signature
/// 2. Each message's signed data includes the correct previous hash
pub fn verify_message_chain(messages: &[SignedMessage]) -> bool {
    let mut prev_hash = [0u8; 32];

    for msg in messages {
        // Reconstruct the signed data
        let mut data_to_verify = Vec::new();
        data_to_verify.extend_from_slice(&msg.payload);
        data_to_verify.extend_from_slice(&prev_hash);

        // Verify signature over the reconstructed data
        if !verify_signature_data(msg, &data_to_verify) {
            return false;
        }

        // Compute hash of this signature for the next iteration
        let hash = Sha256::digest(&msg.signature);
        prev_hash.copy_from_slice(&hash);
    }

    true
}

/// Helper: verify a signature over specific data (not necessarily msg.payload).
fn verify_signature_data(msg: &SignedMessage, data: &[u8]) -> bool {
    let key_bytes: [u8; 32] = match msg.public_key.clone().try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let sig_bytes: [u8; 64] = match msg.signature.clone().try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    verifying_key.verify(data, &signature).is_ok()
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
        assert!(verify_signed_message(&msg));
        assert!(!verify_signed_message_from(&msg, &msg.public_key));
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
        chain[1].payload = b"EVIL".to_vec();
        assert!(!verify_message_chain(&chain));
    }
}
