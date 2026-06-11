# Solution 03: Active-Active Configuration

## Part A: Bi-Directional Logical Replication

### Why Logical Replication (Not Streaming Replication)

Streaming replication ships WAL bytes and creates an exact physical copy. This means both nodes have the same timeline, which prevents them from accepting independent writes. Logical replication replicates the logical changes (INSERT, UPDATE, DELETE) and allows each node to have its own write timeline.

### Setup: us-east Publishes, eu-west Subscribes

```sql
-- Run on us-east (10.1.0.10)

-- Create the replication user
CREATE USER replicator WITH REPLICATION PASSWORD 'Str0ng_BDR_P4ss';

-- Create the orders table (if not exists)
CREATE TABLE IF NOT EXISTS orders (
    order_id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    amount NUMERIC(12, 2) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    origin_region VARCHAR(10) NOT NULL DEFAULT 'us-east'
);

-- Create a publication for the orders table
CREATE PUBLICATION us_east_pub FOR TABLE orders;

-- Grant necessary permissions
GRANT SELECT ON orders TO replicator;
```

```sql
-- Run on eu-west (10.2.0.10)

-- Create the same table structure
CREATE TABLE IF NOT EXISTS orders (
    order_id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    amount NUMERIC(12, 2) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    origin_region VARCHAR(10) NOT NULL DEFAULT 'eu-west'
);

-- Create the subscription to us-east's publication
CREATE SUBSCRIPTION eu_west_sub
    CONNECTION 'host=10.1.0.10 dbname=postgres user=replicator password=Str0ng_BDR_P4ss'
    PUBLICATION us_east_pub
    WITH (
        copy_data = true,
        origin = NONE  -- CRITICAL: prevents replication loops
    );
```

### Setup: eu-west Publishes, us-east Subscribes

```sql
-- Run on eu-west (10.2.0.10)

CREATE PUBLICATION eu_west_pub FOR TABLE orders;
GRANT SELECT ON orders TO replicator;
```

```sql
-- Run on us-east (10.1.0.10)

CREATE SUBSCRIPTION us_east_sub
    CONNECTION 'host=10.2.0.10 dbname=postgres user=replicator password=Str0ng_BDR_P4ss'
    PUBLICATION eu_west_pub
    WITH (
        copy_data = false,  -- Data already exists from the first subscription
        origin = NONE       -- CRITICAL: prevents replication loops
    );
```

### Verify Replication

```sql
-- On us-east: Check subscription status
SELECT subname, subenabled, subconninfo,
       received_lsn, last_msg_send_time, last_msg_receipt_time
FROM pg_stat_subscription;

-- On eu-west: Check subscription status
SELECT subname, subenabled, subconninfo,
       received_lsn, last_msg_send_time, last_msg_receipt_time
FROM pg_stat_subscription;

-- Test: Insert on us-east, verify it appears on eu-west
-- On us-east:
INSERT INTO orders (tenant_id, amount, status, origin_region)
VALUES (1001, 99.99, 'confirmed', 'us-east');

-- On eu-west (after replication delay):
SELECT * FROM orders WHERE tenant_id = 1001;
-- Should show the row with origin_region = 'us-east'
```

### The `origin = NONE` Setting Explained

When `origin = NONE`, changes that arrive via the subscription are tagged as having no replication origin. When the other subscription tries to replicate these changes, it sees they have no origin and skips them. This prevents infinite replication loops:

```
us-east INSERT -> eu-west receives via subscription -> eu-west tries to replicate back
                                                   -> origin = NONE, skip (no loop)
```

## Part B: Conflict-Avoidance Partitioning Strategy

### Completed Design Table

| Aspect | Design Decision | Rationale |
|--------|-----------------|-----------|
| Data ownership model | Each tenant is assigned a home region based on where they signed up or their primary business location | Tenants are the natural partition boundary because their data is self-contained; cross-tenant queries are rare |
| Tenant routing rule | `tenant_id` modulo mapping with a lookup table: tenants 1-10000 in us-east, 10001-20000 in eu-west, with a `tenant_region_map` table for exceptions | Simple modulo is fast but rigid; a lookup table allows overrides for VIP tenants, legal requirements, or migrations |
| What happens if a US tenant is accessed from EU? | Writes are proxied to the tenant's home region via the application router; reads can be served locally for eventual consistency or proxied for strong consistency | Proxied writes avoid conflicts entirely; the latency cost (80-150ms cross-Atlantic) is acceptable for writes which are less frequent than reads |
| How do you handle tenant migration between regions? | 1. Set the tenant to "migrating" state 2. Pause writes for that tenant 3. Wait for replication to catch up 4. Update the region map 5. Resume writes in the new region | A brief write pause (typically <1 second) during migration prevents the split-brain that would occur if writes were accepted in both regions simultaneously during the transition |
| What about shared/global data (no tenant_id)? | Global data (product catalog, system config) uses a single-writer model: one region is designated as the source of truth, and the other region receives it via replication | Global data cannot be partitioned by tenant; single-writer avoids all conflicts at the cost of higher latency for cross-region writes |

### Tenant Region Map Table

```sql
CREATE TABLE tenant_region_map (
    tenant_id BIGINT PRIMARY KEY,
    home_region VARCHAR(10) NOT NULL CHECK (home_region IN ('us-east', 'eu-west')),
    migrated_at TIMESTAMPTZ,
    migration_state VARCHAR(20) DEFAULT 'active' CHECK (migration_state IN ('active', 'migrating', 'frozen'))
);

-- Index for fast lookups
CREATE INDEX idx_tenant_region_map_region ON tenant_region_map(home_region);

-- Example data
INSERT INTO tenant_region_map (tenant_id, home_region) VALUES
    (1001, 'us-east'),
    (20001, 'eu-west');
```

## Part C: Conflict Resolution

### Add Updated_at and Origin Tracking

```sql
-- Add origin tracking column (if not already present)
ALTER TABLE orders ADD COLUMN IF NOT EXISTS origin_region VARCHAR(10);

-- Add a composite timestamp for conflict resolution
-- (updated_at alone is insufficient because clock skew between regions can cause issues)
ALTER TABLE orders ADD COLUMN IF NOT EXISTS conflict_token BIGINT DEFAULT 0;
```

### Last-Writer-Wins Trigger

```sql
-- Create the conflict audit table
CREATE TABLE IF NOT EXISTS conflicts_audit (
    audit_id BIGSERIAL PRIMARY KEY,
    table_name VARCHAR(100) NOT NULL,
    record_id BIGINT NOT NULL,
    conflict_type VARCHAR(50) NOT NULL,
    local_updated_at TIMESTAMPTZ,
    remote_updated_at TIMESTAMPTZ,
    local_origin VARCHAR(10),
    remote_origin VARCHAR(10),
    resolution VARCHAR(50) NOT NULL,
    resolved_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Function to handle conflict resolution
CREATE OR REPLACE FUNCTION resolve_order_conflict()
RETURNS TRIGGER AS $$
DECLARE
    existing_record RECORD;
    local_ts TIMESTAMPTZ;
    remote_ts TIMESTAMPTZ;
BEGIN
    -- Only apply conflict resolution for replicated changes
    -- (local writes should always succeed)
    IF current_setting('session_replication_role', true) = 'replica' THEN
        -- Get the existing row
        SELECT * INTO existing_record
        FROM orders
        WHERE order_id = NEW.order_id;

        IF FOUND THEN
            local_ts := existing_record.updated_at;
            remote_ts := NEW.updated_at;

            -- Last-writer-wins: only apply if the remote change is newer
            IF remote_ts > local_ts THEN
                -- Log the conflict that was resolved by overwrite
                INSERT INTO conflicts_audit
                    (table_name, record_id, conflict_type,
                     local_updated_at, remote_updated_at,
                     local_origin, remote_origin, resolution)
                VALUES
                    ('orders', NEW.order_id, 'update_conflict',
                     local_ts, remote_ts,
                     existing_record.origin_region, NEW.origin_region,
                     'remote_wins_last_writer_wins');

                -- Allow the NEW row to proceed (overwrite local)
                RETURN NEW;
            ELSIF remote_ts = local_ts THEN
                -- Tie-breaker: use origin_region alphabetically
                IF NEW.origin_region < existing_record.origin_region THEN
                    INSERT INTO conflicts_audit
                        (table_name, record_id, conflict_type,
                         local_updated_at, remote_updated_at,
                         local_origin, remote_origin, resolution)
                    VALUES
                        ('orders', NEW.order_id, 'update_conflict_tie',
                         local_ts, remote_ts,
                         existing_record.origin_region, NEW.origin_region,
                         'remote_wins_alphabetical_tiebreak');
                    RETURN NEW;
                ELSE
                    INSERT INTO conflicts_audit
                        (table_name, record_id, conflict_type,
                         local_updated_at, remote_updated_at,
                         local_origin, remote_origin, resolution)
                    VALUES
                        ('orders', NEW.order_id, 'update_conflict_tie',
                         local_ts, remote_ts,
                         existing_record.origin_region, NEW.origin_region,
                         'local_wins_alphabetical_tiebreak');
                    RETURN NULL;  -- Keep the local version
                END IF;
            ELSE
                -- Local is newer, discard the remote change
                INSERT INTO conflicts_audit
                    (table_name, record_id, conflict_type,
                     local_updated_at, remote_updated_at,
                     local_origin, remote_origin, resolution)
                VALUES
                    ('orders', NEW.order_id, 'update_conflict',
                     local_ts, remote_ts,
                     existing_record.origin_region, NEW.origin_region,
                     'local_wins_last_writer_wins');
                RETURN NULL;  -- Discard the remote update
            END IF;
        END IF;
    END IF;

    -- For non-replicated changes, always proceed
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach the trigger
CREATE TRIGGER trg_resolve_order_conflict
    BEFORE UPDATE ON orders
    FOR EACH ROW
    EXECUTE FUNCTION resolve_order_conflict();
```

### Verify Conflict Resolution

```sql
-- Simulate a conflict: update the same order on both regions simultaneously

-- On us-east (run first):
UPDATE orders SET amount = 100.00, updated_at = '2024-01-15 10:00:00+00'
WHERE order_id = 1;

-- On eu-west (run immediately after, with a slightly later timestamp):
UPDATE orders SET amount = 150.00, updated_at = '2024-01-15 10:00:01+00'
WHERE order_id = 1;

-- After replication settles, check which value won:
SELECT order_id, amount, updated_at, origin_region FROM orders WHERE order_id = 1;
-- Expected: amount = 150.00 (eu-west had the later timestamp)

-- Check the audit log:
SELECT * FROM conflicts_audit ORDER BY resolved_at DESC LIMIT 5;
```

## Part D: Application-Level Routing

```python
import psycopg2
from psycopg2 import pool
import logging
import time
from enum import Enum
from typing import Optional

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class Region(Enum):
    US_EAST = "us-east"
    EU_WEST = "eu-west"


class RegionRouter:
    """
    Routes database queries to the correct regional PostgreSQL instance
    based on tenant data ownership.
    """

    def __init__(self, us_east_config: dict, eu_west_config: dict):
        self.pools = {
            Region.US_EAST: self._create_pool(us_east_config, Region.US_EAST),
            Region.EU_WEST: self._create_pool(eu_west_config, Region.EU_WEST),
        }
        self.tenant_map: dict[int, Region] = {}
        self.region_health: dict[Region, bool] = {
            Region.US_EAST: True,
            Region.EU_WEST: True,
        }
        self._load_tenant_map()
        logger.info("RegionRouter initialized with %d tenants mapped", len(self.tenant_map))

    def _create_pool(self, config: dict, region: Region) -> pool.ThreadedConnectionPool:
        """Create a connection pool for a region."""
        try:
            p = pool.ThreadedConnectionPool(
                minconn=2,
                maxconn=config.get("pool_size", 10),
                host=config["host"],
                port=config.get("port", 5432),
                dbname=config.get("database", "postgres"),
                user=config["user"],
                password=config["password"],
                connect_timeout=config.get("connect_timeout", 5),
            )
            logger.info("Connection pool created for %s", region.value)
            return p
        except psycopg2.OperationalError as e:
            logger.error("Failed to create pool for %s: %s", region.value, e)
            self.region_health[region] = False
            raise

    def _load_tenant_map(self):
        """Load tenant-to-region mapping from the database."""
        try:
            conn = self.pools[Region.US_EAST].getconn()
            try:
                with conn.cursor() as cur:
                    cur.execute("SELECT tenant_id, home_region FROM tenant_region_map")
                    for tenant_id, home_region in cur.fetchall():
                        self.tenant_map[tenant_id] = Region(home_region)
            finally:
                self.pools[Region.US_EAST].putconn(conn)
        except Exception as e:
            logger.error("Failed to load tenant map: %s", e)

    def get_tenant_region(self, tenant_id: int) -> Region:
        """Get the home region for a tenant."""
        region = self.tenant_map.get(tenant_id)
        if region is None:
            logger.warning(
                "Tenant %d not found in region map, defaulting to US_EAST", tenant_id
            )
            return Region.US_EAST
        return region

    def get_connection(
        self, tenant_id: int, operation: str = "read", strong_consistency: bool = False
    ):
        """
        Get a database connection for a tenant's query.

        Args:
            tenant_id: The tenant executing the query
            operation: 'read' or 'write'
            strong_consistency: If True, always use the home region for reads
        """
        home_region = self.get_tenant_region(tenant_id)

        if operation == "write":
            # Writes always go to the tenant's home region
            return self._get_connection_from_region(home_region, tenant_id)
        else:
            # Reads: use local region for eventual consistency,
            # home region for strong consistency
            if strong_consistency:
                return self._get_connection_from_region(home_region, tenant_id)
            else:
                # Try local region first, fall back to home region
                return self._get_connection_with_fallback(home_region, tenant_id)

    def _get_connection_from_region(self, region: Region, tenant_id: int):
        """Get a connection from a specific region, with health-aware fallback."""
        if self.region_health.get(region, False):
            try:
                conn = self.pools[region].getconn()
                logger.debug(
                    "Acquired connection from %s for tenant %d",
                    region.value,
                    tenant_id,
                )
                return conn
            except Exception as e:
                logger.error(
                    "Failed to get connection from %s: %s", region.value, e
                )
                self.region_health[region] = False

        # Fallback to the other region
        fallback = Region.EU_WEST if region == Region.US_EAST else Region.US_EAST
        if self.region_health.get(fallback, False):
            logger.warning(
                "Failing over tenant %d from %s to %s",
                tenant_id,
                region.value,
                fallback.value,
            )
            try:
                return self.pools[fallback].getconn()
            except Exception as e:
                logger.error(
                    "Fallback to %s also failed: %s", fallback.value, e
                )
                raise
        raise ConnectionError(
            f"No healthy region available for tenant {tenant_id}"
        )

    def _get_connection_with_fallback(self, home_region: Region, tenant_id: int):
        """Try the nearest region first, then fall back to home region."""
        # In a real system, "nearest" is determined by the application server's location
        # Here, we try home region first as a simplification
        return self._get_connection_from_region(home_region, tenant_id)

    def release_connection(self, conn, tenant_id: int):
        """Return a connection to its pool."""
        home_region = self.get_tenant_region(tenant_id)
        try:
            self.pools[home_region].putconn(conn)
        except Exception:
            # Connection may belong to a different pool (failover case)
            for region, p in self.pools.items():
                try:
                    p.putconn(conn)
                    break
                except Exception:
                    continue

    def check_region_health(self):
        """Periodically check health of each region."""
        for region, p in self.pools.items():
            try:
                conn = p.getconn()
                try:
                    with conn.cursor() as cur:
                        cur.execute("SELECT 1")
                        self.region_health[region] = True
                        logger.debug("Region %s is healthy", region.value)
                finally:
                    p.putconn(conn)
            except Exception as e:
                self.region_health[region] = False
                logger.error("Region %s health check failed: %s", region.value, e)


# --- Usage Example ---

US_EAST_CONFIG = {
    "host": "10.1.0.10",
    "port": 5432,
    "database": "orders_db",
    "user": "app_user",
    "password": "app_password",
    "pool_size": 20,
    "connect_timeout": 5,
}

EU_WEST_CONFIG = {
    "host": "10.2.0.10",
    "port": 5432,
    "database": "orders_db",
    "user": "app_user",
    "password": "app_password",
    "pool_size": 20,
    "connect_timeout": 5,
}

router = RegionRouter(US_EAST_CONFIG, EU_WEST_CONFIG)

# Write to a US tenant's home region
conn = router.get_connection(tenant_id=1001, operation="write")
try:
    with conn.cursor() as cur:
        cur.execute(
            "INSERT INTO orders (tenant_id, amount, status, origin_region) "
            "VALUES (%s, %s, %s, %s)",
            (1001, 299.99, "confirmed", "us-east"),
        )
    conn.commit()
finally:
    router.release_connection(conn, tenant_id=1001)

# Read from the nearest region (eventual consistency)
conn = router.get_connection(tenant_id=1001, operation="read")
try:
    with conn.cursor() as cur:
        cur.execute("SELECT * FROM orders WHERE tenant_id = %s", (1001,))
        orders = cur.fetchall()
finally:
    router.release_connection(conn, tenant_id=1001)
```

## Common Mistakes to Avoid

1. **Using streaming replication instead of logical replication for active-active.** Streaming replication creates a physical copy and requires both nodes to be on the same WAL timeline. This is fundamentally incompatible with multi-writer setups. Logical replication is required because it replicates logical changes that can be applied independently.

2. **Forgetting `origin = NONE` on subscriptions.** Without this setting, changes received from one subscription will be forwarded to the other subscription, creating an infinite replication loop. This will consume all available disk space and CPU within minutes.

3. **Relying solely on `updated_at` for conflict resolution without clock synchronization.** If the clocks on the two nodes differ by more than a few milliseconds, the "last writer wins" strategy will favor whichever node has the faster clock, not the actual latest write. Use NTP with a maximum drift of <10ms, and consider adding a logical clock or sequence number as a tiebreaker.

4. **Not handling INSERT conflicts.** The trigger solution above handles UPDATE conflicts, but INSERT conflicts (same `order_id` on both nodes) require a different strategy. Use `ON CONFLICT` clauses or ensure `order_id` is generated with region-specific sequences (e.g., odd IDs in us-east, even IDs in eu-west).

5. **Assuming active-active means zero latency for cross-region writes.** If a US tenant submits a write while traveling in Europe, the write must still go to the US region (their home region). The 100ms+ latency is unavoidable unless you allow the tenant to temporarily operate in "degraded" mode with local writes and conflict resolution.

6. **Not monitoring the conflicts_audit table.** Conflicts should be rare in a well-partitioned system. If the conflict rate increases, it indicates a routing problem (writes are going to the wrong region) or a partitioning problem (the tenant map is incorrect). Set up alerts when conflict rate exceeds a threshold (e.g., >0.1% of writes).

## Key Takeaway

Active-active is the most complex HA pattern because it requires the application, database, and operational processes to all understand data ownership. The database alone cannot solve the conflict problem -- it can only detect and resolve conflicts after they happen. The real solution is to prevent conflicts through careful partitioning (routing writes to the correct region) and to treat conflict resolution as a safety net, not the primary strategy. If you find yourself resolving many conflicts, your partitioning strategy is wrong.
