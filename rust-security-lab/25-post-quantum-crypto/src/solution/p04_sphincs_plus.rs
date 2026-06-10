//! # Lesson 04: SPHINCS+ — Hash-Based Digital Signatures (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};
use rand::Rng;

/// A simplified WOTS+ key pair.
#[derive(Debug, Clone)]
pub struct WotsKeyPair {
    pub secret_chains: Vec<Vec<u8>>,
    pub public_chains: Vec<Vec<u8>>,
    pub w: usize,
    pub chain_len: usize,
}

/// A WOTS+ signature.
#[derive(Debug, Clone)]
pub struct WotsSignature {
    pub chain_values: Vec<Vec<u8>>,
}

/// Compute a Winternitz chain: apply SHA-256 `steps` times.
pub fn winternitz_chain(seed: &[u8], steps: usize) -> Vec<u8> {
    let mut current = seed.to_vec();
    for _ in 0..steps {
        current = Sha256::digest(&current).to_vec();
    }
    current
}

/// Generate a WOTS+ keypair.
///
/// With w=16, chain_len=2, total chains = 2*n (n message chains + n checksum chains).
pub fn wots_keygen(n: usize, w: usize) -> WotsKeyPair {
    let mut rng = rand::thread_rng();

    // chain_len = ceil(log(256) / log(w))
    let chain_len = match w {
        2 => 8,
        4 => 4,
        16 => 2,
        256 => 1,
        _ => ((256.0f64).ln() / (w as f64).ln()).ceil() as usize,
    };

    // Total chains: n for message + n for checksum (simplified)
    let total_chains = 2 * n;
    let max_digit = w - 1;

    let mut secret_chains = Vec::with_capacity(total_chains);
    let mut public_chains = Vec::with_capacity(total_chains);

    for _ in 0..total_chains {
        let secret: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        let public = winternitz_chain(&secret, max_digit * chain_len);
        secret_chains.push(secret);
        public_chains.push(public);
    }

    WotsKeyPair {
        secret_chains,
        public_chains,
        w,
        chain_len,
    }
}

/// Sign a message hash with WOTS+.
///
/// Converts each byte to a base-w digit and computes chain values.
/// Also computes and signs a checksum.
pub fn wots_sign(kp: &WotsKeyPair, msg_hash: &[u8]) -> WotsSignature {
    let n = msg_hash.len();
    let w = kp.w;
    let chain_len = kp.chain_len;
    let max_digit = w - 1;

    // Convert message bytes to base-w digits
    let digits: Vec<usize> = msg_hash.iter().map(|&b| b as usize % w).collect();

    // Compute checksum: sum of (max_digit - digit) for all message digits
    let checksum_raw: usize = digits.iter().map(|&d| max_digit - d).sum();

    // Encode checksum as base-w digits (n checksum digits)
    let mut checksum = checksum_raw;
    let mut checksum_digits = vec![0usize; n];
    for i in (0..n).rev() {
        checksum_digits[i] = checksum % w;
        checksum /= w;
    }

    // Combine message + checksum digits
    let mut all_digits = digits;
    all_digits.extend(checksum_digits);

    // Compute signature: chain from secret to digit position
    let chain_values: Vec<Vec<u8>> = all_digits
        .iter()
        .enumerate()
        .map(|(i, &digit)| {
            winternitz_chain(&kp.secret_chains[i], digit * chain_len)
        })
        .collect();

    WotsSignature { chain_values }
}

/// Verify a WOTS+ signature.
///
/// Continues each chain from the signature value to the public key value.
pub fn wots_verify(kp: &WotsKeyPair, msg_hash: &[u8], sig: &WotsSignature) -> bool {
    let n = msg_hash.len();
    let w = kp.w;
    let chain_len = kp.chain_len;
    let max_digit = w - 1;

    // Convert message bytes to base-w digits (same as signing)
    let digits: Vec<usize> = msg_hash.iter().map(|&b| b as usize % w).collect();

    // Compute checksum (same as signing)
    let checksum_raw: usize = digits.iter().map(|&d| max_digit - d).sum();
    let mut checksum = checksum_raw;
    let mut checksum_digits = vec![0usize; n];
    for i in (0..n).rev() {
        checksum_digits[i] = checksum % w;
        checksum /= w;
    }

    let mut all_digits = digits;
    all_digits.extend(checksum_digits);

    // Verify each chain
    if sig.chain_values.len() != all_digits.len() {
        return false;
    }

    for (i, &digit) in all_digits.iter().enumerate() {
        let remaining = (max_digit - digit) * chain_len;
        let computed = winternitz_chain(&sig.chain_values[i], remaining);
        if computed != kp.public_chains[i] {
            return false;
        }
    }

    true
}

/// Detect WOTS+ key reuse: if two different messages were signed with the same key.
pub fn detect_key_reuse(hash1: &[u8], hash2: &[u8]) -> bool {
    hash1 != hash2
}

/// Estimate SPHINCS+ signature size.
///
/// Simplified model: hypertree WOTS+ signatures + FORS signature + randomness.
pub fn sphincs_sig_size_estimate(n: usize, h: usize, _d: usize, _w: usize, k: usize) -> usize {
    let wots_sig_size = 2 * n * 2; // 2*n chains, each chain value is n bytes, chain_len=2
    let hypertree_sigs = h; // one WOTS+ sig per layer
    let fors_sig_size = k * (n + 1); // k trees, each with n-byte auth path + 1 leaf
    hypertree_sigs * wots_sig_size + fors_sig_size + n // +n for randomness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winternitz_chain_zero() {
        let seed = b"test_seed";
        let result = winternitz_chain(seed, 0);
        assert_eq!(result, seed);
    }

    #[test]
    fn test_winternitz_chain_one() {
        let seed = b"test_seed";
        let result = winternitz_chain(seed, 1);
        let expected = Sha256::digest(seed).to_vec();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_winternitz_chain_length() {
        let seed = b"seed";
        let result = winternitz_chain(seed, 5);
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_wots_keygen_dimensions() {
        let kp = wots_keygen(16, 16);
        assert_eq!(kp.secret_chains.len(), 32);
        assert_eq!(kp.public_chains.len(), 32);
        assert_eq!(kp.w, 16);
    }

    #[test]
    fn test_wots_sign_verify_roundtrip() {
        let kp = wots_keygen(16, 16);
        let msg_hash = Sha256::digest(b"hello world").to_vec();
        let sig = wots_sign(&kp, &msg_hash);
        assert!(wots_verify(&kp, &msg_hash, &sig));
    }

    #[test]
    fn test_wots_verify_wrong_message() {
        let kp = wots_keygen(16, 16);
        let msg_hash = Sha256::digest(b"hello").to_vec();
        let sig = wots_sign(&kp, &msg_hash);
        let wrong_hash = Sha256::digest(b"world").to_vec();
        assert!(!wots_verify(&kp, &wrong_hash, &sig));
    }

    #[test]
    fn test_detect_key_reuse() {
        let hash1 = Sha256::digest(b"msg1").to_vec();
        let hash2 = Sha256::digest(b"msg2").to_vec();
        assert!(detect_key_reuse(&hash1, &hash2));
    }

    #[test]
    fn test_detect_no_reuse() {
        let hash1 = Sha256::digest(b"same").to_vec();
        assert!(!detect_key_reuse(&hash1, &hash1));
    }

    #[test]
    fn test_sphincs_sig_size_estimate() {
        let size = sphincs_sig_size_estimate(16, 63, 7, 16, 14);
        assert!(size > 1000);
    }

    #[test]
    fn test_wots_signature_chain_value_lengths() {
        let kp = wots_keygen(16, 16);
        let msg_hash = Sha256::digest(b"test").to_vec();
        let sig = wots_sign(&kp, &msg_hash);
        for cv in &sig.chain_values {
            assert_eq!(cv.len(), 32);
        }
    }
}
