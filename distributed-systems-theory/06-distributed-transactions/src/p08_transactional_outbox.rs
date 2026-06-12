//! # Exercise: Transactional Outbox Pattern
//!
//! ## Theory
//!
//! The Transactional Outbox pattern solves the dual-write problem: how to reliably
//! write to a database AND publish events to a message broker atomically.
//!
//! Without the outbox pattern, you face:
//! 1. Write to DB, then publish event. If publish fails, the DB has data but no
//!    event is published.
//! 2. Publish event, then write to DB. If DB write fails, the event is published
//!    but the data is not stored.
//!
//! The outbox pattern:
//! 1. Write business data AND an event entry to an "outbox" table in the SAME
//!    database transaction.
//! 2. A separate process (the "relay" or "poller") reads the outbox table.
//! 3. It publishes events to the message broker.
//! 4. After successful publishing, it marks the event as sent.
//!
//! ## Proof / Intuition
//!
//! The key insight is that writing to two tables in the same database is atomic
//! (single database transaction). The outbox acts as a staging area:
//!
//! - The business logic writes events to the outbox.
//! - The relay process asynchronously publishes them.
//! - If the relay crashes, it resumes from where it left off.
//! - Events are guaranteed to be published eventually (at-least-once).
//!
//! The consumer must be idempotent to handle duplicate deliveries (since the relay
//! may publish the same event multiple times if it crashes after publishing but
//! before marking as sent).
//!
//! ## Implementation Task
//!
//! Implement the transactional outbox pattern:
//!
//! - `OutboxEntry`: an event in the outbox table.
//! - `OutboxWriter`: writes business data + outbox entry atomically.
//! - `OutboxReader`: polls outbox and publishes events.
//! - `IdempotentConsumer`: deduplicates events using event IDs.
//!
//! ## Verification
//!
//! - Verify atomicity: DB write and outbox entry are in the same transaction.
//! - Verify event is eventually published after being written.
//! - Verify idempotent consumption deduplicates events.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// An entry in the outbox table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEntry {
    /// Unique event ID for deduplication.
    pub event_id: u64,
    /// The event type/topic.
    pub event_type: String,
    /// Serialized event payload.
    pub payload: String,
    /// Whether the event has been published.
    pub published: bool,
    /// Timestamp when the entry was created.
    pub created_at: u64,
}

/// Simulated database with business data.
#[derive(Debug, Clone)]
pub struct Database {
    /// Table name -> row data.
    pub tables: HashMap<String, Vec<HashMap<String, String>>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Insert a row into a table.
    pub fn insert(&mut self, table: &str, row: HashMap<String, String>) {
        self.tables
            .entry(table.to_string())
            .or_insert_with(Vec::new)
            .push(row);
    }

    /// Count rows in a table.
    pub fn count(&self, table: &str) -> usize {
        self.tables.get(table).map_or(0, |rows| rows.len())
    }
}

/// The outbox table (separate from business data but written in same transaction).
#[derive(Debug, Clone)]
pub struct OutboxTable {
    pub entries: Vec<OutboxEntry>,
    pub next_event_id: u64,
}

impl OutboxTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_event_id: 1,
        }
    }

    pub fn add_entry(&mut self, event_type: &str, payload: &str) -> u64 {
        let id = self.next_event_id;
        self.next_event_id += 1;
        self.entries.push(OutboxEntry {
            event_id: id,
            event_type: event_type.to_string(),
            payload: payload.to_string(),
            published: false,
            created_at: 0,
        });
        id
    }

    pub fn mark_published(&mut self, event_id: u64) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.event_id == event_id) {
            entry.published = true;
        }
    }

    pub fn unpublished_entries(&self) -> Vec<&OutboxEntry> {
        self.entries.iter().filter(|e| !e.published).collect()
    }
}

/// Writer that atomically writes business data + outbox entry.
pub struct OutboxWriter {
    pub db: Database,
    pub outbox: OutboxTable,
    /// Transaction log for verification.
    pub transaction_log: Vec<String>,
}

impl OutboxWriter {
    pub fn new() -> Self {
        Self {
            db: Database::new(),
            outbox: OutboxTable::new(),
            transaction_log: Vec::new(),
        }
    }

    /// Atomically write business data and outbox entry.
    /// This simulates a single database transaction.
    pub fn write_with_event(
        &mut self,
        table: &str,
        row: HashMap<String, String>,
        event_type: &str,
        payload: &str,
    ) -> u64 {
        // In a real system, this would be a single DB transaction.
        // BEGIN TRANSACTION;
        //   INSERT INTO business_table ...;
        //   INSERT INTO outbox_table ...;
        // COMMIT;

        let event_id = self.outbox.add_entry(event_type, payload);
        self.db.insert(table, row);

        self.transaction_log
            .push(format!("TRANSACTION: write to {} + outbox event {}", table, event_id));

        event_id
    }
}

/// Reader that polls the outbox and publishes events.
pub struct OutboxReader {
    /// Events that have been "published" (simulated).
    pub published_events: Vec<OutboxEntry>,
}

impl OutboxReader {
    pub fn new() -> Self {
        Self {
            published_events: Vec::new(),
        }
    }

    /// Poll the outbox for unpublished events and "publish" them.
    pub fn poll_and_publish(&mut self, outbox: &mut OutboxTable) -> Vec<u64> {
        let mut published_ids = Vec::new();

        for entry in outbox.unpublished_entries() {
            // Simulate publishing to a message broker.
            self.published_events.push(entry.clone());
            published_ids.push(entry.event_id);
        }

        // Mark as published.
        for id in &published_ids {
            outbox.mark_published(*id);
        }

        published_ids
    }
}

/// Consumer that deduplicates events using event IDs.
pub struct IdempotentConsumer {
    /// Set of event IDs already processed.
    pub processed_ids: HashSet<u64>,
    /// Successfully consumed events.
    pub consumed_events: Vec<OutboxEntry>,
    /// Duplicate events that were rejected.
    pub rejected_duplicates: u32,
}

impl IdempotentConsumer {
    pub fn new() -> Self {
        Self {
            processed_ids: HashSet::new(),
            consumed_events: Vec::new(),
            rejected_duplicates: 0,
        }
    }

    /// Consume an event. Returns `true` if this is a new event, `false` if duplicate.
    pub fn consume(&mut self, event: &OutboxEntry) -> bool {
        if self.processed_ids.contains(&event.event_id) {
            self.rejected_duplicates += 1;
            return false;
        }

        self.processed_ids.insert(event.event_id);
        self.consumed_events.push(event.clone());
        true
    }

    /// Check if an event has been processed.
    pub fn is_processed(&self, event_id: u64) -> bool {
        self.processed_ids.contains(&event_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomicity_db_and_outbox_written_together() {
        let mut writer = OutboxWriter::new();

        let mut row = HashMap::new();
        row.insert("id".to_string(), "1".to_string());
        row.insert("data".to_string(), "test".to_string());

        let event_id = writer.write_with_event("orders", row, "OrderCreated", "order-1");

        // Both the business table and outbox should have entries.
        assert_eq!(writer.db.count("orders"), 1);
        assert_eq!(writer.outbox.entries.len(), 1);
        assert_eq!(writer.outbox.entries[0].event_id, event_id);
        assert!(!writer.outbox.entries[0].published);
    }

    #[test]
    fn event_eventually_published() {
        let mut writer = OutboxWriter::new();
        let mut reader = OutboxReader::new();

        let mut row = HashMap::new();
        row.insert("id".to_string(), "1".to_string());
        writer.write_with_event("orders", row, "OrderCreated", "order-1");

        // Poll and publish.
        let published = reader.poll_and_publish(&mut writer.outbox);

        assert_eq!(published.len(), 1);
        assert_eq!(reader.published_events.len(), 1);
        // Outbox entry should be marked as published.
        assert!(writer.outbox.entries[0].published);
    }

    #[test]
    fn idempotent_consumer_deduplicates() {
        let mut consumer = IdempotentConsumer::new();

        let event = OutboxEntry {
            event_id: 1,
            event_type: "Test".to_string(),
            payload: "data".to_string(),
            published: true,
            created_at: 0,
        };

        // First time: accepted.
        assert!(consumer.consume(&event), "First consume should return true");

        // Second time: rejected as duplicate.
        assert!(!consumer.consume(&event), "Second consume should return false (duplicate)");
        assert_eq!(consumer.rejected_duplicates, 1);
        assert_eq!(consumer.consumed_events.len(), 1);
    }

    #[test]
    fn outbox_poll_only_publishes_unpublished() {
        let mut writer = OutboxWriter::new();
        let mut reader = OutboxReader::new();

        // Write two events.
        let mut row1 = HashMap::new();
        row1.insert("id".to_string(), "1".to_string());
        writer.write_with_event("orders", row1, "OrderCreated", "order-1");

        let mut row2 = HashMap::new();
        row2.insert("id".to_string(), "2".to_string());
        writer.write_with_event("orders", row2, "OrderCreated", "order-2");

        // First poll: publishes both.
        let published1 = reader.poll_and_publish(&mut writer.outbox);
        assert_eq!(published1.len(), 2);

        // Second poll: publishes none (already published).
        let published2 = reader.poll_and_publish(&mut writer.outbox);
        assert_eq!(published2.len(), 0);
    }

    #[test]
    fn full_pipeline_end_to_end() {
        let mut writer = OutboxWriter::new();
        let mut reader = OutboxReader::new();
        let mut consumer = IdempotentConsumer::new();

        // Write multiple events.
        for i in 1..=5 {
            let mut row = HashMap::new();
            row.insert("id".to_string(), i.to_string());
            writer.write_with_event("orders", row, "OrderCreated", &format!("order-{}", i));
        }

        // Poll and consume.
        let published = reader.poll_and_publish(&mut writer.outbox);
        for event_id in &published {
            if let Some(entry) = writer.outbox.entries.iter().find(|e| &e.event_id == event_id) {
                consumer.consume(entry);
            }
        }

        assert_eq!(consumer.consumed_events.len(), 5);
        assert_eq!(consumer.rejected_duplicates, 0);

        // Re-publish (simulating relay crash and retry): consumer deduplicates.
        let republished = reader.poll_and_publish(&mut writer.outbox);
        // Should be empty since all are already published.
        assert_eq!(republished.len(), 0);
    }
}
