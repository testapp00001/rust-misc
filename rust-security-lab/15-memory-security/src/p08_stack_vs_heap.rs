//! # Lesson 08: Stack vs Heap for Secrets
//!
//! ## The Problem
//!
//! Where you store a secret affects its security properties:
//!
//! ```text
//! Stack:                          Heap:
//! ├── Fixed size                  ├── Dynamic size
//! ├── Automatic cleanup (scope)   ├── Manual cleanup (drop/free)
//! ├── Harder to inspect remotely  ├── Easier to inspect (pointers)
//! ├── Can overflow                ├── No stack overflow risk
//! └── No allocator reuse          └── Allocator may reuse memory
//! ```
//!
//! ## When to Use Stack
//!
//! - **Small, fixed-size secrets**: Keys (32 bytes), nonces (12 bytes), IVs (16 bytes)
//! - **Short-lived**: Used within a single function scope
//! - **No need for dynamic sizing**: Size known at compile time
//!
//! ```rust,ignore
//! fn encrypt() {
//!     let key: [u8; 32] = derive_key();  // Stack — auto-cleaned on scope exit
//!     let nonce: [u8; 12] = generate_nonce();  // Stack
//!     // ... use key and nonce ...
//! } // key and nonce are gone here
//! ```
//!
//! ## When to Use Heap
//!
//! - **Large secrets**: Long messages, large key sets
//! - **Dynamic size**: Size not known at compile time
//! - **Need ownership transfer**: Pass between functions/threads
//!
//! ## Security Implications
//!
//! | Property | Stack | Heap |
//! |----------|-------|------|
//! | Zeroize on scope exit | Must use zeroize | Must use zeroize |
//! | Reuse by allocator | No (grows/shrinks) | Yes — old data persists! |
//! | Core dump exposure | Yes | Yes |
//! | Swap exposure | Yes | Yes (more likely — longer lifetime) |
//! | Inspector access | Harder (no pointers) | Easier (pointers in metadata) |

use zeroize::Zeroize;

/// Exercise 1: Implement `StackSecret` — a fixed-size secret on the stack.
///
/// Requirements:
/// - Generic over size N (const generic)
/// - Stores `[u8; N]` directly (no heap allocation)
/// - Zeroizes on drop
/// - Provides `new(data: &[u8])` — copies min(len, N) bytes
/// - Provides `as_slice(&self) -> &[u8]`
///
/// Hints:
/// - `#[derive(Zeroize)]` and `#[zeroize(drop)]` work with const generics
/// - Copy bytes with a simple loop
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct StackSecret<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> StackSecret<N> {
    pub fn new(data: &[u8]) -> Self {
        todo!("Create a stack-allocated secret")
    }

    pub fn as_slice(&self) -> &[u8] {
        todo!("Return reference to the data")
    }

    pub fn len(&self) -> usize {
        N
    }
}

/// Exercise 2: Implement `HeapSecret` — a heap-allocated secret with proper cleanup.
///
/// Requirements:
/// - Stores data in `Vec<u8>` (heap)
/// - Implements `Zeroize` — zeroizes the Vec on drop
/// - `new(data: Vec<u8>)` takes ownership
/// - `expose(&self) -> &[u8]` returns reference to data
///
/// Hints:
/// - Implement `Zeroize` manually: iterate and zero each byte
/// - Or use the blanket impl for `Vec<u8>` from `zeroize`
pub struct HeapSecret {
    data: Vec<u8>,
}

impl HeapSecret {
    pub fn new(data: Vec<u8>) -> Self {
        todo!("Create a heap-allocated secret")
    }

    pub fn expose(&self) -> &[u8] {
        todo!("Return reference to the data")
    }

    pub fn len(&self) -> usize {
        todo!("Return the length")
    }
}

/// Exercise 3: Implement `choose_storage` — decide stack vs heap based on size.
///
/// Returns a `Vec<u8>` either way, but demonstrates the decision logic.
/// For sizes <= 32, allocate on stack first (then copy to Vec for return).
/// For sizes > 32, allocate directly on heap.
///
/// Requirements:
/// - Take a `size` parameter
/// - Generate random bytes of that size
/// - Return the bytes as `Vec<u8>`
///
/// Hints:
/// - Use `rand::Rng::gen_range` or `rand::Rng::fill`
/// - This is more about demonstrating the concept
pub fn choose_storage(size: usize) -> Vec<u8> {
    todo!("Generate random bytes, demonstrating stack vs heap decision")
}

/// Exercise 4: Implement `stack_vs_heap_benchmark_info` — return info about
/// storage characteristics.
///
/// Returns a tuple of (stack_properties, heap_properties) as string descriptions.
///
/// Hints:
/// - Return hardcoded descriptive strings about each storage type
pub fn stack_vs_heap_info() -> (&'static str, &'static str) {
    todo!("Return descriptive info about stack vs heap characteristics")
}

/// Exercise 5: Implement `zeroize_on_scope_exit` — demonstrate RAII cleanup.
///
/// Creates a stack secret, passes it to a closure, and ensures it's zeroed
/// after the scope exits.
///
/// Hints:
/// - Create a `StackSecret<32>`
/// - Execute the closure with the secret
/// - The StackSecret will be dropped (and zeroed) at the end of the function
pub fn zeroize_on_scope_exit<F: FnOnce(&[u8])>(data: &[u8], f: F) {
    todo!("Create a stack secret, use it in a closure, drop+zeroize on exit")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_secret_new() {
        let secret = StackSecret::<16>::new(b"0123456789abcdef");
        assert_eq!(secret.as_slice(), b"0123456789abcdef");
        assert_eq!(secret.len(), 16);
    }

    #[test]
    fn test_stack_secret_shorter_input() {
        let secret = StackSecret::<16>::new(b"hello");
        assert_eq!(&secret.as_slice()[..5], b"hello");
        assert!(secret.as_slice()[5..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_stack_secret_longer_input() {
        let secret = StackSecret::<4>::new(b"toolong");
        assert_eq!(secret.as_slice(), b"tool");
    }

    #[test]
    fn test_stack_secret_drop_zeroizes() {
        let mut secret = StackSecret::<32>::new(&[0x42; 32]);
        assert!(secret.as_slice().iter().all(|&b| b == 0x42));
        secret.zeroize();
        assert!(secret.as_slice().iter().all(|&b| b == 0));
    }

    #[test]
    fn test_heap_secret_basic() {
        let secret = HeapSecret::new(vec![0xAA; 64]);
        assert_eq!(secret.len(), 64);
        assert_eq!(secret.expose(), &[0xAA; 64]);
    }

    #[test]
    fn test_heap_secret_drop_zeroizes() {
        let secret = HeapSecret::new(vec![0x42; 128]);
        assert!(secret.expose().iter().all(|&b| b == 0x42));
        drop(secret);
        // After drop, data should be zeroed (we trust the Zeroize impl)
    }

    #[test]
    fn test_choose_storage_size() {
        let data = choose_storage(48);
        assert_eq!(data.len(), 48);
    }

    #[test]
    fn test_zeroize_on_scope_exit_runs() {
        let data = vec![0x42; 32];
        zeroize_on_scope_exit(&data, |secret| {
            assert_eq!(secret.len(), 32);
        });
    }
}
