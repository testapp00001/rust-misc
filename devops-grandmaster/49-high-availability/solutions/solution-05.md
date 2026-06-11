# Solution 05: HA Architecture for Zero-Downtime Operations

## Part A: Architecture Design with Rolling Upgrades

### Complete Architecture Diagram

```
                          +-----------------------+
                          |    External Clients    |
                          +-----------+-----------+
                                      |
                          +-----------+-----------+
                          |     DNS (Route53)     |
                          |   Weighted routing    |
                          |   Health-checked      |
                          +-----------+-----------+
                                      |
                    +-----------------+-----------------+
                    |                                   |
          +---------+---------+               +---------+---------+
          |  LB-VIP (keepalived)             |  LB-VIP (keepalived)|
          |  HAProxy-Primary  |               |  HAProxy-Standby  |
          |  10.0.0.100       |               |  10.0.0.101       |
          +--------+----------+               +--------+----------+
                   |  VRRP heartbeat                      |
                   +--------------------------------------+
                   |
     +-------------+------------------+------------------+
     |             |                  |                  |
+----+----+  +-----+---+  +---------+-+  +---------+    |
| App Srv |  | App Srv |  | App Srv  |  | App Srv |    |
|   #1    |  |   #2    |  |   #3     |  |   #4    |    |
+---------+  +---------+  +----------+  +---------+    |
     |             |                  |                  |
+----+----+  +-----+---+             |                  |
| App Srv |  | App Srv |             |                  |
|   #5    |  |   #6    |             |                  |
+---------+  +---------+             |                  |
                                      |
     +--------------------------------+------------------+
     |                                |                  |
     |           Consensus Layer (etcd cluster)         |
     |     +--------+    +--------+    +--------+       |
     |     | etcd-1 |    | etcd-2 |    | etcd-3 |       |
     |     | .0.10  |    | .0.11  |    | .0.12  |       |
     |     +--------+    +--------+    +--------+       |
     |                                                   |
     |           Database Layer (Patroni + PostgreSQL)   |
     |     +-----------+  +-----------+  +-----------+   |
     |     | PG-Node1  |  | PG-Node2  |  | PG-Node3  |   |
     |     | (Primary) |->| (Replica) |->| (Replica) |   |
     |     | + Patroni |  | + Patroni |  | + Patroni |   |
     |     +-----------+  +-----------+  +-----------+   |
     |                    streaming       streaming      |
     |                    replication     replication    |
     |                                                   |
     |           Cache Layer (Redis Sentinel)            |
     |     +-----------+  +-----------+  +-----------+   |
     |     | Redis-1   |  | Redis-2   |  | Redis-3   |   |
     |     | (Primary) |  | (Replica) |  | (Replica) |   |
     |     +-----------+  +-----------+  +-----------+   |
     |     | Sentinel-1|  | Sentinel-2|  | Sentinel-3|   |
     |     +-----------+  +-----------+  +-----------+   |
     |                                                   |
     |           Message Queue (RabbitMQ Quorum)         |
     |     +-----------+  +-----------+  +-----------+   |
     |     | Rabbit-1  |  | Rabbit-2  |  | Rabbit-3  |   |
     |     | (Quorum)  |  | (Quorum)  |  | (Quorum)  |   |
     |     +-----------+  +-----------+  +-----------+   |
     |                                                   |
     |           Monitoring Stack                        |
     |     +-----------+  +-----------+  +-----------+   |
     |     |Prometheus |  | AlertMgr  |  | Grafana   |   |
     |     +-----------+  +-----------+  +-----------+   |
     +---------------------------------------------------+
```

### Application Tier Rolling Upgrade Procedure

```bash
#!/bin/bash
# rolling-upgrade-app.sh -- Zero-downtime application rolling upgrade
# Minimum healthy servers: 4 out of 6 (67%)
# This allows taking 2 servers offline for upgrades while maintaining capacity

APP_SERVERS=("app-1" "app-2" "app-3" "app-4" "app-5" "app-6")
HAPROXY_STATS_URL="http://haproxy:8404/stats"
HEALTH_ENDPOINT="http://HOST:8080/health"
NEW_VERSION=$1
DRAIN_TIMEOUT=30  # seconds to wait for in-flight requests
HEALTH_CHECK_RETRIES=5
HEALTH_CHECK_INTERVAL=5

if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <new-version>"
    exit 1
fi

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

check_overall_health() {
    local healthy_count=0
    for server in "${APP_SERVERS[@]}"; do
        # Check via HAProxy stats API
        status=$(curl -s "${HAPROXY_STATS_URL};csv" | grep "$server" | awk -F',' '{print $18}')
        if [ "$status" = "UP" ]; then
            ((healthy_count++))
        fi
    done
    log "Healthy servers: ${healthy_count}/${#APP_SERVERS[@]}"
    if [ "$healthy_count" -lt 4 ]; then
        log "ERROR: Less than 4 servers healthy. Aborting upgrade."
        return 1
    fi
    return 0
}

drain_server() {
    local server=$1
    log "Draining connections on ${server}..."
    # Set server to DRAIN mode in HAProxy (no new connections, existing connections continue)
    curl -s -X POST "${HAPROXY_STATS_URL}" \
        -d "s=${server}&action=drain" \
        -H "Content-Type: application/x-www-form-urlencoded"
    # Wait for in-flight requests to complete
    log "Waiting ${DRAIN_TIMEOUT}s for in-flight requests to complete..."
    sleep "$DRAIN_TIMEOUT"
}

stop_server() {
    local server=$1
    log "Stopping application on ${server}..."
    ssh "$server" "sudo systemctl stop app"
}

upgrade_server() {
    local server=$1
    log "Deploying version ${NEW_VERSION} on ${server}..."
    ssh "$server" "
        sudo systemctl stop app 2>/dev/null || true
        sudo /usr/local/bin/deploy-app.sh ${NEW_VERSION}
        sudo systemctl start app
    "
}

health_check_server() {
    local server=$1
    local url="${HEALTH_ENDPOINT//HOST/$server}"
    for i in $(seq 1 "$HEALTH_CHECK_RETRIES"); do
        log "Health check ${i}/${HEALTH_CHECK_RETRIES} for ${server}..."
        if curl -sf "$url" > /dev/null 2>&1; then
            log "${server} is healthy"
            return 0
        fi
        sleep "$HEALTH_CHECK_INTERVAL"
    done
    log "ERROR: ${server} failed health checks after ${HEALTH_CHECK_RETRIES} retries"
    return 1
}

enable_server() {
    local server=$1
    log "Re-enabling ${server} in load balancer..."
    curl -s -X POST "${HAPROXY_STATS_URL}" \
        -d "s=${server}&action=ready" \
        -H "Content-Type: application/x-www-form-urlencoded"
}

rollback_server() {
    local server=$1
    local previous_version=$2
    log "ROLLING BACK ${server} to version ${previous_version}..."
    ssh "$server" "
        sudo systemctl stop app
        sudo /usr/local/bin/deploy-app.sh ${previous_version}
        sudo systemctl start app
    "
    enable_server "$server"
}

# Main rolling upgrade loop
PREVIOUS_VERSION=$(ssh "${APP_SERVERS[0]}" "cat /opt/app/VERSION")
log "Starting rolling upgrade from ${PREVIOUS_VERSION} to ${NEW_VERSION}"
log "Cluster size: ${#APP_SERVERS[@]} servers, minimum healthy: 4"

for server in "${APP_SERVERS[@]}"; do
    log "--- Upgrading ${server} ---"

    # Step 1: Check overall cluster health before starting
    if ! check_overall_health; then
        log "FATAL: Cluster health check failed. Aborting upgrade."
        exit 1
    fi

    # Step 2: Drain connections from this server
    drain_server "$server"

    # Step 3: Deploy new version
    upgrade_server "$server"

    # Step 4: Health check the upgraded server
    if ! health_check_server "$server"; then
        log "ERROR: ${server} failed after upgrade. Rolling back..."
        rollback_server "$server" "$PREVIOUS_VERSION"
        log "FATAL: Upgrade failed on ${server}. Aborting remaining upgrades."
        exit 1
    fi

    # Step 5: Re-enable in load balancer
    enable_server "$server"

    # Step 6: Brief pause to observe for issues
    log "Waiting 30s to observe for issues before continuing..."
    sleep 30

    # Step 7: Check overall health again
    if ! check_overall_health; then
        log "ERROR: Cluster degraded after ${server} upgrade. Rolling back ${server}..."
        rollback_server "$server" "$PREVIOUS_VERSION"
        log "FATAL: Aborting remaining upgrades."
        exit 1
    fi

    log "--- ${server} upgraded successfully ---"
done

log "Rolling upgrade to ${NEW_VERSION} complete. All ${#APP_SERVERS[@]} servers upgraded."
```

### PostgreSQL Rolling Upgrade Procedure (Minor Version)

```bash
#!/bin/bash
# rolling-upgrade-postgresql.sh -- Minor version upgrade for Patroni-managed cluster
# e.g., PostgreSQL 15.3 -> 15.4
# Process: upgrade replicas first, then switchover, then upgrade old primary

PATRONICTL="patronictl -c /etc/patroni/patroni.yml"
PG_VERSION_OLD="15.3"
PG_VERSION_NEW="15.4"
REPLICAS=("node2" "node3")
PRIMARY="node1"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

verify_cluster_healthy() {
    log "Verifying cluster health..."
    $PATRONICTL list
    local leader_count=$($PATRONICTL list | grep -c "Leader")
    if [ "$leader_count" -ne 1 ]; then
        log "ERROR: Expected 1 leader, found $leader_count"
        return 1
    fi
    # Check replication lag
    local max_lag=$($PATRONICTL list | awk 'NR>4 {print $8}' | sort -n | tail -1)
    if [ "${max_lag:-0}" -gt 1 ]; then
        log "WARNING: Replication lag is ${max_lag} MB. Waiting for it to settle..."
        sleep 30
    fi
    return 0
}

upgrade_replica() {
    local node=$1
    log "=== Upgrading replica: ${node} ==="

    # Step 1: Stop Patroni on the replica
    log "Stopping Patroni on ${node}..."
    ssh "$node" "sudo systemctl stop patroni"

    # Step 2: Install new PostgreSQL packages
    log "Installing PostgreSQL ${PG_VERSION_NEW} on ${node}..."
    ssh "$node" "
        sudo apt-get update
        sudo apt-get install -y postgresql-15=${PG_VERSION_NEW}-* \
            postgresql-client-15=${PG_VERSION_NEW}-*
    "

    # Step 3: Start Patroni (it will start PostgreSQL with the new binary)
    log "Starting Patroni on ${node}..."
    ssh "$node" "sudo systemctl start patroni"

    # Step 4: Wait for the replica to rejoin and catch up
    log "Waiting for ${node} to rejoin cluster and catch up on replication..."
    sleep 10
    for i in $(seq 1 12); do
        local status=$($PATRONICTL list | grep "$node" | awk '{print $6}')
        local lag=$($PATRONICTL list | grep "$node" | awk '{print $8}')
        if [ "$status" = "running" ] && [ "${lag:-0}" = "0" ]; then
            log "${node} is running with 0 lag"
            return 0
        fi
        log "Waiting... status=${status}, lag=${lag}MB (attempt ${i}/12)"
        sleep 10
    done

    log "ERROR: ${node} did not rejoin within timeout"
    return 1
}

# Step 0: Verify starting state
log "Starting PostgreSQL minor version upgrade: ${PG_VERSION_OLD} -> ${PG_VERSION_NEW}"
verify_cluster_healthy

# Step 1: Upgrade replicas one at a time
for replica in "${REPLICAS[@]}"; do
    upgrade_replica "$replica"
    if [ $? -ne 0 ]; then
        log "FATAL: Failed to upgrade ${replica}. Aborting."
        exit 1
    fi
    verify_cluster_healthy
done

# Step 2: Perform a controlled switchover
# Promote one of the already-upgraded replicas to primary
UPGRADED_REPLICA="${REPLICAS[0]}"
log "=== Performing switchover: ${PRIMARY} -> ${UPGRADED_REPLICA} ==="
$PATRONICTL switchover --master "$PRIMARY" --candidate "$UPGRADED_REPLICA" --force

sleep 10
verify_cluster_healthy

# Step 3: Upgrade the former primary (now a replica)
log "=== Upgrading former primary: ${PRIMARY} ==="
upgrade_replica "$PRIMARY"

# Step 4: Verify final state
log "=== Final cluster state ==="
$PATRONICTL list
verify_cluster_healthy

log "PostgreSQL minor version upgrade complete. All nodes running ${PG_VERSION_NEW}."
```

## Part B: Blue-Green Deployment for Database Schema Changes

### Safe NOT NULL Column Addition

```sql
-- ============================================================
-- Blue-Green Schema Migration: Adding a NOT NULL column
-- Database: PostgreSQL 15+
-- Table: orders (large table, ~100M rows)
-- ============================================================

-- Step 1: Add the column as nullable (instant, metadata-only)
-- This is safe because it does not rewrite the table.
-- Old application code continues to work (ignores the new column).
-- New application code can write to it but handles NULL gracefully.
ALTER TABLE orders ADD COLUMN IF NOT EXISTS shipping_address TEXT;

-- Step 2: Add a default value for new rows (also metadata-only in PG 11+)
ALTER TABLE orders ALTER COLUMN shipping_address SET DEFAULT '';

-- Step 3: Create an index concurrently if needed (does not lock the table)
-- (Not needed for this column, but shown for completeness)
-- CREATE INDEX CONCURRENTLY idx_orders_shipping_address ON orders(shipping_address);

-- Step 4: Backfill existing rows in batches
-- This is the critical step. Do NOT run a single UPDATE on a large table.
-- It will lock rows, consume WAL, and may cause replication lag.
DO $$
DECLARE
    batch_size INT := 5000;
    rows_updated INT := 1;
    max_id BIGINT;
    current_id BIGINT := 0;
    sleep_ms INT := 100;  -- Sleep between batches to reduce load
BEGIN
    -- Get the maximum order_id
    SELECT COALESCE(MAX(order_id), 0) INTO max_id FROM orders;

    RAISE NOTICE 'Starting backfill: max_id = %, batch_size = %', max_id, batch_size;

    WHILE current_id <= max_id LOOP
        -- Update a batch of rows
        UPDATE orders
        SET shipping_address = COALESCE(shipping_address, '')
        WHERE order_id > current_id
          AND order_id <= current_id + batch_size
          AND shipping_address IS NULL;

        GET DIAGNOSTICS rows_updated = ROW_COUNT;
        RAISE NOTICE 'Backfilled rows % to % (% rows affected)',
            current_id, current_id + batch_size, rows_updated;

        current_id := current_id + batch_size;

        -- Sleep between batches to reduce replication pressure
        PERFORM pg_sleep(sleep_ms / 1000.0);

        -- Commit happens at end of DO block, but we can use
        -- a separate approach with psql for explicit commits
    END LOOP;

    RAISE NOTICE 'Backfill complete';
END $$;

-- For production, use a script with explicit commits per batch:
-- (This prevents holding a transaction open for the entire backfill)

-- Step 5: Add a CHECK constraint as NOT VALID first (instant, no scan)
ALTER TABLE orders
    ADD CONSTRAINT chk_shipping_address_not_null
    CHECK (shipping_address IS NOT NULL) NOT VALID;

-- Step 6: Validate the constraint (scans the table but does not hold ACCESS EXCLUSIVE lock)
ALTER TABLE orders VALIDATE CONSTRAINT chk_shipping_address_not_null;

-- Step 7: Now add the NOT NULL constraint (instant because PG knows the CHECK is valid)
ALTER TABLE orders ALTER COLUMN shipping_address SET NOT NULL;

-- Step 8: Drop the CHECK constraint (it's redundant with NOT NULL)
ALTER TABLE orders DROP CONSTRAINT chk_shipping_address_not_null;

-- Verify:
\d orders
-- shipping_address | text | not null | default ''
```

### Application-Side Schema Version Handling

```python
import os
import logging
from enum import Enum
from typing import Optional, Any
import psycopg2
from psycopg2.extras import RealDictCursor

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class SchemaVersion(Enum):
    V1_LEGACY = "v1"      # No shipping_address column
    V2_TRANSITIONAL = "v2"  # shipping_address exists, nullable
    V3_FINAL = "v3"          # shipping_address is NOT NULL


class SchemaAwareQueryEngine:
    """
    Handles queries across different schema versions during a blue-green
    database migration. Supports running old and new application code
    simultaneously against the same database.
    """

    # Feature flag: controls which schema version the application uses
    # Set via environment variable or feature flag service
    SCHEMA_VERSION = SchemaVersion(
        os.environ.get("DB_SCHEMA_VERSION", "v2")
    )

    def __init__(self, dsn: str):
        self.dsn = dsn
        self._detected_version: Optional[SchemaVersion] = None

    def _get_connection(self):
        return psycopg2.connect(self.dsn)

    def detect_schema_version(self) -> SchemaVersion:
        """Detect the current schema version by checking the database."""
        if self._detected_version:
            return self._detected_version

        conn = self._get_connection()
        try:
            with conn.cursor() as cur:
                # Check if shipping_address column exists
                cur.execute("""
                    SELECT is_nullable
                    FROM information_schema.columns
                    WHERE table_name = 'orders'
                      AND column_name = 'shipping_address'
                """)
                result = cur.fetchone()

                if result is None:
                    self._detected_version = SchemaVersion.V1_LEGACY
                elif result[0] == "YES":
                    self._detected_version = SchemaVersion.V2_TRANSITIONAL
                else:
                    self._detected_version = SchemaVersion.V3_FINAL

                logger.info(
                    "Detected schema version: %s", self._detected_version.value
                )
                return self._detected_version
        finally:
            conn.close()

    def get_insert_columns(self) -> tuple[str, str]:
        """
        Return the column names and placeholders for INSERT statements.
        Adapts based on the configured schema version.
        """
        version = self.SCHEMA_VERSION

        if version == SchemaVersion.V1_LEGACY:
            columns = "tenant_id, amount, status"
            placeholders = "%s, %s, %s"
        elif version == SchemaVersion.V2_TRANSITIONAL:
            # Transitional: write to shipping_address if available, but handle NULL
            columns = "tenant_id, amount, status, shipping_address"
            placeholders = "%s, %s, %s, %s"
        else:  # V3_FINAL
            columns = "tenant_id, amount, status, shipping_address"
            placeholders = "%s, %s, %s, %s"

        return columns, placeholders

    def get_select_columns(self) -> str:
        """Return the column list for SELECT statements."""
        version = self.detect_schema_version()

        base_columns = "order_id, tenant_id, amount, status, created_at"

        if version == SchemaVersion.V1_LEGACY:
            return base_columns
        else:
            return f"{base_columns}, shipping_address"

    def insert_order(
        self,
        tenant_id: int,
        amount: float,
        status: str,
        shipping_address: Optional[str] = None,
    ) -> int:
        """Insert an order, adapting to the current schema version."""
        version = self.SCHEMA_VERSION
        conn = self._get_connection()
        try:
            with conn.cursor() as cur:
                if version == SchemaVersion.V1_LEGACY:
                    cur.execute(
                        "INSERT INTO orders (tenant_id, amount, status) "
                        "VALUES (%s, %s, %s) RETURNING order_id",
                        (tenant_id, amount, status),
                    )
                else:
                    # V2 and V3 both have shipping_address
                    addr = shipping_address or ""
                    cur.execute(
                        "INSERT INTO orders (tenant_id, amount, status, shipping_address) "
                        "VALUES (%s, %s, %s, %s) RETURNING order_id",
                        (tenant_id, amount, status, addr),
                    )
                order_id = cur.fetchone()[0]
                conn.commit()
                return order_id
        except Exception as e:
            conn.rollback()
            logger.error("Failed to insert order: %s", e)
            raise
        finally:
            conn.close()

    def get_order(self, order_id: int) -> Optional[dict[str, Any]]:
        """Retrieve an order, adapting SELECT to available columns."""
        columns = self.get_select_columns()
        conn = self._get_connection()
        try:
            with conn.cursor(cursor_factory=RealDictCursor) as cur:
                cur.execute(
                    f"SELECT {columns} FROM orders WHERE order_id = %s",
                    (order_id,),
                )
                row = cur.fetchone()
                if row:
                    result = dict(row)
                    # Normalize: ensure shipping_address key always exists
                    # for application code that expects it
                    if "shipping_address" not in result:
                        result["shipping_address"] = None
                    return result
                return None
        finally:
            conn.close()

    def get_orders_by_tenant(
        self, tenant_id: int, limit: int = 100
    ) -> list[dict[str, Any]]:
        """Retrieve orders for a tenant."""
        columns = self.get_select_columns()
        conn = self._get_connection()
        try:
            with conn.cursor(cursor_factory=RealDictCursor) as cur:
                cur.execute(
                    f"SELECT {columns} FROM orders "
                    "WHERE tenant_id = %s ORDER BY created_at DESC LIMIT %s",
                    (tenant_id, limit),
                )
                results = []
                for row in cur.fetchall():
                    result = dict(row)
                    if "shipping_address" not in result:
                        result["shipping_address"] = None
                    results.append(result)
                return results
        finally:
            conn.close()


# --- Feature Flag Integration (for more sophisticated control) ---

class FeatureFlagClient:
    """
    In production, integrate with a feature flag service (LaunchDarkly,
    Unleash, Flagsmith, etc.) to control schema version rollout.
    """

    def __init__(self, service_url: str, api_key: str):
        self.service_url = service_url
        self.api_key = api_key

    def get_flag(self, flag_name: str, default: str = "v2") -> str:
        """
        Get a feature flag value. In production, this would call the
        feature flag service API with caching and fallback.
        """
        # Implementation: HTTP GET to feature flag service
        # with exponential backoff and local cache
        return os.environ.get(f"FF_{flag_name.upper()}", default)


# --- Usage Example ---

engine = SchemaAwareQueryEngine(
    dsn="host=10.0.0.100 dbname=orders_db user=app password=secret"
)

# Auto-detect schema version
detected = engine.detect_schema_version()
logger.info("Operating against schema: %s", detected.value)

# Insert (adapts to schema version)
order_id = engine.insert_order(
    tenant_id=1001,
    amount=149.99,
    status="confirmed",
    shipping_address="123 Main St, Springfield, IL 62701",
)
logger.info("Created order %d", order_id)

# Read (adapts to available columns, normalizes output)
order = engine.get_order(order_id)
logger.info("Order: %s", order)
# Both old and new application code can access order["shipping_address"]
# Old code: returns None if column doesn't exist
# New code: returns the actual value
```

## Part C: Connection Draining During Failover

### HAProxy Configuration

```
# /etc/haproxy/haproxy.cfg

global
    maxconn 10000
    log /dev/log local0
    stats socket /var/run/haproxy/admin.sock mode 660 level admin
    stats timeout 30s

defaults
    log     global
    mode    tcp
    option  tcplog
    option  dontlognull
    timeout connect 5s
    timeout client  300s
    timeout server  300s
    default-server inter 3s fall 3 rise 2

# Stats page for monitoring
listen stats
    bind *:8404
    mode http
    stats enable
    stats uri /stats
    stats refresh 10s
    stats admin if TRUE

# PostgreSQL write traffic (primary only)
listen pg_write
    bind *:5432
    mode tcp

    # Health check: use Patroni REST API to verify this node is the leader
    option httpchk GET /primary
    http-check expect status 200

    # Server definitions with health check port (Patroni REST API)
    server pg_node1 10.0.0.10:5432 check port 8008 inter 2s fall 3 rise 2
    server pg_node2 10.0.0.11:5432 check port 8008 inter 2s fall 3 rise 2
    server pg_node3 10.0.0.12:5432 check port 8008 inter 2s fall 3 rise 2

# PostgreSQL read traffic (replicas preferred, primary as fallback)
listen pg_read
    bind *:5433
    mode tcp
    balance leastconn

    # Health check: use Patroni REST API to verify this node is a replica
    option httpchk GET /replica
    http-check expect status 200

    # Replicas first (lower weight)
    server pg_node1 10.0.0.10:5432 check port 8008 inter 2s fall 3 rise 2 weight 50
    server pg_node2 10.0.0.11:5432 check port 8008 inter 2s fall 3 rise 2 weight 100
    server pg_node3 10.0.0.12:5432 check port 8008 inter 2s fall 3 rise 2 weight 100

# Connection draining configuration
# To drain a server manually via the stats socket:
#   echo "set server pg_write/pg_node1 state drain" | socat stdio /var/run/haproxy/admin.sock
#
# To re-enable after drain:
#   echo "set server pg_write/pg_node1 state ready" | socat stdio /var/run/haproxy/admin.sock
#
# To set a server to maintenance (immediate disconnect):
#   echo "set server pg_write/pg_node1 state maint" | socat stdio /var/run/haproxy/admin.sock
```

### Application-Level Connection Pool Manager

```python
import time
import threading
import logging
import json
from typing import Optional
from contextlib import contextmanager
from enum import Enum

import psycopg2
from psycopg2 import pool
import requests

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class PoolState(Enum):
    ACTIVE = "active"
    DRAINING = "draining"
    DRAINED = "drained"
    RECONNECTING = "reconnecting"


class ConnectionPoolManager:
    """
    Manages a PostgreSQL connection pool with support for:
    - Graceful connection draining during failover
    - Automatic primary detection via Patroni REST API
    - Health checking and automatic reconnection
    """

    def __init__(
        self,
        patroni_nodes: list[str],
        dbname: str = "postgres",
        user: str = "app",
        password: str = "secret",
        min_conn: int = 5,
        max_conn: int = 20,
        drain_timeout: int = 30,
        health_check_interval: int = 5,
    ):
        self.patroni_nodes = patroni_nodes
        self.dbname = dbname
        self.user = user
        self.password = password
        self.min_conn = min_conn
        self.max_conn = max_conn
        self.drain_timeout = drain_timeout
        self.health_check_interval = health_check_interval

        self._pool: Optional[pool.ThreadedConnectionPool] = None
        self._state = PoolState.ACTIVE
        self._current_primary: Optional[str] = None
        self._active_connections: int = 0
        self._lock = threading.Lock()
        self._drain_event = threading.Event()
        self._drain_event.set()  # Set = not draining (connections allowed)

        self._health_thread = threading.Thread(
            target=self._health_check_loop, daemon=True
        )
        self._discover_and_connect()
        self._health_thread.start()

    def _discover_primary(self) -> Optional[str]:
        """Query Patroni REST API to find the current primary node."""
        for node in self.patroni_nodes:
            try:
                resp = requests.get(
                    f"http://{node}:8008/cluster", timeout=2
                )
                if resp.status_code == 200:
                    cluster = resp.json()
                    for member in cluster.get("members", []):
                        if member.get("role") == "Leader":
                            return member.get("host")
            except requests.RequestException:
                continue
        return None

    def _create_pool(self, host: str) -> pool.ThreadedConnectionPool:
        """Create a new connection pool to the given host."""
        return pool.ThreadedConnectionPool(
            self.min_conn,
            self.max_conn,
            host=host,
            dbname=self.dbname,
            user=self.user,
            password=self.password,
            connect_timeout=5,
        )

    def _discover_and_connect(self):
        """Discover the primary and create a connection pool."""
        primary = self._discover_primary()
        if primary is None:
            logger.error("Cannot discover primary node from Patroni")
            return

        if primary == self._current_primary and self._pool is not None:
            return  # No change

        logger.info("Discovered primary: %s", primary)
        self._current_primary = primary

        if self._pool is not None:
            self._close_pool()

        self._pool = self._create_pool(primary)
        logger.info("Connection pool established to %s", primary)

    def _close_pool(self):
        """Close all connections in the pool."""
        if self._pool:
            try:
                self._pool.closeall()
            except Exception as e:
                logger.error("Error closing pool: %s", e)
            self._pool = None

    def _health_check_loop(self):
        """Background thread that monitors pool health."""
        while True:
            try:
                self._check_health()
            except Exception as e:
                logger.error("Health check error: %s", e)
            time.sleep(self.health_check_interval)

    def _check_health(self):
        """Check if the current connection is still to the primary."""
        if self._state == PoolState.DRAINING:
            return  # Don't change state during drain

        primary = self._discover_primary()
        if primary != self._current_primary:
            logger.warning(
                "Primary changed from %s to %s. Reconnecting...",
                self._current_primary,
                primary,
            )
            self._state = PoolState.RECONNECTING
            self._drain_event.clear()  # Block new connections
            self._wait_for_in_flight()
            self._close_pool()
            self._current_primary = primary
            self._pool = self._create_pool(primary)
            self._state = PoolState.ACTIVE
            self._drain_event.set()  # Allow new connections
            logger.info("Reconnected to new primary: %s", primary)

    def _wait_for_in_flight(self):
        """Wait for in-flight connections to be returned."""
        deadline = time.time() + self.drain_timeout
        while self._active_connections > 0 and time.time() < deadline:
            logger.info(
                "Draining: %d active connections remaining...",
                self._active_connections,
            )
            time.sleep(1)

        if self._active_connections > 0:
            logger.warning(
                "Drain timeout reached with %d connections still active. "
                "Forcing close.",
                self._active_connections,
            )

    @contextmanager
    def get_connection(self):
        """
        Get a connection from the pool.

        Raises ConnectionError if the pool is draining or unavailable.
        """
        # Wait if we are draining
        if not self._drain_event.wait(timeout=10):
            raise ConnectionError(
                "Connection pool is draining or reconnecting"
            )

        if self._pool is None:
            raise ConnectionError("No connection pool available")

        conn = None
        with self._lock:
            self._active_connections += 1
        try:
            conn = self._pool.getconn()
            yield conn
        except Exception as e:
            logger.error("Connection error: %s", e)
            raise
        finally:
            if conn:
                try:
                    self._pool.putconn(conn)
                except Exception:
                    pass
            with self._lock:
                self._active_connections -= 1

    def drain(self):
        """
        Gracefully drain all connections.
        Stops accepting new connections and waits for in-flight to complete.
        """
        logger.info("Starting connection drain...")
        self._state = PoolState.DRAINING
        self._drain_event.clear()
        self._wait_for_in_flight()
        self._state = PoolState.DRAINED
        logger.info("Connection pool drained")

    def activate(self):
        """Re-activate the pool after draining."""
        if self._state == PoolState.DRAINED:
            self._state = PoolState.ACTIVE
            self._drain_event.set()
            logger.info("Connection pool activated")

    @property
    def state(self) -> PoolState:
        return self._state

    @property
    def active_connections(self) -> int:
        return self._active_connections

    def health_check(self) -> dict:
        """Return health status for monitoring."""
        primary = self._discover_primary()
        return {
            "state": self._state.value,
            "current_primary": self._current_primary,
            "discovered_primary": primary,
            "primary_in_sync": self._current_primary == primary,
            "active_connections": self._active_connections,
            "pool_available": self._pool is not None,
        }


# --- Usage Example ---

pool_manager = ConnectionPoolManager(
    patroni_nodes=["10.0.0.10", "10.0.0.11", "10.0.0.12"],
    dbname="orders_db",
    user="app",
    password="secret",
    max_conn=20,
    drain_timeout=30,
)

# Normal usage
with pool_manager.get_connection() as conn:
    with conn.cursor() as cur:
        cur.execute("SELECT * FROM orders WHERE tenant_id = %s", (1001,))
        results = cur.fetchall()

# Simulate planned failover (e.g., during maintenance)
print("Before drain:", pool_manager.health_check())
pool_manager.drain()
print("After drain:", pool_manager.health_check())
# ... perform maintenance on the primary ...
pool_manager.activate()
print("After activate:", pool_manager.health_check())
```

## Part D: Monitoring and Alerting for HA Components

### Monitoring Matrix (Completed)

| Component | Metric | Warning Threshold | Critical Threshold | Action |
|-----------|--------|-------------------|--------------------|--------|
| PostgreSQL Primary | Replication lag (bytes) | > 16 MB | > 64 MB | Check network, reduce write load, verify replica health |
| PostgreSQL Primary | Connection count | > 80% max_connections | > 95% max_connections | Kill idle connections, increase max_connections, add read replicas |
| PostgreSQL Standby | Replication lag (seconds) | > 5 seconds | > 30 seconds | Check replica I/O, network latency, WAL generation rate |
| etcd Cluster | Leader elections per hour | > 2 | > 10 | Check network stability, disk latency, etcd member health |
| etcd Cluster | Proposal commit latency (p99) | > 200ms | > 500ms | Check disk I/O, network, CPU pressure on etcd nodes |
| Patroni | Failover count per day | > 1 | > 3 | Investigate root cause (network, disk, OOM), review Patroni logs |
| HAProxy | Backend server health | Any backend DOWN | Multiple backends DOWN | Check Patroni/PG status, network connectivity |
| Redis Sentinel | Sentinel reachable count | < 3 (of 3) | < 2 (of 3) | Check Sentinel process, network, Redis node health |
| Application | Error rate (5xx) | > 0.1% | > 1% | Check application logs, database connectivity, dependency health |
| Application | Request latency (p99) | > 500ms | > 2000ms | Check database query time, connection pool saturation, CPU/memory |

### Prometheus Alert Rules

```yaml
# /etc/prometheus/rules/ha_alerts.yml

groups:
  - name: postgresql_ha
    rules:
      # Alert 1: PostgreSQL replication lag
      - alert: PostgreSQLReplicationLagHigh
        expr: |
          pg_replication_lag_bytes > 16777216
        for: 2m
        labels:
          severity: warning
          team: database
        annotations:
          summary: "PostgreSQL replication lag is {{ $value | humanize }} on {{ $labels.instance }}"
          description: >
            Replication lag on {{ $labels.instance }} has exceeded 16MB for more
            than 2 minutes. This may indicate network issues, high write load,
            or replica resource constraints.
          runbook: "https://wiki.internal/runbooks/pg-replication-lag"
          dashboard: "https://grafana.internal/d/postgresql-replication"

      - alert: PostgreSQLReplicationLagCritical
        expr: |
          pg_replication_lag_bytes > 67108864
        for: 1m
        labels:
          severity: critical
          team: database
          pager: "true"
        annotations:
          summary: "CRITICAL: PostgreSQL replication lag is {{ $value | humanize }} on {{ $labels.instance }}"
          description: >
            Replication lag has exceeded 64MB. If this replica is the failover
            target, failover will result in significant data loss. Immediate
            investigation required.
          runbook: "https://wiki.internal/runbooks/pg-replication-lag-critical"

  - name: etcd_ha
    rules:
      # Alert 2: etcd cluster has no leader (quorum lost)
      - alert: EtcdClusterNoLeader
        expr: |
          etcd_server_has_leader == 0
        for: 10s
        labels:
          severity: critical
          team: infrastructure
          pager: "true"
        annotations:
          summary: "etcd cluster has no leader on {{ $labels.instance }}"
          description: >
            The etcd node {{ $labels.instance }} reports no leader. This means
            the Raft consensus has failed, likely due to a network partition
            or multiple node failures. Patroni cannot perform leader election
            until etcd quorum is restored.
          runbook: "https://wiki.internal/runbooks/etcd-no-leader"

      - alert: EtcdProposalCommitLatencyHigh
        expr: |
          histogram_quantile(0.99, rate(etcd_disk_wal_fsync_duration_seconds_bucket[5m])) > 0.2
        for: 5m
        labels:
          severity: warning
          team: infrastructure
        annotations:
          summary: "etcd proposal commit latency p99 is {{ $value }}s on {{ $labels.instance }}"
          description: >
            High etcd commit latency indicates disk I/O pressure. etcd is
            sensitive to disk latency because every Raft proposal must be
            fsynced to disk before being committed.
          runbook: "https://wiki.internal/runbooks/etcd-latency"

  - name: patroni_ha
    rules:
      # Alert 3: Patroni failover detected
      - alert: PatroniFailoverDetected
        expr: |
          changes(patroni_master_timeline[1m]) > 0
        for: 0s
        labels:
          severity: warning
          team: database
        annotations:
          summary: "Patroni failover detected on {{ $labels.instance }}"
          description: >
            A Patroni failover has occurred. The PostgreSQL primary has changed.
            This may be planned (switchover) or unplanned (crash, network issue).
            Investigate immediately to determine root cause and verify data integrity.
          runbook: "https://wiki.internal/runbooks/patroni-failover"

      - alert: PatroniFailoverFrequent
        expr: |
          count_over_time(patroni_master_timeline[1h]) > 3
        for: 0s
        labels:
          severity: critical
          team: database
          pager: "true"
        annotations:
          summary: "Patroni has failed over {{ $value }} times in the last hour"
          description: >
            Frequent failovers indicate an unstable cluster. Possible causes:
            flapping network, etcd instability, resource exhaustion on primary,
            or overly aggressive TTL settings. Each failover risks brief
            unavailability and should be investigated.

      # Alert 4: Patroni node not running
      - alert: PatroniNodeDown
        expr: |
          patroni_postgres_running == 0
        for: 30s
        labels:
          severity: critical
          team: database
          pager: "true"
        annotations:
          summary: "Patroni reports PostgreSQL is not running on {{ $labels.instance }}"
          description: >
            The Patroni-managed PostgreSQL instance on {{ $labels.instance }}
            is not running. If this is the primary, a failover should occur
            automatically. If this is a replica, replication has stopped.
```

### Runbook for Patroni Failover Event

```bash
# ============================================================
# RUNBOOK: Patroni Failover Event
# Alert: PatroniFailoverDetected
# Severity: Warning (investigate) / Critical (if frequent)
# ============================================================

# SECTION 1: Alert Description and Impact
# ========================================
# A Patroni failover means the PostgreSQL primary has changed.
# Impact:
#   - Brief period (1-30s) where writes may fail
#   - Existing connections to the old primary are severed
#   - Applications may see transient errors
#   - If synchronous_mode is off, committed transactions may be lost
#
# This runbook covers both planned switchovers and unplanned failovers.

# SECTION 2: First 5 Minutes (Immediate Triage)
# ==============================================

# 2.1: Check current cluster state
patronictl -c /etc/patroni/patroni.yml list
# Look for: Who is the leader? Are all members running? Any lag?

# 2.2: Check if applications are healthy
curl -s http://app-server:8080/health | python3 -m json.tool
# Look for: database connectivity status, error rates

# 2.3: Check the failover reason (Patroni logs)
sudo journalctl -u patroni --since "10 minutes ago" | grep -i "failover\|leader\|promoted\|demoted"

# 2.4: Check if the old primary is still running
ssh old-primary "sudo systemctl status patroni && sudo systemctl status postgresql"

# 2.5: Verify data integrity (if synchronous_mode was enabled)
psql -h NEW_PRIMARY -U postgres -c "SELECT count(*) FROM pg_stat_replication;"
# Confirm replicas are connected to the new primary

# SECTION 3: Diagnosis (Determine Root Cause)
# ===========================================

# 3.1: Check etcd health (Patroni depends on etcd for leader election)
etcdctl endpoint health --cluster
etcdctl endpoint status --cluster -w table

# 3.2: Check for network issues
# On the old primary:
ping -c 5 etcd-node-1
ping -c 5 etcd-node-2
ping -c 5 etcd-node-3

# 3.3: Check for resource exhaustion on the old primary
ssh old-primary "
    echo '=== Disk ===' && df -h /var/lib/postgresql
    echo '=== Memory ===' && free -h
    echo '=== CPU ===' && uptime
    echo '=== OOM kills ===' && dmesg | grep -i 'oom\|killed' | tail -5
    echo '=== PostgreSQL logs ===' && tail -50 /var/log/postgresql/postgresql-15-main.log
"

# 3.4: Check HAProxy backend status
echo "show servers state" | socat stdio /var/run/haproxy/admin.sock

# 3.5: Determine the cause category
# Category A: Old primary crashed (OOM, disk full, kernel panic)
#   -> Patroni detected it via TTL expiry in etcd
#   -> Action: Fix the root cause on the old primary, let it rejoin as replica
#
# Category B: Network partition (old primary isolated from etcd)
#   -> Old primary could not renew its leader lock
#   -> Action: Fix the network, verify old primary demoted itself
#
# Category C: Planned switchover (operator-initiated)
#   -> No action needed, this is expected
#   -> Verify: new primary is healthy, old primary is replicating
#
# Category D: etcd instability
#   -> etcd lost quorum, triggering leader lock expiry
#   -> Action: Fix etcd (disk, network, CPU), Patroni will stabilize

# SECTION 4: Resolution
# =====================

# 4.1: If the old primary is still running but demoted
# Verify it is now a replica:
psql -h OLD_PRIMARY -U postgres -c "SELECT pg_is_in_recovery();"
# Expected: true

# 4.2: If the old primary is down, bring it back as a replica
ssh old-primary "
    sudo systemctl start patroni
"
# Patroni will automatically detect it is no longer the leader
# and start replicating from the new primary

# 4.3: If the old primary has diverged (split-brain scenario)
# This should not happen with Patroni, but if it does:
ssh old-primary "
    sudo systemctl stop patroni
    sudo systemctl stop postgresql
    sudo -u postgres pg_rewind --target-pgdata=/var/lib/postgresql/15/main \
        --source-server='host=NEW_PRIMARY port=5432 user=postgres'
    sudo systemctl start patroni
"

# 4.4: If you need to force a specific node to be primary
patronictl -c /etc/patroni/patroni.yml switchover --master CURRENT_PRIMARY --candidate DESIRED_PRIMARY --force

# SECTION 5: Verification
# =======================

# 5.1: Verify cluster is healthy
patronictl -c /etc/patroni/patroni.yml list
# All members should be "running", exactly 1 Leader, replicas with 0 lag

# 5.2: Verify replication is working
psql -h NEW_PRIMARY -U postgres -c "
    SELECT client_addr, state, sent_lsn, write_lsn, flush_lsn, replay_lsn
    FROM pg_stat_replication;
"
# All replicas should show "streaming" with recent LSN

# 5.3: Verify applications are functioning
curl -s http://app-server:8080/health | python3 -m json.tool
# Check error rate in monitoring dashboards

# 5.4: Verify etcd is healthy
etcdctl endpoint health --cluster

# 5.5: Verify HAProxy has updated
echo "show servers state" | socat stdio /var/run/haproxy/admin.sock
# The new primary should be "UP" in the pg_write backend

# SECTION 6: Post-Incident Documentation
# =======================================

# Document in the incident tracker:
# 1. Timeline: When did the failover occur? When was it detected?
# 2. Root cause: What triggered the failover?
# 3. Impact: How many requests failed? For how long?
# 4. Resolution: What fixed the issue?
# 5. Action items: What needs to change to prevent recurrence?
#    - Adjust TTL settings?
#    - Add monitoring for the root cause?
#    - Improve runbook?
#    - Add capacity?
```

## Common Mistakes to Avoid

1. **Upgrading all replicas simultaneously.** If you upgrade all replicas at once and the upgrade introduces a data format incompatibility, you lose all replication targets. Always upgrade one replica at a time, verify it catches up, then proceed to the next.

2. **Not draining connections before stopping a server.** Killing a server with active connections causes those transactions to fail and the application to see errors. Even a 10-second drain period significantly reduces user-visible errors. The `DRAIN` state in HAProxy is your friend.

3. **Using `ALTER TABLE ADD COLUMN ... NOT NULL DEFAULT` on large tables in pre-PG11.** In PostgreSQL 10 and earlier, this rewrites the entire table, holding an ACCESS EXCLUSIVE lock for the duration. In PG 11+, the default is stored in metadata and the operation is instant. Always check your PostgreSQL version before running DDL on large tables.

4. **Backfilling in a single transaction.** A single `UPDATE orders SET shipping_address = '' WHERE shipping_address IS NULL` on a 100M-row table will generate 100M WAL entries, hold row locks for the entire duration, and likely cause replication lag that triggers a failover. Batch the backfill and commit between batches.

5. **Monitoring only the happy path.** Monitoring that only tracks "is the database up?" misses the failures that precede an outage. Monitor replication lag, disk usage, connection pool saturation, etcd leader elections, and Patroni failover events. The goal is to detect problems before they cause a failover, not after.

6. **Not testing the full failure chain.** Testing "kill the primary and verify failover" is necessary but not sufficient. Test: kill the primary during a schema migration, kill the primary during a backup, kill the primary while a replica is being rebuilt, kill the primary and the etcd leader simultaneously. The edge cases are where HA architectures break.

## Key Takeaway

Zero-downtime operations require every layer of the stack to support graceful transitions. Rolling upgrades work because you never take down enough nodes to lose quorum. Blue-green schema migrations work because the application handles both old and new schemas during the transition window. Connection draining works because you stop new traffic before killing old connections. And the whole system is held together by monitoring that detects problems before users notice. The architecture is only as strong as its weakest link -- a single component that does not support graceful shutdown will break your zero-downtime promise. Design for failure at every layer, test the failure combinations, and monitor the transitions.
