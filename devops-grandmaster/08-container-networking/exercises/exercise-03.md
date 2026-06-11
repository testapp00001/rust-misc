# Exercise 03: Debug a Networking Issue Between Containers

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Given a broken multi-container application, diagnose and fix the networking
problems. This exercise simulates the most common networking bugs you will
encounter in real Docker environments.

---

## Scenario

A teammate built a three-tier application with Docker Compose. The stack
has a frontend (Nginx), a backend API (Python/Flask), and a database
(PostgreSQL). The teammate says:

> "The frontend loads, but it shows 'Error: Could not reach API.' The API
> logs say 'Connection to database refused.' I do not understand -- all
> three containers are running."

Here is their `docker-compose.yml`:

```yaml
version: "3.9"

services:
  frontend:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./frontend.conf:/etc/nginx/conf.d/default.conf

  api:
    image: python:3.12-slim
    command: python /app/server.py
    ports:
      - "5000:5000"
    environment:
      - DB_HOST=localhost
      - DB_PORT=5432
      - DB_NAME=appdb
      - DB_USER=postgres
      - DB_PASSWORD=secret

  database:
    image: postgres:16
    environment:
      - POSTGRES_DB=appdb
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=secret
    ports:
      - "5432:5432"
```

The `frontend.conf` for Nginx:

```nginx
server {
    listen 80;

    location / {
        root /usr/share/nginx/html;
        index index.html;
    }

    location /api/ {
        proxy_pass http://localhost:5000/;
    }
}
```

---

## Tasks

### Part A: Identify All the Bugs

There are **at least four** networking problems in this configuration. Find
all of them. For each problem, explain:

1. What is wrong
2. Why it causes the symptom described
3. Which component is affected

<details>
<summary>Hint 1</summary>

Look at `DB_HOST=localhost` in the API service. Where is "localhost" from
the API container's perspective? Is the database on the same "localhost"?

</details>

<details>
<summary>Hint 2</summary>

Look at the Nginx `proxy_pass` directive. It says `http://localhost:5000`.
From the frontend container's perspective, is the API running on localhost?

</details>

<details>
<summary>Hint 3</summary>

Think about what Docker Compose creates automatically. Do services share a
network? What DNS names are available?

</details>

<details>
<summary>Hint 4</summary>

Think about whether all three services actually need their ports mapped
to the host. Which mappings are necessary and which are security risks?

</details>

### Part B: Fix the Configuration

Write the corrected `docker-compose.yml` and `frontend.conf`. Your fix must:

1. Allow the frontend to reach the API
2. Allow the API to reach the database
3. Only expose the frontend port (8080) to the host
4. Use container names for service discovery (not IP addresses)

<details>
<summary>Hint</summary>

Docker Compose puts all services in the same default network and provides
DNS resolution using the service name. Change `localhost` references to
the appropriate service names. Remove unnecessary port mappings.

</details>

### Part C: Verify the Fix

After applying your fix, describe the exact commands you would run to verify
each connectivity path works:

1. Frontend can reach the API
2. API can reach the database
3. Database is NOT accessible from the host (except via Docker exec)

<details>
<summary>Hint</summary>

Use `docker compose exec` to run commands inside containers. Use `curl`
to test HTTP endpoints. Use `nc` (netcat) or `pg_isready` to test
database connectivity.

</details>

### Part D: Explain the Network Traffic Path

After the fix, trace the complete network path for an API request:

```
User's browser -> ??? -> ??? -> ??? -> response
```

For each hop, identify:
- Which network the traffic is on
- Whether DNS resolution happens
- Whether port mapping is involved

<details>
<summary>Hint</summary>

There are three hops: browser to frontend, frontend to API, API to database.
Only the first hop uses port mapping. The other two use Docker's internal
network.

</details>

---

## Success Criteria

- [ ] You identified all four networking bugs
- [ ] Your corrected docker-compose.yml resolves all issues
- [ ] Your Nginx config uses the correct hostname for the API
- [ ] Only the frontend port is exposed to the host
- [ ] You can describe the complete request path from browser to database
- [ ] You understand why `localhost` inside a container does not mean the host machine

## Common Mistakes to Watch For

1. Using `localhost` to refer to another container -- inside a container,
   `localhost` means *that container*, not the host or another container.
2. Exposing database ports to the host when they are not needed for external
   access.
3. Not realizing Docker Compose provides automatic DNS resolution using
   service names.
4. Confusing port mapping (host access) with network connectivity
   (container-to-container access).

## What You Should Understand After This Exercise

The most common Docker networking bug is using `localhost` to refer to
another container. Every container has its own network namespace with its
own `localhost`. Container-to-container communication uses the container
name (or service name in Compose) as the hostname. Port mapping is only
for external access.
