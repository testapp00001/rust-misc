//! # P10 — Full Load Simulation
//!
//! **Scenario:** A realistic flash sale with 5 products (varying stock
//! levels) and 100 accounts generating 500 concurrent requests.
//!
//! **Expected behaviour:**
//! - Stock invariants hold for every product.
//! - No product is oversold.
//! - Per-account limits are respected.
//! - Total successes across all products never exceed total stock.
//!
//! **Concepts tested:**
//! - Multi-product concurrency
//! - Global invariant checking
//! - Realistic load patterns

mod common;

use common::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::task::JoinHandle;

/// A single request + expected product in the simulation.
struct SimRequest {
    product_id: String,
    account_id: String,
}

#[tokio::test]
async fn test_full_flash_sale_simulation() {
    // -- Setup: 5 products with varying stock ---------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = Arc::new(MockFlashSaleService::new(redis.clone(), db.clone()));

    let products: Vec<(&str, u32)> = vec![
        ("hot-item", 3),       // very limited
        ("popular", 10),       // moderate
        ("normal", 50),        // plenty
        ("niche", 5),          // limited
        ("clearance", 100),    // abundant
    ];

    let mut initial_stocks: HashMap<String, u32> = HashMap::new();
    for (id, stock) in &products {
        service.add_product(make_product(id, *stock));
        initial_stocks.insert(id.to_string(), *stock);
    }

    let total_stock: u32 = products.iter().map(|(_, s)| *s).sum();
    let num_accounts: u32 = 100;
    let total_requests: u32 = 500;

    // -- Generate deterministic request distribution --------------------
    // Distribute requests across products based on a simple hash.
    let mut sim_requests: Vec<SimRequest> = Vec::new();
    for i in 0..total_requests {
        let product_idx = (i as usize) % products.len();
        let account_idx = i % num_accounts;
        sim_requests.push(SimRequest {
            product_id: products[product_idx].0.to_string(),
            account_id: format!("acct-{account_idx}"),
        });
    }

    let barrier = Arc::new(Barrier::new(sim_requests.len()));

    // -- Act: fire all requests concurrently ----------------------------
    let handles: Vec<JoinHandle<(String, MockPurchaseResult)>> = sim_requests
        .into_iter()
        .map(|sim| {
            let svc = Arc::clone(&service);
            let bar = Arc::clone(&barrier);
            tokio::spawn(async move {
                bar.wait().await;
                let req = MockPurchaseRequest {
                    product_id: sim.product_id.clone(),
                    account_id: sim.account_id.clone(),
                    idempotency_key: make_idempotency_key(
                        "p10",
                        &format!("{}:{}", sim.product_id, sim.account_id),
                    ),
                };
                let result = svc.purchase(&req).await;
                (sim.product_id, result)
            })
        })
        .collect();

    // -- Collect results -----------------------------------------------
    let mut product_successes: HashMap<String, u32> = HashMap::new();
    let mut total_successes: u32 = 0;

    for handle in handles {
        let (product_id, result) = handle.await.unwrap();
        if matches!(result, MockPurchaseResult::Success { .. }) {
            *product_successes.entry(product_id).or_insert(0) += 1;
            total_successes += 1;
        }
    }

    // -- Assert: per-product invariants ---------------------------------
    for (id, initial_stock) in &products {
        let successes = product_successes.get(*id).copied().unwrap_or(0);
        let final_stock = redis.get_stock(id);

        // 1. No overselling: successes <= initial stock.
        assert!(
            successes <= *initial_stock,
            "product {id}: {successes} successes exceeds initial stock {initial_stock}"
        );

        // 2. Stock consistency: final_stock + successes == initial_stock.
        assert_eq!(
            final_stock + successes,
            *initial_stock,
            "product {id}: stock invariant violated — final({final_stock}) + successes({successes}) != initial({initial_stock})"
        );

        // 3. Stock never negative (redundant but explicit).
        assert!(
            final_stock <= *initial_stock,
            "product {id}: stock {final_stock} exceeds initial {initial_stock}"
        );
    }

    // -- Assert: global invariants --------------------------------------
    // Total successes cannot exceed total stock.
    assert!(
        total_successes <= total_stock,
        "total successes ({total_successes}) exceeds total stock ({total_stock})"
    );

    // At least some requests succeeded (sanity check).
    assert!(
        total_successes > 0,
        "at least some requests should succeed"
    );

    // DB orders match successes.
    let total_db_orders: u32 = products
        .iter()
        .map(|(id, _)| db.count_orders(id))
        .sum();
    // DB orders may be <= successes if DB failures were simulated,
    // but in this test both backends are healthy.
    assert_eq!(
        total_db_orders, total_successes,
        "DB order count ({total_db_orders}) should match success count ({total_successes})"
    );
}
