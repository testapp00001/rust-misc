# Solution 03: Design a Kubernetes Blue-Green Deployment

## Part A: Blue Deployment

```yaml
# blue-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-service-blue
  labels:
    app: order-service
    version: blue
spec:
  replicas: 3
  selector:
    matchLabels:
      app: order-service
      version: blue
  template:
    metadata:
      labels:
        app: order-service
        version: blue
    spec:
      containers:
        - name: order-service
          image: order-service:v1
          ports:
            - containerPort: 8080
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
            timeoutSeconds: 3
            failureThreshold: 3
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 20
            timeoutSeconds: 3
            failureThreshold: 3
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
```

### Why This Works

The `readinessProbe` controls when a pod receives traffic. Kubernetes
will not add the pod to the Service endpoints until the readiness probe
passes. The `livenessProbe` controls when a pod should be restarted.
If the liveness probe fails 3 times, Kubernetes kills and restarts the pod.

The `initialDelaySeconds` on the liveness probe is higher than on the
readiness probe because the application needs more time to fully start
before liveness checks make sense.

## Part B: Green Deployment

```yaml
# green-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-service-green
  labels:
    app: order-service
    version: green
spec:
  replicas: 3
  selector:
    matchLabels:
      app: order-service
      version: green
  template:
    metadata:
      labels:
        app: order-service
        version: green
    spec:
      containers:
        - name: order-service
          image: order-service:v2
          ports:
            - containerPort: 8080
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
            timeoutSeconds: 3
            failureThreshold: 3
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 20
            timeoutSeconds: 3
            failureThreshold: 3
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
```

### Why This Works

The green deployment is identical to blue except for the version label
and image tag. Both deployments coexist in the cluster. The green pods
will not receive traffic until the Service selector is changed.

## Part C: Service

```yaml
# service-blue.yaml (initial -- routes to blue)
apiVersion: v1
kind: Service
metadata:
  name: order-service
  labels:
    app: order-service
spec:
  selector:
    app: order-service
    version: blue
  ports:
    - port: 80
      targetPort: 8080
      protocol: TCP
  type: ClusterIP
```

```yaml
# service-green.yaml (after switch -- routes to green)
apiVersion: v1
kind: Service
metadata:
  name: order-service
  labels:
    app: order-service
spec:
  selector:
    app: order-service
    version: green
  ports:
    - port: 80
      targetPort: 8080
      protocol: TCP
  type: ClusterIP
```

### Why This Works

The Service uses a label selector to determine which pods receive traffic.
Changing `version: blue` to `version: green` in the selector instantly
reroutes all traffic. Kubernetes updates the Endpoints object, and the
kube-proxy on each node picks up the change within seconds.

The Service has a fixed name (`order-service`), so all clients continue
to use the same address regardless of which environment is active.

## Part D: Deployment Script

```bash
#!/bin/bash
# switch-traffic.sh -- Switch traffic between blue and green environments

set -euo pipefail

TARGET_ENV="${1:-}"

if [ -z "$TARGET_ENV" ]; then
    echo "Usage: $0 <blue|green>"
    exit 1
fi

if [ "$TARGET_ENV" != "blue" ] && [ "$TARGET_ENV" != "green" ]; then
    echo "Error: TARGET_ENV must be 'blue' or 'green'"
    exit 1
fi

SERVICE_NAME="order-service"
APP_LABEL="order-service"

# Determine current active environment
CURRENT_ENV=$(kubectl get service "$SERVICE_NAME" \
    -o jsonpath='{.spec.selector.version}' 2>/dev/null || echo "unknown")

echo "Current active environment: $CURRENT_ENV"
echo "Target environment: $TARGET_ENV"

if [ "$CURRENT_ENV" = "$TARGET_ENV" ]; then
    echo "Traffic is already routed to $TARGET_ENV. Nothing to do."
    exit 0
fi

# Step 1: Verify target pods are ready
echo "Verifying $TARGET_ENV pods are ready..."
if ! kubectl wait --for=condition=ready pod \
    -l "app=$APP_LABEL,version=$TARGET_ENV" \
    --timeout=120s; then
    echo "ERROR: $TARGET_ENV pods are not ready. Aborting switch."
    exit 1
fi

READY_PODS=$(kubectl get pods -l "app=$APP_LABEL,version=$TARGET_ENV" \
    --field-selector=status.phase=Running --no-headers | wc -l)
echo "$READY_PODS $TARGET_ENV pods are running and ready"

# Step 2: Switch traffic
echo "Switching traffic to $TARGET_ENV..."
kubectl patch service "$SERVICE_NAME" \
    -p '{"spec":{"selector":{"version":"'$TARGET_ENV'"}}}'

# Step 3: Verify the switch
NEW_ENV=$(kubectl get service "$SERVICE_NAME" \
    -o jsonpath='{.spec.selector.version}')
echo "Traffic is now routed to: $NEW_ENV"

# Step 4: Show endpoint status
echo ""
echo "Active endpoints:"
kubectl get endpoints "$SERVICE_NAME" -o wide

echo ""
echo "=== Traffic switch complete ==="
echo "Active: $TARGET_ENV"
echo "Idle: $CURRENT_ENV"
echo "To rollback: $0 $CURRENT_ENV"
```

### Why This Works

The script uses `kubectl patch` to update the Service selector, which is
the Kubernetes-native way to switch traffic. Before switching, it verifies
that the target pods are ready using `kubectl wait`. The `--timeout=120s`
gives pods 2 minutes to pass their readiness probes.

The script also checks if traffic is already routed to the target
environment, preventing unnecessary switches.

## Common Mistakes to Avoid

- **Not verifying pod readiness before switching.** If you switch to
  green pods that are not ready, all traffic goes to pods that cannot
  serve requests, causing a total outage.
- **Using `kubectl edit` for the switch.** Manual editing is error-prone
  and not auditable. Always use `kubectl patch` in a script.
- **Forgetting to update the Service selector.** A common mistake is to
  update the Deployment but not the Service. The new pods run but receive
  no traffic.
- **Not keeping both Deployments running.** Both blue and green must
  coexist. Scaling down the old environment before the new one is proven
  eliminates the rollback option.

## Key Takeaway

Kubernetes blue-green deployment is simpler than the Docker Compose approach
because the Service selector is the traffic router. Switching environments
is a single `kubectl patch` command. The key discipline is keeping both
environments running and verifying readiness before switching.
