//! # Module 10: Data in Transit
//!
//! TLS, certificate validation, replay defenses, and traffic analysis resistance.
//!
//! ## Learning Path
//! 1. Start with `p01_tls_basics` — understand TLS 1.3 handshake concepts
//! 2. Learn certificate validation and pinning
//! 3. Progress through mTLS, replay defense, and traffic analysis
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 10-data-in-transit              # Test your implementation
//! cargo test -p 10-data-in-transit --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_tls_basics;
#[cfg(not(feature = "solution"))]
pub mod p02_certificate_validation;
#[cfg(not(feature = "solution"))]
pub mod p03_certificate_pinning;
#[cfg(not(feature = "solution"))]
pub mod p04_mtls;
#[cfg(not(feature = "solution"))]
pub mod p05_replay_attack;
#[cfg(not(feature = "solution"))]
pub mod p06_message_framing;
#[cfg(not(feature = "solution"))]
pub mod p07_dns_security;
#[cfg(not(feature = "solution"))]
pub mod p08_secure_websocket;
#[cfg(not(feature = "solution"))]
pub mod p09_channel_binding;
#[cfg(not(feature = "solution"))]
pub mod p10_traffic_analysis;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_tls_basics.rs"]
pub mod p01_tls_basics;
#[cfg(feature = "solution")]
#[path = "solution/p02_certificate_validation.rs"]
pub mod p02_certificate_validation;
#[cfg(feature = "solution")]
#[path = "solution/p03_certificate_pinning.rs"]
pub mod p03_certificate_pinning;
#[cfg(feature = "solution")]
#[path = "solution/p04_mtls.rs"]
pub mod p04_mtls;
#[cfg(feature = "solution")]
#[path = "solution/p05_replay_attack.rs"]
pub mod p05_replay_attack;
#[cfg(feature = "solution")]
#[path = "solution/p06_message_framing.rs"]
pub mod p06_message_framing;
#[cfg(feature = "solution")]
#[path = "solution/p07_dns_security.rs"]
pub mod p07_dns_security;
#[cfg(feature = "solution")]
#[path = "solution/p08_secure_websocket.rs"]
pub mod p08_secure_websocket;
#[cfg(feature = "solution")]
#[path = "solution/p09_channel_binding.rs"]
pub mod p09_channel_binding;
#[cfg(feature = "solution")]
#[path = "solution/p10_traffic_analysis.rs"]
pub mod p10_traffic_analysis;
