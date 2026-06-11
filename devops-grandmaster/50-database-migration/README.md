# 50 - Database Migration

**Previous:** [49 - High Availability](../49-high-availability/README.md) | **Next:** [51 - Network Security](../51-network-security/README.md)

---

## Problem

Your application needs a new column. The developer writes `ALTER TABLE users ADD COLUMN phone VARCHAR(20);` and runs it against production. The table has 50 million rows. PostgreSQL acquires an `ACCESS EXCLUSIVE` lock on the table for the duration of the operation. Every query trying to read or write to `users` blocks. Your application grinds to a halt. Customers see 503 errors for 3 minutes.

Or worse: you need to rename a column. The application code references the old name. You deploy the migration and the new code simultaneously, but the migration runs first. Every query using the old column name fails. You rollback, but now the new code is broken. You are stuck.

Database migrations on live systems are one of the most dangerous operations in production engineering. A wrong move can cause outages that affect every user.

---

## Naive Way

```sql
-- "Just run it in production, it'll be quick"
ALTER TABLE orders ADD COLUMN discount_amount DECIMAL(10,2);
UPDATE users SET role = 'member' WHERE role IS NULL;
ALTER TABLE products RENAME COLUMN qty TO quantity;
CREATE INDEX idx_orders_email ON orders(email);  -- locks table for 10 minutes on 100M rows
```

**Why this fails:**
- `ALTER TABLE` acquires exclusive locks that block all reads and writes
- `UPDATE` on millions of rows holds locks for extended periods
- `RENAME COLUMN` breaks every query using the old name instantly
- `CREATE INDEX` locks the table for the entire index build duration
- No way to rollback if something goes wrong mid-migration
- Application and database are deployed independently, causing version mismatches

---

## Right Way

### The Expand-and-Contract Pattern

Never make destructive changes directly. Instead, evolve schemas in three safe phases:

**Phase 1: EXPAND** -- Add new structure without removing old
**Phase 2: MIGRATE** -- Move data to new structure
**Phase 3: CONTRACT** -- Remove old structure after code is updated

### Safe Column Addition

```sql
-- Phase 1: Add column with a default (PostgreSQL 11+ does this instantly)
-- The default is stored in the catalog, not written to each row
ALTER TABLE users ADD COLUMN phone VARCHAR(20) DEFAULT NULL;

-- Phase 2: Backfill in batches (if needed)
-- Process 10,000 rows at a time to avoid long-running transactions
DO $$
DECLARE
    batch_size INT := 10000;
    rows_updated INT;
BEGIN
    LOOP
        UPDATE users
        SET phone = legacy_phone
        WHERE phone IS NULL AND legacy_phone IS NOT NULL
        AND id IN (
            SELECT id FROM users
            WHERE phone IS NULL AND legacy_phone IS NOT NULL
            LIMIT batch_size
        );

        GET DIAGNOSTICS rows_updated = ROW_COUNT;
        EXIT WHEN rows_updated = 0;

        RAISE NOTICE 'Updated % rows', rows_updated;
        PERFORM pg_sleep(0.1);  -- Brief pause to reduce load
    END LOOP;
END $$;

-- Phase 3: After application code is updated, drop old column
ALTER TABLE users DROP COLUMN legacy_phone;
```

### Safe Index Creation Without Downtime

```sql
-- CREATE INDEX blocks reads and writes. Use CONCURRENTLY instead.
-- This builds the index without locking the table.
CREATE INDEX CONCURRENTLY idx_orders_email ON orders(email);

-- CONCURRENTLY takes longer but allows normal operations to continue.
-- Important: CONCURRENTLY cannot run inside a transaction block.
```

### Migration Tooling with sqlx (Rust)

```rust
// migrations/20240115_add_user_phone.rs
use sqlx::PgConnection;

pub async fn up(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    // Phase 1: Add new column (instant in PostgreSQL 11+)
    sqlx::query(
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS phone VARCHAR(20)"
    )
    .execute(&mut *conn)
    .await?;

    // Create index concurrently (must be outside transaction)
    // Handle this separately or in a dedicated migration step
    Ok(())
}

pub async fn down(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("ALTER TABLE users DROP COLUMN IF EXISTS phone")
        .execute(&mut *conn)
        .await?;
    Ok(())
}
```

### Safe Column Rename (Zero-Downtime)

```sql
-- NEVER rename a column directly. Instead, use expand-and-contract.

-- Step 1: Add new column
ALTER TABLE products ADD COLUMN quantity INTEGER;

-- Step 2: Create trigger to keep both columns in sync
CREATE OR REPLACE FUNCTION sync_quantity_columns()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' OR NEW.quantity IS DISTINCT FROM OLD.quantity THEN
        NEW.qty := NEW.quantity;
    ELSIF NEW.qty IS DISTINCT FROM OLD.qty THEN
        NEW.quantity := NEW.qty;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_sync_quantity
    BEFORE INSERT OR UPDATE ON products
    FOR EACH ROW
    EXECUTE FUNCTION sync_quantity_columns();

-- Step 3: Backfill from old to new
UPDATE products SET quantity = qty WHERE quantity IS NULL;

-- Step 4: Deploy application code that reads/writes "quantity"

-- Step 5: After all application instances are updated, remove trigger and old column
DROP TRIGGER trg_sync_quantity ON products;
ALTER TABLE products DROP COLUMN qty;
```

---

## Production Way

### Migration Framework with Safety Checks

```rust
// src/migrations/mod.rs
use sqlx::PgPool;
use std::time::Instant;

pub struct Migration {
    pub version: i64,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
    pub estimated_duration: std::time::Duration,
    pub requires_lock: bool,
    pub table_size_limit: Option<usize>,
}

impl Migration {
    pub async fn apply(&self, pool: &PgPool) -> Result<(), MigrationError> {
        // Pre-flight checks
        self.preflight_checks(pool).await?;

        let start = Instant::now();

        if self.requires_lock {
            // Use lock timeout to prevent indefinite blocking
            sqlx::query("SET lock_timeout = '5s'")
                .execute(pool)
                .await
                .map_err(|e| MigrationError::LockTimeout(e.to_string()))?;
        }

        // Run migration inside a transaction
        let mut tx = pool.begin().await?;

        // Record migration start
        sqlx::query(
            "INSERT INTO schema_migrations (version, name, started_at)
             VALUES ($1, $2, now())"
        )
        .bind(self.version)
        .bind(&self.name)
        .execute(&mut *tx)
        .await?;

        // Execute migration
        sqlx::query(&self.up_sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                MigrationError::ExecutionFailed {
                    version: self.version,
                    error: e.to_string(),
                }
            })?;

        // Record completion
        sqlx::query(
            "UPDATE schema_migrations SET completed_at = now(), duration_ms = $1
             WHERE version = $2"
        )
        .bind(start.elapsed().as_millis() as i64)
        .bind(self.version)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Reset lock timeout
        if self.requires_lock {
            sqlx::query("SET lock_timeout = 'default'")
                .execute(pool)
                .await?;
        }

        println!(
            "Migration {} ({}) applied in {:?}",
            self.version, self.name, start.elapsed()
        );

        Ok(())
    }

    async fn preflight_checks(&self, pool: &PgPool) -> Result<(), MigrationError> {
        // Check if migration already applied
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = $1)"
        )
        .bind(self.version)
        .fetch_one(pool)
        .await?;

        if exists {
            return Err(MigrationError::AlreadyApplied(self.version));
        }

        // Check table size if limited
        if let Some(limit) = self.table_size_limit {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
                .fetch_one(pool)
                .await?;

            if count as usize > limit {
                return Err(MigrationError::TableTooLarge {
                    actual: count as usize,
                    limit,
                });
            }
        }

        // Check for active transactions that would block us
        let blockers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pg_stat_activity
             WHERE state != 'idle' AND pid != pg_backend_pid()"
        )
        .fetch_one(pool)
        .await?;

        if blockers > 10 {
            println!("WARNING: {} active transactions. Migration may be slow.", blockers);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum MigrationError {
    AlreadyApplied(i64),
    LockTimeout(String),
    ExecutionFailed { version: i64, error: String },
    TableTooLarge { actual: usize, limit: usize },
}
```

### Migration Runner with Dry-Run Support

```rust
// src/migrations/runner.rs
pub struct MigrationRunner {
    pool: PgPool,
    dry_run: bool,
}

impl MigrationRunner {
    pub async fn run_pending(&self) -> Result<(), MigrationError> {
        let pending = self.get_pending_migrations().await?;

        if pending.is_empty() {
            println!("No pending migrations.");
            return Ok(());
        }

        println!("Pending migrations:");
        for m in &pending {
            println!("  {} - {} (est. {:?})", m.version, m.name, m.estimated_duration);
        }

        if self.dry_run {
            println!("\n[DRY RUN] Would apply {} migrations. No changes made.", pending.len());
            return Ok(());
        }

        for migration in pending {
            let start = Instant::now();

            println!("\nApplying {} - {}...", migration.version, migration.name);

            match migration.apply(&self.pool).await {
                Ok(()) => {
                    println!("  Completed in {:?}", start.elapsed());
                }
                Err(e) => {
                    eprintln!("  FAILED: {:?}", e);
                    eprintln!("  Halting migration run. Fix the issue and re-run.");
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}
```

### CI/CD Migration Safety Gate

```yaml
# .github/workflows/migrate.yml
name: Database Migration
on:
  push:
    paths:
      - 'migrations/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_DB: testdb
          POSTGRES_PASSWORD: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Apply migrations to test database
        run: cargo run -- migrate --database-url postgres://postgres:test@localhost/testdb

      - name: Run migration tests
        run: cargo test --test migrations

      - name: Dry-run against production schema
        run: |
          cargo run -- migrate \
            --database-url "$PRODUCTION_DB_URL" \
            --dry-run
        env:
          PRODUCTION_DB_URL: ${{ secrets.PROD_DB_URL }}

  apply:
    needs: validate
    runs-on: ubuntu-latest
    environment: production
    steps:
      - name: Apply migrations to production
        run: cargo run -- migrate --database-url "$DATABASE_URL"
        env:
          DATABASE_URL: ${{ secrets.PROD_DB_URL }}
```

### Large Table Migration with Online Schema Change Tools

```bash
# Using gh-ost (GitHub's online schema change tool for MySQL)
gh-ost \
    --host="primary-db" \
    --database="myapp" \
    --table="orders" \
    --alter="ADD COLUMN discount_amount DECIMAL(10,2) DEFAULT 0" \
    --chunk-size=1000 \
    --max-load="Threads_running=25" \
    --critical-load="Threads_running=1000" \
    --initially-drop-ghost-table \
    --serve-socket-file=/tmp/gh-ost.sock \
    --execute

# For PostgreSQL, use pgroll for zero-downtime migrations
pgroll start \
    --postgres-url "postgres://user:pass@host/db" \
    ./migrations/03_add_phone_column.yaml
```

---

## Hands-On Lab

### Lab: Perform Zero-Downtime Schema Migration

**Duration:** 60 minutes

**Prerequisites:**
- PostgreSQL 15 running locally or in Docker
- sqlx-cli installed (`cargo install sqlx-cli`)
- A Rust project with sqlx configured

**Step 1: Create a Large Test Table**

```bash
psql -U postgres -c "
CREATE DATABASE migration_lab;
\c migration_lab

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    customer_id INTEGER NOT NULL,
    total DECIMAL(10,2) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT now()
);

-- Insert 1 million rows
INSERT INTO orders (customer_id, total, status)
SELECT
    (random() * 10000)::int,
    (random() * 1000)::decimal(10,2),
    CASE WHEN random() > 0.5 THEN 'completed' ELSE 'pending' END
FROM generate_series(1, 1000000);

SELECT COUNT(*) FROM orders;
"
```

**Step 2: Attempt an Unsafe Migration (Observe the Problem)**

```sql
-- In one terminal, start a long-running transaction
BEGIN;
SELECT * FROM orders WHERE id = 1 FOR UPDATE;

-- In another terminal, try this (it will HANG waiting for the lock):
ALTER TABLE orders ADD COLUMN discount DECIMAL(10,2);
-- Cancel after 10 seconds with Ctrl+C

ROLLBACK;  -- Release the lock in terminal 1
```

**Step 3: Perform a Safe Migration**

```sql
-- Use lock timeout to prevent indefinite blocking
SET lock_timeout = '5s';

-- Add column with default (instant in PG 11+)
ALTER TABLE orders ADD COLUMN IF NOT EXISTS discount DECIMAL(10,2) DEFAULT 0;

-- Verify: table should not be locked
-- In another terminal:
SELECT * FROM orders LIMIT 5;  -- Should return immediately

-- Create index concurrently (non-blocking)
CREATE INDEX CONCURRENTLY idx_orders_customer ON orders(customer_id);
-- This takes longer but doesn't block reads/writes
```

**Step 4: Implement Expand-and-Contract for Column Rename**

```sql
-- Expand: add new column
ALTER TABLE orders ADD COLUMN order_status VARCHAR(20);

-- Create sync trigger
CREATE OR REPLACE FUNCTION sync_order_status()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.order_status IS NOT NULL THEN
        NEW.status := NEW.order_status;
    ELSE
        NEW.order_status := NEW.status;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_sync_status
    BEFORE INSERT OR UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION sync_order_status();

-- Backfill
UPDATE orders SET order_status = status WHERE order_status IS NULL;

-- Now deploy application code using order_status
-- After confirming everything works:
-- DROP TRIGGER trg_sync_status ON orders;
-- ALTER TABLE orders DROP COLUMN status;
```

**Step 5: Verify Application Compatibility**

```bash
# Application reads should work with both old and new column names
psql -U postgres -d migration_lab -c "
-- Old code
SELECT id, total, status FROM orders LIMIT 5;

-- New code
SELECT id, total, order_status FROM orders LIMIT 5;

-- Both should work simultaneously
"
```

**Deliverable:** Write a migration that safely adds a `tax_amount` column to the `orders` table, backfills it in batches, and creates an index on it -- all without blocking reads or writes.

---

## Limitation

Database migrations handle schema evolution safely, but they operate within the boundaries of your database's **security perimeter**. The migration runner connects to the database with credentials, executes DDL statements, and modifies data. If those credentials are compromised, an attacker can do everything your migration tool can do -- and more.

In production, database credentials are often:
- Hardcoded in configuration files committed to git
- Shared across environments (dev uses the same password as production)
- Never rotated because "it might break something"
- Stored in plaintext in CI/CD pipeline variables

A single leaked database credential can lead to full data exfiltration. The security perimeter of your system is only as strong as the protection around your secrets and network access.

---

## Next Topic

[51 - Network Security](../51-network-security/README.md) -- Learn how to secure your network perimeter with firewalls, VPNs, network segmentation, and defense-in-depth strategies that protect your infrastructure even when credentials are compromised.
