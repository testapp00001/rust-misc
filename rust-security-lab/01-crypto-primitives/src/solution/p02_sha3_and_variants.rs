//! # Lesson 02: SHA-3 Family (Reference Solution)

use sha3::{Digest, Sha3_256, Sha3_512, Shake256};
use sha3::digest::{ExtendableOutput, Update, XofReader};

pub fn sha3_256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn sha3_512(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_512::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn shake256(data: &[u8], output_len: usize) -> Vec<u8> {
    let mut hasher = Shake256::default();
    hasher.update(data);
    let mut output = vec![0u8; output_len];
    let mut reader = hasher.finalize_xof();
    reader.read(&mut output);
    output
}

pub fn sha3_no_length_extension(data: &[u8]) -> Vec<u8> {
    sha3_256(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha3_256_empty() {
        let hash = sha3_256(b"");
        assert_eq!(hash.len(), 32);
        let expected = "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a";
        assert_eq!(hex::encode(&hash), expected);
    }

    #[test]
    fn test_sha3_256_hello() {
        let hash = sha3_256(b"hello");
        assert_eq!(hash.len(), 32);
        let sha256_hello = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert_ne!(hex::encode(&hash), sha256_hello);
    }

    #[test]
    fn test_sha3_512() {
        let hash = sha3_512(b"hello");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_shake256_variable_length() {
        let hash_32 = shake256(b"test", 32);
        let hash_64 = shake256(b"test", 64);
        let hash_100 = shake256(b"test", 100);
        assert_eq!(hash_32.len(), 32);
        assert_eq!(hash_64.len(), 64);
        assert_eq!(hash_100.len(), 100);
        assert_eq!(&hash_32[..], &hash_64[..32]);
        assert_eq!(&hash_32[..], &hash_100[..32]);
    }

    #[test]
    fn test_sha3_different_from_sha2() {
        let data = b"identical input";
        let sha2 = ring::digest::digest(&ring::digest::SHA256, data);
        let sha3 = sha3_256(data);
        assert_ne!(sha2.as_ref(), &sha3[..]);
    }

    #[test]
    fn test_sha3_immune_to_length_extension() {
        let msg = b"secret message";
        let hash1 = sha3_256(msg);
        let extended = [msg.as_ref(), b"evil extension"].concat();
        let hash2 = sha3_256(&extended);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_deterministic() {
        let h1 = sha3_256(b"consistency check");
        let h2 = sha3_256(b"consistency check");
        assert_eq!(h1, h2);
    }
}
