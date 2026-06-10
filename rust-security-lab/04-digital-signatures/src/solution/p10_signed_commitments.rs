//! # Lesson 10: Signed Commitments (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use rand::Rng;
use sha2::{Sha256, Digest};

/// A commitment: the hash of (value || nonce).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Commitment {
    pub hash: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

/// The reveal phase data.
#[derive(Debug, Clone)]
pub struct Reveal {
    pub value: Vec<u8>,
    pub nonce: Vec<u8>,
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

/// Create a signed commitment to a value.
///
/// SECURITY PROPERTIES:
/// - Hiding: The commitment hash reveals nothing about the value (nonce provides entropy)
/// - Binding: Given the commitment, you cannot find a different (value, nonce) pair
///   that produces the same hash (collision resistance of SHA-256)
/// - Signed: The commitment is bound to the signer's identity
pub fn commit(signing_key_bytes: &[u8], value: &[u8]) -> (Commitment, Vec<u8>) {
    let mut rng = rand::thread_rng();
    let nonce: [u8; 32] = rng.gen();

    // Compute commitment hash
    let mut hasher = Sha256::new();
    hasher.update(value);
    hasher.update(&nonce);
    let hash = hasher.finalize().to_vec();

    // Sign the commitment hash
    let key_bytes: [u8; 32] = signing_key_bytes.try_into().expect("key must be 32 bytes");
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = VerifyingKey::from(&signing_key);
    let signature = signing_key.sign(&hash);

    (
        Commitment {
            hash,
            signature: signature.to_bytes().to_vec(),
            public_key: verifying_key.to_bytes().to_vec(),
        },
        nonce.to_vec(),
    )
}

/// Verify a commitment and reveal.
///
/// SECURITY CHECKS:
/// 1. Recompute Hash(value || nonce) and compare with commitment hash
/// 2. Verify the signature on the commitment hash
///
/// Only if both pass do we return the revealed value.
pub fn verify_and_reveal(commitment: &Commitment, reveal: &Reveal) -> Result<Vec<u8>, String> {
    // Recompute commitment hash
    let mut hasher = Sha256::new();
    hasher.update(&reveal.value);
    hasher.update(&reveal.nonce);
    let computed_hash = hasher.finalize().to_vec();

    // Check hash matches
    if computed_hash != commitment.hash {
        return Err("Commitment hash mismatch: value or nonce is incorrect".to_string());
    }

    // Verify signature on the commitment hash
    let key_bytes: [u8; 32] = commitment
        .public_key
        .clone()
        .try_into()
        .map_err(|_| "Invalid public key length")?;
    let verifying_key =
        VerifyingKey::from_bytes(&key_bytes).map_err(|_| "Invalid public key")?;
    let sig_bytes: [u8; 64] = commitment
        .signature
        .clone()
        .try_into()
        .map_err(|_| "Invalid signature length")?;
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);

    if verifying_key.verify(&commitment.hash, &signature).is_ok() {
        Ok(reveal.value.clone())
    } else {
        Err("Signature verification failed".to_string())
    }
}

/// Demonstrate the binding property of commitments.
///
/// Given a commitment, finding a different (value, nonce) that produces the
/// same hash would require finding a SHA-256 collision, which is computationally
/// infeasible (2^128 operations).
pub fn demonstrate_binding(
    signing_key_bytes: &[u8],
    value: &[u8],
) -> (Vec<u8>, Vec<u8>, bool) {
    let (commitment, nonce) = commit(signing_key_bytes, value);

    // Try to find a different value that works with the same nonce
    let forgery_attempt = b"different value";
    let bad_reveal = Reveal {
        value: forgery_attempt.to_vec(),
        nonce: nonce.clone(),
    };

    // Should fail — different value with same nonce produces different hash
    let binding_holds = verify_and_reveal(&commitment, &bad_reveal).is_err();

    (value.to_vec(), forgery_attempt.to_vec(), binding_holds)
}

/// Create a sealed-bid auction commitment.
///
/// The bid is committed as: bidder_id:amount
/// This ensures the bid is tied to the bidder's identity.
pub fn create_auction_bid(
    signing_key_bytes: &[u8],
    bidder_id: &str,
    bid_amount: u64,
) -> (Commitment, Vec<u8>) {
    let bid_data = format!("{}:{}", bidder_id, bid_amount);
    commit(signing_key_bytes, bid_data.as_bytes())
}

/// Resolve a sealed-bid auction.
///
/// Steps:
/// 1. Verify each commitment-reveal pair
/// 2. Parse bidder_id and amount from each valid reveal
/// 3. Find the highest bid
///
/// SECURITY: Invalid reveals are silently skipped. In a real auction,
/// you'd want to handle disputes explicitly.
pub fn resolve_auction(
    bids: &[(Commitment, Reveal)],
) -> Result<(String, u64, Vec<(String, u64)>), String> {
    let mut valid_bids: Vec<(String, u64)> = Vec::new();

    for (commitment, reveal) in bids {
        let revealed = match verify_and_reveal(commitment, reveal) {
            Ok(v) => v,
            Err(_) => continue, // Skip invalid bids
        };

        let bid_str = String::from_utf8_lossy(&revealed);
        let parts: Vec<&str> = bid_str.split(':').collect();
        if parts.len() != 2 {
            continue;
        }

        let bidder_id = parts[0].to_string();
        let amount = match parts[1].parse::<u64>() {
            Ok(a) => a,
            Err(_) => continue,
        };

        valid_bids.push((bidder_id, amount));
    }

    if valid_bids.is_empty() {
        return Err("No valid bids found".to_string());
    }

    let winner = valid_bids
        .iter()
        .max_by_key(|(_, amount)| amount)
        .unwrap()
        .clone();

    Ok((winner.0, winner.1, valid_bids))
}

/// Create a timestamped commitment.
///
/// The signature covers (commitment_hash || timestamp_bytes), binding
/// the commitment to a specific point in time.
pub fn timestamp_commitment(
    signing_key_bytes: &[u8],
    value: &[u8],
    timestamp: u64,
) -> (Commitment, Vec<u8>, u64) {
    let (mut commitment, nonce) = commit(signing_key_bytes, value);

    // Re-sign to include timestamp
    let key_bytes: [u8; 32] = signing_key_bytes.try_into().unwrap();
    let signing_key = SigningKey::from_bytes(&key_bytes);

    let mut data_to_sign = commitment.hash.clone();
    data_to_sign.extend_from_slice(&timestamp.to_be_bytes());
    let signature = signing_key.sign(&data_to_sign);
    commitment.signature = signature.to_bytes().to_vec();

    (commitment, nonce, timestamp)
}

/// Implement a fair coin toss protocol.
///
/// Both parties commit to a random bit. The result is XOR of both bits.
/// This ensures fairness: neither party can influence the outcome after
/// seeing the other's commitment.
pub fn fair_coin_toss(
    key1_bytes: &[u8],
    bit1: u8,
    key2_bytes: &[u8],
    bit2: u8,
) -> (Commitment, Vec<u8>, Commitment, Vec<u8>, u8) {
    let (c1, n1) = commit(key1_bytes, &[bit1]);
    let (c2, n2) = commit(key2_bytes, &[bit2]);
    let result = bit1 ^ bit2;
    (c1, n1, c2, n2, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_and_reveal() {
        let (priv_key, pub_key) = generate_keypair();
        let value = b"secret bid: $100";
        let (commitment, nonce) = commit(&priv_key, value);

        assert_eq!(commitment.public_key, pub_key);

        let reveal = Reveal {
            value: value.to_vec(),
            nonce,
        };
        let revealed = verify_and_reveal(&commitment, &reveal).unwrap();
        assert_eq!(revealed, value);
    }

    #[test]
    fn test_wrong_value_fails() {
        let (priv_key, _) = generate_keypair();
        let value = b"original";
        let (commitment, nonce) = commit(&priv_key, value);

        let bad_reveal = Reveal {
            value: b"forged".to_vec(),
            nonce,
        };
        assert!(verify_and_reveal(&commitment, &bad_reveal).is_err());
    }

    #[test]
    fn test_wrong_nonce_fails() {
        let (priv_key, _) = generate_keypair();
        let value = b"original";
        let (commitment, _nonce) = commit(&priv_key, value);

        let bad_reveal = Reveal {
            value: value.to_vec(),
            nonce: vec![0u8; 32],
        };
        assert!(verify_and_reveal(&commitment, &bad_reveal).is_err());
    }

    #[test]
    fn test_binding_property() {
        let (priv_key, _) = generate_keypair();
        let value = b"committed value";
        let (_, _, binding_holds) = demonstrate_binding(&priv_key, value);
        assert!(binding_holds);
    }

    #[test]
    fn test_auction_single_bid() {
        let (priv_key, _) = generate_keypair();
        let (commitment, nonce) = create_auction_bid(&priv_key, "alice", 100);

        let reveal = Reveal {
            value: b"alice:100".to_vec(),
            nonce,
        };
        let revealed = verify_and_reveal(&commitment, &reveal).unwrap();
        assert!(String::from_utf8_lossy(&revealed).contains("alice"));
    }

    #[test]
    fn test_auction_resolution() {
        let (key1, _) = generate_keypair();
        let (key2, _) = generate_keypair();
        let (key3, _) = generate_keypair();

        let (c1, n1) = create_auction_bid(&key1, "alice", 100);
        let (c2, n2) = create_auction_bid(&key2, "bob", 150);
        let (c3, n3) = create_auction_bid(&key3, "carol", 120);

        let bids = vec![
            (c1, Reveal { value: b"alice:100".to_vec(), nonce: n1 }),
            (c2, Reveal { value: b"bob:150".to_vec(), nonce: n2 }),
            (c3, Reveal { value: b"carol:120".to_vec(), nonce: n3 }),
        ];

        let result = resolve_auction(&bids);
        assert!(result.is_ok());
        let (winner, amount, _) = result.unwrap();
        assert_eq!(winner, "bob");
        assert_eq!(amount, 150);
    }

    #[test]
    fn test_timestamp_commitment() {
        let (priv_key, _) = generate_keypair();
        let value = b"document hash";
        let timestamp = 1700000000u64;
        let (commitment, _nonce, ts) = timestamp_commitment(&priv_key, value, timestamp);
        assert_eq!(ts, timestamp);
        assert!(!commitment.hash.is_empty());
    }

    #[test]
    fn test_fair_coin_toss() {
        let (key1, _) = generate_keypair();
        let (key2, _) = generate_keypair();

        let (c1, n1, c2, n2, result) = fair_coin_toss(&key1, 1, &key2, 0);
        assert_eq!(result, 1);

        let reveal1 = Reveal { value: vec![1], nonce: n1 };
        let reveal2 = Reveal { value: vec![0], nonce: n2 };
        assert!(verify_and_reveal(&c1, &reveal1).is_ok());
        assert!(verify_and_reveal(&c2, &reveal2).is_ok());
    }
}
