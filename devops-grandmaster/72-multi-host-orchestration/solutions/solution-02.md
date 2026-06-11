# Solution 02: Docker Swarm Stack Deployment

## Part A: Stack Definition

```yaml
version: "3.8"

services:
  api:
    image: shop/api:latest
    ports:
      - "8080:8080"
    deploy:
      replicas: 3
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      update_config:
        parallelism: 1
        delay: 30s
        failure_action: rollback
        monitor: 60s
      rollback_config:
        parallelism: 1
        delay: 10s
      placement:
        constraints:
          - node.role == worker
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 128M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 15s
      timeout: 5s
      retries: 3
      start_period: 30s
    networks:
      - shop-frontend
      - shop-backend

  catalog:
    image: shop/catalog:latest
    deploy:
      replicas: 2
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      placement:
        constraints:
          - node.labels.ssd == true
      resources:
        limits:
          cpus: '0.5'
          memory: 256M
        reservations:
          cpus: '0.1'
          memory: 64M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 15s
      timeout: 5s
      retries: 3
      start_period: 20s
    networks:
      - shop-backend

  redis:
    image: redis:7-alpine
    deploy:
      replicas: 1
      restart_policy:
        condition: on-failure
      resources:
        limits:
          cpus: '0.5'
          memory: 256M
    volumes:
      - redis_data:/data
    networks:
      - shop-backend

  postgres:
    image: postgres:15
    deploy:
      replicas: 1
      restart_policy:
        condition: on-failure
      placement:
        constraints:
          - node.labels.ssd == true
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
      POSTGRES_DB: shop
    volumes:
      - pg_data:/var/lib/postgresql/data
    secrets:
      - db_password
    networks:
      - shop-backend

networks:
  shop-frontend:
    driver: overlay
    attachable: true
  shop-backend:
    driver: overlay
    internal: true

volumes:
  redis_data:
    driver: local
  pg_data:
    driver: local

secrets:
  db_password:
    file: ./secrets/db_password.txt
```

### Why This Works

- **Placement constraints** ensure `api` runs on workers (not managers, preserving manager resources for Swarm coordination) and `catalog`/`postgres` run on SSD nodes (for I/O performance).
- **Resource limits** prevent any single service from consuming all host resources. Reservations guarantee minimum allocation.
- **Health checks** enable Swarm to detect failures and restart containers automatically.
- **Internal overlay network** (`shop-backend`) isolates backend services from external access. Only `api` is on `shop-frontend`.
- **Secrets** are encrypted at rest and mounted as files inside the container, never exposed as environment variables.

### Common Mistakes to Avoid

- Putting all services on the frontend network. Only the API gateway should be externally accessible.
- Using `node.role == manager` for application services. Managers should run Swarm infrastructure, not application workloads.
- Forgetting `start_period` on health checks. Without it, containers that need time to initialize will be killed during startup.
- Not setting resource limits. A single service can starve the entire host.

---

## Part B: Deployment Commands

```bash
# 1. Label a node as having SSD storage
docker node update --label-add ssd=true <node_id_or_hostname>

# 2. Deploy the stack
docker stack deploy -c docker-compose.yml shop

# 3. Scale the api service to 5 replicas
docker service scale shop_api=5

# 4. Verify all services are running
docker stack services shop
docker stack ps shop

# 5. View logs for the catalog service
docker service logs -f shop_catalog
```

### Common Mistakes to Avoid

- Using the wrong service name format. Swarm prefixes the stack name: `shop_api`, not just `api`.
- Forgetting to label nodes before deploying. The `catalog` service will stay pending if no SSD-labeled node exists.

---

## Part C: Rolling Update Command

```bash
docker service update \
  --image shop/api:v2 \
  --update-parallelism 1 \
  --update-delay 30s \
  --update-failure-action rollback \
  --update-monitor 60s \
  shop_api
```

Alternatively, set these in the `deploy.update_config` block (already done in Part A) and simply run:

```bash
docker service update --image shop/api:v2 shop_api
```

To rollback if something goes wrong:

```bash
docker service rollback shop_api
```

### Why This Works

- `--update-parallelism 1` updates one container at a time, ensuring capacity is maintained.
- `--update-delay 30s` waits between updates, giving time to detect issues.
- `--update-failure-action rollback` automatically reverts if health checks fail.
- `--update-monitor 60s` monitors the updated container for 60 seconds before considering it healthy.

### Common Mistakes to Avoid

- Setting parallelism too high (e.g., equal to replica count). This updates all containers at once, causing downtime.
- Not setting `--update-monitor`. Without it, Swarm considers the update successful as soon as the container starts, not after it passes health checks.

---

## Key Takeaway

Docker Swarm stack definitions combine Docker Compose syntax with production-grade deployment configuration. The `deploy` block controls scheduling, scaling, updates, and recovery. Proper use of constraints, resource limits, health checks, and overlay networking turns a simple compose file into a production deployment specification.
