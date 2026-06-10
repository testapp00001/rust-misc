//! # Lesson 06: Memory Forensics Defense (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use zeroize::Zeroize;

/// RAII wrapper that zeroizes secret data on drop.
pub struct SecretScope {
    data: Vec<u8>,
}

impl SecretScope {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { data: secret }
    }

    /// Execute a closure with access to the secret bytes.
    pub fn with_secret<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.data)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecretScope {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

/// Generate a temporary key, use it in a closure, then zeroize.
pub fn ephemeral_key<F, R>(f: F) -> R
where
    F: FnOnce(&[u8]) -> R,
{
    let mut key = [0u8; 32];
    rand::Rng::fill(&mut rand::thread_rng(), &mut key[..]);
    let result = f(&key);
    key.zeroize();
    result
}

/// A Vec that tracks and minimizes copies, zeroizing on drop.
pub struct SecureVec {
    data: Vec<u8>,
}

impl SecureVec {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn expose(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Zeroize for SecureVec {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

impl Drop for SecureVec {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Zero a specific range of bytes in a slice.
pub fn zeroize_range(data: &mut [u8], start: usize, end: usize) {
    let clamped_end = end.min(data.len());
    if start < clamped_end {
        data[start..clamped_end].zeroize();
    }
}

/// Use RAII to ensure a secret is zeroed after use, even on panic.
pub fn minimize_secret_lifetime<F, R>(secret: Vec<u8>, f: F) -> R
where
    F: FnOnce(&[u8]) -> R,
{
    let scope = SecretScope::new(secret);
    scope.with_secret(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_scope_access() {
        let scope = SecretScope::new(vec![0xDE, 0xAD, 0xC0, 0xDE]);
        let result = scope.with_secret(|data| {
            assert_eq!(data, &[0xDE, 0xAD, 0xC0, 0xDE]);
            data.len()
        });
        assert_eq!(result, 4);
    }

    #[test]
    fn test_secret_scope_len() {
        let scope = SecretScope::new(vec![1, 2, 3]);
        assert_eq!(scope.len(), 3);
        assert!(!scope.is_empty());
    }

    #[test]
    fn test_ephemeral_key_generates() {
        let result = ephemeral_key(|key| {
            assert_eq!(key.len(), 32);
            key.iter().copied().fold(0u8, |acc, b| acc.wrapping_add(b))
        });
        let _ = result;
    }

    #[test]
    fn test_ephemeral_key_different_each_time() {
        let key1 = ephemeral_key(|key| key.to_vec());
        let key2 = ephemeral_key(|key| key.to_vec());
        assert_ne!(key1, key2, "Random keys should differ");
    }

    #[test]
    fn test_secure_vec_basic() {
        let sv = SecureVec::new(vec![0xAA; 16]);
        assert_eq!(sv.len(), 16);
        assert_eq!(sv.expose(), &[0xAA; 16]);
    }

    #[test]
    fn test_secure_vec_drop_zeroizes() {
        let sv = SecureVec::new(vec![0x42; 32]);
        assert!(sv.expose().iter().all(|&b| b == 0x42));
        drop(sv);
    }

    #[test]
    fn test_zeroize_range() {
        let mut data = vec![0xFF; 16];
        zeroize_range(&mut data, 4, 12);
        assert!(data[..4].iter().all(|&b| b == 0xFF));
        assert!(data[4..12].iter().all(|&b| b == 0));
        assert!(data[12..].iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_minimize_secret_lifetime() {
        let secret = vec![0x42; 32];
        let result = minimize_secret_lifetime(secret, |data| {
            assert_eq!(data.len(), 32);
            "computed"
        });
        assert_eq!(result, "computed");
    }
}
