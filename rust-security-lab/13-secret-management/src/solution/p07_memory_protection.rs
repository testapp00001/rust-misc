//! # Lesson 07: Memory Protection for Secrets (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::cell::Cell;
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey {
    pub key_material: Vec<u8>,
}

pub fn create_secret_key(bytes: &[u8]) -> SecretKey {
    SecretKey {
        key_material: bytes.to_vec(),
    }
}

pub fn wrap_secret(password: &str) -> (SecretString, usize) {
    let secret = SecretString::from(password.to_string());
    let len = secret.expose_secret().len();
    (secret, len)
}

pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

pub struct SecretContainer<T: Zeroize> {
    inner: T,
}

impl<T: Zeroize> SecretContainer<T> {
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }

    pub fn with_secret<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.inner)
    }
}

pub struct SecureBuffer {
    data: Vec<u8>,
    access_count: Cell<usize>,
    zeroized: bool,
}

impl SecureBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            access_count: Cell::new(0),
            zeroized: false,
        }
    }

    pub fn access(&self) -> Option<&[u8]> {
        if self.zeroized {
            None
        } else {
            self.access_count.set(self.access_count.get() + 1);
            Some(&self.data)
        }
    }

    pub fn zeroize(&mut self) {
        self.data.zeroize();
        self.zeroized = true;
    }

    pub fn access_count(&self) -> usize {
        self.access_count.get()
    }

    pub fn is_zeroized(&self) -> bool {
        self.zeroized
    }
}

pub fn shred_secret(data: &mut Vec<u8>, passes: usize) -> usize {
    let len = data.len();
    let mut rng = rand::thread_rng();
    for _ in 0..passes {
        rng.fill_bytes(data.as_mut_slice());
    }
    data.zeroize();
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_secret_key() {
        let bytes = [0x01, 0x02, 0x03, 0x04, 0x05];
        let key = create_secret_key(&bytes);
        assert_eq!(key.key_material.len(), 5);
        assert_eq!(key.key_material, bytes);
    }

    #[test]
    fn test_wrap_secret_length() {
        let (secret, len) = wrap_secret("my_password");
        assert_eq!(len, 11);
        let debug_str = format!("{:?}", secret);
        assert!(!debug_str.contains("my_password"), "Secret should be redacted in Debug");
    }

    #[test]
    fn test_constant_time_compare_equal() {
        assert!(constant_time_compare(b"hello", b"hello"));
    }

    #[test]
    fn test_constant_time_compare_not_equal() {
        assert!(!constant_time_compare(b"hello", b"world"));
    }

    #[test]
    fn test_constant_time_compare_different_lengths() {
        assert!(!constant_time_compare(b"hello", b"hi"));
    }

    #[test]
    fn test_secure_buffer_access() {
        let buf = SecureBuffer::new(vec![1, 2, 3, 4]);
        assert!(!buf.is_zeroized());
        assert_eq!(buf.access_count(), 0);

        let data = buf.access();
        assert!(data.is_some());
        assert_eq!(data.unwrap(), &[1, 2, 3, 4]);
        assert_eq!(buf.access_count(), 1);
    }

    #[test]
    fn test_secure_buffer_zeroize() {
        let mut buf = SecureBuffer::new(vec![0xAA; 32]);
        buf.zeroize();
        assert!(buf.is_zeroized());
        assert!(buf.access().is_none());
    }

    #[test]
    fn test_shred_secret() {
        let mut data = vec![0xDE_u8, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF,
                            0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF,
                            0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF,
                            0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF];
        let original = data.clone();
        let bytes_shredded = shred_secret(&mut data, 3);
        assert_eq!(bytes_shredded, 32);
        assert!(data.iter().all(|&b| b == 0));
        assert_ne!(data, original);
    }
}
