# Exercise 03: Implement JWT-Based Stateless Authentication

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Implement a complete JWT-based authentication system for a Flask application. This
exercise teaches you how stateless authentication works, how tokens are issued and
verified, and how to handle token expiration and refresh.

## Prerequisites

- Docker installed
- Basic understanding of HTTP headers and JSON
- Familiarity with the concept of HMAC signatures

## Tasks

### Part A: Build the JWT Authentication Server

Create a file called `app_jwt.py` that implements the following endpoints:

**POST /login**
- Accept `{"username": "...", "password": "..."}`
- Validate credentials (hardcode: `admin`/`secret` and `user`/`password`)
- On success, return an access token (expires in 15 minutes) and a refresh token (expires in 7 days)
- On failure, return 401

**GET /profile**
- Require a valid JWT in the `Authorization: Bearer <token>` header
- Return the user's ID, username, and role from the token payload
- Return 401 if the token is missing, invalid, or expired

**POST /refresh**
- Accept `{"refresh_token": "..."}`
- If the refresh token is valid and not expired, issue a new access token
- Return 401 if the refresh token is invalid or expired

**POST /logout** (bonus)
- Implement token revocation using a blacklist stored in memory
- After logout, the access token should be rejected even if not expired

Requirements:
- Use the `PyJWT` library (`import jwt`)
- Use HS256 algorithm with a configurable secret key
- Include `user_id`, `username`, `role`, and `exp` claims in access tokens
- Include `user_id`, `type: "refresh"`, and `exp` claims in refresh tokens
- Include a `jti` (JWT ID) claim in every token for identification

<details>
<summary>Hint</summary>

To generate a unique jti for each token:
```python
import uuid
token_id = str(uuid.uuid4())
```

To encode a JWT:
```python
import jwt
import datetime

payload = {
    'user_id': 1,
    'username': 'admin',
    'role': 'admin',
    'jti': str(uuid.uuid4()),
    'exp': datetime.datetime.utcnow() + datetime.timedelta(minutes=15)
}
token = jwt.encode(payload, SECRET_KEY, algorithm='HS256')
```

To decode and verify:
```python
try:
    payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])
except jwt.ExpiredSignatureError:
    # Token has expired
except jwt.InvalidTokenError:
    # Token is invalid
```

</details>

### Part B: Create the Docker Setup

Create a `Dockerfile` and `docker-compose.yml` with:
- The Flask app running on 3 replicas
- An nginx load balancer distributing requests across all 3 replicas
- No session affinity (no sticky sessions)

<details>
<summary>Hint</summary>

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask PyJWT
COPY app_jwt.py .
CMD ["python", "app_jwt.py"]
```

Your nginx upstream should have 3 servers. Since JWT is stateless, any instance
can verify any token -- no shared state needed.

</details>

### Part C: Write a Test Script

Create a shell script called `test_jwt.sh` that:

1. Logs in and captures the access token and refresh token
2. Makes 10 requests to `/profile` using the access token (all should succeed)
3. Waits for the access token to expire (set expiration to 30 seconds for testing)
4. Tries `/profile` with the expired token (should fail with 401)
5. Uses the refresh token to get a new access token
6. Uses the new access token to access `/profile` (should succeed)

<details>
<summary>Hint</summary>

```bash
# Login
RESPONSE=$(curl -s -X POST http://localhost/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}')

ACCESS_TOKEN=$(echo $RESPONSE | jq -r '.access_token')
REFRESH_TOKEN=$(echo $RESPONSE | jq -r '.refresh_token')

# Use access token
curl -s http://localhost/profile \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

For the expiration test, set the access token TTL to 30 seconds during development
so you do not have to wait 15 minutes.

</details>

### Part D: Implement Token Revocation (Bonus)

Add an in-memory blacklist to your application. When a user calls POST /logout:

1. Extract the `jti` from their access token
2. Add it to the blacklist with an expiration matching the token's remaining TTL
3. On every `/profile` request, check if the token's `jti` is in the blacklist
4. If it is, return 401 even though the token signature is valid

Discuss: Why is in-memory blacklisting problematic for a multi-instance deployment?
What would you use in production?

<details>
<summary>Hint</summary>

In-memory blacklists do not work across instances. If instance 1 blacklists a token,
instance 2 does not know about it. In production, you would use Redis for the
blacklist -- but then you have reintroduced shared state, partially defeating the
purpose of stateless JWT.

This is the fundamental tension: pure statelessness vs immediate revocation.

</details>

## Success Criteria

- [ ] POST /login returns both an access token and a refresh token
- [ ] GET /profile with a valid token returns user data from the token payload
- [ ] GET /profile without a token returns 401
- [ ] GET /profile with an expired token returns 401
- [ ] POST /refresh with a valid refresh token returns a new access token
- [ ] POST /refresh with an expired refresh token returns 401
- [ ] Any instance can verify any token (no shared state between instances)
- [ ] The test script passes all assertions

## What You Should Understand After This Exercise

JWT puts authentication state into the token itself. The server only needs the secret
key to verify a token -- no database lookup, no Redis query, no shared state. This
makes horizontal scaling trivial: add instances, and they immediately work. The cost
is that revoking tokens before they expire requires reintroducing some form of shared
state (a blacklist), which is the fundamental trade-off of stateless authentication.
