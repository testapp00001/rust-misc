# Solution 03: Connection Pool Configuration

## Part A: PgBouncer Configuration

```ini
; pgbouncer.ini
[databases]
appdb = host=postgres-primary port=5432 dbname=appdb

[pgbouncer]
; Network settings
listen_addr = 0.0.0.0
listen_port = 6432
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt

; Pool mode: transaction is optimal for web applications
pool_mode = transaction

; Connection limits
default_pool_size = 50
min_pool_size = 10
reserve_pool_size = 5
reserve_pool_timeout = 3
max_client_conn = 200
max_db_connections = 50

; Timeouts
server_connect_timeout = 15
server_login_retry = 3
client_idle_timeout = 300
server_idle_timeout = 300
query_timeout = 30
query_wait_timeout = 30
client_login_timeout = 60

; Logging and admin
admin_users = pgbouncer_admin
stats_users = pgbouncer_stats
log_connections = 1
log_disconnections = 1
log_pooler_errors = 1
stats_period = 60

; DNS and networking
dns_max_ttl = 15
dns_nxdomain_ttl = 15
tcp_keepalive = 1
tcp_keepidle = 300
tcp_keepintvl = 60
```

### Why These Settings
- `pool_mode = transaction`: Server connections are returned after each transaction, allowing 200 clients to share 50 server connections
- `default_pool_size = 50`: Matches our server connection limit
- `max_client_conn = 200`: Supports up to 200 concurrent application connections
- `query_timeout = 30`: Kills queries running longer than 30 seconds to prevent connection hogging
- `client_idle_timeout = 300`: Reclaims connections from idle clients after 5 minutes
- `reserve_pool_size = 5`: Emergency buffer when the main pool is exhausted

## Part B: Pool Sizing Calculation

### Step 1: Queries per Second at Peak
```
QPS = requests_per_second * queries_per_request
QPS = 2,000 * 3
QPS = 6,000 queries/second
```

### Step 2: Optimal Server Connection Pool Size
Using Little's Law: Connections = QPS * Average Query Time
```
Connections = 6,000 * 0.005 (5ms = 0.005 seconds)
Connections = 30 active connections needed
```

Add 20% headroom for spikes:
```
Pool size = 30 * 1.2 = 36 server connections
```

### Step 3: Is max_connections = 200 Sufficient?
```
Required server connections: 36
PostgreSQL max_connections: 200
Utilization: 36/200 = 18%
```

Yes, the current `max_connections` is more than sufficient. In fact, `max_connections = 200` is generous -- you could reduce it to 100 and still have plenty of headroom. Remember that with PgBouncer, the application needs far fewer server connections than client connections.

### Bonus: Client vs Server Connection Ratio
```
Max clients: 2,000 (peak concurrent users)
Max client connections: 200 (PgBouncer limit)
Server connections: 36 (calculated)
Ratio: 200 clients : 36 servers = ~5.5:1
```

This means each server connection serves approximately 5-6 client connections on average, which is a healthy ratio for transaction pooling mode.

## Part C: Pool Exhaustion Handling

### PgBouncer Configuration for Exhaustion

```ini
; When all server connections are busy, queue clients
query_wait_timeout = 30

; Emergency reserve pool
reserve_pool_size = 5
reserve_pool_timeout = 3

; Kill long-running queries to free connections
query_timeout = 30

; Disconnect idle server connections to free them up
server_idle_timeout = 300
server_lifetime = 3600
```

### Application-Level Logic

```python
import time
import random
from contextlib import contextmanager

class DatabasePool:
    def __init__(self, dsn, max_retries=3):
        self.dsn = dsn
        self.max_retries = max_retries

    @contextmanager
    def get_connection(self):
        retries = 0
        while retries < self.max_retries:
            try:
                conn = psycopg2.connect(self.dsn, connect_timeout=10)
                try:
                    yield conn
                finally:
                    conn.close()
                return
            except psycopg2.OperationalError as e:
                retries += 1
                if retries >= self.max_retries:
                    raise ConnectionExhaustedError(
                        f"Failed to get connection after {self.max_retries} retries: {e}"
                    )
                # Exponential backoff with jitter
                wait = (2 ** retries) + random.uniform(0, 1)
                time.sleep(wait)

class ConnectionExhaustedError(Exception):
    pass
```

### What Happens During Exhaustion
1. PgBouncer queues the client in the waiting queue
2. If a server connection becomes free within `query_wait_timeout`, the client gets it
3. If not, PgBouncer returns an error: `ERROR: query_wait_timeout`
4. The application catches this, waits with backoff, and retries
5. If all retries fail, the application returns a 503 to the user

## Common Mistakes to Avoid
- Setting `max_client_conn` equal to `max_db_connections` -- this defeats the purpose of pooling
- Using `session` pooling mode for web applications -- connections are held for the entire session, not just transactions
- Not setting `query_timeout` -- a single long-running query can hold a server connection indefinitely
- Forgetting that PgBouncer itself needs monitoring -- pool exhaustion, connection wait times, and error rates

## Key Takeaway
Connection pooling with PgBouncer allows your application to serve far more concurrent users than the database's connection limit would otherwise allow. Transaction pooling mode is the sweet spot for web applications. Proper sizing requires understanding your workload (QPS and query duration) using Little's Law, and handling exhaustion gracefully with retry logic and backoff.
