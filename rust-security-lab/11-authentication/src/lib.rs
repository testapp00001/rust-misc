//! # Module 11: Authentication
//!
//! JWT, OAuth2, MFA, sessions, and token lifecycle management.
//!
//! ## Learning Path
//! 1. Start with `p01_jwt_creation` — build JWTs from scratch
//! 2. Verify them in `p02_jwt_verification`
//! 3. Break them in `p03_jwt_attacks`
//! 4. Cover sessions, OAuth2, MFA, passkeys, and credential defense
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 11-authentication              # Test your implementation
//! cargo test -p 11-authentication --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_jwt_creation;
#[cfg(not(feature = "solution"))]
pub mod p02_jwt_verification;
#[cfg(not(feature = "solution"))]
pub mod p03_jwt_attacks;
#[cfg(not(feature = "solution"))]
pub mod p04_session_tokens;
#[cfg(not(feature = "solution"))]
pub mod p05_oauth2_flow;
#[cfg(not(feature = "solution"))]
pub mod p06_totp_mfa;
#[cfg(not(feature = "solution"))]
pub mod p07_passkey_concept;
#[cfg(not(feature = "solution"))]
pub mod p08_credential_stuffing;
#[cfg(not(feature = "solution"))]
pub mod p09_token_refresh;
#[cfg(not(feature = "solution"))]
pub mod p10_secure_logout;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_jwt_creation.rs"]
pub mod p01_jwt_creation;
#[cfg(feature = "solution")]
#[path = "solution/p02_jwt_verification.rs"]
pub mod p02_jwt_verification;
#[cfg(feature = "solution")]
#[path = "solution/p03_jwt_attacks.rs"]
pub mod p03_jwt_attacks;
#[cfg(feature = "solution")]
#[path = "solution/p04_session_tokens.rs"]
pub mod p04_session_tokens;
#[cfg(feature = "solution")]
#[path = "solution/p05_oauth2_flow.rs"]
pub mod p05_oauth2_flow;
#[cfg(feature = "solution")]
#[path = "solution/p06_totp_mfa.rs"]
pub mod p06_totp_mfa;
#[cfg(feature = "solution")]
#[path = "solution/p07_passkey_concept.rs"]
pub mod p07_passkey_concept;
#[cfg(feature = "solution")]
#[path = "solution/p08_credential_stuffing.rs"]
pub mod p08_credential_stuffing;
#[cfg(feature = "solution")]
#[path = "solution/p09_token_refresh.rs"]
pub mod p09_token_refresh;
#[cfg(feature = "solution")]
#[path = "solution/p10_secure_logout.rs"]
pub mod p10_secure_logout;
