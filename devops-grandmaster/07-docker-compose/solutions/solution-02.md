# Solution 02: Build a Three-Service Stack

## Part A: The Complete Compose File

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: taskdb
      POSTGRES_USER: taskuser
      POSTGRES_PASSWORD: taskpass
    volumes:
      - pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine

  api:
    image: node:20-alpine
    command: sleep infinity
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://taskuser:taskpass@postgres:5432/taskdb
      REDIS_URL: redis://redis:6379

volumes:
  pgdata:
```

### Why This Works

**Services section**: Each top-level key under `services` defines a container.
The key name (`postgres`, `redis`, `api`) becomes the hostname that other
containers use to connect to it.

**Environment variables**: PostgreSQL uses specific environment variables
(`POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD`) that the `postgres:16-alpine`
image reads on first start to create the database and user.

**Service names as hostnames**: The `DATABASE_URL` uses `postgres` as the hostname
instead of an IP address. Compose creates a shared network and registers DNS entries
for each service. Inside the `api` container, `postgres` resolves to the PostgreSQL
container's IP address.

**Named volume**: `pgdata` is declared under the `postgres` service as a mount
point AND at the top level under `volumes:`. The mount syntax is
`volume_name:container_path`. The top-level declaration tells Compose to create
and manage the volume.

**Port mapping**: `"3000:3000"` maps host port 3000 to container port 3000. The
`api` service uses `sleep infinity` as a placeholder command since we are not
building a real application yet.

---

## Part B: Validation and Startup

Expected output from `docker compose config`:

```yaml
services:
  api:
    command:
      - sleep
      - infinity
    environment:
      DATABASE_URL: postgresql://taskuser:taskpass@postgres:5432/taskdb
      REDIS_URL: redis://redis:6379
    image: node:20-alpine
    networks:
      default: null
    ports:
      - mode: ingress
        host_ip: "0.0.0.0"
        target: 3000
        published: "3000"
        protocol: tcp
  postgres:
    environment:
      POSTGRES_DB: taskdb
      POSTGRES_PASSWORD: taskpass
      POSTGRES_USER: taskuser
    image: postgres:16-alpine
    networks:
      default: null
    volumes:
      - type: volume
        source: pgdata
        target: /var/lib/postgresql/data
  redis:
    image: redis:7-alpine
    networks:
      default: null
networks:
  default:
    name: myproject_default
volumes:
  pgdata:
    name: myproject_pgdata
```

Expected output from `docker compose ps`:

```
NAME                  IMAGE               STATUS          PORTS
myproject-api-1       node:20-alpine      Up 2 minutes    0.0.0.0:3000->3000/tcp
myproject-postgres-1  postgres:16-alpine  Up 2 minutes    5432/tcp
myproject-redis-1     redis:7-alpine      Up 2 minutes    6379/tcp
```

---

## Part C: Connectivity Test

Expected results when running `ping postgres` and `ping redis` from inside the
`api` container:

```
PING postgres (172.18.0.2): 56 data bytes
64 bytes from 172.18.0.2: seq=0 ttl=64 time=0.089 ms

PING redis (172.18.0.3): 56 data bytes
64 bytes from 172.18.0.3: seq=0 ttl=64 time=0.072 ms
```

The IP addresses will vary, but the key point is that `postgres` and `redis`
resolve to actual IP addresses. This is Compose's built-in DNS at work.

---

## Part D: Network Inspection

The network name follows the pattern `<project-name>_default` where the project
name is the directory name. For example, if you created the file in a directory
called `task-app`, the network is `task-app_default`.

The `docker network inspect` output shows:
- The network's subnet (e.g., `172.18.0.0/16`)
- All three containers connected to it
- Each container's IP address within the subnet

All three containers are on the same network, which is why they can reach each
other by service name.

---

## Part E: Clean Up

`docker compose down` removes containers and the default network, but preserves
named volumes. This is by design -- named volumes contain data you want to keep
(database files, uploaded content, etc.).

`docker compose down -v` also removes named volumes. Use this when you want a
completely clean slate.

---

## Common Mistakes

### 1. Forgetting the Top-Level `volumes:` Key

```yaml
# WRONG -- volume mount exists but volume is never declared
services:
  postgres:
    volumes:
      - pgdata:/var/lib/postgresql/data

# No volumes: section at the bottom
```

This creates an anonymous volume instead of the named one. The data is hard to
manage and may not survive `docker compose down`.

### 2. Using `localhost` Instead of Service Name

```yaml
# WRONG -- localhost refers to the container itself, not the host
environment:
  DATABASE_URL: postgresql://taskuser:taskpass@localhost:5432/taskdb

# RIGHT -- use the service name as hostname
environment:
  DATABASE_URL: postgresql://taskuser:taskpass@postgres:5432/taskdb
```

Inside a container, `localhost` means "this container." To reach another container,
use its service name.

### 3. Exposing Unnecessary Ports

```yaml
# RISKY -- exposes the database to the host (and potentially the network)
postgres:
  ports:
    - "5432:5432"

# BETTER -- only expose what the host needs to reach
api:
  ports:
    - "3000:3000"
```

Only the API needs to be accessible from the host. PostgreSQL and Redis should
only be reachable from within the Compose network.

### 4. Using `latest` Tags

```yaml
# BAD -- unpredictable, may break without warning
image: postgres:latest

# GOOD -- predictable, reproducible
image: postgres:16-alpine
```

### 5. Wrong Volume Mount Syntax

```yaml
# WRONG -- reversed order (container path first)
volumes:
  - /var/lib/postgresql/data:pgdata

# RIGHT -- host/volume name first, container path second
volumes:
  - pgdata:/var/lib/postgresql/data
```

The syntax is `source:destination`, matching the pattern of `docker run -v`.

## Relevant README Sections

- [Anatomy of docker-compose.yml](../README.md#anatomy-of-docker-composeyml)
- [The `services` Section](../README.md#the-services-section)
- [Building vs. Pulling](../README.md#building-vs-pulling)
- [Port Mapping](../README.md#port-mapping)
- [Environment Variables](../README.md#environment-variables)
- [Volumes](../README.md#volumes)
- [Networks](../README.md#networks)
