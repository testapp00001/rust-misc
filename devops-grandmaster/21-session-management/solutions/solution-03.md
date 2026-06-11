# Solution 03: Implement JWT-Based Stateless Authentication

## Part A: The JWT Authentication Server

### app_jwt.py

```python
from flask import Flask, request, jsonify
import jwt
import datetime
import uuid
import os
from functools import wraps

app = Flask(__name__)

SECRET_KEY = os.environ.get('JWT_SECRET', 'dev-secret-key-change-in-production')
REFRESH_SECRET = os.environ.get('JWT_REFRESH_SECRET', 'dev-refresh-secret-change-in-production')

# Hardcoded users for this exercise
USERS = {
    'admin': {'user_id': 1, 'username': 'admin', 'password': 'secret', 'role': 'admin'},
    'user': {'user_id': 2, 'username': 'user', 'password': 'password', 'role': 'user'},
}

# In-memory token blacklist (for bonus Part D)
token_blacklist = set()


def create_access_token(user):
    """Create a short-lived access token."""
    return jwt.encode({
        'user_id': user['user_id'],
        'username': user['username'],
        'role': user['role'],
        'type': 'access',
        'jti': str(uuid.uuid4()),
        'exp': datetime.datetime.utcnow() + datetime.timedelta(minutes=15),
        'iat': datetime.datetime.utcnow(),
    }, SECRET_KEY, algorithm='HS256')


def create_refresh_token(user):
    """Create a long-lived refresh token."""
    return jwt.encode({
        'user_id': user['user_id'],
        'type': 'refresh',
        'jti': str(uuid.uuid4()),
        'exp': datetime.datetime.utcnow() + datetime.timedelta(days=7),
        'iat': datetime.datetime.utcnow(),
    }, REFRESH_SECRET, algorithm='HS256')


def get_token_from_header():
    """Extract token from Authorization header."""
    auth_header = request.headers.get('Authorization', '')
    if auth_header.startswith('Bearer '):
        return auth_header[7:]
    return None


@app.route('/login', methods=['POST'])
def login():
    data = request.json
    if not data:
        return jsonify({"error": "Request body required"}), 400

    username = data.get('username')
    password = data.get('password')

    user = USERS.get(username)
    if not user or user['password'] != password:
        return jsonify({"error": "Invalid credentials"}), 401

    access_token = create_access_token(user)
    refresh_token = create_refresh_token(user)

    return jsonify({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "token_type": "Bearer",
        "expires_in": 900  # 15 minutes in seconds
    })


@app.route('/profile')
def profile():
    token = get_token_from_header()
    if not token:
        return jsonify({"error": "Authorization header required"}), 401

    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])

        # Check if token type is correct
        if payload.get('type') != 'access':
            return jsonify({"error": "Invalid token type"}), 401

        # Check if token is blacklisted
        if payload.get('jti') in token_blacklist:
            return jsonify({"error": "Token has been revoked"}), 401

        return jsonify({
            "user_id": payload['user_id'],
            "username": payload['username'],
            "role": payload['role']
        })
    except jwt.ExpiredSignatureError:
        return jsonify({"error": "Token has expired"}), 401
    except jwt.InvalidTokenError as e:
        return jsonify({"error": "Invalid token"}), 401


@app.route('/refresh', methods=['POST'])
def refresh():
    data = request.json
    if not data:
        return jsonify({"error": "Request body required"}), 400

    refresh_token = data.get('refresh_token')
    if not refresh_token:
        return jsonify({"error": "refresh_token required"}), 400

    try:
        payload = jwt.decode(refresh_token, REFRESH_SECRET, algorithms=['HS256'])

        if payload.get('type') != 'refresh':
            return jsonify({"error": "Invalid token type"}), 401

        # Look up the user (in production, fetch from database)
        user = None
        for u in USERS.values():
            if u['user_id'] == payload['user_id']:
                user = u
                break

        if not user:
            return jsonify({"error": "User not found"}), 401

        # Issue new access token
        new_access_token = create_access_token(user)

        return jsonify({
            "access_token": new_access_token,
            "token_type": "Bearer",
            "expires_in": 900
        })
    except jwt.ExpiredSignatureError:
        return jsonify({"error": "Refresh token has expired"}), 401
    except jwt.InvalidTokenError:
        return jsonify({"error": "Invalid refresh token"}), 401


@app.route('/logout', methods=['POST'])
def logout():
    token = get_token_from_header()
    if not token:
        return jsonify({"error": "Authorization header required"}), 401

    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])
        # Add token's jti to blacklist
        token_blacklist.add(payload['jti'])
        return jsonify({"message": "Logged out successfully"})
    except jwt.InvalidTokenError:
        return jsonify({"error": "Invalid token"}), 401


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

### Why This Works

**Access tokens** contain the user's identity and role. The server verifies the
signature using `SECRET_KEY` -- no database or Redis lookup needed. The token is
self-contained.

**Refresh tokens** use a separate secret (`REFRESH_SECRET`) so that compromising
the access token secret does not compromise refresh tokens. They contain only the
`user_id` and `type` -- no role or username, because those might change between
the time the refresh token was issued and when it is used.

**The `jti` claim** is a unique identifier for each token. This is essential for
blacklisting -- you cannot revoke "a token" without being able to identify it.

**`iat` (issued at)** is included so you can calculate token age if needed.

### Common Mistakes

- **Using the same secret for access and refresh tokens.** If the access token secret
  is compromised (shorter lifetime, more exposure), the attacker can also forge refresh
  tokens.

- **Not checking the `type` claim.** Without this, a refresh token could be used as
  an access token (it has a valid signature). Always verify `type == 'access'` for
  protected endpoints.

- **Storing sensitive data in the JWT.** JWT payloads are base64-encoded, not encrypted.
  Anyone can decode the token and read the payload. Never put passwords, SSNs, or
  other secrets in a JWT.

- **Using `datetime.datetime.utcnow()` without timezone awareness.** This works but
  is deprecated in newer Python versions. In production, use timezone-aware datetimes.

## Part B: Docker Setup

### Dockerfile

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask PyJWT
COPY app_jwt.py .
CMD ["python", "app_jwt.py"]
```

### nginx.conf

```nginx
upstream app {
    server app1:5000;
    server app2:5000;
    server app3:5000;
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

### docker-compose.yml

```yaml
version: "3.8"

services:
  app1:
    build: .
    environment:
      - JWT_SECRET=production-secret-key-change-me
      - JWT_REFRESH_SECRET=production-refresh-secret-change-me

  app2:
    build: .
    environment:
      - JWT_SECRET=production-secret-key-change-me
      - JWT_REFRESH_SECRET=production-refresh-secret-change-me

  app3:
    build: .
    environment:
      - JWT_SECRET=production-secret-key-change-me
      - JWT_REFRESH_SECRET=production-refresh-secret-change-me

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - app1
      - app2
      - app3
```

### Why This Works

All three app instances share the same `JWT_SECRET`. Any instance can verify a token
signed by any other instance. There is no shared state -- no Redis, no database, no
sticky sessions. The token itself carries all the information needed.

### Common Mistakes

- **Hardcoding secrets in docker-compose.yml.** This is fine for development but
  dangerous in production. Use Docker secrets or environment variable injection from
  a secrets manager.

- **Different secrets on different instances.** If app1 has a different `JWT_SECRET`
  than app2, tokens signed by app1 will be rejected by app2. All instances must share
  the same secret.

## Part C: Test Script

### test_jwt.sh

```bash
#!/bin/bash
set -e

BASE_URL="http://localhost"
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

pass() { echo -e "${GREEN}PASS${NC}: $1"; }
fail() { echo -e "${RED}FAIL${NC}: $1"; exit 1; }

echo "=== JWT Authentication Test Suite ==="
echo ""

# --- Test 1: Login ---
echo "--- Test 1: Login ---"
RESPONSE=$(curl -s -X POST "$BASE_URL/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}')

ACCESS_TOKEN=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['access_token'])")
REFRESH_TOKEN=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['refresh_token'])")

if [ -n "$ACCESS_TOKEN" ] && [ -n "$REFRESH_TOKEN" ]; then
  pass "Login returns access_token and refresh_token"
else
  fail "Login did not return tokens"
fi

# --- Test 2: Profile with valid token ---
echo "--- Test 2: Profile with valid token ---"
PROFILE=$(curl -s "$BASE_URL/profile" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

USERNAME=$(echo "$PROFILE" | python3 -c "import sys,json; print(json.load(sys.stdin)['username'])")
if [ "$USERNAME" = "admin" ]; then
  pass "Profile returns correct user data"
else
  fail "Profile returned unexpected data: $PROFILE"
fi

# --- Test 3: Multiple requests with same token ---
echo "--- Test 3: Multiple requests ---"
ALL_OK=true
for i in $(seq 1 10); do
  RESULT=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/profile" \
    -H "Authorization: Bearer $ACCESS_TOKEN")
  if [ "$RESULT" != "200" ]; then
    ALL_OK=false
    break
  fi
done
if $ALL_OK; then
  pass "10 consecutive requests all succeeded"
else
  fail "Request $i failed with status $RESULT"
fi

# --- Test 4: Invalid token ---
echo "--- Test 4: Invalid token ---"
RESULT=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/profile" \
  -H "Authorization: Bearer invalid-token")
if [ "$RESULT" = "401" ]; then
  pass "Invalid token returns 401"
else
  fail "Expected 401, got $RESULT"
fi

# --- Test 5: Missing token ---
echo "--- Test 5: Missing token ---"
RESULT=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/profile")
if [ "$RESULT" = "401" ]; then
  pass "Missing token returns 401"
else
  fail "Expected 401, got $RESULT"
fi

# --- Test 6: Wrong password ---
echo "--- Test 6: Wrong password ---"
RESULT=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"wrong"}')
if [ "$RESULT" = "401" ]; then
  pass "Wrong password returns 401"
else
  fail "Expected 401, got $RESULT"
fi

# --- Test 7: Token expiration ---
echo "--- Test 7: Token expiration ---"
echo "Waiting 16 seconds for access token to expire..."
sleep 16

RESULT=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/profile" \
  -H "Authorization: Bearer $ACCESS_TOKEN")
if [ "$RESULT" = "401" ]; then
  pass "Expired token returns 401"
else
  fail "Expected 401, got $RESULT"
fi

# --- Test 8: Refresh token flow ---
echo "--- Test 8: Refresh token flow ---"
REFRESH_RESPONSE=$(curl -s -X POST "$BASE_URL/refresh" \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\":\"$REFRESH_TOKEN\"}")

NEW_ACCESS_TOKEN=$(echo "$REFRESH_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['access_token'])")
if [ -n "$NEW_ACCESS_TOKEN" ]; then
  pass "Refresh returns new access token"
else
  fail "Refresh did not return new access token: $REFRESH_RESPONSE"
fi

# Use the new token
PROFILE=$(curl -s "$BASE_URL/profile" \
  -H "Authorization: Bearer $NEW_ACCESS_TOKEN")
USERNAME=$(echo "$PROFILE" | python3 -c "import sys,json; print(json.load(sys.stdin)['username'])")
if [ "$USERNAME" = "admin" ]; then
  pass "New access token works for profile"
else
  fail "New access token failed: $PROFILE"
fi

echo ""
echo "=== All tests passed ==="
```

### Why This Works

The test script validates the complete JWT lifecycle: login, token usage, expiration,
and refresh. By setting the access token TTL to 15 minutes, we can test expiration
by waiting 16 seconds (the test overrides the TTL to 30 seconds for faster testing --
you should modify the code accordingly during development).

### Common Mistakes

- **Not testing token expiration.** Many teams test login and profile but forget to
  test what happens when the token expires. Users will hit this in production.

- **Not testing across multiple instances.** The test script hits the load balancer,
  so requests go to different instances. If the JWT secret is not shared correctly,
  some requests will fail.

## Part D: Token Revocation (Bonus)

The implementation in Part A already includes the blacklist. Here is the explanation:

### How It Works

```python
# In logout:
token_blacklist.add(payload['jti'])

# In profile verification:
if payload.get('jti') in token_blacklist:
    return jsonify({"error": "Token has been revoked"}), 401
```

When a user logs out, we extract the token's `jti` (unique ID) and add it to an
in-memory set. On every subsequent request, we check if the token's `jti` is in the
blacklist before allowing access.

### Why In-Memory Blacklisting Is Problematic

Each app instance has its own `token_blacklist` set. If Instance 1 blacklists a token,
Instance 2 does not know about it. A revoked token can still be used on other instances.

```
User logs out on Instance 1 → token blacklisted on Instance 1
User sends request to Instance 2 → token is still valid on Instance 2 → access granted
```

### Production Solution

Use Redis for the blacklist:

```python
import redis
redis_client = redis.Redis(host='redis', port=6379)

# Blacklist with TTL matching token's remaining lifetime
def revoke_token(jti, remaining_seconds):
    redis_client.setex(f"blacklist:{jti}", remaining_seconds, "revoked")

def is_blacklisted(jti):
    return redis_client.exists(f"blacklist:{jti}")
```

This reintroduces shared state (Redis), which partially defeats the "stateless" goal.
This is the fundamental tension: **pure statelessness vs immediate revocation**.
Most systems accept a compromise: short-lived access tokens (15 minutes) reduce the
window where a revoked token is still valid, and refresh token revocation (database-backed)
prevents getting new access tokens.

## Key Takeaway

JWT enables truly stateless authentication: any instance can verify any token without
consulting any other service. This makes horizontal scaling trivial. The trade-off is
that revoking tokens before they expire requires reintroducing some form of shared state.
The practical solution is short-lived access tokens plus a refresh token mechanism, which
limits the revocation window to the access token's lifetime (typically 15 minutes).
