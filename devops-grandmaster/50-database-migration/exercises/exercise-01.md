# Exercise 01: Migration Risk Assessment

**Type:** Conceptual | **Time:** 15 min | **Difficulty:** Easy

## Objective

Classify database schema changes by risk level and understand the locking behavior of each change type. This skill is the foundation of safe database operations -- you must know which changes are dangerous before you can plan safe migrations.

## Scenario

You are the database engineer for an e-commerce platform processing 5,000 transactions per second. The application team has submitted six schema change requests that need to be applied to the production `orders` table (200M rows). You need to assess the risk of each change before scheduling them.

The current table definition:

```sql
CREATE TABLE orders (
    id          BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id     BIGINT NOT NULL,
    email       VARCHAR(255) NOT NULL,
    total       DECIMAL(10,2) NOT NULL,
    status      VARCHAR(50) NOT NULL,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_user_id (user_id),
    INDEX idx_status (status)
);
```

## Tasks

### Part A: Classify Each Change

For each of the following schema changes, classify it as **Safe** (no blocking risk), **Risky** (may cause brief locks), or **Dangerous** (will block the table for an extended period). Explain your reasoning.

1. **Adding a nullable column:**
   ```sql
   ALTER TABLE orders ADD COLUMN notes VARCHAR(500) NULL;
   ```

2. **Dropping a column:**
   ```sql
   ALTER TABLE orders DROP COLUMN notes;
   ```

3. **Renaming a column:**
   ```sql
   ALTER TABLE orders RENAME COLUMN email TO email_address;
   ```

4. **Adding an index:**
   ```sql
   ALTER TABLE orders ADD INDEX idx_created_at (created_at);
   ```

5. **Changing a column type:**
   ```sql
   ALTER TABLE orders MODIFY COLUMN status VARCHAR(100) NOT NULL;
   ```

6. **Adding a NOT NULL constraint:**
   ```sql
   ALTER TABLE orders ALTER COLUMN notes SET NOT NULL;
   ```

For each classification, consider:
- Does the operation require a table copy?
- Does it acquire a metadata lock?
- How does the lock duration scale with table size?

<details>
<summary>Hint</summary>

In MySQL (InnoDB), operations fall into categories:
- **Instant** (metadata only, no table rebuild): Adding a nullable column at the end of a table (MySQL 8.0+).
- **In-place** (no table copy, but may block): Adding an index (with ALGORITHM=INPLACE).
- **Copy** (full table rebuild): Changing column types, many constraint changes.

PostgreSQL has different behavior -- most ALTER TABLE operations acquire an ACCESS EXCLUSIVE lock but complete quickly if no table rewrite is needed.

</details>

### Part B: Describe Locking Behavior

For each change classified as **Risky** or **Dangerous** in Part A, describe:

1. What type of lock does the operation acquire?
2. How long will the lock be held on a 200M row table?
3. What operations will be blocked during the lock (reads, writes, or both)?
4. What happens to queries that are waiting for the lock?

Create a summary table:

| Change | Lock Type | Duration | Blocked Operations | Wait Behavior |
|--------|-----------|----------|-------------------|---------------|
| ... | ... | ... | ... | ... |

<details>
<summary>Hint</summary>

MySQL InnoDB uses several lock levels:
- **MDL (Metadata Lock)**: Acquired on any table access. DDL needs an exclusive MDL.
- **Table-level lock**: For full table copies.
- **Row-level lock**: For DML operations.

The key insight: even "fast" DDL operations block if there are long-running queries holding a conflicting MDL.

</details>

### Part C: Propose Safe Alternatives

For each change classified as **Dangerous** in Part A, propose a safe alternative approach that achieves the same result without extended downtime. Your alternatives should:

1. Avoid holding locks that block production traffic
2. Be reversible (you can roll back if something goes wrong)
3. Include a brief description of the tool or technique used

Consider these techniques:
- Online schema change tools (gh-ost, pt-online-schema-change)
- Expand-and-contract pattern
- Shadow table and trigger-based migration
- Application-level dual-write

<details>
<summary>Hint</summary>

The expand-contract pattern works in three phases:
1. **Expand**: Add the new column/constraint alongside the old one
2. **Migrate**: Copy data from old to new, update application to use new
3. **Contract**: Remove the old column/constraint

This is the gold standard for zero-downtime migrations.

</details>

## Success Criteria

- [ ] All six changes are classified with correct risk levels
- [ ] Locking behavior descriptions include lock type and duration estimates
- [ ] Safe alternatives avoid extended table locks
- [ ] Alternatives are reversible where possible
- [ ] Summary table is complete and accurate

## What You Should Understand

After completing this exercise, you should be able to:

1. Look at any DDL statement and immediately assess its risk level
2. Explain why "just run the ALTER TABLE" is dangerous advice in production
3. Describe the difference between instant, in-place, and copy operations in MySQL
4. Know when to reach for online schema change tools vs. the expand-contract pattern
5. Understand that PostgreSQL and MySQL have fundamentally different DDL locking models
