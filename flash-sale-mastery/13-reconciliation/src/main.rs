//! # Reconciliation Job Entry Point
//!
//! Orchestrates the full reconciliation pipeline:
//! 1. Sync inventory between Redis and PostgreSQL
//! 2. Replay events to verify state consistency
//! 3. Scan for cross-system discrepancies
//! 4. Generate and display the reconciliation report
//!
//! This job is designed to run safely both during and after a sale.
//! During a sale, it only reports -- it does not apply corrections
//! to avoid interfering with live traffic.

use reconciliation::p01_inventory_sync::{InventorySync, MockDbStore, MockRedisStore};
use reconciliation::p02_event_replay::{EventReplay, MockCurrentState, MockEventStore};
use reconciliation::p03_discrepancy_detector::{DetectionDataSources, DiscrepancyDetector};
use reconciliation::p04_report_generator::ReportBuilder;
use reconciliation::shared::*;

fn main() {
    println!("=== Flash Sale Reconciliation Job ===\n");

    // ---------------------------------------------------------------
    // Phase 1: Inventory Synchronization
    // ---------------------------------------------------------------
    println!("Phase 1: Inventory Synchronization");
    println!("-----------------------------------");

    let mut redis = MockRedisStore::new();
    let mut db = MockDbStore::new();

    // Simulate: sale is over, Redis and DB mostly agree but a few drifted.
    redis.insert(RedisInventory {
        product_id: "LAPTOP-001".into(),
        available_stock: 45,
        reserved: 3,
        version: 150,
        last_updated: chrono::Utc::now(),
    });
    redis.insert(RedisInventory {
        product_id: "PHONE-002".into(),
        available_stock: 0,
        reserved: 0,
        version: 300,
        last_updated: chrono::Utc::now(),
    });
    redis.insert(RedisInventory {
        product_id: "TABLET-003".into(),
        available_stock: 12,
        reserved: 1,
        version: 80,
        last_updated: chrono::Utc::now(),
    });

    db.insert(DbInventory {
        product_id: "LAPTOP-001".into(),
        available_stock: 42,
        reserved: 3,
        total_sold: 55,
        version: 148,
        last_updated: chrono::Utc::now(),
    });
    db.insert(DbInventory {
        product_id: "PHONE-002".into(),
        available_stock: 0,
        reserved: 0,
        total_sold: 200,
        version: 300,
        last_updated: chrono::Utc::now(),
    });
    db.insert(DbInventory {
        product_id: "TABLET-003".into(),
        available_stock: 15,
        reserved: 1,
        total_sold: 73,
        version: 79,
        last_updated: chrono::Utc::now(),
    });

    let mut sync = InventorySync::new(redis, db);
    let sync_results = sync.sync_all().unwrap_or_default();

    for r in &sync_results {
        println!(
            "  {} => Redis: {}, DB: {}, Mismatch: {}, Action: {:?}",
            r.product_id, r.redis_stock, r.db_stock, r.mismatch, r.action_taken
        );
    }

    let corrections = sync.apply_corrections(&sync_results).unwrap_or(0);
    println!("  Corrections applied: {}\n", corrections);

    // ---------------------------------------------------------------
    // Phase 2: Event Replay Verification
    // ---------------------------------------------------------------
    println!("Phase 2: Event Replay Verification");
    println!("-----------------------------------");

    let mut event_store = MockEventStore::new();
    // Simulate a stream of events for LAPTOP-001.
    for i in 0..10 {
        event_store.push(InventoryEvent {
            event_id: uuid::Uuid::new_v4(),
            product_id: "LAPTOP-001".into(),
            event_type: EventType::Reserved,
            quantity: 1,
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(60 - i),
            metadata: None,
        });
    }
    event_store.push(InventoryEvent {
        event_id: uuid::Uuid::new_v4(),
        product_id: "LAPTOP-001".into(),
        event_type: EventType::StockAdded,
        quantity: 50,
        timestamp: chrono::Utc::now() - chrono::Duration::hours(2),
        metadata: None,
    });

    let mut current_state = MockCurrentState::new();
    // Intentionally set a slightly wrong value to test drift detection.
    current_state.set("LAPTOP-001", 89, 10);

    let replay = EventReplay::new(event_store, current_state, 50);
    match replay.replay_product_events("LAPTOP-001") {
        Ok(state) => {
            println!(
                "  LAPTOP-001 => Available: {}, Reserved: {}, Events: {}, Matches: {}",
                state.computed_available,
                state.computed_reserved,
                state.events_processed,
                state.matches_current
            );
            if !state.matches_current {
                println!("  WARNING: Drift detected between rebuilt and current state!");
            }
        }
        Err(e) => println!("  Error replaying events: {}", e),
    }
    println!();

    // ---------------------------------------------------------------
    // Phase 3: Discrepancy Detection
    // ---------------------------------------------------------------
    println!("Phase 3: Discrepancy Detection");
    println!("------------------------------");

    let mut data = DetectionDataSources::new();
    data.db_orders.push(reconciliation::p03_discrepancy_detector::OrderRecord {
        order_id: "ORD-100".into(),
        product_id: "LAPTOP-001".into(),
        user_id: "user-1".into(),
        voucher_code: None,
    });
    data.redis_orders.push(reconciliation::p03_discrepancy_detector::OrderRecord {
        order_id: "ORD-100".into(),
        product_id: "LAPTOP-001".into(),
        user_id: "user-1".into(),
        voucher_code: None,
    });
    data.redis_orders.push(reconciliation::p03_discrepancy_detector::OrderRecord {
        order_id: "ORD-999".into(),
        product_id: "PHONE-002".into(),
        user_id: "user-5".into(),
        voucher_code: None,
    });
    data.voucher_limits.insert("FLASH50".into(), 1);
    data.voucher_uses.push(("FLASH50".into(), "user-1".into()));
    data.voucher_uses.push(("FLASH50".into(), "user-2".into()));

    let detector = DiscrepancyDetector::new(data);
    let discrepancies = detector.scan_all().unwrap_or_default();

    for d in &discrepancies {
        println!("  [{:?}] {:?}: {}", d.severity, d.discrepancy_type, d.description);
    }
    if discrepancies.is_empty() {
        println!("  No discrepancies found.");
    }
    println!();

    // ---------------------------------------------------------------
    // Phase 4: Report Generation
    // ---------------------------------------------------------------
    println!("Phase 4: Report Generation");
    println!("--------------------------");

    let report_gen = ReportBuilder::new()
        .with_sync_results(sync_results)
        .with_discrepancies(discrepancies)
        .with_corrections(corrections)
        .with_products_checked(3)
        .build();

    // Human-readable output for console.
    println!("{}", report_gen.generate_human_readable());

    // JSON output (would be sent to dashboard API in production).
    match report_gen.generate_json() {
        Ok(json) => {
            println!("--- JSON Report ---");
            println!("{}", json);
        }
        Err(e) => println!("Failed to generate JSON report: {}", e),
    }
}
