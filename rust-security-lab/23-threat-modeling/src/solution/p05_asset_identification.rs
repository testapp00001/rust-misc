//! # Lesson 05: Asset Identification (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetCategory {
    Data,
    System,
    Credential,
    Infrastructure,
    IntellectualProperty,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CiaRating {
    pub confidentiality: u8,
    pub integrity: u8,
    pub availability: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub category: AssetCategory,
    pub description: String,
    pub cia: CiaRating,
    pub owner: String,
}

impl Asset {
    pub fn new(
        id: &str,
        name: &str,
        category: AssetCategory,
        description: &str,
        cia: CiaRating,
        owner: &str,
    ) -> Result<Self, String> {
        let vals = [cia.confidentiality, cia.integrity, cia.availability];
        let names = ["confidentiality", "integrity", "availability"];
        for (i, &v) in vals.iter().enumerate() {
            if v < 1 || v > 10 {
                return Err(format!("{} must be 1-10, got {}", names[i], v));
            }
        }
        Ok(Self {
            id: id.to_string(),
            name: name.to_string(),
            category,
            description: description.to_string(),
            cia,
            owner: owner.to_string(),
        })
    }

    pub fn value_score(&self) -> f64 {
        (self.cia.confidentiality as f64 + self.cia.integrity as f64 + self.cia.availability as f64)
            / 3.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInventory {
    pub name: String,
    pub assets: Vec<Asset>,
}

impl AssetInventory {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            assets: Vec::new(),
        }
    }

    pub fn add_asset(&mut self, asset: Asset) {
        self.assets.push(asset);
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.id == id)
    }

    pub fn by_category(&self, category: AssetCategory) -> Vec<&Asset> {
        self.assets.iter().filter(|a| a.category == category).collect()
    }

    pub fn ranked_by_value(&self) -> Vec<&Asset> {
        let mut ranked: Vec<&Asset> = self.assets.iter().collect();
        ranked.sort_by(|a, b| b.value_score().partial_cmp(&a.value_score()).unwrap());
        ranked
    }

    pub fn count(&self) -> usize {
        self.assets.len()
    }

    pub fn high_value_assets(&self, threshold: u8) -> Vec<&Asset> {
        self.assets
            .iter()
            .filter(|a| {
                a.cia.confidentiality >= threshold
                    || a.cia.integrity >= threshold
                    || a.cia.availability >= threshold
            })
            .collect()
    }
}

pub fn build_webapp_inventory() -> AssetInventory {
    let mut inv = AssetInventory::new("web-application");

    inv.add_asset(Asset::new(
        "A1", "User Database", AssetCategory::Data,
        "Primary PostgreSQL database with user PII, hashed passwords, and session data",
        CiaRating { confidentiality: 9, integrity: 9, availability: 8 }, "platform-team"
    ).unwrap());

    inv.add_asset(Asset::new(
        "A2", "TLS Private Keys", AssetCategory::Credential,
        "Server TLS certificates and private keys for HTTPS termination",
        CiaRating { confidentiality: 10, integrity: 10, availability: 7 }, "security-team"
    ).unwrap());

    inv.add_asset(Asset::new(
        "A3", "API Gateway", AssetCategory::System,
        "Kong/Nginx API gateway handling rate limiting, routing, and TLS termination",
        CiaRating { confidentiality: 5, integrity: 8, availability: 9 }, "platform-team"
    ).unwrap());

    inv.add_asset(Asset::new(
        "A4", "Source Code Repository", AssetCategory::IntellectualProperty,
        "Git repository containing application source code and business logic",
        CiaRating { confidentiality: 8, integrity: 9, availability: 6 }, "engineering-team"
    ).unwrap());

    inv.add_asset(Asset::new(
        "A5", "Cloud Infrastructure", AssetCategory::Infrastructure,
        "AWS/GCP account with IAM roles, VPC configuration, and DNS management",
        CiaRating { confidentiality: 9, integrity: 10, availability: 9 }, "ops-team"
    ).unwrap());

    inv.add_asset(Asset::new(
        "A6", "Redis Cache", AssetCategory::System,
        "In-memory cache for session data and frequently accessed queries",
        CiaRating { confidentiality: 6, integrity: 7, availability: 8 }, "platform-team"
    ).unwrap());

    inv.add_asset(Asset::new(
        "A7", "Audit Logs", AssetCategory::Data,
        "Immutable audit trail of all security-relevant events",
        CiaRating { confidentiality: 6, integrity: 10, availability: 7 }, "security-team"
    ).unwrap());

    inv
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_inventory() -> AssetInventory {
        let mut inv = AssetInventory::new("test-system");
        inv.add_asset(Asset::new(
            "A1", "User Database", AssetCategory::Data,
            "Primary user data store", CiaRating { confidentiality: 9, integrity: 9, availability: 8 }, "db-team"
        ).unwrap());
        inv.add_asset(Asset::new(
            "A2", "API Keys", AssetCategory::Credential,
            "Third-party API keys", CiaRating { confidentiality: 10, integrity: 8, availability: 5 }, "security-team"
        ).unwrap());
        inv.add_asset(Asset::new(
            "A3", "Static Files", AssetCategory::Data,
            "Public static assets", CiaRating { confidentiality: 1, integrity: 5, availability: 3 }, "dev-team"
        ).unwrap());
        inv
    }

    #[test]
    fn test_asset_creation_valid() {
        let asset = Asset::new("A1", "Test", AssetCategory::Data, "desc",
            CiaRating { confidentiality: 5, integrity: 5, availability: 5 }, "owner");
        assert!(asset.is_ok());
    }

    #[test]
    fn test_asset_creation_invalid() {
        let asset = Asset::new("A1", "Test", AssetCategory::Data, "desc",
            CiaRating { confidentiality: 0, integrity: 5, availability: 5 }, "owner");
        assert!(asset.is_err());
    }

    #[test]
    fn test_asset_value_score() {
        let asset = Asset::new("A1", "Test", AssetCategory::Data, "desc",
            CiaRating { confidentiality: 6, integrity: 9, availability: 5 }, "owner").unwrap();
        assert!((asset.value_score() - (6.0 + 9.0 + 5.0) / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_inventory_add_and_count() {
        let inv = make_test_inventory();
        assert_eq!(inv.count(), 3);
    }

    #[test]
    fn test_find_by_id() {
        let inv = make_test_inventory();
        assert!(inv.find_by_id("A1").is_some());
        assert!(inv.find_by_id("NONEXISTENT").is_none());
    }

    #[test]
    fn test_by_category() {
        let inv = make_test_inventory();
        let data_assets = inv.by_category(AssetCategory::Data);
        assert_eq!(data_assets.len(), 2);
        let cred_assets = inv.by_category(AssetCategory::Credential);
        assert_eq!(cred_assets.len(), 1);
    }

    #[test]
    fn test_ranked_by_value() {
        let inv = make_test_inventory();
        let ranked = inv.ranked_by_value();
        // A1: (9+9+8)/3 = 8.67, A2: (10+8+5)/3 = 7.67, A3: (1+5+3)/3 = 3.0
        assert_eq!(ranked[0].id, "A1");
        assert_eq!(ranked[1].id, "A2");
        assert_eq!(ranked[2].id, "A3");
    }

    #[test]
    fn test_high_value_assets() {
        let inv = make_test_inventory();
        let high = inv.high_value_assets(9);
        // A1 has confidentiality=9, integrity=9; A2 has confidentiality=10
        assert_eq!(high.len(), 2);
    }

    #[test]
    fn test_build_webapp_inventory() {
        let inv = build_webapp_inventory();
        assert!(inv.count() >= 5, "Should have at least 5 assets");
        assert!(inv.by_category(AssetCategory::Data).len() >= 1);
        assert!(inv.ranked_by_value().len() >= 5);
    }
}
