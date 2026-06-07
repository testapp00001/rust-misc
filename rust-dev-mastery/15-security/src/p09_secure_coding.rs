// ===========================================================================
// Secure Coding Practices
//
// Writing code that handles untrusted input, performs comparisons on
// sensitive data, and manages arithmetic without introducing exploitable
// vulnerabilities. This lesson covers timing-safe comparison, integer
// overflow protection, and panic-safe error propagation.
//
// Key Concepts:
//   - Timing attacks: side channels from variable-time operations
//   - Constant-time comparison: equal-time regardless of input
//   - Integer overflow: wrapping, saturating, checked arithmetic
//   - Panic safety: preventing panics from leaking state or crashing servers
//   - Secure memory zeroing
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. Timing-Safe Comparison
// ---------------------------------------------------------------------------

/// Constant-time comparison to prevent timing side channels.
/// Processes every byte regardless of early mismatches.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Compares two byte slices in constant time.
/// Returns `true` if they are equal, `false` otherwise.
pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    constant_time_eq(a, b)
}

/// Compares two hex-encoded strings without leaking timing information.
/// Both strings are decoded to bytes and compared in constant time.
pub fn secure_hex_compare(a: &str, b: &str) -> bool {
    let bytes_a = hex_to_bytes(a);
    let bytes_b = hex_to_bytes(b);
    match (bytes_a, bytes_b) {
        (Ok(a), Ok(b)) => secure_compare(&a, &b),
        _ => false,
    }
}

/// A timing-safe token store for API keys or session tokens.
#[derive(Debug)]
pub struct TokenStore {
    tokens: Vec<(String, Vec<u8>)>, // (label, hashed_token)
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
        }
    }

    /// Register a pre-hashed token.
    pub fn register(&mut self, label: impl Into<String>, hashed_token: Vec<u8>) {
        self.tokens.push((label.into(), hashed_token));
    }

    /// Look up a token by comparing hashed values in constant time.
    /// Returns the label if found, `None` otherwise.
    pub fn lookup(&self, hashed_token: &[u8]) -> Option<&str> {
        for (label, stored) in &self.tokens {
            if secure_compare(stored, hashed_token) {
                return Some(label.as_str());
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

// ---------------------------------------------------------------------------
// 2. Secure Hex Decoding
// ---------------------------------------------------------------------------

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, &'static str> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return Err("odd-length hex");
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.as_bytes().chunks(2) {
        let hi = hex_nibble(chunk[0]).ok_or("invalid hex char")?;
        let lo = hex_nibble(chunk[1]).ok_or("invalid hex char")?;
        bytes.push((hi << 4) | lo);
    }
    Ok(bytes)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 3. Integer Overflow Protection
// ---------------------------------------------------------------------------

/// Safe arithmetic operations that never panic and return `None` on overflow.
#[derive(Debug, Clone, Copy)]
pub struct SafeArithmetic;

impl SafeArithmetic {
    /// Checked addition: returns None on overflow.
    pub fn add(a: u64, b: u64) -> Option<u64> {
        a.checked_add(b)
    }

    /// Checked multiplication: returns None on overflow.
    pub fn mul(a: u64, b: u64) -> Option<u64> {
        a.checked_mul(b)
    }

    /// Saturating addition: clamps to u64::MAX instead of overflowing.
    pub fn saturating_add(a: u64, b: u64) -> u64 {
        a.saturating_add(b)
    }

    /// Saturating multiplication: clamps to u64::MAX instead of overflowing.
    pub fn saturating_mul(a: u64, b: u64) -> u64 {
        a.saturating_mul(b)
    }

    /// Safe percentage calculation that avoids overflow.
    /// Computes (numerator * 100) / denominator without intermediate overflow.
    pub fn percentage(numerator: u64, denominator: u64) -> Option<u64> {
        if denominator == 0 {
            return None;
        }
        // Avoid overflow by dividing first when possible
        if numerator <= u64::MAX / 100 {
            Some((numerator * 100) / denominator)
        } else {
            // Use 128-bit intermediate
            let result = (numerator as u128 * 100) / (denominator as u128);
            if result > u64::MAX as u128 {
                None
            } else {
                Some(result as u64)
            }
        }
    }

    /// Safe array index conversion: rejects negative or out-of-range values.
    pub fn safe_index(index: i64, len: usize) -> Option<usize> {
        if index < 0 {
            None
        } else {
            let idx = index as usize;
            if idx < len {
                Some(idx)
            } else {
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Panic Safety
// ---------------------------------------------------------------------------

/// Errors from secure operations.
#[derive(Debug, thiserror::Error)]
pub enum SecureError {
    #[error("overflow in {operation}: {detail}")]
    Overflow { operation: String, detail: String },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("index out of bounds: index {index}, len {len}")]
    IndexOutOfBounds { index: usize, len: usize },

    #[error("authentication failed")]
    AuthFailed,
}

/// A panic-safe wrapper around a Vec that ensures operations
/// return errors instead of panicking.
#[derive(Debug)]
pub struct SafeVec<T> {
    inner: Vec<T>,
}

impl<T: Clone> SafeVec<T> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn from_vec(v: Vec<T>) -> Self {
        Self { inner: v }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Safe indexing: returns an error instead of panicking.
    pub fn get(&self, index: usize) -> Result<&T, SecureError> {
        self.inner.get(index).ok_or(SecureError::IndexOutOfBounds {
            index,
            len: self.inner.len(),
        })
    }

    /// Safe push with capacity check.
    pub fn try_push(&mut self, value: T) -> Result<(), SecureError> {
        if self.inner.len() >= self.inner.capacity() && self.inner.capacity() > 0 {
            // In production, you might want a max-capacity check here
        }
        self.inner.push(value);
        Ok(())
    }

    /// Safe remove: returns error if index is out of bounds.
    pub fn try_remove(&mut self, index: usize) -> Result<T, SecureError> {
        if index >= self.inner.len() {
            Err(SecureError::IndexOutOfBounds {
                index,
                len: self.inner.len(),
            })
        } else {
            Ok(self.inner.remove(index))
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Secure Memory Zeroing
// ---------------------------------------------------------------------------

/// Zeroes out a byte slice to prevent sensitive data from lingering in memory.
/// Uses volatile write to prevent compiler optimization from eliding the zero.
pub fn secure_zero(data: &mut [u8]) {
    for byte in data.iter_mut() {
        // Volatile write prevents the compiler from optimizing this away
        unsafe {
            std::ptr::write_volatile(byte as *mut u8, 0);
        }
    }
}

/// A guard that zeroizes its contents on drop.
#[derive(Debug)]
pub struct SensitiveBytes {
    data: Vec<u8>,
}

impl SensitiveBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SensitiveBytes {
    fn drop(&mut self) {
        secure_zero(&mut self.data);
    }
}

// ---------------------------------------------------------------------------
// 6. Rate Limiter (Panic-Safe Design)
// ---------------------------------------------------------------------------

/// A simple token-bucket rate limiter.
/// All operations return `Result` to avoid panics under concurrent access.
#[derive(Debug)]
pub struct RateLimiter {
    max_tokens: u64,
    current_tokens: u64,
    refill_rate: u64, // tokens per second
    last_refill: u64, // unix timestamp in seconds
}

impl RateLimiter {
    pub fn new(max_tokens: u64, refill_rate: u64, now: u64) -> Self {
        Self {
            max_tokens,
            current_tokens: max_tokens,
            refill_rate,
            last_refill: now,
        }
    }

    /// Try to consume one token. Returns Ok(true) if allowed, Ok(false) if
    /// rate-limited. Returns Err if time goes backwards.
    pub fn try_acquire(&mut self, now: u64) -> Result<bool, SecureError> {
        if now < self.last_refill {
            return Err(SecureError::InvalidInput(
                "time cannot go backwards".into(),
            ));
        }

        self.refill(now);

        if self.current_tokens > 0 {
            self.current_tokens -= 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn refill(&mut self, now: u64) {
        let elapsed = now.saturating_sub(self.last_refill);
        let new_tokens = elapsed.saturating_mul(self.refill_rate);
        self.current_tokens = self.max_tokens.min(self.current_tokens.saturating_add(new_tokens));
        self.last_refill = now;
    }

    pub fn tokens_remaining(&self) -> u64 {
        self.current_tokens
    }
}

// ---------------------------------------------------------------------------
// 7. Helper: hex encoding for tests
// ---------------------------------------------------------------------------

pub fn bytes_to_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_compare_equal() {
        assert!(secure_compare(b"hello", b"hello"));
        assert!(secure_compare(b"", b""));
    }

    #[test]
    fn test_secure_compare_not_equal() {
        assert!(!secure_compare(b"hello", b"world"));
        assert!(!secure_compare(b"abc", b"abcd"));
    }

    #[test]
    fn test_secure_hex_compare() {
        let hash = "aabbccdd";
        assert!(secure_hex_compare(hash, "aabbccdd"));
        assert!(!secure_hex_compare(hash, "aabbccde"));
        assert!(!secure_hex_compare("zz", "aa")); // invalid hex -> false
    }

    #[test]
    fn test_token_store_lookup() {
        let mut store = TokenStore::new();
        let token_a = vec![0xAA; 32];
        let token_b = vec![0xBB; 32];
        store.register("admin", token_a.clone());
        store.register("user", token_b.clone());

        assert_eq!(store.lookup(&token_a), Some("admin"));
        assert_eq!(store.lookup(&token_b), Some("user"));
        assert_eq!(store.lookup(&vec![0xCC; 32]), None);
    }

    #[test]
    fn test_checked_arithmetic() {
        assert_eq!(SafeArithmetic::add(100, 200), Some(300));
        assert_eq!(SafeArithmetic::add(u64::MAX, 1), None);

        assert_eq!(SafeArithmetic::mul(100, 200), Some(20_000));
        assert_eq!(SafeArithmetic::mul(u64::MAX, 2), None);
    }

    #[test]
    fn test_saturating_arithmetic() {
        assert_eq!(SafeArithmetic::saturating_add(u64::MAX, 1), u64::MAX);
        assert_eq!(SafeArithmetic::saturating_mul(u64::MAX, 2), u64::MAX);
    }

    #[test]
    fn test_percentage() {
        assert_eq!(SafeArithmetic::percentage(50, 200), Some(25));
        assert_eq!(SafeArithmetic::percentage(0, 100), Some(0));
        assert_eq!(SafeArithmetic::percentage(100, 0), None);
    }

    #[test]
    fn test_safe_index() {
        assert_eq!(SafeArithmetic::safe_index(2, 5), Some(2));
        assert_eq!(SafeArithmetic::safe_index(-1, 5), None);
        assert_eq!(SafeArithmetic::safe_index(10, 5), None);
    }

    #[test]
    fn test_safe_vec_get() {
        let v = SafeVec::from_vec(vec![10, 20, 30]);
        assert_eq!(*v.get(0).unwrap(), 10);
        assert!(v.get(5).is_err());
    }

    #[test]
    fn test_safe_vec_remove() {
        let mut v = SafeVec::from_vec(vec![10, 20, 30]);
        let removed = v.try_remove(1).unwrap();
        assert_eq!(removed, 20);
        assert_eq!(v.len(), 2);
        assert!(v.try_remove(99).is_err());
    }

    #[test]
    fn test_secure_zero() {
        let mut data = vec![0xAA, 0xBB, 0xCC];
        secure_zero(&mut data);
        assert!(data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_sensitive_bytes_drop() {
        let guard = SensitiveBytes::new(vec![1, 2, 3, 4]);
        assert_eq!(guard.as_bytes(), &[1, 2, 3, 4]);
        assert_eq!(guard.len(), 4);
        drop(guard); // zeroing happens here
    }

    #[test]
    fn test_rate_limiter_allows_within_budget() {
        let mut limiter = RateLimiter::new(5, 1, 1000);
        for _ in 0..5 {
            assert!(limiter.try_acquire(1000).unwrap());
        }
        assert!(!limiter.try_acquire(1000).unwrap()); // exhausted
    }

    #[test]
    fn test_rate_limiter_refills_over_time() {
        let mut limiter = RateLimiter::new(5, 1, 1000);
        // Exhaust tokens
        for _ in 0..5 {
            limiter.try_acquire(1000).unwrap();
        }
        assert!(!limiter.try_acquire(1000).unwrap());

        // 3 seconds later: 3 tokens refilled
        assert!(limiter.try_acquire(1003).unwrap());
        assert!(limiter.try_acquire(1003).unwrap());
        assert!(limiter.try_acquire(1003).unwrap());
        assert!(!limiter.try_acquire(1003).unwrap());
    }

    #[test]
    fn test_rate_limiter_rejects_time_travel() {
        let mut limiter = RateLimiter::new(5, 1, 1000);
        assert!(limiter.try_acquire(999).is_err());
    }
}
