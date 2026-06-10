//! # Lesson 06: Memory Forensics Defense — Minimize Secret Lifetime
//!
//! ## The Problem
//!
//! Memory forensics tools (Volatility, Rekall, LiME) can dump a process's entire
//! address space or an entire system's RAM. If secrets are sitting in memory for
//! a long time, they're easy to find.
//!
//! ```text
//! Forensic analysis of memory dump:
//! ┌─────────────────────────────────────────────┐
//! │ grep -r "BEGIN RSA PRIVATE KEY" memory.dmp  │
//! │ → Found at offset 0x7fff1234                │
//! │ → Found at offset 0x7fff5678 (copy in cache)│
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Defense Principles
//!
//! 1. **Minimize lifetime**: Hold secrets in memory for the shortest time possible
//! 2. **Zero on drop**: Overwrite secrets before freeing
//! 3. **Avoid copies**: Don't clone secret buffers; pass references
//! 4. **Pin in memory**: Use `mlock` to prevent swap to disk
//! 5. **Scope secrets**: Use RAII to ensure cleanup
//!
//! ## Attack: Cold Boot Attack
//!
//! Even after power off, DRAM retains data for seconds to minutes (longer when cooled).
//! An attacker can freeze RAM, remove it, and read it in another machine.
//!
//! Defense: Minimize the time secrets spend in RAM. Zero immediately after use.

use zeroize::Zeroize;

/// Exercise 1: Implement `SecretScope` — RAII wrapper that zeroizes on drop.
///
/// Requirements:
/// - Holds a `Vec<u8>` secret
/// - On `Drop`, zeroizes the data
/// - Provides `with_secret<F, R>(&self, f: F) -> R` that gives access to the
///   secret and returns the closure's result
///
/// Hints:
/// - Store `data: Vec<u8>`
/// - In `Drop`, call `self.data.zeroize()`
/// - `with_secret` passes `&[u8]` to the closure
pub struct SecretScope {
    data: Vec<u8>,
}

impl SecretScope {
    pub fn new(secret: Vec<u8>) -> Self {
        todo!("Create a new SecretScope")
    }

    /// Execute a closure with access to the secret bytes.
    pub fn with_secret<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        todo!("Pass the secret to the closure")
    }

    pub fn len(&self) -> usize {
        todo!("Return the length of the secret")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Check if the secret is empty")
    }
}

/// Exercise 2: Implement `ephemeral_key` — generate a key, use it, and immediately zero.
///
/// Requirements:
/// - Generate a random 32-byte key using `rand`
/// - Pass it to the closure for use
/// - Zeroize the key after the closure returns
/// - Return the closure's result
///
/// Hints:
/// - Use `rand::random::<[u8; 32]>()` or `rand::Rng::fill`
/// - Call `key.zeroize()` after the closure
pub fn ephemeral_key<F, R>(f: F) -> R
where
    F: FnOnce(&[u8]) -> R,
{
    todo!("Generate a temporary key, use it, then zeroize")
}

/// Exercise 3: Implement `SecureVec` — a Vec that tracks and minimizes copies.
///
/// Requirements:
/// - Stores data in a `Vec<u8>`
/// - Implements `Zeroize` via manual implementation (zero all bytes)
/// - Provides `expose(&self) -> &[u8]` (intentional exposure)
/// - The `Drop` impl must zeroize before freeing
///
/// Hints:
/// - Implement `Zeroize` trait manually: iterate and set each byte to 0
/// - In `Drop`, call `self.zeroize()`
pub struct SecureVec {
    data: Vec<u8>,
}

impl SecureVec {
    pub fn new(data: Vec<u8>) -> Self {
        todo!("Create a SecureVec")
    }

    pub fn expose(&self) -> &[u8] {
        todo!("Return reference to the data")
    }

    pub fn len(&self) -> usize {
        todo!("Return length")
    }
}

/// Exercise 4: Implement `zeroize_range` — zero a range of a slice.
///
/// Requirements:
/// - Zero bytes from `start` to `end` (exclusive) within the slice
/// - Use the `zeroize` crate pattern (volatile write)
/// - If range is out of bounds, zero the valid portion
///
/// Hints:
/// - Clamp end to `data.len()`
/// - Call `.zeroize()` on the subslice: `data[start..clamped_end].zeroize()`
pub fn zeroize_range(data: &mut [u8], start: usize, end: usize) {
    todo!("Zero a specific range of bytes")
}

/// Exercise 5: Implement `minimize_secret_lifetime` — a pattern for secure computation.
///
/// Takes a secret, performs a computation, and ensures the secret is zeroed
/// even if the computation panics.
///
/// Requirements:
/// - Take a `Vec<u8>` secret and a closure
/// - Wrap the secret in `SecretScope`
/// - Call the closure with the secret
/// - The `SecretScope` drop will zeroize even on panic (RAII)
/// - Return the closure's result
///
/// Hints:
/// - Create `SecretScope::new(secret)`
/// - Call `scope.with_secret(f)`
/// - scope is dropped at end of function, zeroizing the data
pub fn minimize_secret_lifetime<F, R>(secret: Vec<u8>, f: F) -> R
where
    F: FnOnce(&[u8]) -> R,
{
    todo!("Use RAII to ensure secret is zeroed after use")
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
        // Just verify it ran; the key is zeroed after
        let _ = result;
    }

    #[test]
    fn test_ephemeral_key_different_each_time() {
        let key1 = ephemeral_key(|key| key.to_vec());
        let key2 = ephemeral_key(|key| key.to_vec());
        // Statistically, two random 32-byte keys should differ
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
        let mut sv = SecureVec::new(vec![0x42; 32]);
        // Verify data is there
        assert!(sv.expose().iter().all(|&b| b == 0x42));
        // Drop will zeroize (we trust the impl)
        drop(sv);
    }

    #[test]
    fn test_zeroize_range() {
        let mut data = vec![0xFF; 16];
        zeroize_range(&mut data, 4, 12);
        // Bytes 0-3 should still be 0xFF
        assert!(data[..4].iter().all(|&b| b == 0xFF));
        // Bytes 4-11 should be 0
        assert!(data[4..12].iter().all(|&b| b == 0));
        // Bytes 12-15 should still be 0xFF
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
