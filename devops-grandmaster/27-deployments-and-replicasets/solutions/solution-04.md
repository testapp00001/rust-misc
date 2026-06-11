# Solution 04: Zero-Downtime Deployment Strategies

## Part A: Strategy Selection

| Scenario | Strategy | maxSurge | maxUnavailable | Why |
|----------|----------|----------|----------------|-----|
| 1. Stateless REST API, 1000 req/s, no breaking DB migrations | RollingUpdate | 1 | 0 | Stateless apps can run two versions simultaneously. `maxUnavailable: 0` ensures no request is dropped. `maxSurge: 1` keeps resource overhead minimal while enabling gradual rollout. The old and new versions are backward-compatible, so both can serve traffic safely. |
| 2. Batch processor, must not run two instances simultaneously | Recreate | N/A | N/A | The `Recreate` strategy kills all old Pods before creating new ones. This guarantees no two versions run at the same time, preventing duplicate processing. The trade-off is downtime during the transition, but this is acceptable to avoid data corruption from parallel processing. |
| 3. Critical payment service, must never drop below 5 replicas | RollingUpdate | 1 | 0 | `maxUnavailable: 0` guarantees the available count never drops below the desired replica count. Combined with `maxSurge: 1`, one extra Pod is created before any old Pod is removed. This is the safest configuration for critical services. |
| 4. Internal admin dashboard, 2 users, resource-constrained cluster | RollingUpdate | 0 | 1 | With only 2 users and a resource-constrained cluster, `maxSurge: 0` avoids creating extra Pods (which would double resource usage). `maxUnavailable: 1` allows one Pod to go down briefly. This is a deliberate trade-off: short unavailability for one user is acceptable to stay within resource limits. |
| 5. App with breaking DB schema migration on startup | Recreate | N/A | N/A | When the new version's startup migration breaks the old version's ability to function, both versions cannot coexist. The `Recreate` strategy ensures all old Pods are terminated first, then the new Pods start and run the migration. This requires downtime but prevents the old version from crashing due to schema incompatibility. |

**Why it works:** The strategy choice depends on whether two versions can coexist. If yes, `RollingUpdate` provides zero downtime. If no, `Recreate` is the only safe option. The `maxSurge` and `maxUnavailable` values within `RollingUpdate` control the speed-vs-safety trade-off.

**Common mistakes:**
- Always choosing `maxUnavailable: 0` without considering resource constraints. In a resource-constrained cluster, `maxSurge: 1` might cause Pods to be unschedulable.
- Choosing `RollingUpdate` for applications with breaking changes. If two versions cannot coexist, `RollingUpdate` will cause failures in the old Pods when the new Pods apply breaking changes.
- Forgetting that `Recreate` means downtime. Always weigh whether the downtime is acceptable.

---

## Part B: Implement a Canary Deployment

`canary-stable.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-stable
spec:
  replicas: 9
  selector:
    matchLabels:
      app: myapp
      track: stable
  template:
    metadata:
      labels:
        app: myapp
        track: stable
        version: v1.0.0
    spec:
      containers:
      - name: myapp
        image: myapp:1.0.0
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: 100m
            memory: 64Mi
          limits:
            cpu: 200m
            memory: 128Mi
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 3
```

`canary-new.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-canary
spec:
  replicas: 1
  selector:
    matchLabels:
      app: myapp
      track: canary
  template:
    metadata:
      labels:
        app: myapp
        track: canary
        version: v2.0.0
    spec:
      containers:
      - name: myapp
        image: myapp:2.0.0
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: 100m
            memory: 64Mi
          limits:
            cpu: 200m
            memory: 128Mi
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 3
```

A Service selecting both:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
spec:
  selector:
    app: myapp
  ports:
  - port: 80
    targetPort: 8080
```

**Why it works:** Both Deployments share the label `app: myapp`, so the Service with `selector: { app: myapp }` distributes traffic to all 10 Pods. The traffic split is proportional: 9/10 (90%) to stable, 1/10 (10%) to canary. The `track` label distinguishes the Deployments for independent management without affecting Service selection.

**Common mistakes:**
- Using different `app` labels for stable and canary. The Service would not be able to select both.
- Using the same Deployment name. Each must have a unique name since they are separate API objects.
- Forgetting readiness probes. Without them, the canary Pod might receive traffic before it is ready, causing errors for 10% of users.

---

## Part C: Promote or Rollback the Canary

### Promote the Canary to Full Production

Steps:
1. Update the stable Deployment's image to `myapp:2.0.0` and scale to 10 replicas.
2. Delete the canary Deployment.

```bash
# Update stable to the canary version
kubectl set image deployment myapp-stable myapp=myapp:2.0.0
kubectl scale deployment myapp-stable --replicas=10

# Wait for rollout
kubectl rollout status deployment myapp-stable --timeout=120s

# Remove the canary
kubectl delete deployment myapp-canary
```

What happens: The stable Deployment performs a rolling update from v1.0.0 to v2.0.0 while scaling to 10 replicas. Once all 10 Pods are running v2.0.0, the canary Deployment is deleted, removing its 1 Pod. The Service continues to select `app: myapp` and now serves only v2.0.0 Pods.

### Rollback the Canary

Steps:
1. Delete the canary Deployment.
2. Verify the stable Deployment is still healthy.

```bash
# Remove the canary
kubectl delete deployment myapp-canary

# Verify stable is still running
kubectl get pods -l app=myapp
kubectl get deployment myapp-stable
```

What happens: The canary Deployment and its 1 Pod are deleted. The stable Deployment's 9 Pods continue running v1.0.0. The Service now routes 100% of traffic to v1.0.0. No disruption occurs because the stable Pods were never touched.

**Common mistakes:**
- Updating the stable Deployment before deleting the canary, causing a brief period where both Deployments are running v2.0.0 with different replica counts.
- Not verifying the canary before promoting. The whole point of canary is to test before committing.

---

## Part D: Implement a Blue-Green Deployment

`blue.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app-blue
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
        image: myapp:1.0.0
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: 100m
            memory: 64Mi
          limits:
            cpu: 200m
            memory: 128Mi
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 3
```

`green.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app-green
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
        image: myapp:2.0.0
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: 100m
            memory: 64Mi
          limits:
            cpu: 200m
            memory: 128Mi
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 3
```

Service initially pointing to blue:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
spec:
  selector:
    app: myapp
    version: blue
  ports:
  - port: 80
    targetPort: 8080
```

**Switch traffic from blue to green:**

```bash
kubectl patch service myapp-service -p '{"spec":{"selector":{"version":"green"}}}'
```

Or equivalently:

```bash
kubectl apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
spec:
  selector:
    app: myapp
    version: green
  ports:
  - port: 80
    targetPort: 8080
EOF
```

**Why it works:** The Service selector is the traffic switch. By changing `version: blue` to `version: green`, all traffic instantly routes to the green Pods. The blue Pods remain running, so rollback is a single command (change the selector back). Both Deployments run simultaneously, meaning you need double the resources during the deployment window.

**Common mistakes:**
- Using only `app: myapp` in the Service selector without `version`. Both Deployments would receive traffic simultaneously (which is canary, not blue-green).
- Not keeping the blue Deployment running after switching. The point of blue-green is fast rollback -- if you delete blue, you lose that capability.

---

## Part E: Compare Strategies

| Aspect | RollingUpdate | Canary | Blue-Green |
|--------|---------------|--------|------------|
| Downtime | None (with maxUnavailable: 0) | None | None (atomic switch) |
| Resource overhead during deployment | Low (only maxSurge extra Pods) | Low (only canary Pods, proportional to canary percentage) | High (two full deployments running simultaneously) |
| Rollback speed | Moderate (must scale down new RS and scale up old RS) | Fast (delete canary Deployment, stable is still running) | Instant (change Service selector back) |
| Traffic control granularity | All-or-nothing per Pod (gradual by Pod count) | Percentage-based (proportional to replica ratio) | All-or-nothing (100% switch at once) |
| Complexity | Low (built-in, single Deployment) | Medium (two Deployments, manual replica management) | Medium-High (two Deployments, Service selector management) |
| Use case | Most stateless applications | Testing new versions with a small percentage of real traffic before full rollout | Applications where you need instant rollback and can afford double resources; compliance scenarios requiring full validation before switching |

**Common mistakes:**
- Thinking Canary requires a service mesh. Basic canary (proportional to replica count) works with native Kubernetes Services. Service meshes add weighted routing but are not required.
- Confusing Blue-Green with Canary. Blue-Green switches 100% of traffic atomically; Canary sends a percentage gradually.
- Assuming RollingUpdate provides traffic-level control. It controls Pod replacement, not traffic routing. During a rolling update, traffic goes to both old and new Pods proportionally.
