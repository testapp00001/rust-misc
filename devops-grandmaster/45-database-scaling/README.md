# Module 45: Database Scaling

**Phase 6: Database Operations**
**Previous:** [Module 44: Dashboard Design](../44-dashboard-design/README.md)
**Next:** [Module 46: Replication](../46-replication/README.md)

---

## The Problem

Your application is growing. Users are increasing. Queries that took 10ms now take 500ms. Your database CPU is pinned at 95%. The database is the bottleneck, and every other service is waiting on it.

Database scaling is the first real scaling challenge most teams face because:

- Databases are stateful (unlike stateless app servers)
- You cannot just "add more databases" like you add more API servers
- Data consistency requirements limit how you can distribute load
- A single poorly-written query can bring down the entire system

---

## The Naive Way

**"Just throw hardware at it"**

```bash
# The naive approach: vertical scaling
# Upgrade from db.t3.medium to db.r5.24xlarge
# Cost: $50/month → $15,000/month
aws rds modify-db-instance \
  --db-instance-identifier mydb \
  --db-instance-class db.r5.24xlarge \
  --apply-immediately
```

**Why this fails:**
- Vertical scaling has hard limits (max CPU, max RAM, max disk)
- Cost grows exponentially, not linearly
- Single point of failure remains
- One bad query still brings down the bigger machine
- You are renting a Ferrari to drive to the grocery store

**"Add indexes to everything"**

```sql
-- Index every column! What could go wrong?
CREATE INDEX idx_users_name ON users(name);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created ON users(created_at);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_name_email_status ON users(name, email, status);
-- ... 47 more indexes
```

**Why this fails:**
- Every index slows down writes (INSERT, UPDATE, DELETE)
- Indexes consume disk space and memory
- The query planner may choose the wrong index
- Maintenance overhead (VACUUM, REINDEX) grows
- You have traded write performance for read performance

---

## The Right Way

### 1. Understand Your Query Patterns First

```sql
-- PostgreSQL: Find your slow queries
SELECT
    query,
    calls,
    total_time,
    mean_time,
    rows
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 20;

-- Enable pg_stat_statements in postgresql.conf
-- shared_preload_libraries = 'pg_stat_statements'
-- pg_stat_statements.track = all
```

### 2. Use EXPLAIN ANALYZE Before Adding Indexes

```sql
-- The RIGHT way to understand query performance
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT u.name, COUNT(o.id) as order_count
FROM users u
LEFT JOIN orders o ON o.user_id = u.id
WHERE u.created_at > '2024-01-01'
GROUP BY u.id
ORDER BY order_count DESC
LIMIT 10;

-- Example output:
-- Sort  (cost=15234.56..15234.78 rows=100)
--   Sort Key: (count(o.id)) DESC
--   ->  HashAggregate  (cost=15230.12..15232.34 rows=100)
--         ->  Hash Left Join  (cost=4567.89..14230.12 rows=50000)
--               Hash Cond: (u.id = o.user_id)
--               ->  Index Scan using idx_users_created on users u
--                     Index Cond: (created_at > '2024-01-01')
--               ->  Hash  (cost=3456.78..3456.78 rows=50000)
--                     ->  Seq Scan on orders o
```

**What to look for:**
- `Seq Scan` on large tables = missing index
- `cost` numbers = relative expense
- `rows` vs actual rows = stale statistics
- `Buffers: shared hit/read` = cache efficiency

### 3. Connection Pooling

**Why connection pooling matters:**

```
Without pooling:
  App Server 1 → 50 connections → PostgreSQL
  App Server 2 → 50 connections → PostgreSQL
  App Server 3 → 50 connections → PostgreSQL
  Total: 150 connections (PostgreSQL default max: 100)

With pooling:
  App Server 1 → Pooler (20 connections) → PostgreSQL
  App Server 2 → Pooler (20 connections) → PostgreSQL
  App Server 3 → Pooler (20 connections) → PostgreSQL
  Total: 20 connections (serving 150 app connections)
```

**PgBouncer Configuration:**

```ini
; /etc/pgbouncer/pgbouncer.ini

[databases]
mydb = host=127.0.0.1 port=5432 dbname=mydb

[pgbouncer]
listen_addr = 0.0.0.0
listen_port = 6432
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt

; Pool modes:
; session  - connection returned when client disconnects
; transaction - connection returned after transaction (RECOMMENDED)
; statement - connection returned after each statement (dangerous)
pool_mode = transaction

; Connection limits
default_pool_size = 20        ; connections per user/database pair
max_client_conn = 1000        ; max incoming connections
min_pool_size = 5             ; keep warm connections
reserve_pool_size = 5         ; extra connections for bursts
reserve_pool_timeout = 3      ; seconds before using reserve pool

; Timeouts
server_idle_timeout = 300     ; close idle server connections
client_idle_timeout = 0       ; 0 = no timeout
query_timeout = 0             ; 0 = no timeout (set in production!)
```

**ProxySQL (for MySQL/MariaDB):**

```sql
-- ProxySQL admin interface
-- Add backend MySQL servers
INSERT INTO mysql_servers (hostgroup_id, hostname, port, weight)
VALUES
  (10, 'mysql-primary', 3306, 100),    -- writes
  (20, 'mysql-replica-1', 3306, 50),   -- reads
  (20, 'mysql-replica-2', 3306, 50);   -- reads

-- Route reads to replicas, writes to primary
INSERT INTO mysql_query_rules (rule_id, match_pattern, destination_hostgroup)
VALUES
  (1, '^SELECT .* FOR UPDATE$', 10),   -- SELECT FOR UPDATE → primary
  (2, '^SELECT', 20);                   -- Regular SELECT → replicas

LOAD MYSQL SERVERS TO RUNTIME;
LOAD MYSQL QUERY RULES TO RUNTIME;
SAVE MYSQL SERVERS TO DISK;
SAVE MYSQL QUERY RULES TO DISK;
```

### 4. Read Replicas

```bash
# AWS RDS: Create a read replica
aws rds create-db-instance-read-replica \
  --db-instance-identifier mydb-replica-1 \
  --source-db-instance-identifier mydb \
  --db-instance-class db.r5.large \
  --availability-zone us-east-1b

# Application code: route reads to replica
```

```python
# Python example with SQLAlchemy
from sqlalchemy import create_engine

# Write operations → primary
primary_engine = create_engine(
    'postgresql://user:pass@primary-db:5432/mydb',
    pool_size=20,
    max_overflow=10
)

# Read operations → replica
replica_engine = create_engine(
    'postgresql://user:pass@replica-db:5432/mydb',
    pool_size=30,
    max_overflow=20
)

def get_user(user_id):
    # Reads go to replica
    return replica_engine.execute(
        "SELECT * FROM users WHERE id = %s", user_id
    ).fetchone()

def update_user(user_id, data):
    # Writes go to primary
    primary_engine.execute(
        "UPDATE users SET name = %s WHERE id = %s",
        data['name'], user_id
    )
```

### 5. Query Optimization

```sql
-- BEFORE: Slow query (2.5 seconds)
SELECT * FROM orders
WHERE user_id = 12345
AND status = 'completed'
ORDER BY created_at DESC;

-- ANALYZE reveals: sequential scan on 10M row table

-- AFTER: Add targeted index
CREATE INDEX idx_orders_user_status_created
ON orders (user_id, status, created_at DESC);

-- Now the query uses the index: 2.5s → 3ms

-- COMPOSITE INDEX ORDER MATTERS:
-- Put equality columns first, range columns last
-- WHERE user_id = X AND status = Y ORDER BY created_at
-- Index: (user_id, status, created_at) ✓
-- Index: (created_at, user_id, status) ✗

-- Covering index (index-only scan)
CREATE INDEX idx_orders_covering
ON orders (user_id, status, created_at DESC)
INCLUDE (total_amount, shipping_address);
-- Now the query never touches the table at all
```

---

## The Production Way

### Caching Layer with Redis

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│   App   │────▶│  Redis  │────▶│   DB    │
│ Server  │     │  Cache  │     │Primary  │
└─────────┘     └─────────┘     └─────────┘
                  Cache Hit: 0.5ms
                  Cache Miss: 15ms (then cached)
```

```python
import redis
import json
import hashlib

cache = redis.Redis(host='redis-cluster', port=6379, db=0)

def get_user_cached(user_id):
    cache_key = f"user:{user_id}"

    # Try cache first
    cached = cache.get(cache_key)
    if cached:
        return json.loads(cached)

    # Cache miss: query database
    user = db.execute(
        "SELECT * FROM users WHERE id = %s", user_id
    ).fetchone()

    # Store in cache with TTL
    cache.setex(
        cache_key,
        3600,  # 1 hour TTL
        json.dumps(user, default=str)
    )
    return user

def update_user(user_id, data):
    # Update database
    db.execute("UPDATE users SET ...")

    # INVALIDATE cache (don't update it!)
    cache.delete(f"user:{user_id}")
```

### Vertical Scaling Limits

| Instance Type | vCPUs | RAM    | Max IOPS | Cost/Month |
|---------------|-------|--------|----------|------------|
| db.t3.medium  | 2     | 4 GB   | 3,000    | $50        |
| db.r5.large   | 2     | 16 GB  | 10,000   | $250       |
| db.r5.4xlarge | 16    | 128 GB | 40,000   | $2,000     |
| db.r5.24xlarge| 96    | 768 GB | 80,000   | $15,000    |

**When vertical scaling ends, horizontal scaling begins.**

### Sharding Concepts

```
Before Sharding:
  ┌─────────────────────┐
  │     PostgreSQL       │
  │   users A-Z (100M)  │
  └─────────────────────┘

After Sharding:
  ┌──────────┐ ┌──────────┐ ┌──────────┐
  │  Shard 0 │ │  Shard 1 │ │  Shard 2 │
  │ users A-I│ │ users J-R│ │ users S-Z│
  │  (33M)   │ │  (33M)   │ │  (34M)   │
  └──────────┘ └──────────┘ └──────────┘

Sharding Strategies:
  - Hash-based: shard = hash(user_id) % num_shards
  - Range-based: users 1-1M → shard 0, 1M-2M → shard 1
  - Geographic: US users → US shard, EU users → EU shard
```

**Sharding is a LAST RESORT.** It adds massive complexity:
- Cross-shard queries are expensive
- Rebalancing shards is painful
- Joins across shards require application-level logic
- Transactions across shards are difficult

---

## Hands-On Lab

### Lab: Set Up Read Replicas and Connection Pooling

**Objective:** Configure PostgreSQL with a read replica and PgBouncer connection pool.

**Prerequisites:** Docker, docker-compose

**Step 1: Create the environment**

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres-primary:
    image: postgres:16
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: appuser
      POSTGRES_PASSWORD: secretpass
    ports:
      - "5432:5432"
    volumes:
      - pgdata_primary:/var/lib/postgresql/data
      - ./init-primary.sh:/docker-entrypoint-initdb.d/init.sh
    command: >
      postgres
      -c wal_level=replica
      -c max_wal_senders=3
      -c max_replication_slots=3
      -c hot_standby=on

  postgres-replica:
    image: postgres:16
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: appuser
      POSTGRES_PASSWORD: secretpass
    ports:
      - "5433:5432"
    volumes:
      - ./init-replica.sh:/docker-entrypoint-initdb.d/init.sh
    depends_on:
      - postgres-primary

  pgbouncer:
    image: edoburu/pgbouncer
    environment:
      DATABASE_URL: postgres://appuser:secretpass@postgres-primary:5432/myapp
      POOL_MODE: transaction
      MAX_CLIENT_CONN: 1000
      DEFAULT_POOL_SIZE: 20
      MIN_POOL_SIZE: 5
    ports:
      - "6432:5432"
    depends_on:
      - postgres-primary

volumes:
  pgdata_primary:
```

**Step 2: Configure primary for replication**

```bash
#!/bin/bash
# init-primary.sh

# Create replication user
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD 'replicator_pass';
    GRANT ALL PRIVILEGES ON DATABASE myapp TO replicator;

    -- Create sample data
    CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100),
        email VARCHAR(255) UNIQUE,
        created_at TIMESTAMP DEFAULT NOW()
    );

    INSERT INTO users (name, email)
    SELECT
        'User ' || i,
        'user' || i || '@example.com'
    FROM generate_series(1, 10000) AS i;

    -- Enable pg_stat_statements
    CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
EOSQL
```

**Step 3: Set up the replica**

```bash
#!/bin/bash
# init-replica.sh

# Wait for primary to be ready
until pg_isready -h postgres-primary -U replicator; do
  echo "Waiting for primary..."
  sleep 2
done

# Take base backup from primary
rm -rf /var/lib/postgresql/data/*
PGPASSWORD=replicator_pass pg_basebackup \
  -h postgres-primary \
  -D /var/lib/postgresql/data \
  -U replicator \
  -Fp -Xs -P -R

# -R creates standby.signal and sets primary_conninfo
```

**Step 4: Test the setup**

```bash
# Start the environment
docker-compose up -d

# Wait for services to initialize
sleep 10

# Test direct connection to primary
docker exec -it postgres-primary psql -U appuser -d myapp -c \
  "SELECT count(*) FROM users;"

# Test replica (should have same data)
docker exec -it postgres-replica psql -U appuser -d myapp -c \
  "SELECT count(*) FROM users;"

# Verify replication status on primary
docker exec -it postgres-primary psql -U appuser -d myapp -c \
  "SELECT * FROM pg_stat_replication;"

# Test PgBouncer connection pooling
psql -h localhost -p 6432 -U appuser -d myapp -c \
  "SELECT count(*) FROM users;"

# Test write routing (writes should go through primary)
psql -h localhost -p 6432 -U appuser -d myapp -c \
  "INSERT INTO users (name, email) VALUES ('New User', 'new@example.com');"

# Verify replica received the data
psql -h localhost -p 5433 -U appuser -d myapp -c \
  "SELECT * FROM users WHERE email = 'new@example.com';"
```

**Step 5: Monitor connection pooling**

```bash
# Connect to PgBouncer admin console
psql -h localhost -p 6432 -U appuser -d pgbouncer -c "SHOW POOLS;"
psql -h localhost -p 6432 -U appuser -d pgbouncer -c "SHOW STATS;"
psql -h localhost -p 6432 -U appuser -d pgbouncer -c "SHOW CLIENTS;"
psql -h localhost -p 6432 -U appuser -d pgbouncer -c "SHOW CONFIG;"
```

---

## Limitation

Read replicas solve the read scaling problem, but they introduce a new challenge: **replication lag**. When you write to the primary, the replica may be a few milliseconds (or seconds) behind. If a user writes data and immediately reads from a replica, they might see stale data.

This leads to deeper questions about replication:
- How do you handle replication lag?
- What happens when the primary fails?
- Can you write to multiple nodes simultaneously?

These are questions of **replication strategy**, not just scaling.

**Next:** [Module 46: Replication](../46-replication/README.md) — We will dive deep into master-slave replication, multi-master setups, and conflict resolution strategies.
