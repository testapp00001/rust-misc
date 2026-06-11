# Solution 02: Online Schema Change with gh-ost

## Part A: Install and Configure gh-ost

### Install gh-ost

```bash
# Linux (amd64)
wget https://github.com/github/gh-ost/releases/latest/download/gh-ost-binary-linux-amd64.tar.gz
tar xzf gh-ost-binary-linux-amd64.tar.gz
sudo mv gh-ost /usr/local/bin/

# macOS
brew install gh-ost

# Verify installation
gh-ost --version
```

### Create the Migration User

```sql
-- Connect as root or a user with GRANT privileges
CREATE USER 'gh_ost_user'@'%' IDENTIFIED BY 'use-a-strong-password-here';

-- Required privileges for gh-ost
GRANT REPLICATION SLAVE, REPLICATION CLIENT ON *.* TO 'gh_ost_user'@'%';
GRANT SELECT, INSERT, UPDATE, DELETE, CREATE, DROP, ALTER, INDEX ON ecommerce.* TO 'gh_ost_user'@'%';

-- MySQL 8.0+ alternative (more granular)
GRANT REPLICATION_SLAVE_ADMIN, REPLICATION_CLIENT_ADMIN ON *.* TO 'gh_ost_user'@'%';

FLUSH PRIVILEGES;
```

### Verify Prerequisites

```bash
# Check binary log format (must be ROW)
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "SHOW VARIABLES LIKE 'binlog_format';"
# Expected output: ROW

# Check binary log is enabled
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "SHOW VARIABLES LIKE 'log_bin';"
# Expected output: ON

# Check the table has a unique key
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "
  SHOW INDEX FROM ecommerce.orders WHERE Non_unique = 0;
"
# Expected: at least one unique key (PRIMARY KEY counts)

# Check server-id is set (required for binlog reading)
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "SHOW VARIABLES LIKE 'server_id';"
# Expected: a non-zero value
```

## Part B: Write the gh-ost Migration Command

```bash
gh-ost \
  --host=db-primary.prod.internal \
  --port=3306 \
  --user=gh_ost_user \
  --password="use-a-strong-password-here" \
  --database=ecommerce \
  --table=orders \
  --alter="ADD COLUMN priority INT NOT NULL DEFAULT 0 AFTER status" \
  --assume-master-host=db-primary.prod.internal \
  --allow-master-master \
  --chunk-size=1000 \
  --max-lag-millis=1000 \
  --throttle-control-replicas=replica1.prod.internal:3306,replica2.prod.internal:3306 \
  --max-load="Threads_running=25" \
  --initially-drop-ghost-table \
  --initially-drop-old-table \
  --verbose \
  --execute
```

Flag explanations:

| Flag | Purpose |
|------|---------|
| `--assume-master-host` | Tells gh-ost which host is the master (needed when connecting through a proxy) |
| `--allow-master-master` | Allows running in master-master topology |
| `--chunk-size` | Rows processed per iteration (1000 is conservative for high-traffic tables) |
| `--max-lag-millis` | Auto-throttle when any replica lag exceeds 1 second |
| `--throttle-control-replicas` | Specific replicas to check for lag |
| `--max-load` | Throttle when MySQL status variable exceeds threshold |
| `--initially-drop-ghost-table` | Drop the ghost table if it exists from a previous failed run |
| `--initially-drop-old-table` | Drop the old table if it exists from a previous run |
| `--execute` | Actually run the migration (without this, gh-ost only inspects) |

### Dry Run First

Always do a dry run before executing:

```bash
# Without --execute, gh-ost validates the migration but doesn't run it
gh-ost \
  --host=db-primary.prod.internal \
  --port=3306 \
  --user=gh_ost_user \
  --password="use-a-strong-password-here" \
  --database=ecommerce \
  --table=orders \
  --alter="ADD COLUMN priority INT NOT NULL DEFAULT 0 AFTER status" \
  --assume-master-host=db-primary.prod.internal \
  --verbose
```

## Part C: Monitor Migration Progress

### gh-ost Output Monitoring

gh-ost prints progress to stdout by default:

```
Copy: 12500000/50000000 25.0%; Applied: 15000; Backlog: 0/1000; Time: 2m30s(total), 2m30s(copy); streamer: mysql-bin.000042:123456789; Lag: 0.5s, State: migrating; ETA: 7m30s
```

Key metrics:
- **Copy progress**: `12500000/50000000 25.0%` -- rows copied vs total rows
- **Applied changes**: `15000` -- binlog events applied to the ghost table
- **Backlog**: `0/1000` -- pending binlog events (high backlog = migration can't keep up)
- **Lag**: `0.5s` -- replication lag on the controlled replicas
- **ETA**: Estimated time to completion

### Unix Socket Interaction

gh-ost creates a Unix socket file for runtime interaction:

```bash
# The socket file is created in /tmp by default
SOCKET=/tmp/gh-ost.ecommerce.orders.sock

# Check current status
echo "status" | nc -U $SOCKET

# Throttle the migration (reduce load)
echo "throttle" | nc -U $SOCKET

# Resume after throttle
echo "no-throttle" | nc -U $SOCKET

# Force cut-over (skip remaining backlog, dangerous)
echo "cut-over" | nc -U $SOCKET

# Abort the migration
echo "abort" | nc -U $SOCKET
```

### Replication Lag Monitoring

```bash
# Monitor replica lag continuously
watch -n 1 'mysql -h replica1.prod.internal -u monitoring_user -p -e "SHOW SLAVE STATUS\G" | grep -E "Seconds_Behind_Master|Slave_SQL_Running"'
```

```sql
-- On each replica, check lag
SHOW SLAVE STATUS\G
-- Look for: Seconds_Behind_Master

-- MySQL 8.0.22+ uses:
SHOW REPLICA STATUS\G
```

### Resource Monitoring

```bash
# Monitor MySQL process list for gh-ost connections
mysql -h db-primary.prod.internal -u monitoring_user -p -e "
  SELECT id, user, host, db, command, time, state, info
  FROM information_schema.processlist
  WHERE user = 'gh_ost_user'
  ORDER BY time DESC;
"

# Monitor InnoDB row operations
mysql -h db-primary.prod.internal -u monitoring_user -p -e "
  SHOW GLOBAL STATUS LIKE 'Innodb_rows_%';
"
```

### Throttling Configuration

gh-ost throttles automatically when:
- Replica lag exceeds `--max-lag-millis`
- MySQL status variables exceed `--max-load` thresholds
- A throttle flag file exists: `--throttle-flag-file=/tmp/gh-ost-throttle`
- A replication-related query is running: `--throttle-additional-flag-file`
- You explicitly send "throttle" via the socket

Best practice for production:

```bash
gh-ost \
  --host=db-primary.prod.internal \
  --port=3306 \
  --user=gh_ost_user \
  --password="use-a-strong-password-here" \
  --database=ecommerce \
  --table=orders \
  --alter="ADD COLUMN priority INT NOT NULL DEFAULT 0 AFTER status" \
  --assume-master-host=db-primary.prod.internal \
  --chunk-size=500 \
  --max-lag-millis=500 \
  --throttle-control-replicas=replica1.prod.internal:3306,replica2.prod.internal:3306 \
  --max-load="Threads_running=15" \
  --nice-ratio=0.25 \
  --execute
```

The `--nice-ratio=0.25` adds a 25% pause between chunks to reduce CPU impact.

## Part D: Design the Cut-over and Rollback Plan

### Cut-over Process

gh-ost's cut-over works in these steps:

1. gh-ost finishes copying all rows and catches up on the binlog backlog
2. gh-ost acquires a brief lock on the original table
3. gh-ost renames the original table to `_orders_del`
4. gh-ost renames the ghost table to `orders`
5. gh-ost releases the lock
6. The application is now using the new table with the `priority` column

The lock duration during cut-over is typically **sub-second** because:
- All rows are already copied
- The binlog backlog is caught up
- Only the rename operations need the lock

### Pre-cut-over Checklist

```bash
# 1. Verify migration is complete (all rows copied, backlog empty)
echo "status" | nc -U /tmp/gh-ost.ecommerce.orders.sock
# Look for: "Copy: 50000000/50000000 100.0%" and "Backlog: 0/1000"

# 2. Check replica lag is minimal
mysql -h replica1.prod.internal -u monitoring_user -p -e "SHOW REPLICA STATUS\G" | grep Seconds_Behind_Source

# 3. Verify no long-running transactions on the primary
mysql -h db-primary.prod.internal -u monitoring_user -p -e "
  SELECT id, user, time, state, info
  FROM information_schema.processlist
  WHERE command != 'Sleep' AND time > 60
  ORDER BY time DESC;
"

# 4. Check that the ghost table has the expected column
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "
  DESCRIBE ecommerce._orders_gho;
"

# 5. Verify row counts match
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "
  SELECT
    (SELECT COUNT(*) FROM ecommerce.orders) as original_rows,
    (SELECT COUNT(*) FROM ecommerce._orders_gho) as ghost_rows;
"
```

### Rollback Plan

If the application fails after cut-over:

```bash
# Step 1: Stop the application from writing to the table
# (scale down the application or enable maintenance mode)

# Step 2: Swap the tables back
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "
  RENAME TABLE ecommerce.orders TO ecommerce.orders_failed,
               ecommerce._orders_del TO ecommerce.orders;
"

# Step 3: Verify the original table is back
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "
  DESCRIBE ecommerce.orders;
  SELECT COUNT(*) FROM ecommerce.orders;
"

# Step 4: Restart the application

# Step 5: Investigate what went wrong before retrying
```

**Important**: gh-ost does NOT automatically clean up the old table (`_orders_del`). This is intentional -- it gives you a rollback window. Only drop it after you are confident the migration succeeded:

```bash
# Wait at least 24 hours, then clean up
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "
  DROP TABLE IF EXISTS ecommerce._orders_del;
"
```

### Rollback During Migration (Before Cut-over)

If you need to abort the migration before cut-over:

```bash
# Send abort via socket
echo "abort" | nc -U /tmp/gh-ost.ecommerce.orders.sock

# Or kill the gh-ost process
kill $(pgrep -f "gh-ost.*ecommerce.*orders")

# gh-ost will leave behind: _orders_gho (ghost table) and _orders_ghc (changelog table)
# Clean them up:
mysql -h db-primary.prod.internal -u gh_ost_user -p -e "
  DROP TABLE IF EXISTS ecommerce._orders_gho, ecommerce._orders_ghc;
"
```

## Common Mistakes to Avoid

1. **Not testing on staging first**: Always run the exact same gh-ost command on staging with production-sized data before production. The chunk-size and throttle settings that work on a 1000-row table may be wrong for a 50M-row table.

2. **Using `--initially-drop-old-table` carelessly**: This flag drops the old table from a previous migration. If a previous migration's old table contains data you need for rollback, this flag will destroy it.

3. **Ignoring the backlog metric**: If the backlog is consistently high, gh-ost cannot keep up with write traffic. Reduce chunk-size or increase throttling.

4. **Cut-over during peak traffic**: Even a sub-second lock can cause query queuing during peak traffic. Schedule cut-over during a low-traffic window if possible.

5. **Not monitoring replica lag**: gh-ost generates binlog events that replicas must apply. If replicas fall too far behind, read traffic may see stale data.

## Key Takeaway

gh-ost works by creating a shadow table, copying data in chunks, and streaming binlog changes to keep the shadow table in sync. The cut-over is a brief rename operation. The key to safe production use is proper throttling configuration based on replica lag and server load, combined with a tested rollback plan that exploits the fact that gh-ost preserves the original table under a different name.
