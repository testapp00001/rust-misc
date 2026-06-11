# Cheatsheet: Caching Strategies

## Cache Patterns

| Pattern | Description | Use Case |
|---------|-------------|----------|
| Cache-aside | App manages cache | Most common |
| Write-through | Write to cache + DB | Consistency critical |
| Write-behind | Write to cache, async to DB | High write throughput |
| Read-through | Cache fetches from DB | Simplified app code |

## Cache-Aside Pattern
```python
def get_user(user_id):
    # Check cache
    cached = cache.get(f'user:{user_id}')
    if cached:
        return cached

    # Cache miss — query DB
    user = db.query('SELECT * FROM users WHERE id = %s', user_id)

    # Populate cache
    cache.setex(f'user:{user_id}', 300, json.dumps(user))
    return user

def update_user(user_id, data):
    # Update DB
    db.execute('UPDATE users SET ... WHERE id = %s', user_id)

    # Invalidate cache
    cache.delete(f'user:{user_id}')
```

## Redis Configuration
```bash
# Set max memory
redis-server --maxmemory 256mb --maxmemory-policy allkeys-lru
```

## Cache Invalidation
```
Time-based (TTL)    → Set expiration time
Event-based         → Invalidate on data change
Manual              → Explicitly clear cache
```

## Cache Stampede Prevention
```python
# Use distributed lock
def get_expensive_data(key):
    data = cache.get(key)
    if data:
        return data

    # Try to acquire lock
    if cache.set(f'lock:{key}', '1', nx=True, ex=10):
        try:
            data = compute_expensive_data()
            cache.setex(key, 300, data)
            return data
        finally:
            cache.delete(f'lock:{key}')
    else:
        # Wait and retry
        time.sleep(0.1)
        return get_expensive_data(key)
```
