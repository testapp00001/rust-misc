# Solution 01: Migration Risk Assessment

## Part A: Classify Each Change

### 1. Adding a nullable column -- SAFE

```sql
ALTER TABLE orders ADD COLUMN notes VARCHAR(500) NULL;
```

**Classification: Safe**

In MySQL 8.0+, adding a nullable column at the end of a table is an **instant** operation. MySQL only updates the table metadata (data dictionary) without touching any rows. The lock duration is measured in milliseconds regardless of table size.

In PostgreSQL, this is also safe -- `ALTER TABLE ADD COLUMN` with a nullable column does not rewrite the table and completes nearly instantly.

### 2. Dropping a column -- RISKY

```sql
ALTER TABLE orders DROP COLUMN notes;
```

**Classification: Risky**

In MySQL, dropping a column requires `ALGORITHM=INPLACE` with a table rebuild. It acquires a metadata lock for the duration of the rebuild. For a 200M row table, this could take several minutes.

In PostgreSQL, `ALTER TABLE DROP COLUMN` only marks the column as dropped in the system catalog -- it does not rewrite the table. This is fast but the dead column data remains until `VACUUM FULL`.

### 3. Renaming a column -- RISKY

```sql
ALTER TABLE orders RENAME COLUMN email TO email_address;
```

**Classification: Risky**

In MySQL 8.0+, `RENAME COLUMN` is an instant metadata-only operation at the database level. However, it is **dangerous** from an application perspective -- every query referencing `email` will immediately fail. This is risky because of the application impact, not the database lock.

In PostgreSQL, `RENAME COLUMN` is also a metadata-only instant operation, but the application breakage risk is identical.

### 4. Adding an index -- RISKY

```sql
ALTER TABLE orders ADD INDEX idx_created_at (created_at);
```

**Classification: Risky**

In MySQL with InnoDB, adding an index uses `ALGORITHM=INPLACE` which does not copy the table but does acquire a brief metadata lock at the start and end. The index build itself is online (reads are not blocked), but the final lock swap can block writes for a few seconds.

In PostgreSQL, `CREATE INDEX` does not block reads but blocks writes. `CREATE INDEX CONCURRENTLY` avoids write locks but takes longer and cannot run inside a transaction.

### 5. Changing a column type -- DANGEROUS

```sql
ALTER TABLE orders MODIFY COLUMN status VARCHAR(100) NOT NULL;
```

**Classification: Dangerous**

In MySQL, changing a column type (even widening a VARCHAR) requires a full table copy with `ALGORITHM=COPY`. This acquires an exclusive table-level lock for the entire duration of the copy. On a 200M row table, this could take 30+ minutes.

In PostgreSQL, `ALTER COLUMN TYPE` may or may not require a table rewrite depending on the type change. Widening a VARCHAR does not require a rewrite, but changing from VARCHAR to TEXT or vice versa does.

### 6. Adding a NOT NULL constraint -- DANGEROUS

```sql
ALTER TABLE orders ALTER COLUMN notes SET NOT NULL;
```

**Classification: Dangerous**

In MySQL, adding a NOT NULL constraint requires reading every row to verify no NULL values exist, then rewriting the table. This is a full table copy operation.

In PostgreSQL, adding a NOT NULL constraint is fast if a CHECK constraint already exists that guarantees non-null values. Otherwise, PostgreSQL must scan the entire table to verify, which acquires an ACCESS EXCLUSIVE lock.

## Part B: Describe Locking Behavior

| Change | Lock Type | Duration (200M rows) | Blocked Operations | Wait Behavior |
|--------|-----------|---------------------|-------------------|---------------|
| Drop column (MySQL) | MDL Exclusive | 5-30 minutes | All reads and writes | Queries queue behind the MDL lock |
| Rename column | MDL Exclusive | Milliseconds (metadata only) | All reads and writes during the lock | Brief pause, then all queries fail if they reference the old name |
| Add index (MySQL) | MDL Shared (during build), MDL Exclusive (final swap) | 10-60 min build + seconds for swap | Writes blocked during swap | Write queries wait for the lock swap |
| Change type | MDL Exclusive + table-level lock | 30-120 minutes | All reads and writes | All queries queue for the entire migration duration |
| Add NOT NULL | MDL Exclusive | 30-120 minutes | All reads and writes | All queries queue for the entire table scan |

Key observations:
- The MDL lock in MySQL is the most dangerous because it queues all subsequent queries, even SELECT statements
- Long-running queries that were executing before the DDL will cause the DDL to wait for the MDL, and all new queries will queue behind the DDL
- This creates a "lock convoy" effect where the database appears frozen

## Part C: Propose Safe Alternatives

### Safe Alternative for: Dropping a Column

**Technique: Expand-contract with application coordination**

1. Stop deploying code that references the column (application-level contract)
2. Deploy application code that no longer reads or writes the column
3. Monitor for any remaining references to the column
4. Drop the column during a low-traffic window (instant in PostgreSQL, brief lock in MySQL 8.0+)

For MySQL specifically, if the column was added recently:
```sql
-- Check if the column can be dropped instantly
ALTER TABLE orders DROP COLUMN notes, ALGORITHM=INSTANT;
```

### Safe Alternative for: Renaming a Column

**Technique: Expand-contract (never rename directly)**

1. Add the new column `email_address`
2. Create a trigger to sync both columns
3. Backfill existing data in batches
4. Update application to use `email_address`
5. Remove the trigger and drop the old `email` column

This is detailed in Exercise 03.

### Safe Alternative for: Adding an Index

**Technique: Online index creation**

MySQL:
```sql
ALTER TABLE orders ADD INDEX idx_created_at (created_at), ALGORITHM=INPLACE, LOCK=NONE;
```

PostgreSQL:
```sql
CREATE INDEX CONCURRENTLY idx_created_at ON orders(created_at);
```

Both approaches allow reads and writes during index creation. The trade-off is that they take longer (2-5x) than a blocking index creation.

### Safe Alternative for: Changing a Column Type

**Technique: gh-ost (MySQL) or pg_repack (PostgreSQL) or expand-contract**

MySQL with gh-ost:
```bash
gh-ost \
  --host=db-primary.prod.internal \
  --port=3306 \
  --user=migration_user \
  --password=secret \
  --database=ecommerce \
  --table=orders \
  --alter="MODIFY COLUMN status VARCHAR(100) NOT NULL" \
  --assume-master-host=db-primary.prod.internal \
  --execute
```

Expand-contract approach:
1. Add a new column `status_new VARCHAR(100)`
2. Copy data from `status` to `status_new` in batches
3. Create a trigger to sync both columns
4. Update application to use `status_new`
5. Drop `status`, rename `status_new` to `status`

### Safe Alternative for: Adding a NOT NULL Constraint

**Technique: Phased constraint addition**

PostgreSQL (safest approach):
```sql
-- Step 1: Add a CHECK constraint as NOT VALID (instant, no scan)
ALTER TABLE orders ADD CONSTRAINT chk_notes_not_null
    CHECK (notes IS NOT NULL) NOT VALID;

-- Step 2: Validate the constraint (scans rows but doesn't hold ACCESS EXCLUSIVE for the full scan)
ALTER TABLE orders VALIDATE CONSTRAINT chk_notes_not_null;

-- Step 3: Now add the NOT NULL constraint (instant because PG knows the CHECK guarantees it)
ALTER TABLE orders ALTER COLUMN notes SET NOT NULL;

-- Step 4: Drop the CHECK constraint (it's redundant now)
ALTER TABLE orders DROP CONSTRAINT chk_notes_not_null;
```

MySQL with gh-ost:
```bash
gh-ost \
  --host=db-primary.prod.internal \
  --port=3306 \
  --user=migration_user \
  --password=secret \
  --database=ecommerce \
  --table=orders \
  --alter="MODIFY COLUMN notes VARCHAR(500) NOT NULL" \
  --assume-master-host=db-primary.prod.internal \
  --execute
```

## Common Mistakes to Avoid

1. **Testing on small tables**: A migration that takes 2 seconds on a 1000-row table may take 2 hours on a 200M-row table. Always test on production-sized data.

2. **Ignoring replica lag**: Even online migrations generate binlog events that replicas must apply. Monitor lag throughout.

3. **Forgetting the application**: Renaming a column at the database level without coordinating the application deploy causes instant failures.

4. **Running DDL during peak hours**: Even safe migrations generate I/O and CPU load. Schedule during low-traffic windows.

5. **Not having a rollback plan**: Every migration should have a tested rollback procedure before it runs in production.

## Key Takeaway

The most important insight from this exercise is that **the risk of a schema change is determined by its locking behavior, not its SQL complexity**. A simple `ALTER TABLE` that rewrites the entire table is far more dangerous than a complex trigger-based migration that only holds locks for milliseconds. Always classify migrations by their lock scope and duration before planning how to execute them.
