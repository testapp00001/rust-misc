# Exercise 03: Zero-Downtime Column Migration

**Type:** Independent | **Time:** 30 min | **Difficulty:** Medium

## Objective

Apply the expand-contract pattern to rename a column in a production table with 100M rows and zero downtime. This is one of the most common and most dangerous migrations -- a simple `RENAME COLUMN` will break every query referencing the old name.

## Scenario

The `users` table in your production database has a column called `email`. The application team wants to rename it to `email_address` to match a new naming convention. The table has 100M rows and serves 10,000 reads per second and 500 writes per second across 20 application instances. You cannot take the application offline.

Current table:

```sql
CREATE TABLE users (
    id         BIGINT PRIMARY KEY AUTO_INCREMENT,
    username   VARCHAR(100) NOT NULL,
    email      VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY idx_email (email)
);
```

Application queries that reference `email`:
```sql
-- Read queries
SELECT id, username, email FROM users WHERE id = ?;
SELECT * FROM users WHERE email = ?;

-- Write queries
INSERT INTO users (username, email) VALUES (?, ?);
UPDATE users SET email = ? WHERE id = ?;
```

## Tasks

### Part A: Design the Multi-Step Migration Process

Design a complete migration plan using the expand-contract pattern. Your plan must include:

1. The order of operations (what happens first, second, third)
2. Which steps require application code changes
3. Which steps can be rolled back safely
4. How long each step takes (estimate for 100M rows)
5. The total timeline from start to cleanup

Create a migration plan table:

| Step | Action | Requires Deploy | Reversible | Estimated Duration |
|------|--------|-----------------|------------|-------------------|
| ... | ... | ... | ... | ... |

<details>
<summary>Hint</summary>

The expand-contract pattern for column rename:
1. **Expand**: Add the new column, set up data synchronization
2. **Transition**: Update application to write to both columns, read from new
3. **Backfill**: Copy all data from old column to new column
4. **Switch**: Update application to only use new column
5. **Contract**: Remove the old column

The key insight: at no point should both the old and new column be out of sync for more than a few seconds.

</details>

### Part B: Write the SQL for Each Step

Write the complete SQL migration for each step of your plan. Include:

1. The ALTER TABLE statement to add the new column
2. A trigger to keep both columns in sync during the transition period
3. The backfill query to copy existing data (must be batched for 100M rows)
4. The ALTER TABLE statement to add the unique constraint on the new column
5. The final cleanup to remove the old column and its index

Each SQL statement should be annotated with comments explaining when it runs and who executes it (migration script vs. application vs. manual).

<details>
<summary>Hint</summary>

For the backfill, you need to process rows in batches:

```sql
-- Process 10,000 rows at a time
UPDATE users SET email_address = email
WHERE email_address IS NULL
LIMIT 10000;
```

Repeat until no rows remain with NULL in `email_address`.

For the trigger in MySQL:

```sql
CREATE TRIGGER sync_email_columns
BEFORE INSERT ON users
FOR EACH ROW
SET NEW.email_address = NEW.email;
```

</details>

### Part C: Handle the Transition Period

During the transition period, both `email` and `email_address` columns exist and must stay in sync. Design:

1. The trigger logic for INSERT operations (both columns get the value)
2. The trigger logic for UPDATE operations (changing email must update email_address)
3. How to handle the application deploy that reads from the new column
4. A monitoring query to verify both columns are in sync

Write the complete trigger definitions and the sync verification query.

<details>
<summary>Hint</summary>

You need separate triggers for INSERT and UPDATE:

```sql
-- INSERT trigger
CREATE TRIGGER sync_email_insert
BEFORE INSERT ON users
FOR EACH ROW
SET NEW.email_address = NEW.email;

-- UPDATE trigger
CREATE TRIGGER sync_email_update
BEFORE UPDATE ON users
FOR EACH ROW
SET NEW.email_address = NEW.email;
```

The verification query should find rows where the columns differ:
```sql
SELECT COUNT(*) FROM users WHERE email != email_address;
```

</details>

### Part D: Write the Cleanup Migration

Write the final migration that:
1. Verifies all data is in sync before proceeding
2. Drops the sync triggers
3. Drops the old `email` column
4. Drops the old unique index on `email`
5. Renames the new unique index if needed

This migration must be idempotent (safe to run multiple times) and must verify preconditions before making destructive changes.

<details>
<summary>Hint</summary>

Always check before dropping:

```sql
-- Verify sync before cleanup
SET @mismatched = (SELECT COUNT(*) FROM users WHERE email != email_address);
-- Only proceed if @mismatched = 0

-- Use IF EXISTS for idempotency
DROP TRIGGER IF EXISTS sync_email_insert;
DROP TRIGGER IF EXISTS sync_email_update;
```

</details>

## Success Criteria

- [ ] Migration plan has at least 4 distinct steps with clear ordering
- [ ] SQL for each step is syntactically correct
- [ ] Triggers correctly keep both columns in sync for INSERT and UPDATE
- [ ] Backfill is batched (not a single UPDATE on 100M rows)
- [ ] Cleanup migration verifies preconditions before destructive operations
- [ ] The old `email` unique constraint is replaced by one on `email_address`

## What You Should Understand

After completing this exercise, you should be able to:

1. Design expand-contract migrations for any column rename scenario
2. Write triggers that keep old and new columns in sync during transitions
3. Batch large backfill operations to avoid overwhelming the database
4. Write idempotent cleanup migrations that verify preconditions
5. Understand why a direct RENAME COLUMN is never acceptable in production
