# Exercise 02: Convert a Stateful App to Use Redis for Session Storage

**Type:** Guided
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Convert a Python Flask application that stores sessions in local memory to one that
stores sessions in Redis. This exercise teaches you how external session stores work
and why they solve the horizontal scaling problem.

## Prerequisites

- Docker and Docker Compose installed
- Basic familiarity with Python and Flask
- Understanding of HTTP sessions and cookies

## Starting Code

Create a file called `app_stateful.py`:

```python
from flask import Flask, request, jsonify, make_response
import uuid

app = Flask(__name__)

# PROBLEM: sessions stored in process memory
# If you run two instances, they cannot share sessions
sessions = {}

@app.route('/login', methods=['POST'])
def login():
    data = request.json
    if not data:
        return jsonify({"error": "Request body required"}), 400

    username = data.get('username')
    password = data.get('password')

    # Hardcoded user for this exercise
    if username == 'admin' and password == 'secret':
        session_id = str(uuid.uuid4())
        sessions[session_id] = {
            'user_id': 1,
            'username': 'admin',
            'role': 'admin',
            'login_count': 1
        }
        response = make_response(jsonify({"message": "Logged in"}))
        response.set_cookie('session_id', session_id, httponly=True)
        return response

    return jsonify({"error": "Invalid credentials"}), 401

@app.route('/profile')
def profile():
    session_id = request.cookies.get('session_id')
    if not session_id or session_id not in sessions:
        return jsonify({"error": "Not authenticated"}), 401

    return jsonify(sessions[session_id])

@app.route('/visit')
def visit():
    session_id = request.cookies.get('session_id')
    if not session_id or session_id not in sessions:
        return jsonify({"error": "Not authenticated"}), 401

    sessions[session_id]['login_count'] += 1
    return jsonify({
        "username": sessions[session_id]['username'],
        "visit_count": sessions[session_id]['login_count']
    })

@app.route('/logout', methods=['POST'])
def logout():
    session_id = request.cookies.get('session_id')
    if session_id and session_id in sessions:
        del sessions[session_id]
    response = make_response(jsonify({"message": "Logged out"}))
    response.delete_cookie('session_id')
    return response

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

Create a file called `Dockerfile`:

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask redis
COPY app_stateful.py .
CMD ["python", "app_stateful.py"]
```

Create a file called `docker-compose-stateful.yml`:

```yaml
version: "3.8"

services:
  app1:
    build: .
    ports:
      - "5001:5000"
  app2:
    build: .
    ports:
      - "5002:5000"
```

## Tasks

### Part A: Demonstrate the Problem

1. Start the stateful version: `docker compose -f docker-compose-stateful.yml up -d --build`
2. Log in through app1: `curl -v -X POST http://localhost:5001/login -H "Content-Type: application/json" -d '{"username":"admin","password":"secret"}'`
3. Copy the `session_id` cookie from the response
4. Try to access the profile through app2 using that same cookie
5. Document what happens and why

<details>
<summary>Hint</summary>

Use the `-v` flag with curl to see cookies in the headers. The session_id cookie
value is what identifies the user. When you send that cookie to a different instance,
that instance's `sessions` dictionary does not contain that key.

</details>

### Part B: Create the Redis-Backed Version

Convert `app_stateful.py` to `app_redis.py` by:

1. Replace the in-memory `sessions` dictionary with Redis
2. Use the `redis` Python library to store and retrieve session data
3. Set an expiration (TTL) on session keys so old sessions are automatically cleaned up
4. Serialize session data as JSON when writing to Redis and deserialize when reading

<details>
<summary>Hint</summary>

Use `redis.Redis(host='redis', port=6379)` to connect. Store sessions with:
```python
import json
redis_client.setex(f"session:{session_id}", 3600, json.dumps(session_data))
```

Retrieve with:
```python
data = redis_client.get(f"session:{session_id}")
if data:
    session_data = json.loads(data)
```

`setex` sets a key with an expiration in seconds. This is how Redis handles TTL.

</details>

### Part C: Create the Docker Compose Configuration

Write a `docker-compose-redis.yml` that includes:

1. A Redis service
2. Two app instances that both connect to Redis
3. An nginx load balancer that distributes requests between the two app instances

The nginx configuration should use round-robin (the default).

<details>
<summary>Hint</summary>

Your nginx.conf should define an upstream block:
```
upstream app {
    server app1:5000;
    server app2:5000;
}
```

And proxy requests to it:
```
location / {
    proxy_pass http://app;
}
```

In your Docker Compose file, make sure both app services have the same build context
and that nginx depends on both app services.

</details>

### Part D: Verify It Works

1. Start the Redis-backed version: `docker compose -f docker-compose-redis.yml up -d --build`
2. Log in through the load balancer
3. Make multiple requests to `/profile` and `/visit` through the load balancer
4. Verify that the session persists across requests even though different instances
   handle different requests
5. Verify that the visit counter increments correctly

<details>
<summary>Hint</summary>

Run this test loop to confirm:
```bash
# Login and capture cookie
curl -c cookies.txt -X POST http://localhost/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}'

# Hit the load balancer multiple times
for i in $(seq 1 10); do
  curl -b cookies.txt http://localhost/visit
  echo
done
```

If the visit count increments from 1 to 10, sessions are shared correctly.

</details>

### Part E: Session Expiration Test

1. Set the Redis TTL to 10 seconds instead of 3600
2. Log in, wait 15 seconds, then try to access the profile
3. Verify that the session has expired and you get a "Not authenticated" response

## Success Criteria

- [ ] You can demonstrate that the stateful version fails when requests hit different instances
- [ ] Your Redis-backed version stores sessions with `setex` and retrieves them with `get`
- [ ] Session data is serialized as JSON (not Python pickle)
- [ ] Sessions have a configurable TTL that causes automatic expiration
- [ ] Both app instances can read the same session from Redis
- [ ] The visit counter increments correctly across multiple instances
- [ ] You have a working `docker-compose-redis.yml` with Redis, two app instances, and nginx

## What You Should Understand After This Exercise

Externalizing session state to Redis decouples sessions from individual app instances.
Any instance can read any session because they all share the same Redis store. This
is the foundation of horizontally scalable session management. The trade-off is that
Redis becomes a dependency -- if it goes down, all sessions are lost.
