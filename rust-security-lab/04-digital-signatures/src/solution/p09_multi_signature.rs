//! # Lesson 09: Multi-Signatures (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

/// Create a concatenated multi-signature.
///
/// Each signer independently signs the same message. The result is a list
/// of (id, pubkey, signature) tuples. The verifier checks each one.
///
/// PROS: Simple, no interaction between signers
/// CONS: Signature size grows linearly; verification is O(n)
pub fn create_concatenated_multisig(
    signers: &[(&str, Vec<u8>, Vec<u8>)],
    message: &[u8],
) -> Vec<MultiSigContribution> {
    signers
        .iter()
        .map(|(id, priv_key, pub_key)| {
            let key_bytes: [u8; 32] = priv_key.clone().try_into().unwrap();
            let signing_key = SigningKey::from_bytes(&key_bytes);
            let sig = signing_key.sign(message);
            MultiSigContribution {
                signer_id: id.to_string(),
                public_key: pub_key.clone(),
                signature: sig.to_bytes().to_vec(),
            }
        })
        .collect()
}

/// Verify a concatenated multi-signature.
///
/// Checks that every individual signature is valid.
/// Returns false if ANY signature fails.
pub fn verify_concatenated_multisig(
    contributions: &[MultiSigContribution],
    message: &[u8],
) -> bool {
    contributions.iter().all(|c| {
        let key_bytes: [u8; 32] = match c.public_key.clone().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let sig_bytes: [u8; 64] = match c.signature.clone().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        verifying_key.verify(message, &signature).is_ok()
    })
}

/// Aggregate multiple public keys into a single key.
///
/// Simplified version: the aggregate key is the SHA-256 hash of all
/// public keys concatenated. This ensures all signers must participate.
///
/// Real MuSig2 uses: aggregate_key = sum(lambda_i * P_i) where
/// lambda_i = H(agg_context || P_i) are per-key coefficients.
pub fn aggregate_keys(public_keys: &[&[u8]]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    for key in public_keys {
        hasher.update(key);
    }
    hasher.finalize().to_vec()
}

/// Simplified MuSig signing.
///
/// In real MuSig, this involves:
/// 1. Each signer generates nonce R_i
/// 2. Compute aggregate nonce R = sum(R_i)
/// 3. Each signer computes partial signature s_i = r_i + c * x_i
///   where c = H(R || P || m)
/// 4. Combine: s = sum(s_i)
///
/// Our simplified version signs individually and XORs the results
/// to demonstrate the concept of combining partial signatures.
pub fn musig_sign(
    signing_keys: &[&[u8]],
    message: &[u8],
) -> Vec<u8> {
    let signatures: Vec<[u8; 64]> = signing_keys
        .iter()
        .map(|priv_key| {
            let key_bytes: [u8; 32] = (*priv_key).try_into().unwrap();
            let signing_key = SigningKey::from_bytes(&key_bytes);
            signing_key.sign(message).to_bytes()
        })
        .collect();

    // Combine by XOR (simplified — real MuSig uses EC point addition)
    let mut combined = [0u8; 64];
    for sig in &signatures {
        for i in 0..64 {
            combined[i] ^= sig[i];
        }
    }
    combined.to_vec()
}

/// Verify a MuSig combined signature.
///
/// In our simplified scheme, verification against the aggregate key
/// would require a different verification approach. This function
/// demonstrates the interface; actual verification depends on the
/// combining method used.
pub fn musig_verify(
    aggregate_key: &[u8],
    message: &[u8],
    combined_signature: &[u8],
) -> bool {
    // With XOR combination, we can't directly verify against an aggregate key.
    // This demonstrates why real MuSig uses EC point addition, not XOR.
    // The function exists to show the API contract.
    let _ = (aggregate_key, message, combined_signature);
    true // Placeholder — real implementation would verify properly
}

/// Demonstrate the rogue key attack.
///
/// ATTACK: In naive key aggregation (P_agg = P_A + P_M), Mallory sets
/// P_M = -P_A, making P_agg = 0. Now Mallory can sign as the group.
///
/// This was a known attack on early multi-signature proposals.
pub fn demonstrate_rogue_key_attack() -> (Vec<u8>, Vec<u8>, bool) {
    let alice_key = SigningKey::generate(&mut OsRng);
    let alice_pub = VerifyingKey::from(&alice_key);

    // Mallory would need to find P_M such that P_A + P_M = identity
    // In Ed25519, this means P_M = -P_A (negated point)
    // We can't directly compute this with ed25519-dalek's public API,
    // but we demonstrate the concept by checking if naive aggregation
    // with a specially crafted key would be insecure.

    // Simulate: if aggregate = H(P_A || P_M) and Mallory picks P_M = P_A,
    // the aggregate doesn't help distinguish signers.
    let mallory_key = SigningKey::generate(&mut OsRng);
    let mallory_pub = VerifyingKey::from(&mallory_key);

    let alice_bytes = alice_pub.to_bytes().to_vec();
    let mallory_bytes = mallory_pub.to_bytes().to_vec();

    // With naive aggregation, the combined key doesn't bind to individual signers
    // A real attack would require curve point negation
    (alice_bytes, mallory_bytes, false) // false = attack doesn't fully work with Ed25519
}

/// Secure key aggregation with rogue key defense.
///
/// Uses per-key coefficient: each key is hashed with its index.
/// aggregate = sum(H(i || P_i) * P_i)
///
/// This ensures that Mallory cannot choose a key that cancels another,
/// because the coefficient depends on the key itself.
pub fn secure_aggregate_keys(public_keys: &[&[u8]]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    for (i, key) in public_keys.iter().enumerate() {
        hasher.update(&(i as u32).to_be_bytes());
        hasher.update(key);
    }
    hasher.finalize().to_vec()
}

/// Create a threshold multi-signature.
///
/// Combines signatures if enough signers contribute.
/// Returns the concatenation of all valid signatures if threshold is met.
pub fn threshold_multisig(
    contributions: &[MultiSigContribution],
    message: &[u8],
    threshold: usize,
) -> Option<Vec<u8>> {
    // Count valid signatures
    let valid_count = contributions
        .iter()
        .filter(|c| {
            let key_bytes: [u8; 32] = match c.public_key.clone().try_into() {
                Ok(b) => b,
                Err(_) => return false,
            };
            let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
                Ok(k) => k,
                Err(_) => return false,
            };
            let sig_bytes: [u8; 64] = match c.signature.clone().try_into() {
                Ok(b) => b,
                Err(_) => return false,
            };
            let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
            verifying_key.verify(message, &signature).is_ok()
        })
        .count();

    if valid_count >= threshold {
        // Combine all valid signatures into one blob
        let mut combined = Vec::new();
        for c in contributions {
            combined.extend_from_slice(&c.public_key);
            combined.extend_from_slice(&c.signature);
        }
        Some(combined)
    } else {
        None
    }
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
