# Solution 01: Cache Pattern Selection

## Part A: Match Patterns to Use Cases

### 1. E-commerce Product Catalog: Cache-Aside

**Why cache-aside:**
- Reads dominate writes (10,000 reads/sec vs 10 writes/day).
- Staleness is acceptable for up to 5 minutes.
- Cache-aside only caches data that is actually requested (no wasted cache space).
- On update, the application invalidates the cache. The next read repopulates it.
- If the cache fails, the application falls back to the database (no outage).

### 2. Banking Transaction System: Write-Through

**Why write-through:**
- Every write must be immediately visible to subsequent reads.
- Write-through writes to both cache and database simultaneously.
- After a write, the cache always has the latest data.
- Trade-off: write latency increases (two writes per operation), but consistency is guaranteed.
- If the cache fails, writes still go to the database (no data loss).

### 3. Social Media Feed: Write-Behind (Write-Back)

**Why write-behind:**
- Writes are frequent (1,000/sec) and must be fast.
- Write-behind writes to the cache immediately and flushes to the database asynchronously.
- Users can tolerate a few seconds of staleness (eventual consistency).
- Batch database writes for efficiency.
- Trade-off: risk of data loss if the cache crashes before flushing.

---

## Part B: Explain Trade-offs

### What happens when a product is updated?

1. The application updates the database: `UPDATE products SET ... WHERE id = ?`
2. The application invalidates the cache: `redis.delete(f"product:{product_id}")`
3. The next read for this product misses the cache.
4. The application queries the database and repopulates the cache.
5. Subsequent reads hit the cache with fresh data.

### How long might stale data be served?

- **Best case**: Zero staleness. The application invalidates the cache immediately on update.
- **Worst case**: Up to 5 minutes (the TTL). This happens if:
  - The application forgets to invalidate the cache on update.
  - There is a race condition: a read happens between the database write and the cache invalidation.
  - The invalidation message is lost (in a distributed system).

### What happens if the cache is cold?

- Every request misses the cache and hits the database.
- Database load increases dramatically (from ~0.1% to 100% of reads).
- Latency increases (database queries are slower than cache reads).
- The cache gradually warms up as requests populate it.
- This is why cache warming is important for critical data.

---

## Part C: Invalidation Strategy

### E-commerce: TTL + Event-Based

```python
def update_product(product_id, data):
    db.execute("UPDATE products SET ... WHERE id = ?", product_id, data)
    # Event-based: immediate invalidation
    redis.delete(f"product:{product_id}")
    # TTL: backup in case invalidation is missed
    # Cache entries expire after 5 minutes regardless
```

**Why both**: Event-based invalidation provides immediate freshness. TTL provides a safety net -- even if invalidation is missed, stale data expires within 5 minutes.

### Banking: Write-Through (No Separate Invalidation)

```python
def create_transaction(account_id, amount):
    # Write to database
    db.execute("INSERT INTO transactions ...", account_id, amount)
    # Write to cache simultaneously
    balance = db.query("SELECT balance FROM accounts WHERE id = ?", account_id)
    redis.setex(f"account:{account_id}:balance", 3600, json.dumps(balance))
    return balance
```

**Why**: Write-through ensures the cache always has the latest data. There is no separate invalidation step because the cache is updated on every write.

### Social Media: TTL + Lock-Based Stampede Prevention

```python
def get_post(post_id):
    cached = redis.get(f"post:{post_id}")
    if cached:
        return json.loads(cached)

    # Lock-based stampede prevention
    lock_key = f"lock:post:{post_id}"
    if redis.set(lock_key, "1", nx=True, ex=10):
        try:
            post = db.query("SELECT * FROM posts WHERE id = ?", post_id)
            redis.setex(f"post:{post_id}", 60, json.dumps(post))
            return post
        finally:
            redis.delete(lock_key)
    else:
        time.sleep(0.1)
        return get_post(post_id)  # Retry
```

**Why**: Popular posts expire and trigger stampedes. The lock ensures only one request rebuilds the cache. Others wait and then read from the cache.

---

## Common Mistakes
1. **Using write-through for read-heavy workloads**: Write-through adds latency to every write. If writes are rare, this overhead is wasted. Use cache-aside instead.
2. **Using write-behind when data loss is unacceptable**: Write-behind risks losing data if the cache crashes before flushing to the database. Use write-through for critical data.
3. **Relying only on TTL for invalidation**: TTL provides a safety net but does not guarantee immediate freshness. Use event-based invalidation for data that must be fresh.
4. **Not handling cache failures**: If Redis is down, the application must fall back to the database. Never let a cache failure cause an outage.

## Relevant README Sections
- [Cache Patterns](../README.md#cache-patterns)
- [Redis vs Memcached](../README.md#redis-vs-memcached)
- [Cache Invalidation Strategies](../README.md#cache-invalidation-strategies)
