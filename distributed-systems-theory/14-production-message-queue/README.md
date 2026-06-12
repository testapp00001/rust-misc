# Module 14: Production-Grade Distributed Message Queue

> A mini-Kafka built from scratch, combining every concept from modules 01-13 into a real-world system.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                       Client (Producer/Consumer)                │
│                        TCP + Length-Prefixed Binary             │
└───────────────────────────────┬─────────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────────┐
│                     Network Layer                               │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐     │
│  │ TCP Server  │  │ Conn Pool    │  │ RPC Protocol        │     │
│  │ (tokio)     │  │ (reuse)      │  │ (bincode framing)   │     │
│  └─────────────┘  └──────────────┘  └─────────────────────┘     │
└───────────────────────────────┬─────────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────────┐
│                      Broker Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐    │
│  │ Topic Mgr    │  │ Producer     │  │ Consumer Groups     │    │
│  │ (partitions) │  │ (hash/rrobin)│  │ (rebalancing)       │    │
│  └──────────────┘  └──────────────┘  └─────────────────────┘    │
└───────────────────────────────┬─────────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────────┐
│                     Storage Engine                              │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐    │
│  │ Segments     │  │ WAL          │  │ Offset Index        │    │
│  │ (append-only)│  │ (durability) │  │ (BTreeMap)          │    │
│  └──────────────┘  └──────────────┘  └─────────────────────┘    │
└───────────────────────────────┬─────────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────────┐
│                    Replication Layer                            │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐     │
│  │ Raft Node   │  │ Leader Elect │  │ Log Replication     │     │
│  │ (consensus) │  │ (timeout)    │  │ (AppendEntries)     │     │
│  └─────────────┘  └──────────────┘  └─────────────────────┘     │
└───────────────────────────────┬─────────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────────┐
│                   Cross-Cutting Concerns                        │
│  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌──────────┐           │
│  │ HLC      │ │ CRDTs     │ │ Metrics  │ │ Health   │           │
│  │ (clock)  │ │ (counters)│ │ (atomic) │ │ (k8s)    │           │
│  └──────────┘ └───────────┘ └──────────┘ └──────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

## Distributed Systems Concepts Used

| Concept | Module | How It's Used |
|---------|--------|---------------|
| **Two Generals** | 01 | TCP connection reliability, retry + idempotency for produce/consume |
| **Time & Ordering** | 02 | HLC timestamps on messages for causal ordering |
| **CAP Theorem** | 03 | Configurable consistency: `Acks::None` (AP) vs `Acks::Majority` (CP) |
| **Raft Consensus** | 04 | Leader election, log replication for partition replication |
| **FLP Impossibility** | 05 | Election timeouts with randomized jitter as failure detector |
| **Distributed Transactions** | 06 | Exactly-once delivery via transaction coordinator |
| **Consistent Hashing** | 07 | Partition assignment for topic messages |
| **CRDTs** | 08 | Distributed counters for metrics aggregation |
| **Byzantine Fault Tolerance** | 09 | Message checksums (CRC32) for corruption detection |
| **Clock Synchronization** | 10 | HLC for wall-clock-close causal timestamps |
| **Integration** | 11 | All components wired together |
| **Jepsen-Style Testing** | 12 | Integration tests verify consistency properties |
| **Production Readiness** | — | Config, logging, metrics, health checks, Docker, graceful shutdown |

## Module Structure

```
14-production-message-queue/
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── config/
│   └── default.toml
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # Binary entry point
│   ├── config.rs           # Configuration (TOML + env vars)
│   ├── error.rs            # Error types (thiserror)
│   ├── logging.rs          # Structured JSON logging
│   ├── metrics.rs          # Lock-free metrics (atomics + DashMap)
│   ├── health.rs           # Health checks (k8s probes)
│   ├── network/
│   │   ├── rpc.rs          # RPC protocol (11 request types)
│   │   ├── connection_pool.rs  # TCP connection pooling
│   │   └── server.rs       # TCP server with framing
│   ├── broker/
│   │   ├── message.rs      # Message types
│   │   ├── topic.rs        # Topic management
│   │   ├── partition.rs    # Partition storage
│   │   ├── producer.rs     # Producer API (hash/round-robin)
│   │   └── consumer.rs     # Consumer + consumer groups
│   ├── replication/
│   │   ├── raft_node.rs    # Full Raft state machine
│   │   ├── leader_election.rs  # Election with randomized timeout
│   │   └── log_replication.rs  # AppendEntries + heartbeats
│   ├── partitioning/
│   │   ├── hash_ring.rs    # Consistent hash ring (BTreeMap)
│   │   └── rebalancer.rs   # Partition rebalance planner
│   ├── storage/
│   │   ├── wal.rs          # Write-ahead log (CRC32)
│   │   ├── segment.rs      # Append-only segments
│   │   └── index.rs        # BTreeMap offset index
│   ├── delivery/
│   │   ├── exactly_once.rs # Transaction coordinator
│   │   └── idempotency.rs  # Deduplication store
│   └── clock/
│       └── hlc.rs          # Hybrid Logical Clock
└── tests/
    └── integration.rs      # 10 integration tests
```

## Quick Start

```bash
# Build
cargo build -p production-message-queue

# Run all tests (358 tests)
cargo test -p production-message-queue

# Run with Docker Compose (3-node cluster)
docker-compose up --build

# Configuration
# Edit config/default.toml or set environment variables:
export MQ_NODE_ID=1
export MQ_PORT=9092
export MQ_DATA_DIR=/tmp/mq-data
```

## Configuration

Configuration is loaded from `config/default.toml` with environment variable overrides:

| Env Variable | Config Key | Default | Description |
|-------------|-----------|---------|-------------|
| `MQ_NODE_ID` | `broker.node_id` | 1 | Unique node identifier |
| `MQ_PORT` | `broker.port` | 9092 | TCP listen port |
| `MQ_DATA_DIR` | `broker.data_dir` | `/tmp/mq-data` | Data directory |
| `MQ_BIND_ADDRESS` | `broker.bind_address` | `0.0.0.0` | Bind address |

## Production Features

- **Structured Logging**: JSON output with tracing (thread IDs, file, line)
- **Metrics**: Lock-free atomic counters, latency histograms (p50/p90/p99)
- **Health Checks**: `is_ready()` and `is_alive()` for Kubernetes probes
- **Graceful Shutdown**: SIGTERM/SIGINT handling with cleanup
- **Configuration**: TOML file + 14 environment variable overrides + validation
- **Docker**: Multi-stage build, 3-node docker-compose setup
- **Error Handling**: Comprehensive `thiserror` enum with 15 error variants

## Test Coverage

| Component | Tests |
|-----------|-------|
| Broker (message, topic, partition, producer, consumer) | 78 |
| Storage (WAL, segments, index) | 41 |
| Replication (Raft, election, log replication) | 56 |
| Network (RPC, connection pool, server) | 34 |
| Partitioning (hash ring, rebalancer) | 19 |
| Delivery (exactly-once, idempotency) | 16 |
| Clock (HLC) | 10 |
| Infrastructure (config, metrics, health, error) | 84 |
| Integration tests | 10 |
| **Total** | **348 + 10** |
