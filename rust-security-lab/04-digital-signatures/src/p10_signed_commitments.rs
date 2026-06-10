//! # Lesson 10: Signed Commitments
//!
//! ## What is a Signed Commitment?
//!
//! A commitment scheme lets you commit to a value without revealing it,
//! then later reveal it. Combined with digital signatures, this creates
//! a powerful primitive: you can prove you knew something at a certain time.
//!
## Commit-Reveal Protocol
//!
## Phase 1 (Commit):
//! commitment = Hash(value || nonce)
//! Sign the commitment
//!
## Phase 2 (Reveal):
//! Reveal (value, nonce)
//! Verify: Hash(value || nonce) == commitment
//! Verify: signature on commitment is valid
//!
## Use Cases
//!
## - Sealed-bid auctions: Bidders commit to bids, then reveal simultaneously
## - Voting: Voters commit to choices, then reveal after voting closes
## - Timestamping: Prove a document existed at a certain time
## - Nonce-based authentication: Commit to a nonce before revealing
## - Fair coin toss: Both parties commit to their choices, then reveal
//!
## Attack Scenario: Commitment Binding Failure
//!
## If the commitment scheme is not binding, the committer can open a
## different value than what they committed to. This breaks the entire
## protocol.
//!
## For example, in an auction:
## 1. Alice commits to bid $100
## 2. Mallory sees Alice's commitment
## 3. Mallory commits to bid $101
## 4. If Alice can change her commitment, she can bid $102 after seeing Mallory's
##
## Defense: Use collision-resistant hash functions for commitments

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};

/// A commitment: the hash of (value || nonce).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Commitment {
    pub hash: Vec<u8>,       // SHA-256(value || nonce)
    pub signature: Vec<u8>,  // Signature over the hash
    pub public_key: Vec<u8>, // Committer's public key
}

/// The reveal phase data.
#[derive(Debug, Clone)]
pub struct Reveal {
    pub value: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Exercise 1: Generate a keypair for commitment signing.
///
/// Hints:
/// - Use `SigningKey::generate(&mut OsRng)`
/// - Return (private_key_bytes, public_key_bytes)
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    todo!("Generate an Ed25519 keypair")
}

/// Exercise 2: Create a commitment to a value.
///
/// Steps:
/// 1. Generate a random nonce (32 bytes)
/// 2. Compute commitment = SHA-256(value || nonce)
/// 3. Sign the commitment hash
/// 4. Return the Commitment struct and the nonce (for later reveal)
///
/// Hints:
/// - Use `rand::thread_rng().gen::<[u8; 32]>()` for nonce
/// - Hash: `Sha256::digest(&[value, nonce].concat())`
pub fn commit(signing_key_bytes: &[u8], value: &[u8]) -> (Commitment, Vec<u8>) {
    todo!("Create a signed commitment")
}

/// Exercise 3: Verify a commitment and reveal.
///
/// Check:
/// 1. SHA-256(reveal.value || reveal.nonce) == commitment.hash
/// 2. Signature on commitment.hash is valid
///
/// Returns the committed value if everything checks out.
pub fn verify_and_reveal(commitment: &Commitment, reveal: &Reveal) -> Result<Vec<u8>, String> {
    todo!("Verify commitment and extract the revealed value")
}

/// Exercise 4: Demonstrate the binding property.
///
/// Show that given a commitment, you CANNOT find a different (value, nonce)
/// pair that produces the same hash. This is the binding property.
///
/// Returns: (original_value, attempted_forgery_value, binding_holds)
pub fn demonstrate_binding(
    signing_key_bytes: &[u8],
    value: &[u8],
) -> (Vec<u8>, Vec<u8>, bool) {
    todo!("Demonstrate commitment binding property")
}

/// Exercise 5: Create a sealed-bid auction commitment.
///
/// Each bidder commits to their bid amount. The commitment includes:
/// - Bid amount (as string bytes)
/// - Bidder ID
/// - Random nonce
///
/// Returns (commitment, nonce) for the bidder.
pub fn create_auction_bid(
    signing_key_bytes: &[u8],
    bidder_id: &str,
    bid_amount: u64,
) -> (Commitment, Vec<u8>) {
    todo!("Create a sealed auction bid commitment")
}

/// Exercise 6: Resolve an auction.
///
/// Given all bids (commitments + reveals), find the winner.
///
/// Steps:
/// 1. Verify each commitment-reveal pair
/// 2. Find the highest valid bid
/// 3. Return (winner_id, winning_bid, all_valid_bids)
pub fn resolve_auction(
    bids: &[(Commitment, Reveal)],
) -> Result<(String, u64, Vec<(String, u64)>), String> {
    todo!("Resolve a sealed-bid auction")
}

/// Exercise 7: Create a timestamped commitment.
///
/// Combine a commitment with a timestamp to prove the value existed at
/// a certain time. The signature covers (commitment_hash || timestamp).
///
/// Returns the signed timestamp proof.
pub fn timestamp_commitment(
    signing_key_bytes: &[u8],
    value: &[u8],
    timestamp: u64,
) -> (Commitment, Vec<u8>, u64) {
    todo!("Create a timestamped commitment")
}

/// Exercise 8: Implement a fair coin toss protocol.
///
/// Two parties each commit to a random bit (0 or 1).
/// The coin toss result is the XOR of both bits.
///
/// Returns (commitment1, nonce1, commitment2, nonce2, result).
pub fn fair_coin_toss(
    key1_bytes: &[u8],
    bit1: u8,
    key2_bytes: &[u8],
    bit2: u8,
) -> (Commitment, Vec<u8>, Commitment, Vec<u8>, u8) {
    todo!("Implement fair coin toss with commitments")
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
            nonce: vec![0u8; 32], // wrong nonce
        };
        assert!(verify_and_reveal(&commitment, &bad_reveal).is_err());
    }

    #[test]
    fn test_binding_property() {
        let (priv_key, _) = generate_keypair();
        let value = b"committed value";
        let (_, _, binding_holds) = demonstrate_binding(&priv_key, value);
        assert!(binding_holds, "Commitment should be binding");
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
        // XOR of 1 and 0 should be 1
        assert_eq!(result, 1);

        // Verify both commitments
        let reveal1 = Reveal { value: vec![1], nonce: n1 };
        let reveal2 = Reveal { value: vec![0], nonce: n2 };
        assert!(verify_and_reveal(&c1, &reveal1).is_ok());
        assert!(verify_and_reveal(&c2, &reveal2).is_ok());
    }
}
