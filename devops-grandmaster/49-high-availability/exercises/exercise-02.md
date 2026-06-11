# Exercise 02: Active-Passive Cluster Setup

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Set up a production-style active-passive PostgreSQL cluster with streaming replication, a floating virtual IP, health monitoring, and automated failover using keepalived.

## Scenario

Your team runs a PostgreSQL database that serves a critical ordering system. The current setup is a single instance, and management has mandated active-passive HA. You have two servers:

- **node1** (10.0.0.10): Will be the initial primary
- **node2** (10.0.0.11): Will be the hot standby

A virtual IP (10.0.0.100) must float between the two nodes. If the primary fails, the standby must take over the VIP within 10 seconds.

## Tasks

### Part A: Configure PostgreSQL Streaming Replication

Configure node1 as the primary and node2 as a hot standby using PostgreSQL streaming replication.

On the primary (node1), configure `postgresql.conf`:

```bash
# Write the required postgresql.conf settings for the primary node
# Include: listen_addresses, wal_level, max_wal_senders, wal_keep_size
```

On the primary (node1), configure `pg_hba.conf` to allow replication:

```bash
# Write the pg_hba.conf line that allows node2 to connect for replication
# Use the replication keyword and a dedicated user
```

On the standby (node2), set up the base backup and `standby.signal`:

```bash
# Write the commands to:
# 1. Stop PostgreSQL on node2
# 2. Clear the data directory
# 3. Run pg_basebackup from node1
# 4. Create standby.signal
# 5. Configure primary_conninfo in postgresql.auto.conf
# 6. Start PostgreSQL on node2
```

### Part B: Set Up a Virtual IP (VIP)

Configure a virtual IP (10.0.0.100) on node1 that can be moved to node2.

```bash
# Write the command to add a virtual IP to a network interface on node1
# The VIP should be 10.0.0.100/24 on interface eth0
```

Write the script to check whether the VIP is currently assigned to the local node:

```bash
#!/bin/bash
# vip-status.sh
# Write a script that:
# 1. Checks if the VIP is bound to the local interface
# 2. Returns exit code 0 if present, 1 if absent
```

### Part C: Write a Health Check Script

Create a script that checks whether the local PostgreSQL instance is healthy and capable of serving as primary.

```bash
#!/bin/bash
# pg-health-check.sh
# Write a script that:
# 1. Checks if PostgreSQL is accepting connections (pg_isready)
# 2. Verifies the instance is in "primary" mode (not recovery)
# 3. Checks replication lag if this node is primary
# 4. Returns exit code 0 for healthy, 1 for unhealthy
# 5. Logs the result with a timestamp
```

### Part D: Configure Automatic Failover with keepalived

Write the keepalived configuration that ties the VIP, health check, and failover together.

```bash
# Write the keepalived.conf for BOTH nodes
# Requirements:
# - Use VRRP instance for the VIP
# - node1 starts as MASTER with higher priority (101)
# - node2 starts as BACKUP with lower priority (100)
# - Use the health check script from Part C as a track_script
# - Reduce health_check interval to 2 seconds
# - Set advert_int to 1 second for fast failover detection
# - Configure notification scripts for failover events
```

## Success Criteria

- [ ] PostgreSQL streaming replication is active (verified with `SELECT * FROM pg_stat_replication`)
- [ ] The VIP (10.0.0.100) is pingable and bound to node1
- [ ] The health check script correctly identifies primary vs standby mode
- [ ] Stopping PostgreSQL on node1 causes the VIP to migrate to node2 within 10 seconds
- [ ] Starting PostgreSQL on node1 again does NOT cause a split-brain (VIP stays on node2 unless explicitly failed back)

## Hints

<details>
<summary>Hint 1: Replication setup</summary>

For streaming replication, the critical `postgresql.conf` settings on the primary are:

```
listen_addresses = '*'
wal_level = replica
max_wal_senders = 5
wal_keep_size = '1GB'
```

The `pg_hba.conf` line should look like:

```
host replication replicator 10.0.0.11/32 md5
```

Create the replication user with: `CREATE USER replicator WITH REPLICATION PASSWORD 'secure_password';`

</details>

<details>
<summary>Hint 2: VIP management</summary>

To add a VIP on Linux:

```bash
sudo ip addr add 10.0.0.100/24 dev eth0
```

To check if the VIP is present:

```bash
ip addr show eth0 | grep "10.0.0.100"
```

keepalived will manage the VIP automatically once configured, but you need to remove any manually added VIP first to avoid conflicts.

</details>

<details>
<summary>Hint 3: keepalived configuration structure</summary>

A keepalived.conf has three main blocks:
1. `vrrp_script` -- defines the health check
2. `vrrp_instance` -- defines the VIP and failover behavior
3. `notify_master` / `notify_backup` / `notify_fault` -- scripts called on state transitions

The `track_script` directive inside `vrrp_instance` links the health check to the VRRP instance. If the script returns a non-zero exit code, the node's effective priority is reduced by the `weight` value.

</details>

## What You Should Understand After This Exercise

Active-passive is the simplest HA pattern, but "simple" does not mean "easy." The hard problems are: ensuring the passive node is truly ready to take over (replication lag), avoiding split-brain where both nodes think they are primary, and making failover fast enough to meet your availability target. The health check script is the brain of the system -- if it is wrong, failover will either happen too often (flapping) or not happen when it should.
