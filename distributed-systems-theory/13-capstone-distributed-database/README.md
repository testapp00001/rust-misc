# Module 13: Capstone Distributed Database

## Project Overview

This capstone project implements a simplified distributed database that combines all the
concepts studied throughout the distributed systems theory curriculum. It brings together
consensus, storage, replication, sharding, transactions, networking, CRDTs, logical clocks,
and client-facing APIs into a cohesive system.

The goal is not to build a production-ready database, but to demonstrate how the individual
building blocks of a distributed system fit together and interact.

## Architecture

The system is organized into nine subsystems, each representing a core distributed systems concept:

### Consensus (`consensus/`)
Implements a simplified Raft consensus protocol. A `RaftNode` can transition between
Follower, Candidate, and Leader states. It maintains a replicated log, proposes commands,
and applies committed entries to a deterministic state machine.

**Key types:** `RaftNode`, `Command`, `LogEntry`, `RaftState`, `StateMachine`

### Storage (`storage/`)
Provides an in-memory key-value storage engine and a write-ahead log (WAL) for durability
simulation. The engine supports put, get, delete, and prefix-scan operations. The WAL
records every mutation before it is applied, enabling recovery.

**Key types:** `StorageEngine`, `WAL`, `WALEntry`

### Replication (`replication/`)
Simulates log replication from a leader to followers. Each replication attempt returns a
result indicating success, failure, or a stale-term rejection -- mirroring real-world
leader-follower replication dynamics.

**Key types:** `LogReplicator`, `ReplicationResult`

### Sharding (`sharding/`)
Implements consistent hashing for data partitioning and a rebalancer that computes the
minimum set of key migrations needed when shards are added or removed. The consistent hash
ring uses virtual nodes for better load distribution.

**Key types:** `ConsistentHashRing`, `Rebalancer`

### Transactions (`transactions/`)
Provides a Saga-based distributed transaction coordinator with automatic compensation, and
a transaction abstraction supporting multiple isolation levels (Read Committed, Snapshot
Isolation, Serializable).

**Key types:** `SagaCoordinator`, `SagaStep`, `Transaction`, `IsolationLevel`

### Networking (`networking/`)
Implements a simple in-process RPC framework with handler registration and a heartbeat-based
failure detector that tracks node liveness through Alive / Suspected / Dead states.

**Key types:** `RPCServer`, `RPCRequest`, `RPCResponse`, `FailureDetector`, `NodeStatus`

### CRDTs (`crdt/`)
Implements conflict-free replicated data types: a grow-only counter (G-Counter) and a
positive-negative counter (PN-Counter). Both support merge operations that preserve
convergence guarantees without coordination.

**Key types:** `GCounter`, `PNCounter`

### Clocks (`clock/`)
Implements a Hybrid Logical Clock (HLC) that combines physical time with a logical counter
to provide causally ordered timestamps across distributed nodes.

**Key types:** `HLC`

### API (`api/`)
Exposes GET, PUT, and DELETE handlers that wire together the storage engine and HLC
through a shared `DatabaseState`. Each handler returns a response with the current
HLC timestamp for causal tracking.

**Key types:** `DatabaseState`, `GetResponse`, `PutResponse`, `DeleteResponse`

## How to Build and Test

```bash
# Build the project
cargo build -p capstone-distributed-database

# Run tests
cargo test -p capstone-distributed-database

# Run the demo node
cargo run -p capstone-distributed-database
```

## Design Decisions and Trade-offs

1. **Simplified Raft**: The Raft implementation is single-node and synchronous. Real Raft
   requires network RPCs, election timeouts, and persistent storage. We simulate replication
   and commit for clarity.

2. **In-memory storage**: The storage engine is purely in-memory. The WAL is append-only
   for demonstration. A production system would persist both to disk.

3. **Consistent hashing with virtual nodes**: Virtual nodes improve load distribution at
   the cost of larger ring metadata. The number is configurable.

4. **Saga transactions**: Sagas provide eventual consistency for distributed transactions
   through compensation. This trades immediate atomicity for availability and partition tolerance.

5. **HLC over pure Lamport clocks**: HLC preserves both physical time bounds and causal
   ordering, making it more practical for real systems than purely logical clocks.

6. **In-process RPC**: The RPC layer uses function pointers instead of network transport
   to keep the focus on protocol logic rather than serialization and networking.
