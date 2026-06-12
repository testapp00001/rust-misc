//! # Solution 01: Cache Warming
//!
//! Complete implementation of cache warming for flash sale systems.

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
        Self { products, stock }
    }

    /// Simulate a slow database read for product data.
    pub async fn get_product(&self, product_id: &str) -> Result<ProductData, CacheError> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        self.products
            .get(product_id)
            .cloned()
            .ok_or_else(|| CacheError::ProductNotFound(product_id.to_string()))
    }

    /// Simulate a slow database read for stock count.
    pub async fn get_stock(&self, product_id: &str) -> Result<i64, CacheError> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(self.stock.get(product_id).copied().unwrap_or(0))
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
        Self {
            cache: DashMap::new(),
            db,
        }
    }

    /// Pre-warm the cache with product data for the given product IDs.
    pub async fn warm_product_data(
        &self,
        product_ids: &[String],
    ) -> Result<(), CacheError> {
        for id in product_ids {
            let product = self.db.get_product(id).await?;
            let value = serde_json::to_value(&product)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;
            self.cache.insert(format!("product:{id}"), value);
        }
        Ok(())
    }

    /// Pre-warm the cache with stock counters for the given product IDs.
    pub async fn warm_stock_counters(
        &self,
        product_ids: &[String],
    ) -> Result<(), CacheError> {
        for id in product_ids {
            let stock = self.db.get_stock(id).await?;
            let value = serde_json::to_value(stock)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;
            self.cache.insert(format!("stock:{id}"), value);
        }
        Ok(())
    }

    /// Pre-warm the cache with sale configuration.
    pub async fn warm_sale_config(&self, config: &SaleConfig) -> Result<(), CacheError> {
        let value = serde_json::to_value(config)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        self.cache.insert("sale_config".to_string(), value);
        Ok(())
    }

    /// Get a value from the cache.
    pub fn get_cached(&self, key: &str) -> Option<serde_json::Value> {
        self.cache.get(key).map(|v| v.clone())
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
