//! # Lesson 04: Oblivious Transfer -- Send Without Learning the Choice
//!
//! ## What is Oblivious Transfer?
//!
//! Oblivious Transfer (OT) is a fundamental protocol in cryptography where:
//! - The **sender** has N messages (here: 2 messages m_0, m_1)
//! - The **receiver** has a choice bit b (0 or 1)
//! - After the protocol, the receiver learns m_b but nothing about m_(1-b)
//! - The sender learns nothing about b
//!
//! ## 1-out-of-2 OT (OT_1^2)
//!
//! The most common variant. Protocol using RSA (simplified):
//!
//! 1. Sender generates RSA key pair (N, e, d). Sends (N, e) to receiver.
//! 2. Receiver picks random x, computes:
//!    - k_0 = x^e mod N  (for choice 0)
//!    - k_1 = (x * something)^e mod N  (for choice 1, using blinding)
//!    - Sends k_b to sender.
//! 3. Sender decrypts both: m'_0 = m_0 XOR H(k_0^d), m'_1 = m_1 XOR H(k_1^d)
//!    (Sender doesn't know which k is "real")
//! 4. Receiver can only decrypt m_b since they know the corresponding x.
//!
//! ## Why OT Matters
//!
//! OT is the "atomic" building block for:
//! - **Garbled circuits**: Evaluate any function without revealing inputs
//! - **Private set intersection**: Find common elements privately
//! - **MPC protocols**: Most general MPC uses OT as a subroutine
//! - **Mental poker**: Deal cards obliviously
//!
//! ## Attack: Sender Impersonation
//!
//! If the sender's public key is not authenticated, an adversary can substitute their own
//! key and learn the receiver's choice bit. OT requires authenticated channels or
//! pre-shared keys.

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// Messages held by the sender.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderMessages {
    pub message_0: Vec<u8>,
    pub message_1: Vec<u8>,
}

/// The receiver's choice: which message to learn.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReceiverChoice {
    /// 0 or 1
    pub bit: u8,
}

/// Protocol parameters (simplified RSA-like).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OTParams {
    pub n: i64,
    pub e: i64,
    pub d: i64,
}

/// The receiver's blinded message sent to the sender.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindedChoice {
    /// The blinded value sent to the sender
    pub blinded_value: i64,
}

/// The sender's response (encrypted messages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderResponse {
    pub encrypted_0: Vec<u8>,
    pub encrypted_1: Vec<u8>,
}

/// Exercise 1: Generate OT protocol parameters.
///
/// Given primes p and q, compute:
/// - n = p * q
/// - phi = (p-1) * (q-1)
/// - e = 65537 (standard public exponent, or find one coprime to phi)
/// - d = e^(-1) mod phi
pub fn generate_ot_params(p: i64, q: i64) -> OTParams {
    todo!("Generate RSA-like parameters for OT protocol")
}

/// Exercise 2: Receiver creates their blinded choice.
///
/// The receiver wants message `bit` (0 or 1):
/// 1. Pick a random x in [2, n)
/// 2. Compute v = x^e mod n  (this is the "public key" for their choice)
/// 3. If bit == 0: send v as-is
/// 4. If bit == 1: send v * something (blind it differently)
///
/// For simplicity, we'll use a hash-based approach:
/// - Generate a random secret x
/// - Compute k0 = x (the "key" for choice 0)
/// - Compute k1 derived from x and the choice
///
/// Return (BlindedChoice, receiver_secret)
pub fn receiver_blind(
    choice: &ReceiverChoice,
    params: &OTParams,
) -> (BlindedChoice, i64) {
    todo!("Create blinded choice for the sender")
}

/// Exercise 3: Sender encrypts both messages.
///
/// Given the receiver's blinded choice:
/// 1. Derive two keys from the blinded value (sender doesn't know which is "real")
/// 2. Encrypt message_0 with key_0, message_1 with key_1
/// 3. Return the encrypted pair
///
/// Use XOR with SHA-256(key) as the encryption (stream cipher style).
pub fn sender_encrypt(
    messages: &SenderMessages,
    blinded: &BlindedChoice,
    params: &OTParams,
) -> SenderResponse {
    todo!("Encrypt both messages using blinded choice")
}

/// Exercise 4: Receiver decrypts their chosen message.
///
/// Given the sender's response and the receiver's secret:
/// 1. Derive the key for the chosen message
/// 2. Decrypt only the chosen message
/// 3. Return the plaintext
pub fn receiver_decrypt(
    response: &SenderResponse,
    choice: &ReceiverChoice,
    secret: i64,
) -> Vec<u8> {
    todo!("Decrypt the chosen message using receiver's secret")
}

/// Exercise 5: Verify that the receiver cannot learn the other message.
///
/// Given the protocol transcript, check that the receiver only has enough
/// information to decrypt one message, not both.
///
/// This is a conceptual check: verify that the two encryption keys are
/// independent (one derived from secret, the other would require a different secret).
pub fn verify_receiver_privacy(
    response: &SenderResponse,
    choice: &ReceiverChoice,
    secret: i64,
) -> bool {
    todo!("Verify receiver can only decrypt their chosen message")
}

/// Helper: XOR two byte slices.
pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// Helper: derive a key from an integer using SHA-256.
pub fn derive_key(value: i64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(value.to_le_bytes());
    hasher.finalize().to_vec()
}

/// Helper: simple GCD.
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

/// Helper: modular inverse.
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

    fn test_params() -> OTParams {
        generate_ot_params(61, 53) // n = 3233, standard small RSA example
    }

    #[test]
    fn test_generate_ot_params() {
        let params = test_params();
        assert_eq!(params.n, 3233);
        assert!(params.e > 0);
        assert!(params.d > 0);
        // Verify e * d = 1 mod phi
        let phi = 60 * 52; // (61-1)*(53-1)
        assert_eq!((params.e * params.d) % phi, 1);
    }

    #[test]
    fn test_ot_protocol_choice_0() {
        let params = test_params();
        let messages = SenderMessages {
            message_0: b"secret message zero".to_vec(),
            message_1: b"secret message one".to_vec(),
        };
        let choice = ReceiverChoice { bit: 0 };

        let (blinded, secret) = receiver_blind(&choice, &params);
        let response = sender_encrypt(&messages, &blinded, &params);
        let decrypted = receiver_decrypt(&response, &choice, secret);

        assert_eq!(decrypted, messages.message_0);
    }

    #[test]
    fn test_ot_protocol_choice_1() {
        let params = test_params();
        let messages = SenderMessages {
            message_0: b"secret message zero".to_vec(),
            message_1: b"secret message one".to_vec(),
        };
        let choice = ReceiverChoice { bit: 1 };

        let (blinded, secret) = receiver_blind(&choice, &params);
        let response = sender_encrypt(&messages, &blinded, &params);
        let decrypted = receiver_decrypt(&response, &choice, secret);

        assert_eq!(decrypted, messages.message_1);
    }

    #[test]
    fn test_sender_learns_nothing() {
        // Sender sees only the blinded choice, which looks random
        let params = test_params();
        let (blinded_0, _) = receiver_blind(&ReceiverChoice { bit: 0 }, &params);
        let (blinded_1, _) = receiver_blind(&ReceiverChoice { bit: 1 }, &params);
        // The blinded values should be different (random)
        assert_ne!(blinded_0.blinded_value, blinded_1.blinded_value);
    }

    #[test]
    fn test_xor_bytes() {
        let a = b"hello";
        let key = b"keyyy";
        let encrypted = xor_bytes(a, key);
        let decrypted = xor_bytes(&encrypted, key);
        assert_eq!(decrypted, a);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let k1 = derive_key(42);
        let k2 = derive_key(42);
        assert_eq!(k1, k2);

        let k3 = derive_key(43);
        assert_ne!(k1, k3);
    }

    #[test]
    fn test_verify_receiver_privacy() {
        let params = test_params();
        let messages = SenderMessages {
            message_0: b"alpha".to_vec(),
            message_1: b"beta".to_vec(),
        };
        let choice = ReceiverChoice { bit: 0 };
        let (blinded, secret) = receiver_blind(&choice, &params);
        let response = sender_encrypt(&messages, &blinded, &params);
        assert!(verify_receiver_privacy(&response, &choice, secret));
    }

    #[test]
    fn test_ot_different_messages() {
        let params = test_params();
        let messages = SenderMessages {
            message_0: vec![1, 2, 3],
            message_1: vec![4, 5, 6],
        };

        let c0 = ReceiverChoice { bit: 0 };
        let (b0, s0) = receiver_blind(&c0, &params);
        let r0 = sender_encrypt(&messages, &b0, &params);
        let d0 = receiver_decrypt(&r0, &c0, s0);
        assert_eq!(d0, vec![1, 2, 3]);

        let c1 = ReceiverChoice { bit: 1 };
        let (b1, s1) = receiver_blind(&c1, &params);
        let r1 = sender_encrypt(&messages, &b1, &params);
        let d1 = receiver_decrypt(&r1, &c1, s1);
        assert_eq!(d1, vec![4, 5, 6]);
    }
}
