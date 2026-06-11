# Solution 02: Active-Passive Cluster Setup

## Part A: PostgreSQL Streaming Replication

### Primary Node Configuration (node1 -- 10.0.0.10)

Edit `postgresql.conf` on the primary:

```bash
# /etc/postgresql/15/main/postgresql.conf (primary)

# Accept connections from both nodes and any application servers
listen_addresses = '*'

# Enable streaming replication (minimum wal_level for replication)
wal_level = replica

# Allow up to 5 concurrent replication connections
max_wal_senders = 5

# Keep 1GB of WAL segments for standby catch-up after brief disconnection
wal_keep_size = '1GB'

# Enable WAL archiving for point-in-time recovery (optional but recommended)
archive_mode = on
archive_command = 'cp %p /var/lib/postgresql/wal_archive/%f'

# Connection limits
max_connections = 200
```

Edit `pg_hba.conf` on the primary to allow replication from the standby:

```bash
# /etc/postgresql/15/main/pg_hba.conf

# TYPE  DATABASE        USER            ADDRESS                 METHOD

# Allow replication connections from the standby node
host    replication     replicator      10.0.0.11/32            scram-sha-256

# Allow application connections from both nodes and local
host    all             all             10.0.0.0/24             scram-sha-256
host    all             all             127.0.0.1/32            scram-sha-256
```

Create the replication user on the primary:

```sql
-- Connect to the primary as superuser
-- psql -h 10.0.0.10 -U postgres

CREATE USER replicator WITH REPLICATION PASSWORD 'Str0ng_R3pl1c4t10n_P4ss';
```

Restart PostgreSQL on the primary:

```bash
sudo systemctl restart postgresql
```

### Standby Node Setup (node2 -- 10.0.0.11)

```bash
#!/bin/bash
# setup-standby.sh -- Run on node2

# Step 1: Stop PostgreSQL on the standby
sudo systemctl stop postgresql

# Step 2: Back up and clear the data directory
sudo -u postgres mv /var/lib/postgresql/15/main /var/lib/postgresql/15/main.bak
sudo -u postgres mkdir -p /var/lib/postgresql/15/main

# Step 3: Run pg_basebackup from the primary
# -R creates standby.signal and configures primary_conninfo automatically
sudo -u postgres pg_basebackup \
  -h 10.0.0.10 \
  -U replicator \
  -D /var/lib/postgresql/15/main \
  -Fp \
  -Xs \
  -P \
  -R

# The -R flag automatically creates:
# - standby.signal file (tells PostgreSQL to start in standby mode)
# - primary_conninfo in postgresql.auto.conf

# Step 4: Verify standby.signal was created
ls -la /var/lib/postgresql/15/main/standby.signal

# Step 5: Optionally tune standby-specific settings
sudo -u postgres bash -c "cat >> /var/lib/postgresql/15/main/postgresql.conf << EOF

# Standby-specific settings
hot_standby = on
hot_standby_feedback = on
max_standby_streaming_delay = 30s
max_standby_archive_delay = 60s
EOF"

# Step 6: Start PostgreSQL on the standby
sudo systemctl start postgresql

# Step 7: Verify replication is working
# On the primary (node1):
# psql -c "SELECT client_addr, state, sent_lsn, write_lsn, flush_lsn, replay_lsn FROM pg_stat_replication;"
```

### Verify Replication

```bash
# On the primary (node1), check that the standby is connected:
psql -h 10.0.0.10 -U postgres -c \
  "SELECT client_addr, state, sent_lsn, write_lsn, flush_lsn, replay_lsn FROM pg_stat_replication;"

# Expected output:
#  client_addr | state     | sent_lsn    | write_lsn   | flush_lsn   | replay_lsn
# -------------+-----------+-------------+-------------+-------------+------------
#  10.0.0.11   | streaming | 0/3000060   | 0/3000060   | 0/3000060   | 0/3000060

# On the standby (node2), verify it is in recovery mode:
psql -h 10.0.0.11 -U postgres -c "SELECT pg_is_in_recovery();"
# Expected: true
```

## Part B: Virtual IP (VIP) Setup

### Manual VIP Assignment (for testing)

```bash
# Add the VIP to node1's eth0 interface
sudo ip addr add 10.0.0.100/24 dev eth0

# Verify the VIP is assigned
ip addr show eth0 | grep "10.0.0.100"
# Expected: inet 10.0.0.100/24 scope global secondary eth0

# Test connectivity
ping -c 3 10.0.0.100
```

### VIP Status Check Script

```bash
#!/bin/bash
# vip-status.sh -- Checks if the VIP is bound to the local interface

VIP="10.0.0.100"
INTERFACE="eth0"

# Check if the VIP is present on the interface
if ip addr show "$INTERFACE" | grep -q "inet ${VIP}/"; then
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] VIP ${VIP} is present on ${INTERFACE}"
    exit 0
else
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] VIP ${VIP} is NOT present on ${INTERFACE}"
    exit 1
fi
```

```bash
# Make it executable
chmod +x vip-status.sh
```

## Part C: Health Check Script

```bash
#!/bin/bash
# pg-health-check.sh -- Comprehensive PostgreSQL health check for keepalived

VIP="10.0.0.100"
PGUSER="postgres"
PGDATABASE="postgres"
LOG_FILE="/var/log/pg-health-check.log"
MAX_REPLICATION_LAG_BYTES=104857600  # 100MB

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

# Step 1: Check if PostgreSQL is accepting connections
if ! pg_isready -q -U "$PGUSER" -d "$PGDATABASE" 2>/dev/null; then
    log "FAIL: PostgreSQL is not accepting connections"
    exit 1
fi

# Step 2: Verify the instance is in primary mode (not in recovery)
IS_RECOVERY=$(psql -U "$PGUSER" -d "$PGDATABASE" -tAc "SELECT pg_is_in_recovery();" 2>/dev/null)

if [ "$IS_RECOVERY" = "t" ]; then
    log "FAIL: Instance is in recovery mode (standby), not suitable as primary"
    exit 1
fi

# Step 3: Check if any replicas are connected (optional, for monitoring)
REPLICA_COUNT=$(psql -U "$PGUSER" -d "$PGDATABASE" -tAc \
    "SELECT count(*) FROM pg_stat_replication;" 2>/dev/null)
log "INFO: ${REPLICA_COUNT} replica(s) connected"

# Step 4: Check for long-running transactions that might indicate problems
LONG_TX=$(psql -U "$PGUSER" -d "$PGDATABASE" -tAc \
    "SELECT count(*) FROM pg_stat_activity WHERE state = 'active' AND query_start < now() - interval '5 minutes';" 2>/dev/null)

if [ "$LONG_TX" -gt 0 ]; then
    log "WARNING: ${LONG_TX} long-running transaction(s) detected (>5 minutes)"
    # Not a hard failure, but worth logging
fi

# Step 5: Check disk space on the data directory
DATA_DIR=$(psql -U "$PGUSER" -d "$PGDATABASE" -tAc "SHOW data_directory;" 2>/dev/null)
DISK_USAGE=$(df -h "$DATA_DIR" | awk 'NR==2 {print $5}' | tr -d '%')

if [ "$DISK_USAGE" -gt 90 ]; then
    log "FAIL: Data directory disk usage is at ${DISK_USAGE}%"
    exit 1
elif [ "$DISK_USAGE" -gt 80 ]; then
    log "WARNING: Data directory disk usage is at ${DISK_USAGE}%"
fi

log "OK: PostgreSQL primary is healthy (disk: ${DISK_USAGE}%, replicas: ${REPLICA_COUNT})"
exit 0
```

```bash
chmod +x pg-health-check.sh
sudo mv pg-health-check.sh /usr/local/bin/
sudo mkdir -p /var/log && sudo touch /var/log/pg-health-check.log
```

## Part D: keepalived Configuration

### Install keepalived

```bash
# On both nodes
sudo apt-get update && sudo apt-get install -y keepalived
```

### Node1 (MASTER) keepalived.conf

```bash
# /etc/keepalived/keepalived.conf -- node1 (10.0.0.10, initial MASTER)

# Global settings
global_defs {
    router_id NODE1
    enable_script_security
    script_user root
}

# Health check script definition
vrrp_script pg_health_check {
    script "/usr/local/bin/pg-health-check.sh"
    interval 2          # Run every 2 seconds
    weight 50           # Reduce priority by 50 if script fails
    fall 3              # Require 3 consecutive failures to mark down
    rise 2              # Require 2 consecutive successes to mark up
    timeout 3           # Script must complete within 3 seconds
}

# VRRP instance for the VIP
vrrp_instance VI_PG {
    state MASTER
    interface eth0
    virtual_router_id 51        # Must be the same on both nodes
    priority 101                # Higher priority = preferred MASTER
    advert_int 1                # Send VRRP advertisements every 1 second

    # Authentication between VRRP peers
    authentication {
        auth_type PASS
        auth_pass PgH4_Clust3r    # Must match on both nodes
    }

    # Virtual IP address
    virtual_ipaddress {
        10.0.0.100/24 dev eth0
    }

    # Track the PostgreSQL health check
    track_script {
        pg_health_check
    }

    # Notification scripts
    notify_master "/usr/local/bin/keepalived-notify.sh MASTER"
    notify_backup "/usr/local/bin/keepalived-notify.sh BACKUP"
    notify_fault  "/usr/local/bin/keepalived-notify.sh FAULT"
}
```

### Node2 (BACKUP) keepalived.conf

```bash
# /etc/keepalived/keepalived.conf -- node2 (10.0.0.11, BACKUP)

global_defs {
    router_id NODE2
    enable_script_security
    script_user root
}

vrrp_script pg_health_check {
    script "/usr/local/bin/pg-health-check.sh"
    interval 2
    weight 50
    fall 3
    rise 2
    timeout 3
}

vrrp_instance VI_PG {
    state BACKUP
    interface eth0
    virtual_router_id 51        # Must match node1
    priority 100                # Lower priority = BACKUP
    advert_int 1

    authentication {
        auth_type PASS
        auth_pass PgH4_Clust3r    # Must match node1
    }

    virtual_ipaddress {
        10.0.0.100/24 dev eth0
    }

    track_script {
        pg_health_check
    }

    notify_master "/usr/local/bin/keepalived-notify.sh MASTER"
    notify_backup "/usr/local/bin/keepalived-notify.sh BACKUP"
    notify_fault  "/usr/local/bin/keepalived-notify.sh FAULT"
}
```

### Notification Script

```bash
#!/bin/bash
# /usr/local/bin/keepalived-notify.sh -- Called on VRRP state changes

STATE=$1
VIP="10.0.0.100"
LOG_FILE="/var/log/keepalived-notify.log"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
HOSTNAME=$(hostname)

log() {
    echo "[${TIMESTAMP}] [${HOSTNAME}] $1" >> "$LOG_FILE"
}

case "$STATE" in
    MASTER)
        log "TRANSITION TO MASTER: This node is now serving the VIP ${VIP}"
        # Ensure the VIP is bound (keepalived should handle this, but belt-and-suspenders)
        if ! ip addr show eth0 | grep -q "inet ${VIP}/"; then
            ip addr add ${VIP}/24 dev eth0
            log "VIP ${VIP} added to eth0"
        fi
        # Optional: Send alert to monitoring system
        # curl -X POST http://alertmanager:9093/api/v1/alerts -d '...'
        ;;
    BACKUP)
        log "TRANSITION TO BACKUP: This node is no longer serving the VIP ${VIP}"
        # Remove VIP if it is still bound (should not be needed, but safety check)
        if ip addr show eth0 | grep -q "inet ${VIP}/"; then
            ip addr del ${VIP}/24 dev eth0
            log "VIP ${VIP} removed from eth0"
        fi
        ;;
    FAULT)
        log "FAULT DETECTED: Health check has failed on this node"
        # Remove VIP immediately
        if ip addr show eth0 | grep -q "inet ${VIP}/"; then
            ip addr del ${VIP}/24 dev eth0
            log "VIP ${VIP} removed from eth0 due to fault"
        fi
        # Optional: Page on-call engineer
        ;;
    *)
        log "UNKNOWN STATE: ${STATE}"
        ;;
esac
```

```bash
chmod +x /usr/local/bin/keepalived-notify.sh
```

### Start keepalived on Both Nodes

```bash
# On both nodes
sudo systemctl enable keepalived
sudo systemctl start keepalived

# Verify keepalived is running
sudo systemctl status keepalived

# Check the VRRP state
sudo journalctl -u keepalived -f
```

### Test Failover

```bash
# Verify the VIP is on node1
ping -c 3 10.0.0.100

# Stop PostgreSQL on node1 to trigger failover
sudo systemctl stop postgresql

# Wait up to 10 seconds and check the VIP on node2
# On node2:
ip addr show eth0 | grep "10.0.0.100"
# Should show the VIP has moved to node2

# Verify applications can still connect via the VIP
psql -h 10.0.0.100 -U postgres -c "SELECT pg_is_in_recovery();"
# Should return: false (node2 is now primary -- note: you need to promote it)
```

### Important Note on Promotion

Streaming replication alone does not automatically promote the standby to primary. You need Patroni or a similar tool for automatic promotion. For a manual approach, you can add a `notify_master` script that runs `pg_ctl promote`:

```bash
# Add to notify_master script:
if [ "$STATE" = "MASTER" ]; then
    sudo -u postgres /usr/lib/postgresql/15/bin/pg_ctl promote \
        -D /var/lib/postgresql/15/main
    log "PostgreSQL promoted to primary"
fi
```

## Common Mistakes to Avoid

1. **Using the same `virtual_router_id` on different clusters in the same network.** If two keepalived clusters share the same `virtual_router_id`, they will interfere with each other's VRRP advertisements. Use a unique ID per cluster (valid range: 1-255).

2. **Not configuring `authentication` in keepalived.** Without authentication, any VRRP-speaking device on the network can send advertisements and steal the VIP. Always use at least `PASS` authentication; `AH` (IPsec) is better for production.

3. **Health check script that checks connectivity to the other node instead of local health.** The health check should verify that the LOCAL PostgreSQL instance is healthy and capable of serving as primary. Checking the remote node introduces a dependency on the network, which may be the thing that is broken.

4. **Forgetting `hot_standby = on` on the standby.** Without this setting, the standby cannot accept read queries, which is useful for load balancing read traffic and for health checks that need to connect.

5. **Not setting `max_wal_senders` high enough.** If you have monitoring tools that connect as replication users (e.g., `pg_stat_replication` queries), they count against `max_wal_senders`. Set it to at least `number_of_standbys + 2` for monitoring headroom.

6. **VIP conflict with existing IPs.** Before assigning a VIP, verify that the IP is not already in use on the network. `arping -c 3 10.0.0.100` will tell you if any other host responds to that IP.

## Key Takeaway

Active-passive with keepalived is the simplest HA pattern, but it has a critical gap: keepalived manages the VIP, but it does not manage PostgreSQL promotion. Without a tool like Patroni to promote the standby, you still have a manual step in failover. The health check script is the most important component -- it must accurately determine whether the local node can serve as primary, and it must not be fooled by network issues that affect the check itself but not the node's ability to serve clients.
