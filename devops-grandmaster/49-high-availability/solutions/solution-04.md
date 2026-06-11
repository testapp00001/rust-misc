# Solution 04: Consensus Protocol Implementation

## Part A: Raft Consensus for Database Leader Election

### Question 1: What problem does Raft solve that health checks cannot?

A simple health check (like keepalived's `vrrp_script`) can tell you whether a node is alive, but it cannot tell you which node should be the leader when the network is partitioned. If two nodes both believe the other is dead (due to a network partition), they will both try to become leader -- this is split-brain. Raft solves this by requiring a majority vote before any node can become leader. In a 3-node cluster, a network partition that isolates one node will leave 2 nodes in the majority partition, and those 2 nodes can elect a leader. The isolated node cannot get 2 votes, so it cannot become leader. This mathematical guarantee prevents split-brain without any external coordination.

### Question 2: What is a "term" and why is it important?

A term is a monotonically increasing integer that acts as a logical clock. Each term has at most one leader. When a node starts an election, it increments its term and includes the term number in its vote requests. If a candidate receives votes from a majority of nodes for that term, it becomes the leader. The term mechanism prevents split-brain because: if two nodes start elections simultaneously, the one that gets a majority first becomes leader for that term. The loser sees the higher term and steps down. If a stale leader from term N tries to lead while the cluster is on term N+1, the other nodes will reject its messages because its term is outdated. This ensures that at most one leader exists per term, and only the leader for the current term can make decisions.

### Question 3: Three Raft states and transitions

- **Follower:** The default state. Followers do not initiate elections or replicate data; they receive heartbeats from the leader and vote when asked. A follower transitions to Candidate when it does not receive a heartbeat within the election timeout.

- **Candidate:** A candidate starts an election by incrementing its term, voting for itself, and requesting votes from peers. A candidate becomes Leader if it receives votes from a majority. It becomes a Follower if it receives a heartbeat from a node with a higher or equal term. It remains a Candidate and restarts the election if the election times out without a majority (split vote).

- **Leader:** The leader sends heartbeats to all followers to maintain its authority and replicates log entries. A leader steps down to Follower if it receives a message from a node with a higher term (meaning another leader has been elected, possibly in a different partition).

### Question 4: How does Raft prevent two leaders in a partition?

Raft requires a majority (N/2 + 1) of nodes to agree before a candidate becomes leader. In a 3-node cluster, the majority is 2. If a network partition splits the cluster into 1 node and 2 nodes, only the partition with 2 nodes can achieve a majority vote. The isolated single node can vote for itself, but 1 vote is not a majority, so it cannot become leader. The 2-node partition can elect a leader because they have 2 votes. This means the isolated node cannot accept writes or make decisions, which is the correct behavior -- it is better to reject writes on one node than to allow conflicting writes on two nodes.

### Question 5: Role of etcd in Patroni

In a Patroni-managed PostgreSQL cluster, etcd serves as the distributed key-value store that holds the leader lock. Patroni uses etcd's consensus protocol (Raft internally) to implement leader election without implementing Raft itself. The flow is:

1. Each Patroni instance tries to write a key (e.g., `/service/postgres/leader`) with its own node name as the value.
2. etcd ensures that only one writer succeeds (compare-and-swap with a TTL).
3. The node that holds the key is the PostgreSQL primary.
4. The key has a TTL (e.g., 10 seconds). The leader must periodically renew the key (heartbeat).
5. If the leader fails, the key expires, and another Patroni instance can acquire it.
6. The acquiring node promotes its PostgreSQL instance to primary.

etcd provides the consensus guarantee -- even if the Patroni nodes experience network issues, etcd's own Raft cluster ensures that only one node can hold the leader lock at any time.

## Part B: Simplified Leader Election Pseudocode

```
CLASS LeaderElection:

    CONSTANT FOLLOWER  = "follower"
    CONSTANT CANDIDATE = "candidate"
    CONSTANT LEADER    = "leader"

    CONSTANT MIN_ELECTION_TIMEOUT = 150  // milliseconds
    CONSTANT MAX_ELECTION_TIMEOUT = 300  // milliseconds
    CONSTANT HEARTBEAT_INTERVAL   = 50   // milliseconds

    STATE:
        node_id           : unique string identifier for this node
        current_term      : integer, starts at 0
        voted_for          : nullable string, the node_id voted for in current_term
        state              : FOLLOWER | CANDIDATE | LEADER
        log                : array of log entries
        election_deadline  : timestamp when election timeout expires
        peers              : list of other node_ids in the cluster
        votes_received     : set of node_ids that voted for us in current_term

    METHOD start():
        state = FOLLOWER
        current_term = 0
        voted_for = null
        reset_election_timer()
        LOOP:
            IF state == FOLLOWER:
                handle_follower()
            ELSE IF state == CANDIDATE:
                handle_candidate()
            ELSE IF state == LEADER:
                handle_leader()
            SLEEP(10 milliseconds)

    METHOD reset_election_timer():
        // Random timeout to reduce split-vote probability
        election_deadline = NOW() + RANDOM(MIN_ELECTION_TIMEOUT, MAX_ELECTION_TIMEOUT)

    METHOD handle_follower():
        // If we haven't heard from a leader in time, start an election
        IF NOW() > election_deadline:
            become_candidate()

    METHOD become_candidate():
        state = CANDIDATE
        current_term = current_term + 1
        voted_for = node_id
        votes_received = {node_id}  // Vote for self
        reset_election_timer()
        // Request votes from all peers
        FOR EACH peer IN peers:
            SEND vote_request(current_term, node_id, last_log_index, last_log_term) TO peer

    METHOD handle_candidate():
        // Check if we won the election
        IF SIZE(votes_received) > SIZE(peers) / 2:
            become_leader()
            RETURN

        // Check if election timed out (split vote)
        IF NOW() > election_deadline:
            // Start a new election with a new term
            become_candidate()

    METHOD become_leader():
        state = LEADER
        // Send initial heartbeat to establish authority
        send_heartbeats()

    METHOD handle_leader():
        // Send heartbeats at regular intervals to prevent new elections
        IF time_since_last_heartbeat >= HEARTBEAT_INTERVAL:
            send_heartbeats()

    METHOD send_heartbeats():
        FOR EACH peer IN peers:
            SEND append_entries(current_term, node_id, log, prev_log_index, prev_log_term) TO peer
        last_heartbeat_time = NOW()

    // --- RPC Handlers (called when messages arrive from peers) ---

    METHOD on_receive_vote_request(sender_term, sender_id, sender_last_log_index, sender_last_log_term):
        // If the sender's term is higher, update our term and become follower
        IF sender_term > current_term:
            current_term = sender_term
            state = FOLLOWER
            voted_for = null
            reset_election_timer()

        // Decide whether to grant the vote
        grant_vote = FALSE
        IF sender_term >= current_term:
            IF voted_for == null OR voted_for == sender_id:
                // Check log is at least as up-to-date as ours
                IF sender_last_log_term > last_log_term():
                    grant_vote = TRUE
                ELSE IF sender_last_log_term == last_log_term()
                    AND sender_last_log_index >= last_log_index():
                    grant_vote = TRUE

        IF grant_vote:
            voted_for = sender_id
            reset_election_timer()  // Reset timer when granting a vote

        RETURN vote_response(current_term, grant_vote)

    METHOD on_receive_vote_response(sender_term, vote_granted):
        IF sender_term > current_term:
            current_term = sender_term
            state = FOLLOWER
            voted_for = null
            reset_election_timer()
            RETURN

        IF vote_granted AND state == CANDIDATE:
            votes_received.add(sender_id)

    METHOD on_receive_append_entries(sender_term, sender_id, entries, prev_log_index, prev_log_term):
        // This is a heartbeat or log replication from a leader
        IF sender_term >= current_term:
            // We have a valid leader; become follower
            current_term = sender_term
            state = FOLLOWER
            voted_for = null
            reset_election_timer()  // Reset election timer on valid heartbeat

            // Apply log entries if they match our log at prev_log_index
            IF log_matches(prev_log_index, prev_log_term):
                append_new_entries(entries)

        ELSE IF sender_term < current_term:
            // Sender is stale; reject
            RETURN append_entries_response(current_term, FALSE)

        RETURN append_entries_response(current_term, TRUE)

    // --- Utility Methods ---

    METHOD last_log_index():
        RETURN LENGTH(log)

    METHOD last_log_term():
        IF LENGTH(log) == 0:
            RETURN 0
        RETURN log[LENGTH(log) - 1].term

    METHOD log_matches(index, term):
        IF index == 0:
            RETURN TRUE
        IF index > LENGTH(log):
            RETURN FALSE
        RETURN log[index - 1].term == term
```

### Key Properties of This Pseudocode

1. **Election safety:** At most one leader per term (enforced by majority voting and single vote per term).
2. **Leader append-only:** A leader never overwrites or deletes its log entries.
3. **Log matching:** If two logs contain an entry with the same index and term, all entries up to that index are identical.
4. **Leader completeness:** If a log entry is committed in a given term, that entry will be present in the logs of leaders for all higher-numbered terms.
5. **State machine safety:** If a server has applied a log entry at a given index, no other server will apply a different entry for the same index.

## Part C: etcd Cluster and Patroni Configuration

### etcd Cluster Setup

#### Systemd Override for etcd on node1

```bash
# /etc/systemd/system/etcd.service.d/override.conf (node1 -- 10.0.0.10)

[Service]
ExecStart=
ExecStart=/usr/bin/etcd \
  --name node1 \
  --data-dir /var/lib/etcd/node1 \
  --listen-client-urls http://10.0.0.10:2379,http://127.0.0.1:2379 \
  --advertise-client-urls http://10.0.0.10:2379 \
  --listen-peer-urls http://10.0.0.10:2380 \
  --initial-advertise-peer-urls http://10.0.0.10:2380 \
  --initial-cluster node1=http://10.0.0.10:2380,node2=http://10.0.0.11:2380,node3=http://10.0.0.12:2380 \
  --initial-cluster-state new \
  --initial-cluster-token pg-ha-cluster \
  --heartbeat-interval 250 \
  --election-timeout 1250
```

#### Systemd Override for etcd on node2

```bash
# /etc/systemd/system/etcd.service.d/override.conf (node2 -- 10.0.0.11)

[Service]
ExecStart=
ExecStart=/usr/bin/etcd \
  --name node2 \
  --data-dir /var/lib/etcd/node2 \
  --listen-client-urls http://10.0.0.11:2379,http://127.0.0.1:2379 \
  --advertise-client-urls http://10.0.0.11:2379 \
  --listen-peer-urls http://10.0.0.11:2380 \
  --initial-advertise-peer-urls http://10.0.0.11:2380 \
  --initial-cluster node1=http://10.0.0.10:2380,node2=http://10.0.0.11:2380,node3=http://10.0.0.12:2380 \
  --initial-cluster-state new \
  --initial-cluster-token pg-ha-cluster \
  --heartbeat-interval 250 \
  --election-timeout 1250
```

#### Systemd Override for etcd on node3

```bash
# /etc/systemd/system/etcd.service.d/override.conf (node3 -- 10.0.0.12)

[Service]
ExecStart=
ExecStart=/usr/bin/etcd \
  --name node3 \
  --data-dir /var/lib/etcd/node3 \
  --listen-client-urls http://10.0.0.12:2379,http://127.0.0.1:2379 \
  --advertise-client-urls http://10.0.0.12:2379 \
  --listen-peer-urls http://10.0.0.12:2380 \
  --initial-advertise-peer-urls http://10.0.0.12:2380 \
  --initial-cluster node1=http://10.0.0.10:2380,node2=http://10.0.0.11:2380,node3=http://10.0.0.12:2380 \
  --initial-cluster-state new \
  --initial-cluster-token pg-ha-cluster \
  --heartbeat-interval 250 \
  --election-timeout 1250
```

#### Start etcd on All Nodes

```bash
# On all three nodes:
sudo systemctl daemon-reload
sudo systemctl enable etcd
sudo systemctl start etcd

# Verify the cluster is healthy (run from any node):
etcdctl endpoint health --cluster
# Expected output:
# http://10.0.0.10:2379 is healthy: successfully committed proposal: took = Xms
# http://10.0.0.11:2379 is healthy: successfully committed proposal: took = Xms
# http://10.0.0.12:2379 is healthy: successfully committed proposal: took = Xms

# Check the leader:
etcdctl endpoint status --cluster -w table
# One node will show "true" in the "Is Leader" column
```

### Patroni Configuration for node1

```yaml
# /etc/patroni/patroni.yml (node1 -- 10.0.0.10)

scope: pg-ha-cluster
name: node1

restapi:
  listen: 0.0.0.0:8008
  connect_address: 10.0.0.10:8008
  authentication:
    username: patroni
    password: "Patron1_R3stAPI"

etcd3:
  hosts: 10.0.0.10:2379,10.0.0.11:2379,10.0.0.12:2379
  protocol: http

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576  # 1MB
    synchronous_mode: true
    postgresql:
      use_pg_rewind: true
      use_slots: true
      parameters:
        wal_level: replica
        hot_standby: "on"
        max_wal_senders: 5
        max_replication_slots: 5
        wal_log_hints: "on"
        max_connections: 200
        shared_buffers: "256MB"
        work_mem: "8MB"
        maintenance_work_mem: "128MB"
        effective_cache_size: "768MB"
        synchronous_commit: "on"
        synchronous_standby_names: "*"

  initdb:
    - encoding: UTF8
    - data-checksums

  pg_hba:
    - host replication replicator 10.0.0.0/24 scram-sha-256
    - host all all 10.0.0.0/24 scram-sha-256
    - host all all 0.0.0.0/0 scram-sha-256

  users:
    admin:
      password: "Adm1n_P4ssw0rd"
      options:
        - createrole
        - createdb
    replicator:
      password: "R3pl1c4t0r_P4ss"
      options:
        - replication

postgresql:
  listen: 0.0.0.0:5432
  connect_address: 10.0.0.10:5432
  data_dir: /var/lib/postgresql/15/main
  bin_dir: /usr/lib/postgresql/15/bin
  authentication:
    superuser:
      username: postgres
      password: "Sup3rUs3r_P4ss"
    replication:
      username: replicator
      password: "R3pl1c4t0r_P4ss"
  parameters:
    unix_socket_directories: /var/run/postgresql

tags:
  nofailover: false
  noloadbalance: false
  clonefrom: false
  nosync: false
```

### Patroni Configuration for node2 and node3

The configuration for node2 and node3 is identical to node1 except for:

```yaml
# node2 differences:
name: node2
restapi:
  connect_address: 10.0.0.11:8008
postgresql:
  connect_address: 10.0.0.11:5432

# node3 differences:
name: node3
restapi:
  connect_address: 10.0.0.12:8008
postgresql:
  connect_address: 10.0.0.12:5432
```

### Patroni Systemd Service

```bash
# /etc/systemd/system/patroni.service

[Unit]
Description=Patroni PostgreSQL HA Manager
After=network.target etcd.service postgresql.service
Wants=network.target etcd.service

[Service]
Type=simple
User=postgres
Group=postgres
ExecStart=/usr/local/bin/patroni /etc/patroni/patroni.yml
ExecReload=/bin/kill -HUP $MAINPID
KillMode=process
TimeoutSec=30
Restart=on-failure
RestartSec=10
LimitNOFILE=65536

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=patroni

[Install]
WantedBy=multi-user.target
```

```bash
# On all nodes:
sudo systemctl daemon-reload
sudo systemctl enable patroni
sudo systemctl start patroni

# Verify cluster status:
patronictl -c /etc/patroni/patroni.yml list
# Expected output:
# + Cluster: pg-ha-cluster (7xxxxxxx) ---+----+-----------+
# | Member | Host       | Role    | State   | TL | Lag in MB |
# +--------+------------+---------+---------+----+-----------+
# | node1  | 10.0.0.10  | Leader  | running |  1 |           |
# | node2  | 10.0.0.11  | Replica | running |  1 |         0 |
# | node3  | 10.0.0.12  | Replica | running |  1 |         0 |
# +--------+------------+---------+---------+----+-----------+
```

## Part D: Testing Leader Election

### Scenario 1: Leader (node1) Crashes

```bash
# Step 1: Verify current leader
patronictl -c /etc/patroni/patroni.yml list
# Note which node is the Leader (e.g., node1)

# Step 2: Kill the Patroni process on the leader to simulate a crash
# On node1:
sudo kill -9 $(pgrep -f "patroni /etc/patroni")

# Step 3: Watch for failover (from any other node)
watch -n 1 'patronictl -c /etc/patroni/patroni.yml list'

# Expected behavior:
# - Within 10-30 seconds (depending on TTL settings), the leader key in etcd expires
# - One of the remaining nodes (node2 or node3) acquires the leader key
# - That node promotes its PostgreSQL instance to primary
# - The other replica switches to follow the new primary

# Step 4: Verify the new leader
patronictl -c /etc/patroni/patroni.yml list
# node1 should show as "not running" or absent
# One of node2/node3 should be "Leader"

# Step 5: Verify data integrity
psql -h 10.0.0.100 -U postgres -c "SELECT count(*) FROM orders;"
# (Assuming the VIP is managed by Patroni or a load balancer pointing to the leader)
```

### Scenario 2: Network Partition Isolating One Node

```bash
# Step 1: Verify cluster is healthy
patronictl -c /etc/patroni/patroni.yml list

# Step 2: Simulate a network partition isolating node3
# On node3, block traffic to the other two nodes:
sudo iptables -A INPUT -s 10.0.0.10 -j DROP
sudo iptables -A INPUT -s 10.0.0.11 -j DROP
sudo iptables -A OUTPUT -d 10.0.0.10 -j DROP
sudo iptables -A OUTPUT -d 10.0.0.11 -j DROP

# Step 3: Observe behavior on the majority partition (node1 + node2)
# From node1:
patronictl -c /etc/patroni/patroni.yml list
# node3 should appear as "not running" or absent after its health check times out
# The cluster continues operating with node1 (Leader) and node2 (Replica)

# Step 4: Observe behavior on the isolated node (node3)
# From node3 (if you can access its console):
patronictl -c /etc/patroni/patroni.yml list
# node3 cannot reach etcd (because etcd also lost quorum on node3's partition)
# Patroni on node3 will demote PostgreSQL to read-only (replica mode)
# This is the correct behavior: the isolated node STOPS accepting writes

# Step 5: Verify writes still work on the majority partition
psql -h 10.0.0.10 -U postgres -c \
  "INSERT INTO orders (tenant_id, amount, status) VALUES (9999, 50.00, 'test_partition');"
# Should succeed on node1 (the leader in the majority partition)
```

### Scenario 3: Leader Recovers After Partition

```bash
# Step 1: Heal the network partition on node3
sudo iptables -D INPUT -s 10.0.0.10 -j DROP
sudo iptables -D INPUT -s 10.0.0.11 -j DROP
sudo iptables -D OUTPUT -d 10.0.0.10 -j DROP
sudo iptables -D OUTPUT -d 10.0.0.11 -j DROP

# Step 2: Watch node3 rejoin the cluster
watch -n 1 'patronictl -c /etc/patroni/patroni.yml list'

# Expected behavior:
# - node3's Patroni detects it is in the minority (was isolated)
# - node3's PostgreSQL is already in replica mode (demoted during partition)
# - node3 reconnects to etcd and sees the current leader (node1 or node2)
# - node3 starts replicating from the current leader to catch up
# - There is NO split-brain: node3 does NOT try to become leader

# Step 3: Verify node3 is a healthy replica
patronictl -c /etc/patroni/patroni.yml list
# node3 should show as "Replica" with "Lag in MB" = 0 (after catching up)

# Step 4: Verify no data loss
# The write from Scenario 2 should be visible on node3 after replication:
psql -h 10.0.0.12 -U postgres -c \
  "SELECT * FROM orders WHERE tenant_id = 9999;"
# Should show the row inserted during the partition
```

### Scenario 4: Rolling Restart of All Nodes

```bash
# Step 1: Verify cluster is healthy
patronictl -c /etc/patroni/patroni.yml list

# Step 2: Restart node3 (a replica) first -- safest because it is not the leader
sudo systemctl restart patroni
# Wait for node3 to rejoin as a healthy replica
watch -n 1 'patronictl -c /etc/patroni/patroni.yml list'
# Confirm node3 is "Replica" and "running" with 0 lag

# Step 3: Perform a controlled switchover from node1 to node2
# This makes node2 the leader, so we can restart node1 next
patronictl -c /etc/patroni/patroni.yml switchover --master node1 --candidate node2 --force

# Verify:
patronictl -c /etc/patroni/patroni.yml list
# node2 should now be Leader, node1 should be Replica

# Step 4: Restart node1 (now a replica)
sudo systemctl restart patroni
# Wait for node1 to rejoin as a healthy replica

# Step 5: Perform a switchover back to node1 (optional)
patronictl -c /etc/patroni/patroni.yml switchover --master node2 --candidate node1 --force

# Step 6: Restart node2 (now a replica again)
sudo systemctl restart patroni

# Verify: At no point was there more than one leader
# The cluster maintained availability throughout (at least 2 of 3 nodes were always up)
```

## Common Mistakes to Avoid

1. **Hardcoding etcd endpoints instead of using discovery.** If you hardcode `hosts: 10.0.0.10:2379` and that node is down, Patroni cannot connect to etcd even though the cluster is healthy. Always list all etcd nodes so Patroni can fail over to another endpoint.

2. **Setting TTL too low.** A TTL of 5 seconds means the leader must renew its lock every 5 seconds. If the leader is briefly busy (e.g., a large checkpoint), it might miss a renewal and trigger an unnecessary failover. Start with TTL=30 and reduce only if faster failover is required.

3. **Not configuring `synchronous_mode`.** Without synchronous replication, a failover might lose committed transactions that were replicated asynchronously. `synchronous_mode: true` ensures that at least one replica has confirmed receipt of each transaction before the commit is acknowledged to the client. This adds latency but guarantees zero data loss.

4. **Forgetting `use_pg_rewind`.** Without `pg_pg_rewind`, a former primary that was down must be completely rebuilt from scratch (pg_basebackup). `pg_rewind` allows the former primary to "rewind" its timeline to match the new primary, which is much faster (seconds vs. minutes for large databases).

5. **Testing failover only in a clean environment.** Real failovers happen during problems: disk full, OOM kill, network partition, split-brain. Test failover under stress (high write load, network degradation, disk pressure) to find the edge cases.

6. **Not monitoring etcd health independently.** etcd is a single point of failure for Patroni. If etcd loses quorum, Patroni cannot perform leader election, and the cluster will eventually stop accepting writes (when the current leader's TTL expires). Monitor etcd cluster health, disk performance (etcd is sensitive to disk latency), and leader election frequency.

## Key Takeaway

Consensus protocols like Raft provide the mathematical guarantee that prevents split-brain: a majority of nodes must agree before any decision is made. Patroni leverages etcd's Raft implementation to store a leader lock with a TTL, ensuring that only one PostgreSQL instance can be primary at any time. The key insight is that the consensus layer (etcd) is separate from the data layer (PostgreSQL) -- this separation means you can use a mature consensus implementation (etcd's Raft) without implementing it yourself, while Patroni focuses on the PostgreSQL-specific logic (promotion, replication, rewind).
