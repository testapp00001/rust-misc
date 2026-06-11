# Solution 05: Production-Mirror Architecture

## Part A: Network Topology

The architecture requires two isolated networks:

```
[frontend network]
    |
    +--- nginx (reverse proxy)
    +--- api (bridges both networks)
    +--- frontend-static (serves built files)

[backend network]
    |
    +--- api (bridges both networks)
    +--- postgres (database)
    +--- redis (cache)
```

The API is the only service on both networks. It acts as the bridge between the
public-facing frontend and the private backend. This means:

- Nginx can reach the API (both on `frontend`).
- The API can reach PostgreSQL and Redis (both on `backend`).
- Nginx cannot reach PostgreSQL or Redis (different networks).
- The frontend-static server cannot reach the database or cache.

This follows the principle of least privilege: each service only has access to
what it needs.

---

## Part B: Base Compose File

**docker-compose.yml:**

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: ${POSTGRES_DB:-catalog}
      POSTGRES_USER: ${POSTGRES_USER:-catalog_user}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./db/init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-catalog_user} -d ${POSTGRES_DB:-catalog}"]
      interval: 5s
      timeout: 3s
      retries: 5
      start_period: 10s
    restart: unless-stopped
    networks:
      - backend

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5
    restart: unless-stopped
    networks:
      - backend

  api:
    build:
      context: ./api
      dockerfile: Dockerfile
    environment:
      DATABASE_URL: postgresql://${POSTGRES_USER:-catalog_user}:${POSTGRES_PASSWORD}@postgres:5432/${POSTGRES_DB:-catalog}
      REDIS_URL: redis://redis:6379
      NODE_ENV: production
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 3
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: "0.50"
          memory: 256M
        reservations:
          cpus: "0.25"
          memory: 128M
    networks:
      - frontend
      - backend

  frontend-static:
    image: nginx:alpine
    volumes:
      - ./frontend/dist:/usr/share/nginx/html:ro
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:80"]
      interval: 10s
      timeout: 5s
      retries: 3
    restart: unless-stopped
    networks:
      - frontend

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      api:
        condition: service_healthy
      frontend-static:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:80/health"]
      interval: 10s
      timeout: 5s
      retries: 3
    restart: unless-stopped
    networks:
      - frontend

volumes:
  pgdata:
  redis_data:

networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
```

**nginx/nginx.conf:**

```nginx
events {
    worker_connections 1024;
}

http {
    upstream api {
        server api:3000;
    }

    upstream frontend {
        server frontend-static:80;
    }

    server {
        listen 80;

        # API requests
        location /api/ {
            proxy_pass http://api;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Health check for Nginx itself
        location /health {
            return 200 'ok';
            add_header Content-Type text/plain;
        }

        # Everything else goes to the frontend
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```

**db/init.sql:**

```sql
CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL,
    stock INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO products (name, description, price, stock)
SELECT 'Wireless Mouse', 'Ergonomic wireless mouse with USB receiver', 29.99, 150
WHERE NOT EXISTS (SELECT 1 FROM products WHERE name = 'Wireless Mouse');

INSERT INTO products (name, description, price, stock)
SELECT 'Mechanical Keyboard', 'Cherry MX Blue switches, full-size', 89.99, 75
WHERE NOT EXISTS (SELECT 1 FROM products WHERE name = 'Mechanical Keyboard');

INSERT INTO products (name, description, price, stock)
SELECT 'USB-C Hub', '7-in-1 hub with HDMI and ethernet', 45.50, 200
WHERE NOT EXISTS (SELECT 1 FROM products WHERE name = 'USB-C Hub');
```

---

## Part C: Production Override

**docker-compose.prod.yml:**

```yaml
services:
  postgres:
    restart: always
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 1G
        reservations:
          cpus: "0.50"
          memory: 512M
    healthcheck:
      interval: 15s
      timeout: 5s
      retries: 3

  redis:
    restart: always
    deploy:
      resources:
        limits:
          cpus: "0.25"
          memory: 128M

  api:
    restart: always
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 512M
        reservations:
          cpus: "0.50"
          memory: 256M
    healthcheck:
      interval: 15s
      timeout: 5s
      retries: 5

  frontend-static:
    restart: always
    deploy:
      resources:
        limits:
          cpus: "0.25"
          memory: 64M

  nginx:
    restart: always
    deploy:
      resources:
        limits:
          cpus: "0.25"
          memory: 64M

volumes:
  pgdata:
    driver: local
  redis_data:
    driver: local
```

### Why This Works

**`restart: always`**: In production, every service must restart after crashes AND
after host reboots. `always` is the right choice for production services that should
never stay down.

**Increased resource limits**: Production gets more resources because it handles
real traffic. The API gets 1 CPU and 512M instead of 0.5 CPU and 256M. The database
gets 1G of memory for query caching and connection handling.

**Stricter health checks in production**: Longer intervals (15s instead of 5s)
reduce overhead on production services. More retries (5 instead of 3) give transient
issues more time to resolve before marking a service unhealthy.

**Volume driver explicit**: Specifying `driver: local` makes the storage driver
explicit. This is the default, but being explicit in production configs prevents
surprises if the default changes.

---

## Part D: Development Override

**docker-compose.override.yml:**

```yaml
services:
  postgres:
    ports:
      - "5432:5432"

  redis:
    ports:
      - "6379:6379"

  api:
    volumes:
      - ./api/src:/app/src
    command: npx nodemon src/index.js
    environment:
      DATABASE_URL: postgresql://${POSTGRES_USER:-catalog_user}:${POSTGRES_PASSWORD}@postgres:5432/${POSTGRES_DB:-catalog}
      REDIS_URL: redis://redis:6379
      NODE_ENV: development
      DEBUG: "catalog:*"
    ports:
      - "3000:3000"
    deploy:
      resources:
        limits:
          cpus: "0.50"
          memory: 256M

  adminer:
    image: adminer
    ports:
      - "8081:8080"
    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - backend

  redis-commander:
    image: rediscommander/redis-commander
    ports:
      - "8082:8081"
    environment:
      REDIS_HOSTS: local:redis:6379
    depends_on:
      - redis
    networks:
      - backend
```

### Why This Works

**Debug tools on the backend network**: Adminer and Redis Commander need to reach
the database and cache, so they are placed on the `backend` network. They are not
on the `frontend` network because they do not need to be reached from Nginx.

**Hot reload**: The bind mount and `nodemon` command enable instant feedback during
development. The developer edits files on the host, `nodemon` detects the change
inside the container, and the API restarts.

**Relaxed resource limits**: Development gets lower limits because the workload
is just one developer testing, not production traffic.

---

## Part E: End-to-End Verification

### Development Mode

```bash
docker compose up -d
docker compose ps
```

Expected: All 7 services running (postgres, redis, api, frontend-static, nginx,
adminer, redis-commander).

```bash
# Test API through Nginx
curl http://localhost/api/health
# Expected: {"status":"healthy","postgres":"ok","redis":"ok"}

# Test frontend through Nginx
curl http://localhost/
# Expected: HTML from the React/Vue build

# Test network isolation (should fail/timeout)
docker compose exec frontend-static ping -c 1 postgres
# Expected: ping: bad address 'postgres' (DNS resolution fails)

# Verify adminer
curl -s http://localhost:8081 | head -5
# Expected: HTML for Adminer login page
```

### Production Mode

```bash
docker compose down
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
docker compose ps
```

Expected: Only 5 services (no adminer, no redis-commander). No debug ports exposed.
Only port 80 (Nginx) is mapped.

```bash
# Verify only Nginx port is exposed
docker compose ps --format "table {{.Name}}\t{{.Ports}}"
# Expected: only nginx shows 0.0.0.0:80->80/tcp

# Verify API works through Nginx
curl http://localhost/api/health
# Expected: {"status":"healthy"}
```

### Network Isolation Verification

```bash
# From the host, try to reach PostgreSQL directly (should fail)
curl http://localhost:5432
# Expected: connection refused (no port mapping in production)

# From frontend-static, try to reach postgres (should fail)
docker compose exec frontend-static wget -q -O- http://postgres:5432
# Expected: bad address 'postgres' (different network)
```

---

## Architecture Decisions Explained

### Why Two Networks Instead of One?

A single network would work functionally, but it violates the principle of least
privilege. If the frontend static server has a vulnerability (e.g., a path
traversal in Nginx), an attacker on the `frontend` network could potentially
reach the database if it were also on that network. With two networks, the
attacker can only reach the API, which enforces application-level access control.

### Why Nginx as a Reverse Proxy?

Nginx handles several concerns that the application should not:
- TLS termination (add SSL certificates in production)
- Rate limiting
- Static file serving (more efficient than Node.js)
- Request buffering (protects slow backends from fast clients)
- Load balancing (future: multiple API instances)

### Why Health Checks on Every Service?

A health check on the frontend-static server ensures Nginx does not route to a
container that is starting up or has crashed. Without it, Nginx might send
requests to a dead upstream, resulting in 502 errors for users.

### Why Separate Volumes for PostgreSQL and Redis?

Each service gets its own named volume. This allows independent backup, restore,
and lifecycle management. You can `docker compose down -v` to wipe everything,
or selectively remove one volume without affecting the other.

---

## Common Mistakes

### 1. Forgetting to Place Services on Networks

```yaml
# WRONG -- api is not on any custom network, falls back to default
services:
  api:
    build: ./api

# RIGHT -- explicitly place on both networks
services:
  api:
    networks:
      - frontend
      - backend
```

If you define custom networks but forget to assign a service, Compose puts it on
the default network, which is separate from your custom networks. The service
cannot reach any other service.

### 2. Placing Debug Tools on the Wrong Network

```yaml
# WRONG -- adminer is on frontend, cannot reach postgres
adminer:
  networks:
    - frontend

# RIGHT -- adminer needs backend access
adminer:
  networks:
    - backend
```

### 3. Not Using Health Checks with depends_on

```yaml
# WRONG -- nginx starts before api is ready
nginx:
  depends_on:
    - api

# RIGHT -- nginx waits for api health check
nginx:
  depends_on:
    api:
      condition: service_healthy
```

Without `condition: service_healthy`, Nginx starts immediately. If the API is
still initializing, Nginx logs 502 errors until the API comes up.

### 4. Hardcoding Secrets in the Base File

```yaml
# WRONG -- password visible in version control
environment:
  POSTGRES_PASSWORD: supersecret123

# RIGHT -- use variable interpolation
environment:
  POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
```

The `.env` file (which is in `.gitignore`) holds the actual value. The compose
file references it without exposing it.

### 5. Ignoring Resource Limits

Without resource limits, a single misbehaving service can consume all host
resources. In production, this means one buggy endpoint can take down the entire
stack including the database. Always set limits -- they are the container
equivalent of a circuit breaker.

## Relevant README Sections

- [Anatomy of docker-compose.yml](../README.md#anatomy-of-docker-composeyml)
- [Networks](../README.md#networks)
- [Production Way: Health Checks and Dependency Ordering](../README.md#production-way-health-checks-and-dependency-ordering)
- [Override Files](../README.md#override-files)
- [Scaling Services](../README.md#scaling-services)
- [Hands-On Lab: Full-Stack Application](../README.md#hands-on-lab-full-stack-application)
- [Compose File Best Practices](../README.md#compose-file-best-practices)
