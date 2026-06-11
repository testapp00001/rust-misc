# Cheatsheet: Health Checks

## Liveness vs Readiness

| Probe | Question | Failure Action | Use Case |
|-------|----------|----------------|----------|
| Liveness | Is it alive? | Restart container | Deadlocks, infinite loops |
| Readiness | Is it ready? | Stop sending traffic | Startup, dependency checks |
| Startup | Has it started? | Wait, then check liveness | Slow startup |

## Dockerfile
```dockerfile
HEALTHCHECK --interval=30s --timeout=5s --retries=3 --start-period=10s \
  CMD curl -f http://localhost:8080/health || exit 1
```

## Docker Compose
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 30s
  timeout: 5s
  retries: 3
  start_period: 10s
```

## Kubernetes
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 15
  periodSeconds: 20

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10

startupProbe:
  httpGet:
    path: /health
    port: 8080
  failureThreshold: 30
  periodSeconds: 10
```

## Check Health Status
```bash
docker ps  # Shows "(healthy)" or "(unhealthy)"
docker inspect --format='{{.State.Health.Status}}' my-app
```

## Restart Policies
```bash
docker run --restart unless-stopped my-app  # Always restart
docker run --restart on-failure:5 my-app    # Max 5 retries
```
