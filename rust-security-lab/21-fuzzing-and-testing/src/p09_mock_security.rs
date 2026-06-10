//! # Lesson 09: Mocking Security Components
//!
//! ## Why Mock Security Components?
//!
//! When testing code that depends on cryptographic operations, key storage, or
//! network services, we need to isolate the unit under test. Mocking replaces
//! real dependencies with controllable substitutes.
//!
//! ## Mock Strategies in Rust
//!
//! 1. **Trait-based mocking**: Define a trait, implement mock and real versions
//! 2. **Function pointer injection**: Pass `fn` pointers instead of hard-coding
//! 3. **Feature-flag mocking**: Use `#[cfg(feature = "mock")]` to swap implementations
//!
//! ## Security Perspective
//!
//! ### Attack: Mock Leaking to Production
//! If mock implementations are accidentally compiled into production, security is
//! completely bypassed (e.g., always-return-true authentication).
//!
//! ### Defense: Feature-Gated Mocks
//! - Mock implementations behind `#[cfg(test)]` or `#[cfg(feature = "mock")]`
//! - Never enable mock features in release builds
//! - CI should test both mock and real implementations

use ring::digest;

/// Trait for a hash function that can be mocked.
pub trait Hasher {
    fn hash(&self, data: &[u8]) -> Vec<u8>;
    fn output_size(&self) -> usize;
}

/// Real SHA-256 implementation.
pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA256, data).as_ref().to_vec()
    }

    fn output_size(&self) -> usize {
        32
    }
}

/// Mock hasher that returns a fixed value.
///
/// Useful for testing code that depends on a hasher without actually computing hashes.
pub struct MockHasher {
    fixed_output: Vec<u8>,
}

impl MockHasher {
    pub fn new(fixed_output: Vec<u8>) -> Self {
        todo!("Initialize mock hasher")
    }

    pub fn always_zeroes(size: usize) -> Self {
        todo!("Create mock hasher that always returns zeroes")
    }
}

impl Hasher for MockHasher {
    fn hash(&self, _data: &[u8]) -> Vec<u8> {
        todo!("Return fixed output regardless of input")
    }

    fn output_size(&self) -> usize {
        todo!("Return length of fixed output")
    }
}

/// Trait for a key-value store (simulating secret storage).
pub trait KeyValueStore {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&mut self, key: &str, value: Vec<u8>);
    fn delete(&mut self, key: &str) -> bool;
}

/// In-memory key-value store (mock).
pub struct MockStore {
    data: std::collections::HashMap<String, Vec<u8>>,
}

impl MockStore {
    pub fn new() -> Self {
        todo!("Initialize empty mock store")
    }
}

impl KeyValueStore for MockStore {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        todo!("Get value from in-memory store")
    }

    fn set(&mut self, key: &str, value: Vec<u8>) {
        todo!("Set value in in-memory store")
    }

    fn delete(&mut self, key: &str) -> bool {
        todo!("Delete value from in-memory store")
    }
}

/// Trait for a "timing" source, allowing tests to control time.
pub trait Clock {
    fn now_millis(&self) -> u64;
}

/// Mock clock that returns a controllable time.
pub struct MockClock {
    current_time: std::cell::Cell<u64>,
}

impl MockClock {
    pub fn new(start_time: u64) -> Self {
        todo!("Initialize mock clock")
    }

    pub fn advance(&self, millis: u64) {
        todo!("Advance the clock by millis")
    }
}

impl Clock for MockClock {
    fn now_millis(&self) -> u64 {
        todo!("Return current mock time")
    }
}

/// A rate limiter that uses a Clock trait for testability.
///
/// Allows at most `max_requests` per `window_millis` milliseconds.
pub struct RateLimiter<C: Clock> {
    clock: C,
    window_millis: u64,
    max_requests: usize,
    timestamps: Vec<u64>,
}

impl<C: Clock> RateLimiter<C> {
    pub fn new(clock: C, window_millis: u64, max_requests: usize) -> Self {
        todo!("Initialize rate limiter")
    }

    /// Try to allow a request. Returns true if allowed, false if rate limited.
    pub fn allow_request(&mut self) -> bool {
        todo!("Check rate limit and record timestamp")
    }

    /// Return the number of requests in the current window.
    pub fn current_count(&self) -> usize {
        todo!("Count requests in current window")
    }
}

/// A function that uses a Hasher trait for testability.
///
/// Computes a "content fingerprint" by hashing the data and prepending a 4-byte length.
pub fn content_fingerprint<H: Hasher>(hasher: &H, data: &[u8]) -> Vec<u8> {
    todo!("Compute content fingerprint with pluggable hasher")
}

/// A function that uses a KeyValueStore for testability.
///
/// Stores a hash of the data (not the data itself) under the given key.
pub fn store_data_hash<S: KeyValueStore, H: Hasher>(
    store: &mut S,
    hasher: &H,
    key: &str,
    data: &[u8],
) {
    todo!("Store hash of data in key-value store")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- MockHasher ---

    #[test]
    fn test_mock_hasher_fixed_output() {
        let mock = MockHasher::new(vec![0xAB; 32]);
        assert_eq!(mock.hash(b"anything"), vec![0xAB; 32]);
        assert_eq!(mock.hash(b"something else"), vec![0xAB; 32]);
    }

    #[test]
    fn test_mock_hasher_output_size() {
        let mock = MockHasher::new(vec![0; 16]);
        assert_eq!(mock.output_size(), 16);
    }

    #[test]
    fn test_mock_hasher_always_zeroes() {
        let mock = MockHasher::always_zeroes(32);
        assert_eq!(mock.hash(b"test"), vec![0u8; 32]);
        assert_eq!(mock.output_size(), 32);
    }

    // --- MockStore ---

    #[test]
    fn test_mock_store_basic() {
        let mut store = MockStore::new();
        assert!(store.get("key").is_none());

        store.set("key", vec![1, 2, 3]);
        assert_eq!(store.get("key"), Some(vec![1, 2, 3]));

        assert!(store.delete("key"));
        assert!(store.get("key").is_none());
    }

    #[test]
    fn test_mock_store_delete_nonexistent() {
        let mut store = MockStore::new();
        assert!(!store.delete("missing"));
    }

    // --- MockClock and RateLimiter ---

    #[test]
    fn test_mock_clock() {
        let clock = MockClock::new(1000);
        assert_eq!(clock.now_millis(), 1000);
        clock.advance(500);
        assert_eq!(clock.now_millis(), 1500);
    }

    #[test]
    fn test_rate_limiter_basic() {
        let clock = MockClock::new(0);
        let mut limiter = RateLimiter::new(clock, 1000, 3);

        assert!(limiter.allow_request());
        assert!(limiter.allow_request());
        assert!(limiter.allow_request());
        assert!(!limiter.allow_request()); // 4th should be rate limited
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let clock = MockClock::new(0);
        let mut limiter = RateLimiter::new(clock, 1000, 2);

        limiter.allow_request();
        limiter.allow_request();
        assert!(!limiter.allow_request());

        // Advance past the window
        // Note: we need access to the clock through the limiter
        // For this test, we rely on the MockClock's interior mutability
    }

    // --- content_fingerprint ---

    #[test]
    fn test_content_fingerprint_real() {
        let hasher = Sha256Hasher;
        let fp = content_fingerprint(&hasher, b"hello");
        // 4 bytes length + 32 bytes hash = 36 bytes
        assert_eq!(fp.len(), 36);
        // First 4 bytes should be the length of "hello" = 5
        assert_eq!(&fp[0..4], &5u32.to_be_bytes());
    }

    #[test]
    fn test_content_fingerprint_mock() {
        let mock = MockHasher::new(vec![0xAA; 32]);
        let fp = content_fingerprint(&mock, b"test");
        assert_eq!(fp.len(), 36);
        assert_eq!(&fp[4..], vec![0xAA; 32].as_slice());
    }

    // --- store_data_hash ---

    #[test]
    fn test_store_data_hash() {
        let mut store = MockStore::new();
        let hasher = Sha256Hasher;

        store_data_hash(&mut store, &hasher, "file1", b"important data");

        let stored = store.get("file1").unwrap();
        // Should be the SHA-256 hash, not the original data
        assert_eq!(stored.len(), 32);
        assert_ne!(stored, b"important data");
    }

    #[test]
    fn test_store_data_hash_mock() {
        let mut store = MockStore::new();
        let mock = MockHasher::new(vec![0xBB; 32]);

        store_data_hash(&mut store, &mock, "k", b"data");

        assert_eq!(store.get("k"), Some(vec![0xBB; 32]));
    }

    // --- Trait object usage ---

    #[test]
    fn test_trait_object_hasher() {
        let hashers: Vec<Box<dyn Hasher>> = vec![
            Box::new(Sha256Hasher),
            Box::new(MockHasher::new(vec![0; 32])),
        ];
        for hasher in &hashers {
            let h = hasher.hash(b"test");
            assert_eq!(h.len(), hasher.output_size());
        }
    }

    proptest::proptest! {
        #[test]
        fn test_sha256_hasher_output_size(
            data in prop::collection::vec(prop::num::u8::ANY, 0..1000)
        ) {
            let hasher = Sha256Hasher;
            let hash = hasher.hash(&data);
            prop_assert_eq!(hash.len(), 32);
            prop_assert_eq!(hash.len(), hasher.output_size());
        }

        #[test]
        fn test_mock_store_roundtrip(
            key in "[a-z]{1,20}",
            value in prop::collection::vec(prop::num::u8::ANY, 0..100)
        ) {
            let mut store = MockStore::new();
            store.set(&key, value.clone());
            prop_assert_eq!(store.get(&key), Some(value));
        }
    }
}
