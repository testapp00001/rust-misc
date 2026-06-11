# Solution 02: Convert a Stateful App to Use Redis for Session Storage

## Part A: Demonstrate the Problem

### Expected Behavior

```bash
# Start the stateful app
docker compose -f docker-compose-stateful.yml up -d --build

# Login through app1
curl -v -X POST http://localhost:5001/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}'
# Response: Set-Cookie: session_id=<some-uuid>

# Access profile through app2 using the same cookie
curl http://localhost:5002/profile \
  -H "Cookie: session_id=<some-uuid>"
# Response: {"error": "Not authenticated"}
```

### What Happens

App 1 created the session in its own `sessions` dictionary. App 2 has a completely
separate `sessions` dictionary. The session ID exists only in App 1's memory. When
you send that session ID to App 2, it does a dictionary lookup, finds nothing, and
returns 401.

This is the fundamental problem: **in-memory sessions are local to the process**.
Each Docker container runs a separate Python process with its own memory space.

### Why This Works (the Problem)

Each Flask process maintains its own `sessions = {}` dictionary in Python's heap memory.
There is no mechanism for sharing this dictionary between containers. Even if both
containers run on the same host, they are isolated by Docker's namespace mechanism.

## Part B: Create the Redis-Backed Version

### app_redis.py

```python
from flask import Flask, request, jsonify, make_response
import uuid
import json
import redis
import os

app = Flask(__name__)

# Connect to Redis using environment variable for flexibility
REDIS_HOST = os.environ.get('REDIS_HOST', 'localhost')
redis_client = redis.Redis(host=REDIS_HOST, port=6379, db=0, decode_responses=True)

SESSION_TTL = 3600  # 1 hour in seconds

@app.route('/login', methods=['POST'])
def login():
    data = request.json
    if not data:
        return jsonify({"error": "Request body required"}), 400

    username = data.get('username')
    password = data.get('password')

    if username == 'admin' and password == 'secret':
        session_id = str(uuid.uuid4())
        session_data = {
            'user_id': 1,
            'username': 'admin',
            'role': 'admin',
            'login_count': 1
        }
        # Store in Redis with TTL
        redis_client.setex(
            f"session:{session_id}",
            SESSION_TTL,
            json.dumps(session_data)
        )
        response = make_response(jsonify({"message": "Logged in"}))
        response.set_cookie('session_id', session_id, httponly=True)
        return response

    return jsonify({"error": "Invalid credentials"}), 401

def get_session(session_id):
    """Retrieve session from Redis, return None if not found or expired."""
    if not session_id:
        return None
    data = redis_client.get(f"session:{session_id}")
    if data:
        return json.loads(data)
    return None

@app.route('/profile')
def profile():
    session_id = request.cookies.get('session_id')
    session_data = get_session(session_id)
    if not session_data:
        return jsonify({"error": "Not authenticated"}), 401
    return jsonify(session_data)

@app.route('/visit')
def visit():
    session_id = request.cookies.get('session_id')
    session_data = get_session(session_id)
    if not session_data:
        return jsonify({"error": "Not authenticated"}), 401

    session_data['login_count'] += 1

    # Update in Redis (refresh TTL too)
    redis_client.setex(
        f"session:{session_id}",
        SESSION_TTL,
        json.dumps(session_data)
    )
    return jsonify({
        "username": session_data['username'],
        "visit_count": session_data['login_count']
    })

@app.route('/logout', methods=['POST'])
def logout():
    session_id = request.cookies.get('session_id')
    if session_id:
        redis_client.delete(f"session:{session_id}")
    response = make_response(jsonify({"message": "Logged out"}))
    response.delete_cookie('session_id')
    return response

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

### Key Changes Explained

1. **`redis_client` replaces `sessions` dictionary:** Instead of a Python dict, we use
   Redis as the key-value store. The connection is established once at startup.

2. **`setex` instead of `dict[key] = value`:** `setex` sets a key with an expiration
   time. After 3600 seconds, Redis automatically deletes the key. This handles session
   cleanup without a separate garbage collection process.

3. **`json.dumps` / `json.loads`:** Redis stores strings, not Python objects. We serialize
   session data to JSON when writing and deserialize when reading. Using JSON instead
   of pickle is safer (no arbitrary code execution risk) and portable (other languages
   can read it).

4. **`decode_responses=True`:** By default, Redis returns bytes. This flag makes it
   return strings, which is more convenient for JSON handling.

5. **`REDIS_HOST` environment variable:** In Docker Compose, the Redis service hostname
   is `redis`. Locally, it is `localhost`. Using an environment variable makes the app
   work in both contexts.

### Why This Works

Every app instance connects to the same Redis server. When App 1 stores a session with
`setex("session:abc", 3600, data)`, App 2 can immediately retrieve it with
`get("session:abc")`. The session data is no longer tied to any specific app process.

### Common Mistakes

- **Using pickle instead of JSON for serialization.** Pickle is Python-specific and
  can execute arbitrary code during deserialization. JSON is safe and portable.

- **Not setting a TTL.** Without expiration, sessions accumulate forever in Redis.
  Old, abandoned sessions consume memory until Redis runs out.

- **Using `set` instead of `setex`.** The `set` command does not set an expiration.
  You would need a separate `expire` call, which is a race condition if the process
  crashes between the two calls.

- **Not handling Redis connection failures.** If Redis is down, `redis_client.get()`
  will raise an exception. In production, you need error handling or a fallback.

## Part C: Create the Docker Compose Configuration

### nginx.conf

```nginx
upstream app {
    server app1:5000;
    server app2:5000;
}

server {
    listen 80;

    location / {
        proxy_pass http://app;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### docker-compose-redis.yml

```yaml
version: "3.8"

services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  app1:
    build: .
    environment:
      - REDIS_HOST=redis
    depends_on:
      - redis

  app2:
    build: .
    environment:
      - REDIS_HOST=redis
    depends_on:
      - redis

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - app1
      - app2
```

### Why This Works

- Both `app1` and `app2` connect to the same Redis service using the hostname `redis`
  (Docker Compose DNS resolution).
- Nginx distributes requests across both app instances using round-robin (the default).
- No sticky sessions configured in nginx -- every request can go to any instance.

### Common Mistakes

- **Not setting `depends_on`.** Without it, the app might start before Redis is ready
  and fail to connect. Note: `depends_on` only waits for the container to start, not
  for Redis to be ready to accept connections. In production, add a retry mechanism.

- **Using `localhost` for Redis in Docker.** Inside a container, `localhost` refers to
  the container itself, not the host machine or other containers. Use the service name
  (`redis`) instead.

## Part D: Verify It Works

```bash
# Start the Redis-backed version
docker compose -f docker-compose-redis.yml up -d --build

# Wait for services to be ready
sleep 5

# Login and capture cookie
curl -c cookies.txt -X POST http://localhost/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}'

# Hit the load balancer 10 times
for i in $(seq 1 10); do
  curl -s -b cookies.txt http://localhost/visit
  echo
done
```

### Expected Output

```json
{"username":"admin","visit_count":1}
{"username":"admin","visit_count":2}
{"username":"admin","visit_count":3}
{"username":"admin","visit_count":4}
{"username":"admin","visit_count":5}
{"username":"admin","visit_count":6}
{"username":"admin","visit_count":7}
{"username":"admin","visit_count":8}
{"username":"admin","visit_count":9}
{"username":"admin","visit_count":10}
```

The visit count increments correctly even though requests alternate between app1 and app2.

### Why This Works

Each request goes through nginx, which alternates between app1 and app2. Both instances
read the session from Redis, update the `login_count`, and write it back. The session
is shared, so the count is consistent regardless of which instance handles the request.

## Part E: Session Expiration Test

Change `SESSION_TTL = 3600` to `SESSION_TTL = 10` in `app_redis.py`, then:

```bash
# Rebuild and restart
docker compose -f docker-compose-redis.yml up -d --build

# Login
curl -c cookies.txt -X POST http://localhost/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}'

# Verify session works
curl -s -b cookies.txt http://localhost/profile
# Returns user data

# Wait for expiration
sleep 15

# Try again
curl -s -b cookies.txt http://localhost/profile
# Returns: {"error": "Not authenticated"}
```

### Why This Works

Redis's `setex` command atomically sets a key and its expiration. After 10 seconds,
Redis deletes the key. When the app tries `redis_client.get("session:abc")`, it
returns `None`. The app treats this as an unauthenticated request.

This is one of Redis's key advantages over a database-backed session store: automatic
expiration without a separate cleanup job.

## Key Takeaway

Externalizing sessions to Redis decouples session state from application instances.
Any instance can handle any request because they all share the same session store.
The cost is a new infrastructure dependency (Redis) and a small latency increase
(Redis round-trip on every request). For most applications, this trade-off is worth it.
