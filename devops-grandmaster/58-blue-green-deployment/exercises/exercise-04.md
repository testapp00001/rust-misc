# Exercise 04: Database Migrations in Blue-Green

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design a strategy for handling database schema migrations during blue-green
deployments. This is the hardest problem in blue-green deployment because
both environments share the same database, but each version may expect a
different schema.

## Scenario

Your `user-service` is deployed using blue-green. Version 1 (blue) uses
this database schema:

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

Version 2 (green) requires a schema change: splitting `name` into
`first_name` and `last_name`:

```sql
ALTER TABLE users ADD COLUMN first_name VARCHAR(255);
ALTER TABLE users ADD COLUMN last_name VARCHAR(255);
UPDATE users SET first_name = split_part(name, ' ', 1),
                 last_name = split_part(name, ' ', 2);
-- Eventually: ALTER TABLE users DROP COLUMN name;
```

## Tasks

### Part A: Identify the Problem

Explain why running this migration while both blue and green are live
causes problems. What happens to blue's requests if you drop the `name`
column?

<details>
<summary>Hint</summary>

Both blue (v1) and green (v2) connect to the same database. If you run
the migration before switching traffic, blue (v1) still reads and writes
the `name` column. If you drop `name`, blue crashes. If you do not drop
`name`, green's code that expects `first_name`/`last_name` may have
inconsistent data.

</details>

### Part B: Design a Safe Migration Strategy

Design a migration process that allows both versions to work with the
same database simultaneously. The migration must be backward-compatible
during the transition period.

<details>
<summary>Hint</summary>

The key insight is to use **expand and contract** migrations:

1. **Expand:** Add new columns, keep old column, add triggers or
   application-level code to keep both in sync
2. **Switch:** Deploy v2, switch traffic from blue to green
3. **Contract:** After blue is no longer serving traffic, drop the old
   column

During the expand phase, both versions can read from their preferred
columns.

</details>

### Part C: Write the Migration SQL

Write the complete migration SQL that:
1. Adds `first_name` and `last_name` columns
2. Backfills existing data
3. Creates a trigger to keep `name` and `first_name`/`last_name` in sync
4. Does NOT drop the `name` column (that comes later)

<details>
<summary>Hint</summary>

Use a PostgreSQL trigger that fires on INSERT and UPDATE:

```sql
CREATE OR REPLACE FUNCTION sync_name_columns()
RETURNS TRIGGER AS $$
BEGIN
    -- If first_name/last_name are set, update name
    IF NEW.first_name IS NOT NULL AND NEW.last_name IS NOT NULL THEN
        NEW.name := NEW.first_name || ' ' || NEW.last_name;
    -- If name is set, update first_name/last_name
    ELSIF NEW.name IS NOT NULL THEN
        NEW.first_name := split_part(NEW.name, ' ', 1);
        NEW.last_name := split_part(NEW.name, ' ', 2);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

</details>

### Part D: Design the Full Deployment Sequence

Write the complete sequence of operations for deploying v2 with the
database migration. Include every step from "blue is running v1" to
"green is running v2 and blue is idle."

<details>
<summary>Hint</summary>

The sequence should be:
1. Run the expand migration (add columns, backfill, create trigger)
2. Deploy v2 to green environment
3. Wait for green to pass health checks
4. Switch traffic from blue to green
5. Verify green is serving correctly
6. (Later, after confidence period) Run the contract migration (drop old column)

</details>

## Success Criteria

- [ ] You can explain why naive migration breaks blue during the transition
- [ ] Your expand phase keeps both columns in sync with a trigger
- [ ] Your migration SQL backfills existing data without downtime
- [ ] Your deployment sequence includes the expand-migrate-switch-contract pattern
- [ ] You understand why the contract phase (dropping old columns) happens last

## What You Should Understand After This Exercise

Database migrations in blue-green deployments require the **expand and
contract** pattern. You never make a breaking schema change during the
transition. First expand (add new columns, keep old), then switch traffic,
then contract (drop old columns). This ensures both versions can work
with the same database simultaneously.
