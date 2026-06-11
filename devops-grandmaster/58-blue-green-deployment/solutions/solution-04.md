# Solution 04: Database Migrations in Blue-Green

## Part A: Identify the Problem

Both blue (v1) and green (v2) connect to the same database. If you run
the migration before switching:

1. **If you drop `name` column:** Blue (v1) immediately crashes because its
   code references `SELECT name FROM users`. Every request returns a
   database error. Users see 500 errors.

2. **If you add columns but do not backfill:** Green (v2) reads
   `first_name` and `last_name`, but existing rows have NULL values. Users
   see empty names.

3. **If you backfill but do not sync:** A new user signs up through blue
   (v1), which writes to the `name` column. Green (v2) reads
   `first_name`/`last_name`, which are NULL for the new user. Data is
   inconsistent.

**The core problem:** A breaking schema change cannot be applied while both
versions are live. You need a migration strategy that keeps both versions
functional.

### Why This Matters

Database migrations are the hardest part of blue-green deployment. The
application code changes atomically (you switch traffic), but the database
schema cannot change atomically. You need a period where both schemas
coexist.

## Part B: Design a Safe Migration Strategy

The **expand and contract** pattern solves this:

| Phase | Database State | Blue (v1) | Green (v2) |
|-------|---------------|-----------|------------|
| **Expand** | Both `name` and `first_name`/`last_name` exist, synced | Reads `name` (works) | Reads `first_name`/`last_name` (works) |
| **Switch** | Same as expand | Stops receiving traffic | Starts receiving traffic |
| **Contract** | `name` column dropped | No longer running | Reads `first_name`/`last_name` (works) |

During the expand phase, a trigger keeps both column sets in sync. Any
write to `name` updates `first_name`/`last_name`, and vice versa. Both
versions can read and write without data loss.

## Part C: Write the Migration SQL

```sql
-- Phase 1: Expand (run BEFORE deploying v2)

-- Step 1: Add new columns (nullable, so existing rows are not affected)
ALTER TABLE users ADD COLUMN first_name VARCHAR(255);
ALTER TABLE users ADD COLUMN last_name VARCHAR(255);

-- Step 2: Backfill existing data
UPDATE users
SET first_name = split_part(name, ' ', 1),
    last_name = CASE
        WHEN position(' ' IN name) > 0
        THEN substring(name FROM position(' ' IN name) + 1)
        ELSE ''
    END
WHERE first_name IS NULL;

-- Step 3: Create trigger to keep columns in sync
CREATE OR REPLACE FUNCTION sync_user_name_columns()
RETURNS TRIGGER AS $$
BEGIN
    -- If first_name/last_name are being set, update name
    IF TG_OP = 'INSERT' OR
       (TG_OP = 'UPDATE' AND (NEW.first_name IS DISTINCT FROM OLD.first_name
                            OR NEW.last_name IS DISTINCT FROM OLD.last_name)) THEN
        NEW.name := COALESCE(NEW.first_name, '') ||
                    CASE WHEN NEW.last_name IS NOT NULL AND NEW.last_name != ''
                         THEN ' ' || NEW.last_name
                         ELSE ''
                    END;
    END IF;

    -- If name is being set directly, update first_name/last_name
    IF TG_OP = 'INSERT' OR
       (TG_OP = 'UPDATE' AND NEW.name IS DISTINCT FROM OLD.name) THEN
        IF NEW.first_name IS NULL OR NEW.first_name = '' THEN
            NEW.first_name := split_part(NEW.name, ' ', 1);
        END IF;
        IF NEW.last_name IS NULL OR NEW.last_name = '' THEN
            NEW.last_name := CASE
                WHEN position(' ' IN NEW.name) > 0
                THEN substring(NEW.name FROM position(' ' IN NEW.name) + 1)
                ELSE ''
            END;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_name_sync
    BEFORE INSERT OR UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION sync_user_name_columns();

-- Step 4: Verify backfill
SELECT COUNT(*) AS missing_names
FROM users
WHERE first_name IS NULL OR last_name IS NULL;
-- Should return 0
```

```sql
-- Phase 3: Contract (run AFTER green is stable and blue is decommissioned)

-- Step 1: Drop the trigger
DROP TRIGGER IF EXISTS user_name_sync ON users;
DROP FUNCTION IF EXISTS sync_user_name_columns();

-- Step 2: Drop the old column
ALTER TABLE users DROP COLUMN name;

-- Step 3: Add NOT NULL constraints (now safe because only v2 writes)
ALTER TABLE users ALTER COLUMN first_name SET NOT NULL;
ALTER TABLE users ALTER COLUMN last_name SET NOT NULL;
```

### Why This Works

The trigger ensures that no matter which version writes data, both column
sets stay in sync. When blue writes `INSERT INTO users (name) VALUES
('John Doe')`, the trigger automatically sets `first_name = 'John'` and
`last_name = 'Doe'`. When green writes `INSERT INTO users (first_name,
last_name) VALUES ('Jane', 'Smith')`, the trigger sets `name = 'Jane Smith'`.

The backfill handles existing data. The `CASE` in the UPDATE handles
names without spaces (single-word names).

## Part D: Full Deployment Sequence

```
Timeline:
                                                                   
  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐
  │ Blue v1  │   │ Expand  │   │ Deploy  │   │ Switch  │   │Contract │
  │ Running  │──>│Migration│──>│ Green   │──>│ Traffic │──>│Migration│
  │          │   │         │   │ v2      │   │         │   │         │
  └─────────┘   └─────────┘   └─────────┘   └─────────┘   └─────────┘
       |             |              |              |              |
       |        Both schemas    Green has      Traffic now    Old column
       |        coexist with   all pods       goes to        removed
       |        sync trigger   ready          green only     safely
```

### Step-by-step

```bash
#!/bin/bash
# deploy-v2-blue-green.sh

set -euo pipefail

echo "=== Step 1: Expand Migration ==="
psql "$DATABASE_URL" -f migrations/001_expand_name_columns.sql
echo "Expand migration complete. Both schemas coexist."

echo "=== Step 2: Deploy v2 to Green ==="
kubectl set image deployment/order-service-green \
    order-service=order-service:v2
kubectl rollout status deployment/order-service-green --timeout=180s
echo "Green deployment ready."

echo "=== Step 3: Verify Green Health ==="
kubectl wait --for=condition=ready pod \
    -l app=order-service,version=green --timeout=120s

# Smoke test against green directly
kubectl port-forward deployment/order-service-green 8080:8080 &
PF_PID=$!
sleep 2
curl -sf http://localhost:8080/health
kill $PF_PID
echo "Green health check passed."

echo "=== Step 4: Switch Traffic ==="
kubectl patch service order-service \
    -p '{"spec":{"selector":{"version":"green"}}}'
echo "Traffic switched to green."

echo "=== Step 5: Production Verification ==="
sleep 5
curl -sf https://order-service.example.com/health
echo "Production health check passed."

echo "=== Step 6: Monitor ==="
echo "Green is live. Monitor for 30 minutes before running contract migration."
echo "To rollback: kubectl patch service order-service -p '{\"spec\":{\"selector\":{\"version\":\"blue\"}}}'"
echo ""
echo "After 30 minutes of stable operation, run:"
echo "  psql \$DATABASE_URL -f migrations/002_contract_name_column.sql"
```

### Why This Matters

The contract migration is deliberately separated from the deployment. You
wait 30 minutes (or longer, depending on confidence) before dropping the
old column. This gives you time to:
- Monitor green for errors
- Roll back to blue if needed (blue still works because `name` column exists)
- Verify data consistency

Only after you are confident that green is stable do you drop the `name`
column. At that point, blue can no longer work, but you do not need it.

## Common Mistakes to Avoid

- **Running the contract migration too early.** If you drop the `name`
  column before green is proven stable, you lose the ability to roll back
  to blue. Wait until you are confident.
- **Not backfilling existing data.** Adding columns without backfilling
  means green reads NULL values for all existing users. Always backfill
  before deploying.
- **Forgetting the trigger.** Without the trigger, new writes through blue
  do not update the new columns. Green sees stale data for any user
  created or updated after the migration.
- **Not handling names without spaces.** `split_part('Alice', ' ', 2)`
  returns an empty string. Your backfill and trigger must handle
  single-word names gracefully.

## Key Takeaway

Database migrations in blue-green deployments require the expand-and-contract
pattern. Expand (add new columns, keep old, sync with trigger), switch
traffic, then contract (drop old columns). The expand phase ensures both
versions work with the same database. The contract phase only happens after
the new version is proven stable. This pattern applies to any breaking
schema change, not just column renames.
