//! # Module 03: Asymmetric Encryption
//!
//! RSA, ECC, key exchange, hybrid encryption, and the attacks that break them.
//!
//! ## Learning Path
//! 1. Start with `p01_rsa_basics` — understand public-key encryption with RSA
//! 2. Progress through ECC, key exchange, and hybrid encryption
//! 3. End with forward secrecy — the gold standard for modern key management
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 03-asymmetric-encryption              # Test your implementation
//! cargo test -p 03-asymmetric-encryption --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_rsa_basics;
#[cfg(not(feature = "solution"))]
pub mod p02_rsa_key_sizes;
#[cfg(not(feature = "solution"))]
pub mod p03_x25519_key_exchange;
#[cfg(not(feature = "solution"))]
pub mod p04_ecdh_key_exchange;
#[cfg(not(feature = "solution"))]
pub mod p05_hybrid_encryption;
#[cfg(not(feature = "solution"))]
pub mod p06_key_encapsulation;
#[cfg(not(feature = "solution"))]
pub mod p07_chosen_ciphertext_attack;
#[cfg(not(feature = "solution"))]
pub mod p08_key_serialization;
#[cfg(not(feature = "solution"))]
pub mod p09_key_validation;
#[cfg(not(feature = "solution"))]
pub mod p10_forward_secrecy;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_rsa_basics.rs"]
pub mod p01_rsa_basics;
#[cfg(feature = "solution")]
#[path = "solution/p02_rsa_key_sizes.rs"]
pub mod p02_rsa_key_sizes;
#[cfg(feature = "solution")]
#[path = "solution/p03_x25519_key_exchange.rs"]
pub mod p03_x25519_key_exchange;
#[cfg(feature = "solution")]
#[path = "solution/p04_ecdh_key_exchange.rs"]
pub mod p04_ecdh_key_exchange;
#[cfg(feature = "solution")]
#[path = "solution/p05_hybrid_encryption.rs"]
pub mod p05_hybrid_encryption;
#[cfg(feature = "solution")]
#[path = "solution/p06_key_encapsulation.rs"]
pub mod p06_key_encapsulation;
#[cfg(feature = "solution")]
#[path = "solution/p07_chosen_ciphertext_attack.rs"]
pub mod p07_chosen_ciphertext_attack;
#[cfg(feature = "solution")]
#[path = "solution/p08_key_serialization.rs"]
pub mod p08_key_serialization;
#[cfg(feature = "solution")]
#[path = "solution/p09_key_validation.rs"]
pub mod p09_key_validation;
#[cfg(feature = "solution")]
#[path = "solution/p10_forward_secrecy.rs"]
pub mod p10_forward_secrecy;
