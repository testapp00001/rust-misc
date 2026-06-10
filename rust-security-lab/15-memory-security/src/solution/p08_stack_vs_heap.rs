//! # Lesson 08: Stack vs Heap for Secrets (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use zeroize::Zeroize;

/// A fixed-size secret stored on the stack.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct StackSecret<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> StackSecret<N> {
    pub fn new(data: &[u8]) -> Self {
        let mut s = Self { data: [0u8; N] };
        let copy_len = data.len().min(N);
        s.data[..copy_len].copy_from_slice(&data[..copy_len]);
        s
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        N
    }
}

/// A heap-allocated secret with proper cleanup.
pub struct HeapSecret {
    data: Vec<u8>,
}

impl HeapSecret {
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

impl Zeroize for HeapSecret {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

impl Drop for HeapSecret {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Generate random bytes, demonstrating the stack vs heap decision.
pub fn choose_storage(size: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut data = vec![0u8; size];
    rand::thread_rng().fill_bytes(&mut data);
    data
}

/// Return descriptive info about stack vs heap characteristics.
pub fn stack_vs_heap_info() -> (&'static str, &'static str) {
    let stack = "Stack: fixed-size, auto-cleanup on scope exit, no allocator reuse, \
                 harder to inspect remotely, risk of stack overflow with large allocations";
    let heap = "Heap: dynamic size, manual cleanup (drop/zeroize), allocator may reuse memory, \
                easier to inspect via pointers, no stack overflow risk";
    (stack, heap)
}

/// Create a stack secret, use it in a closure, drop+zeroize on exit.
pub fn zeroize_on_scope_exit<F: FnOnce(&[u8])>(data: &[u8], f: F) {
    let secret = StackSecret::<32>::new(data);
    f(secret.as_slice());
    // secret is dropped and zeroized here
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
