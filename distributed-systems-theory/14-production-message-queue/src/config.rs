use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{MqError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqConfig {
    pub broker: BrokerConfig,
    pub storage: StorageConfig,
    pub replication: ReplicationConfig,
    pub network: NetworkConfig,
    pub delivery: DeliveryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    pub node_id: u64,
    pub bind_address: String,
    pub port: u16,
    pub data_dir: String,
    pub default_partitions: u32,
    pub default_replication_factor: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub segment_max_bytes: u64,
    pub index_max_entries: u64,
    pub flush_interval_ms: u64,
    pub wal_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    pub election_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub snapshot_threshold: u64,
    pub peers: Vec<PeerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    pub node_id: u64,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_connections: usize,
    pub connection_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub max_message_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryConfig {
    pub enable_idempotency: bool,
    pub max_in_flight: u32,
    pub ack_timeout_ms: u64,
}

impl Default for MqConfig {
    fn default() -> Self {
        Self {
            broker: BrokerConfig::default(),
            storage: StorageConfig::default(),
            replication: ReplicationConfig::default(),
            network: NetworkConfig::default(),
            delivery: DeliveryConfig::default(),
        }
    }
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            node_id: 1,
            bind_address: "0.0.0.0".to_string(),
            port: 9092,
            data_dir: "/var/lib/mq/data".to_string(),
            default_partitions: 6,
            default_replication_factor: 1,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            segment_max_bytes: 1024 * 1024 * 1024, // 1 GiB
            index_max_entries: 10 * 1000 * 1000,   // 10M entries
            flush_interval_ms: 1000,
            wal_enabled: true,
        }
    }
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            election_timeout_ms: 1500,
            heartbeat_interval_ms: 300,
            snapshot_threshold: 10_000,
            peers: Vec::new(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_connections: 1024,
            connection_timeout_ms: 10_000,
            request_timeout_ms: 30_000,
            max_message_size: 1024 * 1024, // 1 MiB
        }
    }
}

impl Default for DeliveryConfig {
    fn default() -> Self {
        Self {
            enable_idempotency: true,
            max_in_flight: 5,
            ack_timeout_ms: 5000,
        }
    }
}

impl MqConfig {
    /// Load configuration from a TOML file, applying environment variable
    /// overrides and validating the result.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            MqError::Config(format!(
                "Failed to read config from '{}': {}",
                path.as_ref().display(),
                e
            ))
        })?;
        let mut config: MqConfig = toml::from_str(&content)
            .map_err(|e| MqError::Config(format!("Failed to parse config TOML: {}", e)))?;
        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    /// Produce a sensible default configuration for local development.
    pub fn default_config() -> Self {
        Self::default()
    }

    /// Serialize the current configuration to a TOML string.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| MqError::Config(format!("Failed to serialize config: {}", e)))
    }

    /// Apply environment variable overrides. Recognized variables:
    ///
    /// - `MQ_NODE_ID`          -> `broker.node_id`
    /// - `MQ_BIND_ADDRESS`     -> `broker.bind_address`
    /// - `MQ_PORT`             -> `broker.port`
    /// - `MQ_DATA_DIR`         -> `broker.data_dir`
    /// - `MQ_SEGMENTS_MAX`     -> `storage.segment_max_bytes`
    /// - `MQ_WAL_ENABLED`      -> `storage.wal_enabled`  (true/false)
    /// - `MQ_ELECTION_TIMEOUT` -> `replication.election_timeout_ms`
    /// - `MQ_HEARTBEAT_INTERVAL` -> `replication.heartbeat_interval_ms`
    /// - `MQ_MAX_CONNECTIONS`  -> `network.max_connections`
    /// - `MQ_REQUEST_TIMEOUT`  -> `network.request_timeout_ms`
    /// - `MQ_MAX_MESSAGE_SIZE` -> `network.max_message_size`
    /// - `MQ_IDEMPOTENCY`      -> `delivery.enable_idempotency` (true/false)
    /// - `MQ_MAX_IN_FLIGHT`    -> `delivery.max_in_flight`
    /// - `MQ_ACK_TIMEOUT`      -> `delivery.ack_timeout_ms`
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("MQ_NODE_ID") {
            if let Ok(id) = val.parse() {
                self.broker.node_id = id;
            }
        }
        if let Ok(val) = std::env::var("MQ_BIND_ADDRESS") {
            self.broker.bind_address = val;
        }
        if let Ok(val) = std::env::var("MQ_PORT") {
            if let Ok(port) = val.parse() {
                self.broker.port = port;
            }
        }
        if let Ok(val) = std::env::var("MQ_DATA_DIR") {
            self.broker.data_dir = val;
        }
        if let Ok(val) = std::env::var("MQ_SEGMENTS_MAX") {
            if let Ok(v) = val.parse() {
                self.storage.segment_max_bytes = v;
            }
        }
        if let Ok(val) = std::env::var("MQ_WAL_ENABLED") {
            if let Ok(v) = val.parse() {
                self.storage.wal_enabled = v;
            }
        }
        if let Ok(val) = std::env::var("MQ_ELECTION_TIMEOUT") {
            if let Ok(v) = val.parse() {
                self.replication.election_timeout_ms = v;
            }
        }
        if let Ok(val) = std::env::var("MQ_HEARTBEAT_INTERVAL") {
            if let Ok(v) = val.parse() {
                self.replication.heartbeat_interval_ms = v;
            }
        }
        if let Ok(val) = std::env::var("MQ_MAX_CONNECTIONS") {
            if let Ok(v) = val.parse() {
                self.network.max_connections = v;
            }
        }
        if let Ok(val) = std::env::var("MQ_REQUEST_TIMEOUT") {
            if let Ok(v) = val.parse() {
                self.network.request_timeout_ms = v;
            }
        }
        if let Ok(val) = std::env::var("MQ_MAX_MESSAGE_SIZE") {
            if let Ok(v) = val.parse() {
                self.network.max_message_size = v;
            }
        }
        if let Ok(val) = std::env::var("MQ_IDEMPOTENCY") {
            if let Ok(v) = val.parse() {
                self.delivery.enable_idempotency = v;
            }
        }
        if let Ok(val) = std::env::var("MQ_MAX_IN_FLIGHT") {
            if let Ok(v) = val.parse() {
                self.delivery.max_in_flight = v;
            }
        }
        if let Ok(val) = std::env::var("MQ_ACK_TIMEOUT") {
            if let Ok(v) = val.parse() {
                self.delivery.ack_timeout_ms = v;
            }
        }
    }

    /// Validate configuration invariants that, if violated, would cause
    /// runtime failures.
    pub fn validate(&self) -> Result<()> {
        if self.broker.port == 0 {
            return Err(MqError::Config(
                "broker.port must not be 0".to_string(),
            ));
        }

        if self.broker.default_partitions == 0 {
            return Err(MqError::Config(
                "broker.default_partitions must be > 0".to_string(),
            ));
        }

        if self.broker.default_replication_factor == 0 {
            return Err(MqError::Config(
                "broker.default_replication_factor must be > 0".to_string(),
            ));
        }

        if self.storage.segment_max_bytes == 0 {
            return Err(MqError::Config(
                "storage.segment_max_bytes must be > 0".to_string(),
            ));
        }

        if self.storage.index_max_entries == 0 {
            return Err(MqError::Config(
                "storage.index_max_entries must be > 0".to_string(),
            ));
        }

        if self.replication.election_timeout_ms <= self.replication.heartbeat_interval_ms {
            return Err(MqError::Config(
                "replication.election_timeout_ms must be greater than \
                 replication.heartbeat_interval_ms"
                    .to_string(),
            ));
        }

        if self.network.max_connections == 0 {
            return Err(MqError::Config(
                "network.max_connections must be > 0".to_string(),
            ));
        }

        if self.network.max_message_size == 0 {
            return Err(MqError::Config(
                "network.max_message_size must be > 0".to_string(),
            ));
        }

        if self.delivery.max_in_flight == 0 {
            return Err(MqError::Config(
                "delivery.max_in_flight must be > 0".to_string(),
            ));
        }

        if self.network.connection_timeout_ms == 0 {
            return Err(MqError::Config(
                "network.connection_timeout_ms must be > 0".to_string(),
            ));
        }

        if self.network.request_timeout_ms == 0 {
            return Err(MqError::Config(
                "network.request_timeout_ms must be > 0".to_string(),
            ));
        }

        if self.delivery.ack_timeout_ms == 0 {
            return Err(MqError::Config(
                "delivery.ack_timeout_ms must be > 0".to_string(),
            ));
        }

        // Warn if replication factor exceeds peer count + self
        let total_nodes = self.replication.peers.len() as u32 + 1;
        if self.broker.default_replication_factor > total_nodes {
            return Err(MqError::Config(format!(
                "broker.default_replication_factor ({}) exceeds available nodes ({})",
                self.broker.default_replication_factor, total_nodes,
            )));
        }

        Ok(())
    }

    /// Return the full list of node IDs including self and all configured peers.
    pub fn all_node_ids(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = std::iter::once(self.broker.node_id)
            .chain(self.replication.peers.iter().map(|p| p.node_id))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Compute a quorum size: strictly more than half the cluster.
    pub fn quorum_size(&self) -> usize {
        let total = self.all_node_ids().len();
        (total / 2) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = MqConfig::default_config();
        config.validate().expect("default config should be valid");
    }

    #[test]
    fn reject_zero_port() {
        let mut config = MqConfig::default_config();
        config.broker.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn reject_zero_partitions() {
        let mut config = MqConfig::default_config();
        config.broker.default_partitions = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn reject_zero_replication_factor() {
        let mut config = MqConfig::default_config();
        config.broker.default_replication_factor = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn reject_zero_segment_max() {
        let mut config = MqConfig::default_config();
        config.storage.segment_max_bytes = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn reject_election_timeout_not_greater_than_heartbeat() {
        let mut config = MqConfig::default_config();
        config.replication.election_timeout_ms = 300;
        config.replication.heartbeat_interval_ms = 300;
        assert!(config.validate().is_err());

        config.replication.election_timeout_ms = 200;
        assert!(config.validate().is_err());
    }

    #[test]
    fn reject_replication_factor_exceeding_nodes() {
        let mut config = MqConfig::default_config();
        config.replication.peers = vec![
            PeerConfig { node_id: 2, address: "127.0.0.1:9093".into() },
            PeerConfig { node_id: 3, address: "127.0.0.1:9094".into() },
        ];
        // 3 nodes total, factor of 4 is impossible
        config.broker.default_replication_factor = 4;
        assert!(config.validate().is_err());
    }

    #[test]
    fn accept_replication_factor_equal_to_nodes() {
        let mut config = MqConfig::default_config();
        config.replication.peers = vec![
            PeerConfig { node_id: 2, address: "127.0.0.1:9093".into() },
            PeerConfig { node_id: 3, address: "127.0.0.1:9094".into() },
        ];
        config.broker.default_replication_factor = 3;
        config.validate().expect("factor == nodes should be fine");
    }

    #[test]
    fn all_node_ids_deduped_and_sorted() {
        let mut config = MqConfig::default_config();
        config.broker.node_id = 2;
        config.replication.peers = vec![
            PeerConfig { node_id: 1, address: "x".into() },
            PeerConfig { node_id: 3, address: "y".into() },
        ];
        assert_eq!(config.all_node_ids(), vec![1, 2, 3]);
    }

    #[test]
    fn quorum_size() {
        let mut config = MqConfig::default_config();
        config.replication.peers = vec![
            PeerConfig { node_id: 2, address: "x".into() },
            PeerConfig { node_id: 3, address: "y".into() },
        ];
        // 3 nodes -> quorum = 2
        assert_eq!(config.quorum_size(), 2);

        config.replication.peers.push(PeerConfig {
            node_id: 4,
            address: "z".into(),
        });
        // 4 nodes -> quorum = 3
        assert_eq!(config.quorum_size(), 3);

        config.replication.peers.push(PeerConfig {
            node_id: 5,
            address: "w".into(),
        });
        // 5 nodes -> quorum = 3
        assert_eq!(config.quorum_size(), 3);
    }

    #[test]
    fn toml_roundtrip() {
        let config = MqConfig::default_config();
        let toml_str = config.to_toml().unwrap();
        let parsed: MqConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.broker.node_id, config.broker.node_id);
        assert_eq!(parsed.broker.port, config.broker.port);
        assert_eq!(
            parsed.storage.segment_max_bytes,
            config.storage.segment_max_bytes
        );
    }

    #[test]
    fn load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        let config = MqConfig::default_config();
        let toml_str = config.to_toml().unwrap();
        std::fs::write(&path, toml_str).unwrap();

        let loaded = MqConfig::load(&path).unwrap();
        assert_eq!(loaded.broker.node_id, config.broker.node_id);
        assert_eq!(loaded.broker.port, config.broker.port);
    }

    #[test]
    fn load_missing_file_fails() {
        let result = MqConfig::load("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn load_invalid_toml_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "this is not valid toml {{{{").unwrap();
        let result = MqConfig::load(path);
        assert!(result.is_err());
    }
}
