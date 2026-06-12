use production_message_queue::{
    config::MqConfig,
    health::HealthMonitor,
    logging,
    metrics::Metrics,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging (JSON to stdout).
    logging::init_logging();

    tracing::info!("Starting Production Message Queue");

    // Load configuration from file, falling back to defaults if not found.
    let config = MqConfig::load("config/default.toml").unwrap_or_else(|e| {
        tracing::warn!(error = %e, "No config found, using defaults");
        MqConfig::default_config()
    });

    // Validate configuration one more time (covers default config path).
    config.validate().map_err(|e| {
        tracing::error!(error = %e, "Invalid configuration");
        e
    })?;

    let metrics = Metrics::new();
    let health = HealthMonitor::shared();

    tracing::info!(
        node_id = config.broker.node_id,
        bind = %config.broker.bind_address,
        port = config.broker.port,
        data_dir = %config.broker.data_dir,
        default_partitions = config.broker.default_partitions,
        default_replication_factor = config.broker.default_replication_factor,
        "Broker configuration loaded"
    );

    tracing::info!(
        segment_max_bytes = config.storage.segment_max_bytes,
        wal_enabled = config.storage.wal_enabled,
        "Storage configured"
    );

    tracing::info!(
        peers = config.replication.peers.len(),
        election_timeout_ms = config.replication.election_timeout_ms,
        heartbeat_interval_ms = config.replication.heartbeat_interval_ms,
        "Replication configured"
    );

    tracing::info!(
        max_connections = config.network.max_connections,
        max_message_size = config.network.max_message_size,
        "Network configured"
    );

    tracing::info!(
        idempotency = config.delivery.enable_idempotency,
        max_in_flight = config.delivery.max_in_flight,
        ack_timeout_ms = config.delivery.ack_timeout_ms,
        "Delivery configured"
    );

    // Log cluster topology.
    let all_nodes = config.all_node_ids();
    tracing::info!(
        nodes = ?all_nodes,
        quorum_size = config.quorum_size(),
        "Cluster topology"
    );

    // In a full implementation, the broker would be created and started here:
    //
    // let mut broker = Broker::new(config);
    // broker.start().await?;
    //
    // For now, we demonstrate the subsystems are initialized correctly.
    tracing::info!(
        status = %health.check().status,
        "Health check at startup"
    );

    let snap = metrics.snapshot();
    tracing::info!(
        messages_produced = snap.messages_produced,
        messages_consumed = snap.messages_consumed,
        uptime_secs = snap.uptime_secs,
        "Metrics initialized"
    );

    tracing::info!("Production Message Queue ready");

    // Graceful shutdown on SIGTERM/SIGINT.
    tokio::signal::ctrl_c().await?;
    tracing::info!("Received shutdown signal, shutting down gracefully...");

    // In a full implementation:
    // broker.stop().await;

    tracing::info!("Shutdown complete");
    Ok(())
}
