# Exercise 04: Large Table Migration Strategy

**Type:** Challenge | **Time:** 45 min | **Difficulty:** Medium-Hard

## Objective

Design a migration strategy for a 500GB table that cannot afford more than 5 minutes of downtime. This requires evaluating multiple approaches, understanding their trade-offs, and designing a process that handles edge cases like foreign key constraints and data consistency verification.

## Scenario

The `orders` table in your PostgreSQL production database is 500GB with 2 billion rows. The current schema needs significant restructuring:

Current schema:
```sql
CREATE TABLE orders (
    id           BIGSERIAL PRIMARY KEY,
    user_id      BIGINT NOT NULL REFERENCES users(id),
    product_id   BIGINT NOT NULL REFERENCES products(id),
    quantity     INTEGER NOT NULL,
    unit_price   DECIMAL(10,2) NOT NULL,
    total_price  DECIMAL(10,2) NOT NULL,
    status       VARCHAR(50) NOT NULL,
    created_at   TIMESTAMPTZ DEFAULT NOW(),
    updated_at   TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);
```

Target schema:
```sql
CREATE TABLE orders (
    id           BIGSERIAL PRIMARY KEY,
    user_id      BIGINT NOT NULL REFERENCES users(id),
    items        JSONB NOT NULL,          -- replaces product_id, quantity, unit_price
    total_price  DECIMAL(10,2) NOT NULL,
    status       VARCHAR(50) NOT NULL,
    shipping_id  BIGINT REFERENCES shipments(id),  -- new column
    created_at   TIMESTAMPTZ DEFAULT NOW(),
    updated_at   TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);
CREATE INDEX idx_orders_items ON orders USING GIN (items);
```

The changes are:
1. Replace `product_id`, `quantity`, `unit_price` with a single `items` JSONB column
2. Add a new `shipping_id` foreign key
3. Add a GIN index on the new `items` column
4. The table has 3 foreign keys pointing to it from other tables

Constraints:
- Maximum 5 minutes of downtime for the final switchover
- 24/7 traffic with 3,000 writes/second during peak hours
- 2 read replicas must stay within 10 seconds of lag
- 500GB of available disk space (the database server has 1TB total)

## Tasks

### Part A: Evaluate Migration Approaches

Evaluate each of the following approaches for this migration. For each, describe:
- Feasibility given the constraints
- Expected downtime
- Risk level
- Resource requirements (disk, CPU, replication impact)

1. **Direct ALTER TABLE**: Run the migration directly on the live table.

2. **pg_repack / pg_squeeze**: Online table reorganization tools for PostgreSQL.

3. **Logical replication**: Create the new table, replicate changes via logical replication slots.

4. **Create-and-copy with triggers**: Create the new table, use triggers to capture changes, batch-copy historical data.

5. **Application-level dual-write**: Modify the application to write to both old and new tables during migration.

Create a comparison matrix:

| Approach | Downtime | Risk | Disk Required | Replication Impact | Complexity |
|----------|----------|------|---------------|-------------------|------------|
| ... | ... | ... | ... | ... | ... |

<details>
<summary>Hint</summary>

Consider these factors:
- PostgreSQL's ACCESS EXCLUSIVE lock for ALTER TABLE means direct migration blocks all access
- pg_repack can reorganize tables online but cannot change column types or structures
- Logical replication has limitations with DDL changes and large initial syncs
- Triggers add write overhead but allow online migration
- Dual-write requires application changes and has a consistency risk window

For 500GB with structural changes (column types changing), trigger-based migration is often the most practical approach.

</details>

### Part B: Design the Batch Migration Process

Design the batch migration process for the create-and-copy approach. Include:

1. The new table creation with the target schema
2. A batch copy strategy that processes the 2 billion rows without overwhelming the system
3. Progress tracking (how to know how far along you are)
4. How to handle rows that are being updated during the copy
5. Resource throttling to prevent impacting production traffic

Write the SQL and shell scripts for the batch process.

<details>
<summary>Hint</summary>

For batch copying, use the primary key to paginate:

```sql
-- Copy in batches of 10,000
INSERT INTO orders_new (id, user_id, items, total_price, status, created_at, updated_at)
SELECT
    id, user_id,
    jsonb_build_object(
        'product_id', product_id,
        'quantity', quantity,
        'unit_price', unit_price
    ),
    total_price, status, created_at, updated_at
FROM orders
WHERE id > $last_id
ORDER BY id
LIMIT 10000;
```

Track progress with:
```sql
SELECT
    (SELECT MAX(id) FROM orders_new) as copied_up_to,
    (SELECT MAX(id) FROM orders) as total_rows,
    pg_size_pretty(pg_total_relation_size('orders')) as original_size;
```

</details>

### Part C: Handle Foreign Key Constraints

The `orders` table has 3 foreign keys pointing to it:
- `order_items.order_id REFERENCES orders(id)`
- `payments.order_id REFERENCES orders(id)`
- `order_history.order_id REFERENCES orders(id)`

Design the process to handle these foreign keys during migration:

1. How do you create the new table without breaking existing foreign keys?
2. When and how do you switch the foreign keys to point to the new table?
3. What happens if a row is inserted into `order_items` during the switchover?
4. How do you ensure referential integrity is maintained throughout?

<details>
<summary>Hint</summary>

Options for foreign key handling:
- **Approach 1**: Keep the same table name, use a rename swap
- **Approach 2**: Use a view that unions both tables during transition
- **Approach 3**: Disable foreign keys temporarily (dangerous)

The rename swap approach:
1. Create `orders_new` with the new schema
2. Copy data
3. In a single transaction: rename `orders` to `orders_old`, rename `orders_new` to `orders`
4. Recreate foreign keys pointing to the new `orders`
5. This is the 5-minute downtime window

</details>

### Part D: Design the Verification Process

Design a verification process that runs after the migration to ensure data consistency:

1. Row count verification
2. Data integrity checks (no corruption during JSONB conversion)
3. Foreign key integrity verification
4. Index verification (GIN index is correct)
5. Application-level smoke tests

Write the SQL queries for each verification step.

<details>
<summary>Hint</summary>

For JSONB integrity verification:

```sql
-- Verify all items have required fields
SELECT COUNT(*) FROM orders_new
WHERE NOT (items ? 'product_id' AND items ? 'quantity' AND items ? 'unit_price');

-- Verify total_price consistency
SELECT COUNT(*) FROM orders_new
WHERE total_price != (
    (items->>'quantity')::int * (items->>'unit_price')::decimal
);
```

</details>

## Success Criteria

- [ ] At least 3 migration approaches are evaluated with honest trade-off analysis
- [ ] Batch copy strategy processes rows without locking the entire table
- [ ] Progress tracking shows real-time migration status
- [ ] Foreign key handling maintains referential integrity throughout
- [ ] Verification process catches data corruption and missing rows
- [ ] Total downtime for switchover is under 5 minutes

## What You Should Understand

After completing this exercise, you should be able to:

1. Evaluate migration approaches based on table size, traffic, and constraints
2. Design batch processes that handle billions of rows without overwhelming the database
3. Manage foreign key constraints during table swaps
4. Build verification processes that catch data corruption
5. Understand why large table migrations require more planning than coding
