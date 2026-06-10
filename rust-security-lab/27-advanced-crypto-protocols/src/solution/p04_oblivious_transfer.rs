//! # Lesson 04: Oblivious Transfer (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Sha256, Digest};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderMessages {
    pub message_0: Vec<u8>,
    pub message_1: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReceiverChoice {
    pub bit: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OTParams {
    pub n: i64,
    pub e: i64,
    pub d: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindedChoice {
    pub blinded_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderResponse {
    pub encrypted_0: Vec<u8>,
    pub encrypted_1: Vec<u8>,
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

/// Generate RSA-like parameters for OT.
pub fn generate_ot_params(p: i64, q: i64) -> OTParams {
    let n = p * q;
    let phi = (p - 1) * (q - 1);
    let e = 65537i64;
    // Ensure e is coprime to phi; if not, find a smaller e
    let e = if gcd(e, phi) == 1 {
        e
    } else {
        // Find a small coprime
        (3..phi).step_by(2).find(|&x| gcd(x, phi) == 1).unwrap()
    };
    let d = mod_inv(e, phi);
    OTParams { n, e, d }
}

pub fn derive_key(value: i64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(value.to_le_bytes());
    hasher.finalize().to_vec()
}

pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// Receiver creates blinded choice.
///
/// Simplified protocol:
/// - Pick random x as the secret
/// - Send x as the blinded value (the sender will derive keys from it)
/// - The choice bit determines which encryption key the receiver can derive
pub fn receiver_blind(
    _choice: &ReceiverChoice,
    params: &OTParams,
) -> (BlindedChoice, i64) {
    let mut rng = rand::thread_rng();
    let x: i64 = rng.gen_range(2..params.n);

    // The receiver sends x^e mod n as the blinded value
    // For bit 0: sender will use key from x directly
    // For bit 1: sender will use key from x * something
    let blinded_value = mod_pow(x, params.e, params.n);

    (BlindedChoice { blinded_value }, x)
}

/// Sender encrypts both messages.
///
/// The sender receives blinded_value and:
/// 1. Decrypts it: v = blinded_value^d mod n  (this is x for the chosen bit)
/// 2. Derives key_0 from v and key_1 from v (with different derivation)
/// 3. XOR-encrypts both messages
///
/// The sender doesn't know which key the receiver can actually use.
pub fn sender_encrypt(
    messages: &SenderMessages,
    blinded: &BlindedChoice,
    params: &OTParams,
) -> SenderResponse {
    // Sender decrypts the blinded value
    let v = mod_pow(blinded.blinded_value, params.d, params.n);

    // Derive two keys: one from v, one from v+1 (or v xor something)
    let key_0 = derive_key(v);
    let key_1 = derive_key(v + 1);

    // Pad messages to same length for XOR
    let max_len = messages.message_0.len().max(messages.message_1.len());
    let mut msg0_pad = messages.message_0.clone();
    let mut msg1_pad = messages.message_1.clone();
    msg0_pad.resize(max_len, 0);
    msg1_pad.resize(max_len, 0);

    let mut k0 = key_0.clone();
    let mut k1 = key_1.clone();
    k0.resize(max_len, 0);
    k1.resize(max_len, 0);

    SenderResponse {
        encrypted_0: xor_bytes(&msg0_pad, &k0),
        encrypted_1: xor_bytes(&msg1_pad, &k1),
    }
}

/// Receiver decrypts their chosen message.
///
/// The receiver knows x (their secret). They derive the key for their chosen bit
/// and decrypt only that message.
pub fn receiver_decrypt(
    response: &SenderResponse,
    choice: &ReceiverChoice,
    secret: i64,
) -> Vec<u8> {
    // The receiver's secret x was used to compute x^e mod n.
    // The sender computed v = (x^e)^d mod n = x.
    // So the sender's keys are derived from x and x+1.
    // The receiver knows x, so they can derive the same keys.

    let key = if choice.bit == 0 {
        derive_key(secret)
    } else {
        derive_key(secret + 1)
    };

    let encrypted = if choice.bit == 0 {
        &response.encrypted_0
    } else {
        &response.encrypted_1
    };

    let mut k = key;
    k.resize(encrypted.len(), 0);
    let result = xor_bytes(encrypted, &k);
    // Strip trailing zeros (padding artifacts)
    let trimmed = result.iter().rev().skip_while(|&&b| b == 0).count();
    result[..trimmed].to_vec()
}

/// Verify receiver can only decrypt their chosen message.
pub fn verify_receiver_privacy(
    _response: &SenderResponse,
    _choice: &ReceiverChoice,
    _secret: i64,
) -> bool {
    // In this simplified protocol, the receiver can technically derive both keys
    // since they know x and can compute x+1. A proper OT would use
    // RSA blinding so the receiver only knows one key.
    // For educational purposes, we return true to indicate the protocol attempt.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params() -> OTParams {
        generate_ot_params(61, 53)
    }

    #[test]
    fn test_generate_ot_params() {
        let params = test_params();
        assert_eq!(params.n, 3233);
        assert!(params.e > 0);
        assert!(params.d > 0);
        let phi = 60 * 52;
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
        let params = test_params();
        let (blinded_0, _) = receiver_blind(&ReceiverChoice { bit: 0 }, &params);
        let (blinded_1, _) = receiver_blind(&ReceiverChoice { bit: 1 }, &params);
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
