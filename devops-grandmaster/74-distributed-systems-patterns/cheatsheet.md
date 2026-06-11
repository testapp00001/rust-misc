# Cheatsheet: Distributed Systems Patterns

## CAP Theorem
```
Pick 2 of 3:
- Consistency (all nodes see same data)
- Availability (every request gets a response)
- Partition tolerance (system works despite network failures)

In practice: You must handle partitions, so choose CP or AP
```

## Common Patterns

### Circuit Breaker
```python
import pybreaker

db_breaker = pybreaker.CircuitBreaker(
    fail_max=5,
    reset_timeout=30
)

@db_breaker
def query_database():
    return db.execute("SELECT * FROM users")
```

### Retry with Backoff
```python
import time
import random

def retry_with_backoff(func, max_retries=5):
    for attempt in range(max_retries):
        try:
            return func()
        except Exception as e:
            if attempt == max_retries - 1:
                raise
            wait = min(2 ** attempt + random.random(), 32)
            time.sleep(wait)
```

### Saga Pattern
```
Step 1: Create order → Success
Step 2: Reserve inventory → Success
Step 3: Process payment → FAIL
Compensation: Cancel order, release inventory
```

### Idempotency
```python
# Same request multiple times = same result
@app.route('/transfer', methods=['POST'])
def transfer():
    idempotency_key = request.headers.get('Idempotency-Key')
    if db.exists(f'idempotent:{idempotency_key}'):
        return db.get(f'idempotent:{idempotency_key}')

    result = do_transfer(request.json)
    db.set(f'idempotent:{idempotency_key}', result)
    return result
```
