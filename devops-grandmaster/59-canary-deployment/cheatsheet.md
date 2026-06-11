# Cheatsheet: Canary Deployment

## Concept
```
                    ┌─────────┐
                    │   LB    │
                    └────┬────┘
                         │
              ┌──────────┴──────────┐
              │                     │
        ┌─────▼─────┐        ┌─────▼─────┐
        │  Stable   │        │  Canary   │
        │   (95%)   │        │   (5%)    │
        │  v1.0     │        │  v2.0     │
        └───────────┘        └───────────┘
```

## Nginx Canary
```nginx
upstream stable {
    server backend1:8080 weight=95;
    server backend2:8080 weight=95;
}

upstream canary {
    server backend3:8080 weight=5;
}

split_clients "${remote_addr}" $variant {
    5%    canary;
    *     stable;
}
```

## Kubernetes Canary
```yaml
# Stable deployment (95%)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app-stable
spec:
  replicas: 19
  selector:
    matchLabels:
      app: my-app
      track: stable

---
# Canary deployment (5%)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app-canary
spec:
  replicas: 1
  selector:
    matchLabels:
      app: my-app
      track: canary

---
# Service routes to both
apiVersion: v1
kind: Service
metadata:
  name: my-app
spec:
  selector:
    app: my-app  # Matches both stable and canary
```

## Promotion Steps
```
1. Deploy canary (5%)
2. Monitor metrics for 15 minutes
3. If healthy → increase to 25%
4. Monitor for 15 minutes
5. If healthy → increase to 50%
6. Monitor for 15 minutes
7. If healthy → promote to 100%
8. If unhealthy → rollback
```
