//! # Lesson 03: JWT Attacks — alg=none, Weak Secret, Key Confusion
//!
//! ## Attack 1: alg=none Bypass
//!
//! The `alg` field in the JWT header tells the server how to verify the signature.
//! If an attacker sets `alg: "none"`, some libraries skip verification entirely.
//!
//! ```text
//! Original: {"alg":"HS256","typ":"JWT"}.{"sub":"user","role":"user"}.<sig>
//! Attacked: {"alg":"none","typ":"JWT"}.{"sub":"admin","role":"admin"}.
//! ```
//!
//! **Defense**: NEVER trust `alg` from the token. Enforce it server-side.
//!
//! ## Attack 2: Weak Secret Brute Force
//!
//! If the HMAC secret is weak (short, dictionary word, predictable), an attacker
//! can capture any valid token and brute-force the secret offline.
//!
//! With a weak secret like "password123", an attacker can:
//! 1. Capture a valid JWT
//! 2. Try millions of candidate secrets per second
//! 3. Once found, forge tokens for any user/role
//!
//! **Defense**: Use 256+ bit cryptographically random secrets.
//!
//! ## Attack 3: Key Confusion (RS256 -> HS256)
//!
//! If a server uses RS256 (asymmetric) but an attacker changes `alg` to HS256,
//! the server might use the RSA public key as the HMAC secret.
//!
//! Since the attacker knows the public key, they can sign tokens!
//!
//! **Defense**: Validate algorithm against expected value, not from token.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

use super::p01_jwt_creation::{base64url_encode, JwtClaims, JwtHeader};

/// Forge a JWT with alg=none (no signature).
///
/// This simulates the attack where an attacker:
/// 1. Sets the header algorithm to "none"
/// 2. Modifies the payload (e.g., changes role to "admin")
/// 3. Omits the signature entirely
///
/// The resulting token is: `base64url(header).base64url(payload).`
pub fn forge_alg_none(claims: serde_json::Value) -> String {
    todo!("Forge a JWT with alg=none — no signature needed")
}

/// Verify a JWT, demonstrating the alg=none vulnerability.
///
/// This is a VULNERABLE verifier that trusts the `alg` field from the token.
/// It shows why this is dangerous — DO NOT use this pattern in production!
pub fn verify_vulnerable(token: &str, secret: &[u8]) -> Result<serde_json::Value, String> {
    todo!("Implement a VULNERABLE JWT verifier that trusts alg from the token")
}

/// Verify a JWT with a SECURE algorithm allowlist.
///
/// This is the correct way to handle the `alg` field:
/// - Maintain a server-side allowlist of accepted algorithms
/// - Reject any token whose `alg` is not in the list
/// - Never trust the algorithm from the token itself
pub fn verify_secure(
    token: &str,
    secret: &[u8],
    allowed_algorithms: &[&str],
) -> Result<serde_json::Value, String> {
    todo!("Implement a SECURE JWT verifier with algorithm allowlist")
}

/// Brute-force a weak JWT secret by trying candidate secrets.
///
/// Given a valid JWT token, this tries each candidate and returns the
/// first secret that produces a matching signature.
///
/// In a real attack, an attacker would use a wordlist of millions of candidates.
/// This function demonstrates the concept with a small list.
pub fn brute_force_secret(token: &str, candidates: &[&str]) -> Option<Vec<u8>> {
    todo!("Brute-force a weak JWT secret from a list of candidates")
}

/// Generate a cryptographically secure JWT secret of the given byte length.
///
/// For HMAC-SHA256, use at least 32 bytes (256 bits).
/// More bytes don't help beyond the hash output size.
pub fn generate_secure_secret(length: usize) -> Vec<u8> {
    todo!("Generate a cryptographically secure random secret")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p01_jwt_creation::{create_jwt, encode_jwt, JwtClaims};

    const WEAK_SECRET: &[u8] = b"password";
    const STRONG_SECRET: &[u8] = b"super-secret-key-at-least-32-bytes-long!!";

    fn sample_claims() -> JwtClaims {
        JwtClaims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1700000000,
            iss: None,
            aud: None,
        }
    }

    #[test]
    fn test_forge_alg_none_basic() {
        let claims = serde_json::json!({
            "sub": "user123",
            "role": "admin",
            "exp": 9999999999u64
        });
        let token = forge_alg_none(claims.clone());
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
        // Signature part should be empty
        assert!(parts[2].is_empty());
    }

    #[test]
    fn test_alg_none_bypasses_vulnerable_verifier() {
        // Attacker forges claims with elevated privileges
        let claims = serde_json::json!({
            "sub": "victim",
            "role": "admin",
            "exp": 9999999999u64
        });
        let token = forge_alg_none(claims);
        // Vulnerable verifier accepts alg=none without checking signature
        let result = verify_vulnerable(&token, STRONG_SECRET);
        assert!(result.is_ok(), "Vulnerable verifier should accept alg=none token");
        assert_eq!(result.unwrap()["role"], "admin");
    }

    #[test]
    fn test_alg_none_rejected_by_secure_verifier() {
        let claims = serde_json::json!({
            "sub": "victim",
            "role": "admin",
            "exp": 9999999999u64
        });
        let token = forge_alg_none(claims);
        let result = verify_secure(&token, STRONG_SECRET, &["HS256"]);
        assert!(result.is_err(), "Secure verifier should reject alg=none token");
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_brute_force_weak_secret() {
        let jwt = create_jwt(sample_claims(), WEAK_SECRET);
        let token = encode_jwt(&jwt);

        let candidates = vec!["wrong1", "wrong2", "password", "wrong3"];
        let found = brute_force_secret(&token, &candidates);
        assert!(found.is_some(), "Should find the weak secret");
        assert_eq!(found.unwrap(), WEAK_SECRET);
    }

    #[test]
    fn test_brute_force_strong_secret_not_found() {
        let jwt = create_jwt(sample_claims(), STRONG_SECRET);
        let token = encode_jwt(&jwt);

        let candidates = vec!["password", "123456", "admin", "letmein"];
        let found = brute_force_secret(&token, &candidates);
        assert!(found.is_none(), "Strong secret should not be in common wordlists");
    }

    #[test]
    fn test_generate_secure_secret_length() {
        let secret = generate_secure_secret(32);
        assert_eq!(secret.len(), 32);
    }

    #[test]
    fn test_generate_secure_secret_random() {
        let s1 = generate_secure_secret(32);
        let s2 = generate_secure_secret(32);
        assert_ne!(s1, s2, "Two generated secrets should differ");
    }

    #[test]
    fn test_secure_verifier_accepts_valid_token() {
        let jwt = create_jwt(sample_claims(), STRONG_SECRET);
        let token = encode_jwt(&jwt);
        let result = verify_secure(&token, STRONG_SECRET, &["HS256"]);
        assert!(result.is_ok());
    }
}
