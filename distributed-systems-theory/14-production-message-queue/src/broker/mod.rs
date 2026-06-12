//! Broker module -- orchestrates the message queue lifecycle.
//!
//! The broker ties together storage, replication, partitioning, networking,
//! and delivery into a coherent server that accepts produce and consume
//! requests.

pub mod consumer;
pub mod message;
pub mod partition;
pub mod producer;
pub mod topic;

use std::sync::Arc;

use tracing::{info, warn};

use crate::config::MqConfig;
use crate::delivery::DeliveryEngine;
use crate::health::HealthMonitor;
use crate::metrics::Metrics;
use crate::network::NetworkServer;
use crate::partitioning::PartitionManager;
use crate::replication::ReplicationManager;
use crate::storage::StorageEngine;

/// The top-level broker that owns all subsystems.
pub struct Broker {
    config: MqConfig,
    metrics: Arc<Metrics>,
    health: Arc<HealthMonitor>,
    storage: Arc<StorageEngine>,
    partition_manager: Arc<PartitionManager>,
    replication: Arc<ReplicationManager>,
    delivery: Arc<DeliveryEngine>,
    network: Option<NetworkServer>,
}

impl Broker {
    /// Create a new broker from the given configuration. Does not start
    /// networking; call `start()` to begin accepting connections.
    pub fn new(config: MqConfig) -> Self {
        let metrics = Metrics::new();
        let health = HealthMonitor::shared();
        let storage = StorageEngine::new(
            &config.broker.data_dir,
            config.storage.segment_max_bytes,
            config.storage.wal_enabled,
        );
        let partition_manager = PartitionManager::new(
            config.broker.default_partitions,
            config.broker.default_replication_factor,
        );
        let replication = ReplicationManager::new(
            config.broker.node_id,
            config.replication.election_timeout_ms,
            config.replication.heartbeat_interval_ms,
        );
        let delivery = DeliveryEngine::new(
            config.delivery.enable_idempotency,
            config.delivery.max_in_flight,
            config.delivery.ack_timeout_ms,
        );

        Broker {
            config,
            metrics,
            health,
            storage: Arc::new(storage),
            partition_manager: Arc::new(partition_manager),
            replication: Arc::new(replication),
            delivery: Arc::new(delivery),
            network: None,
        }
    }

    /// Start the broker: begin replication heartbeats and open the network
    /// listener.
    pub async fn start(&mut self) -> crate::error::Result<()> {
        info!(
            node_id = self.config.broker.node_id,
            port = self.config.broker.port,
            "Starting broker"
        );

        // Start replication.
        self.replication.start().await;

        // Create network server.
        let addr = format!(
            "{}:{}",
            self.config.broker.bind_address, self.config.broker.port
        );
        let network = NetworkServer::new(
            addr,
            self.config.network.max_connections,
            Arc::clone(&self.metrics),
            Arc::clone(&self.health),
        );
        self.network = Some(network);

        info!("Broker started and listening");
        Ok(())
    }

    /// Stop the broker gracefully: stop accepting new requests, drain
    /// in-flight, flush storage.
    pub async fn stop(&mut self) {
        info!("Stopping broker");

        if let Some(mut network) = self.network.take() {
            network.shutdown().await;
        }

        self.replication.stop().await;
        self.storage.flush();
        info!("Broker stopped");
    }

    /// Return a reference to the broker configuration.
    pub fn config(&self) -> &MqConfig {
        &self.config
    }

    /// Return a reference to the metrics collector.
    pub fn metrics(&self) -> &Arc<Metrics> {
        &self.metrics
    }

    /// Return a reference to the health monitor.
    pub fn health(&self) -> &Arc<HealthMonitor> {
        &self.health
    }

    /// Return a reference to the storage engine.
    pub fn storage(&self) -> &Arc<StorageEngine> {
        &self.storage
    }

    /// Return a reference to the partition manager.
    pub fn partition_manager(&self) -> &Arc<PartitionManager> {
        &self.partition_manager
    }

    /// Return a reference to the replication manager.
    pub fn replication(&self) -> &Arc<ReplicationManager> {
        &self.replication
    }

    /// Return a reference to the delivery engine.
    pub fn delivery(&self) -> &Arc<DeliveryEngine> {
        &self.delivery
    }

    /// Create a topic with the configured default partition count and
    /// replication factor.
    pub async fn create_topic(
        &self,
        name: &str,
    ) -> crate::error::Result<()> {
        self.partition_manager.create_topic(
            name,
            self.config.broker.default_partitions,
            self.config.broker.default_replication_factor,
        )?;
        info!(topic = name, "Topic created");
        Ok(())
    }

    /// Delete a topic and all its partitions.
    pub async fn delete_topic(
        &self,
        name: &str,
    ) -> crate::error::Result<()> {
        self.partition_manager.delete_topic(name)?;
        info!(topic = name, "Topic deleted");
        Ok(())
    }

    /// List all known topics.
    pub fn list_topics(&self) -> Vec<String> {
        self.partition_manager.list_topics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broker_creation() {
        let config = MqConfig::default_config();
        let broker = Broker::new(config);
        assert_eq!(broker.config().broker.node_id, 1);
    }

    #[test]
    fn create_and_list_topics() {
        let config = MqConfig::default_config();
        let broker = Broker::new(config);

        // Broker::new is sync, but create_topic is async
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            broker.create_topic("orders").await.unwrap();
            broker.create_topic("payments").await.unwrap();

            let mut topics = broker.list_topics();
            topics.sort();
            assert_eq!(topics, vec!["orders", "payments"]);
        });
    }

    #[test]
    fn delete_topic() {
        let config = MqConfig::default_config();
        let broker = Broker::new(config);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            broker.create_topic("temp").await.unwrap();
            assert_eq!(broker.list_topics().len(), 1);

            broker.delete_topic("temp").await.unwrap();
            assert!(broker.list_topics().is_empty());
        });
    }

    #[test]
    fn delete_nonexistent_topic_fails() {
        let config = MqConfig::default_config();
        let broker = Broker::new(config);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = broker.delete_topic("nonexistent").await;
            assert!(result.is_err());
        });
    }
}
