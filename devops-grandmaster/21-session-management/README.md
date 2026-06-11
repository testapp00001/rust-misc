# Module 21: Session Management

> **Previous module (20):** Reverse proxy — routing traffic to different services through a single entry point.
> **Limitation:** Load balancing breaks sessions. A user logs in on one instance, the next request goes to another instance, and they appear logged out.
> **This module:** Sticky sessions, shared session stores, JWT tokens, and stateless design.

---

## 1. The Problem

A user logs in. Your app creates a session: `{user_id: 42, role: "admin", login_time: "..."}`. This session is stored in the app instance's memory.

```
User → Login → App 1 (creates session in memory) → OK, logged in
User → Next request → Load Balancer → App 2 (no session) → "Please log in"
```

The user is logged out. They log in again. The next request goes to App 3. Logged out again. Infuriating.

This is the fundamental problem: **stateful sessions are tied to a specific server**. Load balancing distributes requests across servers. These two facts are incompatible.

---

## 2. The Naive Way — Sticky Sessions

Tell the load balancer: "once a client connects to App 1, keep sending them to App 1."

```
# HAProxy sticky sessions
backend app_servers
    balance roundrobin
    cookie SERVERID insert indirect nocache
    server app1 app1:5000 check cookie app1
    server app2 app2:5000 check cookie app2
    server app3 app3:5000 check cookie app3
```

The load balancer sets a cookie (`SERVERID=app1`). All subsequent requests from that client go to App 1. Session works.

**Why it's a band-aid, not a solution:**

1. **Uneven distribution.** If 80% of users happen to get routed to App 1 first, it carries 80% of the load. You lose the benefit of load balancing.

2. **No failover.** If App 1 crashes, all its sticky users lose their sessions. They are forced to re-login and get reassigned to another server.

3. **Scaling pain.** Adding a new instance does not help existing users — they are stuck on their assigned instance. Removing an instance forces all its users to re-authenticate.

4. **Deployment nightmares.** Rolling updates become complicated. You cannot drain an instance without breaking sessions.

5. **Does not solve the real problem.** You still have state in memory. Sticky sessions just hide the problem.

```
Sticky sessions are to load balancing
what duct tape is to plumbing.
It works until it doesn't, and then it's catastrophic.
```

---

## 3. The Right Way — Externalize Session State

The real solution: do not store sessions in app instance memory. Store them in an external service that all instances can access.

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   App 1     │  │   App 2     │  │   App 3     │
│ (stateless) │  │ (stateless) │  │ (stateless) │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │
       └────────────────┼────────────────┘
                        │
                        v
                ┌───────────────┐
                │  Redis        │
                │  (sessions)   │
                │               │
                │ session_abc → │
                │   {user_id:42,│
                │    role:admin}│
                └───────────────┘
```

Any instance can handle any request. All instances read/write sessions from Redis. No sticky sessions needed.

### Server-Side Sessions in Redis

**Python (Flask) example:**
```python
from flask import Flask, session
from flask_session import Session
import redis

app = Flask(__name__)

# Session configuration: store sessions in Redis
app.config['SESSION_TYPE'] = 'redis'
app.config['SESSION_REDIS'] = redis.Redis(host='redis', port=6379)
app.config['SESSION_PERMANENT'] = False
app.config['SESSION_USE_SIGNER'] = True
app.config['SECRET_KEY'] = 'your-secret-key'

Session(app)

@app.route('/login', methods=['POST'])
def login():
    # Validate credentials
    user = authenticate(request.json)
    if user:
        session['user_id'] = user.id
        session['role'] = user.role
        return jsonify({"message": "Logged in"})
    return jsonify({"error": "Invalid credentials"}), 401

@app.route('/profile')
def profile():
    if 'user_id' not in session:
        return jsonify({"error": "Not authenticated"}), 401
    return jsonify({
        "user_id": session['user_id'],
        "role": session['role']
    })
```

**Node.js (Express) example:**
```javascript
const express = require('express');
const session = require('express-session');
const RedisStore = require('connect-redis').default;
const { createClient } = require('redis');

const redisClient = createClient({ url: 'redis://redis:6379' });
redisClient.connect();

const app = express();

app.use(session({
    store: new RedisStore({ client: redisClient }),
    secret: 'your-secret-key',
    resave: false,
    saveUninitialized: false,
    cookie: {
        secure: false,  // Set to true in production with HTTPS
        httpOnly: true,
        maxAge: 3600000  // 1 hour
    }
}));

app.post('/login', (req, res) => {
    const user = authenticate(req.body);
    if (user) {
        req.session.userId = user.id;
        req.session.role = user.role;
        res.json({ message: 'Logged in' });
    } else {
        res.status(401).json({ error: 'Invalid credentials' });
    }
});

app.get('/profile', (req, res) => {
    if (!req.session.userId) {
        return res.status(401).json({ error: 'Not authenticated' });
    }
    res.json({
        userId: req.session.userId,
        role: req.session.role
    });
});
```

### Database-Backed Sessions

If you do not want to add Redis, store sessions in your existing database.

```sql
CREATE TABLE sessions (
    id VARCHAR(128) PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    data JSONB,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Cleanup expired sessions (run periodically)
DELETE FROM sessions WHERE expires_at < NOW();
```

**Trade-off:** Redis is faster (in-memory) and has built-in TTL (automatic expiration). Database is one less service to manage but adds load to your database.

---

## 4. The Production Way — JWT Tokens (Stateless Authentication)

The most scalable approach: do not store sessions at all. Put everything the server needs in the token itself.

### How JWT Works

```
LOGIN:
  Client → POST /login {username, password} → Server
  Server validates credentials
  Server creates JWT: {user_id: 42, role: "admin", exp: tomorrow}
  Server signs JWT with secret key
  Server → JWT token → Client stores it

SUBSEQUENT REQUESTS:
  Client → GET /profile (Authorization: Bearer <JWT>) → Server
  Server verifies signature (no database/Redis lookup!)
  Server reads user_id and role from token
  Server processes request
```

### JWT Token Structure

```
eyJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjo0Miwicm9sZSI6ImFkbWluIiwiZXhwIjoxNzAwMDAwMDAwfQ.signature

Header:   {"alg": "HS256"}
Payload:  {"user_id": 42, "role": "admin", "exp": 1700000000}
Signature: HMAC-SHA256(header + payload, secret_key)
```

### JWT Implementation (Python)

```python
from flask import Flask, request, jsonify
import jwt
import datetime

app = Flask(__name__)
SECRET_KEY = 'your-secret-key'

@app.route('/login', methods=['POST'])
def login():
    user = authenticate(request.json)
    if not user:
        return jsonify({"error": "Invalid credentials"}), 401

    token = jwt.encode({
        'user_id': user.id,
        'role': user.role,
        'exp': datetime.datetime.utcnow() + datetime.timedelta(hours=1)
    }, SECRET_KEY, algorithm='HS256')

    return jsonify({"token": token})

@app.route('/profile')
def profile():
    token = request.headers.get('Authorization', '').replace('Bearer ', '')
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])
        return jsonify({
            "user_id": payload['user_id'],
            "role": payload['role']
        })
    except jwt.ExpiredSignatureError:
        return jsonify({"error": "Token expired"}), 401
    except jwt.InvalidTokenError:
        return jsonify({"error": "Invalid token"}), 401
```

### Why JWT is Stateless

```
Server-side session:
  Client request → Server → Look up session in Redis/DB → Process
  (State stored on server)

JWT:
  Client request → Server → Verify signature locally → Process
  (State stored in token, no lookup needed)
```

The server does not store anything. Every request carries all the information needed. Any instance can verify the token without consulting any other service.

### JWT Trade-offs

```
Advantages:
  + No session storage needed
  + Perfect for horizontal scaling
  + Works across services (microservices)
  + No sticky sessions
  + Reduced database/Redis load

Disadvantages:
  - Cannot revoke tokens easily (until they expire)
  - Token grows with more claims
  - Secret key compromise = total compromise
  - Client must store and send token securely
  - Refresh token rotation adds complexity
```

### Token Revocation (The Hard Problem)

JWT tokens are valid until they expire. How do you revoke a user's access immediately (e.g., they change their password, their account is compromised)?

**Short-lived tokens + refresh tokens:**
```
Access token:  expires in 15 minutes
Refresh token: expires in 7 days

Flow:
  1. User logs in → gets access_token + refresh_token
  2. access_token used for API requests
  3. When access_token expires → use refresh_token to get new access_token
  4. To revoke: delete refresh_token from database
  5. User can no longer get new access_tokens
```

**Token blacklist (if you need immediate revocation):**
```python
# When revoking a token, add its JTI (unique ID) to Redis
@app.route('/logout', methods=['POST'])
def logout():
    token = request.headers.get('Authorization', '').replace('Bearer ', '')
    payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])
    # Blacklist until token's natural expiration
    redis.setex(f"blacklist:{payload['jti']}", 3600, "revoked")
    return jsonify({"message": "Logged out"})

# Check blacklist on every request (adds one Redis lookup)
def verify_token(token):
    payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])
    if redis.get(f"blacklist:{payload['jti']}"):
        raise InvalidTokenError("Token revoked")
    return payload
```

This is the trade-off: pure statelessness vs immediate revocation. Most systems accept a short window (15 minutes) where a revoked token is still valid.

---

## 5. Hands-On Lab

### Lab: Convert Stateful to Stateless Application

**Step 1: The stateful (broken) version**

**app_stateful.py:**
```python
from flask import Flask, request, jsonify

app = Flask(__name__)

# STATEFUL: sessions stored in process memory
sessions = {}

@app.route('/login', methods=['POST'])
def login():
    data = request.json
    if data['username'] == 'admin' and data['password'] == 'secret':
        import uuid
        session_id = str(uuid.uuid4())
        sessions[session_id] = {
            'user_id': 1,
            'username': 'admin',
            'role': 'admin'
        }
        return jsonify({"session_id": session_id})
    return jsonify({"error": "Invalid"}), 401

@app.route('/profile')
def profile():
    session_id = request.headers.get('X-Session-ID')
    if session_id in sessions:
        return jsonify(sessions[session_id])
    return jsonify({"error": "Not authenticated"}), 401

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

Test with two instances:
```bash
# Start two instances
docker run -d --name app1 -p 5001:5000 stateful-app
docker run -d --name app2 -p 5002:5000 stateful-app

# Login on instance 1
curl -X POST http://localhost:5001/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}'
# Returns: {"session_id": "abc-123"}

# Try to use session on instance 2
curl http://localhost:5002/profile \
  -H "X-Session-ID: abc-123"
# Returns: {"error": "Not authenticated"}  ← BROKEN
```

**Step 2: The stateless (correct) version**

**app_stateless.py:**
```python
from flask import Flask, request, jsonify
import jwt
import datetime

app = Flask(__name__)
SECRET_KEY = 'lab-secret-key'

@app.route('/login', methods=['POST'])
def login():
    data = request.json
    if data['username'] == 'admin' and data['password'] == 'secret':
        token = jwt.encode({
            'user_id': 1,
            'username': 'admin',
            'role': 'admin',
            'exp': datetime.datetime.utcnow() + datetime.timedelta(hours=1)
        }, SECRET_KEY, algorithm='HS256')
        return jsonify({"token": token})
    return jsonify({"error": "Invalid"}), 401

@app.route('/profile')
def profile():
    token = request.headers.get('Authorization', '').replace('Bearer ', '')
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])
        return jsonify({
            "user_id": payload['user_id'],
            "username": payload['username'],
            "role": payload['role']
        })
    except jwt.InvalidTokenError:
        return jsonify({"error": "Not authenticated"}), 401

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

Test with two instances:
```bash
# Start two instances
docker run -d --name app1 -p 5001:5000 stateless-app
docker run -d --name app2 -p 5002:5000 stateless-app

# Login on instance 1
curl -X POST http://localhost:5001/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}'
# Returns: {"token": "eyJhbG..."}

# Use token on instance 2 — works!
curl http://localhost:5002/profile \
  -H "Authorization: Bearer eyJhbG..."
# Returns: {"user_id":1,"username":"admin","role":"admin"}  ← WORKS
```

**Step 3: Docker Compose with load balancer**

**docker-compose.yml:**
```yaml
version: "3.8"

services:
  app:
    build: .
    deploy:
      replicas: 3

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - app
```

```bash
docker compose up -d

# Login
TOKEN=$(curl -s -X POST http://localhost/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}' | jq -r '.token')

# Profile — works every time, regardless of which instance handles it
for i in $(seq 1 6); do
  curl -s http://localhost/profile -H "Authorization: Bearer $TOKEN"
  echo
done
```

---

## 6. Limitation

Sessions are solved. Your app is stateless. Traffic is distributed across instances. But how do you know if your service is actually reliable?

How many minutes of downtime per year is acceptable? How do you measure it? How do you commit to a reliability target and hold yourself accountable?

---

## 7. Next Topic

**Module 22: Uptime Commitment** — SLA, SLO, SLI. What 99.9% actually means, error budgets, and how to define reliability targets. [Go to Module 22 →](../22-uptime-commitment/README.md)
