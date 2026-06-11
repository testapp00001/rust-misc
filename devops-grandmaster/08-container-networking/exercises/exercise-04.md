# Exercise 04: Design a Network Topology for a Microservices App

**Type:** Challenge
**Time:** 60 minutes
**Difficulty:** Medium-Hard

## Objective

Design and implement a Docker network topology for a microservices
application that requires network isolation between tiers. You will
create multiple networks, connect containers to the correct networks,
and verify that the isolation rules are enforced.

---

## Scenario

You are building an e-commerce platform with these services:

| Service | Description | Needs to Reach |
|---------|-------------|----------------|
| `frontend` | Nginx serving static files and proxying API calls | `api-gateway` |
| `api-gateway` | Routes requests to backend services | `product-svc`, `order-svc`, `user-svc` |
| `product-svc` | Product catalog API | `product-db` |
| `order-svc` | Order processing API | `order-db`, `product-svc` |
| `user-svc` | User authentication API | `user-db` |
| `product-db` | PostgreSQL for products | (nothing -- only product-svc) |
| `order-db` | PostgreSQL for orders | (nothing -- only order-svc) |
| `user-db` | PostgreSQL for users | (nothing -- only user-svc) |

### Isolation Requirements

1. The `frontend` must be able to reach the `api-gateway` and nothing else.
2. The `api-gateway` must be able to reach the three backend services but NOT the databases.
3. Each backend service must be able to reach its own database but NOT the databases of other services.
4. The `order-svc` must be able to reach `product-svc` (to look up product details when processing orders).
5. No database should be reachable from the host (no port mapping on databases).

---

## Tasks

### Part A: Design the Network Layout

Before writing any code, design the network topology on paper or in a text
file. For each network, specify:

- Network name
- Which containers are connected to it
- Why this particular grouping exists

Your design should use the **minimum number of networks** necessary to
satisfy all isolation requirements.

<details>
<summary>Hint</summary>

Think in tiers:
- A "frontend" tier for external-facing services
- A "gateway" tier between the frontend and the APIs
- Separate "data" tiers for each service and its database

A container can be on multiple networks. The `api-gateway` needs to be
on a network that reaches the backend services, and the `frontend` needs
to be on a network that reaches the `api-gateway`.

</details>

### Part B: Implement with Docker Compose

Write a `docker-compose.yml` file that implements your design. Use:

- Named networks with explicit definitions
- The `alpine` or lightweight images for services that do not need a real app
  (use `sleep infinity` to keep them running)
- Real images for databases (`postgres:16`)
- Port mapping only for the `frontend`

Your compose file must define all 8 services and all networks.

<details>
<summary>Hint</summary>

Structure your network definitions at the bottom of the compose file:

```yaml
networks:
  frontend-net:
  gateway-net:
  product-net:
  order-net:
  user-net:
```

Then assign services to networks using the `networks` key under each
service. A service listed on multiple networks participates in all of them.

</details>

### Part C: Verify Isolation

Write a verification script (bash) that tests every connectivity rule.
The script should:

1. Test that connections **should work** succeed (exit code 0)
2. Test that connections **should not work** fail (timeout or connection refused)
3. Print PASS or FAIL for each test

Example test structure:

```bash
# This should PASS (connectivity exists)
docker compose exec api-gateway ping -c 1 -W 2 product-svc && echo "PASS" || echo "FAIL"

# This should PASS (connectivity blocked)
docker compose exec frontend ping -c 1 -W 2 product-db && echo "FAIL" || echo "PASS"
```

Write tests for at least **8 connectivity rules**: 4 that should work and
4 that should be blocked.

<details>
<summary>Hint</summary>

Use `ping -c 1 -W 2` for quick reachability tests. The `-W 2` flag sets
a 2-second timeout, so blocked connections fail fast instead of hanging.
For TCP services, you can use `nc -z -w 2 <host> <port>` instead.

</details>

### Part D: Document the Topology

Create a text-based diagram (ASCII art) showing:

- All 8 services as boxes
- All networks as labeled connections between boxes
- Which services can and cannot communicate

<details>
<summary>Hint</summary>

Use a layout like this:

```
                 [frontend]
                     |
              ---- frontend-net ----
                     |
               [api-gateway]
              /      |       \
    gateway-net  gateway-net  gateway-net
        /            |             \
 [product-svc] [order-svc]    [user-svc]
        |            |             |
  product-net    order-net     user-net
        |            |             \
 [product-db]   [order-db]     [user-db]
```

Note: `order-svc` also connects to `product-net` to reach `product-svc`.

</details>

---

## Success Criteria

- [ ] You designed a network topology with the minimum number of networks
- [ ] Your docker-compose.yml defines all services and networks correctly
- [ ] Frontend can only reach api-gateway
- [ ] api-gateway can reach all three backend services but no databases
- [ ] Each backend service can reach its own database but not other databases
- [ ] order-svc can reach product-svc
- [ ] No database ports are mapped to the host
- [ ] Your verification script tests at least 8 rules (4 allowed, 4 blocked)
- [ ] You created an ASCII diagram of the topology

## What You Should Understand After This Exercise

Network isolation in Docker is achieved by placing containers on different
networks. A container that is on two networks acts as a bridge between them,
but only for traffic it is involved in. This is the foundation of the
principle of least privilege applied to networking: every service should
have access only to the resources it needs and nothing more.
