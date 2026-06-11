# Cheatsheet: Database Scaling

## Scaling Strategies

| Strategy | Complexity | Best For |
|----------|------------|----------|
| Vertical scaling | Low | Small-medium databases |
| Read replicas | Medium | Read-heavy workloads |
| Connection pooling | Low | High connection count |
| Caching (Redis) | Medium | Repeated queries |
| Sharding | High | Write-heavy, massive scale |

## Read Replicas (PostgreSQL)
```sql
-- On primary
ALTER SYSTEM SET wal_level = 'replica';
ALTER SYSTEM SET max_wal_senders = 5;

-- On replica
primary_conninfo = 'host=primary port=5432 user=replicator'
```

## Connection Pooling (PgBouncer)
```ini
[databases]
mydb = host=postgres port=5432 dbname=mydb

[pgbouncer]
listen_port = 6432
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 20
```

## Redis Caching
```python
import redis

cache = redis.from_url('redis://redis:6379')

def get_user(user_id):
    # Check cache first
    cached = cache.get(f'user:{user_id}')
    if cached:
        return json.loads(cached)

    # Query database
    user = db.query('SELECT * FROM users WHERE id = %s', user_id)

    # Cache for 5 minutes
    cache.setex(f'user:{user_id}', 300, json.dumps(user))
    return user
```
