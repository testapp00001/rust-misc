# Solution 03: Zero-Downtime Column Migration

## Part A: Multi-Step Migration Process

### Migration Plan

| Step | Action | Requires Deploy | Reversible | Estimated Duration |
|------|--------|-----------------|------------|-------------------|
| 1 | Add `email_address` column (nullable) | No | Yes (DROP COLUMN) | Instant (metadata only) |
| 2 | Create sync triggers | No | Yes (DROP TRIGGER) | < 1 second |
| 3 | Backfill `email_address` from `email` | No | Yes (SET NULL) | 2-4 hours (batched) |
| 4 | Deploy app code: write to both, read from `email` | Yes | Yes (revert deploy) | Deploy time |
| 5 | Add unique index on `email_address` (CONCURRENTLY) | No | Yes (DROP INDEX) | 30-60 minutes |
| 6 | Deploy app code: read from `email_address`, write to both | Yes | Yes (revert deploy) | Deploy time |
| 7 | Verify both columns are in sync | No | N/A | 5 minutes |
| 8 | Drop triggers, drop `email` column, drop old index | No | No (destructive) | Instant to minutes |

**Total timeline: 1-2 days** (mostly waiting for backfill to complete)

**Critical insight**: Steps 1-3 are database-only and can be rolled back independently. Steps 4 and 6 are application deploys that can be reverted. Step 8 is destructive and not reversible -- only proceed after thorough verification.

## Part B: SQL for Each Step

### Step 1: Add the New Column

```sql
-- Migration: V001__add_email_address_column.sql
-- Run by: migration tool (Flyway, gh-ost, or manual)
-- When: During normal operations, no downtime
-- Rollback: ALTER TABLE users DROP COLUMN email_address;

ALTER TABLE users ADD COLUMN email_address VARCHAR(255) NULL;
```

This is instant in both MySQL and PostgreSQL because the column is nullable and added at the end of the table.

### Step 2: Create Sync Triggers

```sql
-- Migration: V002__create_email_sync_triggers.sql
-- Run by: migration tool
-- When: Immediately after Step 1
-- Rollback: DROP TRIGGER IF EXISTS sync_email_address_insert; DROP TRIGGER IF EXISTS sync_email_address_update;

-- MySQL version:
DELIMITER //

CREATE TRIGGER sync_email_address_insert
BEFORE INSERT ON users
FOR EACH ROW
BEGIN
    IF NEW.email_address IS NULL THEN
        SET NEW.email_address = NEW.email;
    END IF;
    IF NEW.email IS NULL THEN
        SET NEW.email = NEW.email_address;
    END IF;
END//

CREATE TRIGGER sync_email_address_update
BEFORE UPDATE ON users
FOR EACH ROW
BEGIN
    IF NEW.email != OLD.email THEN
        SET NEW.email_address = NEW.email;
    ELSEIF NEW.email_address != OLD.email_address THEN
        SET NEW.email = NEW.email_address;
    END IF;
END//

DELIMITER ;
```

```sql
-- PostgreSQL version:
CREATE OR REPLACE FUNCTION sync_email_columns()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.email_address IS NULL THEN
            NEW.email_address := NEW.email;
        END IF;
        IF NEW.email IS NULL THEN
            NEW.email := NEW.email_address;
        END IF;
    ELSIF TG_OP = 'UPDATE' THEN
        IF NEW.email IS DISTINCT FROM OLD.email THEN
            NEW.email_address := NEW.email;
        ELSIF NEW.email_address IS DISTINCT FROM OLD.email_address THEN
            NEW.email := NEW.email_address;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER sync_email_address
BEFORE INSERT OR UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION sync_email_columns();
```

The trigger logic is bidirectional: if `email` changes, update `email_address`, and vice versa. This handles both the old-app (writes to `email`) and new-app (writes to `email_address`) scenarios.

### Step 3: Backfill Existing Data

```sql
-- Migration: V003__backfill_email_address.sql
-- Run by: migration script (not a single SQL statement)
-- When: After triggers are in place
-- Rollback: UPDATE users SET email_address = NULL;

-- Backfill script (bash + mysql):
```

```bash
#!/bin/bash
# backfill_email_address.sh
# Run this on the migration workstation

DB_HOST="db-primary.prod.internal"
DB_USER="migration_user"
DB_PASS="secure-password"
DB_NAME="ecommerce"
BATCH_SIZE=10000
SLEEP_BETWEEN=0.5  # seconds between batches

echo "Starting backfill of email_address column..."

while true; do
    # Update a batch of rows
    RESULT=$(mysql -h "$DB_HOST" -u "$DB_USER" -p"$DB_PASS" "$DB_NAME" -N -e "
        UPDATE users
        SET email_address = email
        WHERE email_address IS NULL
        LIMIT $BATCH_SIZE;
        SELECT ROW_COUNT();
    ")

    UPDATED=$(echo "$RESULT" | tail -1)
    echo "$(date '+%Y-%m-%d %H:%M:%S') - Updated $UPDATED rows"

    if [ "$UPDATED" -eq 0 ]; then
        echo "Backfill complete!"
        break
    fi

    # Sleep between batches to reduce load
    sleep "$SLEEP_BETWEEN"
done
```

```bash
# For PostgreSQL, use a similar approach with psql:
#!/bin/bash
# backfill_email_address_pg.sh

DB_HOST="db-primary.prod.internal"
DB_USER="migration_user"
DB_NAME="ecommerce"
BATCH_SIZE=10000
SLEEP_BETWEEN=0.5

export PGPASSWORD="secure-password"

echo "Starting backfill..."

while true; do
    UPDATED=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -t -c "
        WITH batch AS (
            SELECT id FROM users
            WHERE email_address IS NULL
            ORDER BY id
            LIMIT $BATCH_SIZE
            FOR UPDATE SKIP LOCKED
        )
        UPDATE users u
        SET email_address = u.email
        FROM batch b
        WHERE u.id = b.id;
    " | tr -d ' ')

    echo "$(date '+%Y-%m-%d %H:%M:%S') - Updated $UPDATED rows"

    if [ "$UPDATED" -eq 0 ]; then
        echo "Backfill complete!"
        break
    fi

    sleep "$SLEEP_BETWEEN"
done
```

Key design decisions in the backfill:
- **Batched**: 10,000 rows at a time to avoid long-running transactions
- **Ordered by primary key**: Ensures consistent pagination and avoids missing rows
- **`FOR UPDATE SKIP LOCKED`** (PostgreSQL): Avoids blocking on rows being updated by the application
- **Sleep between batches**: Reduces CPU and I/O impact on the primary

### Step 4: Application Deploy -- Write to Both, Read from Old

```python
# Application code change (Python/SQLAlchemy example)
# Phase 1: Write to both columns, read from email

class UserRepository:
    def get_user(self, user_id: int) -> dict:
        # Still reading from 'email' column
        row = db.execute(
            "SELECT id, username, email, created_at FROM users WHERE id = %s",
            (user_id,)
        )
        return {"id": row[0], "username": row[1], "email": row[2]}

    def create_user(self, username: str, email: str) -> int:
        # Writing to both columns
        result = db.execute(
            "INSERT INTO users (username, email, email_address) VALUES (%s, %s, %s)",
            (username, email, email)
        )
        return result.lastrowid

    def update_email(self, user_id: int, new_email: str):
        # Writing to both columns
        db.execute(
            "UPDATE users SET email = %s, email_address = %s WHERE id = %s",
            (new_email, new_email, user_id)
        )
```

### Step 5: Add Unique Index on email_address

```sql
-- MySQL: Online index creation
ALTER TABLE users ADD UNIQUE INDEX idx_email_address (email_address), ALGORITHM=INPLACE, LOCK=NONE;

-- PostgreSQL: Concurrent index creation (no write lock)
CREATE UNIQUE INDEX CONCURRENTLY idx_email_address ON users(email_address);
```

Note: `CREATE INDEX CONCURRENTLY` in PostgreSQL cannot run inside a transaction. Run it as a standalone statement. It will take longer than a regular index creation but will not block writes.

### Step 6: Application Deploy -- Read from New Column

```python
# Phase 2: Read from email_address, write to both
class UserRepository:
    def get_user(self, user_id: int) -> dict:
        # Now reading from 'email_address' column
        row = db.execute(
            "SELECT id, username, email_address, created_at FROM users WHERE id = %s",
            (user_id,)
        )
        return {"id": row[0], "username": row[1], "email": row[2]}

    def create_user(self, username: str, email: str) -> int:
        # Still writing to both columns (safety net)
        result = db.execute(
            "INSERT INTO users (username, email, email_address) VALUES (%s, %s, %s)",
            (username, email, email)
        )
        return result.lastrowid
```

## Part C: Handle the Transition Period

### Trigger Logic (Complete)

The triggers in Part B handle the transition period. The key scenarios:

1. **Old app code inserts with `email` only**: Trigger sets `email_address = email`
2. **New app code inserts with `email_address` only**: Trigger sets `email = email_address`
3. **New app code inserts with both**: Both values are set, no trigger action needed
4. **Update changes `email`**: Trigger updates `email_address`
5. **Update changes `email_address`**: Trigger updates `email`

### Monitoring Sync During Transition

```sql
-- MySQL: Check for sync mismatches
SELECT
    COUNT(*) as total_rows,
    SUM(CASE WHEN email = email_address THEN 1 ELSE 0 END) as synced_rows,
    SUM(CASE WHEN email != email_address THEN 1 ELSE 0 END) as mismatched_rows,
    SUM(CASE WHEN email_address IS NULL THEN 1 ELSE 0 END) as null_email_address
FROM users;

-- PostgreSQL: Same query works

-- Set up alerting if mismatched_rows > 0
```

```bash
#!/bin/bash
# monitor_sync.sh - Run this continuously during the transition period

DB_HOST="db-primary.prod.internal"
DB_USER="monitoring_user"
DB_PASS="monitoring-password"
DB_NAME="ecommerce"

while true; do
    RESULT=$(mysql -h "$DB_HOST" -u "$DB_USER" -p"$DB_PASS" "$DB_NAME" -N -e "
        SELECT
            COUNT(*),
            SUM(CASE WHEN email != email_address THEN 1 ELSE 0 END),
            SUM(CASE WHEN email_address IS NULL THEN 1 ELSE 0 END)
        FROM users;
    ")

    TOTAL=$(echo "$RESULT" | awk '{print $1}')
    MISMATCHED=$(echo "$RESULT" | awk '{print $2}')
    NULLS=$(echo "$RESULT" | awk '{print $3}')

    echo "$(date '+%Y-%m-%d %H:%M:%S') - Total: $TOTAL, Mismatched: $MISMATCHED, Nulls: $NULLS"

    if [ "$MISMATCHED" -gt 0 ] || [ "$NULLS" -gt 0 ]; then
        echo "WARNING: Sync issues detected!"
        # Send alert (PagerDuty, Slack, etc.)
    fi

    sleep 60
done
```

## Part D: Cleanup Migration

```sql
-- Migration: V004__cleanup_old_email_column.sql
-- Run by: migration tool
-- When: After Step 6 deploy is stable for at least 24 hours
-- Rollback: NOT REVERSIBLE -- ensure backups exist

-- Step 1: Verify all data is in sync
DO $$
DECLARE
    mismatched_count BIGINT;
    null_count BIGINT;
BEGIN
    -- Check for mismatches
    SELECT COUNT(*) INTO mismatched_count
    FROM users WHERE email IS DISTINCT FROM email_address;

    IF mismatched_count > 0 THEN
        RAISE EXCEPTION 'Cannot cleanup: % rows have mismatched email/email_address', mismatched_count;
    END IF;

    -- Check for nulls in email_address
    SELECT COUNT(*) INTO null_count
    FROM users WHERE email_address IS NULL;

    IF null_count > 0 THEN
        RAISE EXCEPTION 'Cannot cleanup: % rows have NULL email_address', null_count;
    END IF;

    RAISE NOTICE 'All checks passed. Proceeding with cleanup.';
END $$;

-- Step 2: Drop sync triggers
DROP TRIGGER IF EXISTS sync_email_address ON users;  -- PostgreSQL
-- MySQL: DROP TRIGGER IF EXISTS sync_email_address_insert; DROP TRIGGER IF EXISTS sync_email_address_update;

-- Step 3: Drop the old unique index on email
DROP INDEX IF EXISTS idx_email;  -- Adjust name to match your actual index name

-- Step 4: Drop the old email column (idempotent via DO block)
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users' AND column_name = 'email'
    ) THEN
        ALTER TABLE users DROP COLUMN email;
    END IF;
END $$;

-- Step 5: Rename the new unique index to the canonical name
-- This makes the new index indistinguishable from the original
ALTER INDEX IF EXISTS idx_email_address RENAME TO idx_email;
```

For MySQL, the cleanup is slightly different:

```sql
-- MySQL cleanup
-- Step 1: Verify sync
SELECT COUNT(*) as mismatched FROM users WHERE email != email_address;
-- Must return 0

SELECT COUNT(*) as nulls FROM users WHERE email_address IS NULL;
-- Must return 0

-- Step 2: Drop triggers
DROP TRIGGER IF EXISTS sync_email_address_insert;
DROP TRIGGER IF EXISTS sync_email_address_update;

-- Step 3: Drop old index and column
ALTER TABLE users DROP INDEX idx_email;
ALTER TABLE users DROP COLUMN email;
```

## Common Mistakes to Avoid

1. **Not creating triggers before backfill**: If you backfill without triggers, any INSERT/UPDATE during the backfill will create rows where the columns are out of sync.

2. **Single UPDATE for backfill**: Running `UPDATE users SET email_address = email` on 100M rows will lock the table for hours and generate massive WAL/binlog. Always batch.

3. **Dropping the old column too soon**: Wait at least 24 hours after the final app deploy before dropping the old column. If a hidden cron job or stored procedure references the old column, you want to find it before it becomes a problem.

4. **Not monitoring sync during transition**: If the triggers have a bug, the columns will silently diverge. Monitor the mismatch count continuously.

5. **Forgetting the unique index**: The new column needs its own unique index. Without it, the email uniqueness constraint is lost when the old column is dropped.

## Key Takeaway

The expand-contract pattern transforms a dangerous column rename into a series of safe, reversible steps. The key insight is that the old and new columns coexist during a transition period, kept in sync by database triggers. At no point does the application have to stop, and at every step except the final cleanup, you can roll back safely. The trade-off is operational complexity -- this approach requires 4 migrations and 2 application deploys instead of a single ALTER TABLE.
