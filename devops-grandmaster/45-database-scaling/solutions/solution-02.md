# Solution 02: Read Replica Setup with ProxySQL

## Part A: Docker Compose Configuration

```yaml
version: '3.8'

services:
  postgres-primary:
    image: postgres:15
    container_name: pg-primary
    environment:
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: secret
      POSTGRES_DB: appdb
    ports:
      - "5432:5432"
    volumes:
      - primary_data:/var/lib/postgresql/data
      - ./primary-init.sh:/docker-entrypoint-initdb.d/init.sh
    command: >
      postgres
        -c wal_level=replica
        -c max_wal_senders=5
        -c max_replication_slots=3
        -c hot_standby=on
        -c shared_preload_libraries='pg_stat_statements'
    networks:
      - pgnet

  postgres-replica:
    image: postgres:15
    container_name: pg-replica
    environment:
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: secret
    ports:
      - "5433:5432"
    volumes:
      - replica_data:/var/lib/postgresql/data
      - ./replica-init.sh:/docker-entrypoint-initdb.d/init.sh
    depends_on:
      - postgres-primary
    networks:
      - pgnet

  proxysql:
    image: proxysql/proxysql:2.6.3
    container_name: proxysql
    ports:
      - "6033:6033"
      - "6032:6032"
    volumes:
      - ./proxysql.cnf:/etc/proxysql.cnf
    depends_on:
      - postgres-primary
      - postgres-replica
    networks:
      - pgnet

volumes:
  primary_data:
  replica_data:

networks:
  pgnet:
    driver: bridge
```

The primary initialization script (`primary-init.sh`):

```bash
#!/bin/bash
set -e

# Create replication user
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD 'replpass';
    CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
EOSQL

# Create replication slot
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    SELECT * FROM pg_create_physical_replication_slot('replica_slot_1');
EOSQL
```

The replica initialization script (`replica-init.sh`):

```bash
#!/bin/bash
set -e

# Stop PostgreSQL, clear data directory, and base backup from primary
pg_ctl stop -D /var/lib/postgresql/data

rm -rf /var/lib/postgresql/data/*

PGPASSWORD=replpass pg_basebackup \
  -h postgres-primary \
  -D /var/lib/postgresql/data \
  -U replicator \
  -Fp \
  -Xs \
  -P \
  -R

# -R creates standby.signal and sets primary_conninfo automatically
# Start PostgreSQL in standby mode
pg_ctl start -D /var/lib/postgresql/data
```

## Part B: ProxySQL Configuration

```ini
datadir="/var/lib/proxysql"

admin_variables=
{
    admin_credentials="admin:admin;radmin:radmin"
    mysql_ifaces="0.0.0.0:6032"
}

mysql_variables=
{
    threads=4
    max_connections=2000
    default_query_delay=0
    default_query_timeout=36000000
    have_compress=true
    poll_timeout=2000
    interfaces="0.0.0.0:6033"
    default_schema="appdb"
    stacksize=1048576
    server_version="15.0"
    connect_timeout_server=3000
    monitor_username="monitor"
    monitor_password="monitorpass"
    monitor_history=600000
    monitor_connect_interval=60000
    monitor_ping_interval=10000
    monitor_read_only_interval=1500
    monitor_read_only_timeout=500
}

mysql_servers=
(
    {
        address="postgres-primary"
        port=5432
        hostgroup=10
        max_connections=100
        weight=1
    },
    {
        address="postgres-replica"
        port=5432
        hostgroup=20
        max_connections=200
        weight=1
    }
)

mysql_users=
(
    {
        username="admin"
        password="secret"
        default_hostgroup=10
        max_connections=2000
        default_schema="appdb"
        active=1
    }
)

mysql_query_rules=
(
    {
        rule_id=100
        active=1
        match_digest="^SELECT .* FOR UPDATE$"
        destination_hostgroup=10
        apply=1
        description="SELECT FOR UPDATE goes to primary (writer)"
    },
    {
        rule_id=200
        active=1
        match_digest="^SELECT"
        destination_hostgroup=20
        apply=1
        description="All other SELECTs go to replica (reader)"
    },
    {
        rule_id=300
        active=1
        match_digest="^INSERT|^UPDATE|^DELETE|^CREATE|^ALTER|^DROP"
        destination_hostgroup=10
        apply=1
        description="All writes go to primary (writer)"
    }
)
```

### Why This Works
- Hostgroup 10 is the writer group (primary), hostgroup 20 is the reader group (replica)
- `SELECT ... FOR UPDATE` is routed to the primary because it acquires row locks
- All other SELECTs go to the replica for read distribution
- Write operations always go to the primary
- Rule IDs determine evaluation order -- lower IDs are evaluated first

## Part C: Verification Queries

```sql
-- 1. Check replication status on the replica
SELECT
    status,
    received_lsn,
    latest_end_lsn,
    last_msg_send_time,
    last_msg_receipt_time,
    replay_lsn
FROM pg_stat_wal_receiver;

-- Expected: status = 'streaming', received_lsn close to primary's current LSN

-- 2. Check replication lag on the replica
SELECT
    CASE WHEN pg_last_wal_receive_lsn() = pg_last_wal_replay_lsn()
         THEN 0
         ELSE EXTRACT(EPOCH FROM now() - pg_last_xact_replay_timestamp())
    END AS replication_lag_seconds;

-- 3. Verify ProxySQL routing (run via ProxySQL admin interface)
SELECT
    hostgroup,
    schemaname,
    digest_text,
    count_star,
    sum_time
FROM stats_mysql_query_digest
ORDER BY count_star DESC
LIMIT 20;

-- Expected: SELECT queries in hostgroup 20, writes in hostgroup 10
```

## Common Mistakes to Avoid
- Forgetting `SELECT ... FOR UPDATE` must go to the primary -- it acquires locks
- Not creating a replication slot -- without it, the primary may purge WAL before the replica consumes it
- Using the same PostgreSQL version for primary and replica -- version mismatches can cause replication failures
- Not monitoring replication lag -- a replica that falls behind serves stale data

## Key Takeaway
Read replicas offload read traffic from the primary but require careful query routing. ProxySQL automates this with query rules based on SQL patterns. The critical detail is ensuring queries that need write access or row locks (like `SELECT ... FOR UPDATE`) always reach the primary.
