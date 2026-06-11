# Cheatsheet: Blue-Green Deployment

## Concept
```
                    ┌─────────┐
                    │   LB    │
                    └────┬────┘
                         │
              ┌──────────┴──────────┐
              │                     │
        ┌─────▼─────┐        ┌─────▼─────┐
        │   Blue    │        │   Green   │
        │  (v1.0)   │        │  (v2.0)   │
        └───────────┘        └───────────┘
```

## Docker Compose Blue-Green
```yaml
# docker-compose.blue.yml
services:
  app:
    image: my-app:1.0
    ports:
      - "8080:8080"

# docker-compose.green.yml
services:
  app:
    image: my-app:2.0
    ports:
      - "8081:8080"
```

## Nginx Traffic Switch
```nginx
upstream app {
    server localhost:8080;  # Blue (active)
    # server localhost:8081;  # Green (inactive)
}
```

## Kubernetes Blue-Green
```yaml
# Blue deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app-blue
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-app
      version: blue
  template:
    metadata:
      labels:
        app: my-app
        version: blue
    spec:
      containers:
        - name: my-app
          image: my-app:1.0

---
# Service switches between blue and green
apiVersion: v1
kind: Service
metadata:
  name: my-app
spec:
  selector:
    app: my-app
    version: blue  # Change to 'green' to switch
```

## Rollback
```bash
# Just switch back to previous version
# No deployment needed!
```
