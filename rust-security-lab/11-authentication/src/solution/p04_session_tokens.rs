//! # Lesson 04: Session Tokens — Cryptographic Randomness and Entropy
//!
//! ## What Makes a Good Session Token?
//!
//! A session token must be:
//! - **Unpredictable**: Cannot be guessed by an attacker
//! - **Sufficiently long**: At least 128 bits of entropy (16 bytes)
//! - **Cryptographically random**: Uses CSPRNG, not `rand::random()`
//! - **Unique**: No two sessions share a token
//!
//! ## Entropy and Brute Force
//!
//! Entropy measures unpredictability in bits:
//! - 128 bits = 3.4 x 10^38 possible values — unguessable
//! - 64 bits = 1.8 x 10^19 — potentially brute-forceable
//! - 32 bits = 4.3 x 10^9 — trivially brute-forceable
//!
//! Formula: time_to_crack = 2^entropy / attempts_per_second
//! At 1 billion attempts/sec, 128 bits takes ~10^21 years.
//!
//! ## Attack Context
//!
//! Weak session tokens have led to real-world breaches:
//! - PHP sessions used predictable seeds (timestamp-based)
//! - Some frameworks used sequential IDs
//! - Short tokens (< 16 bytes) are brute-forceable
//!
//! **Defense**: Always use `OsRng` (CSPRNG) and at least 16 bytes.

use rand::RngCore;
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, SecretBox};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

/// A session associated with a user.
#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub data: HashMap<String, String>,
}

/// Generate a cryptographically random session token.
///
/// Requirements:
/// - Use `OsRng` (OS-provided CSPRNG) — NOT `rand::thread_rng()` for security tokens
/// - Generate at least `length` random bytes
/// - Encode as hex for URL-safe representation
///
/// Why OsRng?
/// - `thread_rng()` uses a userspace PRNG seeded from OsRng — fine for most purposes
/// - `OsRng` directly uses the OS CSPRNG (getrandom syscall) — no userspace state to leak
/// - For session tokens, the extra security margin of OsRng is preferred
pub fn generate_session_token(length: usize) -> SecretBox<String> {
    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    let token = hex::encode(&bytes);
    bytes.zeroize(); // Clear the raw bytes from memory
    SecretBox::new(Box::new(token))
}

/// Calculate the entropy (in bits) of a session token.
///
/// Entropy = length_in_bytes * 8
/// This assumes the bytes are cryptographically random.
pub fn token_entropy_bits(length_bytes: usize) -> u32 {
    (length_bytes as u32) * 8
}

/// Estimate brute-force time in seconds given entropy and attempts per second.
///
/// Expected attempts to find the token = 2^(entropy-1) (on average, search half the space)
pub fn estimated_brute_force_seconds(entropy_bits: u32, attempts_per_second: u64) -> f64 {
    if entropy_bits == 0 || attempts_per_second == 0 {
        return 0.0;
    }
    // 2^(n-1) / attempts_per_second
    // Use floating point to avoid overflow
    let expected_attempts = 2.0_f64.powi(entropy_bits as i32 - 1);
    expected_attempts / attempts_per_second as f64
}

/// A simple in-memory session store.
///
/// In production, this would be Redis, a database, or similar.
/// Tokens are stored as hex-encoded strings.
pub struct SessionStore {
    sessions: HashMap<String, Session>,
    token_length: usize,
    session_duration_secs: u64,
}

impl SessionStore {
    /// Create a new session store.
    ///
    /// `token_length` is in bytes (e.g., 32 for 256-bit tokens).
    /// `session_duration_secs` is how long sessions last.
    pub fn new(token_length: usize, session_duration_secs: u64) -> Self {
        Self {
            sessions: HashMap::new(),
            token_length,
            session_duration_secs,
        }
    }

    /// Create a new session for a user, returning the token.
    pub fn create_session(&mut self, user_id: &str) -> SecretBox<String> {
        let token = generate_session_token(self.token_length);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session = Session {
            user_id: user_id.to_string(),
            created_at: now,
            expires_at: now + self.session_duration_secs,
            data: HashMap::new(),
        };

        self.sessions.insert(token.expose_secret().clone(), session);
        token
    }

    /// Look up a session by token. Returns None if not found or expired.
    pub fn get_session(&self, token: &str) -> Option<&Session> {
        let session = self.sessions.get(token)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if session.expires_at < now {
            return None; // Expired
        }

        Some(session)
    }

    /// Destroy (invalidate) a session.
    pub fn destroy_session(&mut self, token: &str) -> bool {
        self.sessions.remove(token).is_some()
    }

    /// Number of active sessions.
    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_token_length() {
        let token = generate_session_token(16);
        // 16 bytes = 32 hex chars
        assert_eq!(token.expose_secret().len(), 32);
    }

    #[test]
    fn test_generate_session_token_hex_only() {
        let token = generate_session_token(32);
        assert!(
            token.expose_secret().chars().all(|c| c.is_ascii_hexdigit()),
            "Token should only contain hex characters"
        );
    }

    #[test]
    fn test_generate_session_token_uniqueness() {
        let t1 = generate_session_token(16);
        let t2 = generate_session_token(16);
        assert_ne!(
            t1.expose_secret(),
            t2.expose_secret(),
            "Two tokens should never be equal"
        );
    }

    #[test]
    fn test_token_entropy_bits() {
        assert_eq!(token_entropy_bits(16), 128);
        assert_eq!(token_entropy_bits(32), 256);
        assert_eq!(token_entropy_bits(8), 64);
    }

    #[test]
    fn test_brute_force_time_128_bits() {
        let seconds = estimated_brute_force_seconds(128, 1_000_000_000);
        // Should be astronomically large (> 10^20 years)
        let years = seconds / (365.25 * 24.0 * 3600.0);
        assert!(years > 1e20, "128-bit entropy should take > 10^20 years to brute force");
    }

    #[test]
    fn test_brute_force_time_32_bits() {
        let seconds = estimated_brute_force_seconds(32, 1_000_000_000);
        // 2^31 / 10^9 ≈ 2.15 seconds — trivially brute-forceable!
        assert!(seconds < 10.0, "32-bit entropy should be brute-forceable in seconds");
    }

    #[test]
    fn test_session_store_create_and_get() {
        let mut store = SessionStore::new(32, 3600);
        let token = store.create_session("user123");
        let session = store.get_session(token.expose_secret());
        assert!(session.is_some());
        assert_eq!(session.unwrap().user_id, "user123");
    }

    #[test]
    fn test_session_store_destroy() {
        let mut store = SessionStore::new(32, 3600);
        let token = store.create_session("user123");
        let token_str = token.expose_secret().clone();
        assert!(store.destroy_session(&token_str));
        assert!(store.get_session(&token_str).is_none());
    }

    #[test]
    fn test_session_store_count() {
        let mut store = SessionStore::new(16, 3600);
        assert_eq!(store.active_count(), 0);
        store.create_session("user1");
        store.create_session("user2");
        assert_eq!(store.active_count(), 2);
    }
}
