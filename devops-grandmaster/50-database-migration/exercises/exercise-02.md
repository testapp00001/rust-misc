# Exercise 02: Online Schema Change with gh-ost

**Type:** Guided | **Time:** 30 min | **Difficulty:** Easy-Medium

## Objective

Learn to use gh-ost (GitHub's Online Schema Transmogrifier) to perform schema changes on a live MySQL table without downtime. gh-ost works by creating a shadow table, applying the schema change to it, then streaming changes from the original table using the binary log.

## Scenario

You need to add a `priority` column to the `orders` table in production. The table has 50M rows and receives 2,000 writes per second. The application cannot tolerate any downtime. You have been given a 4-hour maintenance window during low-traffic hours, but the goal is zero visible impact to the application.

The production MySQL instance:
- Host: `db-primary.prod.internal:3306`
- Database: `ecommerce`
- Table: `orders`
- Replicas: 2 read replicas
- Binary logging: ROW format, enabled

## Tasks

### Part A: Install and Configure gh-ost

1. Install gh-ost on your migration workstation.
2. Verify the MySQL instance meets gh-ost prerequisites:
   - Binary logging is enabled in ROW format
   - The migration user has required privileges
   - The table has a unique key
3. Create the migration user with minimum required privileges.

Write the SQL to create the migration user and the command to verify prerequisites.

<details>
<summary>Hint</summary>

gh-ost requires:
- `REPLICATION SLAVE`, `REPLICATION CLIENT`, `SUPER` (or `REPLICATION_SLAVE_ADMIN`, `REPLICATION_CLIENT_ADMIN` in MySQL 8.0)
- `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `CREATE`, `DROP`, `ALTER` on the target database
- Binary log in ROW format: `SHOW VARIABLES LIKE 'binlog_format';`

</details>

### Part B: Write the gh-ost Migration Command

Write the gh-ost command to add the `priority` column. The column should:
- Be an integer with a default value of 0
- Be added after the `status` column
- Use the `id` column as the unique key for chunking

Your command should include:
- Connection details for the primary
- The ALTER statement
- Appropriate throttling flags
- Verbose output for monitoring

<details>
<summary>Hint</summary>

The basic gh-ost command structure:

```bash
gh-ost \
  --host=<host> \
  --port=<port> \
  --user=<user> \
  --password=<password> \
  --database=<db> \
  --table=<table> \
  --alter="<ALTER STATEMENT>" \
  --assume-master-host=<master> \
  --verbose
```

For adding a column, the alter statement is: `"ADD COLUMN priority INT NOT NULL DEFAULT 0 AFTER status"`

</details>

### Part C: Monitor Migration Progress

You have launched the migration. Describe how you would:

1. Monitor the migration progress (what metrics matter?)
2. Detect if the migration is causing replication lag on replicas
3. Configure automatic throttling based on replication lag
4. Check if the migration is consuming too many resources on the primary

Write the monitoring commands and the gh-ost flags for replication-lag-aware throttling.

<details>
<summary>Hint</summary>

gh-ost provides several monitoring interfaces:
- `--hooks-path`: Directory for hook scripts executed at migration events
- `--max-lag-millis`: Auto-throttle when replica lag exceeds this threshold
- `--throttle-control-replicas`: Comma-separated list of replicas to monitor for lag
- `--chunk-size`: Number of rows processed per iteration (smaller = less load)
- `--max-load`: Throttle when status variables exceed thresholds

You can also interact with a running migration via a Unix socket file.

</details>

### Part D: Design the Cut-over and Rollback Plan

The migration has completed copying all rows and is now applying the final changes. Design:

1. The cut-over process: What happens when gh-ost swaps the tables?
2. A rollback plan: If the application starts failing after cut-over, how do you revert?
3. A pre-cut-over checklist: What should you verify before allowing the cut-over?
4. What happens to in-flight transactions during the cut-over?

<details>
<summary>Hint</summary>

gh-ost's cut-over process:
1. It locks the original table briefly
2. It renames the original table to `_orders_del` and the ghost table to `orders`
3. The lock duration is typically sub-second

gh-ost keeps the old table around (`_orders_del`) so you can rollback by renaming it back.

</details>

## Success Criteria

- [ ] gh-ost command is syntactically correct with all required flags
- [ ] Migration user has minimum required privileges (not root)
- [ ] Monitoring approach covers replication lag, progress, and resource usage
- [ ] Rollback plan is concrete and tested (not just "rename the table back")
- [ ] Cut-over strategy addresses in-flight transactions

## What You Should Understand

After completing this exercise, you should be able to:

1. Explain how gh-ost uses the binary log to capture changes during migration
2. Configure gh-ost for safe production use with appropriate throttling
3. Monitor an ongoing migration and detect problems early
4. Execute a cut-over with confidence and have a tested rollback plan
5. Understand the trade-offs between gh-ost and pt-online-schema-change
