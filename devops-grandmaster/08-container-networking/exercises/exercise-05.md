# Exercise 05: Implement Network Segmentation for Security

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Implement network segmentation for a production-like application stack
where security is the primary concern. You will enforce the principle of
least privilege at the network level, implement internal-only services,
and verify that the attack surface is minimized.

---

## Scenario

Your security team has audited your Docker deployment and found these issues:

1. **All containers are on the default bridge network.** Any container can
   reach any other container.
2. **Database ports are mapped to the host.** PostgreSQL (5432) and Redis
   (6379) are accessible from any machine that can reach the host.
3. **No network segmentation.** A compromised frontend container could
   directly access the database.
4. **Services use `--privileged` or unnecessary capabilities.** (You will
   not fix this one here, but be aware of it.)

Your job is to fix the networking layer.

---

## The Current (Insecure) Configuration

```yaml
version: "3.9"

services:
  frontend:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"

  api:
    image: python:3.12-slim
    command: python /app/server.py
    ports:
      - "5000:5000"

  worker:
    image: python:3.12-slim
    command: python /app/worker.py
    ports:
      - "5001:5000"

  database:
    image: postgres:16
    ports:
      - "5432:5432"
    environment:
      POSTGRES_PASSWORD: secret

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  admin-panel:
    image: adminer:latest
    ports:
      - "8080:8080"
```

---

## Tasks

### Part A: Classify Each Service

For each service, determine:

1. **Visibility**: Should it be accessible from the internet, from the host only, or only from other containers?
2. **Required connections**: Which other services does it need to reach?
3. **Network tier**: Public, application, or data?

Fill in this table:

| Service | Visibility | Needs to Reach | Tier |
|---------|-----------|----------------|------|
| frontend | | | |
| api | | | |
| worker | | | |
| database | | | |
| redis | | | |
| admin-panel | | | |

<details>
<summary>Hint</summary>

- `frontend` is the public entry point. It proxies to the API.
- `api` handles requests and talks to the database and Redis.
- `worker` processes background jobs and talks to the database and Redis.
- `database` and `redis` should only be reachable from the application tier.
- `admin-panel` is for internal use only -- it should be accessible from
  the host but NOT from the internet.

</details>

### Part B: Design the Secure Network Architecture

Design networks that enforce the isolation requirements. For each network:

1. Give it a descriptive name
2. List which services are connected
3. Explain what isolation it provides

<details>
<summary>Hint</summary>

Consider three tiers:
- A public-facing network for the frontend
- An application network for the API and worker
- A data network for the database and Redis

The `admin-panel` needs access to the data network (to manage the database)
but should not be on the public network.

</details>

### Part C: Implement the Secure Configuration

Write the corrected `docker-compose.yml`. Your implementation must:

1. Define named networks for each tier
2. Remove ALL unnecessary port mappings (databases must NOT be on the host)
3. Remove the admin-panel from the public network
4. Map only the frontend's ports (80 and 443) to the host
5. Map the admin-panel to `127.0.0.1` only (accessible from the host for
   debugging, but not from the network)
6. Assign each service to the correct network(s)
7. Ensure the `api` and `worker` can reach both `database` and `redis`
8. Ensure the `admin-panel` can reach `database` and `redis`

<details>
<summary>Hint</summary>

To bind a port to localhost only:
```yaml
ports:
  - "127.0.0.1:8080:8080"
```

For multi-network services, use a list:
```yaml
api:
  networks:
    - app-net
    - data-net
```

</details>

### Part D: Write a Security Verification Script

Write a bash script that verifies the security posture. The script should
test:

1. Frontend is reachable on port 80 from outside (simulated)
2. Database is NOT reachable on port 5432 from the host network
3. Redis is NOT reachable on port 6379 from the host network
4. Frontend cannot directly reach the database
5. Frontend cannot directly reach Redis
6. Admin panel is reachable on localhost only
7. API can reach the database
8. API can reach Redis
9. Worker can reach the database
10. Worker can reach Redis

Each test should print a clear PASS or FAIL with a description.

<details>
<summary>Hint</summary>

For host-level port tests, use `nc -z` from the host:
```bash
nc -z -w 2 localhost 5432 && echo "FAIL: DB exposed" || echo "PASS: DB not exposed"
```

For container-to-container tests, use `docker compose exec`:
```bash
docker compose exec frontend sh -c "nc -z -w 2 database 5432" \
  && echo "FAIL: frontend can reach DB" \
  || echo "PASS: frontend cannot reach DB"
```

</details>

### Part E: Answer the Security Questions

1. If an attacker compromises the `frontend` container, what can they reach?
   What can they NOT reach?
2. If an attacker compromises the `api` container, what is the blast radius?
   What is NOT affected?
3. Why is binding the admin-panel to `127.0.0.1` not a complete security
   solution? What additional measures would you take in production?
4. An intern suggests putting all containers on one network "to make
   debugging easier." Write a brief response explaining why this is
   a bad idea.

<details>
<summary>Hint</summary>

Think about:
- What networks each compromised container is on
- Whether a compromised container can act as a bridge between networks
- Defense in depth: network segmentation is one layer, not the only layer
- The blast radius concept: how much damage can one compromised component cause?

</details>

---

## Success Criteria

- [ ] You classified all services by visibility and tier
- [ ] Your docker-compose.yml uses named networks with correct service assignments
- [ ] No database or Redis ports are accessible from the host network
- [ ] Admin panel is bound to 127.0.0.1 only
- [ ] Only the frontend's ports (80, 443) are publicly mapped
- [ ] Your verification script tests at least 10 security rules
- [ ] You can explain the blast radius for each potential compromise
- [ ] You understand that network segmentation is necessary but not sufficient for security

## What You Should Understand After This Exercise

Network segmentation is the Docker equivalent of firewall rules. By placing
services on different networks, you limit the blast radius of a compromise.
A container can only reach other containers on the same network. Combined
with binding ports to 127.0.0.1 for internal tools and removing unnecessary
port mappings, you significantly reduce the attack surface. However, network
segmentation is only one layer of defense -- you also need image scanning,
least-privilege user accounts, read-only file systems, and other controls.
