# Module 58: Blue-Green Deployment — Zero-Downtime Releases

> **Previous Module (57):** Pipeline Design
> **Phase:** CI/CD Mastery
> **Estimated Time:** 3-4 hours

---

## The Problem

You have a CI/CD pipeline that builds, tests, and pushes artifacts. Now you need to deploy to production. The traditional approach:

1. Stop the old version
2. Start the new version
3. Pray nothing goes wrong

This creates downtime. Even if the downtime is only 30 seconds, that's 30 seconds of errors for every user. And if the new version has a bug? You need to go through the same stop-start cycle to roll back, doubling the downtime.

**The core tension:** You need to deploy new versions, but users expect 100% availability. You can't take the application offline to update it.

---

## The Naive Way

### Rolling Update (Kubernetes Default)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 1
  template:
    spec:
      containers:
        - name: myapp
          image: myapp:v1
```

Kubernetes kills old pods one at a time and starts new ones. During the transition:

- Both v1 and v2 pods serve traffic simultaneously
- If v2 crashes on startup, you have fewer healthy pods serving traffic
- Rollback requires another rolling update (killing v2 pods, starting v1 pods)
- There's a window where requests hit a crashing new version

**Problems:**
- Traffic hits unhealthy pods during failed deployments
- Rollback is slow (another full rolling update)
- No instant switch-back capability
- Database migrations can break old pods running alongside new pods

### Stop-and-Start

```bash
# The "maintenance window" approach
docker stop myapp-v1
docker run -d --name myapp-v2 myapp:v2
```

**Problems:**
- Guaranteed downtime
- No rollback without restarting v1
- Users see errors during the gap

---

## The Right Way

### Blue-Green: Two Identical Environments

The concept is simple: maintain two complete, identical production environments.

```
                    ┌──────────────┐
                    │ Load Balancer │
                    └──────┬───────┘
                           │
              ┌────────────┴────────────┐
              │                         │
        ┌─────┴─────┐            ┌─────┴─────┐
        │   BLUE    │            │   GREEN   │
        │  (v1)     │            │  (v2)     │
        │  ACTIVE   │            │  IDLE     │
        └───────────┘            └───────────┘
```

**Blue** is the current live environment serving all traffic.
**Green** is the new version, fully deployed and tested but not receiving traffic.

**Deployment process:**
1. Deploy v2 to the Green environment
2. Run smoke tests against Green (no user impact)
3. Switch the load balancer from Blue to Green
4. Green is now live, Blue is idle
5. If something goes wrong, switch back to Blue instantly

**Rollback is instant** — just point the load balancer back to Blue.

---

## The Production Way

### Docker Compose Blue-Green

```yaml
# docker-compose.blue-green.yml
version: "3.8"

services:
  # ── BLUE ENVIRONMENT ──
  app-blue:
    image: myapp:v1
    container_name: app-blue
    networks:
      - app-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 3

  # ── GREEN ENVIRONMENT ──
  app-green:
    image: myapp:v2
    container_name: app-green
    networks:
      - app-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 3

  # ── NGINX ROUTER ──
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./active-env.conf:/etc/nginx/conf.d/active-env.conf:ro
    depends_on:
      - app-blue
      - app-green
    networks:
      - app-network

networks:
  app-network:
```

```nginx
# nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream active {
        include /etc/nginx/conf.d/active-env.conf;
    }

    server {
        listen 80;

        location / {
            proxy_pass http://active;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        location /health {
            access_log off;
            return 200 "ok";
        }
    }
}
```

```nginx
# active-env.conf — swap this file to switch environments
server app-blue:8080;
# server app-green:8080;
```

**Deployment script:**

```bash
#!/bin/bash
# deploy-blue-green.sh

set -euo pipefail

CURRENT_ENV=$(cat /tmp/active-env 2>/dev/null || echo "blue")

if [ "$CURRENT_ENV" = "blue" ]; then
    NEW_ENV="green"
    NEW_IMAGE="myapp:v2"
else
    NEW_ENV="blue"
    NEW_IMAGE="myapp:v1"
fi

echo "Current active: $CURRENT_ENV"
echo "Deploying to: $NEW_ENV"

# Step 1: Deploy new version to idle environment
echo "Building and starting $NEW_ENV environment..."
docker compose pull "app-$NEW_ENV"
docker compose up -d "app-$NEW_ENV"

# Step 2: Wait for health check
echo "Waiting for $NEW_ENV to be healthy..."
for i in $(seq 1 30); do
    if docker compose exec -T "app-$NEW_ENV" curl -sf http://localhost:8080/health > /dev/null 2>&1; then
        echo "$NEW_ENV is healthy!"
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "ERROR: $NEW_ENV failed health check after 30 attempts"
        docker compose logs "app-$NEW_ENV"
        exit 1
    fi
    echo "Attempt $i/30 - waiting..."
    sleep 2
done

# Step 3: Run smoke tests against new environment
echo "Running smoke tests against $NEW_ENV..."
RESPONSE=$(docker compose exec -T "app-$NEW_ENV" curl -sf http://localhost:8080/health)
if echo "$RESPONSE" | grep -q "healthy"; then
    echo "Smoke tests passed!"
else
    echo "ERROR: Smoke tests failed"
    exit 1
fi

# Step 4: Switch traffic
echo "Switching traffic from $CURRENT_ENV to $NEW_ENV..."
if [ "$NEW_ENV" = "green" ]; then
    echo "server app-green:8080;" > active-env.conf
else
    echo "server app-blue:8080;" > active-env.conf
fi

# Step 5: Reload nginx
docker compose exec nginx nginx -s reload
echo "$NEW_ENV" > /tmp/active-env

# Step 6: Verify the switch
echo "Verifying traffic is routed to $NEW_ENV..."
sleep 2
RESPONSE=$(curl -sf http://localhost/health)
echo "Production response: $RESPONSE"

# Step 7: Keep old environment running for rollback
echo ""
echo "=== Deployment Complete ==="
echo "Active environment: $NEW_ENV"
echo "Previous environment ($CURRENT_ENV) is still running for rollback"
echo "To rollback: bash rollback.sh"
```

**Rollback script:**

```bash
#!/bin/bash
# rollback.sh

set -euo pipefail

CURRENT_ENV=$(cat /tmp/active-env 2>/dev/null || echo "blue")

if [ "$CURRENT_ENV" = "blue" ]; then
    ROLLBACK_ENV="green"
else
    ROLLBACK_ENV="blue"
fi

echo "Rolling back from $CURRENT_ENV to $ROLLBACK_ENV..."

if [ "$ROLLBACK_ENV" = "green" ]; then
    echo "server app-green:8080;" > active-env.conf
else
    echo "server app-blue:8080;" > active-env.conf
fi

docker compose exec nginx nginx -s reload
echo "$ROLLBACK_ENV" > /tmp/active-env

echo "Rollback complete. Active environment: $ROLLBACK_ENV"
```

### Kubernetes Blue-Green Deployment

```yaml
# blue-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-blue
  labels:
    app: myapp
    version: blue
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
      version: blue
  template:
    metadata:
      labels:
        app: myapp
        version: blue
    spec:
      containers:
        - name: myapp
          image: myapp:v1
          ports:
            - containerPort: 8080
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 20
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
---
# green-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-green
  labels:
    app: myapp
    version: green
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
      version: green
  template:
    metadata:
      labels:
        app: myapp
        version: green
    spec:
      containers:
        - name: myapp
          image: myapp:v2
          ports:
            - containerPort: 8080
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 20
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
---
# service.yaml — switch selector to change traffic
apiVersion: v1
kind: Service
metadata:
  name: myapp
spec:
  selector:
    app: myapp
    version: blue  # Change to "green" to switch traffic
  ports:
    - port: 80
      targetPort: 8080
```

**Switch traffic in Kubernetes:**

```bash
# Point service to green
kubectl patch service myapp -p '{"spec":{"selector":{"version":"green"}}}'

# Verify
kubectl get endpoints myapp

# Rollback to blue
kubectl patch service myapp -p '{"spec":{"selector":{"version":"blue"}}}'
```

### Database Challenges

Blue-green deployment has a critical challenge: **database schema changes**.

```
Blue (v1) expects schema v1
Green (v2) expects schema v2
Both are running simultaneously during the switch
```

**Strategy: Expand and Contract**

```sql
-- Phase 1: EXPAND (add new column, keep old one)
ALTER TABLE users ADD COLUMN email_normalized VARCHAR(255);
UPDATE users SET email_normalized = LOWER(TRIM(email));

-- Now both v1 (uses email) and v2 (uses email_normalized) work

-- Phase 2: CONTRACT (after blue is decommissioned)
ALTER TABLE users DROP COLUMN email;
ALTER TABLE users RENAME COLUMN email_normalized TO email;
```

**Rules:**
- Never rename or drop columns during a blue-green switch
- Add new columns as nullable or with defaults
- Migrate data in a backward-compatible way
- Only remove old columns after the old environment is fully decommissioned

### Load Balancer Configuration

**Nginx upstream switching:**

```nginx
# Instant switch via symlink
upstream backend {
    include /etc/nginx/conf.d/upstream.conf;
}

# upstream.conf (point to blue)
server 10.0.1.10:8080;
server 10.0.1.11:8080;
server 10.0.1.12:8080;

# upstream-green.conf (point to green)
server 10.0.2.10:8080;
server 10.0.2.11:8080;
server 10.0.2.12:8080;
```

```bash
# Switch
ln -sf /etc/nginx/conf.d/upstream-green.conf /etc/nginx/conf.d/upstream.conf
nginx -s reload
```

**AWS ALB target group switching:**

```bash
# Register green targets
aws elbv2 register-targets \
    --target-group-arn $GREEN_TG_ARN \
    --targets Id=$GREEN_INSTANCE_1 Id=$GREEN_INSTANCE_2

# Switch listener to green target group
aws elbv2 modify-rule \
    --rule-arn $LISTENER_RULE_ARN \
    --actions Type=forward,TargetGroupArn=$GREEN_TG_ARN
```

### Cost Implications

Blue-green requires **2x the resources** during deployment:

```
Blue (active):  3 servers, 2GB RAM each = 6GB total
Green (idle):   3 servers, 2GB RAM each = 6GB total
                                        = 12GB total during deployment
```

**Mitigation strategies:**
- Use auto-scaling groups — scale Green up before switch, scale Blue down after
- Use spot/preemptible instances for the idle environment
- Keep the idle environment at minimal capacity (1 replica) and scale up before switch
- In cloud environments, use blue-green only for critical services

---

## Hands-On Lab

### Lab: Implement Blue-Green Deployment

#### Prerequisites

- Docker and Docker Compose installed
- curl installed

#### Step 1: Create the Application

```bash
mkdir blue-green-lab && cd blue-green-lab
```

Create a simple Rust web server:

```toml
# Cargo.toml
[package]
name = "bg-app"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
serde_json = "1"
```

```rust
// src/main.rs
use actix_web::{web, App, HttpServer, HttpResponse};

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn version_info() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Blue-Green Deployment Demo"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap();

    println!("Starting server on port {}", port);

    HttpServer::new(move || {
        App::new()
            .route("/health", web::get().to(health))
            .route("/version", web::get().to(version_info))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
```

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/bg-app /usr/local/bin/
EXPOSE 8080
CMD ["bg-app"]
```

#### Step 2: Create Blue-Green Docker Compose Setup

```yaml
# docker-compose.yml
version: "3.8"

services:
  app-blue:
    build:
      context: .
      args:
        VERSION: "1.0.0"
    container_name: app-blue
    environment:
      - PORT=8080
    networks:
      - bg-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 5s
      timeout: 3s
      retries: 3

  app-green:
    build:
      context: .
      args:
        VERSION: "2.0.0"
    container_name: app-green
    environment:
      - PORT=8080
    networks:
      - bg-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 5s
      timeout: 3s
      retries: 3

  nginx:
    image: nginx:alpine
    container_name: nginx-router
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./upstream.conf:/etc/nginx/conf.d/upstream.conf:ro
    depends_on:
      app-blue:
        condition: service_healthy
      app-green:
        condition: service_healthy
    networks:
      - bg-network

networks:
  bg-network:
```

```nginx
# nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream backend {
        include /etc/nginx/conf.d/upstream.conf;
    }

    server {
        listen 80;

        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }
    }
}
```

```nginx
# upstream.conf — initially pointing to blue
server app-blue:8080;
```

#### Step 3: Deploy and Switch

```bash
# Build and start all services
docker compose up -d --build

# Verify blue is serving traffic
curl http://localhost/health
# Expected: {"status":"healthy","version":"1.0.0"}

# Test green directly (internal)
docker compose exec nginx curl http://app-green:8080/health
# Expected: {"status":"healthy","version":"2.0.0"}

# Switch traffic to green
echo "server app-green:8080;" > upstream.conf
docker compose exec nginx nginx -s reload

# Verify green is now serving traffic
curl http://localhost/health
# Expected: {"status":"healthy","version":"2.0.0"}

# Rollback to blue
echo "server app-blue:8080;" > upstream.conf
docker compose exec nginx nginx -s reload

# Verify blue is serving again
curl http://localhost/health
# Expected: {"status":"healthy","version":"1.0.0"}
```

#### Step 4: Automate with a Script

```bash
#!/bin/bash
# switch-env.sh
set -euo pipefail

TARGET_ENV=${1:-}

if [ -z "$TARGET_ENV" ]; then
    echo "Usage: ./switch-env.sh [blue|green]"
    exit 1
fi

if [ "$TARGET_ENV" != "blue" ] && [ "$TARGET_ENV" != "green" ]; then
    echo "Error: Environment must be 'blue' or 'green'"
    exit 1
fi

# Check health before switching
echo "Checking health of app-$TARGET_ENV..."
if ! docker compose exec -T "app-$TARGET_ENV" curl -sf http://localhost:8080/health > /dev/null 2>&1; then
    echo "Error: app-$TARGET_ENV is not healthy. Aborting switch."
    exit 1
fi

# Switch
echo "server app-$TARGET_ENV:8080;" > upstream.conf
docker compose exec nginx nginx -s reload

# Verify
sleep 1
RESPONSE=$(curl -sf http://localhost/health)
echo "Switched to $TARGET_ENV. Response: $RESPONSE"
```

```bash
chmod +x switch-env.sh
./switch-env.sh green
./switch-env.sh blue
```

#### Step 5: Observe Zero Downtime

```bash
# In terminal 1: continuously poll the health endpoint
while true; do
    curl -sf http://localhost/version 2>/dev/null || echo "ERROR: request failed"
    sleep 0.1
done

# In terminal 2: switch environments
./switch-env.sh green
# Watch terminal 1 — no errors, version changes seamlessly
```

#### Verification Checklist

- [ ] Blue environment starts and passes health check
- [ ] Green environment starts and passes health check
- [ ] Traffic routes to blue initially
- [ ] Switching to green happens with no dropped requests
- [ ] Rollback to blue is instant
- [ ] Health checks prevent switching to unhealthy environments
- [ ] Script automates the entire process

---

## Key Takeaways

1. **Two Identical Environments** — Blue and Green are full copies of production
2. **Instant Rollback** — Switch the load balancer pointer, no redeployment needed
3. **Test Before Switch** — Green is fully deployed and tested before receiving traffic
4. **Database Migrations** — Use expand-and-contract pattern for backward compatibility
5. **Cost** — 2x resources during deployment; mitigate with auto-scaling
6. **Atomic Switch** — The load balancer switch is a single configuration change
7. **Verification** — Always verify the switch with health checks and smoke tests

---

## Limitation

Blue-green deployment switches 100% of traffic at once. If the new version has a subtle bug that only manifests under production load or with a specific subset of users, everyone is affected immediately. You detect the problem by monitoring error rates, then roll back — but by then, all users have been impacted.

**Next:** Module 59 — Canary Deployment solves this by gradually shifting traffic (1% -> 10% -> 50% -> 100%), allowing you to detect problems before they affect all users.
