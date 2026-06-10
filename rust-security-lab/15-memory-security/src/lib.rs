//! # Module 15: Memory Security
//!
//! Rust's memory safety does NOT equal cryptographic safety.
//! Learn to protect secrets in memory: zeroize, secrecy, constant-time ops,
//! mlock, guard pages, core dump protection, and side-channel defense.
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 15-memory-security              # Test your implementation
//! cargo test -p 15-memory-security --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_zeroize_basics;
#[cfg(not(feature = "solution"))]
pub mod p02_secrecy_crate;
#[cfg(not(feature = "solution"))]
pub mod p03_constant_time_ops;
#[cfg(not(feature = "solution"))]
pub mod p04_mlock_memory;
#[cfg(not(feature = "solution"))]
pub mod p05_guard_pages;
#[cfg(not(feature = "solution"))]
pub mod p06_memory_forensics_defense;
#[cfg(not(feature = "solution"))]
pub mod p07_core_dump_protection;
#[cfg(not(feature = "solution"))]
pub mod p08_stack_vs_heap;
#[cfg(not(feature = "solution"))]
pub mod p09_secure_allocator;
#[cfg(not(feature = "solution"))]
pub mod p10_side_channel_defense;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_zeroize_basics.rs"]
pub mod p01_zeroize_basics;
#[cfg(feature = "solution")]
#[path = "solution/p02_secrecy_crate.rs"]
pub mod p02_secrecy_crate;
#[cfg(feature = "solution")]
#[path = "solution/p03_constant_time_ops.rs"]
pub mod p03_constant_time_ops;
#[cfg(feature = "solution")]
#[path = "solution/p04_mlock_memory.rs"]
pub mod p04_mlock_memory;
#[cfg(feature = "solution")]
#[path = "solution/p05_guard_pages.rs"]
pub mod p05_guard_pages;
#[cfg(feature = "solution")]
#[path = "solution/p06_memory_forensics_defense.rs"]
pub mod p06_memory_forensics_defense;
#[cfg(feature = "solution")]
#[path = "solution/p07_core_dump_protection.rs"]
pub mod p07_core_dump_protection;
#[cfg(feature = "solution")]
#[path = "solution/p08_stack_vs_heap.rs"]
pub mod p08_stack_vs_heap;
#[cfg(feature = "solution")]
#[path = "solution/p09_secure_allocator.rs"]
pub mod p09_secure_allocator;
#[cfg(feature = "solution")]
#[path = "solution/p10_side_channel_defense.rs"]
pub mod p10_side_channel_defense;
