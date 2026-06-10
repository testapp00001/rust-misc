//! # Module 29: Capstone — Secure Messenger
//!
//! Build a Signal-like secure messenger from scratch. Each lesson adds a new
//! component: key generation, key exchange, encryption, forward secrecy,
//! group messaging, padding, and backup. The final lesson integrates everything
//! into a working end-to-end encrypted protocol.
//!
//! ## Learning Path
//! 1. Start with `p01_key_generation` — understand identity vs ephemeral keys
//! 2. Progress through X3DH, encryption, and forward secrecy
//! 3. Build up to group messaging, padding, and backup
//! 4. Complete the full protocol in `p10_full_protocol`
//!
//! ## Quick Test
//! ```bash
//! cargo test -p capstone_secure_messenger              # Test your implementation
//! cargo test -p capstone_secure_messenger --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_key_generation;
#[cfg(not(feature = "solution"))]
pub mod p02_key_exchange;
#[cfg(not(feature = "solution"))]
pub mod p03_message_encryption;
#[cfg(not(feature = "solution"))]
pub mod p04_message_decryption;
#[cfg(not(feature = "solution"))]
pub mod p05_forward_secrecy;
#[cfg(not(feature = "solution"))]
pub mod p06_key_verification;
#[cfg(not(feature = "solution"))]
pub mod p07_group_messaging;
#[cfg(not(feature = "solution"))]
pub mod p08_message_padding;
#[cfg(not(feature = "solution"))]
pub mod p09_key_backup;
#[cfg(not(feature = "solution"))]
pub mod p10_full_protocol;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_key_generation.rs"]
pub mod p01_key_generation;
#[cfg(feature = "solution")]
#[path = "solution/p02_key_exchange.rs"]
pub mod p02_key_exchange;
#[cfg(feature = "solution")]
#[path = "solution/p03_message_encryption.rs"]
pub mod p03_message_encryption;
#[cfg(feature = "solution")]
#[path = "solution/p04_message_decryption.rs"]
pub mod p04_message_decryption;
#[cfg(feature = "solution")]
#[path = "solution/p05_forward_secrecy.rs"]
pub mod p05_forward_secrecy;
#[cfg(feature = "solution")]
#[path = "solution/p06_key_verification.rs"]
pub mod p06_key_verification;
#[cfg(feature = "solution")]
#[path = "solution/p07_group_messaging.rs"]
pub mod p07_group_messaging;
#[cfg(feature = "solution")]
#[path = "solution/p08_message_padding.rs"]
pub mod p08_message_padding;
#[cfg(feature = "solution")]
#[path = "solution/p09_key_backup.rs"]
pub mod p09_key_backup;
#[cfg(feature = "solution")]
#[path = "solution/p10_full_protocol.rs"]
pub mod p10_full_protocol;
