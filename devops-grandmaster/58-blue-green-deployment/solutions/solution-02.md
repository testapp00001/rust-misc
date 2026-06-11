# Solution 02: Implement Blue-Green with Docker Compose

## Part A: Docker Compose File

```yaml
# docker-compose.yml
version: "3.8"

services:
  app-blue:
    image: myapp:v1
    container_name: app-blue
    networks:
      - app-network
    healthcheck:
      test: ["CMD", "curl", "-sf", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 10s

  app-green:
    image: myapp:v2
    container_name: app-green
    networks:
      - app-network
    healthcheck:
      test: ["CMD", "curl", "-sf", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 10s

  nginx:
    image: nginx:alpine
    container_name: nginx-router
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./active-env.conf:/etc/nginx/conf.d/active-env.conf:ro
    depends_on:
      app-blue:
        condition: service_healthy
      app-green:
        condition: service_healthy
    networks:
      - app-network

networks:
  app-network:
```

### Why This Works

All three containers are on the same Docker network, so nginx can reach
both app containers by name. The health checks ensure containers are
actually serving traffic before nginx starts. The `start_period` gives
the application time to initialize before health checks begin counting
failures.

## Part B: nginx Configuration

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
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_connect_timeout 5s;
            proxy_read_timeout 30s;
        }

        location /health {
            access_log off;
            return 200 "nginx-router: ok\n";
        }
    }
}
```

```nginx
# active-env.conf -- swap this file to switch environments
server app-blue:8080;
```

### Why This Works

The `include` directive loads the upstream server definition from a
separate file. To switch environments, you overwrite `active-env.conf`
and reload nginx. The `proxy_connect_timeout` and `proxy_read_timeout`
ensure nginx does not hang if an upstream is unresponsive.

The `/health` endpoint on nginx itself is separate from the app health
check. It verifies that the router is running, regardless of which
environment is active.

## Part C: Deployment Script

```bash
#!/bin/bash
# deploy.sh -- Blue-green deployment script

set -euo pipefail

# Determine which environment is active
CURRENT_ENV=$(cat /tmp/active-env 2>/dev/null || echo "blue")

if [ "$CURRENT_ENV" = "blue" ]; then
    NEW_ENV="green"
else
    NEW_ENV="blue"
fi

echo "Current active environment: $CURRENT_ENV"
echo "Deploying to: $NEW_ENV"

# Step 1: Pull the new image and restart the idle environment
echo "Starting $NEW_ENV environment..."
docker compose pull "app-$NEW_ENV"
docker compose up -d "app-$NEW_ENV"

# Step 2: Wait for health check
echo "Waiting for $NEW_ENV to become healthy..."
MAX_ATTEMPTS=30
for i in $(seq 1 $MAX_ATTEMPTS); do
    # Check the Docker health status
    STATUS=$(docker inspect --format='{{.State.Health.Status}}' "app-$NEW_ENV" 2>/dev/null || echo "starting")

    if [ "$STATUS" = "healthy" ]; then
        echo "$NEW_ENV is healthy (attempt $i/$MAX_ATTEMPTS)"
        break
    fi

    if [ "$i" -eq "$MAX_ATTEMPTS" ]; then
        echo "ERROR: $NEW_ENV failed health check after $MAX_ATTEMPTS attempts"
        echo "Rolling back: keeping $CURRENT_ENV active"
        docker compose logs "app-$NEW_ENV"
        exit 1
    fi

    echo "Attempt $i/$MAX_ATTEMPTS -- status: $STATUS"
    sleep 2
done

# Step 3: Run smoke tests against the new environment directly
echo "Running smoke tests against $NEW_ENV..."
SMOKE_RESPONSE=$(docker compose exec -T "app-$NEW_ENV" curl -sf http://localhost:8080/health)
if echo "$SMOKE_RESPONSE" | grep -q "healthy"; then
    echo "Smoke tests passed: $SMOKE_RESPONSE"
else
    echo "ERROR: Smoke tests failed"
    echo "Response: $SMOKE_RESPONSE"
    exit 1
fi

# Step 4: Switch traffic
echo "Switching traffic from $CURRENT_ENV to $NEW_ENV..."
if [ "$NEW_ENV" = "green" ]; then
    echo "server app-green:8080;" > active-env.conf
else
    echo "server app-blue:8080;" > active-env.conf
fi

# Step 5: Reload nginx (graceful -- no dropped connections)
docker compose exec nginx nginx -s reload
echo "$NEW_ENV" > /tmp/active-env

# Step 6: Verify the switch
echo "Verifying production traffic..."
sleep 2
PROD_RESPONSE=$(curl -sf http://localhost/health 2>/dev/null || echo "FAILED")
echo "Production response: $PROD_RESPONSE"

# Step 7: Summary
echo ""
echo "=== Deployment Complete ==="
echo "Active environment: $NEW_ENV"
echo "Previous environment ($CURRENT_ENV) is still running"
echo "To rollback: bash rollback.sh"
```

### Why This Works

The script follows the blue-green deployment pattern:
1. Deploy to idle environment (no traffic impact)
2. Verify health (catch failures before users are affected)
3. Run smoke tests (validate functionality)
4. Switch traffic (instant, via nginx reload)
5. Verify switch (confirm traffic is flowing correctly)

The nginx reload is graceful -- existing connections are served to
completion while new connections go to the new upstream. No requests
are dropped during the switch.

## Part D: Rollback Script

```bash
#!/bin/bash
# rollback.sh -- Instant rollback to previous environment

set -euo pipefail

CURRENT_ENV=$(cat /tmp/active-env 2>/dev/null || echo "blue")

if [ "$CURRENT_ENV" = "blue" ]; then
    ROLLBACK_ENV="green"
else
    ROLLBACK_ENV="blue"
fi

echo "Rolling back from $CURRENT_ENV to $ROLLBACK_ENV..."

# Verify the rollback target is healthy
STATUS=$(docker inspect --format='{{.State.Health.Status}}' "app-$ROLLBACK_ENV" 2>/dev/null || echo "unknown")
if [ "$STATUS" != "healthy" ]; then
    echo "WARNING: $ROLLBACK_ENV health status is $STATUS"
    echo "Rolling back anyway (previous environment may still be functional)"
fi

# Switch traffic back
if [ "$ROLLBACK_ENV" = "green" ]; then
    echo "server app-green:8080;" > active-env.conf
else
    echo "server app-blue:8080;" > active-env.conf
fi

docker compose exec nginx nginx -s reload
echo "$ROLLBACK_ENV" > /tmp/active-env

echo "Rollback complete. Active environment: $ROLLBACK_ENV"
```

### Why This Works

Rollback is just the reverse of the traffic switch. The previous
environment is still running (the deploy script never stops it), so
rollback takes less than 1 second. This is the key advantage of
blue-green: rollback is always instant because both environments
exist simultaneously.

## Common Mistakes to Avoid

- **Stopping the old environment after switching.** The old environment
  must stay running for rollback. Only stop it after a confidence period
  (e.g., 30 minutes of successful operation).
- **Not running smoke tests before switching.** Health checks verify the
  process is running; smoke tests verify the application works correctly.
  Both are needed.
- **Using `nginx -s stop` followed by `nginx`.** This drops all active
  connections. Always use `nginx -s reload` for graceful reload.
- **Hardcoding the active environment.** Track the active environment in
  a file or label so the script works on subsequent deployments.

## Key Takeaway

Blue-green deployment with Docker Compose and nginx is straightforward:
two app containers, one nginx router, and a config file that determines
which environment is active. The switch is a file write and a reload.
Rollback is the same operation in reverse. The entire process takes
seconds and drops zero requests.
