# Exercise 04: Consensus Protocol Implementation

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Understand how consensus protocols (specifically Raft) enable leader election in distributed databases, implement a simplified leader election algorithm, and configure a real etcd + Patroni cluster for PostgreSQL HA.

## Scenario

Your team has decided that keepalived-based failover (from Exercise 02) is not sophisticated enough for your needs. You need a consensus-based approach that can handle network partitions, prevent split-brain without human intervention, and support automatic leader election with proper term tracking. You are deploying a 3-node cluster:

- **node1** (10.0.0.10)
- **node2** (10.0.0.11)
- **node3** (10.0.0.12)

## Tasks

### Part A: Explain Raft Consensus for Database Leader Election

Answer the following questions in your own words. Each answer should be 2-4 sentences.

1. What problem does the Raft consensus protocol solve that a simple health check (like in keepalived) cannot?

2. In Raft, what is a "term" and why is it important for preventing split-brain?

3. Explain the three states a Raft node can be in (follower, candidate, leader) and what causes transitions between them.

4. How does Raft ensure that a network partition cannot result in two nodes both believing they are the leader?

5. In the context of PostgreSQL HA with Patroni, what specific role does the Raft-based key-value store (etcd) play?

### Part B: Write Pseudocode for Simplified Leader Election

Implement a simplified leader election algorithm in pseudocode. This is not full Raft -- it is a minimal version that captures the core safety properties.

```
# Write pseudocode for a LeaderElection class with the following behavior:
#
# State:
#   - node_id: unique identifier for this node
#   - current_term: monotonically increasing term number
#   - voted_for: which node this node voted for in the current term
#   - state: FOLLOWER | CANDIDATE | LEADER
#   - election_timeout: random timeout between 150-300ms
#   - heartbeat_interval: 50ms
#
# Behavior:
#   - On startup: become FOLLOWER, start election timer
#   - On election timeout: become CANDIDATE, increment term, request votes
#   - On receiving majority votes: become LEADER, send heartbeats
#   - On receiving heartbeat from higher term: become FOLLOWER
#   - On receiving vote request with higher term: update term, grant vote
#   - On network partition: minority partition cannot elect a leader
```

### Part C: Configure etcd Cluster for Patroni-Based Leader Election

Write the configuration files for a 3-node etcd cluster and a Patroni configuration that uses it.

etcd systemd service override for node1:

```bash
# Write the etcd.conf.yaml or systemd override for node1
# Include: name, initial-cluster, listen-client-urls, advertise-client-urls
# Include: listen-peer-urls, initial-advertise-peer-urls
# Include: initial-cluster-state
```

Patroni configuration (patroni.yml) for node1:

```yaml
# Write the complete patroni.yml for node1
# Include:
#   - scope (cluster name)
#   - name (node name)
#   - restapi listen address
#   - etcd3 connection settings
#   - PostgreSQL bootstrap configuration
#   - PostgreSQL parameters for replication
#   - TTL and loop timing settings
#   - retry timeout and maximum lag settings
```

Patroni systemd service file:

```bash
# Write the systemd unit file for Patroni
# Include: Description, After (postgresql, etcd, network)
# Include: ExecStart, User, Restart policy
```

### Part D: Test Leader Election by Simulating Node Failures

Write a test plan that validates the cluster behavior under various failure scenarios.

For each scenario below, write the exact commands you would run and the expected outcome:

**Scenario 1: Leader (node1) crashes**

```bash
# Commands to simulate leader crash and verify failover
# Expected: How long until a new leader is elected? Which node wins?
```

**Scenario 2: Network partition isolating one node**

```bash
# Commands to simulate a network partition using iptables
# Expected: Does the majority partition maintain a leader?
# Does the isolated node stop accepting writes?
```

**Scenario 3: Leader recovers after partition**

```bash
# Commands to heal the partition and verify behavior
# Expected: Does the recovered node become a follower?
# Is there any data loss or split-brain?
```

**Scenario 4: Rolling restart of all nodes**

```bash
# Commands to restart nodes one at a time without losing the leader
# Expected: At any point, is there more than one leader?
```

## Success Criteria

- [ ] Raft explanation answers demonstrate understanding of term-based leader election and split-brain prevention
- [ ] Pseudocode correctly handles the follower -> candidate -> leader transition with majority voting
- [ ] etcd cluster configuration forms a working 3-node cluster (verified with `etcdctl endpoint health`)
- [ ] Patroni configuration connects to etcd and manages PostgreSQL lifecycle
- [ ] Test plan covers crash, partition, recovery, and rolling restart scenarios with specific commands

## Hints

<details>
<summary>Hint 1: Raft term mechanics</summary>

A Raft term is a logical clock. Each term has at most one leader. When a node starts an election, it increments its term. If a node receives a message with a higher term, it immediately steps down to follower. This is the key mechanism that prevents split-brain: even if two nodes both start elections, the one with the higher term wins, and once a majority votes for a candidate, that candidate becomes the leader for that term. A stale leader from a previous term will see the higher term and step down.

</details>

<details>
<summary>Hint 2: Patroni timing parameters</summary>

The critical timing parameters in Patroni are:
- `ttl`: How long the leader lock in etcd lives before expiring (default: 30s)
- `loop_wait`: How often Patroni checks its state (default: 10s)
- `retry_timeout`: Maximum time to retry failed operations (default: 10s)
- `maximum_lag_on_failover`: Maximum replication lag (in bytes) for a standby to be eligible as failover target

Fast failover requires: `ttl` = 10s, `loop_wait` = 2s, `retry_timeout` = 5s. But aggressive timing increases the risk of unnecessary failovers during brief network hiccups.

</details>

<details>
<summary>Hint 3: etcd cluster formation</summary>

For a 3-node etcd cluster, each node needs to know about all peers at initial startup. The `initial-cluster` flag is a comma-separated list of `name=peer_url` entries. The `initial-cluster-state` must be `new` for the first boot and can be changed to `existing` when adding nodes later. Use `etcdctl member add` to add nodes to an existing cluster rather than modifying `initial-cluster`.

</details>

## What You Should Understand After This Exercise

Consensus protocols are the foundation of reliable distributed systems. Raft works because it imposes a total order on events through terms and requires a majority to agree before any decision is made. This means a minority partition can never elect a leader or accept writes, which is exactly the property that prevents split-brain. Patroni leverages this by storing the leader lock in etcd -- only the node that holds the lock can be primary, and the lock expires if the leader fails, guaranteeing automatic failover without data corruption.
