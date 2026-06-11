# Exercise 05: Production-Mirror Architecture

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design a complete Compose setup that mirrors a production architecture: a reverse
proxy, a load-balanced API, a database with persistent storage, a cache, proper
network isolation, resource limits, and environment-specific override files. This
exercise combines everything from Module 07 with concepts from earlier modules.

## Scenario

You are deploying a product catalog service for an e-commerce platform. The
production architecture looks like this:

```
Internet --> Nginx (port 80)
                |
                +--> /api/* --> API (Node.js, 3 replicas)
                |                  |
                |                  +--> PostgreSQL (product catalog)
                |                  +--> Redis (session cache)
                |
                +--> /* --> Static Frontend (Nginx serving built files)
```

Requirements:
- Nginx reverse proxy routes `/api/*` to the API and everything else to the frontend.
- The API has 3 replicas for load balancing (simulated in Compose).
- PostgreSQL stores product data with a named volume.
- Redis caches API responses.
- Backend services are isolated from the frontend network.
- Each service has health checks, restart policies, and resource limits.
- A production override file configures replicas and stricter limits.
- A development override file enables hot reload and debug tools.

## Tasks

### Part A: Design the Network Topology

Create two custom networks:
- `frontend`: connects Nginx, the API, and the frontend static server.
- `backend`: connects the API, PostgreSQL, and Redis.

The API service bridges both networks. Database and cache services are on the
backend network only -- they should not be reachable from the frontend network.

Draw the network diagram and then implement it in YAML.

<details>
<summary>Hint 1: Network Configuration</summary>

```yaml
services:
  postgres:
    networks:
      - backend

  redis:
    networks:
      - backend

  api:
    networks:
      - backend
      - frontend

  nginx:
    networks:
      - frontend

  frontend-static:
    networks:
      - frontend

networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
```

</details>

<details>
<summary>Hint 2: Why Isolate Networks?</summary>

Network isolation is a security measure. If the frontend service is compromised,
the attacker cannot directly reach the database -- they can only talk to the API,
which enforces application-level access control. This follows the principle of
least privilege.

</details>

### Part B: Implement the Base Compose File

Create `docker-compose.yml` with all five services:

1. **postgres**: PostgreSQL 16 with health check, named volume, backend network only.
2. **redis**: Redis 7 with health check, AOF persistence, backend network only.
3. **api**: Node.js API with health check, depends on healthy postgres and redis,
   both networks, resource limits.
4. **frontend-static**: Nginx serving static files from a `./frontend/dist` directory,
   frontend network only.
5. **nginx**: Reverse proxy with custom config, ports 80 exposed, depends on api and
   frontend-static, frontend network only.

<details>
<summary>Hint 1: Nginx Reverse Proxy Config</summary>

Create `nginx/nginx.conf`:

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

        location /api/ {
            proxy_pass http://api;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
        }

        location /health {
            return 200 'ok';
            add_header Content-Type text/plain;
        }
    }
}
```

</details>

<details>
<summary>Hint 2: API Resource Limits</summary>

```yaml
services:
  api:
    deploy:
      resources:
        limits:
          cpus: "0.50"
          memory: 256M
        reservations:
          cpus: "0.25"
          memory: 128M
```

Note: `deploy.resources` works with `docker compose up` in Compose V2.
In older versions, it only worked with Docker Swarm.

</details>

### Part C: Create the Production Override

Create `docker-compose.prod.yml` that:

1. Sets `restart: always` on all services.
2. Increases resource limits (e.g., 1 CPU, 512M for API).
3. Removes any development-only settings.
4. Adds stricter health check intervals.

<details>
<summary>Hint</summary>

Production override usage:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

The second file merges with and overrides the first. You only need to specify
the values that change.

</details>

### Part D: Create the Development Override

Create `docker-compose.override.yml` (auto-loaded) that:

1. Exposes PostgreSQL port 5432 to the host.
2. Exposes Redis port 6379 to the host.
3. Adds hot reload for the API via bind mount.
4. Adds Adminer for database debugging.
5. Relaxes resource limits for development.

<details>
<summary>Hint</summary>

The override file is loaded automatically by `docker compose up`. You do not
need to specify it with `-f`. To run without the override:

```bash
docker compose -f docker-compose.yml up -d
```

</details>

### Part E: End-to-End Verification

Run the complete verification suite:

```bash
# Start in development mode
docker compose up -d

# Verify all services are healthy
docker compose ps

# Test the API through Nginx
curl http://localhost/api/health

# Test the frontend through Nginx
curl http://localhost/

# Verify network isolation (should fail)
docker compose exec frontend-static ping postgres

# Test hot reload
# Edit ./api/src/index.js and watch the API restart
docker compose logs -f api

# Test production mode
docker compose down
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
docker compose ps
```

Document the results of each test.

<details>
<summary>Hint</summary>

The `ping postgres` test from `frontend-static` should fail or time out because
`frontend-static` is only on the `frontend` network, and `postgres` is only on
the `backend` network. If it succeeds, your network isolation is not working.

</details>

## Success Criteria

- [ ] Five services are defined: postgres, redis, api, frontend-static, nginx
- [ ] Two custom networks (`frontend`, `backend`) provide isolation
- [ ] PostgreSQL and Redis are unreachable from the frontend network
- [ ] All services have health checks and restart policies
- [ ] The API has CPU and memory resource limits
- [ ] `docker-compose.prod.yml` configures production settings
- [ ] `docker-compose.override.yml` enables hot reload and debug tools
- [ ] Nginx correctly routes `/api/*` to the API and `/` to the frontend
- [ ] Hot reload works in development mode
- [ ] You can explain why each architectural decision was made

## What You Should Understand After This Exercise

A production-like Compose setup is more than a list of containers. Network isolation
limits blast radius. Health checks prevent cascading failures. Resource limits prevent
resource starvation. Override files separate development agility from production
stability. Nginx as a reverse proxy decouples external routing from internal service
discovery. Together, these patterns form the foundation for container orchestration
at scale -- the topic of later modules.
