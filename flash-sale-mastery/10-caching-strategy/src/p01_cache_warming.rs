//! # Exercise 01: Cache Warming
//!
//! ## Learning Objective
//! Learn how to pre-warm a cache before a flash sale starts. Cache warming
//! eliminates cold-start cache misses that would otherwise cause a thundering
//! herd at sale launch time.
//!
//! ## Flash Sale Context
//! When a flash sale launches, thousands of users hit the same product pages
//! within seconds. If the cache is cold, every request misses and hits the
//! database -- a recipe for disaster. By warming the cache with product data,
//! stock counters, and sale configuration *before* the sale starts, you ensure
//! that the first wave of requests all hit warm cache entries.
//!
//! ## Instructions
//! 1. Implement `MockDatabase::new` to create a mock database with products and stock
//! 2. Implement `CacheWarmer::warm_product_data` to load product details into cache
//! 3. Implement `CacheWarmer::warm_stock_counters` to load stock counts into cache
//! 4. Implement `CacheWarmer::warm_sale_config` to load sale configuration into cache
//! 5. Implement `CacheWarmer::get_cached` to retrieve a value from the cache
//!
//! ## Hints
//! - Use `DashMap::insert` to populate the cache
//! - Serialize structs to `serde_json::Value` before caching
//! - The mock database simulates a slow DB with `tokio::time::sleep`

use std::collections::HashMap;
use std::time::Duration;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Error type for cache warming operations.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Product not found: {0}")]
    ProductNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Product data as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductData {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub description: String,
}

/// Sale configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaleConfig {
    pub sale_id: String,
    pub start_time: String,
    pub end_time: String,
    pub max_discount_percent: f64,
}

/// Mock database that simulates slow reads.
pub struct MockDatabase {
    pub products: HashMap<String, ProductData>,
    pub stock: HashMap<String, i64>,
}

impl MockDatabase {
    /// Create a new mock database with the given products and stock.
    pub fn new(
        products: HashMap<String, ProductData>,
        stock: HashMap<String, i64>,
    ) -> Self {
        // TODO: Store the products and stock maps
        todo!("Implement MockDatabase::new")
    }

    /// Simulate a slow database read for product data.
    pub async fn get_product(&self, product_id: &str) -> Result<ProductData, CacheError> {
        // TODO: Sleep for 10ms to simulate DB latency
        // TODO: Look up the product in self.products
        // TODO: Return CacheError::ProductNotFound if not found
        todo!("Implement MockDatabase::get_product")
    }

    /// Simulate a slow database read for stock count.
    pub async fn get_stock(&self, product_id: &str) -> Result<i64, CacheError> {
        // TODO: Sleep for 10ms to simulate DB latency
        // TODO: Look up the stock in self.stock, return 0 if not found
        todo!("Implement MockDatabase::get_stock")
    }
}

/// Cache warmer that pre-populates an in-memory cache.
pub struct CacheWarmer {
    pub cache: DashMap<String, serde_json::Value>,
    pub db: MockDatabase,
}

impl CacheWarmer {
    /// Create a new cache warmer with the given mock database.
    pub fn new(db: MockDatabase) -> Self {
        // TODO: Initialize with an empty DashMap cache
        todo!("Implement CacheWarmer::new")
    }

    /// Pre-warm the cache with product data for the given product IDs.
    ///
    /// # Arguments
    /// * `product_ids` - List of product IDs to warm
    pub async fn warm_product_data(
        &self,
        product_ids: &[String],
    ) -> Result<(), CacheError> {
        // TODO: For each product ID:
        // TODO:   1. Fetch product from the mock DB
        // TODO:   2. Serialize to serde_json::Value
        // TODO:   3. Insert into cache with key "product:{id}"
        todo!("Implement CacheWarmer::warm_product_data")
    }

    /// Pre-warm the cache with stock counters for the given product IDs.
    ///
    /// # Arguments
    /// * `product_ids` - List of product IDs to warm stock for
    pub async fn warm_stock_counters(
        &self,
        product_ids: &[String],
    ) -> Result<(), CacheError> {
        // TODO: For each product ID:
        // TODO:   1. Fetch stock from the mock DB
        // TODO:   2. Serialize to serde_json::Value
        // TODO:   3. Insert into cache with key "stock:{id}"
        todo!("Implement CacheWarmer::warm_stock_counters")
    }

    /// Pre-warm the cache with sale configuration.
    ///
    /// # Arguments
    /// * `config` - The sale configuration to cache
    pub async fn warm_sale_config(&self, config: &SaleConfig) -> Result<(), CacheError> {
        // TODO: Serialize config to serde_json::Value
        // TODO: Insert into cache with key "sale_config"
        todo!("Implement CacheWarmer::warm_sale_config")
    }

    /// Get a value from the cache.
    ///
    /// # Arguments
    /// * `key` - The cache key to look up
    pub fn get_cached(&self, key: &str) -> Option<serde_json::Value> {
        // TODO: Look up the key in the DashMap cache
        // TODO: Clone and return the value if found
        todo!("Implement CacheWarmer::get_cached")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_products() -> (HashMap<String, ProductData>, HashMap<String, i64>) {
        let mut products = HashMap::new();
        products.insert(
            "product:1001".to_string(),
            ProductData {
                id: "product:1001".to_string(),
                name: "Flash Sale Widget".to_string(),
                price: 99.99,
                description: "Limited edition widget".to_string(),
            },
        );
        products.insert(
            "product:1002".to_string(),
            ProductData {
                id: "product:1002".to_string(),
                name: "Mega Deal Gadget".to_string(),
                price: 149.99,
                description: "Premium gadget at a steal".to_string(),
            },
        );

        let mut stock = HashMap::new();
        stock.insert("product:1001".to_string(), 100);
        stock.insert("product:1002".to_string(), 50);

        (products, stock)
    }

    #[tokio::test]
    async fn test_cache_populated_after_warming() {
        let (products, stock) = make_test_products();
        let db = MockDatabase::new(products, stock);
        let warmer = CacheWarmer::new(db);

        let product_ids = vec![
            "product:1001".to_string(),
            "product:1002".to_string(),
        ];

        warmer
            .warm_product_data(&product_ids)
            .await
            .expect("Warming should succeed");
        warmer
            .warm_stock_counters(&product_ids)
            .await
            .expect("Stock warming should succeed");

        assert!(
            warmer.get_cached("product:product:1001").is_some(),
            "Product 1001 should be cached"
        );
        assert!(
            warmer.get_cached("product:product:1002").is_some(),
            "Product 1002 should be cached"
        );
        assert!(
            warmer.get_cached("stock:product:1001").is_some(),
            "Stock for 1001 should be cached"
        );
        assert!(
            warmer.get_cached("stock:product:1002").is_some(),
            "Stock for 1002 should be cached"
        );
    }

    #[tokio::test]
    async fn test_all_products_accessible() {
        let (products, stock) = make_test_products();
        let db = MockDatabase::new(products, stock);
        let warmer = CacheWarmer::new(db);

        let product_ids = vec![
            "product:1001".to_string(),
            "product:1002".to_string(),
        ];
        warmer
            .warm_product_data(&product_ids)
            .await
            .expect("Warming should succeed");

        let cached = warmer
            .get_cached("product:product:1001")
            .expect("Product 1001 should be in cache");
        let product: ProductData =
            serde_json::from_value(cached).expect("Should deserialize");
        assert_eq!(product.name, "Flash Sale Widget");
        assert_eq!(product.price, 99.99);
    }

    #[tokio::test]
    async fn test_sale_config_cached() {
        let (products, stock) = make_test_products();
        let db = MockDatabase::new(products, stock);
        let warmer = CacheWarmer::new(db);

        let config = SaleConfig {
            sale_id: "FLASH_SALE_001".to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            end_time: "2025-01-01T01:00:00Z".to_string(),
            max_discount_percent: 50.0,
        };
        warmer
            .warm_sale_config(&config)
            .await
            .expect("Config warming should succeed");

        let cached = warmer
            .get_cached("sale_config")
            .expect("Sale config should be cached");
        let loaded: SaleConfig =
            serde_json::from_value(cached).expect("Should deserialize");
        assert_eq!(loaded.sale_id, "FLASH_SALE_001");
    }
}
