//! # Lesson 01: Zeroize Basics (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use zeroize::Zeroize;

/// Zero a byte vector using the Zeroize trait.
pub fn zeroize_vec(data: &mut Vec<u8>) {
    data.zeroize();
}

/// A struct that automatically zeroes on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SecretKey {
    pub key: [u8; 32],
}

/// Constant-time byte comparison — no early exit on difference.
pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// A fixed-size buffer that zeroizes on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SecretBuffer<const N: usize> {
    pub buf: [u8; N],
}

impl<const N: usize> SecretBuffer<N> {
    pub fn new(data: &[u8]) -> Self {
        let mut buf = [0u8; N];
        let copy_len = data.len().min(N);
        buf[..copy_len].copy_from_slice(&data[..copy_len]);
        Self { buf }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

/// Create a stack-allocated secret string, zeroize it, return the length.
pub fn zeroize_stack_string(s: &str) -> usize {
    let mut data = s.as_bytes().to_vec();
    let len = data.len();
    data.zeroize();
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeroize_vec() {
        let mut data = vec![0xDE, 0xAD, 0xC0, 0xDE, 0xBE, 0xEF];
        zeroize_vec(&mut data);
        assert!(
            data.iter().all(|&b| b == 0),
            "All bytes should be zero after zeroize, got: {:?}",
            data
        );
    }

    #[test]
    fn test_zeroize_vec_empty() {
        let mut data = vec![];
        zeroize_vec(&mut data);
        assert!(data.is_empty());
    }

    #[test]
    fn test_secret_key_zeroize_on_drop() {
        let bytes = {
            let key = SecretKey { key: [0x42; 32] };
            key.key
        };
        assert_eq!(bytes, [0x42; 32]);
    }

    #[test]
    fn test_secure_compare_equal() {
        let a = b"super_secret_key_1234567890";
        let b = b"super_secret_key_1234567890";
        assert!(secure_compare(a, b));
    }

    #[test]
    fn test_secure_compare_not_equal() {
        let a = b"super_secret_key_1234567890";
        let b = b"super_secret_key_1234567891";
        assert!(!secure_compare(a, b));
    }

    #[test]
    fn test_secure_compare_different_lengths() {
        let a = b"short";
        let b = b"longer_data";
        assert!(!secure_compare(a, b));
    }

    #[test]
    fn test_secret_buffer_new() {
        let data = b"hello";
        let buf = SecretBuffer::<16>::new(data);
        assert_eq!(&buf.as_slice()[..5], b"hello");
        assert!(buf.as_slice()[5..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_secret_buffer_full() {
        let data = [0xAA; 32];
        let buf = SecretBuffer::<32>::new(&data);
        assert_eq!(buf.as_slice(), &[0xAA; 32]);
    }

    #[test]
    fn test_zeroize_stack_string() {
        let len = zeroize_stack_string("my_secret_password");
        assert_eq!(len, 18);
    }

    #[test]
    fn test_zeroize_stack_string_empty() {
        let len = zeroize_stack_string("");
        assert_eq!(len, 0);
    }
}
