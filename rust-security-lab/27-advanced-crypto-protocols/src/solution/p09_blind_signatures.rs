//! # Lesson 09: Blind Signatures (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerPublicKey {
    pub n: i64,
    pub e: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerPrivateKey {
    pub n: i64,
    pub d: i64,
    pub p: i64,
    pub q: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindedMessage {
    pub blinded_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindSignature {
    pub blinded_signature: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnblindedSignature {
    pub signature: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindingState {
    pub blinding_factor: i64,
    pub message_hash: i64,
}

pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if a == 0 {
        return (b, 0, 1);
    }
    let (g, x, y) = extended_gcd(b % a, a);
    (g, y - (b / a) * x, x)
}

pub fn mod_inv(a: i64, m: i64) -> i64 {
    let (g, x, _) = extended_gcd(a % m, m);
    if g != 1 {
        panic!("No modular inverse");
    }
    ((x % m) + m) % m
}

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

/// Generate RSA key pair for blind signing.
pub fn generate_signing_key(p: i64, q: i64) -> (SignerPublicKey, SignerPrivateKey) {
    let n = p * q;
    let phi = (p - 1) * (q - 1);
    let mut e = 65537i64;
    if gcd(e, phi) != 1 {
        e = (3..phi).step_by(2).find(|&x| gcd(x, phi) == 1).unwrap();
    }
    let d = mod_inv(e, phi);

    (
        SignerPublicKey { n, e },
        SignerPrivateKey { n, d, p, q },
    )
}

/// Hash a message to a field element mod n.
pub fn hash_message(message: &[u8], n: i64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(message);
    let hash = hasher.finalize();
    let val = i64::from_le_bytes(hash[..8].try_into().unwrap()).abs();
    val % n
}

/// Blind a message for signing.
pub fn blind_message(
    message: &[u8],
    pk: &SignerPublicKey,
) -> (BlindedMessage, BlindingState) {
    let mut rng = rand::thread_rng();

    let h = hash_message(message, pk.n);

    // Pick random blinding factor r coprime to n
    let mut r: i64;
    loop {
        r = rng.gen_range(2..pk.n);
        if gcd(r, pk.n) == 1 {
            break;
        }
    }

    // blinded = h * r^e mod n
    let re = mod_pow(r, pk.e, pk.n);
    let blinded_value = (h * re) % pk.n;

    (
        BlindedMessage { blinded_value },
        BlindingState {
            blinding_factor: r,
            message_hash: h,
        },
    )
}

/// Sign a blinded message.
pub fn sign_blinded(
    blinded: &BlindedMessage,
    sk: &SignerPrivateKey,
) -> BlindSignature {
    let blinded_signature = mod_pow(blinded.blinded_value, sk.d, sk.n);
    BlindSignature { blinded_signature }
}

/// Unblind a signature.
pub fn unblind_signature(
    blind_sig: &BlindSignature,
    state: &BlindingState,
    n: i64,
) -> UnblindedSignature {
    let r_inv = mod_inv(state.blinding_factor, n);
    let signature = (blind_sig.blinded_signature * r_inv) % n;
    UnblindedSignature { signature }
}

/// Verify a blind signature.
pub fn verify_signature(
    message: &[u8],
    signature: &UnblindedSignature,
    pk: &SignerPublicKey,
) -> bool {
    let h = hash_message(message, pk.n);
    // Check: signature^e mod n == h
    let recovered = mod_pow(signature.signature, pk.e, pk.n);
    recovered == h
}

/// Demonstrate unlinkability of blinded messages.
pub fn verify_unlinkability(
    message1: &[u8],
    message2: &[u8],
    pk: &SignerPublicKey,
) -> bool {
    let (blinded1, _) = blind_message(message1, pk);
    let (blinded2, _) = blind_message(message2, pk);

    // Blinded values should be different from each other and from the hashes
    let h1 = hash_message(message1, pk.n);
    let h2 = hash_message(message2, pk.n);

    blinded1.blinded_value != h1
        && blinded2.blinded_value != h2
        && blinded1.blinded_value != blinded2.blinded_value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keys() -> (SignerPublicKey, SignerPrivateKey) {
        generate_signing_key(61, 53)
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

        assert!(!verify_signature(b"vote for candidate B", &signature, &pk));
    }

    #[test]
    fn test_signer_cannot_see_message() {
        let (pk, _) = test_keys();
        let message1 = b"secret ballot 1";
        let message2 = b"secret ballot 2";

        let (blinded1, _) = blind_message(message1, &pk);
        let (blinded2, _) = blind_message(message2, &pk);

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
        assert_ne!(state1.blinding_factor, state2.blinding_factor);
    }

    #[test]
    fn test_blind_signature_is_valid_rsa() {
        let (pk, sk) = test_keys();
        let message = b"test";

        let (blinded, state) = blind_message(message, &pk);
        let blind_sig = sign_blinded(&blinded, &sk);
        let signature = unblind_signature(&blind_sig, &state, pk.n);

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
