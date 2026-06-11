# Solution 04: Large Table Migration Strategy

## Part A: Evaluate Migration Approaches

### Comparison Matrix

| Approach | Downtime | Risk | Disk Required | Replication Impact | Complexity |
|----------|----------|------|---------------|-------------------|------------|
| Direct ALTER TABLE | 4-12 hours | Very High | ~500GB temp | Heavy (full table rewrite in WAL) | Low |
| pg_repack / pg_squeeze | 0 (online) | Medium | ~500GB temp | Moderate | Medium |
| Logical replication | < 5 min | Medium | ~500GB new table | Moderate | High |
| Create-and-copy + triggers | < 5 min | Low | ~500GB new table + trigger overhead | Low-Moderate | High |
| Application dual-write | 0 | High | ~500GB new table | None | Very High |

### Detailed Evaluation

#### 1. Direct ALTER TABLE

```sql
-- This will block the table for hours
ALTER TABLE orders
    DROP COLUMN product_id,
    DROP COLUMN quantity,
    DROP COLUMN unit_price,
    ADD COLUMN items JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN shipping_id BIGINT REFERENCES shipments(id);
```

**Feasibility**: Technically possible but completely unacceptable.

**Expected downtime**: 4-12 hours for a 500GB table. PostgreSQL must rewrite the entire table for most ALTER TABLE operations that change column types or drop columns.

**Risk level**: Very high. If the migration fails partway through, PostgreSQL must roll back the entire operation, which also takes hours. During this time, the table is locked.

**Verdict**: Rejected. Violates the 5-minute downtime constraint.

#### 2. pg_repack / pg_squeeze

pg_repack can reorganize a table online by creating a copy and swapping it. However, it has significant limitations for this scenario:

**Limitations**:
- pg_repack cannot change column types or add/remove columns
- It can only reorganize (repack) the table to reclaim space or reorder rows
- It does not support structural schema changes

**pg_squeeze** is similar but focused on reclaiming bloat.

**Verdict**: Not applicable. These tools reorganize data within the existing schema but cannot perform the structural changes required (adding JSONB column, removing columns).

#### 3. Logical Replication

PostgreSQL logical replication can stream changes from one table to another, even with different schemas.

**Process**:
1. Create `orders_new` with the target schema
2. Set up logical replication from `orders` to `orders_new`
3. Wait for initial sync to complete
4. Catch up on ongoing changes
5. Brief downtime to swap tables

**Feasibility**: Possible but complex.

**Expected downtime**: 2-5 minutes for the final swap.

**Risk level**: Medium. Logical replication has several gotchas:
- DDL changes are not replicated
- Initial sync for 500GB takes hours and generates significant WAL
- Sequences are not replicated (must be synced manually)
- The replication slot can cause WAL accumulation if the subscriber falls behind

**Disk required**: 500GB for the new table + WAL retention for the replication slot.

**Verdict**: Viable but complex. The initial sync time and WAL management make this risky for a 500GB table.

#### 4. Create-and-Copy with Triggers (Recommended)

**Process**:
1. Create `orders_new` with the target schema
2. Set up triggers on `orders` to capture INSERT/UPDATE/DELETE into `orders_new`
3. Batch-copy historical data from `orders` to `orders_new`
4. Once caught up, swap tables in a brief downtime window

**Feasibility**: Best fit for this scenario.

**Expected downtime**: 1-5 minutes for the final swap.

**Risk level**: Low. Each step is independently reversible.

**Disk required**: 500GB for the new table (temporary, can be dropped after swap).

**Replication impact**: Moderate during backfill (large INSERT generates WAL), but controllable with batching.

**Verdict**: Recommended approach.

#### 5. Application-Level Dual-Write

**Process**:
1. Modify the application to write to both old and new tables
2. Batch-copy historical data
3. Switch reads to the new table
4. Stop writing to the old table

**Feasibility**: Requires application code changes to every service that writes to the orders table.

**Expected downtime**: 0 (fully online).

**Risk level**: High. Dual-write has consistency risks:
- If one write succeeds and the other fails, the tables diverge
- Requires distributed transaction or eventual consistency handling
- Application complexity increases significantly

**Verdict**: Rejected. Too much application complexity for what should be a database-level operation.

## Part B: Design the Batch Migration Process

### Step 1: Create the New Table

```sql
-- Create the new table with the target schema
CREATE TABLE orders_new (
    id           BIGSERIAL PRIMARY KEY,
    user_id      BIGINT NOT NULL,
    items        JSONB NOT NULL,
    total_price  DECIMAL(10,2) NOT NULL,
    status       VARCHAR(50) NOT NULL,
    shipping_id  BIGINT,
    created_at   TIMESTAMPTZ DEFAULT NOW(),
    updated_at   TIMESTAMPTZ DEFAULT NOW()
);

-- Add foreign keys
ALTER TABLE orders_new ADD CONSTRAINT fk_orders_user_id
    FOREIGN KEY (user_id) REFERENCES users(id);
ALTER TABLE orders_new ADD CONSTRAINT fk_orders_shipping_id
    FOREIGN KEY (shipping_id) REFERENCES shipments(id);

-- Create indexes (do this BEFORE backfill for better performance)
CREATE INDEX idx_orders_new_user_id ON orders_new(user_id);
CREATE INDEX idx_orders_new_status ON orders_new(status);
CREATE INDEX idx_orders_new_created_at ON orders_new(created_at);
CREATE INDEX idx_orders_new_items ON orders_new USING GIN (items);
```

### Step 2: Set Up Change Capture Triggers

```sql
-- Function to convert old columns to JSONB
CREATE OR REPLACE FUNCTION orders_to_jsonb()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO orders_new (id, user_id, items, total_price, status, shipping_id, created_at, updated_at)
        VALUES (
            NEW.id,
            NEW.user_id,
            jsonb_build_object(
                'product_id', NEW.product_id,
                'quantity', NEW.quantity,
                'unit_price', NEW.unit_price
            ),
            NEW.total_price,
            NEW.status,
            NULL,  -- shipping_id doesn't exist yet
            NEW.created_at,
            NEW.updated_at
        );
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        UPDATE orders_new SET
            user_id = NEW.user_id,
            items = jsonb_build_object(
                'product_id', NEW.product_id,
                'quantity', NEW.quantity,
                'unit_price', NEW.unit_price
            ),
            total_price = NEW.total_price,
            status = NEW.status,
            updated_at = NEW.updated_at
        WHERE id = NEW.id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        DELETE FROM orders_new WHERE id = OLD.id;
        RETURN OLD;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER orders_capture_changes
AFTER INSERT OR UPDATE OR DELETE ON orders
FOR EACH ROW
EXECUTE FUNCTION orders_to_jsonb();
```

**Important**: Triggers fire for every row operation, so they add overhead to every INSERT/UPDATE/DELETE. Monitor the impact on write latency.

### Step 3: Batch Copy Historical Data

```sql
-- Batch copy script
-- Run this as a long-running process

DO $$
DECLARE
    batch_size INT := 5000;
    last_id BIGINT := 0;
    max_id BIGINT;
    rows_copied BIGINT := 0;
    batch_rows INT;
    start_time TIMESTAMPTZ;
    elapsed INTERVAL;
BEGIN
    SELECT MAX(id) INTO max_id FROM orders;
    RAISE NOTICE 'Starting backfill. Max ID: %, Total rows: ~%',
        max_id, (SELECT reltuples::bigint FROM pg_class WHERE oid = 'orders'::regclass);

    start_time := clock_timestamp();

    LOOP
        -- Insert a batch
        WITH batch AS (
            SELECT id, user_id, product_id, quantity, unit_price,
                   total_price, status, created_at, updated_at
            FROM orders
            WHERE id > last_id
            ORDER BY id
            LIMIT batch_size
        )
        INSERT INTO orders_new (id, user_id, items, total_price, status, created_at, updated_at)
        SELECT
            id, user_id,
            jsonb_build_object(
                'product_id', product_id,
                'quantity', quantity,
                'unit_price', unit_price
            ),
            total_price, status, created_at, updated_at
        FROM batch;

        GET DIAGNOSTICS batch_rows = ROW_COUNT;
        rows_copied := rows_copied + batch_rows;

        -- Update last_id
        SELECT MAX(id) INTO last_id FROM (
            SELECT id FROM orders WHERE id > last_id ORDER BY id LIMIT batch_size
        ) sub;

        -- Progress report every 100,000 rows
        IF rows_copied % 100000 < batch_size THEN
            elapsed := clock_timestamp() - start_time;
            RAISE NOTICE 'Copied % rows in % (ETA: %)',
                rows_copied,
                elapsed,
                CASE WHEN rows_copied > 0 THEN
                    (elapsed * (max_id::float / rows_copied - 1))::interval
                ELSE 'unknown' END;
        END IF;

        -- Exit when done
        EXIT WHEN batch_rows = 0;

        -- Throttle: sleep between batches to reduce load
        PERFORM pg_sleep(0.1);
    END LOOP;

    RAISE NOTICE 'Backfill complete. Total rows copied: %', rows_copied;
END $$;
```

### Step 4: Monitor Progress

```sql
-- Create a progress tracking view
CREATE OR REPLACE VIEW migration_progress AS
SELECT
    (SELECT COUNT(*) FROM orders_new) as rows_copied,
    (SELECT reltuples::bigint FROM pg_class WHERE oid = 'orders'::regclass) as estimated_total,
    pg_size_pretty(pg_total_relation_size('orders')) as old_table_size,
    pg_size_pretty(pg_total_relation_size('orders_new')) as new_table_size,
    (SELECT MAX(id) FROM orders_new) as copied_up_to_id,
    (SELECT MAX(id) FROM orders) as max_id;

-- Check progress
SELECT * FROM migration_progress;
```

```bash
#!/bin/bash
# monitor_migration.sh - Continuous progress monitoring

DB_HOST="db-primary.prod.internal"
DB_USER="monitoring_user"
DB_NAME="ecommerce"

export PGPASSWORD="monitoring-password"

while true; do
    clear
    echo "=== Migration Progress Monitor ==="
    echo "$(date '+%Y-%m-%d %H:%M:%S')"
    echo ""

    psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
        SELECT
            (SELECT COUNT(*) FROM orders_new) as rows_copied,
            (SELECT MAX(id) FROM orders_new) as copied_up_to_id,
            (SELECT MAX(id) FROM orders) as max_id,
            pg_size_pretty(pg_total_relation_size('orders_new')) as new_table_size;
    "

    echo ""
    echo "=== Replication Lag ==="
    psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
        SELECT
            client_addr,
            state,
            pg_wal_lsn_diff(sent_lsn, replay_lsn) as lag_bytes,
            pg_size_pretty(pg_wal_lsn_diff(sent_lsn, replay_lsn)) as lag_pretty
        FROM pg_stat_replication;
    "

    echo ""
    echo "=== Active Queries ==="
    psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
        SELECT pid, now() - query_start AS duration, state, left(query, 80)
        FROM pg_stat_activity
        WHERE state = 'active' AND query NOT LIKE '%pg_stat_activity%'
        ORDER BY duration DESC LIMIT 5;
    "

    sleep 10
done
```

### Step 5: Resource Throttling

```sql
-- Reduce checkpoint pressure during large copy
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET max_wal_size = '4GB';
SELECT pg_reload_conf();

-- Monitor I/O
SELECT * FROM pg_stat_io;

-- If the copy is impacting production, reduce batch size and increase sleep
```

## Part C: Handle Foreign Key Constraints

### The Problem

The `orders` table has foreign keys pointing to it from `order_items`, `payments`, and `order_history`. During the migration, new rows in these tables will reference `orders.id`. We need to ensure that after the swap, all foreign keys point to the new table.

### Solution: Table Rename Swap

The cleanest approach is to perform the swap in a single transaction:

```sql
-- Step 1: Verify orders_new is caught up (triggers have been capturing changes)
-- This should show 0 or very few rows
SELECT COUNT(*) FROM orders o
WHERE NOT EXISTS (SELECT 1 FROM orders_new n WHERE n.id = o.id);

-- Step 2: Acquire an exclusive lock and perform the swap
BEGIN;

-- Lock the tables to prevent any changes during swap
LOCK TABLE orders IN ACCESS EXCLUSIVE MODE;
LOCK TABLE orders_new IN ACCESS EXCLUSIVE MODE;
LOCK TABLE order_items IN ACCESS EXCLUSIVE MODE;
LOCK TABLE payments IN ACCESS EXCLUSIVE MODE;
LOCK TABLE order_history IN ACCESS EXCLUSIVE MODE;

-- Capture any final changes that slipped in before the lock
-- (The triggers should have handled these, but let's be safe)

-- Rename tables
ALTER TABLE orders RENAME TO orders_old;
ALTER TABLE orders_new RENAME TO orders;

-- Update foreign keys to point to the new table
-- Note: PostgreSQL automatically updates foreign keys when you rename
-- the referenced table IF the FK was created with the correct references.
-- However, FKs in OTHER tables pointing to orders need to be recreated.

-- Drop old foreign keys
ALTER TABLE order_items DROP CONSTRAINT IF EXISTS order_items_order_id_fkey;
ALTER TABLE payments DROP CONSTRAINT IF EXISTS payments_order_id_fkey;
ALTER TABLE order_history DROP CONSTRAINT IF EXISTS order_history_order_id_fkey;

-- Recreate foreign keys pointing to the new orders table
ALTER TABLE order_items ADD CONSTRAINT order_items_order_id_fkey
    FOREIGN KEY (order_id) REFERENCES orders(id);
ALTER TABLE payments ADD CONSTRAINT payments_order_id_fkey
    FOREIGN KEY (order_id) REFERENCES orders(id);
ALTER TABLE order_history ADD CONSTRAINT order_history_order_id_fkey
    FOREIGN KEY (order_id) REFERENCES orders(id);

-- Drop the change capture trigger (no longer needed)
DROP TRIGGER IF EXISTS orders_capture_changes ON orders_old;

COMMIT;
```

**This is the 5-minute downtime window.** The entire transaction above takes seconds if the FK recreation is fast.

### Handling In-flight Inserts During Switchover

If a row is inserted into `order_items` during the switchover:

1. The `BEGIN` + `LOCK TABLE` will wait for any active transactions to finish
2. New transactions will queue behind the lock
3. Once the lock is acquired, no new inserts can happen until `COMMIT`
4. After `COMMIT`, all foreign keys point to the new table

The brief lock ensures referential integrity is never violated.

### Alternative: Deferrable Foreign Keys

For even safer handling, you can make foreign keys deferrable:

```sql
-- Make FKs deferrable (do this before migration)
ALTER TABLE order_items DROP CONSTRAINT order_items_order_id_fkey;
ALTER TABLE order_items ADD CONSTRAINT order_items_order_id_fkey
    FOREIGN KEY (order_id) REFERENCES orders(id) DEFERRABLE INITIALLY DEFERRED;

-- During swap, set constraints deferred
BEGIN;
SET CONSTRAINTS ALL DEFERRED;
-- ... perform swap ...
COMMIT;
```

## Part D: Design the Verification Process

### 1. Row Count Verification

```sql
-- Compare row counts
SELECT
    (SELECT COUNT(*) FROM orders) as new_count,
    (SELECT COUNT(*) FROM orders_old) as old_count,
    (SELECT COUNT(*) FROM orders) - (SELECT COUNT(*) FROM orders_old) as difference;
-- difference should be 0 (or slightly positive if new rows were inserted during migration)
```

### 2. Data Integrity Checks

```sql
-- Verify JSONB conversion preserved all fields
SELECT COUNT(*) as missing_fields
FROM orders
WHERE NOT (items ? 'product_id' AND items ? 'quantity' AND items ? 'unit_price');

-- Verify total_price consistency
SELECT COUNT(*) as price_mismatch
FROM orders o
JOIN orders_old oo ON o.id = oo.id
WHERE o.total_price != oo.total_price;

-- Verify JSONB values match original columns
SELECT COUNT(*) as value_mismatch
FROM orders o
JOIN orders_old oo ON o.id = oo.id
WHERE
    (o.items->>'product_id')::bigint != oo.product_id OR
    (o.items->>'quantity')::int != oo.quantity OR
    (o.items->>'unit_price')::decimal != oo.unit_price;

-- Verify no null items
SELECT COUNT(*) as null_items FROM orders WHERE items IS NULL;
```

### 3. Foreign Key Integrity

```sql
-- Verify all order_items reference valid orders
SELECT COUNT(*) as orphaned_items
FROM order_items oi
WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.id = oi.order_id);

-- Verify all payments reference valid orders
SELECT COUNT(*) as orphaned_payments
FROM payments p
WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.id = p.order_id);

-- Verify all order_history references valid orders
SELECT COUNT(*) as orphaned_history
FROM order_history oh
WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.id = oh.order_id);

-- Verify all orders reference valid users
SELECT COUNT(*) as invalid_users
FROM orders o
WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = o.user_id);

-- Verify shipping_id references (if not null)
SELECT COUNT(*) as invalid_shipments
FROM orders o
WHERE o.shipping_id IS NOT NULL
AND NOT EXISTS (SELECT 1 FROM shipments s WHERE s.id = o.shipping_id);
```

### 4. Index Verification

```sql
-- Verify GIN index is valid
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename = 'orders' AND indexname LIKE '%items%';

-- Verify index is not invalid
SELECT indexrelid::regclass, indisvalid, indisready
FROM pg_index
WHERE indrelid = 'orders'::regclass;

-- Test GIN index with a query
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM orders WHERE items @> '{"product_id": 12345}';
-- Should show "Index Scan using idx_orders_new_items"
```

### 5. Application Smoke Tests

```python
# smoke_test.py - Run after migration
import psycopg2
import json

conn = psycopg2.connect("host=db-primary.prod.internal dbname=ecommerce user=app_user")
cur = conn.cursor()

# Test 1: Read an order
cur.execute("SELECT id, user_id, items, total_price, status FROM orders LIMIT 1")
order = cur.fetchone()
assert order is not None, "No orders found"
assert isinstance(order[2], dict), "items should be a dict/JSONB"
print(f"PASS: Read order {order[0]}")

# Test 2: Insert a new order
cur.execute("""
    INSERT INTO orders (user_id, items, total_price, status)
    VALUES (%s, %s, %s, %s)
    RETURNING id
""", (1, json.dumps({"product_id": 999, "quantity": 2, "unit_price": 29.99}), 59.98, 'pending'))
new_id = cur.fetchone()[0]
print(f"PASS: Inserted order {new_id}")

# Test 3: Query with JSONB containment
cur.execute("SELECT COUNT(*) FROM orders WHERE items @> %s", (json.dumps({"product_id": 999}),))
count = cur.fetchone()[0]
assert count >= 1, "JSONB query failed"
print(f"PASS: JSONB query returned {count} results")

# Test 4: Foreign key integrity
cur.execute("""
    INSERT INTO order_items (order_id, product_id, quantity)
    VALUES (%s, %s, %s)
""", (new_id, 999, 2))
print(f"PASS: Foreign key insert succeeded")

# Cleanup
conn.rollback()
cur.close()
conn.close()
print("All smoke tests passed!")
```

## Common Mistakes to Avoid

1. **Creating indexes after backfill**: For a 500GB table, creating indexes on an empty table is instant. Creating them after the table is full requires a full table scan. Always create indexes before the backfill.

2. **Not locking all related tables during swap**: If you only lock `orders` during the swap, a concurrent INSERT into `order_items` could reference a row that no longer exists after the rename.

3. **Forgetting to sync sequences**: After the swap, the `orders_id_seq` sequence must be set to the maximum ID in the new table, or you will get duplicate key errors.

    ```sql
    -- After swap, sync the sequence
    SELECT setval('orders_id_seq', (SELECT MAX(id) FROM orders));
    ```

4. **Not verifying the trigger overhead**: Triggers fire for every INSERT/UPDATE/DELETE. On a table with 3,000 writes/second, the trigger adds significant overhead. Test on staging first.

5. **Dropping the old table too soon**: Keep `orders_old` for at least a week. If a data issue is discovered, you can compare against the old table.

## Key Takeaway

For large table migrations with structural changes, the create-and-copy with triggers approach provides the best balance of safety, downtime, and complexity. The key principles are: (1) create the new table and indexes before copying, (2) use triggers to capture changes during the copy, (3) batch the copy to avoid overwhelming the system, (4) swap in a brief locked transaction, and (5) verify thoroughly before dropping the old table. The entire process is designed so that at every step except the final cleanup, you can roll back by simply dropping the new table and removing the triggers.
