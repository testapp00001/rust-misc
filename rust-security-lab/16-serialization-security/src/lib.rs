//! # Module 16: Serialization Security
//!
//! Deserialization attacks, serde safety patterns, format-specific threats, and
//! safe default configurations. This module covers every major serialization
//! attack surface: JSON, XML, YAML, and Protobuf.
//!
//! ## Learning Path
//! 1. Start with `p01_deserialization_attacks` -- understand the threat model
//! 2. Learn serde's safety features: deny_unknown_fields (p02), depth limits (p03)
//! 3. Build schema validation (p04) and understand format confusion (p05)
//! 4. Study format-specific attacks: XML (p08), YAML (p09), Protobuf (p07)
//! 5. Internalize safe defaults (p10)
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 16-serialization-security              # Test your implementation
//! cargo test -p 16-serialization-security --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_deserialization_attacks;
#[cfg(not(feature = "solution"))]
pub mod p02_serde_security;
#[cfg(not(feature = "solution"))]
pub mod p03_json_depth_limit;
#[cfg(not(feature = "solution"))]
pub mod p04_schema_validation;
#[cfg(not(feature = "solution"))]
pub mod p05_format_confusion;
#[cfg(not(feature = "solution"))]
pub mod p06_untrusted_input;
#[cfg(not(feature = "solution"))]
pub mod p07_protobuf_security;
#[cfg(not(feature = "solution"))]
pub mod p08_xml_security;
#[cfg(not(feature = "solution"))]
pub mod p09_yaml_security;
#[cfg(not(feature = "solution"))]
pub mod p10_safe_defaults;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_deserialization_attacks.rs"]
pub mod p01_deserialization_attacks;
#[cfg(feature = "solution")]
#[path = "solution/p02_serde_security.rs"]
pub mod p02_serde_security;
#[cfg(feature = "solution")]
#[path = "solution/p03_json_depth_limit.rs"]
pub mod p03_json_depth_limit;
#[cfg(feature = "solution")]
#[path = "solution/p04_schema_validation.rs"]
pub mod p04_schema_validation;
#[cfg(feature = "solution")]
#[path = "solution/p05_format_confusion.rs"]
pub mod p05_format_confusion;
#[cfg(feature = "solution")]
#[path = "solution/p06_untrusted_input.rs"]
pub mod p06_untrusted_input;
#[cfg(feature = "solution")]
#[path = "solution/p07_protobuf_security.rs"]
pub mod p07_protobuf_security;
#[cfg(feature = "solution")]
#[path = "solution/p08_xml_security.rs"]
pub mod p08_xml_security;
#[cfg(feature = "solution")]
#[path = "solution/p09_yaml_security.rs"]
pub mod p09_yaml_security;
#[cfg(feature = "solution")]
#[path = "solution/p10_safe_defaults.rs"]
pub mod p10_safe_defaults;
