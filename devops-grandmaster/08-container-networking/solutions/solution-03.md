# Solution 03: Debug a Networking Issue Between Containers

## Part A: All the Bugs

### Bug 1: `DB_HOST=localhost` in the API service

**What is wrong:** The API connects to `localhost:5432` to reach the
database. Inside the API container, `localhost` refers to the API
container itself, not the host machine and not the database container.

**Why it causes the symptom:** The API tries to connect to PostgreSQL on
its own loopback interface. There is no PostgreSQL running inside the API
container, so the connection is refused.

**Affected component:** API service.

### Bug 2: `proxy_pass http://localhost:5000` in Nginx config

**What is wrong:** The Nginx `proxy_pass` directive points to
`localhost:5000`. Inside the frontend container, `localhost` is the
frontend container itself. The API is running in a separate container.

**Why it causes the symptom:** Nginx tries to forward API requests to
port 5000 on its own loopback. No process is listening there, so the
request fails with a connection error. The frontend displays "Error:
Could not reach API."

**Affected component:** Frontend (Nginx configuration).

### Bug 3: Database and Redis ports exposed to the host

**What is wrong:** The database maps port `5432:5432` and is accessible
from the host network. This is a security risk -- any process on the
host (or the network, if the host's firewall is not configured) can
connect to the database directly.

**Why it is a problem:** It violates the principle of least privilege.
The database should only be reachable from the API and worker containers,
not from the outside world.

**Affected component:** Security posture (not a connectivity bug, but
a critical configuration error).

### Bug 4: No explicit networks defined

**What is wrong:** No networks are defined in the compose file, so all
services end up on the default Compose network. While this does work for
connectivity (Compose default networks do support DNS), it means all
services can reach all other services with no isolation.

**Why it is a problem:** A compromised frontend could directly access the
database. This is not a connectivity bug (things do connect), but it is
a security and architectural problem.

**Affected component:** Network architecture.

---

## Part B: Corrected Configuration

### Corrected `docker-compose.yml`

```yaml
version: "3.9"

services:
  frontend:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./frontend.conf:/etc/nginx/conf.d/default.conf
    networks:
      - frontend-net

  api:
    image: python:3.12-slim
    command: python /app/server.py
    environment:
      - DB_HOST=database
      - DB_PORT=5432
      - DB_NAME=appdb
      - DB_USER=postgres
      - DB_PASSWORD=secret
    networks:
      - frontend-net
      - backend-net

  database:
    image: postgres:16
    environment:
      - POSTGRES_DB=appdb
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=secret
    networks:
      - backend-net

networks:
  frontend-net:
  backend-net:
```

### Corrected `frontend.conf`

```nginx
server {
    listen 80;

    location / {
        root /usr/share/nginx/html;
        index index.html;
    }

    location /api/ {
        proxy_pass http://api:5000/;
    }
}
```

### Key Changes Explained

1. **`DB_HOST=database`** instead of `localhost`. On the `backend-net`
   Docker network, the service name `database` resolves to the PostgreSQL
   container's IP address via the embedded DNS server.

2. **`proxy_pass http://api:5000/`** instead of `localhost:5000`. On the
   `frontend-net` Docker network, the service name `api` resolves to the
   API container's IP address.

3. **Removed port mappings** from `api` and `database`. Only the frontend
   needs to be accessible from the host. The API and database communicate
   internally through Docker networks.

4. **Two named networks:**
   - `frontend-net`: connects `frontend` and `api`
   - `backend-net`: connects `api` and `database`

   The `api` service is on both networks, acting as a bridge between the
   frontend and backend tiers. The `frontend` cannot directly reach the
   `database`.

---

## Part C: Verify the Fix

```bash
# Start the stack
docker compose up -d

# Wait for services to start
sleep 5

# 1. Frontend can reach the API
docker compose exec frontend curl -s http://api:5000/
# Expected: API response (not a connection error)

# 2. API can reach the database
docker compose exec api sh -c "nc -z -w 2 database 5432"
# Expected: exits successfully (0)

# 3. Database is NOT accessible from the host
nc -z -w 2 localhost 5432
# Expected: connection refused or timeout

# 4. Frontend cannot directly reach the database
docker compose exec frontend sh -c "nc -z -w 2 database 5432"
# Expected: connection refused or timeout (different networks)

# 5. Only frontend is exposed on the host
curl -s http://localhost:8080/
# Expected: Nginx default page
```

---

## Part D: Network Traffic Path

```
User's Browser
     |
     | HTTP request to host:8080
     | [PORT MAPPING: host:8080 -> frontend:80]
     v
Frontend (Nginx)
     |
     | proxy_pass http://api:5000/
     | [DOCKER NETWORK: frontend-net, DNS resolves "api" to IP]
     v
API (Python/Flask)
     |
     | Connects to database:5432
     | [DOCKER NETWORK: backend-net, DNS resolves "database" to IP]
     v
Database (PostgreSQL)
     |
     | Query result returned
     v
API -> Frontend -> Browser
```

### Hop-by-Hop Analysis

| Hop | Source | Destination | Network | DNS? | Port Mapping? |
|-----|--------|-------------|---------|------|---------------|
| 1 | Browser | Frontend | Host network -> Docker bridge | No (uses host IP) | Yes (8080:80) |
| 2 | Frontend | API | `frontend-net` (custom bridge) | Yes (`api` -> IP) | No |
| 3 | API | Database | `backend-net` (custom bridge) | Yes (`database` -> IP) | No |

Only the first hop uses port mapping. Hops 2 and 3 are entirely within
Docker's internal networking, with DNS resolution handled by the embedded
DNS server at `127.0.0.11` in each container.

---

## Common Mistakes People Make with This Exercise

1. **Changing `localhost` to `172.17.0.x` (IP address).** This works until
   the container restarts and gets a different IP. Always use service names.

2. **Putting all services on one network.** It fixes connectivity but
   removes isolation. The frontend should not be able to reach the database.

3. **Keeping the database port mapping "for debugging."** If you need to
   connect a GUI tool, bind to localhost only: `"127.0.0.1:5432:5432"`.

4. **Forgetting to update the Nginx config volume.** If you change
   `frontend.conf` but do not rebuild or restart the frontend container,
   Nginx still uses the old config.
