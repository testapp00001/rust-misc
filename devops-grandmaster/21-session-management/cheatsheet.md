# Cheatsheet: Session Management

## The Problem
```
User → Load Balancer → Server A (creates session)
User → Load Balancer → Server B (session not found!) ❌
```

## Solutions

### 1. Sticky Sessions (Band-aid)
```nginx
upstream backend {
    ip_hash;  # Same IP → same server
    server backend1:8080;
    server backend2:8080;
}
```

### 2. External Session Store (Better)
```python
# Store sessions in Redis
import redis
session_store = redis.from_url('redis://redis:6379')

# On login: session_store.set(f'session:{user_id}', session_data)
# On request: session_data = session_store.get(f'session:{user_id}')
```

### 3. JWT Tokens (Best for APIs)
```python
# Stateless — no server-side session
import jwt

# On login: token = jwt.encode({'user_id': 123}, SECRET_KEY)
# On request: payload = jwt.decode(token, SECRET_KEY)
```

## 12-Factor App: Stateless Processes
```
- No local session storage
- No sticky sessions
- Store state in Redis/database
- Use JWT for authentication
```
