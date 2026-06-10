//! # Lesson 09: Mocking Security Components (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use proptest::prelude::*;
use ring::digest;
use std::cell::Cell;
use std::collections::HashMap;

/// Trait for a hash function.
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
pub struct MockHasher {
    fixed_output: Vec<u8>,
}

impl MockHasher {
    pub fn new(fixed_output: Vec<u8>) -> Self {
        Self { fixed_output }
    }

    pub fn always_zeroes(size: usize) -> Self {
        Self {
            fixed_output: vec![0u8; size],
        }
    }
}

impl Hasher for MockHasher {
    fn hash(&self, _data: &[u8]) -> Vec<u8> {
        self.fixed_output.clone()
    }

    fn output_size(&self) -> usize {
        self.fixed_output.len()
    }
}

/// Trait for a key-value store.
pub trait KeyValueStore {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&mut self, key: &str, value: Vec<u8>);
    fn delete(&mut self, key: &str) -> bool;
}

/// In-memory mock key-value store.
pub struct MockStore {
    data: HashMap<String, Vec<u8>>,
}

impl MockStore {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl KeyValueStore for MockStore {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: Vec<u8>) {
        self.data.insert(key.to_string(), value);
    }

    fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }
}

/// Trait for a timing source.
pub trait Clock {
    fn now_millis(&self) -> u64;
}

/// Mock clock with controllable time.
pub struct MockClock {
    current_time: Cell<u64>,
}

impl MockClock {
    pub fn new(start_time: u64) -> Self {
        Self {
            current_time: Cell::new(start_time),
        }
    }

    pub fn advance(&self, millis: u64) {
        self.current_time.set(self.current_time.get() + millis);
    }
}

impl Clock for MockClock {
    fn now_millis(&self) -> u64 {
        self.current_time.get()
    }
}

/// Rate limiter using a pluggable Clock.
pub struct RateLimiter<C: Clock> {
    clock: C,
    window_millis: u64,
    max_requests: usize,
    timestamps: Vec<u64>,
}

impl<C: Clock> RateLimiter<C> {
    pub fn new(clock: C, window_millis: u64, max_requests: usize) -> Self {
        Self {
            clock,
            window_millis,
            max_requests,
            timestamps: Vec::new(),
        }
    }

    pub fn allow_request(&mut self) -> bool {
        let now = self.clock.now_millis();
        // Remove expired timestamps
        self.timestamps.retain(|&t| now - t < self.window_millis);
        if self.timestamps.len() < self.max_requests {
            self.timestamps.push(now);
            true
        } else {
            false
        }
    }

    pub fn current_count(&self) -> usize {
        self.timestamps.len()
    }
}

/// Compute a content fingerprint using a pluggable hasher.
pub fn content_fingerprint<H: Hasher>(hasher: &H, data: &[u8]) -> Vec<u8> {
    let hash = hasher.hash(data);
    let len = (data.len() as u32).to_be_bytes();
    let mut result = Vec::with_capacity(4 + hash.len());
    result.extend_from_slice(&len);
    result.extend_from_slice(&hash);
    result
}

/// Store a hash of data using pluggable hasher and store.
pub fn store_data_hash<S: KeyValueStore, H: Hasher>(
    store: &mut S,
    hasher: &H,
    key: &str,
    data: &[u8],
) {
    let hash = hasher.hash(data);
    store.set(key, hash);
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
        assert!(!limiter.allow_request());
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let clock = MockClock::new(0);
        let mut limiter = RateLimiter::new(clock, 1000, 2);

        limiter.allow_request();
        limiter.allow_request();
        assert!(!limiter.allow_request());

        // Advance past the window
        limiter.clock.advance(1001);
        assert!(limiter.allow_request());
    }

    // --- content_fingerprint ---

    #[test]
    fn test_content_fingerprint_real() {
        let hasher = Sha256Hasher;
        let fp = content_fingerprint(&hasher, b"hello");
        assert_eq!(fp.len(), 36);
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
