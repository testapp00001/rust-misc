//! # Lesson 07: Memory Protection for Secrets
//!
//! ## The Problem
//!
//! Secrets in memory can be exposed through:
//!
//! 1. **Core dumps**: A crashed process writes its memory to disk
//! 2. **Swap space**: OS may write memory pages to disk
//! 3. **Cold boot attacks**: RAM retains data briefly after power off
//! 4. **Memory forensics**: Tools like `strings` can find secrets in process memory
//! 5. **Spectre/Meltdown**: CPU vulnerabilities can leak memory contents
//!
//! ## Defense: Memory Hygiene
//!
//! - **Zeroize**: Overwrite secret memory when done (don't just drop it)
//! - **mlock**: Prevent secrets from being swapped to disk
//! - **Guard pages**: Detect buffer overflows around secret memory
//! - **Avoid copies**: Minimize the number of places a secret exists in memory
//!
//! ## The `zeroize` Crate
//!
//! The `zeroize` crate provides a `Zeroize` trait that overwrites memory with zeros
//! when a value is dropped. It uses compiler fences to prevent the optimizer from
//! removing the zeroing operation.
//!
//! ```rust
//! use zeroize::Zeroize;
//!
//! let mut secret = vec![1u8, 2, 3, 4];
//! secret.zeroize(); // Memory is now all zeros
//! ```
//!
//! ## The `secrecy` Crate
//!
//! The `secrecy` crate wraps secrets in a type that:
//! - Prevents accidental logging (no Debug output)
//! - Zeroizes on drop
//! - Makes it explicit where secrets are used
//!
//! ```rust
//! use secrecy::{Secret, ExposeSecret};
//!
//! let secret = Secret::new("password".to_string());
//! // println!("{:?}", secret);  // ERROR: Debug is redacted
//! let value = secret.expose_secret(); // Explicit opt-in
//! ```
//!
//! ## This Exercise
//!
//! We'll practice using `zeroize` and `secrecy` to protect secrets in memory.

use std::cell::Cell;
use secrecy::{ExposeSecret, SecretString};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secret key that is zeroized when dropped.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey {
    pub key_material: Vec<u8>,
}

/// Exercise 1: Create a SecretKey and verify it can be zeroized.
///
/// Create a SecretKey with the given bytes. Return the key material length.
///
/// Hints:
/// - Create a `SecretKey` with `key_material` set to a copy of the input
/// - Return the length of `key_material`
pub fn create_secret_key(bytes: &[u8]) -> SecretKey {
    todo!("Create a SecretKey from bytes")
}

/// Exercise 2: Use the secrecy crate to wrap a string secret.
///
/// Given a plaintext password, wrap it in a `SecretString`.
/// Return a function that, when called, returns the length of the secret
/// WITHOUT exposing the actual value.
///
/// Hints:
/// - Create `SecretString::from(password.to_string())`
/// - Use `secret.expose_secret()` to access the inner value for length
/// - Return the length
pub fn wrap_secret(password: &str) -> (SecretString, usize) {
    todo!("Wrap a password in SecretString")
}

/// Exercise 3: Compare two secrets in constant time.
///
/// Return true if the two byte slices are equal, using a constant-time
/// comparison to avoid timing side channels.
///
/// Hints:
/// - Use `ring::constant_time::verify_slices_are_equal` if available
/// - Or implement: XOR each byte pair, OR the results together
/// - The comparison should take the same time regardless of where the
///   first difference occurs
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    todo!("Compare two byte slices in constant time")
}

/// Exercise 4: Create a secret container that only exposes the secret
/// through a callback.
///
/// The `SecretContainer` holds a value and only allows access through
/// a `with_secret` method that takes a closure.
///
/// Implement:
/// - `SecretContainer::new(value)` — create a new container
/// - `SecretContainer::with_secret(f)` — call `f` with a reference to the value
/// - `SecretContainer::length()` — return the length without exposing the value
pub struct SecretContainer<T: Zeroize> {
    inner: T,
}

impl<T: Zeroize> SecretContainer<T> {
    /// Create a new secret container.
    pub fn new(value: T) -> Self {
        todo!("Create a SecretContainer")
    }

    /// Access the secret through a callback.
    /// The callback receives a reference to the inner value.
    pub fn with_secret<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        todo!("Access secret through callback")
    }
}

/// Exercise 5: Implement a `SecureBuffer` that tracks access count.
///
/// The SecureBuffer:
/// - Holds a Vec<u8> that is zeroized on drop
/// - Tracks how many times the data has been accessed (use Cell<usize> for interior mutability)
/// - Only allows access if the buffer has not been zeroized
///
/// Implement:
/// - `SecureBuffer::new(data: Vec<u8>)` — create a new buffer
/// - `SecureBuffer::access(&self)` — return `Some(&[u8])` if not zeroized, `None` otherwise
/// - `SecureBuffer::zeroize(&mut self)` — zero out the data and mark as consumed
/// - `SecureBuffer::access_count(&self)` — return how many times access() was called
/// - `SecureBuffer::is_zeroized(&self)` — return whether the buffer has been zeroized
///
/// Hints:
/// - Use `std::cell::Cell<usize>` for `access_count` to allow interior mutability
/// - `Cell::get()` and `Cell::set()` don't require `&mut self`
pub struct SecureBuffer {
    data: Vec<u8>,
    access_count: Cell<usize>,
    zeroized: bool,
}

impl SecureBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        todo!("Create a SecureBuffer")
    }

    pub fn access(&self) -> Option<&[u8]> {
        todo!("Access buffer data if not zeroized")
    }

    pub fn zeroize(&mut self) {
        todo!("Zero out the buffer")
    }

    pub fn access_count(&self) -> usize {
        todo!("Return access count")
    }

    pub fn is_zeroized(&self) -> bool {
        todo!("Check if zeroized")
    }
}

/// Exercise 6: Implement a memory-safe secret shredder.
///
/// Given a mutable Vec<u8>, overwrite it with random data N times,
/// then zero it. This makes recovery harder (defense in depth, not
/// a guarantee against hardware-level attacks).
///
/// The function should:
/// 1. Overwrite with random data `passes` times
/// 2. Final overwrite with zeros
/// 3. Return the number of bytes shredded
///
/// Hints:
/// - Use `rand::RngCore::fill_bytes` for random overwrites
/// - Use `zeroize::Zeroize` for the final zero pass
pub fn shred_secret(data: &mut Vec<u8>, passes: usize) -> usize {
    todo!("Overwrite secret data with random bytes then zeroize")
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
        // The secret should not be printable via Debug
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
        let mut data = vec![0xDE, 0xAD, 0xBE, 0xEF; 8];
        let original = data.clone();
        let bytes_shredded = shred_secret(&mut data, 3);
        assert_eq!(bytes_shredded, 32);
        // After shredding, data should be all zeros
        assert!(data.iter().all(|&b| b == 0));
        // And definitely not the original
        assert_ne!(data, original);
    }
}
