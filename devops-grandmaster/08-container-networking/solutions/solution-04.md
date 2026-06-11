# Solution 04: Design a Network Topology for a Microservices App

## Part A: Network Layout Design

### Minimum Networks Required: 5

| Network | Connected Containers | Purpose |
|---------|---------------------|---------|
| `frontend-net` | frontend, api-gateway | External traffic enters through the frontend and reaches the gateway |
| `api-net` | api-gateway, product-svc, order-svc, user-svc | Gateway routes requests to backend services |
| `product-net` | product-svc, product-db | Product service accesses its database |
| `order-net` | order-svc, order-db | Order service accesses its database |
| `user-net` | user-svc, user-db | User service accesses its database |

### Additional connection for cross-service communication

`order-svc` also needs to reach `product-svc`. Since both are on `api-net`,
this is already satisfied -- no additional network needed.

### Why This Design

- **Each database is isolated** on its own network with only its service.
  No other service can reach a database it does not own.
- **The gateway is the only entry point** to the backend services. The
  frontend cannot bypass the gateway.
- **The api-net** allows the gateway to reach all three backend services,
  and allows `order-svc` to reach `product-svc` (both are on `api-net`).
- **Minimum networks:** 5 networks is the minimum to satisfy all isolation
  requirements. Fewer networks would mean some databases share a network,
  violating the isolation rule.

### Containers on Multiple Networks

| Container | Networks |
|-----------|----------|
| frontend | frontend-net |
| api-gateway | frontend-net, api-net |
| product-svc | api-net, product-net |
| order-svc | api-net, order-net |
| user-svc | api-net, user-net |
| product-db | product-net |
| order-db | order-net |
| user-db | user-net |

---

## Part B: Docker Compose Implementation

```yaml
version: "3.9"

services:
  frontend:
    image: nginx:alpine
    container_name: frontend
    ports:
      - "80:80"
    networks:
      - frontend-net
    depends_on:
      - api-gateway

  api-gateway:
    image: nginx:alpine
    container_name: api-gateway
    networks:
      - frontend-net
      - api-net
    depends_on:
      - product-svc
      - order-svc
      - user-svc

  product-svc:
    image: alpine:3.19
    container_name: product-svc
    command: sleep infinity
    networks:
      - api-net
      - product-net
    depends_on:
      - product-db

  order-svc:
    image: alpine:3.19
    container_name: order-svc
    command: sleep infinity
    networks:
      - api-net
      - order-net
    depends_on:
      - order-db
      - product-svc

  user-svc:
    image: alpine:3.19
    container_name: user-svc
    command: sleep infinity
    networks:
      - api-net
      - user-net
    depends_on:
      - user-db

  product-db:
    image: postgres:16
    container_name: product-db
    environment:
      POSTGRES_DB: products
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: product_pass
    networks:
      - product-net

  order-db:
    image: postgres:16
    container_name: order-db
    environment:
      POSTGRES_DB: orders
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: order_pass
    networks:
      - order-net

  user-db:
    image: postgres:16
    container_name: user-db
    environment:
      POSTGRES_DB: users
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: user_pass
    networks:
      - user-net

networks:
  frontend-net:
  api-net:
  product-net:
  order-net:
  user-net:
```

### Key Design Decisions

1. **No database port mappings.** Databases are only accessible from within
   their Docker networks. This prevents external access.

2. **The api-gateway is on two networks.** It bridges the frontend tier and
   the API tier. It can forward requests from `frontend` to the backend
   services, but `frontend` cannot reach the backend services directly.

3. **Each backend service is on two networks.** It bridges the API tier
   (where it receives requests from the gateway) and its own data tier
   (where it talks to its database).

4. **order-svc can reach product-svc.** Both are on `api-net`, so
   `order-svc` can resolve `product-svc` by name and make API calls to it.
   `order-svc` cannot reach `product-db` because `order-svc` is not on
   `product-net`.

---

## Part C: Verification Script

```bash
#!/bin/bash
# verify-network-isolation.sh
# Tests all connectivity rules for the e-commerce microservices topology

set -e

PASS=0
FAIL=0

test_connection() {
    local description="$1"
    local container="$2"
    local target="$3"
    local should_work="$4"  # "yes" or "no"

    if docker compose exec "$container" ping -c 1 -W 2 "$target" > /dev/null 2>&1; then
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

echo "=== Testing Allowed Connections ==="

test_connection "frontend -> api-gateway" \
    frontend api-gateway yes

test_connection "api-gateway -> product-svc" \
    api-gateway product-svc yes

test_connection "api-gateway -> order-svc" \
    api-gateway order-svc yes

test_connection "api-gateway -> user-svc" \
    api-gateway user-svc yes

test_connection "product-svc -> product-db" \
    product-svc product-db yes

test_connection "order-svc -> order-db" \
    order-svc order-db yes

test_connection "order-svc -> product-svc" \
    order-svc product-svc yes

test_connection "user-svc -> user-db" \
    user-svc user-db yes

echo ""
echo "=== Testing Blocked Connections ==="

test_connection "frontend -> product-svc (should be blocked)" \
    frontend product-svc no

test_connection "frontend -> product-db (should be blocked)" \
    frontend product-db no

test_connection "api-gateway -> product-db (should be blocked)" \
    api-gateway product-db no

test_connection "api-gateway -> order-db (should be blocked)" \
    api-gateway order-db no

test_connection "product-svc -> order-db (should be blocked)" \
    product-svc order-db no

test_connection "product-svc -> user-db (should be blocked)" \
    product-svc user-db no

test_connection "order-svc -> product-db (should be blocked)" \
    order-svc product-db no

test_connection "user-svc -> product-db (should be blocked)" \
    user-svc product-db no

echo ""
echo "=== Results ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo "Total:  $((PASS + FAIL))"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
```

### Running the Script

```bash
# Start the stack
docker compose up -d

# Wait for all services to be ready
sleep 5

# Run the verification
chmod +x verify-network-isolation.sh
./verify-network-isolation.sh
```

---

## Part D: Topology Diagram

```
                    [ frontend ]
                         |
                         |  frontend-net
                         |
                   [ api-gateway ]
                   /       |       \
                  /        |        \
          api-net    api-net    api-net
                /          |          \
    [product-svc]   [order-svc]    [user-svc]
         |               |              |
    product-net     order-net      user-net
         |               |              |
    [product-db]    [order-db]     [user-db]

    Additional connection:
    order-svc ---api-net---> product-svc
    (order-svc can reach product-svc for product lookups)
```

### Isolation Matrix

| From \ To | frontend | api-gateway | product-svc | order-svc | user-svc | product-db | order-db | user-db |
|-----------|----------|-------------|-------------|-----------|----------|------------|----------|---------|
| frontend | -- | YES | no | no | no | no | no | no |
| api-gateway | YES | -- | YES | YES | YES | no | no | no |
| product-svc | no | YES | -- | YES | YES | YES | no | no |
| order-svc | no | YES | YES | -- | YES | no | YES | no |
| user-svc | no | YES | YES | YES | -- | no | no | YES |
| product-db | no | no | YES | no | no | -- | no | no |
| order-db | no | no | no | YES | no | no | -- | no |
| user-db | no | no | no | no | YES | no | no | -- |

---

## Common Mistakes

1. **Putting all databases on one network.** This would let `product-svc`
   reach `order-db`, violating isolation. Each database needs its own
   network.

2. **Forgetting order-svc needs product-svc access.** The requirement says
   order processing needs to look up product details. Both are on `api-net`,
   so this works without an extra network.

3. **Using too many networks.** Some designs create a separate network for
   every pair of communicating services. This is unnecessary -- the `api-net`
   serves all gateway-to-backend and backend-to-backend communication.

4. **Exposing database ports.** The compose file should have no `ports`
   mappings on any database service.

5. **Not using `depends_on`.** Without it, services might start before
   their dependencies, causing transient connection errors.
