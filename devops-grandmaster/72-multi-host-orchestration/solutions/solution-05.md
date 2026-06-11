# Solution 05: Orchestrator Migration Plan

## Part A: Kubernetes Manifest Translation

### Postgres (StatefulSet)

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: db-password
  namespace: shop
type: Opaque
data:
  password: <base64-encoded-password>
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: pg-data
  namespace: shop
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: standard
  resources:
    requests:
      storage: 10Gi
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: shop
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
        - name: postgres
          image: postgres:15
          ports:
            - containerPort: 5432
          env:
            - name: POSTGRES_DB
              value: shop
            - name: POSTGRES_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: db-password
                  key: password
          volumeMounts:
            - name: pg-data
              mountPath: /var/lib/postgresql/data
          resources:
            limits:
              cpu: "2"
              memory: "2Gi"
            requests:
              cpu: "500m"
              memory: "512Mi"
  volumeClaimTemplates:
    - metadata:
        name: pg-data
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 10Gi
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: shop
spec:
  selector:
    app: postgres
  ports:
    - port: 5432
      targetPort: 5432
  clusterIP: None  # Headless service for StatefulSet
```

### Redis (Deployment)

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: redis-data
  namespace: shop
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 2Gi
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: shop
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
        - name: redis
          image: redis:7-alpine
          ports:
            - containerPort: 6379
          volumeMounts:
            - name: redis-data
              mountPath: /data
          resources:
            limits:
              cpu: "500m"
              memory: "256Mi"
---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: shop
spec:
  selector:
    app: redis
  ports:
    - port: 6379
      targetPort: 6379
```

### Worker (Deployment)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: worker
  namespace: shop
spec:
  replicas: 6
  selector:
    matchLabels:
      app: worker
  template:
    metadata:
      labels:
        app: worker
    spec:
      containers:
        - name: worker
          image: myapp/worker:v3
          resources:
            limits:
              cpu: "1"
              memory: "512Mi"
            requests:
              cpu: "250m"
              memory: "128Mi"
```

### API (Deployment + Service + Ingress)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
  namespace: shop
spec:
  replicas: 4
  selector:
    matchLabels:
      app: api
  template:
    metadata:
      labels:
        app: api
    spec:
      containers:
        - name: api
          image: myapp/api:v3
          ports:
            - containerPort: 8080
          env:
            - name: DB_HOST
              value: postgres.shop.svc.cluster.local
            - name: DB_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: db-password
                  key: password
            - name: REDIS_HOST
              value: redis.shop.svc.cluster.local
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5
          resources:
            limits:
              cpu: "1"
              memory: "512Mi"
            requests:
              cpu: "250m"
              memory: "128Mi"
---
apiVersion: v1
kind: Service
metadata:
  name: api
  namespace: shop
spec:
  selector:
    app: api
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: api-ingress
  namespace: shop
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
    - host: api.shop.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: api
                port:
                  number: 80
```

### Why This Works

- **StatefulSet for Postgres** provides stable network identity (`postgres-0`) and persistent storage via `volumeClaimTemplates`. Each pod gets its own PVC, which survives pod restarts.
- **Secrets are base64-encoded** in the manifest. In production, use a sealed-secrets controller or external secrets operator for encryption at rest.
- **Headless Service** (`clusterIP: None`) for the StatefulSet allows direct pod DNS resolution (`postgres-0.postgres.shop.svc.cluster.local`).
- **Liveness and readiness probes** on the API replace Swarm's `healthcheck`. Liveness restarts unhealthy pods; readiness removes pods from service endpoints until they are ready.
- **Ingress** replaces Swarm's port mapping for external access. It provides host-based routing, TLS termination, and path-based routing.

### Common Mistakes to Avoid

- Using a Deployment for Postgres. Deployments do not guarantee stable network identity or ordered pod management.
- Storing secrets in plaintext in the manifest. Always use base64 encoding at minimum, and a secrets manager in production.
- Forgetting the namespace. All resources must be in the same namespace for service discovery to work.
- Using `hostPath` volumes for database storage. Use PersistentVolumeClaims for portable, cloud-managed storage.

---

## Part B: Migration Rollout Strategy

### Phase 1: Infrastructure Setup (Week 1-2)

**What changes:**
- Provision Kubernetes cluster (EKS/GKE/AKS or on-prem).
- Set up CNI networking (Calico, Cilium, or cloud-native).
- Deploy Ingress controller (nginx-ingress or cloud-native LB).
- Set up monitoring (Prometheus, Grafana) and logging (Loki, EFK).
- Configure RBAC, network policies, and secrets management.

**How to verify:**
- Deploy a simple test application and confirm external access.
- Verify DNS resolution between pods.
- Confirm monitoring dashboards show cluster metrics.

**Rollback:** Destroy the Kubernetes cluster. No production impact since nothing is running on it yet.

### Phase 2: Stateful Services Migration (Week 3-4)

**What changes:**
- Deploy Postgres StatefulSet with PersistentVolumeClaim.
- Migrate data from Swarm Postgres to Kubernetes Postgres using `pg_dump` / `pg_restore` or logical replication.
- Deploy Redis with persistent storage.
- Verify data integrity.

**How to verify:**
- Run data integrity checks (`SELECT count(*)` on critical tables).
- Compare checksums between old and new databases.
- Test read/write operations from a test application.

**Rollback:** Point applications back to Swarm Postgres. The Swarm instance is still running. Data written to K8s Postgres during testing can be re-migrated.

### Phase 3: Stateless Services Migration (Week 5-6)

**What changes:**
- Deploy `worker` Deployment (6 replicas).
- Deploy `api` Deployment (4 replicas) with Ingress.
- Configure both to connect to K8s Postgres and Redis.
- Run both Swarm and Kubernetes deployments in parallel behind a load balancer.

**How to verify:**
- Route 10% of traffic to Kubernetes API, 90% to Swarm.
- Monitor error rates, latency, and throughput.
- Gradually increase to 50%, then 100%.

**Rollback:** Route all traffic back to Swarm. Scale down K8s deployments.

### Phase 4: Cutover and Decommission (Week 7-8)

**What changes:**
- Update DNS to point to Kubernetes Ingress.
- Remove Swarm stack.
- Decommission Swarm cluster.
- Update monitoring and alerting for K8s-native metrics.

**How to verify:**
- Confirm all traffic flows through Kubernetes.
- Verify no errors in application logs.
- Run load tests to confirm performance parity.

**Rollback:** If critical issues arise, re-deploy Swarm stack from the existing compose file and revert DNS. This is why we keep the Swarm configuration in version control.

### Common Mistakes to Avoid

- Migrating everything at once. Phased migration limits blast radius.
- Not running both systems in parallel. You need a fallback path.
- Decommissioning Swarm before confirming Kubernetes stability for at least 1-2 weeks.
- Not testing the rollback procedure. An untested rollback is not a rollback plan.

---

## Part C: Service Discovery Mapping

### 1. API finds Postgres in Swarm

In Swarm, the `api` service discovers `postgres` via built-in DNS on the overlay network. The service name `postgres` resolves to a Virtual IP (VIP) that load-balances across all replicas.

```
api container -> DNS lookup "postgres" -> VIP 10.0.x.x -> routes to postgres container
```

The connection string is simply `postgres:5432`. No configuration needed beyond being on the same overlay network.

### 2. API finds Postgres in Kubernetes

In Kubernetes, the `api` pod discovers `postgres` via CoreDNS. The fully qualified domain name is `postgres.shop.svc.cluster.local`, which resolves to the ClusterIP of the postgres Service.

```
api pod -> DNS lookup "postgres.shop.svc.cluster.local" -> ClusterIP 10.96.x.x -> routes to postgres pod
```

For a StatefulSet with a headless service, you can also use `postgres-0.postgres.shop.svc.cluster.local` for direct pod access.

### 3. DNS Name Changes

| Context | Swarm DNS Name | Kubernetes DNS Name |
|---------|---------------|-------------------|
| Same network, same stack | `postgres` | `postgres` (short) or `postgres.shop.svc.cluster.local` (FQDN) |
| Cross-namespace | Not applicable (Swarm has no namespaces) | `postgres.<namespace>.svc.cluster.local` |
| Cross-cluster | Not supported natively | Requires service mesh (Istio, Linkerd) |

### 4. Health Check Differences

| Aspect | Docker Swarm | Kubernetes |
|--------|-------------|------------|
| Mechanism | `HEALTHCHECK` instruction or `healthcheck` in compose | Liveness, readiness, and startup probes |
| Effect of failure | Swarm restarts the container | Liveness probe: restarts pod. Readiness probe: removes from Service endpoints. |
| Probe types | `CMD`, `HTTP`, `TCP` | `httpGet`, `tcpSocket`, `exec`, `grpc` |
| Configuration | `interval`, `timeout`, `retries`, `start_period` | `initialDelaySeconds`, `periodSeconds`, `timeoutSeconds`, `failureThreshold` |
| Granularity | Single health check per container | Separate liveness (restart) and readiness (traffic) probes |

### Common Mistakes to Avoid

- Hardcoding DNS names from Swarm in Kubernetes manifests. Use Kubernetes Service names.
- Forgetting that Kubernetes has namespaces. The FQDN includes the namespace.
- Using only liveness probes. Readiness probes are critical for zero-downtime deployments (they prevent traffic to pods that are starting up or shutting down).

---

## Key Takeaway

Migrating between orchestrators requires mapping concepts, not just translating syntax. Swarm's overlay networks become Kubernetes Services and CNI. Swarm's health checks become liveness/readiness probes. Swarm's secrets become Kubernetes Secrets (with proper encryption). The migration must be phased, with the database first (data gravity), stateless services second, and DNS cutover last. Every phase must have a tested rollback path.
