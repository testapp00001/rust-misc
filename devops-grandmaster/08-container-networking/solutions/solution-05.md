# Solution 05: Implement Network Segmentation for Security

## Part A: Service Classification

| Service | Visibility | Needs to Reach | Tier |
|---------|-----------|----------------|------|
| frontend | Public (internet) | api | Public |
| api | Internal only | database, redis | Application |
| worker | Internal only | database, redis | Application |
| database | Internal only | (nothing) | Data |
| redis | Internal only | (nothing) | Data |
| admin-panel | Host only (internal tool) | database, redis | Data (management) |

### Reasoning

- **frontend** is the entry point for users. It must be accessible from
  the internet on ports 80 and 443.
- **api** and **worker** are backend services. They should never be
  directly accessible from outside Docker.
- **database** and **redis** are data stores. They should only be
  reachable from the services that use them.
- **admin-panel** is a management tool. It should be accessible from
  the host (for developers) but not from the internet.

---

## Part B: Network Architecture

| Network | Services | Purpose |
|---------|----------|---------|
| `public-net` | frontend | Isolates public-facing traffic. Only the frontend is here. |
| `app-net` | frontend, api, worker | Application tier. Frontend proxies to API. API and worker share this for service communication. |
| `data-net` | api, worker, database, redis, admin-panel | Data tier. Application services reach data stores. Admin panel reaches databases for management. |

### Why Three Networks

- `public-net` exists so the frontend is isolated from the backend when
  accessed from outside. However, since the frontend needs to reach the
  API, we also put it on `app-net`.
- `app-net` connects the application services. The frontend can reach the
  API here. The API and worker can communicate if needed.
- `data-net` connects application services to their data stores. The
  admin-panel is here because it needs to manage databases.
- The frontend is NOT on `data-net`, so it cannot reach databases or Redis.
- The admin-panel is NOT on `public-net` or `app-net`, so it cannot be
  reached from the internet or from the frontend.

### Multi-Network Assignments

| Service | Networks |
|---------|----------|
| frontend | public-net, app-net |
| api | app-net, data-net |
| worker | app-net, data-net |
| database | data-net |
| redis | data-net |
| admin-panel | data-net |

---

## Part C: Secure Docker Compose Configuration

```yaml
version: "3.9"

services:
  frontend:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    networks:
      - public-net
      - app-net
    depends_on:
      - api

  api:
    image: python:3.12-slim
    command: python /app/server.py
    # NO ports mapping -- not accessible from host
    networks:
      - app-net
      - data-net
    depends_on:
      - database
      - redis

  worker:
    image: python:3.12-slim
    command: python /app/worker.py
    # NO ports mapping -- not accessible from host
    networks:
      - app-net
      - data-net
    depends_on:
      - database
      - redis

  database:
    image: postgres:16
    # NO ports mapping -- not accessible from host network
    environment:
      POSTGRES_PASSWORD: secret
    networks:
      - data-net

  redis:
    image: redis:7-alpine
    # NO ports mapping -- not accessible from host network
    networks:
      - data-net

  admin-panel:
    image: adminer:latest
    ports:
      - "127.0.0.1:8080:8080"  # Bound to localhost only
    networks:
      - data-net

networks:
  public-net:
  app-net:
  data-net:
```

### Key Security Changes

1. **Removed all port mappings** from `database` and `redis`. They are no
   longer accessible from the host network at all.

2. **Removed port mapping** from `api` and `worker`. They are internal
   services -- only the frontend needs to be publicly accessible.

3. **Bound admin-panel to `127.0.0.1:8080:8080`.** The `127.0.0.1` prefix
   means the port is only accessible from the host's loopback interface.
   Someone on the same network cannot reach it.

4. **Placed frontend on `public-net` and `app-net`.** The frontend can
   receive public traffic and forward it to the API, but it cannot reach
   the data tier.

5. **Placed admin-panel only on `data-net`.** It can manage databases but
   is not accessible from the application tier or the public tier.

---

## Part D: Security Verification Script

```bash
#!/bin/bash
# verify-security-posture.sh
# Tests that the network segmentation enforces the security policy

PASS=0
FAIL=0

check_port_open() {
    local description="$1"
    local host="$2"
    local port="$3"

    if nc -z -w 2 "$host" "$port" 2>/dev/null; then
        echo "PASS: $description (port is open)"
        PASS=$((PASS + 1))
    else
        echo "FAIL: $description (port should be open but is closed)"
        FAIL=$((FAIL + 1))
    fi
}

check_port_closed() {
    local description="$1"
    local host="$2"
    local port="$3"

    if nc -z -w 2 "$host" "$port" 2>/dev/null; then
        echo "FAIL: $description (port is open but should be closed)"
        FAIL=$((FAIL + 1))
    else
        echo "PASS: $description (port is closed)"
        PASS=$((PASS + 1))
    fi
}

check_container_to_container() {
    local description="$1"
    local from_container="$2"
    local to_host="$3"
    local to_port="$4"
    local should_work="$5"

    if docker compose exec "$from_container" sh -c "nc -z -w 2 $to_host $to_port" 2>/dev/null; then
        if [ "$should_work" = "yes" ]; then
            echo "PASS: $description"
            PASS=$((PASS + 1))
        else
            echo "FAIL: $description (connection succeeded but should be blocked)"
            FAIL=$((FAIL + 1))
        fi
    else
        if [ "$should_work" = "no" ]; then
            echo "PASS: $description (correctly blocked)"
            PASS=$((PASS + 1))
        else
            echo "FAIL: $description (connection failed but should work)"
            FAIL=$((FAIL + 1))
        fi
    fi
}

echo "=== Test 1-3: Host-level port exposure ==="

check_port_open "Frontend reachable on port 80" localhost 80

check_port_closed "Database NOT reachable on port 5432" localhost 5432

check_port_closed "Redis NOT reachable on port 6379" localhost 6379

echo ""
echo "=== Test 4-5: Frontend isolation from data tier ==="

check_container_to_container "Frontend CANNOT reach database" \
    frontend database 5432 no

check_container_to_container "Frontend CANNOT reach Redis" \
    frontend redis 6379 no

echo ""
echo "=== Test 6: Admin panel localhost binding ==="

check_port_open "Admin panel reachable on localhost" localhost 8080

echo ""
echo "=== Test 7-10: Application tier data access ==="

check_container_to_container "API CAN reach database" \
    api database 5432 yes

check_container_to_container "API CAN reach Redis" \
    api redis 6379 yes

check_container_to_container "Worker CAN reach database" \
    worker database 5432 yes

check_container_to_container "Worker CAN reach Redis" \
    worker redis 6379 yes

echo ""
echo "=== Results ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo "Total:  $((PASS + FAIL))"

if [ "$FAIL" -gt 0 ]; then
    echo ""
    echo "SECURITY POSTURE: FAILED"
    exit 1
else
    echo ""
    echo "SECURITY POSTURE: PASSED"
fi
```

### Running the Script

```bash
# Start the stack
docker compose up -d

# Wait for services to initialize
sleep 10

# Run verification
chmod +x verify-security-posture.sh
./verify-security-posture.sh
```

---

## Part E: Security Questions Answered

### 1. If an attacker compromises the frontend container

**What they can reach:**
- The `api` service (via `app-net`)
- The frontend's own filesystem and processes

**What they CANNOT reach:**
- The database (frontend is not on `data-net`)
- Redis (frontend is not on `data-net`)
- The admin panel (not on `app-net` or `public-net`)
- The worker (only accessible via `app-net`, but the attacker could
  potentially reach it)

**Blast radius:** Limited to the public and application tiers. The data
tier is protected by network segmentation.

### 2. If an attacker compromises the API container

**What they can reach:**
- The database (via `data-net`)
- Redis (via `data-net`)
- The admin panel (via `data-net`)
- The worker (via `app-net`)
- The frontend (via `app-net`)

**What is NOT affected:**
- The host network (no port mappings on the API)
- Other hosts on the network

**Blast radius:** The entire application. The API is the most privileged
service because it bridges `app-net` and `data-net`. This is why the API
should have the most rigorous security controls: least-privilege user,
read-only filesystem, regular image scanning, and strict input validation.

### 3. Why `127.0.0.1` binding is not sufficient

Binding to `127.0.0.1` prevents external network access, but:
- Any process on the host can still reach the admin panel
- If an attacker gains shell access to the host, they can use the admin
  panel to exfiltrate or destroy data
- The admin panel itself might have vulnerabilities

**Additional measures for production:**
- Put the admin panel behind authentication (e.g., HTTP basic auth or
  a VPN)
- Run it only when needed (scale to 0 in production, scale to 1 for
  maintenance)
- Use a reverse proxy with access control
- Audit admin panel access logs
- Consider using a bastion host or SSH tunnel instead

### 4. Response to the intern

> "I understand the temptation -- one network means you can reach everything
> easily. But the reason we separate networks is the same reason we have
> separate locks on different doors in a building. If someone picks the lock
> on the front door (compromises the frontend), they should not automatically
> have access to the safe (the database).
>
> With one network, a single compromised container gives an attacker access
> to everything. With our segmentation, compromising the frontend only gives
> access to the API -- the database is on a different network that the
> frontend cannot reach.
>
> For debugging, you can temporarily connect to any container using
> `docker compose exec` and test connectivity from there. You do not need
> everything on one network to debug."

---

## Common Mistakes

1. **Removing port mappings without updating connection strings.** When you
   remove a port mapping, tools that connected via `localhost:5432` can no
   longer reach the database. They need to connect through the Docker
   network using the service name.

2. **Forgetting that `docker compose exec` bypasses network isolation.**
   Running `docker compose exec database psql` works even if the database
   has no port mapping. This is by design -- exec enters the container's
   namespace directly.

3. **Over-segmenting.** Creating a network for every pair of services makes
   the configuration hard to understand and maintain. Use tiers (public,
   application, data) and only create additional networks when a specific
   isolation rule requires it.

4. **Not testing the negative cases.** It is not enough to verify that
   services can reach what they need. You must also verify that they
   CANNOT reach what they should not. The verification script tests both
   directions.

5. **Binding to `0.0.0.0` by default.** Docker's default behavior is to
   bind ports to all interfaces. Always use `127.0.0.1:` prefix for
   services that should only be accessible from the host.
