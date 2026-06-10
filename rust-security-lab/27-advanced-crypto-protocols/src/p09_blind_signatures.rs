//! # Lesson 09: Blind Signatures -- Sign Without Seeing
//!
//! ## What is a Blind Signature?
//!
//! A blind signature allows a signer to sign a message without seeing its content.
//! The workflow:
//!
//! 1. **Blinding**: The user transforms the message using a random blinding factor
//! 2. **Signing**: The signer signs the blinded message
//! 3. **Unblinding**: The user removes the blinding to get a valid signature on the original
//!
//! After unblinding, the signature is indistinguishable from a direct signature on the
//! original message. The signer cannot link the signed message to the blinding session.
//!
//! ## RSA Blind Signing (Simplified)
//!
//! Signer has RSA key (n, e, d).
//!
//! 1. User wants signature on message m:
//!    - Pick random blinding factor r where gcd(r, n) = 1
//!    - Compute blinded message: m' = m * r^e mod n
//!    - Send m' to signer
//!
//! 2. Signer computes: s' = (m')^d mod n
//!
//! 3. User unblinds: s = s' * r^(-1) mod n
//!    Now s = m^d mod n is a valid RSA signature on m
//!
//! ## Applications
//!
//! - **E-voting**: Authority signs blinded ballots. Voter unblinds to get a signed ballot.
//!   The authority cannot trace which ballot belongs to which voter.
//! - **Digital cash**: Bank signs a blinded coin. User unblinds. Merchant verifies the signature.
//!   Bank cannot trace the coin to the user.
//! - **Anonymous credentials**: Prove you have a valid credential without revealing which one.
//!
//! ## Attack: Blinding Factor Reuse
//!
//! If the same blinding factor r is used for two different messages m1 and m2:
//! - m1' = m1 * r^e, m2' = m2 * r^e
//! - m1'/m2' = m1/m2 (the blinding cancels)
//! - This links the two blinded messages, breaking unlinkability
//!
//! ## Attack: Blinding the Wrong Thing
//!
//! If the signer doesn't verify that the blinded message has a valid structure,
//! the user can blind arbitrary values, getting the signer to unknowingly sign
//! something harmful. The signer should use a "hash-then-sign" approach with
//! a message format the user cannot control.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// The signer's public key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerPublicKey {
    pub n: i64,
    pub e: i64,
}

/// The signer's private key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerPrivateKey {
    pub n: i64,
    pub d: i64,
    pub p: i64,
    pub q: i64,
}

/// A blinded message sent from user to signer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindedMessage {
    /// The blinded message value
    pub blinded_value: i64,
}

/// A blind signature (the signer's response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindSignature {
    pub blinded_signature: i64,
}

/// An unblinded signature that can be verified against the original message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnblindedSignature {
    pub signature: i64,
}

/// User's blinding state (needed for unblinding).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindingState {
    /// The blinding factor r
    pub blinding_factor: i64,
    /// The original message hash
    pub message_hash: i64,
}

/// Exercise 1: Generate RSA key pair for blind signing.
///
/// Given primes p and q:
/// 1. n = p * q
/// 2. phi = (p-1) * (q-1)
/// 3. e = 65537 (or find coprime)
/// 4. d = e^(-1) mod phi
pub fn generate_signing_key(p: i64, q: i64) -> (SignerPublicKey, SignerPrivateKey) {
    todo!("Generate RSA key pair for blind signing")
}

/// Exercise 2: Hash a message to a field element.
///
/// SHA-256(message), interpret as i64 mod n.
/// This is the value that will be signed.
pub fn hash_message(message: &[u8], n: i64) -> i64 {
    todo!("Hash message to a field element mod n")
}

/// Exercise 3: Blind a message.
///
/// 1. Compute h = hash_message(m, n)
/// 2. Pick random r in [2, n) with gcd(r, n) = 1
/// 3. blinded = h * r^e mod n
/// 4. Return (BlindedMessage { blinded_value }, BlindingState { blinding_factor: r, message_hash: h })
pub fn blind_message(
    message: &[u8],
    pk: &SignerPublicKey,
) -> (BlindedMessage, BlindingState) {
    todo!("Blind a message for signing")
}

/// Exercise 4: Sign a blinded message.
///
/// The signer computes: s' = (blinded_value)^d mod n
///
/// The signer does NOT know what message they are signing!
pub fn sign_blinded(
    blinded: &BlindedMessage,
    sk: &SignerPrivateKey,
) -> BlindSignature {
    todo!("Sign a blinded message")
}

/// Exercise 5: Unblind a signature.
///
/// Given the blind signature s' and the blinding state:
/// 1. Compute r_inv = r^(-1) mod n
/// 2. s = s' * r_inv mod n
///
/// Now s is a valid RSA signature on hash_message(m).
pub fn unblind_signature(
    blind_sig: &BlindSignature,
    state: &BlindingState,
    n: i64,
) -> UnblindedSignature {
    todo!("Remove blinding to get the real signature")
}

/// Exercise 6: Verify a blind signature.
///
/// Check that signature^e mod n == hash_message(m, n)
///
/// This is standard RSA verification.
pub fn verify_signature(
    message: &[u8],
    signature: &UnblindedSignature,
    pk: &SignerPublicKey,
) -> bool {
    todo!("Verify a blind signature")
}

/// Exercise 7: Demonstrate that the signer cannot link blinded messages.
///
/// Given two different messages, show that their blinded forms are unlinkable
/// (i.e., blinded values are independent random-looking numbers).
pub fn verify_unlinkability(
    message1: &[u8],
    message2: &[u8],
    pk: &SignerPublicKey,
) -> bool {
    todo!("Demonstrate unlinkability of blinded messages")
}

/// Helper: GCD.
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

/// Helper: modular inverse via extended Euclidean algorithm.
pub fn mod_inv(a: i64, m: i64) -> i64 {
    let (g, x, _) = extended_gcd(a % m, m);
    if g != 1 {
        panic!("No modular inverse");
    }
    ((x % m) + m) % m
}

fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if a == 0 {
        return (b, 0, 1);
    }
    let (g, x, y) = extended_gcd(b % a, a);
    (g, y - (b / a) * x, x)
}

/// Helper: modular exponentiation.
pub fn mod_pow(mut base: i64, mut exp: i64, m: i64) -> i64 {
    if m == 1 {
        return 0;
    }
    let mut result = 1i64;
    base = ((base % m) + m) % m;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % m;
        }
        exp >>= 1;
        base = base * base % m;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use small primes for testing
    fn test_keys() -> (SignerPublicKey, SignerPrivateKey) {
        generate_signing_key(61, 53) // n = 3233
    }

    #[test]
    fn test_generate_signing_key() {
        let (pk, sk) = test_keys();
        assert_eq!(pk.n, 3233);
        assert_eq!(sk.n, 3233);
        assert_eq!((pk.e * sk.d) % (60 * 52), 1);
    }

    #[test]
    fn test_blind_sign_and_verify() {
        let (pk, sk) = test_keys();
        let message = b"vote for candidate A";

        let (blinded, state) = blind_message(message, &pk);
        let blind_sig = sign_blinded(&blinded, &sk);
        let signature = unblind_signature(&blind_sig, &state, pk.n);

        assert!(verify_signature(message, &signature, &pk));
    }

    #[test]
    fn test_wrong_message_fails() {
        let (pk, sk) = test_keys();
        let message = b"vote for candidate A";

        let (blinded, state) = blind_message(message, &pk);
        let blind_sig = sign_blinded(&blinded, &sk);
        let signature = unblind_signature(&blind_sig, &state, pk.n);

        // Verify against a different message should fail
        assert!(!verify_signature(b"vote for candidate B", &signature, &pk));
    }

    #[test]
    fn test_signer_cannot_see_message() {
        let (pk, sk) = test_keys();
        let message1 = b"secret ballot 1";
        let message2 = b"secret ballot 2";

        let (blinded1, _) = blind_message(message1, &pk);
        let (blinded2, _) = blind_message(message2, &pk);

        // The blinded values should look completely different from the messages
        // and from each other
        assert_ne!(blinded1.blinded_value, blinded2.blinded_value);
    }

    #[test]
    fn test_unlinkability() {
        let (pk, _) = test_keys();
        assert!(verify_unlinkability(b"alice", b"bob", &pk));
    }

    #[test]
    fn test_different_blinding_factors() {
        let (pk, _) = test_keys();
        let message = b"test message";
        let (_, state1) = blind_message(message, &pk);
        let (_, state2) = blind_message(message, &pk);
        // Different sessions should use different blinding factors
        assert_ne!(state1.blinding_factor, state2.blinding_factor);
    }

    #[test]
    fn test_blind_signature_is_valid_rsa() {
        let (pk, sk) = test_keys();
        let message = b"test";

        let (blinded, state) = blind_message(message, &pk);
        let blind_sig = sign_blinded(&blinded, &sk);
        let signature = unblind_signature(&blind_sig, &state, pk.n);

        // The unblinded signature should be m^d mod n
        let h = hash_message(message, pk.n);
        let expected = mod_pow(h, sk.d, pk.n);
        assert_eq!(signature.signature, expected);
    }

    #[test]
    fn test_multiple_signatures_independent() {
        let (pk, sk) = test_keys();
        let msg = b"same message";

        let (b1, s1) = blind_message(msg, &pk);
        let (b2, s2) = blind_message(msg, &pk);

        // The blinded values seen by the signer are different
        // (different blinding factors), so the signer cannot link them
        assert_ne!(b1.blinded_value, b2.blinded_value);

        let bs1 = sign_blinded(&b1, &sk);
        let bs2 = sign_blinded(&b2, &sk);
        let sig1 = unblind_signature(&bs1, &s1, pk.n);
        let sig2 = unblind_signature(&bs2, &s2, pk.n);

        // Both are valid signatures on the same message
        assert!(verify_signature(msg, &sig1, &pk));
        assert!(verify_signature(msg, &sig2, &pk));
    }
}
