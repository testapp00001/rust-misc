//! # Module 12: Async Order Worker
//!
//! This module teaches asynchronous order processing using Redis Streams as a
//! message queue. After the hot path (Module 11) accepts a flash sale claim in
//! under 5ms, the order worker picks up the event and performs the heavier work:
//! creating database records, processing payments, and sending notifications.
//!
//! ## Module Structure
//!
//! - `p01_consumer` - Redis Streams consumer with consumer groups and crash recovery
//! - `p02_order_processor` - Idempotent order event processing
//! - `p03_payment_handler` - Mock payment processing with circuit breaker
//! - `p04_notification_handler` - Best-effort notification delivery
//! - `p05_dead_letter` - Dead letter queue for failed events

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_consumer;
#[cfg(not(feature = "solution"))]
pub mod p02_order_processor;
#[cfg(not(feature = "solution"))]
pub mod p03_payment_handler;
#[cfg(not(feature = "solution"))]
pub mod p04_notification_handler;
#[cfg(not(feature = "solution"))]
pub mod p05_dead_letter;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_consumer.rs"]
pub mod p01_consumer;
#[cfg(feature = "solution")]
#[path = "solution/p02_order_processor.rs"]
pub mod p02_order_processor;
#[cfg(feature = "solution")]
#[path = "solution/p03_payment_handler.rs"]
pub mod p03_payment_handler;
#[cfg(feature = "solution")]
#[path = "solution/p04_notification_handler.rs"]
pub mod p04_notification_handler;
#[cfg(feature = "solution")]
#[path = "solution/p05_dead_letter.rs"]
pub mod p05_dead_letter;
