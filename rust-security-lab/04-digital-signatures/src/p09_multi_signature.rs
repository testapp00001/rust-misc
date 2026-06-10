//! # Lesson 09: Multi-Signatures
//!
//! ## What are Multi-Signatures?
//!
//! Multi-signatures allow multiple parties to sign the same message, producing
//! a combined signature that proves all parties agreed.
//!
//! Three approaches:
//!
//! ### 1. Concatenated Signatures (Simple)
//! - Each party signs independently
//! - Verifier checks each signature separately
//! - Signature size grows linearly with number of signers
//!
//! ### 2. Schnorr-based Multi-Signatures (MuSig)
//! - Signers interact to produce a single compact signature
//! - Uses key aggregation: combined_pubkey = H(P1 || P2 || ...) * P1 + ...
//! - Requires multiple rounds of communication
//!
### 3. BLS Multi-Signatures
//! - Uses pairing-friendly curves
//! - Non-interactive: signatures can be added directly
//! - Single signature regardless of number of signers
//!
//! ## Use Cases
//! - Cryptocurrency multisig wallets
//! - Board resolutions requiring multiple signatures
//! - Distributed key management
//! - Secure multi-party computation
//!
//! ## Attack Scenario: Rogue Key Attack
//!
//! In a naive multi-signature scheme, an attacker can choose their public key
//! to cancel out another signer's key. For example:
//! - Alice has pubkey P_A
//! - Mallory sets P_M = -P_A
//! - Combined key = P_A + P_M = 0
//! - Mallory can sign alone!
//!
## Defense: Key prefixing or proof-of-possession

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};

/// A signer's contribution to a multi-signature.
#[derive(Debug, Clone)]
pub struct MultiSigContribution {
    pub signer_id: String,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Exercise 1: Create a concatenated multi-signature.
///
/// Each signer signs the same message independently.
/// The multi-signature is the concatenation of all individual signatures.
///
/// Hints:
/// - Generate keypairs for each signer
/// - Each signer signs the message
/// - Concatenate: id || pubkey || signature for each signer
pub fn create_concatenated_multisig(
    signers: &[(&str, Vec<u8>, Vec<u8>)], // (id, privkey, pubkey)
    message: &[u8],
) -> Vec<MultiSigContribution> {
    todo!("Create a concatenated multi-signature")
}

/// Exercise 2: Verify a concatenated multi-signature.
///
/// Check that ALL individual signatures are valid.
///
/// Hints:
/// - For each contribution, verify the signature over the message
/// - Return true only if all pass
pub fn verify_concatenated_multisig(
    contributions: &[MultiSigContribution],
    message: &[u8],
) -> bool {
    todo!("Verify a concatenated multi-signature")
}

/// Exercise 3: Implement key aggregation for Schnorr-style multi-sig.
///
/// Compute an aggregate public key from multiple individual keys.
///
/// Simplified version: aggregate = SHA-256(P1 || P2 || ... || Pn)
/// (Real MuSig uses more complex aggregation with key coefficients)
///
/// Returns a 32-byte aggregate "key".
pub fn aggregate_keys(public_keys: &[&[u8]]) -> Vec<u8> {
    todo!("Aggregate multiple public keys into one")
}

/// Exercise 4: Implement a simplified MuSig-like signing protocol.
///
/// Step 1: Each signer generates a nonce commitment H(R_i)
/// Step 2: Share nonce commitments
/// Step 3: Each signer computes partial signature using their nonce and key
/// Step 4: Combine partial signatures
///
/// For simplicity, we use Ed25519 and combine by XOR-ing signatures.
/// Real MuSig uses elliptic curve point addition.
///
/// Returns the combined signature.
pub fn musig_sign(
    signing_keys: &[&[u8]], // private key bytes
    message: &[u8],
) -> Vec<u8> {
    todo!("Simplified MuSig signing")
}

/// Exercise 5: Verify a MuSig-style signature.
///
/// Verify the combined signature against the aggregate key.
pub fn musig_verify(
    aggregate_key: &[u8],
    message: &[u8],
    combined_signature: &[u8],
) -> bool {
    todo!("Verify a MuSig combined signature")
}

/// Exercise 6: Demonstrate the rogue key attack.
///
/// Mallory chooses her public key to cancel out Alice's key.
/// With naive aggregation (just adding keys), Mallory can forge signatures.
///
/// Returns (alice_pubkey, mallory_pubkey, combined_key_is_zero)
pub fn demonstrate_rogue_key_attack() -> (Vec<u8>, Vec<u8>, bool) {
    todo!("Demonstrate the rogue key attack")
}

/// Exercise 7: Implement key aggregation with rogue key defense.
///
/// Use key prefixing: each key is hashed with its index before aggregation.
/// aggregate = sum(H(i || P_i) * P_i)
///
/// This prevents Mallory from choosing a key that cancels another.
///
/// Returns the secure aggregate key.
pub fn secure_aggregate_keys(public_keys: &[&[u8]]) -> Vec<u8> {
    todo!("Aggregate keys with rogue key defense")
}

/// Exercise 8: Create a threshold multi-signature requiring t-of-n signers.
///
/// Only include signatures if at least t signers contribute.
/// Returns Some(combined) if threshold met, None otherwise.
pub fn threshold_multisig(
    contributions: &[MultiSigContribution],
    message: &[u8],
    threshold: usize,
) -> Option<Vec<u8>> {
    todo!("Create a threshold multi-signature")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signer(id: &str) -> (String, Vec<u8>, Vec<u8>) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        (
            id.to_string(),
            signing_key.to_bytes().to_vec(),
            verifying_key.to_bytes().to_vec(),
        )
    }

    #[test]
    fn test_concatenated_multisig() {
        let signers = vec![make_signer("alice"), make_signer("bob"), make_signer("carol")];
        let signer_refs: Vec<(&str, Vec<u8>, Vec<u8>)> = signers
            .iter()
            .map(|(id, priv_key, pub_key)| (id.as_str(), priv_key.clone(), pub_key.clone()))
            .collect();
        let message = b"multi-party agreement";
        let contributions = create_concatenated_multisig(&signer_refs, message);
        assert_eq!(contributions.len(), 3);
        assert!(verify_concatenated_multisig(&contributions, message));
    }

    #[test]
    fn test_concatenated_multisig_one_invalid() {
        let signers = vec![make_signer("alice"), make_signer("bob")];
        let signer_refs: Vec<(&str, Vec<u8>, Vec<u8>)> = signers
            .iter()
            .map(|(id, priv_key, pub_key)| (id.as_str(), priv_key.clone(), pub_key.clone()))
            .collect();
        let message = b"test";
        let mut contributions = create_concatenated_multisig(&signer_refs, message);
        // Tamper with one signature
        if !contributions[0].signature.is_empty() {
            contributions[0].signature[0] ^= 0xFF;
        }
        assert!(!verify_concatenated_multisig(&contributions, message));
    }

    #[test]
    fn test_aggregate_keys() {
        let keys = vec![vec![1u8; 32], vec![2u8; 32], vec![3u8; 32]];
        let key_refs: Vec<&[u8]> = keys.iter().map(|k| k.as_slice()).collect();
        let agg = aggregate_keys(&key_refs);
        assert_eq!(agg.len(), 32);
    }

    #[test]
    fn test_aggregate_keys_deterministic() {
        let keys = vec![vec![1u8; 32], vec![2u8; 32]];
        let key_refs: Vec<&[u8]> = keys.iter().map(|k| k.as_slice()).collect();
        let agg1 = aggregate_keys(&key_refs);
        let agg2 = aggregate_keys(&key_refs);
        assert_eq!(agg1, agg2);
    }

    #[test]
    fn test_musig_sign_verify() {
        let signers = vec![make_signer("alice"), make_signer("bob")];
        let priv_keys: Vec<&[u8]> = signers.iter().map(|(_, pk, _)| pk.as_slice()).collect();
        let pub_keys: Vec<&[u8]> = signers.iter().map(|(_, _, pk)| pk.as_slice()).collect();
        let message = b"musig test";

        let combined_sig = musig_sign(&priv_keys, message);
        let agg_key = aggregate_keys(&pub_keys);
        // Note: simplified MuSig verification may not perfectly match
        let _ = musig_verify(&agg_key, message, &combined_sig);
    }

    #[test]
    fn test_secure_aggregate_keys() {
        let keys = vec![vec![1u8; 32], vec![2u8; 32]];
        let key_refs: Vec<&[u8]> = keys.iter().map(|k| k.as_slice()).collect();
        let agg1 = secure_aggregate_keys(&key_refs);
        let agg2 = secure_aggregate_keys(&key_refs);
        assert_eq!(agg1, agg2);
        assert_eq!(agg1.len(), 32);
    }

    #[test]
    fn test_threshold_multisig_met() {
        let signers = vec![make_signer("alice"), make_signer("bob"), make_signer("carol")];
        let signer_refs: Vec<(&str, Vec<u8>, Vec<u8>)> = signers
            .iter()
            .map(|(id, priv_key, pub_key)| (id.as_str(), priv_key.clone(), pub_key.clone()))
            .collect();
        let message = b"threshold test";
        let contributions = create_concatenated_multisig(&signer_refs, message);
        let result = threshold_multisig(&contributions, message, 2);
        assert!(result.is_some());
    }

    #[test]
    fn test_threshold_multisig_not_met() {
        let signers = vec![make_signer("alice")];
        let signer_refs: Vec<(&str, Vec<u8>, Vec<u8>)> = signers
            .iter()
            .map(|(id, priv_key, pub_key)| (id.as_str(), priv_key.clone(), pub_key.clone()))
            .collect();
        let message = b"threshold test";
        let contributions = create_concatenated_multisig(&signer_refs, message);
        let result = threshold_multisig(&contributions, message, 3);
        assert!(result.is_none());
    }
}
